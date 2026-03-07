use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
#[command(
    name = "port-guest-agent",
    version,
    about = "Guest-side agent daemon for Port guest operations"
)]
struct Cli {
    #[arg(long, value_name = "PATH")]
    socket: PathBuf,

    #[arg(long, value_name = "PORT")]
    vsock_port: Option<u32>,

    #[arg(long, value_name = "PATH")]
    root: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    port_guest_agent::serve_with_vsock(&cli.socket, cli.root, cli.vsock_port)
}
