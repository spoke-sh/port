use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortConfig {
    pub artifacts: ArtifactCatalog,
    pub hosts: BTreeMap<String, HostSpec>,
    pub machines: BTreeMap<String, MachineSpec>,
}

impl PortConfig {
    #[must_use]
    pub fn sample() -> Self {
        let artifacts = ArtifactCatalog {
            kernels: BTreeMap::from([(
                String::from("demo-kernel"),
                ArtifactSpec {
                    reference: ArtifactReference {
                        registry: String::from("demo-fs"),
                        repository: String::from("port/demo-kernel"),
                        version: String::from("v1"),
                    },
                    build: String::from("port artifacts build --artifact demo-kernel"),
                    validate: String::from("port artifacts validate --artifact demo-kernel"),
                    distribution: ArtifactDistribution {
                        push: ArtifactStore::FileSystem {
                            root: PathBuf::from("artifact-store/demo-fs"),
                        },
                        pull: ArtifactStore::FileSystem {
                            root: PathBuf::from("artifact-store/demo-fs"),
                        },
                        cache_root: PathBuf::from(".port/cache"),
                    },
                    variants: vec![
                        sample_artifact_variant(
                            "artifacts/kernel/demo/x86_64/firecracker/standard/vmlinux",
                            MachineArchitecture::X86_64,
                            ExecutionSubstrate::Firecracker,
                            ProtectionMode::Standard,
                        ),
                        sample_artifact_variant(
                            "artifacts/kernel/demo/aarch64/firecracker/standard/vmlinux",
                            MachineArchitecture::Aarch64,
                            ExecutionSubstrate::Firecracker,
                            ProtectionMode::Standard,
                        ),
                    ],
                },
            )]),
            guest_images: BTreeMap::from([(
                String::from("demo-guest"),
                ArtifactSpec {
                    reference: ArtifactReference {
                        registry: String::from("demo-fs"),
                        repository: String::from("port/demo-guest"),
                        version: String::from("v1"),
                    },
                    build: String::from("port artifacts build --artifact demo-guest"),
                    validate: String::from("port artifacts validate --artifact demo-guest"),
                    distribution: ArtifactDistribution {
                        push: ArtifactStore::FileSystem {
                            root: PathBuf::from("artifact-store/demo-fs"),
                        },
                        pull: ArtifactStore::FileSystem {
                            root: PathBuf::from("artifact-store/demo-fs"),
                        },
                        cache_root: PathBuf::from(".port/cache"),
                    },
                    variants: vec![
                        sample_artifact_variant(
                            "artifacts/guest/demo/x86_64/firecracker/standard/rootfs.ext4",
                            MachineArchitecture::X86_64,
                            ExecutionSubstrate::Firecracker,
                            ProtectionMode::Standard,
                        ),
                        sample_artifact_variant(
                            "artifacts/guest/demo/aarch64/firecracker/standard/rootfs.ext4",
                            MachineArchitecture::Aarch64,
                            ExecutionSubstrate::Firecracker,
                            ProtectionMode::Standard,
                        ),
                    ],
                },
            )]),
        };

        let hosts = BTreeMap::from([
            (
                String::from("local"),
                HostSpec {
                    platform: HostPlatform::Linux,
                    provider: HostProvider::Local,
                    connection: HostConnection::Local,
                    firecracker: FirecrackerSupport {
                        local_launch: true,
                        notes: vec![String::from("Requires /dev/kvm and the firecracker binary")],
                    },
                },
            ),
            (
                String::from("generic-linux"),
                ssh_host(
                    HostProvider::GenericLinux,
                    "linux.example.internal",
                    "port",
                    vec![String::from(
                        "Remote Linux host reserved for the future cloud/control lane.",
                    )],
                ),
            ),
            (
                String::from("aws-linux"),
                ssh_host(
                    HostProvider::Aws,
                    "aws.example.internal",
                    "ec2-user",
                    vec![String::from(
                        "AWS is a justified future Firecracker provider lane for Port.",
                    )],
                ),
            ),
            (
                String::from("gcp-linux"),
                ssh_host(
                    HostProvider::Gcp,
                    "gcp.example.internal",
                    "port",
                    vec![String::from(
                        "GCP is a justified future Firecracker provider lane for Port.",
                    )],
                ),
            ),
            (
                String::from("azure-linux"),
                ssh_host(
                    HostProvider::Azure,
                    "azure.example.internal",
                    "port",
                    vec![String::from(
                        "Azure is modeled explicitly so diagnostics can report it as unsupported.",
                    )],
                ),
            ),
        ]);

        let machines = BTreeMap::from([
            (String::from("demo"), sample_machine("local", "demo", 52)),
            (
                String::from("cloud-generic"),
                sample_machine("generic-linux", "cloud-generic", 60),
            ),
            (
                String::from("cloud-aws"),
                sample_machine("aws-linux", "cloud-aws", 61),
            ),
            (
                String::from("cloud-gcp"),
                sample_machine("gcp-linux", "cloud-gcp", 62),
            ),
            (
                String::from("cloud-azure"),
                sample_machine("azure-linux", "cloud-azure", 63),
            ),
        ]);

        Self {
            artifacts,
            hosts,
            machines,
        }
    }

