use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::tempdir;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root should resolve")
}

fn package_script() -> PathBuf {
    repo_root().join("scripts/package-port.sh")
}

fn package_command(target: &str, output_dir: &Path) -> Command {
    let mut command = Command::new("bash");
    command.arg(package_script());
    command.arg(target);
    command.arg(output_dir);
    command.current_dir(repo_root());
    command
}

fn write_fake_port_binary(dir: &Path) -> PathBuf {
    let path = dir.join("port");
    fs::write(&path, "#!/bin/sh\nprintf 'port test binary\\n'\n")
        .expect("fake binary should write");
    let mut permissions = fs::metadata(&path)
        .expect("fake binary metadata should exist")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&path, permissions).expect("fake binary should be executable");
    path
}

fn output_text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "expected success\nstdout:\n{}\nstderr:\n{}",
        output_text(&output.stdout),
        output_text(&output.stderr)
    );
}

fn assert_failure(output: &Output) {
    assert!(
        !output.status.success(),
        "expected failure\nstdout:\n{}\nstderr:\n{}",
        output_text(&output.stdout),
        output_text(&output.stderr)
    );
}

fn extract_archive(archive: &Path, destination: &Path) {
    let output = Command::new("tar")
        .arg("-xzf")
        .arg(archive)
        .arg("-C")
        .arg(destination)
        .output()
        .expect("archive extraction should run");
    assert_success(&output);
}

#[test]
fn package_workflow_emits_versioned_artifact_and_reports_target_and_included_files() {
    let temp = tempdir().expect("tempdir should exist");
    let fake_bin = write_fake_port_binary(temp.path());
    let output_dir = temp.path().join("packages");
    let target = "aarch64-apple-darwin";
    let version = "9.9.9-test";
    let package_name = format!("port-{version}-{target}");
    let archive = output_dir.join(format!("{package_name}.tar.gz"));

    let output = package_command(target, &output_dir)
        .env("PORT_PACKAGE_BIN", &fake_bin)
        .env("PORT_PACKAGE_VERSION", version)
        .output()
        .expect("package workflow should run");
    assert_success(&output);

    let stdout = output_text(&output.stdout);
    assert!(stdout.contains(&format!("artifact: {}", archive.display())));
    assert!(stdout.contains(&format!("target: {target}")));
    assert!(stdout.contains(&format!("version: {version}")));
    assert!(stdout.contains("included-files:"));
    assert!(stdout.contains("bin/port"));
    assert!(stdout.contains("README.md"));
    assert!(stdout.contains("RELEASE.md"));
    assert!(stdout.contains("docs/install.md"));
    assert!(stdout.contains("PACKAGE_METADATA.txt"));
    assert!(stdout.contains("PACKAGE_MANIFEST.txt"));
    assert!(
        archive.exists(),
        "expected archive at {}",
        archive.display()
    );

    let extract_dir = temp.path().join("extract");
    fs::create_dir_all(&extract_dir).expect("extract dir should exist");
    extract_archive(&archive, &extract_dir);

    let package_root = extract_dir.join(&package_name);
    assert!(package_root.join("bin/port").exists());
    assert!(package_root.join("README.md").exists());
    assert!(package_root.join("RELEASE.md").exists());
    assert!(package_root.join("docs/install.md").exists());

    let metadata = fs::read_to_string(package_root.join("PACKAGE_METADATA.txt"))
        .expect("package metadata should exist");
    assert!(metadata.contains(&format!("version: {version}")));
    assert!(metadata.contains(&format!("target: {target}")));

    let manifest = fs::read_to_string(package_root.join("PACKAGE_MANIFEST.txt"))
        .expect("package manifest should exist");
    assert!(manifest.contains("bin/port"));
    assert!(manifest.contains("README.md"));
    assert!(manifest.contains("RELEASE.md"));
    assert!(manifest.contains("docs/install.md"));
}

#[test]
fn package_determinism_keeps_archive_bytes_stable_across_repeated_runs() {
    let temp = tempdir().expect("tempdir should exist");
    let fake_bin = write_fake_port_binary(temp.path());
    let output_dir = temp.path().join("packages");
    let target = "x86_64-unknown-linux-gnu";
    let version = "9.9.9-test";
    let archive = output_dir.join(format!("port-{version}-{target}.tar.gz"));

    let first = package_command(target, &output_dir)
        .env("PORT_PACKAGE_BIN", &fake_bin)
        .env("PORT_PACKAGE_VERSION", version)
        .output()
        .expect("first package run should execute");
    assert_success(&first);
    let first_bytes = fs::read(&archive).expect("first archive should exist");

    let second = package_command(target, &output_dir)
        .env("PORT_PACKAGE_BIN", &fake_bin)
        .env("PORT_PACKAGE_VERSION", version)
        .output()
        .expect("second package run should execute");
    assert_success(&second);
    let second_bytes = fs::read(&archive).expect("second archive should exist");

    assert_eq!(first_bytes, second_bytes);
}

#[test]
fn package_failure_rejects_unsupported_target_with_guidance() {
    let temp = tempdir().expect("tempdir should exist");
    let fake_bin = write_fake_port_binary(temp.path());
    let output_dir = temp.path().join("packages");

    let output = package_command("x86_64-pc-windows-msvc", &output_dir)
        .env("PORT_PACKAGE_BIN", &fake_bin)
        .env("PORT_PACKAGE_VERSION", "9.9.9-test")
        .output()
        .expect("package workflow should run");
    assert_failure(&output);

    let stderr = output_text(&output.stderr);
    assert!(stderr.contains("unsupported package target 'x86_64-pc-windows-msvc'"));
    assert!(stderr.contains("Supported targets:"));
    assert!(stderr.contains("x86_64-unknown-linux-gnu"));
    assert!(stderr.contains("x86_64-apple-darwin"));
    assert!(stderr.contains("aarch64-apple-darwin"));
    assert!(stderr.contains("docs/install.md"));
    assert!(!output_dir.exists());
}

#[test]
fn package_failure_reports_missing_required_tool_with_guidance() {
    let temp = tempdir().expect("tempdir should exist");
    let fake_bin = write_fake_port_binary(temp.path());
    let output_dir = temp.path().join("packages");
    let missing_tar = temp.path().join("missing-tar");
    let archive = output_dir.join("port-9.9.9-test-x86_64-unknown-linux-gnu.tar.gz");

    let output = package_command("x86_64-unknown-linux-gnu", &output_dir)
        .env("PORT_PACKAGE_BIN", &fake_bin)
        .env("PORT_PACKAGE_VERSION", "9.9.9-test")
        .env("PORT_PACKAGE_TAR", &missing_tar)
        .output()
        .expect("package workflow should run");
    assert_failure(&output);

    let stderr = output_text(&output.stderr);
    assert!(stderr.contains("missing required packaging tool 'tar'"));
    assert!(stderr.contains("Install it in the dev environment and rerun"));
    assert!(!archive.exists());
}
