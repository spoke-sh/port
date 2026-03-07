use std::fs;
use std::io::{BufReader, BufWriter, Read};
use std::net::{TcpListener, TcpStream};
use std::os::unix::net::UnixListener;
use std::path::{Component, Path, PathBuf};
use std::thread;

use anyhow::{Context, Result, anyhow, bail};
use port_agent_protocol::{
    CopyRequest, ExecRequest, ExecResult, ForwardRequest, ForwardResult, GuestOperation,
    LogsRequest, LogsResult, OperationResult, PtyRequest, PtyResult, RequestEnvelope,
    ResponseEnvelope, read_frame, write_frame,
};
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use vsock::{VMADDR_CID_ANY, VsockListener};

#[derive(Debug, Clone)]
pub struct AgentService {
    root: PathBuf,
}

impl AgentService {
    #[must_use]
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn handle(&self, request: RequestEnvelope) -> ResponseEnvelope {
        let id = request.id;
        match self.handle_operation(request.operation) {
            Ok((exit_code, result)) => ResponseEnvelope::Completed {
                id,
                exit_code,
                result,
            },
            Err(error) => ResponseEnvelope::Failed {
                id,
                message: error.to_string(),
            },
        }
    }

    fn handle_operation(&self, operation: GuestOperation) -> Result<(i32, OperationResult)> {
        match operation {
            GuestOperation::Exec(request) => self.exec(request),
            GuestOperation::Copy(request) => self.copy(request),
            GuestOperation::Pty(request) => self.pty(request),
            GuestOperation::Logs(request) => self.logs(request),
            GuestOperation::Forward(request) => self.forward(request),
        }
    }

    fn exec(&self, request: ExecRequest) -> Result<(i32, OperationResult)> {
        let (program, args) = request
            .command
            .split_first()
            .ok_or_else(|| anyhow!("exec request requires a command"))?;

        let cwd = request
            .cwd
            .as_deref()
            .map(|path| self.resolve_guest_path(path))
            .transpose()?
            .unwrap_or_else(|| self.root.clone());

        let output = std::process::Command::new(program)
            .args(args)
            .current_dir(cwd)
            .envs(request.env)
            .output()
            .with_context(|| format!("failed to spawn '{}'", program))?;

        let exit_code = output.status.code().unwrap_or(1);
        let result = OperationResult::Exec(ExecResult {
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        });

        Ok((exit_code, result))
    }

    fn copy(&self, request: CopyRequest) -> Result<(i32, OperationResult)> {
        let (source, destination, result_path) = match request.direction {
            port_agent_protocol::CopyDirection::HostToGuest => {
                let destination = self.resolve_guest_path(&request.destination)?;
                (
                    PathBuf::from(&request.source),
                    destination,
                    request.destination,
                )
            }
            port_agent_protocol::CopyDirection::GuestToHost => {
                let source = self.resolve_guest_path(&request.source)?;
                (
                    source,
                    PathBuf::from(&request.destination),
                    request.destination,
                )
            }
        };

        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create '{}'", parent.display()))?;
        }

        let bytes_copied = fs::copy(&source, &destination).with_context(|| {
            format!(
                "failed to copy '{}' -> '{}'",
                source.display(),
                destination.display()
            )
        })?;

        let result = OperationResult::Copy(port_agent_protocol::CopyResult {
            bytes_copied,
            path: result_path,
            direction: request.direction,
        });

        Ok((0, result))
    }

    fn pty(&self, request: PtyRequest) -> Result<(i32, OperationResult)> {
        let (program, args) = request
            .command
            .split_first()
            .ok_or_else(|| anyhow!("pty request requires a command"))?;
        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows: request.rows,
                cols: request.cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .context("failed to allocate PTY")?;
        let mut builder = CommandBuilder::new(program);
        builder.args(args);
        builder.cwd(&self.root);
        let mut child = pair
            .slave
            .spawn_command(builder)
            .context("failed to spawn PTY command")?;
        drop(pair.slave);

        let mut reader = pair
            .master
            .try_clone_reader()
            .context("failed to clone PTY reader")?;
        let status = child.wait().context("failed to wait for PTY child")?;
        let mut transcript = String::new();
        reader
            .read_to_string(&mut transcript)
            .context("failed to read PTY transcript")?;

        let exit_code =
            i32::try_from(status.exit_code()).context("PTY child exit code overflowed i32")?;
        let result = OperationResult::Pty(PtyResult { transcript });
        Ok((exit_code, result))
    }

    fn logs(&self, request: LogsRequest) -> Result<(i32, OperationResult)> {
        let path = self.resolve_guest_path(&request.path)?;
        let contents = fs::read_to_string(&path)
            .with_context(|| format!("failed to read log '{}'", path.display()))?;
        let contents = if let Some(tail_lines) = request.tail_lines {
            tail(&contents, tail_lines as usize)
        } else {
            contents
        };

        let result = OperationResult::Logs(LogsResult { contents });
        Ok((0, result))
    }

    fn forward(&self, request: ForwardRequest) -> Result<(i32, OperationResult)> {
        let listener = TcpListener::bind(&request.listen)
            .with_context(|| format!("failed to bind '{}'", request.listen))?;
        let listen = listener
            .local_addr()
            .context("failed to resolve local listener address")?
            .to_string();
        let target = request.target.clone();

        thread::spawn(move || {
            for inbound in listener.incoming() {
                let Ok(mut inbound) = inbound else { break };
                let target = target.clone();
                thread::spawn(move || {
                    let Ok(mut outbound) = TcpStream::connect(&target) else {
                        return;
                    };
                    let Ok(mut inbound_read) = inbound.try_clone() else {
                        return;
                    };
                    let Ok(mut outbound_read) = outbound.try_clone() else {
                        return;
                    };
                    let first =
                        thread::spawn(move || std::io::copy(&mut inbound_read, &mut outbound));
                    let second =
                        thread::spawn(move || std::io::copy(&mut outbound_read, &mut inbound));
                    let _ = first.join();
                    let _ = second.join();
                });
            }
        });

        let result = OperationResult::Forward(ForwardResult {
            listen,
            target: request.target,
        });
        Ok((0, result))
    }

    fn resolve_guest_path(&self, input: impl AsRef<Path>) -> Result<PathBuf> {
        let input = input.as_ref();
        let mut relative = PathBuf::new();
        for component in input.components() {
            match component {
                Component::RootDir | Component::CurDir => {}
                Component::ParentDir => bail!(
                    "parent path segments are not allowed: '{}'",
                    input.display()
                ),
                Component::Normal(part) => relative.push(part),
                Component::Prefix(_) => {
                    bail!("path prefixes are not supported: '{}'", input.display())
                }
            }
        }

        Ok(self.root.join(relative))
    }
}

