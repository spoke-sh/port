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

fn write_executable(path: &Path, body: &str) {
    fs::write(path, body).expect("script should write");
    let mut permissions = fs::metadata(path)
        .expect("script metadata should exist")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).expect("script should be executable");
}

fn git(repo: &Path, args: &[&str]) -> Output {
    let mut command = Command::new("git");
    command.current_dir(repo).args(args);
    command.output().expect("git should run")
}

fn init_fake_source_repo(root: &Path) -> (PathBuf, String) {
    let repo = root.join("fake-port");
    fs::create_dir_all(repo.join("docs")).expect("docs dir should exist");
    fs::create_dir_all(repo.join("examples/bootstrap/demo-k3s"))
        .expect("examples dir should exist");
    fs::create_dir_all(repo.join("scripts/artifacts")).expect("scripts dir should exist");
    fs::write(
        repo.join("Cargo.toml"),
        "[workspace]\nmembers = []\nresolver = \"2\"\n\n[workspace.package]\nrust-version = \"1.85\"\nversion = \"1.2.3\"\n",
    )
    .expect("Cargo.toml should write");
    fs::write(repo.join("README.md"), "# Fake Port\n").expect("README should write");
    fs::write(repo.join("RELEASE.md"), "# Fake Release\n").expect("RELEASE should write");
    fs::write(repo.join("docs/install.md"), "# Fake Install\n").expect("install doc should write");
    fs::write(
        repo.join("examples/bootstrap/demo-k3s/install-k3s-offline.sh"),
        "#!/usr/bin/env bash\nexit 0\n",
    )
    .expect("bootstrap script should write");
    fs::write(
        repo.join("scripts/artifacts/validate-kernel.sh"),
        "#!/usr/bin/env bash\nexit 0\n",
    )
    .expect("validate-kernel script should write");
    fs::write(
        repo.join("scripts/artifacts/validate-guest-image.sh"),
        "#!/usr/bin/env bash\nexit 0\n",
    )
    .expect("validate-guest-image script should write");

    assert_success(&git(&repo, &["init", "-q"]));
    assert_success(&git(&repo, &["config", "user.name", "Port Tests"]));
    assert_success(&git(
        &repo,
        &["config", "user.email", "port-tests@example.com"],
    ));
    assert_success(&git(&repo, &["add", "."]));
    assert_success(&git(&repo, &["commit", "-qm", "initial"]));
    assert_success(&git(&repo, &["tag", "v1.2.3"]));

    let rev_parse = git(&repo, &["rev-parse", "HEAD"]);
    assert_success(&rev_parse);
    let sha = output_text(&rev_parse.stdout).trim().to_string();
    (repo, sha)
}

fn install_fake_toolchain(root: &Path) -> PathBuf {
    let bin_dir = root.join("fake-bin");
    fs::create_dir_all(&bin_dir).expect("fake bin dir should exist");
    let cargo_script = bin_dir.join("fake-cargo.sh");
    let rustup_script = bin_dir.join("rustup");

    write_executable(
        &cargo_script,
        "#!/usr/bin/env bash\nset -euo pipefail\n\
target=''\n\
while [[ $# -gt 0 ]]; do\n\
  case \"$1\" in\n\
    --target)\n\
      target=\"$2\"\n\
      shift 2\n\
      ;;\n\
    *)\n\
      shift\n\
      ;;\n\
  esac\n\
done\n\
if [[ -z \"${target}\" ]]; then\n\
  printf 'missing --target\\n' >&2\n\
  exit 1\n\
fi\n\
out_dir=\"${CARGO_TARGET_DIR:?}/${target}/release\"\n\
mkdir -p \"${out_dir}\"\n\
cat > \"${out_dir}/port\" <<'EOF'\n\
#!/usr/bin/env bash\n\
printf 'fake source-built port\\n'\n\
EOF\n\
chmod 755 \"${out_dir}/port\"\n",
    );

    let cargo_script_quoted = cargo_script.display().to_string();
    write_executable(
        &rustup_script,
        &format!(
            "#!/usr/bin/env bash\nset -euo pipefail\n\
case \"${{1:-}}\" in\n\
  toolchain)\n\
    if [[ \"${{2:-}}\" == \"list\" ]]; then\n\
      printf 'stable-x86_64-unknown-linux-gnu (default)\\n'\n\
      exit 0\n\
    fi\n\
    ;;\n\
  run)\n\
    shift 2\n\
    case \"${{1:-}}\" in\n\
      rustc)\n\
        if [[ \"${{2:-}}\" == \"-V\" ]]; then\n\
          printf 'rustc 1.85.0 (fake 2026-01-01)\\n'\n\
          exit 0\n\
        fi\n\
        if [[ \"${{2:-}}\" == \"-vV\" ]]; then\n\
          cat <<'EOF'\n\
rustc 1.85.0 (fake 2026-01-01)\n\
binary: rustc\n\
commit-hash: deadbeef\n\
commit-date: 2026-01-01\n\
host: x86_64-unknown-linux-gnu\n\
release: 1.85.0\n\
LLVM version: 20.1.0\n\
EOF\n\
          exit 0\n\
        fi\n\
        ;;\n\
      cargo)\n\
        shift\n\
        exec \"{cargo_script_quoted}\" \"$@\"\n\
        ;;\n\
    esac\n\
    ;;\n\
esac\n\
printf 'unexpected rustup invocation: %s\\n' \"$*\" >&2\n\
exit 1\n"
        ),
    );

    bin_dir
}

