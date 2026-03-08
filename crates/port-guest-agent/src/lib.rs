use std::fs;
use std::io::{BufReader, BufWriter, Read};
use std::net::{Shutdown, TcpStream};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Component, Path, PathBuf};
use std::thread;

use anyhow::{Context, Result, anyhow, bail};
use port_agent_protocol::{
    CopyRequest, ExecRequest, ExecResult, ForwardEndpoint, ForwardRequest, GuestOperation,
    LogsRequest, LogsResult, OperationResult, PtyRequest, PtyResult, RequestEnvelope,
    ResponseEnvelope, StreamKind, parse_forward_endpoint, read_frame, write_frame,
};
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use vsock::{VMADDR_CID_ANY, VsockListener, VsockStream};

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
        match self.handle_non_streaming_operation(request.operation) {
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

    fn handle_non_streaming_operation(
        &self,
        operation: GuestOperation,
    ) -> Result<(i32, OperationResult)> {
        match operation {
            GuestOperation::Exec(request) => self.exec(request),
            GuestOperation::Pty(request) => self.pty(request),
            GuestOperation::Logs(request) => self.logs(request),
            GuestOperation::Copy(_) | GuestOperation::Forward(_) => {
                bail!("operation requires a streaming guest-agent connection")
            }
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

trait AgentStream: Read + std::io::Write + Send + 'static {
    fn try_clone_stream(&self) -> std::io::Result<Self>
    where
        Self: Sized;

    fn shutdown_write(&self) -> std::io::Result<()>;
}

enum ForwardTargetStream {
    Tcp(TcpStream),
    Unix(UnixStream),
}

impl ForwardTargetStream {
    fn try_clone_stream(&self) -> std::io::Result<Self> {
        match self {
            Self::Tcp(stream) => stream.try_clone().map(Self::Tcp),
            Self::Unix(stream) => stream.try_clone().map(Self::Unix),
        }
    }

    fn shutdown_write(&self) -> std::io::Result<()> {
        match self {
            Self::Tcp(stream) => stream.shutdown(Shutdown::Write),
            Self::Unix(stream) => stream.shutdown(Shutdown::Write),
        }
    }
}

impl Read for ForwardTargetStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Self::Tcp(stream) => stream.read(buf),
            Self::Unix(stream) => stream.read(buf),
        }
    }
}

impl std::io::Write for ForwardTargetStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Self::Tcp(stream) => stream.write(buf),
            Self::Unix(stream) => stream.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Self::Tcp(stream) => stream.flush(),
            Self::Unix(stream) => stream.flush(),
        }
    }
}

impl AgentStream for std::os::unix::net::UnixStream {
    fn try_clone_stream(&self) -> std::io::Result<Self> {
        self.try_clone()
    }

    fn shutdown_write(&self) -> std::io::Result<()> {
        self.shutdown(Shutdown::Write)
    }
}

impl AgentStream for VsockStream {
    fn try_clone_stream(&self) -> std::io::Result<Self> {
        self.try_clone()
    }

    fn shutdown_write(&self) -> std::io::Result<()> {
        self.shutdown(Shutdown::Write)
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
        let service = service.clone();
        thread::spawn(move || {
            if let Err(error) = handle_protocol_stream(stream, &service) {
                eprintln!("port-guest-agent Unix transport connection failed: {error}");
            }
        });
    }

    Ok(())
}

fn serve_vsock_listener(listener: VsockListener, service: &AgentService) -> Result<()> {
    for stream in listener.incoming() {
        let stream = stream.context("failed to accept guest-agent vsock connection")?;
        let service = service.clone();
        thread::spawn(move || {
            if let Err(error) = handle_protocol_stream(stream, &service) {
                eprintln!("port-guest-agent vsock transport connection failed: {error}");
            }
        });
    }

    Ok(())
}

fn handle_protocol_stream<S>(stream: S, service: &AgentService) -> Result<()>
where
    S: AgentStream,
{
    let reader_stream = stream
        .try_clone_stream()
        .context("failed to clone guest-agent stream")?;
    let mut reader = BufReader::new(reader_stream);
    let request: RequestEnvelope =
        read_frame(&mut reader).map_err(|error| anyhow!("protocol error: {error}"))?;
    match request.operation {
        GuestOperation::Copy(copy) => service.copy_stream(request.id, copy, &mut reader, stream),
        GuestOperation::Forward(forward) => service.forward_stream(request.id, forward, stream),
        operation => {
            let response = service.handle(RequestEnvelope {
                id: request.id,
                operation,
            });
            let mut writer = BufWriter::new(stream);
            write_frame(&mut writer, &response).map_err(|error| anyhow!("protocol error: {error}"))
        }
    }
}

