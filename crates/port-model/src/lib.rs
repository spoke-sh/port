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
                    path: PathBuf::from("artifacts/kernel/demo/vmlinux"),
                    build: String::from("port artifacts build --artifact demo-kernel"),
                    validate: String::from("port artifacts validate --artifact demo-kernel"),
                },
            )]),
            guest_images: BTreeMap::from([(
                String::from("demo-guest"),
                ArtifactSpec {
                    path: PathBuf::from("artifacts/guest/demo/rootfs.ext4"),
                    build: String::from("port artifacts build --artifact demo-guest"),
                    validate: String::from("port artifacts validate --artifact demo-guest"),
                },
            )]),
        };

        let hosts = BTreeMap::from([(
            String::from("local"),
            HostSpec {
                platform: HostPlatform::Linux,
                connection: HostConnection::Local,
                firecracker: FirecrackerSupport {
                    local_launch: true,
                    notes: vec![String::from("Requires /dev/kvm and the firecracker binary")],
                },
            },
        )]);

        let machines = BTreeMap::from([(
            String::from("demo"),
            MachineSpec {
                host: String::from("local"),
                kernel: String::from("demo-kernel"),
                guest_image: String::from("demo-guest"),
                vcpu_count: 2,
                memory_mib: 512,
                kernel_args: String::from(
                    "console=ttyS0 reboot=k panic=1 pci=off root=/dev/vda rw",
                ),
                rootfs_read_only: false,
                guest: GuestControl {
                    vsock_cid: 52,
                    control_port: 7000,
                    console_log: PathBuf::from("runtime/demo/console.log"),
                },
            },
        )]);

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
    pub path: PathBuf,
    pub build: String,
    pub validate: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HostPlatform {
    Linux,
    Macos,
    Windows,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostSpec {
    pub platform: HostPlatform,
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

#[cfg(test)]
mod tests {
    use super::PortConfig;

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
        assert!(encoded.contains("[machines.demo.guest]"));
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
}
