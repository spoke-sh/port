use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use port_model::PortConfig;
use serde::Serialize;

const EXAMPLES: &str = "\
Examples:
  port doctor
  port --config examples/port.toml artifacts build --artifact demo-kernel
  port --config examples/port.toml machine launch --machine demo
  port --config examples/port.toml guest exec --machine demo -- /bin/sh -lc 'uname -a'";

#[derive(Debug, Parser)]
#[command(
    name = "port",
    version,
    about = "CLI-first Firecracker orchestration for local and cloud Linux hosts",
    long_about = "Port manages Firecracker-backed Linux workloads through one canonical CLI and shared machine model.",
    after_help = EXAMPLES
)]
pub struct Cli {
    #[arg(
        long,
        global = true,
        value_name = "PATH",
        help = "Load a Port model from a TOML file instead of using the built-in sample"
    )]
    pub config: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    #[command(about = "Inspect platform support and host requirements")]
    Doctor {
        #[arg(long, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },
    #[command(subcommand, about = "Build and validate kernel or guest artifacts")]
    Artifacts(ArtifactCommand),
    #[command(subcommand, about = "Launch and inspect Port-managed machines")]
    Machine(MachineCommand),
    #[command(subcommand, about = "Reach guest agent capabilities")]
    Guest(GuestCommand),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text => f.write_str("text"),
            Self::Json => f.write_str("json"),
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum ArtifactCommand {
    #[command(about = "Build a named artifact from the model")]
    Build {
        #[arg(long)]
        artifact: String,
    },
    #[command(about = "Validate a named artifact from the model")]
    Validate {
        #[arg(long)]
        artifact: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum MachineCommand {
    #[command(about = "Launch a named machine from the model")]
    Launch {
        #[arg(long)]
        machine: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum GuestCommand {
    #[command(about = "Run a non-interactive command in the guest")]
    Exec {
        #[arg(long)]
        machine: String,
        #[arg(last = true, required = true)]
        command: Vec<String>,
    },
    #[command(about = "Copy files between host and guest")]
    Copy {
        #[arg(long)]
        machine: String,
        #[arg(long)]
        source: String,
        #[arg(long)]
        destination: String,
    },
    #[command(about = "Open an interactive PTY-backed session in the guest")]
    Pty {
        #[arg(long)]
        machine: String,
        #[arg(long, default_value = "/bin/sh")]
        command: String,
    },
    #[command(about = "Stream guest logs exposed by the agent")]
    Logs {
        #[arg(long)]
        machine: String,
        #[arg(long, default_value = "/var/log/port-agent.log")]
        path: String,
        #[arg(long)]
        follow: bool,
    },
    #[command(about = "Forward a local listener into the guest through the agent")]
    Forward {
        #[arg(long)]
        machine: String,
        #[arg(long)]
        listen: String,
        #[arg(long)]
        target: String,
    },
}

#[derive(Debug, Serialize)]
struct DoctorReport {
    host_os: String,
    local_firecracker_supported: bool,
    notes: Vec<String>,
}

pub fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Doctor { format } => doctor(format),
        Command::Artifacts(command) => {
            let config = load_config(cli.config)?;
            run_artifacts(command, &config)
        }
        Command::Machine(command) => {
            let config = load_config(cli.config)?;
            run_machine(command, &config)
        }
        Command::Guest(command) => {
            let config = load_config(cli.config)?;
            run_guest(command, &config)
        }
    }
}

fn doctor(format: OutputFormat) -> Result<()> {
    let host_os = std::env::consts::OS.to_string();
    let local_firecracker_supported = cfg!(target_os = "linux");
    let notes = if local_firecracker_supported {
        vec![
            String::from("Linux operators can target local Firecracker hosts."),
            String::from("macOS and Windows operators are expected to target remote Linux hosts."),
        ]
    } else {
        vec![String::from(
            "Local Firecracker launch is Linux-only; use a Linux host or WSL-backed workflow.",
        )]
    };

    let report = DoctorReport {
        host_os,
        local_firecracker_supported,
        notes,
    };

    match format {
        OutputFormat::Text => {
            println!("host_os: {}", report.host_os);
            println!(
                "local_firecracker_supported: {}",
                report.local_firecracker_supported
            );
            for note in report.notes {
                println!("note: {note}");
            }
        }
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&report).context("failed to encode doctor report")?
            );
        }
    }

    Ok(())
}