impl AgentService {
    fn copy_stream<R, W>(
        &self,
        id: u64,
        request: CopyRequest,
        reader: &mut BufReader<R>,
        writer_stream: W,
    ) -> Result<()>
    where
        R: Read,
        W: std::io::Write,
    {
        let mut writer = BufWriter::new(writer_stream);
        match request.direction {
            port_agent_protocol::CopyDirection::HostToGuest => {
                let destination = match self.resolve_guest_path(&request.destination) {
                    Ok(path) => path,
                    Err(error) => return write_failed_response(&mut writer, id, error),
                };
                if let Some(parent) = destination.parent() {
                    if let Err(error) = fs::create_dir_all(parent)
                        .with_context(|| format!("failed to create '{}'", parent.display()))
                    {
                        return write_failed_response(&mut writer, id, error);
                    }
                }
                let Some(size_bytes) = request.size_bytes else {
                    return write_failed_response(
                        &mut writer,
                        id,
                        anyhow!("host-to-guest copy requires size_bytes"),
                    );
                };
                let mut destination_file = match fs::File::create(&destination)
                    .with_context(|| format!("failed to create '{}'", destination.display()))
                {
                    Ok(file) => file,
                    Err(error) => return write_failed_response(&mut writer, id, error),
                };
                write_frame(
                    &mut writer,
                    &ResponseEnvelope::Accepted {
                        id,
                        stream: StreamKind::Bytes,
                        size_bytes: None,
                    },
                )
                .map_err(|error| anyhow!("protocol error: {error}"))?;
                let mut limited = reader.take(size_bytes);
                let bytes_copied = std::io::copy(&mut limited, &mut destination_file)
                    .with_context(|| format!("failed to write '{}'", destination.display()))?;
                if bytes_copied != size_bytes {
                    bail!(
                        "expected {size_bytes} bytes for host-to-guest copy, received {bytes_copied}"
                    );
                }

                write_frame(
                    &mut writer,
                    &ResponseEnvelope::Completed {
                        id,
                        exit_code: 0,
                        result: OperationResult::Copy(port_agent_protocol::CopyResult {
                            bytes_copied,
                            path: request.destination,
                            direction: request.direction,
                        }),
                    },
                )
                .map_err(|error| anyhow!("protocol error: {error}"))?;
            }
            port_agent_protocol::CopyDirection::GuestToHost => {
                let source = match self.resolve_guest_path(&request.source) {
                    Ok(path) => path,
                    Err(error) => return write_failed_response(&mut writer, id, error),
                };
                let mut source_file = match fs::File::open(&source)
                    .with_context(|| format!("failed to open '{}'", source.display()))
                {
                    Ok(file) => file,
                    Err(error) => return write_failed_response(&mut writer, id, error),
                };
                let size_bytes = match source_file
                    .metadata()
                    .with_context(|| format!("failed to stat '{}'", source.display()))
                {
                    Ok(metadata) => metadata.len(),
                    Err(error) => return write_failed_response(&mut writer, id, error),
                };
                write_frame(
                    &mut writer,
                    &ResponseEnvelope::Accepted {
                        id,
                        stream: StreamKind::Bytes,
                        size_bytes: Some(size_bytes),
                    },
                )
                .map_err(|error| anyhow!("protocol error: {error}"))?;
                let bytes_copied = std::io::copy(&mut source_file, &mut writer)
                    .with_context(|| format!("failed to stream '{}'", source.display()))?;
                if bytes_copied != size_bytes {
                    bail!(
                        "expected to stream {size_bytes} bytes for guest-to-host copy, wrote {bytes_copied}"
                    );
                }
                write_frame(
                    &mut writer,
                    &ResponseEnvelope::Completed {
                        id,
                        exit_code: 0,
                        result: OperationResult::Copy(port_agent_protocol::CopyResult {
                            bytes_copied,
                            path: request.destination,
                            direction: request.direction,
                        }),
                    },
                )
                .map_err(|error| anyhow!("protocol error: {error}"))?;
            }
        }

        Ok(())
    }

