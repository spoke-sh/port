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

fn package_proof_script() -> PathBuf {
    repo_root().join("scripts/package-proof.sh")
}

fn package_proof_command(target: &str, package_output_dir: &Path, proof_root: &Path) -> Command {
    let mut command = Command::new("bash");
    command.arg(package_proof_script());
    command.arg(target);
    command.arg(package_output_dir);
    command.arg(proof_root);
    command.current_dir(repo_root());
    command
}

fn write_fake_port_binary(dir: &Path) -> PathBuf {
    let path = dir.join("port");
    fs::write(
        &path,
        "#!/usr/bin/env bash\nset -euo pipefail\ncase \"${1:-}\" in\n  --version)\n    printf 'port 9.9.9-test\\n'\n    ;;\n  doctor)\n    printf 'doctor ok\\n'\n    ;;\n  *)\n    printf 'unexpected args: %s\\n' \"$*\" >&2\n    exit 1\n    ;;\nesac\n",
    )
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

#[test]
fn package_proof_installs_packaged_port_and_runs_version_and_doctor_from_prefix() {
    let temp = tempdir().expect("tempdir should exist");
    let fake_bin = write_fake_port_binary(temp.path());
    let package_output_dir = temp.path().join("packages");
    let proof_root = temp.path().join("proof");
    let target = "x86_64-unknown-linux-gnu";
    let version = "9.9.9-test";
    let artifact = package_output_dir.join(format!("port-{version}-{target}.tar.gz"));
    let installed_binary = proof_root.join("prefix/bin/port");

    let output = package_proof_command(target, &package_output_dir, &proof_root)
        .env("PORT_PACKAGE_BIN", &fake_bin)
        .env("PORT_PACKAGE_VERSION", version)
        .output()
        .expect("package proof should run");
    assert_success(&output);

    let stdout = output_text(&output.stdout);
    assert!(stdout.contains(&format!("artifact: {}", artifact.display())));
    assert!(stdout.contains(&format!("proof-root: {}", proof_root.display())));
    assert!(stdout.contains(&format!("binary: {}", installed_binary.display())));
    assert!(stdout.contains("version-output: port 9.9.9-test"));
    assert!(stdout.contains("doctor-command: "));
    assert!(stdout.contains("doctor-status: ok"));
    assert!(stdout.contains("doctor-output:"));
    assert!(stdout.contains("doctor ok"));

    assert!(
        artifact.exists(),
        "expected archive at {}",
        artifact.display()
    );
    assert!(
        installed_binary.exists(),
        "expected installed binary at {}",
        installed_binary.display()
    );
    assert!(proof_root.join("prefix/share/port/README.md").exists());
    assert!(proof_root.join("prefix/share/port/RELEASE.md").exists());
    assert!(
        proof_root
            .join("prefix/share/port/docs/install.md")
            .exists()
    );
}
