use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;

use anyhow::{Context, Result, bail};
use reqwest::blocking::Client;
use serde::Deserialize;

use crate::UpgradeCommand;

const DEFAULT_RELEASE_INSTALLER_URL: &str =
    "https://github.com/spoke-sh/port/releases/latest/download/port-installer.sh";
const DEFAULT_REPO_URL: &str = "https://github.com/spoke-sh/port.git";
const SUPPORTED_TARGETS: &[&str] = &[
    "x86_64-unknown-linux-gnu",
    "x86_64-apple-darwin",
    "aarch64-apple-darwin",
];

pub fn run(command: UpgradeCommand) -> Result<()> {
    match (command.tag.as_deref(), command.sha.as_deref()) {
        (Some(tag), None) => install_revision(Revision::Tag(tag)),
        (None, Some(sha)) => install_revision(Revision::Sha(sha)),
        (None, None) => install_latest_release(),
        _ => bail!("choose either --tag or --sha"),
    }
}

fn install_latest_release() -> Result<()> {
    let installer_path = if let Some(configured) = env::var_os("PORT_RELEASE_INSTALLER_PATH") {
        PathBuf::from(configured)
    } else {
        let installer_url = env::var("PORT_RELEASE_INSTALLER_URL")
            .unwrap_or_else(|_| DEFAULT_RELEASE_INSTALLER_URL.to_string());
        let downloads_root = cache_root()?.join("downloads");
        fs::create_dir_all(&downloads_root)
            .with_context(|| format!("failed to create {}", downloads_root.display()))?;
        let installer_path = downloads_root.join("port-installer.sh");
        download_to_path(&installer_url, &installer_path)?;
        installer_path
    };

    if !installer_path.is_file() {
        bail!(
            "release installer '{}' does not exist",
            installer_path.display()
        );
    }

    println!("upgrade mode: latest release");
    println!("installer: {}", installer_path.display());
    let mut command = ProcessCommand::new("sh");
    command.arg(&installer_path);
    run_command_forwarding_output(command, "run the Port release installer")
}

fn install_revision(revision: Revision<'_>) -> Result<()> {
    let cache_root = cache_root()?;
    let repo_cache = cache_root.join("src");
    let target_dir = cache_root.join("target");
    let repo_url = env::var("PORT_UPGRADE_REPO_URL").unwrap_or_else(|_| DEFAULT_REPO_URL.into());

    fs::create_dir_all(&cache_root)
        .with_context(|| format!("failed to create {}", cache_root.display()))?;
    refresh_repo(&repo_url, &repo_cache)?;
    let resolved_sha = checkout_revision(&repo_cache, revision)?;
    let toolchain = select_toolchain(&repo_cache)?;
    let host_target = toolchain.host_target()?;
    ensure_supported_target(&host_target)?;
    let binary_path = build_port_binary(&repo_cache, &target_dir, &toolchain, &host_target)?;
    let install_script = resolve_local_install_script()?;

    println!("upgrade mode: source");
    println!("cache root: {}", cache_root.display());
    println!("source repo: {}", repo_cache.display());
    println!("resolved revision: {}", resolved_sha);
    println!("toolchain: {}", toolchain.label());
    println!("target: {}", host_target);
    println!("built binary: {}", binary_path.display());
    println!("installer: {}", install_script.display());

    let mut command = ProcessCommand::new("sh");
    command.arg(&install_script).arg(&repo_cache).arg(&binary_path);
    run_command_forwarding_output(command, "run the local Port installer")
}

fn download_to_path(url: &str, destination: &Path) -> Result<()> {
    let client = Client::builder()
        .build()
        .context("failed to construct HTTP client for Port upgrade")?;
    let response = client
        .get(url)
        .send()
        .with_context(|| format!("failed to download Port installer from {url}"))?
        .error_for_status()
        .with_context(|| format!("Port installer download failed for {url}"))?;
    let bytes = response
        .bytes()
        .context("failed to read downloaded Port installer body")?;
    fs::write(destination, &bytes)
        .with_context(|| format!("failed to write {}", destination.display()))?;
    Ok(())
}