    pub fn from_toml_str(input: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(input)
    }

    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, ModelError> {
        let path = path.as_ref();
        let input = std::fs::read_to_string(path).map_err(|source| ModelError::Read {
            path: path.to_path_buf(),
            source,
        })?;

        Self::from_toml_str(&input).map_err(|source| ModelError::Parse {
            path: path.to_path_buf(),
            source,
        })
    }

    pub fn to_toml_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    pub fn artifact(&self, name: &str) -> Option<&ArtifactSpec> {
        self.artifacts.lookup(name)
    }

    pub fn machine_control_contract(
        &self,
        machine_name: &str,
    ) -> Result<MachineControlContract, ValidationError> {
        let machine = self
            .machines
            .get(machine_name)
            .ok_or_else(|| ValidationError::new(format!("unknown machine '{}'", machine_name)))?;
        let host = self.hosts.get(&machine.host).ok_or_else(|| {
            ValidationError::new(format!(
                "machine '{}' references unknown host '{}'",
                machine_name, machine.host
            ))
        })?;

        Ok(MachineControlContract::for_connection(&host.connection))
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        for (machine_name, machine) in &self.machines {
            let host = self.hosts.get(&machine.host).ok_or_else(|| {
                ValidationError::new(format!(
                    "machine '{}' references unknown host '{}'",
                    machine_name, machine.host
                ))
            })?;
            let kernel = self.artifact(&machine.kernel).ok_or_else(|| {
                ValidationError::new(format!(
                    "machine '{}' references unknown kernel artifact '{}'",
                    machine_name, machine.kernel
                ))
            })?;
            let guest_image = self.artifact(&machine.guest_image).ok_or_else(|| {
                ValidationError::new(format!(
                    "machine '{}' references unknown guest image artifact '{}'",
                    machine_name, machine.guest_image
                ))
            })?;
            let resolved_architecture = resolve_machine_architecture(machine.architecture)
                .map_err(|message| {
                    ValidationError::new(format!("machine '{}': {message}", machine_name))
                })?;

            let mut issues = Vec::new();
            match machine.substrate {
                ExecutionSubstrate::Firecracker => {
                    if host.platform != HostPlatform::Linux {
                        issues.push(String::from(
                            "Firecracker execution requires a Linux host platform.",
                        ));
                    }
                    if machine.protection_mode == ProtectionMode::Pvm
                        && resolved_architecture == MachineArchitecture::Aarch64
                    {
                        issues.push(String::from(
                            "Firecracker/PVM currently requires x86_64; arm64 remains a research lane.",
                        ));
                    }
                }
                ExecutionSubstrate::CloudHypervisor => {
                    if host.platform != HostPlatform::Linux {
                        issues.push(String::from(
                            "Cloud Hypervisor execution currently expects a Linux host platform.",
                        ));
                    }
                    if machine.protection_mode == ProtectionMode::Pvm {
                        issues.push(String::from(
                            "Port does not currently define a Cloud Hypervisor PVM lane.",
                        ));
                    }
                }
                ExecutionSubstrate::Avf => {
                    if host.platform != HostPlatform::Macos {
                        issues.push(String::from(
                            "Apple Virtualization Framework requires a macOS host platform.",
                        ));
                    }
                    if machine.protection_mode == ProtectionMode::Pvm {
                        issues.push(String::from(
                            "Apple Virtualization Framework does not currently define a PVM lane.",
                        ));
                    }
                }
            }

            if !kernel.supports(
                resolved_architecture,
                machine.substrate,
                machine.protection_mode,
            ) {
                issues.push(format!(
                    "Kernel artifact '{}' has no variant for {:?}/{:?}/{:?}.",
                    machine.kernel,
                    machine.substrate,
                    machine.protection_mode,
                    resolved_architecture
                ));
            }
            if !guest_image.supports(
                resolved_architecture,
                machine.substrate,
                machine.protection_mode,
            ) {
                issues.push(format!(
                    "Guest image artifact '{}' has no variant for {:?}/{:?}/{:?}.",
                    machine.guest_image,
                    machine.substrate,
                    machine.protection_mode,
                    resolved_architecture
                ));
            }

            validate_artifact_spec(machine_name, "kernel", &machine.kernel, kernel)
                .map_err(|message| ValidationError::new(message))?;
            validate_artifact_spec(
                machine_name,
                "guest image",
                &machine.guest_image,
                guest_image,
            )
            .map_err(|message| ValidationError::new(message))?;

            if !issues.is_empty() {
                return Err(ValidationError::new(format!(
                    "machine '{}': {}",
                    machine_name,
                    issues.join(" ")
                )));
            }
        }

        Ok(())
    }
}

