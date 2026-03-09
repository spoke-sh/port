use port_agent_protocol::ExecRequest;
use port_model::PortConfig;
use port_sdk::{HostedClient, ServiceApplyRequest, ServiceKind, ServiceSecretBinding};

fn main() -> anyhow::Result<()> {
    let config = PortConfig::sample();
    let client = HostedClient::from_machine(&config, "cloud-aws", "demo-token")?;

    let machine = client.machines().monitor("cloud-aws");
    let exec = client.guest().exec(
        "cloud-aws",
        ExecRequest {
            command: vec![String::from("/bin/echo"), String::from("hello")],
            cwd: None,
            env: Default::default(),
        },
    )?;
    let service = client.services().apply(
        "cloud-aws",
        ServiceApplyRequest {
            name: String::from("buildbox"),
            kind: ServiceKind::Sandbox,
            host_group: Some(String::from("aws-builders")),
            command: vec![
                String::from("/bin/sh"),
                String::from("-lc"),
                String::from("make test"),
            ],
            secret_bindings: vec![ServiceSecretBinding {
                env: String::from("API_TOKEN"),
                secret: String::from("demo-token"),
            }],
        },
    )?;

    println!("machine monitor url: {}", machine.url);
    println!("guest exec url: {}", exec.url);
    println!("service apply url: {}", service.url);
    Ok(())
}