fn refresh_repo(repo_url: &str, repo_dir: &Path) -> Result<()> {
    if repo_dir.join(".git").is_dir() {
        run_command_capture(
            {
                let mut command = ProcessCommand::new("git");
                command
                    .arg("-C")
                    .arg(repo_dir)
                    .args(["remote", "set-url", "origin", repo_url]);
                command
            },
            "point the cached Port repo at the requested origin",
        )?;
        run_command_capture(
            {
                let mut command = ProcessCommand::new("git");
                command
                    .arg("-C")
                    .arg(repo_dir)
                    .args(["fetch", "--tags", "--force", "--prune", "origin"]);
                command
            },
            "refresh the cached Port repo",
        )?;
    } else {
        if let Some(parent) = repo_dir.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
        run_command_capture(
            {
                let mut command = ProcessCommand::new("git");
                command
                    .arg("clone")
                    .arg("--no-checkout")
                    .arg(repo_url)
                    .arg(repo_dir);
                command
            },
            "clone the Port source cache",
        )?;
        run_command_capture(
            {
                let mut command = ProcessCommand::new("git");
                command
                    .arg("-C")
                    .arg(repo_dir)
                    .args(["fetch", "--tags", "--force", "--prune", "origin"]);
                command
            },
            "refresh the cloned Port source cache",
        )?;
    }

    Ok(())
}

fn checkout_revision(repo_dir: &Path, revision: Revision<'_>) -> Result<String> {
    let resolved = match revision {
        Revision::Tag(tag) => {
            let tag_ref = format!("refs/tags/{tag}^{{}}");
            run_command_capture(
                {
                    let mut command = ProcessCommand::new("git");
                    command
                        .arg("-C")
                        .arg(repo_dir)
                        .args(["rev-parse", "--verify", &tag_ref]);
                    command
                },
                &format!("resolve git tag '{tag}'"),
            )?
        }
        Revision::Sha(sha) => run_command_capture(
            {
                let mut command = ProcessCommand::new("git");
                command
                    .arg("-C")
                    .arg(repo_dir)
                    .args(["rev-parse", "--verify", sha]);
                command
            },
            &format!("resolve git sha '{sha}'"),
        )?,
    };
    let resolved = resolved.trim().to_string();

    run_command_capture(
        {
            let mut command = ProcessCommand::new("git");
            command
                .arg("-C")
                .arg(repo_dir)
                .args(["checkout", "--force", "--detach", &resolved]);
            command
        },
        "check out the requested Port revision",
    )?;

    Ok(resolved)
}

fn build_port_binary(
    repo_dir: &Path,
    target_dir: &Path,
    toolchain: &ToolchainSelection,
    target: &str,
) -> Result<PathBuf> {
    fs::create_dir_all(target_dir)
        .with_context(|| format!("failed to create {}", target_dir.display()))?;
    let mut command = toolchain.cargo_command();
    command
        .current_dir(repo_dir)
        .env("CARGO_TARGET_DIR", target_dir)
        .args([
            "build", "--locked", "--release", "--target", target, "-p", "port", "--bin", "port",
        ]);
    run_command_capture(command, "build Port from source")?;

    let binary_path = target_dir.join(target).join("release").join("port");
    if !binary_path.is_file() {
        bail!(
            "source build succeeded but '{}' was not produced",
            binary_path.display()
        );
    }

    Ok(binary_path)
}

fn resolve_local_install_script() -> Result<PathBuf> {
    if let Some(configured) = env::var_os("PORT_LOCAL_INSTALL_SCRIPT") {
        let path = PathBuf::from(configured);
        if path.is_file() {
            return Ok(path);
        }
        bail!(
            "configured local installer '{}' does not exist",
            path.display()
        );
    }

    let mut candidates = Vec::new();

    if let Some(repo_root) = env::var_os("PORT_REPO_ROOT") {
        candidates.push(PathBuf::from(repo_root).join("scripts/install-local-port.sh"));
    }

    if let Ok(current_exe) = env::current_exe() {
        if let Some(prefix_root) = current_exe.parent().and_then(Path::parent) {
            candidates.push(prefix_root.join("share/port/scripts/install-local-port.sh"));
            candidates.push(prefix_root.join("scripts/install-local-port.sh"));
            candidates.push(prefix_root.join("install-local-port.sh"));
        }
    }

    if let Ok(current_dir) = env::current_dir() {
        for ancestor in current_dir.ancestors() {
            candidates.push(ancestor.join("scripts/install-local-port.sh"));
        }
    }

    if cfg!(debug_assertions) {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        if let Some(repo_root) = manifest_dir.parent().and_then(Path::parent) {
            candidates.push(repo_root.join("scripts/install-local-port.sh"));
        }
    }

    if let Some(path) = candidates.into_iter().find(|candidate| candidate.is_file()) {
        return Ok(path);
    }

    bail!("failed to resolve scripts/install-local-port.sh for Port source installs")
}