fn sample_artifact_variant(
    path: &str,
    architecture: MachineArchitecture,
    substrate: ExecutionSubstrate,
    protection_mode: ProtectionMode,
) -> ArtifactVariant {
    ArtifactVariant {
        selector: ArtifactSelector {
            architecture,
            substrate,
            protection_mode,
        },
        path: PathBuf::from(path),
    }
}

fn ssh_host(provider: HostProvider, address: &str, user: &str, notes: Vec<String>) -> HostSpec {
    HostSpec {
        platform: HostPlatform::Linux,
        provider,
        connection: HostConnection::Ssh {
            address: address.to_string(),
            user: user.to_string(),
            port: 22,
        },
        firecracker: FirecrackerSupport {
            local_launch: false,
            notes,
        },
    }
}

fn sample_machine(host: &str, name: &str, vsock_cid: u32) -> MachineSpec {
    MachineSpec {
        host: host.to_string(),
        kernel: String::from("demo-kernel"),
        guest_image: String::from("demo-guest"),
        substrate: ExecutionSubstrate::Firecracker,
        protection_mode: ProtectionMode::Standard,
        architecture: MachineArchitecture::Native,
        vcpu_count: 2,
        memory_mib: 512,
        kernel_args: String::from("console=ttyS0 reboot=k panic=1 pci=off root=/dev/vda rw"),
        rootfs_read_only: false,
        guest: GuestControl {
            vsock_cid,
            control_port: 7000,
            console_log: PathBuf::from(format!("runtime/{name}/console.log")),
        },
    }
}

#[derive(Debug)]
pub enum ModelError {
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    Parse {
        path: PathBuf,
        source: toml::de::Error,
    },
}

impl std::fmt::Display for ModelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Read { path, source } => {
                write!(f, "failed to read config '{}': {source}", path.display())
            }
            Self::Parse { path, source } => {
                write!(f, "failed to parse config '{}': {source}", path.display())
            }
        }
    }
}

impl std::error::Error for ModelError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    message: String,
}

