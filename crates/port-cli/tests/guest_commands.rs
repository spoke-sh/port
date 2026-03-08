use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

use port_model::PortConfig;
use tempfile::tempdir;

fn write_config(path: &Path, config: &PortConfig) {
    fs::write(path, config.to_toml_string().expect("config should encode"))
        .expect("config should write");
}

fn port_bin() -> &'static str {
    env!("CARGO_BIN_EXE_port")
}

fn spawn_agent(socket: &Path, root: &Path) {
    if let Some(parent) = socket.parent() {
        fs::create_dir_all(parent).expect("runtime dir should exist");
    }
    let socket = socket.to_path_buf();
    let serve_socket = socket.clone();
    let root = root.to_path_buf();
    thread::spawn(move || {
        port_guest_agent::serve(&serve_socket, root).expect("agent should serve");
    });

    for _ in 0..100 {
        if socket.exists() {
            return;
        }
        thread::sleep(Duration::from_millis(20));
    }

    panic!(
        "guest agent socket did not appear at '{}'",
        socket.display()
    );
}

fn runtime_socket(runtime_root: &Path, machine: &str) -> PathBuf {
    runtime_root.join(machine).join("guest-agent.sock")
}

fn add_runtime_root(command: &mut Command, runtime_root: Option<&Path>) {
    if let Some(runtime_root) = runtime_root {
        command.arg("--runtime-root").arg(runtime_root);
    }
}

fn run_guest_capability_suite(
    config_path: &Path,
    machine: &str,
    runtime_root: Option<&Path>,
    guest_root: &Path,
    temp_root: &Path,
) {
    let mut exec = Command::new(port_bin());
    exec.arg("--config")
        .arg(config_path)
        .arg("guest")
        .arg("exec")
        .arg("--machine")
        .arg(machine);
    add_runtime_root(&mut exec, runtime_root);
    let exec = exec
        .arg("--")
        .arg("/bin/sh")
        .arg("-lc")
        .arg("printf exec-ok")
        .output()
        .expect("exec command");
    assert!(exec.status.success());
    assert_eq!(String::from_utf8_lossy(&exec.stdout), "exec-ok");

    let host_source = temp_root.join("host.txt");
    fs::write(&host_source, "copy-ok").expect("host file");
    let mut copy = Command::new(port_bin());
    copy.arg("--config")
        .arg(config_path)
        .arg("guest")
        .arg("copy")
        .arg("--machine")
        .arg(machine);
    add_runtime_root(&mut copy, runtime_root);
    let copy = copy
        .arg("--direction")
        .arg("host-to-guest")
        .arg("--source")
        .arg(&host_source)
        .arg("--destination")
        .arg("/workspace/copied.txt")
        .output()
        .expect("copy command");
    assert!(copy.status.success());
    assert_eq!(
        fs::read_to_string(guest_root.join("workspace/copied.txt")).expect("copied file"),
        "copy-ok"
    );
    let host_roundtrip = temp_root.join("roundtrip.txt");
    let mut copy_back = Command::new(port_bin());
    copy_back
        .arg("--config")
        .arg(config_path)
        .arg("guest")
        .arg("copy")
        .arg("--machine")
        .arg(machine);
    add_runtime_root(&mut copy_back, runtime_root);
    let copy_back = copy_back
        .arg("--direction")
        .arg("guest-to-host")
        .arg("--source")
        .arg("/workspace/copied.txt")
        .arg("--destination")
        .arg(&host_roundtrip)
        .output()
        .expect("copy back command");
    assert!(copy_back.status.success());
    assert_eq!(
        fs::read_to_string(&host_roundtrip).expect("roundtrip file"),
        "copy-ok"
    );
    assert!(
        String::from_utf8_lossy(&copy_back.stdout).contains(&host_roundtrip.display().to_string())
    );

    let mut pty = Command::new(port_bin());
    pty.arg("--config")
        .arg(config_path)
        .arg("guest")
        .arg("pty")
        .arg("--machine")
        .arg(machine);
    add_runtime_root(&mut pty, runtime_root);
    let pty = pty
        .arg("--")
        .arg("/bin/sh")
        .arg("-lc")
        .arg("printf pty-ok")
        .output()
        .expect("pty command");
    assert!(pty.status.success());
    assert!(String::from_utf8_lossy(&pty.stdout).contains("pty-ok"));

    let mut logs = Command::new(port_bin());
    logs.arg("--config")
        .arg(config_path)
        .arg("guest")
        .arg("logs")
        .arg("--machine")
        .arg(machine);
    add_runtime_root(&mut logs, runtime_root);
    let logs = logs
        .arg("--path")
        .arg("/var/log/app.log")
        .arg("--tail-lines")
        .arg("1")
        .output()
        .expect("logs command");
    assert!(logs.status.success());
    assert_eq!(String::from_utf8_lossy(&logs.stdout), "second\n");

    let target = TcpListener::bind("127.0.0.1:0").expect("target listener");
    let target_addr = target.local_addr().expect("target addr");
    thread::spawn(move || {
        let (mut stream, _) = target.accept().expect("accept target");
        let mut buf = [0_u8; 32];
        let len = stream.read(&mut buf).expect("read target");
        stream.write_all(&buf[..len]).expect("write target");
    });
    let reserved = TcpListener::bind("127.0.0.1:0").expect("reserve listen port");
    let listen_addr = reserved.local_addr().expect("listen addr");
    drop(reserved);

    let mut forward = Command::new(port_bin());
    forward
        .arg("--config")
        .arg(config_path)
        .arg("guest")
        .arg("forward")
        .arg("--machine")
        .arg(machine);
    add_runtime_root(&mut forward, runtime_root);
    let mut forward = forward
        .arg("--listen")
        .arg(listen_addr.to_string())
        .arg("--target")
        .arg(target_addr.to_string())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("forward command");

    let mut forwarded = None;
    for _ in 0..100 {
        match TcpStream::connect(listen_addr) {
            Ok(stream) => {
                forwarded = Some(stream);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(20)),
        }
    }
    let mut forwarded = forwarded.expect("connect forwarded listener");
    forwarded.write_all(b"forward-ok").expect("write forwarded");
    let mut buf = [0_u8; 32];
    let len = forwarded.read(&mut buf).expect("read forwarded");
    assert_eq!(&buf[..len], b"forward-ok");

    let _ = forward.kill();
    let status = forward.wait().expect("forward process should exit");
    assert!(
        !status.success(),
        "forward process should have been terminated"
    );
}