fn cache_root() -> Result<PathBuf> {
    if let Some(configured) = env::var_os("PORT_CACHE_ROOT") {
        return Ok(PathBuf::from(configured));
    }

    let home = env::var_os("HOME").context("set HOME or PORT_CACHE_ROOT so Port can resolve ~/.cache/port")?;
    Ok(PathBuf::from(home).join(".cache/port"))
}

fn ensure_supported_target(target: &str) -> Result<()> {
    if SUPPORTED_TARGETS.iter().any(|supported| supported == &target) {
        return Ok(());
    }

    bail!(
        "unsupported Port upgrade target '{target}'. Supported targets: {}",
        SUPPORTED_TARGETS.join(", ")
    )
}

fn run_command_capture(mut command: ProcessCommand, action: &str) -> Result<String> {
    let output = command
        .output()
        .with_context(|| format!("failed to {action}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let detail = if !stderr.is_empty() { stderr } else { stdout };
        if detail.is_empty() {
            bail!("{action} failed with status {}", output.status);
        }
        bail!("{action} failed: {detail}");
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn run_command_forwarding_output(mut command: ProcessCommand, action: &str) -> Result<()> {
    let output = command
        .output()
        .with_context(|| format!("failed to {action}"))?;
    if !output.stdout.is_empty() {
        print!("{}", String::from_utf8_lossy(&output.stdout));
    }
    if !output.stderr.is_empty() {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
    }
    if output.status.success() {
        return Ok(());
    }

    bail!("{action} failed with status {}", output.status)
}

fn select_toolchain(repo_dir: &Path) -> Result<ToolchainSelection> {
    let required = workspace_rust_version(repo_dir)?;
    let mut candidates = BTreeSet::new();

    if let Ok(explicit) = env::var("PORT_UPGRADE_TOOLCHAIN") {
        candidates.insert(explicit);
    }

    if command_exists("rustup") {
        for toolchain in rustup_toolchains()? {
            candidates.insert(toolchain);
        }
        candidates.insert(required.to_toolchain_string());
        candidates.insert(format!("{}.{}", required.major, required.minor));
        candidates.insert("stable".to_string());

        for toolchain in candidates {
            let selection = ToolchainSelection::Rustup(toolchain);
            if selection.rustc_version().is_ok_and(|version| version >= required) {
                return Ok(selection);
            }
        }
    }

    let plain = ToolchainSelection::Plain;
    if plain.rustc_version().is_ok_and(|version| version >= required) {
        return Ok(plain);
    }

    bail!(
        "no supported Rust toolchain found for Port source installs. Install Rust {} or newer, or set PORT_UPGRADE_TOOLCHAIN.",
        required
    )
}

fn command_exists(binary: &str) -> bool {
    env::var_os("PATH").is_some_and(|paths| {
        env::split_paths(&paths).any(|path| path.join(binary).is_file())
    })
}

fn rustup_toolchains() -> Result<Vec<String>> {
    let output = run_command_capture(
        {
            let mut command = ProcessCommand::new("rustup");
            command.args(["toolchain", "list"]);
            command
        },
        "list installed Rust toolchains",
    )?;

    Ok(output
        .lines()
        .filter_map(|line| line.split_whitespace().next())
        .filter(|entry| !entry.is_empty())
        .map(|entry| entry.to_string())
        .collect())
}

fn workspace_rust_version(repo_dir: &Path) -> Result<RustVersion> {
    let manifest_path = repo_dir.join("Cargo.toml");
    let manifest = fs::read_to_string(&manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;
    let workspace: WorkspaceManifest = toml::from_str(&manifest)
        .with_context(|| format!("failed to parse {}", manifest_path.display()))?;
    let rust_version = workspace
        .workspace
        .and_then(|workspace| workspace.package)
        .and_then(|package| package.rust_version)
        .context("workspace.package.rust-version is missing from Cargo.toml")?;
    RustVersion::parse(&rust_version)
}

#[derive(Debug, Clone, Copy)]
enum Revision<'a> {
    Tag(&'a str),
    Sha(&'a str),
}

#[derive(Debug, Clone)]
enum ToolchainSelection {
    Plain,
    Rustup(String),
}

impl ToolchainSelection {
    fn label(&self) -> &str {
        match self {
            Self::Plain => "default",
            Self::Rustup(toolchain) => toolchain,
        }
    }

    fn cargo_command(&self) -> ProcessCommand {
        match self {
            Self::Plain => ProcessCommand::new("cargo"),
            Self::Rustup(toolchain) => {
                let mut command = ProcessCommand::new("rustup");
                command.arg("run").arg(toolchain).arg("cargo");
                command
            }
        }
    }

    fn rustc_command(&self) -> ProcessCommand {
        match self {
            Self::Plain => ProcessCommand::new("rustc"),
            Self::Rustup(toolchain) => {
                let mut command = ProcessCommand::new("rustup");
                command.arg("run").arg(toolchain).arg("rustc");
                command
            }
        }
    }

    fn rustc_version(&self) -> Result<RustVersion> {
        let mut command = self.rustc_command();
        command.arg("-V");
        let output = run_command_capture(command, "inspect Rust toolchain version")?;
        let version = output
            .split_whitespace()
            .nth(1)
            .context("rustc -V did not report a version")?;
        RustVersion::parse(version)
    }

    fn host_target(&self) -> Result<String> {
        let mut command = self.rustc_command();
        command.arg("-vV");
        let output = run_command_capture(command, "inspect Rust host target")?;
        output
            .lines()
            .find_map(|line| line.strip_prefix("host: "))
            .map(|line| line.trim().to_string())
            .context("rustc -vV did not report a host target")
    }
}

#[derive(Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
struct RustVersion {
    major: u64,
    minor: u64,
    patch: u64,
}

impl RustVersion {
    fn parse(input: &str) -> Result<Self> {
        let cleaned = input
            .trim()
            .trim_start_matches('v')
            .split('-')
            .next()
            .unwrap_or(input)
            .trim();
        let mut parts = cleaned.split('.');
        let major = parts
            .next()
            .context("missing Rust major version")?
            .parse()
            .with_context(|| format!("invalid Rust version '{input}'"))?;
        let minor = parts
            .next()
            .unwrap_or("0")
            .parse()
            .with_context(|| format!("invalid Rust version '{input}'"))?;
        let patch = parts
            .next()
            .unwrap_or("0")
            .parse()
            .with_context(|| format!("invalid Rust version '{input}'"))?;
        Ok(Self {
            major,
            minor,
            patch,
        })
    }

    fn to_toolchain_string(self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl std::fmt::Display for RustVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[derive(Debug, Deserialize)]
struct WorkspaceManifest {
    workspace: Option<WorkspaceSection>,
}

#[derive(Debug, Deserialize)]
struct WorkspaceSection {
    package: Option<WorkspacePackage>,
}

#[derive(Debug, Deserialize)]
struct WorkspacePackage {
    #[serde(rename = "rust-version")]
    rust_version: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::RustVersion;

    #[test]
    fn parses_rust_versions_with_missing_patch() {
        let version = RustVersion::parse("1.85").expect("rust version should parse");
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 85);
        assert_eq!(version.patch, 0);
    }

    #[test]
    fn compares_rust_versions() {
        let minimum = RustVersion::parse("1.85").unwrap();
        let current = RustVersion::parse("1.85.1").unwrap();
        assert!(current >= minimum);
    }
}