#[test]
fn upgrade_runs_release_installer_for_latest_version() {
    let temp = tempdir().expect("tempdir should exist");
    let home = temp.path().join("home");
    let cargo_home = temp.path().join("cargo-home");
    let installer = temp.path().join("port-installer.sh");
    let log = temp.path().join("latest-install.log");

    fs::create_dir_all(&home).expect("home should exist");
    fs::create_dir_all(&cargo_home).expect("cargo home should exist");
    write_executable(
        &installer,
        &format!(
            "#!/usr/bin/env bash\nset -euo pipefail\n\
printf 'latest-release\\n' > '{}'\n\
install -d '{}'/bin\n\
cat > '{}'/bin/port <<'EOF'\n\
#!/usr/bin/env bash\n\
printf 'latest release port\\n'\n\
EOF\n\
chmod 755 '{}'/bin/port\n",
            log.display(),
            cargo_home.display(),
            cargo_home.display(),
            cargo_home.display()
        ),
    );

    let output = Command::new(assert_cmd::cargo::cargo_bin!("port"))
        .current_dir(repo_root())
        .env("HOME", &home)
        .env("CARGO_HOME", &cargo_home)
        .env("PORT_RELEASE_INSTALLER_PATH", &installer)
        .arg("upgrade")
        .output()
        .expect("upgrade command should run");
    assert_success(&output);

    assert_eq!(
        fs::read_to_string(&log).expect("log should exist"),
        "latest-release\n"
    );
    assert!(cargo_home.join("bin/port").exists());
}

#[test]
fn upgrade_builds_and_installs_tagged_revision_from_source_cache() {
    let temp = tempdir().expect("tempdir should exist");
    let home = temp.path().join("home");
    let cargo_home = temp.path().join("cargo-home");
    let cache_root = temp.path().join("cache");
    let (repo, _) = init_fake_source_repo(temp.path());
    let fake_bin_dir = install_fake_toolchain(temp.path());
    let path = format!(
        "{}:{}",
        fake_bin_dir.display(),
        std::env::var("PATH").expect("PATH should exist")
    );

    fs::create_dir_all(&home).expect("home should exist");
    fs::create_dir_all(&cargo_home).expect("cargo home should exist");

    let output = Command::new(assert_cmd::cargo::cargo_bin!("port"))
        .current_dir(repo_root())
        .env("HOME", &home)
        .env("CARGO_HOME", &cargo_home)
        .env("PATH", &path)
        .env("PORT_CACHE_ROOT", &cache_root)
        .env("PORT_UPGRADE_REPO_URL", &repo)
        .args(["upgrade", "--tag", "v1.2.3"])
        .output()
        .expect("upgrade command should run");
    assert_success(&output);

    assert!(cache_root.join("src/.git").is_dir());
    assert!(cargo_home.join("bin/port").exists());
    assert!(cargo_home.join("share/port/README.md").exists());
    assert!(cargo_home.join("share/port/RELEASE.md").exists());
    assert!(cargo_home.join("share/port/docs/install.md").exists());
    assert!(
        cargo_home
            .join("share/port/scripts/artifacts/validate-kernel.sh")
            .exists()
    );
    assert!(
        cargo_home
            .join("share/port/examples/bootstrap/demo-k3s/install-k3s-offline.sh")
            .exists()
    );
}

#[test]
fn upgrade_builds_and_installs_sha_revision_from_source_cache() {
    let temp = tempdir().expect("tempdir should exist");
    let home = temp.path().join("home");
    let cargo_home = temp.path().join("cargo-home");
    let cache_root = temp.path().join("cache");
    let (repo, sha) = init_fake_source_repo(temp.path());
    let fake_bin_dir = install_fake_toolchain(temp.path());
    let path = format!(
        "{}:{}",
        fake_bin_dir.display(),
        std::env::var("PATH").expect("PATH should exist")
    );

    fs::create_dir_all(&home).expect("home should exist");
    fs::create_dir_all(&cargo_home).expect("cargo home should exist");

    let output = Command::new(assert_cmd::cargo::cargo_bin!("port"))
        .current_dir(repo_root())
        .env("HOME", &home)
        .env("CARGO_HOME", &cargo_home)
        .env("PATH", &path)
        .env("PORT_CACHE_ROOT", &cache_root)
        .env("PORT_UPGRADE_REPO_URL", &repo)
        .args(["upgrade", "--sha", &sha])
        .output()
        .expect("upgrade command should run");
    assert_success(&output);

    assert!(cargo_home.join("bin/port").exists());
    assert!(cargo_home.join("share/port/docs/install.md").exists());
}
