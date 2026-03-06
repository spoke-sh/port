use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::Command;
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
    let forward = Command::new(port_bin())
        .arg("--config")
        .arg(&config_path)
        .arg("guest")
        .arg("forward")
        .arg("--machine")
        .arg("demo")
        .arg("--runtime-root")
        .arg(&runtime_root)
        .arg("--listen")
        .arg("127.0.0.1:0")
        .arg("--target")
        .arg(target_addr.to_string())
        .output()
        .expect("forward command");
    assert!(forward.status.success());
    let stdout = String::from_utf8_lossy(&forward.stdout);
    let listen = stdout
        .lines()
        .find_map(|line| line.strip_prefix("forward listening: "))
        .expect("forward listener output");
    let mut forwarded = TcpStream::connect(listen).expect("connect forwarded listener");
    forwarded.write_all(b"forward-ok").expect("write forwarded");
    let mut buf = [0_u8; 32];
    let len = forwarded.read(&mut buf).expect("read forwarded");
    assert_eq!(&buf[..len], b"forward-ok");
}