    fn forward_stream<S>(&self, id: u64, request: ForwardRequest, stream: S) -> Result<()>
    where
        S: AgentStream,
    {
        let mut control_writer = BufWriter::new(
            stream
                .try_clone_stream()
                .context("failed to clone guest transport stream for forward ack")?,
        );
        let mut outbound = match connect_forward_target(&request.target)
            .with_context(|| format!("failed to connect to '{}'", request.target))
        {
            Ok(stream) => stream,
            Err(error) => return write_failed_response(&mut control_writer, id, error),
        };
        write_frame(
            &mut control_writer,
            &ResponseEnvelope::Accepted {
                id,
                stream: StreamKind::Bytes,
                size_bytes: None,
            },
        )
        .map_err(|error| anyhow!("protocol error: {error}"))?;
        drop(control_writer);

        let mut inbound_read = stream
            .try_clone_stream()
            .context("failed to clone guest transport stream for forward read")?;
        let mut inbound_write = stream;
        let mut outbound_read = outbound
            .try_clone_stream()
            .context("failed to clone target stream")?;

        let first = thread::spawn(move || {
            let result = std::io::copy(&mut inbound_read, &mut outbound);
            let _ = outbound.shutdown_write();
            result
        });
        let second = thread::spawn(move || {
            let result = std::io::copy(&mut outbound_read, &mut inbound_write);
            let _ = inbound_write.shutdown_write();
            result
        });

        let _ = first.join();
        let _ = second.join();
        Ok(())
    }
}

fn write_failed_response<W>(writer: &mut BufWriter<W>, id: u64, error: anyhow::Error) -> Result<()>
where
    W: std::io::Write,
{
    write_frame(
        writer,
        &ResponseEnvelope::Failed {
            id,
            message: error.to_string(),
        },
    )
    .map_err(|frame_error| anyhow!("protocol error: {frame_error}"))
}

