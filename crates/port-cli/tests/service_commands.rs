use std::fs;
use std::path::Path;
use std::process::Command;

use port_model::PortConfig;
use serde_json::json;
use tempfile::tempdir;

fn write_config(path: &Path, config: &PortConfig) {
    fs::write(path, config.to_toml_string().expect("config should encode"))
        .expect("config should write");
}

fn port_bin() -> &'static str {
    env!("CARGO_BIN_EXE_port")
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

fn write_machine_manifest(runtime_root: &Path, machine: &str, pid: u32) {
    let runtime_dir = runtime_root.join(machine);
    fs::create_dir_all(&runtime_dir).expect("runtime dir should exist");
    let manifest_path = runtime_dir.join("manifest.json");
    let manifest = json!({
        "machine_name": machine,
        "pid": pid,
        "launched_at_unix_s": 1,
        "runtime_dir": runtime_dir,
        "firecracker_binary": "/usr/bin/firecracker",
        "config_path": runtime_dir.join("firecracker-config.json"),
        "log_path": runtime_dir.join("firecracker.log"),
        "stdout_path": runtime_dir.join("console.stdout.log"),
        "stderr_path": runtime_dir.join("console.stderr.log"),
        "manifest_path": manifest_path,
    });
    fs::write(
        &manifest_path,
        format!(
            "{}\n",
            serde_json::to_string_pretty(&manifest).expect("manifest should encode")
        ),
    )
    .expect("manifest should write");
}

#[test]
fn cli_service_commands_cover_hosted_secret_service_and_sandbox_contracts() {
    let temp = tempdir().expect("tempdir should exist");
    let hosted_runtime_root = temp.path().join("hosted/aws-linux-node");
    let config_path = temp.path().join("port.toml");
    write_config(&config_path, &hosted_config(&hosted_runtime_root));
    write_machine_manifest(&hosted_runtime_root, "cloud-aws", 424242);

    let secret_put = Command::new(port_bin())
        .arg("--config")
        .arg(&config_path)
        .arg("service")
        .arg("secret")
        .arg("put")
        .arg("--machine")
        .arg("cloud-aws")
        .arg("--name")
        .arg("demo-token")
        .arg("--value")
        .arg("s3cr3t")
        .output()
        .expect("secret put should run");
    assert!(secret_put.status.success());
    let secret_stdout = String::from_utf8_lossy(&secret_put.stdout);
    assert!(secret_stdout.contains("secret: demo-token"));
    assert!(secret_stdout.contains("service route: hosted-control-plane"));
    assert!(secret_stdout.contains("control plane: demo"));
    assert!(secret_stdout.contains("node: aws-linux-node"));

    let apply_service = Command::new(port_bin())
        .arg("--config")
        .arg(&config_path)
        .arg("service")
        .arg("apply")
        .arg("--machine")
        .arg("cloud-aws")
        .arg("--name")
        .arg("api")
        .arg("--kind")
        .arg("service")
        .arg("--secret")
        .arg("API_TOKEN=demo-token")
        .arg("--")
        .arg("/app/api")
        .arg("--listen")
        .arg(":8080")
        .output()
        .expect("service apply should run");
    assert!(apply_service.status.success());
    let service_stdout = String::from_utf8_lossy(&apply_service.stdout);
    assert!(service_stdout.contains("name: api"));
    assert!(service_stdout.contains("kind: service"));
    assert!(service_stdout.contains("desired state: active"));
    assert!(service_stdout.contains("runtime state: stored"));
    assert!(
        service_stdout.contains("runtime record: ")
            && service_stdout.contains("hosted/aws-linux-node/cloud-aws/services/runtime/api.json")
    );
    assert!(service_stdout.contains("secret bindings: API_TOKEN=demo-token"));

    let apply_sandbox = Command::new(port_bin())
        .arg("--config")
        .arg(&config_path)
        .arg("service")
        .arg("apply")
        .arg("--machine")
        .arg("cloud-aws")
        .arg("--name")
        .arg("buildbox")
        .arg("--kind")
        .arg("sandbox")
        .arg("--secret")
        .arg("API_TOKEN=demo-token")
        .arg("--")
        .arg("/bin/sh")
        .arg("-lc")
        .arg("make test")
        .output()
        .expect("sandbox apply should run");
    assert!(apply_sandbox.status.success());
    let sandbox_stdout = String::from_utf8_lossy(&apply_sandbox.stdout);
    assert!(sandbox_stdout.contains("name: buildbox"));
    assert!(sandbox_stdout.contains("kind: sandbox"));

    let list = Command::new(port_bin())
        .arg("--config")
        .arg(&config_path)
        .arg("service")
        .arg("list")
        .arg("--machine")
        .arg("cloud-aws")
        .output()
        .expect("service list should run");
    assert!(list.status.success());
    let list_stdout = String::from_utf8_lossy(&list.stdout);
    assert!(list_stdout.contains("name: api"));
    assert!(list_stdout.contains("name: buildbox"));

    let status = Command::new(port_bin())
        .arg("--config")
        .arg(&config_path)
        .arg("service")
        .arg("status")
        .arg("--machine")
        .arg("cloud-aws")
        .arg("--name")
        .arg("api")
        .output()
        .expect("service status should run");
    assert!(status.status.success());
    let status_stdout = String::from_utf8_lossy(&status.stdout);
    assert!(status_stdout.contains("guest broker: control-plane-node-agent-tunnel"));
    assert!(status_stdout.contains("service route: hosted-control-plane"));
    assert!(status_stdout.contains("runtime state: stored"));
    assert!(
        status_stdout.contains("runtime record: ")
            && status_stdout.contains("hosted/aws-linux-node/cloud-aws/services/runtime/api.json")
    );
    assert!(status_stdout.contains("command: /app/api --listen :8080"));

    let stop = Command::new(port_bin())
        .arg("--config")
        .arg(&config_path)
        .arg("service")
        .arg("stop")
        .arg("--machine")
        .arg("cloud-aws")
        .arg("--name")
        .arg("api")
        .output()
        .expect("service stop should run");
    assert!(stop.status.success());
    let stop_stdout = String::from_utf8_lossy(&stop.stdout);
    assert!(stop_stdout.contains("desired state: stopped"));
    assert!(stop_stdout.contains("runtime state: stored"));
}