pub fn serve(socket_path: &Path, root: PathBuf) -> Result<()> {
    serve_with_vsock(socket_path, root, None)
}

pub fn serve_with_vsock(socket_path: &Path, root: PathBuf, vsock_port: Option<u32>) -> Result<()> {
    let service = AgentService::new(root);
    let unix_listener = bind_unix_listener(socket_path)?;
    let vsock_listener = vsock_port
        .map(|port| {
            VsockListener::bind_with_cid_port(VMADDR_CID_ANY, port).with_context(|| {
                format!("failed to bind guest-agent vsock listener on port {port}")
            })
        })
        .transpose()?;

    if let Some(vsock_listener) = vsock_listener {
        let service = service.clone();
        thread::spawn(move || {
            if let Err(error) = serve_vsock_listener(vsock_listener, &service) {
                eprintln!("port-guest-agent vsock listener exited: {error}");
            }
        });
    }

    serve_unix_listener(unix_listener, &service)
}

fn bind_unix_listener(socket_path: &Path) -> Result<UnixListener> {
    if socket_path.exists() {
        fs::remove_file(socket_path)
            .with_context(|| format!("failed to remove '{}'", socket_path.display()))?;
    }
    if let Some(parent) = socket_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create '{}'", parent.display()))?;
    }

    UnixListener::bind(socket_path)
        .with_context(|| format!("failed to bind '{}'", socket_path.display()))
}

fn serve_unix_listener(listener: UnixListener, service: &AgentService) -> Result<()> {
    for stream in listener.incoming() {
        let stream = stream.context("failed to accept guest-agent connection")?;
        let reader = stream.try_clone().context("failed to clone UnixStream")?;
        handle_protocol_stream(reader, stream, service)?;
    }

    Ok(())
}

fn serve_vsock_listener(listener: VsockListener, service: &AgentService) -> Result<()> {
    for stream in listener.incoming() {
        let stream = stream.context("failed to accept guest-agent vsock connection")?;
        let reader = stream.try_clone().context("failed to clone VsockStream")?;
        handle_protocol_stream(reader, stream, service)?;
    }

    Ok(())
}

fn handle_protocol_stream<R, W>(reader_stream: R, writer_stream: W, service: &AgentService) -> Result<()>
where
    R: std::io::Read,
    W: std::io::Write,
{
    let mut reader = BufReader::new(reader_stream);
    let mut writer = BufWriter::new(writer_stream);
    let request: RequestEnvelope =
        read_frame(&mut reader).map_err(|error| anyhow!("protocol error: {error}"))?;
    let response = service.handle(request);
    write_frame(&mut writer, &response).map_err(|error| anyhow!("protocol error: {error}"))?;
    Ok(())
}

