use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

use port_model::PortConfig;
use tempfile::tempdir;

fn write_config(path: &Path) {
    fs::write(
        path,
        PortConfig::sample()
            .to_toml_string()
            .expect("sample config should encode"),
    )
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

#[test]
fn cli_guest_commands_cover_all_capabilities() {
    let temp = tempdir().expect("tempdir should exist");
    let guest_root = temp.path().join("guest-root");
    let runtime_root = temp.path().join("runtime");
    let config_path = temp.path().join("port.toml");
    fs::create_dir_all(guest_root.join("var/log")).expect("guest root");
    fs::write(guest_root.join("var/log/app.log"), "first\nsecond\n").expect("log file");
    write_config(&config_path);

    let socket_path = runtime_socket(&runtime_root, "demo");
    spawn_agent(&socket_path, &guest_root);

    let exec = Command::new(port_bin())
        .arg("--config")
        .arg(&config_path)
        .arg("guest")
        .arg("exec")
        .arg("--machine")
        .arg("demo")
        .arg("--runtime-root")
        .arg(&runtime_root)
        .arg("--")
        .arg("/bin/sh")
        .arg("-lc")
        .arg("printf exec-ok")
        .output()
        .expect("exec command");
    assert!(exec.status.success());
    assert_eq!(String::from_utf8_lossy(&exec.stdout), "exec-ok");

    let host_source = temp.path().join("host.txt");
    fs::write(&host_source, "copy-ok").expect("host file");
    let copy = Command::new(port_bin())
        .arg("--config")
        .arg(&config_path)
        .arg("guest")
        .arg("copy")
        .arg("--machine")
        .arg("demo")
        .arg("--runtime-root")
        .arg(&runtime_root)
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
    let host_roundtrip = temp.path().join("roundtrip.txt");
    let copy_back = Command::new(port_bin())
        .arg("--config")
        .arg(&config_path)
        .arg("guest")
        .arg("copy")
        .arg("--machine")
        .arg("demo")
        .arg("--runtime-root")
        .arg(&runtime_root)
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

    let pty = Command::new(port_bin())
        .arg("--config")
        .arg(&config_path)
        .arg("guest")
        .arg("pty")
        .arg("--machine")
        .arg("demo")
        .arg("--runtime-root")
        .arg(&runtime_root)
        .arg("--")
        .arg("/bin/sh")
        .arg("-lc")
        .arg("printf pty-ok")
        .output()
        .expect("pty command");
    assert!(pty.status.success());
    assert!(String::from_utf8_lossy(&pty.stdout).contains("pty-ok"));

    let logs = Command::new(port_bin())
        .arg("--config")
        .arg(&config_path)
        .arg("guest")
        .arg("logs")
        .arg("--machine")
        .arg("demo")
        .arg("--runtime-root")
        .arg(&runtime_root)
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

    let mut forward = Command::new(port_bin())
        .arg("--config")
        .arg(&config_path)
        .arg("guest")
        .arg("forward")
        .arg("--machine")
        .arg("demo")
        .arg("--runtime-root")
        .arg(&runtime_root)
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