fn run_artifacts(command: ArtifactCommand, config: &PortConfig) -> Result<()> {
    match command {
        ArtifactCommand::Build { artifact } => {
            let spec = config
                .artifacts
                .kernels
                .get(&artifact)
                .or_else(|| config.artifacts.guest_images.get(&artifact))
                .with_context(|| format!("unknown artifact '{artifact}'"))?;
            println!("planned build command: {}", spec.build);
            println!("artifact path: {}", spec.path.display());
        }
        ArtifactCommand::Validate { artifact } => {
            let spec = config
                .artifacts
                .kernels
                .get(&artifact)
                .or_else(|| config.artifacts.guest_images.get(&artifact))
                .with_context(|| format!("unknown artifact '{artifact}'"))?;
            println!("planned validate command: {}", spec.validate);
            println!("artifact path: {}", spec.path.display());
        }
    }

    Ok(())
}

fn run_machine(command: MachineCommand, config: &PortConfig) -> Result<()> {
    match command {
        MachineCommand::Launch { machine } => {
            let spec = config
                .machines
                .get(&machine)
                .with_context(|| format!("unknown machine '{machine}'"))?;
            println!(
                "planned launch: machine={} host={} kernel={} guest_image={} vcpu={} memory_mib={} vsock={}:{}",
                machine,
                spec.host,
                spec.kernel,
                spec.guest_image,
                spec.vcpu_count,
                spec.memory_mib,
                spec.guest.vsock_cid,
                spec.guest.control_port,
            );
            println!("console log: {}", spec.guest.console_log.display());
        }
    }

    Ok(())
}

fn run_guest(command: GuestCommand, config: &PortConfig) -> Result<()> {
    match command {
        GuestCommand::Exec { machine, command } => {
            ensure_machine_exists(config, &machine)?;
            println!(
                "planned guest exec: machine={machine} command={}",
                command.join(" ")
            );
        }
        GuestCommand::Copy {
            machine,
            source,
            destination,
        } => {
            ensure_machine_exists(config, &machine)?;
            println!("planned guest copy: machine={machine} {source} -> {destination}");
        }
        GuestCommand::Pty { machine, command } => {
            ensure_machine_exists(config, &machine)?;
            println!("planned guest pty: machine={machine} command={command}");
        }
        GuestCommand::Logs {
            machine,
            path,
            follow,
        } => {
            ensure_machine_exists(config, &machine)?;
            println!("planned guest logs: machine={machine} path={path} follow={follow}");
        }
        GuestCommand::Forward {
            machine,
            listen,
            target,
        } => {
            ensure_machine_exists(config, &machine)?;
            println!("planned guest forward: machine={machine} listen={listen} target={target}");
        }
    }

    Ok(())
}

fn ensure_machine_exists(config: &PortConfig, machine: &str) -> Result<()> {
    if config.machines.contains_key(machine) {
        Ok(())
    } else {
        bail!("unknown machine '{machine}'")
    }
}

fn load_config(path: Option<PathBuf>) -> Result<PortConfig> {
    match path {
        Some(path) => PortConfig::from_path(&path)
            .with_context(|| format!("failed to load Port config from '{}'", path.display())),
        None => Ok(PortConfig::sample()),
    }
}

pub fn render_help() -> String {
    let mut command = Cli::command();
    command.render_long_help().to_string()
}

pub fn render_subcommand_help(name: &str) -> Option<String> {
    let mut command = Cli::command();
    let subcommand = command.find_subcommand_mut(name)?;
    Some(subcommand.render_long_help().to_string())
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::{Cli, Command, GuestCommand, MachineCommand, render_help, render_subcommand_help};

    #[test]
    fn help_includes_primary_surfaces() {
        let help = render_help();
        let guest_help = render_subcommand_help("guest").expect("guest help should exist");

        for keyword in ["doctor", "artifacts", "machine", "guest"] {
            assert!(help.contains(keyword), "missing help keyword: {keyword}");
        }

        for keyword in ["exec", "copy", "pty", "logs", "forward"] {
            assert!(
                guest_help.contains(keyword),
                "missing guest help keyword: {keyword}"
            );
        }
    }

    #[test]
    fn parses_machine_launch_arguments() {
        let cli = Cli::parse_from([
            "port",
            "--config",
            "examples/port.toml",
            "machine",
            "launch",
            "--machine",
            "demo",
        ]);

        match cli.command {
            Command::Machine(MachineCommand::Launch { machine }) => {
                assert_eq!(machine, "demo");
                assert_eq!(
                    cli.config.as_deref(),
                    Some(std::path::Path::new("examples/port.toml"))
                );
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn parses_guest_exec_trailing_command() {
        let cli = Cli::parse_from([
            "port",
            "guest",
            "exec",
            "--machine",
            "demo",
            "--",
            "/bin/sh",
            "-lc",
            "uname -a",
        ]);

        match cli.command {
            Command::Guest(GuestCommand::Exec { machine, command }) => {
                assert_eq!(machine, "demo");
                assert_eq!(command, ["/bin/sh", "-lc", "uname -a"]);
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }
}