fn connect_forward_target(target: &str) -> Result<ForwardTargetStream> {
    match parse_forward_endpoint(target).map_err(|error| anyhow!(error.to_string()))? {
        ForwardEndpoint::Tcp(address) => TcpStream::connect(&address)
            .map(ForwardTargetStream::Tcp)
            .with_context(|| format!("failed to connect to TCP target '{address}'")),
        ForwardEndpoint::Unix(path) => UnixStream::connect(&path)
            .map(ForwardTargetStream::Unix)
            .with_context(|| format!("failed to connect to Unix target '{}'", path.display())),
    }
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
    use std::net::{Shutdown, TcpListener};
    use std::os::unix::net::UnixStream;
    use std::thread;
    use std::time::Duration;

    use port_agent_protocol::{
        CopyDirection, CopyRequest, ExecRequest, ForwardRequest, GuestOperation, LogsRequest,
        PtyRequest, RequestEnvelope, ResponseEnvelope, StreamKind, read_frame, write_frame,
    };
    use tempfile::tempdir;

    use super::{AgentService, handle_protocol_stream, serve_with_vsock};

    #[test]
    fn service_handles_exec_pty_and_logs() {
        let temp = tempdir().expect("tempdir should exist");
        let guest_root = temp.path().join("guest");
        fs::create_dir_all(guest_root.join("var/log")).expect("guest root should exist");
        fs::write(guest_root.join("var/log/app.log"), "line-1\nline-2\n").expect("log file");
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

        let pty = service.handle(RequestEnvelope {
            id: 2,
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
            id: 3,
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
    }

    #[test]
    fn connection_streams_copy_and_forward() {
        let temp = tempdir().expect("tempdir should exist");
        let guest_root = temp.path().join("guest");
        fs::create_dir_all(guest_root.join("workspace")).expect("guest root should exist");
        let host_source = temp.path().join("host.txt");
        fs::write(&host_source, "copy-ok").expect("host file");
        let service = AgentService::new(guest_root.clone());

        let (mut client, server) = UnixStream::pair().expect("stream pair");
        let service_for_upload = service.clone();
        let upload_thread = thread::spawn(move || {
            handle_protocol_stream(server, &service_for_upload).expect("upload should succeed")
        });
        let upload_reader_stream = client.try_clone().expect("upload stream should clone");
        let mut upload_reader = BufReader::new(upload_reader_stream);
        write_frame(
            &mut client,
            &RequestEnvelope {
                id: 4,
                operation: GuestOperation::Copy(CopyRequest {
                    source: host_source.display().to_string(),
                    destination: String::from("/workspace/copied.txt"),
                    direction: CopyDirection::HostToGuest,
                    size_bytes: Some(7),
                }),
            },
        )
        .expect("upload request should write");
        let upload_response: ResponseEnvelope = read_frame(&mut upload_reader).expect("upload ack");
        assert!(matches!(
            upload_response,
            ResponseEnvelope::Accepted {
                stream: StreamKind::Bytes,
                ..
            }
        ));
        client
            .write_all(b"copy-ok")
            .expect("upload bytes should write");
        client.flush().expect("upload bytes should flush");
        let upload_complete: ResponseEnvelope =
            read_frame(&mut upload_reader).expect("upload completion");
        assert!(matches!(
            upload_complete,
            ResponseEnvelope::Completed { .. }
        ));
        assert_eq!(
            fs::read_to_string(guest_root.join("workspace/copied.txt")).expect("copied file"),
            "copy-ok"
        );
        upload_thread.join().expect("upload thread should finish");

        let (mut client, server) = UnixStream::pair().expect("stream pair");
        let service_for_download = service.clone();
        let download_thread = thread::spawn(move || {
            handle_protocol_stream(server, &service_for_download).expect("download should succeed")
        });
        let download_reader_stream = client.try_clone().expect("download stream should clone");
        let mut download_reader = BufReader::new(download_reader_stream);
        write_frame(
            &mut client,
            &RequestEnvelope {
                id: 5,
                operation: GuestOperation::Copy(CopyRequest {
                    source: String::from("/workspace/copied.txt"),
                    destination: temp.path().join("downloaded.txt").display().to_string(),
                    direction: CopyDirection::GuestToHost,
                    size_bytes: None,
                }),
            },
        )
        .expect("download request should write");
        let download_response: ResponseEnvelope =
            read_frame(&mut download_reader).expect("download ack");
        let ResponseEnvelope::Accepted {
            size_bytes: Some(size_bytes),
            ..
        } = download_response
        else {
            panic!("unexpected download response: {download_response:?}");
        };
        let mut bytes = Vec::new();
        download_reader
            .by_ref()
            .take(size_bytes)
            .read_to_end(&mut bytes)
            .expect("download bytes should read");
        assert_eq!(bytes, b"copy-ok");
        let download_complete: ResponseEnvelope =
            read_frame(&mut download_reader).expect("download completion");
        assert!(matches!(
            download_complete,
            ResponseEnvelope::Completed { .. }
        ));
        download_thread
            .join()
            .expect("download thread should finish");

        let target = TcpListener::bind("127.0.0.1:0").expect("target listener");
        let target_addr = target.local_addr().expect("target addr");
        thread::spawn(move || {
            let (mut stream, _) = target.accept().expect("accept target");
            let mut buf = [0_u8; 32];
            let len = stream.read(&mut buf).expect("read target");
            stream.write_all(&buf[..len]).expect("write target");
        });
        let (mut client, server) = UnixStream::pair().expect("stream pair");
        let service_for_forward = service.clone();
        let forward_thread = thread::spawn(move || {
            handle_protocol_stream(server, &service_for_forward).expect("forward should succeed")
        });
        let forward_reader_stream = client.try_clone().expect("forward stream should clone");
        let mut forward_reader = BufReader::new(forward_reader_stream);
        write_frame(
            &mut client,
            &RequestEnvelope {
                id: 6,
                operation: GuestOperation::Forward(ForwardRequest {
                    listen: String::new(),
                    target: target_addr.to_string(),
                }),
            },
        )
        .expect("forward request should write");
        let forward_response: ResponseEnvelope =
            read_frame(&mut forward_reader).expect("forward ack");
        assert!(matches!(
            forward_response,
            ResponseEnvelope::Accepted {
                stream: StreamKind::Bytes,
                ..
            }
        ));
        client.write_all(b"forward-ok").expect("write forwarded");
        client
            .shutdown(Shutdown::Write)
            .expect("shutdown forward write");
        let mut buf = [0_u8; 32];
        let len = forward_reader.read(&mut buf).expect("read forwarded");
        assert_eq!(&buf[..len], b"forward-ok");
        forward_thread.join().expect("forward thread should finish");
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