fn hosted_config(runtime_root: &Path) -> PortConfig {
    let mut config = PortConfig::sample();
    config
        .nodes
        .get_mut("aws-linux-node")
        .expect("aws-linux-node should exist")
        .runtime_root = runtime_root.to_path_buf();
    config
}

#[test]
fn cli_guest_commands_cover_all_capabilities() {
    let temp = tempdir().expect("tempdir should exist");
    let guest_root = temp.path().join("guest-root");
    let runtime_root = temp.path().join("runtime");
    let config_path = temp.path().join("port.toml");
    fs::create_dir_all(guest_root.join("var/log")).expect("guest root");
    fs::write(guest_root.join("var/log/app.log"), "first\nsecond\n").expect("log file");

    let config = PortConfig::sample();
    write_config(&config_path, &config);

    let socket_path = runtime_socket(&runtime_root, "demo");
    spawn_agent(&socket_path, &guest_root);

    run_guest_capability_suite(
        &config_path,
        "demo",
        Some(&runtime_root),
        &guest_root,
        temp.path(),
    );
}

#[test]
fn cli_guest_commands_cover_hosted_control_plane_runtime() {
    let temp = tempdir().expect("tempdir should exist");
    let guest_root = temp.path().join("guest-root");
    let hosted_runtime_root = temp.path().join("hosted/aws-linux-node");
    let config_path = temp.path().join("port.toml");
    fs::create_dir_all(guest_root.join("var/log")).expect("guest root");
    fs::write(guest_root.join("var/log/app.log"), "first\nsecond\n").expect("log file");

    let config = hosted_config(&hosted_runtime_root);
    write_config(&config_path, &config);

    let socket_path = runtime_socket(&hosted_runtime_root, "cloud-aws");
    spawn_agent(&socket_path, &guest_root);

    run_guest_capability_suite(&config_path, "cloud-aws", None, &guest_root, temp.path());
}

#[test]
fn cli_guest_forward_supports_hosted_unix_socket_mode() {
    let temp = tempdir().expect("tempdir should exist");
    let guest_root = temp.path().join("guest-root");
    let hosted_runtime_root = temp.path().join("hosted/aws-linux-node");
    let config_path = temp.path().join("port.toml");
    fs::create_dir_all(guest_root.join("var/log")).expect("guest root");
    write_config(&config_path, &hosted_config(&hosted_runtime_root));

    let socket_path = runtime_socket(&hosted_runtime_root, "cloud-aws");
    spawn_agent(&socket_path, &guest_root);

    let target_path = temp.path().join("target.sock");
    let listen_path = temp.path().join("listen.sock");
    let target_listener = UnixListener::bind(&target_path).expect("target listener");
    thread::spawn(move || {
        let (mut stream, _) = target_listener.accept().expect("accept target");
        let mut buf = [0_u8; 32];
        let len = stream.read(&mut buf).expect("read target");
        stream.write_all(&buf[..len]).expect("write target");
    });

    let mut forward = Command::new(port_bin())
        .arg("--config")
        .arg(&config_path)
        .arg("guest")
        .arg("forward")
        .arg("--machine")
        .arg("cloud-aws")
        .arg("--listen")
        .arg(format!("unix:{}", listen_path.display()))
        .arg("--target")
        .arg(format!("unix:{}", target_path.display()))
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("forward command");

    let mut forwarded = None;
    for _ in 0..100 {
        match UnixStream::connect(&listen_path) {
            Ok(stream) => {
                forwarded = Some(stream);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(20)),
        }
    }
    let mut forwarded = forwarded.expect("connect forwarded listener");
    forwarded.write_all(b"unix-ok").expect("write forwarded");
    let mut buf = [0_u8; 32];
    let len = forwarded.read(&mut buf).expect("read forwarded");
    assert_eq!(&buf[..len], b"unix-ok");

    let _ = forward.kill();
    let status = forward.wait().expect("forward process should exit");
    assert!(
        !status.success(),
        "forward process should have been terminated"
    );
}