fn tail(contents: &str, lines: usize) -> String {
    let mut collected = contents.lines().map(str::to_owned).collect::<Vec<_>>();
    if collected.len() > lines {
        collected = collected.split_off(collected.len() - lines);
    }
    let mut result = collected.join("\n");
    if contents.ends_with('\n') && !result.is_empty() {
        result.push('\n');
    }
    result
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io::{BufReader, Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::os::unix::net::UnixStream;
    use std::thread;
    use std::time::Duration;

    use port_agent_protocol::{
        CopyDirection, CopyRequest, ExecRequest, ForwardRequest, GuestOperation, LogsRequest,
        PtyRequest, RequestEnvelope, ResponseEnvelope,
    };
    use tempfile::tempdir;

    use super::{AgentService, serve_with_vsock};

    #[test]
    fn service_handles_exec_copy_pty_logs_and_forward() {
        let temp = tempdir().expect("tempdir should exist");
        let guest_root = temp.path().join("guest");
        fs::create_dir_all(guest_root.join("var/log")).expect("guest root should exist");
        fs::write(guest_root.join("var/log/app.log"), "line-1\nline-2\n").expect("log file");
        let host_source = temp.path().join("host.txt");
        fs::write(&host_source, "copy-ok").expect("host file");
        let service = AgentService::new(guest_root.clone());

        let exec = service.handle(RequestEnvelope {
            id: 1,
            operation: GuestOperation::Exec(ExecRequest {
                command: vec![
                    String::from("/bin/sh"),
                    String::from("-lc"),
                    String::from("printf exec-ok"),
                ],
                cwd: None,
                env: Default::default(),
            }),
        });
        assert!(matches!(exec, ResponseEnvelope::Completed { .. }));

        let copy = service.handle(RequestEnvelope {
            id: 2,
            operation: GuestOperation::Copy(CopyRequest {
                source: host_source.display().to_string(),
                destination: String::from("/workspace/copied.txt"),
                direction: CopyDirection::HostToGuest,
            }),
        });
        assert!(matches!(copy, ResponseEnvelope::Completed { .. }));
        assert_eq!(
            fs::read_to_string(guest_root.join("workspace/copied.txt")).expect("copied file"),
            "copy-ok"
        );

        let pty = service.handle(RequestEnvelope {
            id: 3,
            operation: GuestOperation::Pty(PtyRequest {
                command: vec![
                    String::from("/bin/sh"),
                    String::from("-lc"),
                    String::from("printf pty-ok"),
                ],
                cols: 80,
                rows: 24,
            }),
        });
        let pty_text = match pty {
            ResponseEnvelope::Completed { result, .. } => format!("{result:?}"),
            other => panic!("unexpected PTY response: {other:?}"),
        };
        assert!(pty_text.contains("pty-ok"));

        let logs = service.handle(RequestEnvelope {
            id: 4,
            operation: GuestOperation::Logs(LogsRequest {
                path: String::from("/var/log/app.log"),
                follow: false,
                tail_lines: Some(1),
            }),
        });
        let logs_text = match logs {
            ResponseEnvelope::Completed { result, .. } => format!("{result:?}"),
            other => panic!("unexpected logs response: {other:?}"),
        };
        assert!(logs_text.contains("line-2"));

        let target = TcpListener::bind("127.0.0.1:0").expect("target listener");
        let target_addr = target.local_addr().expect("target addr");
        thread::spawn(move || {
            let (mut stream, _) = target.accept().expect("accept target");
            let mut buf = [0_u8; 32];
            let len = stream.read(&mut buf).expect("read target");
            stream.write_all(&buf[..len]).expect("write target");
        });
        let forward = service.handle(RequestEnvelope {
            id: 5,
            operation: GuestOperation::Forward(ForwardRequest {
                listen: String::from("127.0.0.1:0"),
                target: target_addr.to_string(),
            }),
        });
        let listen_addr = match forward {
            ResponseEnvelope::Completed { result, .. } => match result {
                port_agent_protocol::OperationResult::Forward(forward) => forward.listen,
                other => panic!("unexpected forward result: {other:?}"),
            },
            other => panic!("unexpected forward response: {other:?}"),
        };
        thread::sleep(Duration::from_millis(50));
        let mut client = TcpStream::connect(&listen_addr).expect("connect forwarded listener");
        client.write_all(b"forward-ok").expect("write forwarded");
        let mut buf = [0_u8; 32];
        let len = client.read(&mut buf).expect("read forwarded");
        assert_eq!(&buf[..len], b"forward-ok");
    }

    #[test]
    fn daemon_serves_requests_over_unix_socket() {
        let temp = tempdir().expect("tempdir should exist");
        let guest_root = temp.path().join("guest");
        fs::create_dir_all(&guest_root).expect("guest root");
        let socket_path = temp.path().join("agent.sock");
        let socket_for_thread = socket_path.clone();
        let root = guest_root.clone();
        thread::spawn(move || {
            serve_with_vsock(&socket_for_thread, root, None).expect("server should run")
        });

        for _ in 0..50 {
            if socket_path.exists() {
                break;
            }
            thread::sleep(Duration::from_millis(20));
        }

        let mut stream = UnixStream::connect(&socket_path).expect("connect socket");
        port_agent_protocol::write_frame(
            &mut stream,
            &RequestEnvelope {
                id: 9,
                operation: GuestOperation::Exec(ExecRequest {
                    command: vec![
                        String::from("/bin/sh"),
                        String::from("-lc"),
                        String::from("printf daemon-ok"),
                    ],
                    cwd: None,
                    env: Default::default(),
                }),
            },
        )
        .expect("write request");
        let mut reader = BufReader::new(stream);
        let response: ResponseEnvelope =
            port_agent_protocol::read_frame(&mut reader).expect("read response");
        let text = format!("{response:?}");
        assert!(text.contains("daemon-ok"));
    }
}