impl ValidationError {
    fn new(message: String) -> Self {
        Self { message }
    }
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ValidationError {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactCatalog {
    pub kernels: BTreeMap<String, ArtifactSpec>,
    pub guest_images: BTreeMap<String, ArtifactSpec>,
}

impl ArtifactCatalog {
    pub fn lookup(&self, name: &str) -> Option<&ArtifactSpec> {
        self.lookup_named(name).map(|(_, spec)| spec)
    }

    pub fn lookup_named(&self, name: &str) -> Option<(ArtifactKind, &ArtifactSpec)> {
        self.kernels
            .get(name)
            .map(|spec| (ArtifactKind::Kernel, spec))
            .or_else(|| {
                self.guest_images
                    .get(name)
                    .map(|spec| (ArtifactKind::GuestImage, spec))
            })
    }

    pub fn all(&self) -> impl Iterator<Item = (&str, &ArtifactSpec)> {
        self.kernels
            .iter()
            .chain(self.guest_images.iter())
            .map(|(name, spec)| (name.as_str(), spec))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactKind {
    Kernel,
    GuestImage,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactSpec {
    pub reference: ArtifactReference,
    pub build: String,
    pub validate: String,
    pub distribution: ArtifactDistribution,
    pub variants: Vec<ArtifactVariant>,
}

impl ArtifactSpec {
    #[must_use]
    pub fn supports(
        &self,
        architecture: MachineArchitecture,
        substrate: ExecutionSubstrate,
        protection_mode: ProtectionMode,
    ) -> bool {
        self.variant(architecture, substrate, protection_mode)
            .is_some()
    }

    #[must_use]
    pub fn variant(
        &self,
        architecture: MachineArchitecture,
        substrate: ExecutionSubstrate,
        protection_mode: ProtectionMode,
    ) -> Option<&ArtifactVariant> {
        self.variants.iter().find(|variant| {
            variant.selector.architecture == architecture
                && variant.selector.substrate == substrate
                && variant.selector.protection_mode == protection_mode
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactReference {
    pub registry: String,
    pub repository: String,
    pub version: String,
}

impl std::fmt::Display for ArtifactReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}:{}", self.registry, self.repository, self.version)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactDistribution {
    pub push: ArtifactStore,
    pub pull: ArtifactStore,
    pub cache_root: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "backend", rename_all = "kebab-case")]
pub enum ArtifactStore {
    FileSystem { root: PathBuf },
    OciRegistry { reference: String },
    HostedApi { endpoint: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactVariant {
    pub selector: ArtifactSelector,
    pub path: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactSelector {
    pub architecture: MachineArchitecture,
    pub substrate: ExecutionSubstrate,
    pub protection_mode: ProtectionMode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HostPlatform {
    Linux,
    Macos,
    Windows,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HostProvider {
    Local,
    GenericLinux,
    Aws,
    Gcp,
    Azure,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostSpec {
    pub platform: HostPlatform,
    pub provider: HostProvider,
    pub connection: HostConnection,
    pub firecracker: FirecrackerSupport,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "kebab-case")]
pub enum HostConnection {
    Local,
    Ssh {
        address: String,
        user: String,
        port: u16,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FirecrackerSupport {
    pub local_launch: bool,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MachineSpec {
    pub host: String,
    pub kernel: String,
    pub guest_image: String,
    pub substrate: ExecutionSubstrate,
    pub protection_mode: ProtectionMode,
    pub architecture: MachineArchitecture,
    pub vcpu_count: u8,
    pub memory_mib: u32,
    pub kernel_args: String,
    pub rootfs_read_only: bool,
    pub guest: GuestControl,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuestControl {
    pub vsock_cid: u32,
    pub control_port: u16,
    pub console_log: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MachineControlContract {
    pub inventory_scope: MachineInventoryScope,
    pub inventory_owner: MachineInventoryOwner,
    pub lifecycle_owner: MachineLifecycleOwner,
    pub guest_broker: MachineGuestBroker,
    pub status_source: MachineStatusSource,
    pub launch_route: MachineCommandRoute,
    pub inventory_route: MachineCommandRoute,
    pub status_route: MachineCommandRoute,
    pub stop_route: MachineCommandRoute,
    pub guest_route: MachineCommandRoute,
}

impl MachineControlContract {
    #[must_use]
    pub fn for_connection(connection: &HostConnection) -> Self {
        match connection {
            HostConnection::Local => Self::local_runtime_root(),
            HostConnection::Ssh { .. } => Self::hosted_control_plane(),
        }
    }

    #[must_use]
    pub fn local_runtime_root() -> Self {
        Self {
            inventory_scope: MachineInventoryScope::LocalRuntimeRoot,
            inventory_owner: MachineInventoryOwner::LocalRuntimeRoot,
            lifecycle_owner: MachineLifecycleOwner::LocalPortRuntime,
            guest_broker: MachineGuestBroker::LocalRuntimeTransport,
            status_source: MachineStatusSource::RuntimeManifestAndHostProcess,
            launch_route: MachineCommandRoute::DirectLocalRuntime,
            inventory_route: MachineCommandRoute::DirectLocalRuntime,
            status_route: MachineCommandRoute::DirectLocalRuntime,
            stop_route: MachineCommandRoute::DirectLocalRuntime,
            guest_route: MachineCommandRoute::DirectLocalRuntime,
        }
    }

    #[must_use]
    pub fn hosted_control_plane() -> Self {
        Self {
            inventory_scope: MachineInventoryScope::HostedFleet,
            inventory_owner: MachineInventoryOwner::HostedControlPlane,
            lifecycle_owner: MachineLifecycleOwner::HostedNodeAgent,
            guest_broker: MachineGuestBroker::ControlPlaneNodeAgentTunnel,
            status_source: MachineStatusSource::ControlPlaneInventoryAndNodeAgentRuntime,
            launch_route: MachineCommandRoute::HostedControlPlane,
            inventory_route: MachineCommandRoute::HostedControlPlane,
            status_route: MachineCommandRoute::HostedControlPlane,
            stop_route: MachineCommandRoute::HostedControlPlane,
            guest_route: MachineCommandRoute::HostedControlPlane,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MachineInventoryScope {
    LocalRuntimeRoot,
    HostedFleet,
}

impl std::fmt::Display for MachineInventoryScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            Self::LocalRuntimeRoot => "local-runtime-root",
            Self::HostedFleet => "hosted-fleet",
        };
        f.write_str(label)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MachineInventoryOwner {
    LocalRuntimeRoot,
    HostedControlPlane,
}

impl std::fmt::Display for MachineInventoryOwner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            Self::LocalRuntimeRoot => "local-runtime-root",
            Self::HostedControlPlane => "hosted-control-plane",
        };
        f.write_str(label)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MachineLifecycleOwner {
    LocalPortRuntime,
    HostedNodeAgent,
}

impl std::fmt::Display for MachineLifecycleOwner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            Self::LocalPortRuntime => "local-port-runtime",
            Self::HostedNodeAgent => "hosted-node-agent",
        };
        f.write_str(label)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MachineGuestBroker {
    LocalRuntimeTransport,
    ControlPlaneNodeAgentTunnel,
}

impl std::fmt::Display for MachineGuestBroker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            Self::LocalRuntimeTransport => "local-runtime-transport",
            Self::ControlPlaneNodeAgentTunnel => "control-plane-node-agent-tunnel",
        };
        f.write_str(label)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MachineStatusSource {
    RuntimeManifestAndHostProcess,
    ControlPlaneInventoryAndNodeAgentRuntime,
}

impl std::fmt::Display for MachineStatusSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            Self::RuntimeManifestAndHostProcess => "runtime-manifest-and-host-process",
            Self::ControlPlaneInventoryAndNodeAgentRuntime => {
                "control-plane-inventory-and-node-agent-runtime"
            }
        };
        f.write_str(label)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MachineCommandRoute {
    DirectLocalRuntime,
    HostedControlPlane,
}

impl std::fmt::Display for MachineCommandRoute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            Self::DirectLocalRuntime => "direct-local-runtime",
            Self::HostedControlPlane => "hosted-control-plane",
        };
        f.write_str(label)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExecutionSubstrate {
    Firecracker,
    CloudHypervisor,
    Avf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProtectionMode {
    Standard,
    Pvm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MachineArchitecture {
    Native,
    X86_64,
    Aarch64,
}

fn resolve_machine_architecture(
    architecture: MachineArchitecture,
) -> Result<MachineArchitecture, &'static str> {
    match architecture {
        MachineArchitecture::Native => match std::env::consts::ARCH {
            "x86_64" => Ok(MachineArchitecture::X86_64),
            "aarch64" => Ok(MachineArchitecture::Aarch64),
            _ => Err("host architecture is not yet modeled by Port"),
        },
        concrete => Ok(concrete),
    }
}

fn validate_artifact_spec(
    machine_name: &str,
    artifact_kind: &str,
    artifact_name: &str,
    artifact: &ArtifactSpec,
) -> Result<(), String> {
    if artifact.variants.is_empty() {
        return Err(format!(
            "machine '{}': {} artifact '{}' does not declare any variants",
            machine_name, artifact_kind, artifact_name
        ));
    }

    let mut seen = Vec::new();
    for variant in &artifact.variants {
        if seen.contains(&variant.selector) {
            return Err(format!(
                "machine '{}': {} artifact '{}' declares duplicate variant {:?}/{:?}/{:?}",
                machine_name,
                artifact_kind,
                artifact_name,
                variant.selector.architecture,
                variant.selector.substrate,
                variant.selector.protection_mode
            ));
        }
        seen.push(variant.selector);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        ArtifactStore, ExecutionSubstrate, HostProvider, MachineArchitecture, MachineCommandRoute,
        MachineControlContract, MachineGuestBroker, MachineInventoryOwner, MachineInventoryScope,
        MachineLifecycleOwner, MachineStatusSource, PortConfig, ProtectionMode,
    };

    #[test]
    fn sample_config_round_trips_through_toml() {
        let sample = PortConfig::sample();
        let encoded = sample.to_toml_string().expect("sample should encode");
        let decoded = PortConfig::from_toml_str(&encoded).expect("sample should decode");

        assert_eq!(decoded, sample);
    }

    #[test]
    fn sample_config_exposes_expected_sections() {
        let encoded = PortConfig::sample()
            .to_toml_string()
            .expect("sample should encode");

        assert!(encoded.contains("[artifacts.kernels.demo-kernel]"));
        assert!(encoded.contains("[hosts.local]"));
        assert!(encoded.contains("provider = \"local\""));
        assert!(encoded.contains("provider = \"generic-linux\""));
        assert!(encoded.contains("provider = \"aws\""));
        assert!(encoded.contains("provider = \"gcp\""));
        assert!(encoded.contains("provider = \"azure\""));
        assert!(encoded.contains("substrate = \"firecracker\""));
        assert!(encoded.contains("protection_mode = \"standard\""));
        assert!(encoded.contains("architecture = \"native\""));
        assert!(encoded.contains("[machines.demo.guest]"));
        assert!(encoded.contains("[machines.cloud-aws]"));
        assert!(encoded.contains("[artifacts.kernels.demo-kernel.reference]"));
        assert!(encoded.contains("[artifacts.kernels.demo-kernel.distribution.push]"));
        assert!(encoded.contains("[artifacts.kernels.demo-kernel.variants]"));
    }

    #[test]
    fn artifact_catalog_reports_kernel_and_guest_image_kinds() {
        let config = PortConfig::sample();

        let (kernel_kind, _) = config
            .artifacts
            .lookup_named("demo-kernel")
            .expect("kernel artifact should exist");
        let (guest_kind, _) = config
            .artifacts
            .lookup_named("demo-guest")
            .expect("guest image artifact should exist");

        assert_eq!(kernel_kind, super::ArtifactKind::Kernel);
        assert_eq!(guest_kind, super::ArtifactKind::GuestImage);
    }

    #[test]
    fn sample_config_models_all_remote_provider_lanes() {
        let config = PortConfig::sample();

        assert_eq!(config.hosts["local"].provider, HostProvider::Local);
        assert_eq!(
            config.hosts["generic-linux"].provider,
            HostProvider::GenericLinux
        );
        assert_eq!(config.hosts["aws-linux"].provider, HostProvider::Aws);
        assert_eq!(config.hosts["gcp-linux"].provider, HostProvider::Gcp);
        assert_eq!(config.hosts["azure-linux"].provider, HostProvider::Azure);
        assert_eq!(config.machines["cloud-aws"].host, "aws-linux");
        assert_eq!(
            config.machines["demo"].substrate,
            ExecutionSubstrate::Firecracker
        );
        assert_eq!(
            config.machines["demo"].protection_mode,
            ProtectionMode::Standard
        );
        assert_eq!(
            config.machines["demo"].architecture,
            MachineArchitecture::Native
        );
        assert_eq!(
            config.artifacts.kernels["demo-kernel"]
                .reference
                .to_string(),
            "demo-fs/port/demo-kernel:v1"
        );
    }

    #[test]
    fn sample_config_derives_local_machine_control_contract() {
        let config = PortConfig::sample();

        let contract = config
            .machine_control_contract("demo")
            .expect("demo contract should resolve");

        assert_eq!(contract, MachineControlContract::local_runtime_root());
        assert_eq!(
            contract.inventory_scope,
            MachineInventoryScope::LocalRuntimeRoot
        );
        assert_eq!(
            contract.inventory_owner,
            MachineInventoryOwner::LocalRuntimeRoot
        );
        assert_eq!(
            contract.lifecycle_owner,
            MachineLifecycleOwner::LocalPortRuntime
        );
        assert_eq!(
            contract.guest_broker,
            MachineGuestBroker::LocalRuntimeTransport
        );
        assert_eq!(
            contract.status_source,
            MachineStatusSource::RuntimeManifestAndHostProcess
        );
        assert_eq!(
            contract.status_route,
            MachineCommandRoute::DirectLocalRuntime
        );
    }

    #[test]
    fn sample_config_derives_hosted_machine_control_contract() {
        let config = PortConfig::sample();

        let contract = config
            .machine_control_contract("cloud-aws")
            .expect("cloud contract should resolve");

        assert_eq!(contract, MachineControlContract::hosted_control_plane());
        assert_eq!(contract.inventory_scope, MachineInventoryScope::HostedFleet);
        assert_eq!(
            contract.inventory_owner,
            MachineInventoryOwner::HostedControlPlane
        );
        assert_eq!(
            contract.lifecycle_owner,
            MachineLifecycleOwner::HostedNodeAgent
        );
        assert_eq!(
            contract.guest_broker,
            MachineGuestBroker::ControlPlaneNodeAgentTunnel
        );
        assert_eq!(
            contract.status_source,
            MachineStatusSource::ControlPlaneInventoryAndNodeAgentRuntime
        );
        assert_eq!(
            contract.status_route,
            MachineCommandRoute::HostedControlPlane
        );
    }

    #[test]
    fn checked_in_example_models_all_provider_variants() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/port.toml")
            .canonicalize()
            .expect("example config path should resolve");
        let config = PortConfig::from_path(&path).expect("example config should parse");

        assert_eq!(config.hosts["local"].provider, HostProvider::Local);
        assert_eq!(
            config.hosts["generic-linux"].provider,
            HostProvider::GenericLinux
        );
        assert_eq!(config.hosts["aws-linux"].provider, HostProvider::Aws);
        assert_eq!(config.hosts["gcp-linux"].provider, HostProvider::Gcp);
        assert_eq!(config.hosts["azure-linux"].provider, HostProvider::Azure);
        assert_eq!(config.machines["cloud-azure"].host, "azure-linux");
        assert_eq!(
            config.machines["demo"].substrate,
            ExecutionSubstrate::Firecracker
        );
        assert_eq!(
            config.machines["demo"].protection_mode,
            ProtectionMode::Standard
        );
        assert_eq!(
            config.artifacts.kernels["demo-kernel"].variants[0]
                .selector
                .architecture,
            MachineArchitecture::X86_64
        );
        assert_eq!(
            config.artifacts.guest_images["demo-guest"].variants[0]
                .selector
                .substrate,
            ExecutionSubstrate::Firecracker
        );
    }

    #[test]
    fn artifact_compatibility_rejects_unsupported_pvm_lane() {
        let config = PortConfig::sample();
        let guest = &config.artifacts.guest_images["demo-guest"];

        assert!(guest.supports(
            MachineArchitecture::X86_64,
            ExecutionSubstrate::Firecracker,
            ProtectionMode::Standard
        ));
        assert!(!guest.supports(
            MachineArchitecture::X86_64,
            ExecutionSubstrate::Firecracker,
            ProtectionMode::Pvm
        ));
    }

    #[test]
    fn artifact_variants_cover_file_store_distribution_and_resolution() {
        let config = PortConfig::sample();
        let kernel = &config.artifacts.kernels["demo-kernel"];

        assert!(matches!(
            kernel.distribution.push,
            ArtifactStore::FileSystem { .. }
        ));
        assert!(kernel
            .variant(
                MachineArchitecture::Aarch64,
                ExecutionSubstrate::Firecracker,
                ProtectionMode::Standard
            )
            .is_some());
        assert!(kernel
            .variant(
                MachineArchitecture::X86_64,
                ExecutionSubstrate::Firecracker,
                ProtectionMode::Pvm
            )
            .is_none());
    }
}