#[test]
fn cli_guest_forward_supports_hosted_detached_lifecycle() {
    let temp = tempdir().expect("tempdir should exist");
    let guest_root = temp.path().join("guest-root");
    let hosted_runtime_root = temp.path().join("hosted/aws-linux-node");
    let config_path = temp.path().join("port.toml");
    fs::create_dir_all(guest_root.join("var/log")).expect("guest root");
    write_config(&config_path, &hosted_config(&hosted_runtime_root));

    let socket_path = runtime_socket(&hosted_runtime_root, "cloud-aws");
    spawn_agent(&socket_path, &guest_root);

    let target = TcpListener::bind("127.0.0.1:0").expect("target listener");
    let target_addr = target.local_addr().expect("target addr");
    thread::spawn(move || {
        let (mut stream, _) = target.accept().expect("accept target");
        let mut buf = [0_u8; 32];
        let len = stream.read(&mut buf).expect("read target");
        stream.write_all(&buf[..len]).expect("write target");
    });
    let reserved = TcpListener::bind("127.0.0.1:0").expect("reserve listen port");
    let listen_addr = reserved.local_addr().expect("listen addr");
    drop(reserved);

    let start = Command::new(port_bin())
        .arg("--config")
        .arg(&config_path)
        .arg("guest")
        .arg("forward")
        .arg("--machine")
        .arg("cloud-aws")
        .arg("--listen")
        .arg(listen_addr.to_string())
        .arg("--target")
        .arg(target_addr.to_string())
        .arg("--lifecycle")
        .arg("detached")
        .arg("--name")
        .arg("hosted-detached")
        .output()
        .expect("detached start command");
    assert!(start.status.success());
    assert!(
        String::from_utf8_lossy(&start.stdout).contains("forward lifecycle: detached"),
        "stdout: {} stderr: {}",
        String::from_utf8_lossy(&start.stdout),
        String::from_utf8_lossy(&start.stderr)
    );

    let list = Command::new(port_bin())
        .arg("--config")
        .arg(&config_path)
        .arg("guest")
        .arg("forward")
        .arg("--machine")
        .arg("cloud-aws")
        .arg("--list")
        .output()
        .expect("detached list command");
    assert!(list.status.success());
    let list_stdout = String::from_utf8_lossy(&list.stdout);
    assert!(list_stdout.contains("forward: hosted-detached"));
    assert!(list_stdout.contains("state: running"));

    let mut forwarded = None;
    for _ in 0..100 {
        match TcpStream::connect(listen_addr) {
            Ok(stream) => {
                forwarded = Some(stream);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(20)),
        }
    }
    let mut forwarded = forwarded.expect("connect detached listener");
    forwarded
        .write_all(b"detached-ok")
        .expect("write forwarded");
    let mut buf = [0_u8; 32];
    let len = forwarded.read(&mut buf).expect("read forwarded");
    assert_eq!(&buf[..len], b"detached-ok");

    let stop = Command::new(port_bin())
        .arg("--config")
        .arg(&config_path)
        .arg("guest")
        .arg("forward")
        .arg("--machine")
        .arg("cloud-aws")
        .arg("--stop")
        .arg("--name")
        .arg("hosted-detached")
        .output()
        .expect("detached stop command");
    assert!(stop.status.success());
    assert!(String::from_utf8_lossy(&stop.stdout).contains("forward state: stopped"));

    let list_after = Command::new(port_bin())
        .arg("--config")
        .arg(&config_path)
        .arg("guest")
        .arg("forward")
        .arg("--machine")
        .arg("cloud-aws")
        .arg("--list")
        .output()
        .expect("detached list after stop");
    assert!(list_after.status.success());
    assert!(
        String::from_utf8_lossy(&list_after.stdout)
            .contains("no detached forwards found for machine 'cloud-aws'")
    );
}
