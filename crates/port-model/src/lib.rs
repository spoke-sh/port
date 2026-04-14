use std::collections::{BTreeMap, BTreeSet, btree_map::Entry};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortConfig {
    pub artifacts: ArtifactCatalog,
    #[serde(default)]
    pub control_planes: BTreeMap<String, HostedControlPlaneSpec>,
    pub hosts: BTreeMap<String, HostSpec>,
    #[serde(default)]
    pub nodes: BTreeMap<String, HostedNodeSpec>,
    #[serde(default)]
    pub host_groups: BTreeMap<String, HostedHostGroupSpec>,
    pub machines: BTreeMap<String, MachineSpec>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub clusters: BTreeMap<String, ClusterSpec>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub k3s_clusters: BTreeMap<String, K3sClusterSpec>,
    #[serde(skip)]
    state_root: Option<PathBuf>,
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
                            "artifacts/kernel/demo/x86_64/firecracker/pvm/vmlinux",
                            MachineArchitecture::X86_64,
                            ExecutionSubstrate::Firecracker,
                            ProtectionMode::Pvm,
                        ),
                        sample_artifact_variant(
                            "artifacts/kernel/demo/aarch64/firecracker/standard/vmlinux",
                            MachineArchitecture::Aarch64,
                            ExecutionSubstrate::Firecracker,
                            ProtectionMode::Standard,
                        ),
                        sample_artifact_variant(
                            "artifacts/kernel/demo/x86_64/cloud-hypervisor/standard/vmlinux",
                            MachineArchitecture::X86_64,
                            ExecutionSubstrate::CloudHypervisor,
                            ProtectionMode::Standard,
                        ),
                        sample_artifact_variant(
                            "artifacts/kernel/demo/aarch64/cloud-hypervisor/standard/vmlinux",
                            MachineArchitecture::Aarch64,
                            ExecutionSubstrate::CloudHypervisor,
                            ProtectionMode::Standard,
                        ),
                        sample_artifact_variant(
                            "artifacts/kernel/demo/x86_64/avf/standard/vmlinux",
                            MachineArchitecture::X86_64,
                            ExecutionSubstrate::Avf,
                            ProtectionMode::Standard,
                        ),
                        sample_artifact_variant(
                            "artifacts/kernel/demo/aarch64/avf/standard/vmlinux",
                            MachineArchitecture::Aarch64,
                            ExecutionSubstrate::Avf,
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
                            "artifacts/guest/demo/x86_64/firecracker/pvm/rootfs.ext4",
                            MachineArchitecture::X86_64,
                            ExecutionSubstrate::Firecracker,
                            ProtectionMode::Pvm,
                        ),
                        sample_artifact_variant(
                            "artifacts/guest/demo/aarch64/firecracker/standard/rootfs.ext4",
                            MachineArchitecture::Aarch64,
                            ExecutionSubstrate::Firecracker,
                            ProtectionMode::Standard,
                        ),
                        sample_artifact_variant(
                            "artifacts/guest/demo/x86_64/cloud-hypervisor/standard/rootfs.ext4",
                            MachineArchitecture::X86_64,
                            ExecutionSubstrate::CloudHypervisor,
                            ProtectionMode::Standard,
                        ),
                        sample_artifact_variant(
                            "artifacts/guest/demo/aarch64/cloud-hypervisor/standard/rootfs.ext4",
                            MachineArchitecture::Aarch64,
                            ExecutionSubstrate::CloudHypervisor,
                            ProtectionMode::Standard,
                        ),
                        sample_artifact_variant(
                            "artifacts/guest/demo/x86_64/avf/standard/rootfs.ext4",
                            MachineArchitecture::X86_64,
                            ExecutionSubstrate::Avf,
                            ProtectionMode::Standard,
                        ),
                        sample_artifact_variant(
                            "artifacts/guest/demo/aarch64/avf/standard/rootfs.ext4",
                            MachineArchitecture::Aarch64,
                            ExecutionSubstrate::Avf,
                            ProtectionMode::Standard,
                        ),
                    ],
                },
            )]),
        };

        let control_planes = BTreeMap::from([(
            String::from("demo"),
            HostedControlPlaneSpec {
                endpoint: String::from("https://port.example.internal"),
                audience: String::from("port-hosted-demo"),
                auth: HostedAuthTokenContract {
                    scheme: HostedAuthScheme::Bearer,
                    header: String::from("authorization"),
                    source: HostedAuthTokenSource::Env {
                        variable: String::from("PORT_DEMO_TOKEN"),
                    },
                },
            },
        )]);

        let hosts = BTreeMap::from([
            (
                String::from("local"),
                HostSpec {
                    platform: HostPlatform::Linux,
                    provider: HostProvider::Local,
                    connection: HostConnection::Local,
                    firecracker: FirecrackerSupport {
                        local_launch: true,
                        pvm_lanes: firecracker_pvm_lanes(),
                        notes: vec![String::from("Requires /dev/kvm and the firecracker binary")],
                    },
                },
            ),
            (
                String::from("generic-linux"),
                hosted_host(
                    HostProvider::GenericLinux,
                    "demo",
                    vec![String::from(
                        "Remote Linux host is modeled through the demo hosted control plane contract.",
                    )],
                ),
            ),
            (
                String::from("aws-linux"),
                hosted_host(
                    HostProvider::Aws,
                    "demo",
                    vec![String::from(
                        "AWS is a justified future Firecracker provider lane and is modeled through the demo hosted control plane contract.",
                    )],
                ),
            ),
            (
                String::from("gcp-linux"),
                hosted_host(
                    HostProvider::Gcp,
                    "demo",
                    vec![String::from(
                        "GCP is a justified future Firecracker provider lane and is modeled through the demo hosted control plane contract.",
                    )],
                ),
            ),
            (
                String::from("azure-linux"),
                hosted_host(
                    HostProvider::Azure,
                    "demo",
                    vec![String::from(
                        "Azure is modeled explicitly through the demo hosted control plane so diagnostics can report it as unsupported.",
                    )],
                ),
            ),
            (
                String::from("mac-local"),
                HostSpec {
                    platform: HostPlatform::Macos,
                    provider: HostProvider::Local,
                    connection: HostConnection::Local,
                    firecracker: FirecrackerSupport {
                        local_launch: false,
                        pvm_lanes: Vec::new(),
                        notes: vec![String::from(
                            "AVF local execution is modeled separately from Firecracker.",
                        )],
                    },
                },
            ),
        ]);

        let nodes = BTreeMap::from([
            (
                String::from("generic-linux-node"),
                HostedNodeSpec {
                    host: String::from("generic-linux"),
                    runtime_root: PathBuf::from("runtime/hosted/generic-linux-node"),
                    capabilities: HostedNodeCapabilities {
                        providers: vec![HostProvider::GenericLinux],
                        platforms: vec![HostPlatform::Linux],
                        substrates: vec![ExecutionSubstrate::Firecracker],
                        architectures: vec![MachineArchitecture::X86_64],
                        protection_modes: vec![ProtectionMode::Standard, ProtectionMode::Pvm],
                        pvm_lanes: vec![HostedPvmCapability {
                            architecture: MachineArchitecture::X86_64,
                            state: PvmCapabilityState::Planned,
                            host_kit: None,
                            notes: vec![String::from(
                                "Generic Linux is modeled as PVM-planned but not host-kit-ready in the sample inventory.",
                            )],
                        }],
                    },
                    notes: vec![String::from(
                        "Generic Linux is the baseline hosted node contract before scheduler policy exists.",
                    )],
                },
            ),
            (
                String::from("aws-linux-node"),
                HostedNodeSpec {
                    host: String::from("aws-linux"),
                    runtime_root: PathBuf::from("runtime/hosted/aws-linux-node"),
                    capabilities: HostedNodeCapabilities {
                        providers: vec![HostProvider::Aws],
                        platforms: vec![HostPlatform::Linux],
                        substrates: vec![ExecutionSubstrate::Firecracker],
                        architectures: vec![MachineArchitecture::X86_64],
                        protection_modes: vec![ProtectionMode::Standard, ProtectionMode::Pvm],
                        pvm_lanes: vec![HostedPvmCapability {
                            architecture: MachineArchitecture::X86_64,
                            state: PvmCapabilityState::Planned,
                            host_kit: Some(x86_64_firecracker_pvm_host_kit()),
                            notes: vec![String::from(
                                "AWS declares the sample x86_64 PVM host-kit contract and becomes ready through imported preparation.",
                            )],
                        }],
                    },
                    notes: vec![String::from(
                        "AWS stays explicit because later host-group and PVM planning will care about provider identity.",
                    )],
                },
            ),
            (
                String::from("gcp-linux-node"),
                HostedNodeSpec {
                    host: String::from("gcp-linux"),
                    runtime_root: PathBuf::from("runtime/hosted/gcp-linux-node"),
                    capabilities: HostedNodeCapabilities {
                        providers: vec![HostProvider::Gcp],
                        platforms: vec![HostPlatform::Linux],
                        substrates: vec![ExecutionSubstrate::Firecracker],
                        architectures: vec![MachineArchitecture::X86_64],
                        protection_modes: vec![ProtectionMode::Standard, ProtectionMode::Pvm],
                        pvm_lanes: vec![HostedPvmCapability {
                            architecture: MachineArchitecture::X86_64,
                            state: PvmCapabilityState::Planned,
                            host_kit: None,
                            notes: vec![String::from(
                                "GCP remains modeled as a planned PVM node until a prepared host kit is explicitly advertised.",
                            )],
                        }],
                    },
                    notes: vec![String::from(
                        "GCP is modeled as a hosted node so placement and lifecycle work can remain provider-aware.",
                    )],
                },
            ),
        ]);

        let host_groups = BTreeMap::from([
            (
                String::from("remote-linux"),
                HostedHostGroupSpec {
                    placement: HostedPlacementPolicy::ExplicitMembership,
                    scheduler: HostedSchedulerPolicy::DeterministicFirstFit,
                    nodes: vec![
                        String::from("generic-linux-node"),
                        String::from("aws-linux-node"),
                        String::from("gcp-linux-node"),
                    ],
                    notes: vec![String::from(
                        "This group is the first hosted placement boundary; later scheduler, monitoring, and services work should reuse it instead of inventing another inventory axis.",
                    )],
                },
            ),
            (
                String::from("aws-builders"),
                HostedHostGroupSpec {
                    placement: HostedPlacementPolicy::ExplicitMembership,
                    scheduler: HostedSchedulerPolicy::DeterministicFirstFit,
                    nodes: vec![String::from("aws-linux-node")],
                    notes: vec![String::from(
                        "Provider-specific groups stay explicit so later scheduling and service placement can target them without creating a second host taxonomy.",
                    )],
                },
            ),
        ]);

        let machines = BTreeMap::from([
            (String::from("demo"), sample_machine("local", "demo", 52)),
            (
                String::from("demo-ch"),
                sample_cloud_hypervisor_machine("local", "demo-ch", 54),
            ),
            (
                String::from("demo-avf"),
                sample_avf_machine("mac-local", "demo-avf", 53),
            ),
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

        let clusters = BTreeMap::from([(
            String::from("demo"),
            ClusterSpec {
                flavor: ClusterFlavor::K3s,
                provider: ClusterProvider::Local,
                count: 1,
                machine: String::from("demo"),
                version: None,
                args: vec![String::from("--disable=traefik")],
                bootstrap: ClusterBootstrapSpec {
                    stage_root: PathBuf::from("/opt/port/clusters/demo"),
                    install_script: PathBuf::from(
                        "examples/bootstrap/demo-k3s/install-k3s-offline.sh",
                    ),
                    binary: PathBuf::from("examples/bootstrap/demo-k3s/k3s"),
                    guest_profile: ClusterGuestProfileSpec {
                        name: String::from("kube-ready"),
                        required_commands: vec![
                            String::from("sh"),
                            String::from("install"),
                            String::from("ln"),
                            String::from("chmod"),
                            String::from("dirname"),
                            String::from("setsid"),
                            String::from("modprobe"),
                        ],
                    },
                },
                lifecycle: ClusterLifecycleSpec {
                    health_command: vec![
                        String::from("opt/port/clusters/demo/bin/k3s"),
                        String::from("kubectl"),
                        String::from("get"),
                        String::from("nodes"),
                        String::from("-o"),
                        String::from("wide"),
                        String::from("--request-timeout=15s"),
                    ],
                    kubeconfig_path: PathBuf::from("/etc/rancher/k3s/k3s.yaml"),
                    api_forward_target: String::from("127.0.0.1:6443"),
                    forwards: Vec::new(),
                },
            },
        )]);

        Self {
            artifacts,
            control_planes,
            hosts,
            nodes,
            host_groups,
            machines,
            clusters,
            k3s_clusters: BTreeMap::new(),
            state_root: None,
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

        let mut config = Self::from_toml_str(&input).map_err(|source| ModelError::Parse {
            path: path.to_path_buf(),
            source,
        })?;
        config.set_state_root(path.parent().unwrap_or_else(|| Path::new(".")));
        Ok(config)
    }

    pub fn to_toml_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    pub fn state_root(&self) -> Option<&Path> {
        self.state_root.as_deref()
    }

    pub fn set_state_root(&mut self, path: impl Into<PathBuf>) {
        self.state_root = Some(path.into());
    }

    #[must_use]
    pub fn with_state_root(mut self, path: impl Into<PathBuf>) -> Self {
        self.set_state_root(path);
        self
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

    pub fn hosted_api_identity_contract(
        &self,
        machine_name: &str,
    ) -> Result<Option<HostedApiIdentityContract>, ValidationError> {
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

        match &host.connection {
            HostConnection::Local => Ok(None),
            HostConnection::HostedControlPlane { control_plane } => {
                let spec = self.control_planes.get(control_plane).ok_or_else(|| {
                    ValidationError::new(format!(
                        "host '{}' references unknown control plane '{}'",
                        machine.host, control_plane
                    ))
                })?;
                Ok(Some(HostedApiIdentityContract {
                    control_plane: control_plane.clone(),
                    endpoint: spec.endpoint.clone(),
                    audience: spec.audience.clone(),
                    auth: spec.auth.clone(),
                    route: MachineCommandRoute::HostedControlPlane,
                }))
            }
            HostConnection::Ssh { .. } => Ok(None),
        }
    }

    pub fn hosted_artifact_identity_contract(
        &self,
        endpoint: &str,
    ) -> Result<HostedArtifactIdentityContract, ValidationError> {
        let endpoint = endpoint.trim();
        if endpoint.is_empty() {
            return Err(ValidationError::new(String::from(
                "hosted artifact backend must declare a non-empty endpoint",
            )));
        }

        let mut matches = self
            .control_planes
            .iter()
            .filter_map(|(name, spec)| (spec.endpoint == endpoint).then_some((name, spec)));
        let (control_plane, spec) = matches.next().ok_or_else(|| {
            ValidationError::new(format!(
                "no control plane is configured for hosted artifact endpoint '{}'",
                endpoint
            ))
        })?;
        if let Some((other, _)) = matches.next() {
            return Err(ValidationError::new(format!(
                "hosted artifact endpoint '{}' is ambiguous across control planes '{}' and '{}'",
                endpoint, control_plane, other
            )));
        }

        Ok(HostedArtifactIdentityContract {
            control_plane: control_plane.clone(),
            endpoint: spec.endpoint.clone(),
            audience: spec.audience.clone(),
            auth: spec.auth.clone(),
        })
    }

    pub fn hosted_inventory_contract(&self) -> Result<HostedInventoryContract, ValidationError> {
        let mut nodes = BTreeMap::new();
        for (node_name, node) in &self.nodes {
            let host = self.hosts.get(&node.host).ok_or_else(|| {
                ValidationError::new(format!(
                    "node '{}' references unknown host '{}'",
                    node_name, node.host
                ))
            })?;
            let control_plane = match &host.connection {
                HostConnection::Local => {
                    return Err(ValidationError::new(format!(
                        "node '{}' references local host '{}' but hosted nodes must target a hosted control plane",
                        node_name, node.host
                    )));
                }
                HostConnection::HostedControlPlane { control_plane } => control_plane.clone(),
                HostConnection::Ssh { .. } => {
                    return Err(ValidationError::new(format!(
                        "node '{}' references ssh-managed host '{}' but hosted nodes must target a hosted control plane",
                        node_name, node.host
                    )));
                }
            };
            nodes.insert(
                node_name.clone(),
                HostedNodeContract {
                    host: node.host.clone(),
                    runtime_root: node.runtime_root.clone(),
                    control_plane,
                    inventory_owner: MachineInventoryOwner::HostedControlPlane,
                    lifecycle_owner: MachineLifecycleOwner::HostedNodeAgent,
                    capabilities: node.capabilities.clone(),
                    notes: node.notes.clone(),
                },
            );
        }

        let mut host_groups = BTreeMap::new();
        for (group_name, group) in &self.host_groups {
            let mut members = Vec::new();
            let mut control_plane = None::<String>;
            for node_name in &group.nodes {
                let node = nodes.get(node_name).ok_or_else(|| {
                    ValidationError::new(format!(
                        "host group '{}' references unknown node '{}'",
                        group_name, node_name
                    ))
                })?;
                if let Some(current) = &control_plane {
                    if current != &node.control_plane {
                        return Err(ValidationError::new(format!(
                            "host group '{}' mixes nodes from control planes '{}' and '{}'",
                            group_name, current, node.control_plane
                        )));
                    }
                } else {
                    control_plane = Some(node.control_plane.clone());
                }
                members.push(node_name.clone());
            }
            host_groups.insert(
                group_name.clone(),
                HostedHostGroupContract {
                    control_plane: control_plane.ok_or_else(|| {
                        ValidationError::new(format!(
                            "host group '{}' must contain at least one node",
                            group_name
                        ))
                    })?,
                    inventory_owner: MachineInventoryOwner::HostedControlPlane,
                    placement: group.placement,
                    scheduler: group.scheduler,
                    nodes: members,
                    notes: group.notes.clone(),
                },
            );
        }

        Ok(HostedInventoryContract { nodes, host_groups })
    }

    pub fn hosted_machine_summary_contract(
        &self,
        machine_name: &str,
    ) -> Result<Option<HostedMachineSummaryContract>, ValidationError> {
        let control = self.machine_control_contract(machine_name)?;
        if control.inventory_scope != MachineInventoryScope::HostedFleet {
            return Ok(None);
        }

        let machine = self
            .machines
            .get(machine_name)
            .ok_or_else(|| ValidationError::new(format!("unknown machine '{}'", machine_name)))?;
        let hosted_identity = self
            .hosted_api_identity_contract(machine_name)?
            .ok_or_else(|| {
                ValidationError::new(format!(
                    "machine '{}' does not resolve to a hosted control plane",
                    machine_name
                ))
            })?;
        let inventory = self.hosted_inventory_contract()?;
        let host_nodes = inventory
            .nodes
            .iter()
            .filter(|(_, node)| node.host == machine.host)
            .collect::<Vec<_>>();
        if host_nodes.is_empty() {
            return Err(ValidationError::new(format!(
                "machine '{}' targets hosted host '{}' but no hosted node inventory record matches that host",
                machine_name, machine.host
            )));
        }
        let mut candidate_nodes = Vec::new();
        let mut rejected_nodes = BTreeMap::new();
        let host_node_names = host_nodes
            .iter()
            .map(|(node_name, _)| (*node_name).clone())
            .collect::<Vec<_>>();
        for (node_name, node) in host_nodes {
            if let Some(reason) =
                hosted_node_rejection_reason(machine_name, node_name, machine, node)?
            {
                rejected_nodes.insert(node_name.clone(), reason);
            } else {
                candidate_nodes.push(node_name.clone());
            }
        }

        let host_groups = inventory
            .host_groups
            .iter()
            .filter(|(_, group)| {
                group
                    .nodes
                    .iter()
                    .any(|node| host_node_names.contains(node))
            })
            .map(|(group_name, _)| group_name.clone())
            .collect::<Vec<_>>();
        let host_group_policies = inventory
            .host_groups
            .iter()
            .filter(|(group_name, _)| host_groups.contains(group_name))
            .map(|(group_name, group)| (group_name.clone(), group.scheduler))
            .collect::<BTreeMap<_, _>>();
        let host = self.hosts.get(&machine.host).ok_or_else(|| {
            ValidationError::new(format!(
                "machine '{}' references unknown host '{}'",
                machine_name, machine.host
            ))
        })?;
        let placement_detail = hosted_placement_detail(
            machine_name,
            machine,
            &machine.host,
            host.provider,
            &candidate_nodes,
            &rejected_nodes,
        )?;

        Ok(Some(HostedMachineSummaryContract {
            machine_name: machine_name.to_string(),
            host_name: machine.host.clone(),
            provider: host.provider,
            control_plane: hosted_identity.control_plane,
            candidate_nodes,
            rejected_nodes: rejected_nodes.clone(),
            host_groups,
            host_group_policies,
            placement_detail,
            control,
        }))
    }

    pub fn hosted_machine_status_contract(
        &self,
        machine_name: &str,
    ) -> Result<Option<HostedMachineStatusContract>, ValidationError> {
        let summary = match self.hosted_machine_summary_contract(machine_name)? {
            Some(summary) => summary,
            None => return Ok(None),
        };
        Ok(Some(HostedMachineStatusContract {
            machine: summary.clone(),
            status_source: summary.control.status_source,
            status_route: summary.control.status_route,
            detail: String::from(
                "Hosted machine status is modeled as control-plane inventory plus node-agent runtime state; the canonical `port machine status` verb remains the future surface.",
            ),
        }))
    }

    pub fn hosted_machine_stop_contract(
        &self,
        machine_name: &str,
    ) -> Result<Option<HostedMachineStopContract>, ValidationError> {
        let summary = match self.hosted_machine_summary_contract(machine_name)? {
            Some(summary) => summary,
            None => return Ok(None),
        };
        Ok(Some(HostedMachineStopContract {
            machine: summary.clone(),
            lifecycle_owner: summary.control.lifecycle_owner,
            stop_route: summary.control.stop_route,
            detail: String::from(
                "Hosted machine stop remains routed through the control plane, with the node agent owning the host-local stop action once the runtime path exists.",
            ),
        }))
    }

    pub fn hosted_machine_monitor_contract(
        &self,
        machine_name: &str,
    ) -> Result<Option<HostedMachineMonitorContract>, ValidationError> {
        let summary = match self.hosted_machine_summary_contract(machine_name)? {
            Some(summary) => summary,
            None => return Ok(None),
        };
        Ok(Some(HostedMachineMonitorContract {
            machine: summary.clone(),
            lifecycle_owner: summary.control.lifecycle_owner,
            status_source: summary.control.status_source,
            monitor_route: summary.control.monitor_route,
            top_route: summary.control.top_route,
            detail: String::from(
                "Hosted monitoring and top remain routed through the control plane, with the node agent owning host-local runtime inspection and detached forward state.",
            ),
        }))
    }

    pub fn hosted_service_contract(
        &self,
        machine_name: &str,
    ) -> Result<Option<HostedServiceContract>, ValidationError> {
        let summary = match self.hosted_machine_summary_contract(machine_name)? {
            Some(summary) => summary,
            None => return Ok(None),
        };
        Ok(Some(HostedServiceContract {
            machine: summary.clone(),
            lifecycle_owner: summary.control.lifecycle_owner,
            guest_broker: summary.control.guest_broker,
            service_route: summary.control.service_route,
            detail: String::from(
                "Hosted secrets, services, and sandboxes stay on the canonical service surface and reuse the same control-plane plus node-agent runtime ownership as machine and guest operations.",
            ),
        }))
    }

    pub fn hosted_guest_attach_contract(
        &self,
        machine_name: &str,
    ) -> Result<Option<HostedGuestAttachContract>, ValidationError> {
        let summary = match self.hosted_machine_summary_contract(machine_name)? {
            Some(summary) => summary,
            None => return Ok(None),
        };
        Ok(Some(HostedGuestAttachContract {
            machine: summary.clone(),
            guest_broker: summary.control.guest_broker,
            guest_route: summary.control.guest_route,
            command_surface: vec![
                GuestCommandVerb::Exec,
                GuestCommandVerb::Copy,
                GuestCommandVerb::Pty,
                GuestCommandVerb::Logs,
                GuestCommandVerb::Forward,
            ],
            protocol: HostedGuestProtocolContract::PortAgentProtocol,
            attach_path: vec![
                HostedGuestAttachHop {
                    actor: HostedGuestAttachActor::Cli,
                    role: String::from("initiates a canonical `port guest ...` request"),
                },
                HostedGuestAttachHop {
                    actor: HostedGuestAttachActor::HostedControlPlane,
                    role: String::from("authorizes guest attachment and resolves the owning node"),
                },
                HostedGuestAttachHop {
                    actor: HostedGuestAttachActor::HostedNodeAgent,
                    role: String::from(
                        "opens the host-local guest transport and bridges the byte stream",
                    ),
                },
                HostedGuestAttachHop {
                    actor: HostedGuestAttachActor::GuestAgent,
                    role: String::from(
                        "continues serving the existing guest request and response frames",
                    ),
                },
            ],
            detail: String::from(
                "Hosted guest attach preserves the canonical `port guest exec|copy|pty|logs|forward` surface. The control plane authorizes the attach, the node agent bridges the host-local guest transport, and the existing Port guest protocol frames continue unchanged to the in-guest agent.",
            ),
        }))
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        let mut endpoint_owners = BTreeMap::new();
        for (control_plane_name, control_plane) in &self.control_planes {
            validate_hosted_control_plane(control_plane_name, control_plane)?;
            let endpoint = control_plane.endpoint.trim().to_string();
            match endpoint_owners.entry(endpoint) {
                Entry::Vacant(slot) => {
                    slot.insert(control_plane_name);
                }
                Entry::Occupied(existing) => {
                    return Err(ValidationError::new(format!(
                        "duplicate hosted control-plane endpoint '{}' is configured for '{}' and '{}'",
                        existing.key(),
                        existing.get(),
                        control_plane_name
                    )));
                }
            }
        }
        for (host_name, host) in &self.hosts {
            validate_host(host_name, host)?;
        }
        for (node_name, node) in &self.nodes {
            validate_hosted_node(self, node_name, node)?;
        }
        for (group_name, group) in &self.host_groups {
            validate_hosted_host_group(self, group_name, group)?;
        }
        for (artifact_name, artifact) in self.artifacts.all() {
            validate_artifact_distribution(artifact_name, artifact)?;
        }

        for (machine_name, machine) in &self.machines {
            let host = self.hosts.get(&machine.host).ok_or_else(|| {
                ValidationError::new(format!(
                    "machine '{}' references unknown host '{}'",
                    machine_name, machine.host
                ))
            })?;
            if let HostConnection::HostedControlPlane { control_plane } = &host.connection {
                if !self.control_planes.contains_key(control_plane) {
                    return Err(ValidationError::new(format!(
                        "machine '{}' references host '{}' which references unknown control plane '{}'",
                        machine_name, machine.host, control_plane
                    )));
                }
            }
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
                    if !matches!(host.connection, HostConnection::Local) {
                        issues.push(String::from(
                            "AVF local runtime currently requires a local host connection.",
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
                .map_err(ValidationError::new)?;
            validate_artifact_spec(
                machine_name,
                "guest image",
                &machine.guest_image,
                guest_image,
            )
            .map_err(ValidationError::new)?;
            validate_machine_rootfs_overlay(machine_name, machine, resolved_architecture)?;
            validate_machine_runtime_class(machine_name, machine)?;
            validate_machine_volumes(machine_name, machine)?;
            validate_machine_volume_lane(machine_name, &machine.host, host, machine)?;

            if !issues.is_empty() {
                return Err(ValidationError::new(format!(
                    "machine '{}': {}",
                    machine_name,
                    issues.join(" ")
                )));
            }
        }
        for (cluster_name, cluster) in &self.clusters {
            validate_cluster(self, cluster_name, cluster)?;
        }
        for (cluster_name, cluster) in &self.k3s_clusters {
            validate_k3s_cluster(self, cluster_name, cluster)?;
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

const fn default_ssh_port() -> u16 {
    22
}

fn hosted_host(provider: HostProvider, control_plane: &str, notes: Vec<String>) -> HostSpec {
    HostSpec {
        platform: HostPlatform::Linux,
        provider,
        connection: HostConnection::HostedControlPlane {
            control_plane: control_plane.to_string(),
        },
        firecracker: FirecrackerSupport {
            local_launch: false,
            pvm_lanes: Vec::new(),
            notes,
        },
    }
}

fn firecracker_pvm_lanes() -> Vec<FirecrackerPvmLaneContract> {
    vec![
        FirecrackerPvmLaneContract::for_architecture(MachineArchitecture::X86_64),
        FirecrackerPvmLaneContract::for_architecture(MachineArchitecture::Aarch64),
    ]
}

fn x86_64_firecracker_pvm_host_kit() -> PvmHostKit {
    PvmHostKit {
        package: PvmHostKitPackage {
            name: String::from("firecracker-pvm-host-kit"),
            version: String::from("2026.04"),
            host_kernel_release: String::from("6.12.0-port-pvm"),
            firecracker_build: String::from("v1.13.0-dev+loopholelabs.pvm.7f6c070fa09c"),
        },
        host_platform: HostPlatform::Linux,
        host_architecture: MachineArchitecture::X86_64,
        requires_custom_host_kernel: true,
        requires_patched_firecracker: true,
        firecracker_binary_name: String::from("firecracker-pvm"),
        firecracker_binary_env: Some(String::from("PORT_PVM_FIRECRACKER_BINARY")),
        host_boot_args: vec![String::from("pti=off")],
        notes: vec![
            String::from(
                "The host kernel must carry the Firecracker/PVM-capable KVM changes rather than stock KVM alone.",
            ),
            String::from(
                "The VMM binary must be a PVM-capable Firecracker build, not the current standard lane binary.",
            ),
        ],
    }
}

fn x86_64_firecracker_pvm_artifact_kit() -> PvmArtifactKit {
    PvmArtifactKit {
        kernel_selector: ArtifactSelector {
            architecture: MachineArchitecture::X86_64,
            substrate: ExecutionSubstrate::Firecracker,
            protection_mode: ProtectionMode::Pvm,
        },
        guest_image_selector: ArtifactSelector {
            architecture: MachineArchitecture::X86_64,
            substrate: ExecutionSubstrate::Firecracker,
            protection_mode: ProtectionMode::Pvm,
        },
        requires_dedicated_variants: true,
        notes: vec![
            String::from(
                "PVM guests require dedicated kernel and guest-image variants; standard Firecracker artifacts are insufficient.",
            ),
            String::from(
                "The guest image must boot with the guest-side PVM expectations rather than the current standard guest contract.",
            ),
        ],
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
        memory_mib: 2048,
        kernel_args: String::from("console=ttyS0 reboot=k panic=1 pci=off root=/dev/vda rw"),
        rootfs_read_only: false,
        rootfs_overlay: None,
        runtime_class: None,
        volumes: Vec::new(),
        guest: GuestControl {
            vsock_cid,
            control_port: 7000,
            console_log: PathBuf::from(format!("runtime/{name}/console.log")),
        },
        network: Some(MachineNetworkSpec {
            enabled: false,
            ..MachineNetworkSpec::default()
        }),
    }
}

fn sample_avf_machine(host: &str, name: &str, vsock_cid: u32) -> MachineSpec {
    MachineSpec {
        host: host.to_string(),
        substrate: ExecutionSubstrate::Avf,
        ..sample_machine(host, name, vsock_cid)
    }
}

fn sample_cloud_hypervisor_machine(host: &str, name: &str, vsock_cid: u32) -> MachineSpec {
    MachineSpec {
        host: host.to_string(),
        substrate: ExecutionSubstrate::CloudHypervisor,
        ..sample_machine(host, name, vsock_cid)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
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

impl ArtifactReference {
    #[must_use]
    pub fn oci_remote_reference(&self, selector: ArtifactSelector) -> String {
        format!(
            "{}/{}:{}-{}-{}-{}",
            self.registry,
            self.repository,
            self.version,
            machine_architecture_label(selector.architecture),
            execution_substrate_label(selector.substrate),
            protection_mode_label(selector.protection_mode)
        )
    }
}

pub fn hosted_artifact_store_path(
    control_plane: &str,
    reference: &ArtifactReference,
    selector: ArtifactSelector,
    filename: impl AsRef<Path>,
) -> PathBuf {
    PathBuf::from(".port")
        .join("hosted")
        .join(control_plane)
        .join("artifacts")
        .join(&reference.registry)
        .join(&reference.repository)
        .join(&reference.version)
        .join(machine_architecture_label(selector.architecture))
        .join(execution_substrate_label(selector.substrate))
        .join(protection_mode_label(selector.protection_mode))
        .join(filename.as_ref())
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
    FileSystem {
        root: PathBuf,
    },
    OciRegistry {
        transport: OciRegistryTransport,
        auth: OciRegistryAuth,
    },
    HostedApi {
        endpoint: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OciRegistryTransport {
    Https,
    PlainHttp,
}

impl OciRegistryTransport {
    #[must_use]
    pub fn describe(&self) -> &'static str {
        match self {
            Self::Https => "https",
            Self::PlainHttp => "plain-http",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum OciRegistryAuth {
    Anonymous,
    BasicEnv {
        username_variable: String,
        password_variable: String,
    },
}

impl OciRegistryAuth {
    #[must_use]
    pub fn describe(&self) -> String {
        match self {
            Self::Anonymous => String::from("anonymous"),
            Self::BasicEnv {
                username_variable,
                password_variable,
            } => format!("basic-env:{username_variable}:{password_variable}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostedControlPlaneSpec {
    pub endpoint: String,
    pub audience: String,
    pub auth: HostedAuthTokenContract,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostedAuthTokenContract {
    pub scheme: HostedAuthScheme,
    pub header: String,
    pub source: HostedAuthTokenSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HostedAuthScheme {
    Bearer,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum HostedAuthTokenSource {
    Env { variable: String },
}

impl HostedAuthTokenSource {
    #[must_use]
    pub fn describe(&self) -> String {
        match self {
            Self::Env { variable } => format!("env:{variable}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostedApiIdentityContract {
    pub control_plane: String,
    pub endpoint: String,
    pub audience: String,
    pub auth: HostedAuthTokenContract,
    pub route: MachineCommandRoute,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostedArtifactIdentityContract {
    pub control_plane: String,
    pub endpoint: String,
    pub audience: String,
    pub auth: HostedAuthTokenContract,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostedNodeSpec {
    pub host: String,
    pub runtime_root: PathBuf,
    pub capabilities: HostedNodeCapabilities,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostedNodeCapabilities {
    pub providers: Vec<HostProvider>,
    pub platforms: Vec<HostPlatform>,
    pub substrates: Vec<ExecutionSubstrate>,
    pub architectures: Vec<MachineArchitecture>,
    pub protection_modes: Vec<ProtectionMode>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pvm_lanes: Vec<HostedPvmCapability>,
}

impl HostedNodeCapabilities {
    #[must_use]
    pub fn pvm_lane_for(&self, architecture: MachineArchitecture) -> Option<&HostedPvmCapability> {
        let architecture = match architecture {
            MachineArchitecture::Native => resolve_native_pvm_architecture(),
            other => other,
        };
        self.pvm_lanes
            .iter()
            .find(|lane| lane.architecture == architecture)
    }

    #[must_use]
    pub fn without_imported_pvm_readiness(&self) -> Self {
        let mut next = self.clone();
        for lane in &mut next.pvm_lanes {
            if lane.state == PvmCapabilityState::Ready {
                lane.state = PvmCapabilityState::Planned;
            }
        }
        next
    }

    #[must_use]
    pub fn is_populated(&self) -> bool {
        !self.providers.is_empty()
            && !self.platforms.is_empty()
            && !self.substrates.is_empty()
            && !self.architectures.is_empty()
            && !self.protection_modes.is_empty()
    }

    #[must_use]
    pub fn is_subset_of(&self, configured: &Self) -> bool {
        self.providers
            .iter()
            .all(|provider| configured.providers.contains(provider))
            && self
                .platforms
                .iter()
                .all(|platform| configured.platforms.contains(platform))
            && self
                .substrates
                .iter()
                .all(|substrate| configured.substrates.contains(substrate))
            && self
                .architectures
                .iter()
                .all(|architecture| configured.architectures.contains(architecture))
            && self
                .protection_modes
                .iter()
                .all(|mode| configured.protection_modes.contains(mode))
            && self
                .pvm_lanes
                .iter()
                .all(|lane| configured.pvm_lanes.contains(lane))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostedPvmCapability {
    pub architecture: MachineArchitecture,
    pub state: PvmCapabilityState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host_kit: Option<PvmHostKit>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostedPvmHostKitPackageAttachment {
    pub architecture: MachineArchitecture,
    pub package: PvmHostKitPackage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PvmCapabilityState {
    Ready,
    Planned,
    ResearchOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostedHostGroupSpec {
    pub placement: HostedPlacementPolicy,
    pub scheduler: HostedSchedulerPolicy,
    pub nodes: Vec<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HostedPlacementPolicy {
    ExplicitMembership,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HostedSchedulerPolicy {
    DeterministicFirstFit,
    Spread,
}

impl std::fmt::Display for HostedSchedulerPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DeterministicFirstFit => f.write_str("deterministic-first-fit"),
            Self::Spread => f.write_str("spread"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostedInventoryContract {
    pub nodes: BTreeMap<String, HostedNodeContract>,
    pub host_groups: BTreeMap<String, HostedHostGroupContract>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostedImportedNodeRecord {
    pub provider: HostProvider,
    pub provenance: String,
    pub imported_at: u64,
    pub capability_summary: HostedNodeCapabilities,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pvm_host_kit_packages: Vec<HostedPvmHostKitPackageAttachment>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostedNodeRegistration {
    pub endpoint: String,
    pub token: String,
    pub registered_at: u64,
    pub refreshed_at: u64,
    pub ttl_seconds: u64,
}

impl HostedNodeRegistration {
    pub fn freshness(
        &self,
        control_plane: &str,
        node_name: &str,
    ) -> Result<HostedNodeFreshnessContract, ValidationError> {
        if self.endpoint.trim().is_empty() {
            return Err(ValidationError::new(format!(
                "registered node '{}' for control plane '{}' must declare a non-empty endpoint",
                node_name, control_plane
            )));
        }
        if self.token.trim().is_empty() {
            return Err(ValidationError::new(format!(
                "registered node '{}' for control plane '{}' must declare a non-empty token",
                node_name, control_plane
            )));
        }
        if self.ttl_seconds == 0 {
            return Err(ValidationError::new(format!(
                "registered node '{}' for control plane '{}' must declare a ttl_seconds greater than zero",
                node_name, control_plane
            )));
        }
        if self.refreshed_at < self.registered_at {
            return Err(ValidationError::new(format!(
                "registered node '{}' for control plane '{}' cannot refresh before its initial registration ({} < {})",
                node_name, control_plane, self.refreshed_at, self.registered_at
            )));
        }
        let fresh_until = self
            .refreshed_at
            .checked_add(self.ttl_seconds)
            .ok_or_else(|| {
                ValidationError::new(format!(
                    "registered node '{}' for control plane '{}' overflowed its freshness window",
                    node_name, control_plane
                ))
            })?;
        Ok(HostedNodeFreshnessContract {
            registered_at: self.registered_at,
            refreshed_at: self.refreshed_at,
            ttl_seconds: self.ttl_seconds,
            fresh_until,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostedNodeFreshnessContract {
    pub registered_at: u64,
    pub refreshed_at: u64,
    pub ttl_seconds: u64,
    pub fresh_until: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostedRegisteredNodeContract {
    pub node_name: String,
    pub endpoint: String,
    pub freshness: HostedNodeFreshnessContract,
    pub node: HostedNodeContract,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub host_groups: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostedNodeContract {
    pub host: String,
    pub runtime_root: PathBuf,
    pub control_plane: String,
    pub inventory_owner: MachineInventoryOwner,
    pub lifecycle_owner: MachineLifecycleOwner,
    pub capabilities: HostedNodeCapabilities,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostedHostGroupContract {
    pub control_plane: String,
    pub inventory_owner: MachineInventoryOwner,
    pub placement: HostedPlacementPolicy,
    pub scheduler: HostedSchedulerPolicy,
    pub nodes: Vec<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostedMachineSummaryContract {
    pub machine_name: String,
    pub host_name: String,
    pub provider: HostProvider,
    pub control_plane: String,
    pub candidate_nodes: Vec<String>,
    pub rejected_nodes: BTreeMap<String, String>,
    pub host_groups: Vec<String>,
    pub host_group_policies: BTreeMap<String, HostedSchedulerPolicy>,
    pub placement_detail: String,
    pub control: MachineControlContract,
}

impl HostedInventoryContract {
    pub fn hosted_registered_node_contract(
        &self,
        control_plane: &str,
        node_name: &str,
        registration: &HostedNodeRegistration,
    ) -> Result<HostedRegisteredNodeContract, ValidationError> {
        let node = self.nodes.get(node_name).ok_or_else(|| {
            ValidationError::new(format!(
                "registered node '{}' does not exist in hosted inventory for control plane '{}'",
                node_name, control_plane
            ))
        })?;
        if node.control_plane != control_plane {
            return Err(ValidationError::new(format!(
                "registered node '{}' belongs to control plane '{}', not '{}'",
                node_name, node.control_plane, control_plane
            )));
        }
        let freshness = registration.freshness(control_plane, node_name)?;
        let host_groups = self
            .host_groups
            .iter()
            .filter(|(_, group)| group.nodes.iter().any(|member| member == node_name))
            .map(|(group_name, _)| group_name.clone())
            .collect();
        Ok(HostedRegisteredNodeContract {
            node_name: node_name.to_string(),
            endpoint: registration.endpoint.trim().to_string(),
            freshness,
            node: node.clone(),
            host_groups,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostedMachineStatusContract {
    pub machine: HostedMachineSummaryContract,
    pub status_source: MachineStatusSource,
    pub status_route: MachineCommandRoute,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostedMachineStopContract {
    pub machine: HostedMachineSummaryContract,
    pub lifecycle_owner: MachineLifecycleOwner,
    pub stop_route: MachineCommandRoute,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostedMachineMonitorContract {
    pub machine: HostedMachineSummaryContract,
    pub lifecycle_owner: MachineLifecycleOwner,
    pub status_source: MachineStatusSource,
    pub monitor_route: MachineCommandRoute,
    pub top_route: MachineCommandRoute,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostedServiceContract {
    pub machine: HostedMachineSummaryContract,
    pub lifecycle_owner: MachineLifecycleOwner,
    pub guest_broker: MachineGuestBroker,
    pub service_route: MachineCommandRoute,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostedGuestAttachContract {
    pub machine: HostedMachineSummaryContract,
    pub guest_broker: MachineGuestBroker,
    pub guest_route: MachineCommandRoute,
    pub command_surface: Vec<GuestCommandVerb>,
    pub protocol: HostedGuestProtocolContract,
    pub attach_path: Vec<HostedGuestAttachHop>,
    pub detail: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GuestCommandVerb {
    Exec,
    Copy,
    Pty,
    Logs,
    Forward,
}

impl std::fmt::Display for GuestCommandVerb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            Self::Exec => "exec",
            Self::Copy => "copy",
            Self::Pty => "pty",
            Self::Logs => "logs",
            Self::Forward => "forward",
        };
        f.write_str(label)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HostedGuestProtocolContract {
    PortAgentProtocol,
}

impl std::fmt::Display for HostedGuestProtocolContract {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            Self::PortAgentProtocol => "port-agent-protocol",
        };
        f.write_str(label)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostedGuestAttachHop {
    pub actor: HostedGuestAttachActor,
    pub role: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostedGuestAttachActor {
    Cli,
    HostedControlPlane,
    HostedNodeAgent,
    GuestAgent,
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
    HostedControlPlane {
        control_plane: String,
    },
    Ssh {
        destination: String,
        user: String,
        #[serde(default = "default_ssh_port")]
        port: u16,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FirecrackerSupport {
    pub local_launch: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pvm_lanes: Vec<FirecrackerPvmLaneContract>,
    pub notes: Vec<String>,
}

impl FirecrackerSupport {
    #[must_use]
    pub fn pvm_lane_for(
        &self,
        architecture: MachineArchitecture,
    ) -> Option<&FirecrackerPvmLaneContract> {
        let architecture = match architecture {
            MachineArchitecture::Native => resolve_native_pvm_architecture(),
            other => other,
        };
        self.pvm_lanes
            .iter()
            .find(|lane| lane.architecture == architecture)
    }
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rootfs_overlay: Option<MachineRootfsOverlaySpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_class: Option<MachineRuntimeClassSpec>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub volumes: Vec<MachineVolumeSpec>,
    pub guest: GuestControl,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network: Option<MachineNetworkSpec>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MachineRuntimeClassSpec {
    pub kind: MachineRuntimeClassKind,
    pub trust: MachineRuntimeTrustPosture,
    pub state_isolation: MachineRuntimeStateIsolation,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub writable_roots: Vec<MachineRuntimeWritableRoot>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub declared_inputs: Vec<MachineRuntimeDeclaredInput>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace: Option<MachineRuntimeWorkspaceBinding>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MachineRuntimeClassKind {
    WorkspaceScratchBuilder,
    BlessedClosurePromotionRunner,
}

impl std::fmt::Display for MachineRuntimeClassKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WorkspaceScratchBuilder => f.write_str("workspace-scratch-builder"),
            Self::BlessedClosurePromotionRunner => f.write_str("blessed-closure-promotion-runner"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MachineRuntimeTrustPosture {
    WorkspaceUntrusted,
    PromotionTrusted,
}

impl std::fmt::Display for MachineRuntimeTrustPosture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WorkspaceUntrusted => f.write_str("workspace-untrusted"),
            Self::PromotionTrusted => f.write_str("promotion-trusted"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MachineRuntimeStateIsolation {
    WorkspaceWritable,
    CleanRoom,
}

impl std::fmt::Display for MachineRuntimeStateIsolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WorkspaceWritable => f.write_str("workspace-writable"),
            Self::CleanRoom => f.write_str("clean-room"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MachineRuntimeWritableRoot {
    NixStore,
    SourceRoot,
    TempRoot,
    EvidenceRoot,
}

impl std::fmt::Display for MachineRuntimeWritableRoot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(runtime_writable_root_label(*self))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MachineRuntimeDeclaredInput {
    SourceBundle,
    RequestedOutputs,
    PolicySnapshot,
    CandidateClosure,
}

impl std::fmt::Display for MachineRuntimeDeclaredInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(runtime_declared_input_label(*self))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MachineRuntimeWorkspaceBinding {
    pub workspace: String,
    pub lane: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MachineRootfsOverlaySpec {
    pub size_mib: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MachineVolumeBackend {
    HostFile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MachineVolumePersistence {
    Persistent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MachineVolumeSpec {
    pub name: String,
    pub backend: MachineVolumeBackend,
    pub persistence: MachineVolumePersistence,
    pub path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuestControl {
    pub vsock_cid: u32,
    pub control_port: u16,
    pub console_log: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MachineNetworkSpec {
    #[serde(default = "default_network_enabled")]
    pub enabled: bool,
    #[serde(default = "default_guest_ip")]
    pub guest_ip: String,
    #[serde(default = "default_host_ip")]
    pub host_ip: String,
    #[serde(default = "default_network_prefix_len")]
    pub prefix_len: u8,
    #[serde(default = "default_guest_mac")]
    pub guest_mac: String,
    #[serde(default = "default_dns_servers", skip_serializing_if = "Vec::is_empty")]
    pub dns_servers: Vec<String>,
}

impl Default for MachineNetworkSpec {
    fn default() -> Self {
        Self {
            enabled: default_network_enabled(),
            guest_ip: default_guest_ip(),
            host_ip: default_host_ip(),
            prefix_len: default_network_prefix_len(),
            guest_mac: default_guest_mac(),
            dns_servers: default_dns_servers(),
        }
    }
}

const fn default_network_enabled() -> bool {
    true
}
fn default_guest_ip() -> String {
    String::from("172.16.0.2")
}
fn default_host_ip() -> String {
    String::from("172.16.0.1")
}
const fn default_network_prefix_len() -> u8 {
    24
}
fn default_guest_mac() -> String {
    String::from("AA:FC:00:00:00:01")
}
fn default_dns_servers() -> Vec<String> {
    vec![String::from("8.8.8.8"), String::from("8.8.4.4")]
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClusterSpec {
    pub flavor: ClusterFlavor,
    pub provider: ClusterProvider,
    #[serde(default = "default_cluster_count")]
    pub count: u16,
    pub machine: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    pub bootstrap: ClusterBootstrapSpec,
    pub lifecycle: ClusterLifecycleSpec,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClusterBootstrapSpec {
    pub stage_root: PathBuf,
    pub install_script: PathBuf,
    pub binary: PathBuf,
    pub guest_profile: ClusterGuestProfileSpec,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClusterGuestProfileSpec {
    pub name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_commands: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClusterLifecycleSpec {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub health_command: Vec<String>,
    pub kubeconfig_path: PathBuf,
    pub api_forward_target: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub forwards: Vec<ServiceForwardSpec>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceForwardSpec {
    pub name: String,
    pub target: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ClusterFlavor {
    K3s,
}

impl std::fmt::Display for ClusterFlavor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::K3s => f.write_str("k3s"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ClusterProvider {
    Local,
    Hosted,
    Aws,
}

impl std::fmt::Display for ClusterProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Local => f.write_str("local"),
            Self::Hosted => f.write_str("hosted"),
            Self::Aws => f.write_str("aws"),
        }
    }
}

const fn default_cluster_count() -> u16 {
    1
}

const fn default_control_plane_scheduler() -> HostedSchedulerPolicy {
    HostedSchedulerPolicy::DeterministicFirstFit
}

fn control_plane_scheduler_is_default(policy: &HostedSchedulerPolicy) -> bool {
    *policy == HostedSchedulerPolicy::DeterministicFirstFit
}

pub const HOSTED_K3S_REAL_HA_MIN_CONTROL_PLANES: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HostedK3sHaTopologyPosture {
    NonHaTopology,
    HaEligibleTopology,
}

impl std::fmt::Display for HostedK3sHaTopologyPosture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NonHaTopology => f.write_str("non-ha-topology"),
            Self::HaEligibleTopology => f.write_str("ha-eligible-topology"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct K3sClusterSpec {
    pub control_plane: String,
    pub host_group: String,
    pub server_machines: Vec<String>,
    pub worker_machines: Vec<String>,
    pub api_endpoint: String,
    #[serde(
        default = "default_control_plane_scheduler",
        skip_serializing_if = "control_plane_scheduler_is_default"
    )]
    pub control_plane_scheduler: HostedSchedulerPolicy,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub server_args: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub worker_args: Vec<String>,
}

impl K3sClusterSpec {
    pub fn ha_topology_posture(&self) -> HostedK3sHaTopologyPosture {
        if self.server_machines.len() >= HOSTED_K3S_REAL_HA_MIN_CONTROL_PLANES
            && self.control_plane_scheduler == HostedSchedulerPolicy::Spread
        {
            HostedK3sHaTopologyPosture::HaEligibleTopology
        } else {
            HostedK3sHaTopologyPosture::NonHaTopology
        }
    }
}

pub const DEFAULT_K3S_VERSION_LABEL: &str = "default (packaged guest image)";

#[must_use]
pub fn render_k3s_version_label(version: Option<&str>) -> String {
    version
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| DEFAULT_K3S_VERSION_LABEL.to_string())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ServiceKind {
    Service,
    Sandbox,
}

impl std::fmt::Display for ServiceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Service => f.write_str("service"),
            Self::Sandbox => f.write_str("sandbox"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceSecretBinding {
    pub env: String,
    pub secret: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ServiceSecretBackend {
    #[default]
    RuntimeFile,
}

impl std::fmt::Display for ServiceSecretBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RuntimeFile => f.write_str("runtime-file"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ServiceSecretMaterialization {
    #[default]
    Env,
}

impl std::fmt::Display for ServiceSecretMaterialization {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Env => f.write_str("env"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceSecretSourceStatus {
    pub env: String,
    pub secret: String,
    pub backend: ServiceSecretBackend,
    pub materialization: ServiceSecretMaterialization,
    pub path: PathBuf,
    pub detail: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ServiceRestartPolicy {
    #[default]
    Never,
    OnFailure,
    Always,
}

impl std::fmt::Display for ServiceRestartPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Never => f.write_str("never"),
            Self::OnFailure => f.write_str("on-failure"),
            Self::Always => f.write_str("always"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ServiceHealthPolicy {
    #[default]
    None,
    Command,
}

impl std::fmt::Display for ServiceHealthPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => f.write_str("none"),
            Self::Command => f.write_str("command"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ServiceHealthState {
    #[default]
    Unknown,
    Healthy,
    Unhealthy,
}

impl std::fmt::Display for ServiceHealthState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unknown => f.write_str("unknown"),
            Self::Healthy => f.write_str("healthy"),
            Self::Unhealthy => f.write_str("unhealthy"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ServiceHealthcheck {
    #[serde(default)]
    pub policy: ServiceHealthPolicy,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub command: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ServicePolicy {
    #[serde(default)]
    pub restart: ServiceRestartPolicy,
    #[serde(default)]
    pub healthcheck: ServiceHealthcheck,
}

impl ServicePolicy {
    pub fn validate_for_kind(&self, kind: ServiceKind) -> std::result::Result<(), String> {
        if matches!(kind, ServiceKind::Sandbox)
            && !matches!(self.restart, ServiceRestartPolicy::Never)
        {
            return Err(String::from(
                "sandbox services only support restart policy 'never'",
            ));
        }
        if matches!(kind, ServiceKind::Sandbox)
            && !matches!(self.healthcheck.policy, ServiceHealthPolicy::None)
        {
            return Err(String::from(
                "sandbox services only support health policy 'none'",
            ));
        }
        if matches!(self.healthcheck.policy, ServiceHealthPolicy::Command)
            && self.healthcheck.command.is_empty()
        {
            return Err(String::from(
                "health policy 'command' requires at least one health command token",
            ));
        }
        if matches!(self.healthcheck.policy, ServiceHealthPolicy::None)
            && !self.healthcheck.command.is_empty()
        {
            return Err(String::from(
                "health command requires health policy 'command', not health policy 'none'",
            ));
        }
        Ok(())
    }
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
    pub monitor_route: MachineCommandRoute,
    pub top_route: MachineCommandRoute,
    pub service_route: MachineCommandRoute,
    pub guest_route: MachineCommandRoute,
}

impl MachineControlContract {
    #[must_use]
    pub fn for_connection(connection: &HostConnection) -> Self {
        match connection {
            HostConnection::Local => Self::local_runtime_root(),
            HostConnection::HostedControlPlane { .. } => Self::hosted_control_plane(),
            HostConnection::Ssh { .. } => Self::ssh_managed_remote(),
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
            monitor_route: MachineCommandRoute::DirectLocalRuntime,
            top_route: MachineCommandRoute::DirectLocalRuntime,
            service_route: MachineCommandRoute::DirectLocalRuntime,
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
            monitor_route: MachineCommandRoute::HostedControlPlane,
            top_route: MachineCommandRoute::HostedControlPlane,
            service_route: MachineCommandRoute::HostedControlPlane,
            guest_route: MachineCommandRoute::HostedControlPlane,
        }
    }

    #[must_use]
    pub fn ssh_managed_remote() -> Self {
        Self {
            inventory_scope: MachineInventoryScope::SshRuntimeRoot,
            inventory_owner: MachineInventoryOwner::SshRemoteRuntime,
            lifecycle_owner: MachineLifecycleOwner::SshRemotePortRuntime,
            guest_broker: MachineGuestBroker::SshRemoteRuntimeTransport,
            status_source: MachineStatusSource::SshRuntimeManifestAndHostProcess,
            launch_route: MachineCommandRoute::SshManagedRemote,
            inventory_route: MachineCommandRoute::SshManagedRemote,
            status_route: MachineCommandRoute::SshManagedRemote,
            stop_route: MachineCommandRoute::SshManagedRemote,
            monitor_route: MachineCommandRoute::SshManagedRemote,
            top_route: MachineCommandRoute::SshManagedRemote,
            service_route: MachineCommandRoute::SshManagedRemote,
            guest_route: MachineCommandRoute::SshManagedRemote,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FirecrackerPvmLaneContract {
    pub architecture: MachineArchitecture,
    pub decision: PvmLaneDecision,
    pub host_kit: Option<PvmHostKit>,
    pub artifact_kit: Option<PvmArtifactKit>,
    pub validation: Vec<PvmValidationExpectation>,
    pub operator_prerequisites: Vec<String>,
    pub follow_on_work: Vec<String>,
}

impl FirecrackerPvmLaneContract {
    #[must_use]
    pub fn capability_state(&self) -> PvmCapabilityState {
        match self.decision {
            PvmLaneDecision::Planned => PvmCapabilityState::Planned,
            PvmLaneDecision::ResearchOnly => PvmCapabilityState::ResearchOnly,
        }
    }

    #[must_use]
    pub fn for_architecture(architecture: MachineArchitecture) -> Self {
        match architecture {
            MachineArchitecture::X86_64 => Self {
                architecture,
                decision: PvmLaneDecision::Planned,
                host_kit: Some(x86_64_firecracker_pvm_host_kit()),
                artifact_kit: Some(x86_64_firecracker_pvm_artifact_kit()),
                validation: vec![
                    PvmValidationExpectation {
                        name: String::from("host-architecture"),
                        blocking: true,
                        detail: String::from(
                            "Confirm the execution host is Linux/x86_64 before attempting the Firecracker/PVM lane.",
                        ),
                    },
                    PvmValidationExpectation {
                        name: String::from("host-kernel"),
                        blocking: true,
                        detail: String::from(
                            "Confirm the host is booted into the custom PVM-capable kernel and that the host boot line includes pti=off.",
                        ),
                    },
                    PvmValidationExpectation {
                        name: String::from("firecracker-binary"),
                        blocking: true,
                        detail: String::from(
                            "Confirm the selected Firecracker binary is the patched PVM-capable build rather than the standard local-launch binary.",
                        ),
                    },
                    PvmValidationExpectation {
                        name: String::from("artifact-variants"),
                        blocking: true,
                        detail: String::from(
                            "Confirm both kernel and guest-image artifacts exist for x86_64/firecracker/pvm and pass their variant-specific validation steps.",
                        ),
                    },
                ],
                operator_prerequisites: vec![
                    String::from(
                        "Prepare a dedicated Linux/x86_64 host kit before enabling Firecracker/PVM in Port.",
                    ),
                    String::from(
                        "Do not reuse the standard Firecracker host or standard guest artifacts for the PVM lane.",
                    ),
                ],
                follow_on_work: vec![
                    String::from(
                        "Teach port doctor to validate the x86_64 PVM host kit and host boot-line requirements.",
                    ),
                    String::from(
                        "Add build, pull, and validate pipelines for x86_64/firecracker/pvm kernel and guest-image variants.",
                    ),
                    String::from(
                        "Add a Firecracker/PVM driver path that selects the PVM host kit and fails fast when the host kit is absent.",
                    ),
                ],
            },
            MachineArchitecture::Aarch64 => Self {
                architecture,
                decision: PvmLaneDecision::ResearchOnly,
                host_kit: None,
                artifact_kit: None,
                validation: vec![PvmValidationExpectation {
                    name: String::from("runtime-path"),
                    blocking: true,
                    detail: String::from(
                        "Upstream arm64 protected virtualization work exists, but Port does not yet have a supportable Firecracker/PVM runtime path to validate.",
                    ),
                }],
                operator_prerequisites: vec![String::from(
                    "Treat arm64 Firecracker/PVM as research-only until Port ships a host-kit, VMM, and artifact contract backed by a real runtime path.",
                )],
                follow_on_work: vec![
                    String::from(
                        "Track upstream arm64 protected-virtualization and guest-memory work relevant to Firecracker.",
                    ),
                    String::from(
                        "Reassess arm64 only after a supportable Firecracker runtime path exists, not only because upstream kernel capability exists.",
                    ),
                ],
            },
            MachineArchitecture::Native => {
                Self::for_architecture(resolve_native_pvm_architecture())
            }
        }
    }
}

fn resolve_native_pvm_architecture() -> MachineArchitecture {
    match std::env::consts::ARCH {
        "x86_64" => MachineArchitecture::X86_64,
        "aarch64" => MachineArchitecture::Aarch64,
        _ => MachineArchitecture::Native,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PvmLaneDecision {
    Planned,
    ResearchOnly,
}

impl std::fmt::Display for PvmLaneDecision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            Self::Planned => "planned",
            Self::ResearchOnly => "research-only",
        };
        f.write_str(label)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PvmHostKitPackage {
    pub name: String,
    pub version: String,
    pub host_kernel_release: String,
    pub firecracker_build: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PvmHostKit {
    pub package: PvmHostKitPackage,
    pub host_platform: HostPlatform,
    pub host_architecture: MachineArchitecture,
    pub requires_custom_host_kernel: bool,
    pub requires_patched_firecracker: bool,
    pub firecracker_binary_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub firecracker_binary_env: Option<String>,
    pub host_boot_args: Vec<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PvmArtifactKit {
    pub kernel_selector: ArtifactSelector,
    pub guest_image_selector: ArtifactSelector,
    pub requires_dedicated_variants: bool,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PvmValidationExpectation {
    pub name: String,
    pub blocking: bool,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AvfExecutionContract {
    pub host_platform: HostPlatform,
    pub supported_host_architectures: Vec<MachineArchitecture>,
    pub launch_owners: Vec<AvfLaunchOwner>,
    pub guest_transport: AvfGuestTransport,
    pub console_transport: AvfConsoleTransport,
    pub directory_share: AvfDirectoryShareContract,
    pub operator_prerequisites: Vec<String>,
    pub follow_on_work: Vec<String>,
}

impl AvfExecutionContract {
    #[must_use]
    pub fn linux_guest() -> Self {
        Self {
            host_platform: HostPlatform::Macos,
            supported_host_architectures: vec![
                MachineArchitecture::Aarch64,
                MachineArchitecture::X86_64,
            ],
            launch_owners: vec![
                AvfLaunchOwner::LocalPortRuntime,
                AvfLaunchOwner::HostedNodeAgent,
            ],
            guest_transport: AvfGuestTransport::VirtioSocket,
            console_transport: AvfConsoleTransport::SerialPort,
            directory_share: AvfDirectoryShareContract {
                supported: true,
                required_for_rosetta: true,
                notes: vec![
                    String::from(
                        "Directory sharing is optional for Port guest control, but required when enabling Rosetta support for Linux guests on Apple silicon.",
                    ),
                    String::from(
                        "Port should keep guest exec/copy/pty/logs/forward on the guest-agent protocol rather than replacing it with host directory mounts.",
                    ),
                ],
            },
            operator_prerequisites: vec![
                String::from(
                    "Run the AVF lane on macOS with the Virtualization framework available.",
                ),
                String::from(
                    "Distributed macOS app targets need Apple's virtualization entitlement; sandboxed distributions also need the relevant network and file-access entitlements.",
                ),
            ],
            follow_on_work: vec![
                String::from(
                    "Implement an AVF driver that maps machine launch onto VZVirtualMachineConfiguration plus a Linux boot loader.",
                ),
                String::from(
                    "Bridge the guest agent through AVF virtio sockets and map console/log capture onto AVF serial ports.",
                ),
                String::from(
                    "Add macOS-focused port doctor checks for AVF availability, entitlements, and optional Rosetta support.",
                ),
            ],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AvfLaunchOwner {
    LocalPortRuntime,
    HostedNodeAgent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AvfGuestTransport {
    VirtioSocket,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AvfConsoleTransport {
    SerialPort,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AvfDirectoryShareContract {
    pub supported: bool,
    pub required_for_rosetta: bool,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MachineInventoryScope {
    LocalRuntimeRoot,
    HostedFleet,
    SshRuntimeRoot,
}

impl std::fmt::Display for MachineInventoryScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            Self::LocalRuntimeRoot => "local-runtime-root",
            Self::HostedFleet => "hosted-fleet",
            Self::SshRuntimeRoot => "ssh-runtime-root",
        };
        f.write_str(label)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MachineInventoryOwner {
    LocalRuntimeRoot,
    HostedControlPlane,
    SshRemoteRuntime,
}

impl std::fmt::Display for MachineInventoryOwner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            Self::LocalRuntimeRoot => "local-runtime-root",
            Self::HostedControlPlane => "hosted-control-plane",
            Self::SshRemoteRuntime => "ssh-remote-runtime",
        };
        f.write_str(label)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MachineLifecycleOwner {
    LocalPortRuntime,
    HostedNodeAgent,
    SshRemotePortRuntime,
}

impl std::fmt::Display for MachineLifecycleOwner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            Self::LocalPortRuntime => "local-port-runtime",
            Self::HostedNodeAgent => "hosted-node-agent",
            Self::SshRemotePortRuntime => "ssh-remote-port-runtime",
        };
        f.write_str(label)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MachineGuestBroker {
    LocalRuntimeTransport,
    ControlPlaneNodeAgentTunnel,
    SshRemoteRuntimeTransport,
}

impl std::fmt::Display for MachineGuestBroker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            Self::LocalRuntimeTransport => "local-runtime-transport",
            Self::ControlPlaneNodeAgentTunnel => "control-plane-node-agent-tunnel",
            Self::SshRemoteRuntimeTransport => "ssh-remote-runtime-transport",
        };
        f.write_str(label)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MachineStatusSource {
    RuntimeManifestAndHostProcess,
    ControlPlaneInventoryAndNodeAgentRuntime,
    SshRuntimeManifestAndHostProcess,
}

impl std::fmt::Display for MachineStatusSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            Self::RuntimeManifestAndHostProcess => "runtime-manifest-and-host-process",
            Self::ControlPlaneInventoryAndNodeAgentRuntime => {
                "control-plane-inventory-and-node-agent-runtime"
            }
            Self::SshRuntimeManifestAndHostProcess => "ssh-runtime-manifest-and-host-process",
        };
        f.write_str(label)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MachineCommandRoute {
    DirectLocalRuntime,
    HostedControlPlane,
    SshManagedRemote,
}

impl std::fmt::Display for MachineCommandRoute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            Self::DirectLocalRuntime => "direct-local-runtime",
            Self::HostedControlPlane => "hosted-control-plane",
            Self::SshManagedRemote => "ssh-managed-remote",
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

fn validate_artifact_distribution(
    artifact_name: &str,
    artifact: &ArtifactSpec,
) -> Result<(), ValidationError> {
    validate_artifact_store(artifact_name, "push", &artifact.distribution.push)?;
    validate_artifact_store(artifact_name, "pull", &artifact.distribution.pull)?;
    Ok(())
}

fn validate_artifact_store(
    artifact_name: &str,
    direction: &str,
    store: &ArtifactStore,
) -> Result<(), ValidationError> {
    match store {
        ArtifactStore::FileSystem { .. } | ArtifactStore::HostedApi { .. } => Ok(()),
        ArtifactStore::OciRegistry { auth, .. } => match auth {
            OciRegistryAuth::Anonymous => Ok(()),
            OciRegistryAuth::BasicEnv {
                username_variable,
                password_variable,
            } => {
                if username_variable.trim().is_empty() {
                    return Err(ValidationError::new(format!(
                        "artifact '{}' {} OCI registry backend must declare a non-empty username environment variable",
                        artifact_name, direction
                    )));
                }
                if password_variable.trim().is_empty() {
                    return Err(ValidationError::new(format!(
                        "artifact '{}' {} OCI registry backend must declare a non-empty password environment variable",
                        artifact_name, direction
                    )));
                }
                Ok(())
            }
        },
    }
}

fn validate_machine_volumes(
    machine_name: &str,
    machine: &MachineSpec,
) -> Result<(), ValidationError> {
    if machine.volumes.len() > 1 {
        return Err(ValidationError::new(format!(
            "machine '{}' currently supports at most one attached volume in the first storage slice",
            machine_name
        )));
    }

    let mut seen_names = Vec::new();
    for volume in &machine.volumes {
        if volume.name.trim().is_empty() {
            return Err(ValidationError::new(format!(
                "machine '{}' volume name must be non-empty",
                machine_name
            )));
        }
        if volume.path.as_os_str().is_empty() {
            return Err(ValidationError::new(format!(
                "machine '{}' volume '{}' must declare a non-empty path",
                machine_name, volume.name
            )));
        }
        if seen_names.iter().any(|seen| seen == &volume.name) {
            return Err(ValidationError::new(format!(
                "machine '{}' declares duplicate volume '{}'",
                machine_name, volume.name
            )));
        }
        seen_names.push(volume.name.clone());
    }

    Ok(())
}

fn validate_machine_rootfs_overlay(
    machine_name: &str,
    machine: &MachineSpec,
    resolved_architecture: MachineArchitecture,
) -> Result<(), ValidationError> {
    let Some(overlay) = &machine.rootfs_overlay else {
        return Ok(());
    };

    if overlay.size_mib == 0 {
        return Err(ValidationError::new(format!(
            "machine '{}' rootfs overlay size must be greater than zero MiB",
            machine_name
        )));
    }
    if !machine.rootfs_read_only {
        return Err(ValidationError::new(format!(
            "machine '{}' rootfs overlay requires rootfs_read_only = true so Port can boot a read-only base image with a writable overlay",
            machine_name
        )));
    }
    if machine.substrate != ExecutionSubstrate::Firecracker {
        return Err(ValidationError::new(format!(
            "machine '{}' rootfs overlay is only supported on the Firecracker lane in this slice",
            machine_name
        )));
    }
    if !matches!(
        machine.protection_mode,
        ProtectionMode::Standard | ProtectionMode::Pvm
    ) {
        return Err(ValidationError::new(format!(
            "machine '{}' rootfs overlay requires Firecracker protection mode 'standard' or 'pvm'",
            machine_name
        )));
    }
    if resolved_architecture != MachineArchitecture::X86_64 {
        return Err(ValidationError::new(format!(
            "machine '{}' rootfs overlay currently requires x86_64 Firecracker guests because the overlay initrd contract is only shipped for that architecture",
            machine_name
        )));
    }

    Ok(())
}

fn runtime_writable_root_label(root: MachineRuntimeWritableRoot) -> &'static str {
    match root {
        MachineRuntimeWritableRoot::NixStore => "nix-store",
        MachineRuntimeWritableRoot::SourceRoot => "source-root",
        MachineRuntimeWritableRoot::TempRoot => "temp-root",
        MachineRuntimeWritableRoot::EvidenceRoot => "evidence-root",
    }
}

fn runtime_declared_input_label(input: MachineRuntimeDeclaredInput) -> &'static str {
    match input {
        MachineRuntimeDeclaredInput::SourceBundle => "source-bundle",
        MachineRuntimeDeclaredInput::RequestedOutputs => "requested-outputs",
        MachineRuntimeDeclaredInput::PolicySnapshot => "policy-snapshot",
        MachineRuntimeDeclaredInput::CandidateClosure => "candidate-closure",
    }
}

fn validate_machine_runtime_class(
    machine_name: &str,
    machine: &MachineSpec,
) -> Result<(), ValidationError> {
    let Some(runtime_class) = &machine.runtime_class else {
        return Ok(());
    };

    let mut seen_writable_roots = Vec::new();
    for root in &runtime_class.writable_roots {
        if seen_writable_roots.contains(root) {
            return Err(ValidationError::new(format!(
                "machine '{}' runtime class '{}' declares duplicate writable root '{}'",
                machine_name,
                runtime_class.kind,
                runtime_writable_root_label(*root)
            )));
        }
        seen_writable_roots.push(*root);
    }

    let mut seen_declared_inputs = Vec::new();
    for input in &runtime_class.declared_inputs {
        if seen_declared_inputs.contains(input) {
            return Err(ValidationError::new(format!(
                "machine '{}' runtime class '{}' declares duplicate declared input '{}'",
                machine_name,
                runtime_class.kind,
                runtime_declared_input_label(*input)
            )));
        }
        seen_declared_inputs.push(*input);
    }

    match runtime_class.kind {
        MachineRuntimeClassKind::WorkspaceScratchBuilder => {
            let workspace = runtime_class.workspace.as_ref().ok_or_else(|| {
                ValidationError::new(format!(
                    "machine '{}' runtime class '{}' must declare a workspace binding",
                    machine_name, runtime_class.kind
                ))
            })?;
            if workspace.workspace.trim().is_empty() {
                return Err(ValidationError::new(format!(
                    "machine '{}' runtime class '{}' must declare a non-empty workspace name",
                    machine_name, runtime_class.kind
                )));
            }
            if workspace.lane.trim().is_empty() {
                return Err(ValidationError::new(format!(
                    "machine '{}' runtime class '{}' must declare a non-empty workspace lane",
                    machine_name, runtime_class.kind
                )));
            }
            if runtime_class.trust != MachineRuntimeTrustPosture::WorkspaceUntrusted {
                return Err(ValidationError::new(format!(
                    "machine '{}' runtime class '{}' must stay 'workspace-untrusted' and cannot imply publish trust",
                    machine_name, runtime_class.kind
                )));
            }
            if runtime_class.state_isolation != MachineRuntimeStateIsolation::WorkspaceWritable {
                return Err(ValidationError::new(format!(
                    "machine '{}' runtime class '{}' must use state isolation 'workspace-writable'",
                    machine_name, runtime_class.kind
                )));
            }
            if !machine.rootfs_read_only || machine.rootfs_overlay.is_none() {
                return Err(ValidationError::new(format!(
                    "machine '{}' runtime class '{}' requires a read-only base guest plus a writable rootfs overlay so scratch state stays explicit",
                    machine_name, runtime_class.kind
                )));
            }
            for required_root in [
                MachineRuntimeWritableRoot::NixStore,
                MachineRuntimeWritableRoot::SourceRoot,
                MachineRuntimeWritableRoot::TempRoot,
            ] {
                if !runtime_class.writable_roots.contains(&required_root) {
                    return Err(ValidationError::new(format!(
                        "machine '{}' runtime class '{}' must declare writable root '{}'",
                        machine_name,
                        runtime_class.kind,
                        runtime_writable_root_label(required_root)
                    )));
                }
            }
        }
        MachineRuntimeClassKind::BlessedClosurePromotionRunner => {
            if runtime_class.workspace.is_some() {
                return Err(ValidationError::new(format!(
                    "machine '{}' runtime class '{}' must not carry a workspace binding or creator-scoped identity",
                    machine_name, runtime_class.kind
                )));
            }
            if runtime_class.trust != MachineRuntimeTrustPosture::PromotionTrusted {
                return Err(ValidationError::new(format!(
                    "machine '{}' runtime class '{}' must stay 'promotion-trusted'",
                    machine_name, runtime_class.kind
                )));
            }
            if runtime_class.state_isolation != MachineRuntimeStateIsolation::CleanRoom {
                return Err(ValidationError::new(format!(
                    "machine '{}' runtime class '{}' must use state isolation 'clean-room'",
                    machine_name, runtime_class.kind
                )));
            }
            if !machine.rootfs_read_only || machine.rootfs_overlay.is_none() {
                return Err(ValidationError::new(format!(
                    "machine '{}' runtime class '{}' requires a read-only base guest plus a writable rootfs overlay for clean-room execution",
                    machine_name, runtime_class.kind
                )));
            }
            if runtime_class
                .writable_roots
                .contains(&MachineRuntimeWritableRoot::NixStore)
                || runtime_class
                    .writable_roots
                    .contains(&MachineRuntimeWritableRoot::SourceRoot)
                || runtime_class
                    .writable_roots
                    .contains(&MachineRuntimeWritableRoot::TempRoot)
            {
                return Err(ValidationError::new(format!(
                    "machine '{}' runtime class '{}' must not reuse scratch writable roots 'nix-store', 'source-root', or 'temp-root'",
                    machine_name, runtime_class.kind
                )));
            }
            if !runtime_class
                .writable_roots
                .contains(&MachineRuntimeWritableRoot::EvidenceRoot)
            {
                return Err(ValidationError::new(format!(
                    "machine '{}' runtime class '{}' must declare writable root 'evidence-root'",
                    machine_name, runtime_class.kind
                )));
            }
            for required_input in [
                MachineRuntimeDeclaredInput::SourceBundle,
                MachineRuntimeDeclaredInput::RequestedOutputs,
                MachineRuntimeDeclaredInput::PolicySnapshot,
            ] {
                if !runtime_class.declared_inputs.contains(&required_input) {
                    return Err(ValidationError::new(format!(
                        "machine '{}' runtime class '{}' must declare input '{}'",
                        machine_name,
                        runtime_class.kind,
                        runtime_declared_input_label(required_input)
                    )));
                }
            }
        }
    }

    Ok(())
}

fn machine_volume_backend_label(backend: MachineVolumeBackend) -> &'static str {
    match backend {
        MachineVolumeBackend::HostFile => "host-file",
    }
}

fn machine_volume_persistence_label(persistence: MachineVolumePersistence) -> &'static str {
    match persistence {
        MachineVolumePersistence::Persistent => "persistent",
    }
}

fn machine_volume_lane_supported(host: &HostSpec, machine: &MachineSpec) -> bool {
    matches!(host.connection, HostConnection::Local)
        && host.platform == HostPlatform::Linux
        && machine.substrate == ExecutionSubstrate::Firecracker
        && machine.protection_mode == ProtectionMode::Standard
}

fn validate_machine_volume_lane(
    machine_name: &str,
    host_name: &str,
    host: &HostSpec,
    machine: &MachineSpec,
) -> Result<(), ValidationError> {
    if machine.volumes.is_empty() || machine_volume_lane_supported(host, machine) {
        return Ok(());
    }

    let control = MachineControlContract::for_connection(&host.connection);
    let route = control.launch_route;
    let inventory_owner = control.inventory_owner;
    let lifecycle_owner = control.lifecycle_owner;

    let volume = machine
        .volumes
        .first()
        .expect("machine without attached volumes should return early");
    let backend = machine_volume_backend_label(volume.backend);
    let persistence = machine_volume_persistence_label(volume.persistence);

    Err(ValidationError::new(format!(
        "machine '{machine_name}' attached volume '{}' backend '{}' persistence '{}' host path '{}' targets host '{}' through launch route '{}' with inventory owner '{}' and lifecycle owner '{}'; attached volumes are only supported on the local Firecracker standard lane in this slice",
        volume.name,
        backend,
        persistence,
        volume.path.display(),
        host_name,
        route,
        inventory_owner,
        lifecycle_owner
    )))
}

fn validate_hosted_control_plane(
    control_plane_name: &str,
    control_plane: &HostedControlPlaneSpec,
) -> Result<(), ValidationError> {
    if control_plane.endpoint.trim().is_empty() {
        return Err(ValidationError::new(format!(
            "control plane '{}' must declare a non-empty endpoint",
            control_plane_name
        )));
    }
    if control_plane.audience.trim().is_empty() {
        return Err(ValidationError::new(format!(
            "control plane '{}' must declare a non-empty audience",
            control_plane_name
        )));
    }
    if control_plane.auth.header.trim().is_empty() {
        return Err(ValidationError::new(format!(
            "control plane '{}' must declare a non-empty auth header",
            control_plane_name
        )));
    }
    match &control_plane.auth.source {
        HostedAuthTokenSource::Env { variable } if variable.trim().is_empty() => {
            return Err(ValidationError::new(format!(
                "control plane '{}' must declare a non-empty token environment variable",
                control_plane_name
            )));
        }
        HostedAuthTokenSource::Env { .. } => {}
    }

    Ok(())
}

fn validate_host(host_name: &str, host: &HostSpec) -> Result<(), ValidationError> {
    for lane in &host.firecracker.pvm_lanes {
        validate_firecracker_pvm_lane(host_name, lane)?;
    }

    match &host.connection {
        HostConnection::Local => {}
        HostConnection::HostedControlPlane { control_plane } => {
            if control_plane.trim().is_empty() {
                return Err(ValidationError::new(format!(
                    "host '{}' must declare a non-empty hosted control plane name",
                    host_name
                )));
            }
        }
        HostConnection::Ssh {
            destination,
            user,
            port,
        } => {
            if destination.trim().is_empty() {
                return Err(ValidationError::new(format!(
                    "host '{}' ssh connection must declare a non-empty destination",
                    host_name
                )));
            }
            if user.trim().is_empty() {
                return Err(ValidationError::new(format!(
                    "host '{}' ssh connection must declare a non-empty user",
                    host_name
                )));
            }
            if *port == 0 {
                return Err(ValidationError::new(format!(
                    "host '{}' ssh connection must declare a non-zero port",
                    host_name
                )));
            }
        }
    }

    Ok(())
}

fn validate_hosted_node(
    config: &PortConfig,
    node_name: &str,
    node: &HostedNodeSpec,
) -> Result<(), ValidationError> {
    let host = config.hosts.get(&node.host).ok_or_else(|| {
        ValidationError::new(format!(
            "node '{}' references unknown host '{}'",
            node_name, node.host
        ))
    })?;
    match &host.connection {
        HostConnection::Local => {
            return Err(ValidationError::new(format!(
                "node '{}' references local host '{}' but hosted nodes must resolve through a hosted control plane",
                node_name, node.host
            )));
        }
        HostConnection::HostedControlPlane { .. } => {}
        HostConnection::Ssh { .. } => {
            return Err(ValidationError::new(format!(
                "node '{}' references ssh-managed host '{}' but hosted nodes must resolve through a hosted control plane",
                node_name, node.host
            )));
        }
    }
    if node.runtime_root.as_os_str().is_empty() {
        return Err(ValidationError::new(format!(
            "node '{}' must declare a non-empty runtime_root",
            node_name
        )));
    }
    if node.capabilities.providers.is_empty()
        || node.capabilities.platforms.is_empty()
        || node.capabilities.substrates.is_empty()
        || node.capabilities.architectures.is_empty()
        || node.capabilities.protection_modes.is_empty()
    {
        return Err(ValidationError::new(format!(
            "node '{}' must declare non-empty provider, platform, substrate, architecture, and protection-mode capabilities",
            node_name
        )));
    }
    for lane in &node.capabilities.pvm_lanes {
        if lane.state != PvmCapabilityState::ResearchOnly
            && lane.architecture != MachineArchitecture::X86_64
        {
            return Err(ValidationError::new(format!(
                "node '{}' cannot declare a non-research PVM lane for architecture '{:?}'",
                node_name, lane.architecture
            )));
        }
        if !node.capabilities.architectures.contains(&lane.architecture) {
            return Err(ValidationError::new(format!(
                "node '{}' declares PVM lane '{:?}' without advertising that architecture in capabilities.architectures",
                node_name, lane.architecture
            )));
        }
        validate_hosted_pvm_capability(node_name, lane)?;
    }
    if node
        .capabilities
        .pvm_lanes
        .iter()
        .any(|lane| lane.state != PvmCapabilityState::ResearchOnly)
        && !node
            .capabilities
            .protection_modes
            .contains(&ProtectionMode::Pvm)
    {
        return Err(ValidationError::new(format!(
            "node '{}' declares a PVM-ready or planned lane but does not advertise protection mode 'pvm'",
            node_name
        )));
    }

    Ok(())
}

fn validate_firecracker_pvm_lane(
    host_name: &str,
    lane: &FirecrackerPvmLaneContract,
) -> Result<(), ValidationError> {
    match lane.decision {
        PvmLaneDecision::Planned => {
            let host_kit = lane.host_kit.as_ref().ok_or_else(|| {
                ValidationError::new(format!(
                    "host '{}' planned PVM lane '{:?}' must declare a host-kit contract",
                    host_name, lane.architecture
                ))
            })?;
            validate_pvm_host_kit(
                &format!("host '{}' PVM lane '{:?}'", host_name, lane.architecture),
                lane.architecture,
                host_kit,
            )?;
        }
        PvmLaneDecision::ResearchOnly => {
            if lane.host_kit.is_some() {
                return Err(ValidationError::new(format!(
                    "host '{}' research-only PVM lane '{:?}' must not declare a host-kit contract",
                    host_name, lane.architecture
                )));
            }
        }
    }

    Ok(())
}

fn validate_hosted_pvm_capability(
    node_name: &str,
    lane: &HostedPvmCapability,
) -> Result<(), ValidationError> {
    match lane.state {
        PvmCapabilityState::Ready => {
            let host_kit = lane.host_kit.as_ref().ok_or_else(|| {
                ValidationError::new(format!(
                    "node '{}' ready PVM lane '{:?}' must declare a host-kit contract",
                    node_name, lane.architecture
                ))
            })?;
            validate_pvm_host_kit(
                &format!(
                    "node '{}' ready PVM lane '{:?}'",
                    node_name, lane.architecture
                ),
                lane.architecture,
                host_kit,
            )?;
        }
        PvmCapabilityState::Planned => {
            if let Some(host_kit) = lane.host_kit.as_ref() {
                validate_pvm_host_kit(
                    &format!(
                        "node '{}' planned PVM lane '{:?}'",
                        node_name, lane.architecture
                    ),
                    lane.architecture,
                    host_kit,
                )?;
            }
        }
        PvmCapabilityState::ResearchOnly => {
            if lane.host_kit.is_some() {
                return Err(ValidationError::new(format!(
                    "node '{}' research-only PVM lane '{:?}' must not declare a host-kit contract",
                    node_name, lane.architecture
                )));
            }
        }
    }

    Ok(())
}

fn validate_pvm_host_kit(
    context: &str,
    expected_architecture: MachineArchitecture,
    host_kit: &PvmHostKit,
) -> Result<(), ValidationError> {
    if host_kit.package.name.trim().is_empty() {
        return Err(ValidationError::new(format!(
            "{context} must declare a non-empty host-kit package name"
        )));
    }
    if host_kit.package.version.trim().is_empty() {
        return Err(ValidationError::new(format!(
            "{context} must declare a non-empty host-kit package version"
        )));
    }
    if host_kit.package.host_kernel_release.trim().is_empty() {
        return Err(ValidationError::new(format!(
            "{context} must declare a non-empty host-kernel release in the host-kit package"
        )));
    }
    if host_kit.package.firecracker_build.trim().is_empty() {
        return Err(ValidationError::new(format!(
            "{context} must declare a non-empty Firecracker build in the host-kit package"
        )));
    }
    if host_kit.host_platform != HostPlatform::Linux {
        return Err(ValidationError::new(format!(
            "{context} must target host platform 'linux' for Firecracker/PVM"
        )));
    }
    if host_kit.host_architecture != expected_architecture {
        return Err(ValidationError::new(format!(
            "{context} must target host architecture '{:?}', not '{:?}'",
            expected_architecture, host_kit.host_architecture
        )));
    }
    if host_kit.firecracker_binary_name.trim().is_empty() {
        return Err(ValidationError::new(format!(
            "{context} must declare a non-empty firecracker binary name in the host-kit contract"
        )));
    }
    if host_kit
        .firecracker_binary_env
        .as_deref()
        .is_some_and(|name| name.trim().is_empty())
    {
        return Err(ValidationError::new(format!(
            "{context} must declare a non-empty firecracker binary environment variable in the host-kit contract when firecracker_binary_env is set"
        )));
    }
    if host_kit
        .host_boot_args
        .iter()
        .any(|argument| argument.trim().is_empty())
    {
        return Err(ValidationError::new(format!(
            "{context} host-kit contract must not contain empty host boot arguments"
        )));
    }
    if host_kit.requires_custom_host_kernel && host_kit.host_boot_args.is_empty() {
        return Err(ValidationError::new(format!(
            "{context} host-kit contract must declare at least one host boot argument for the custom host kernel"
        )));
    }

    Ok(())
}

fn validate_hosted_host_group(
    config: &PortConfig,
    group_name: &str,
    group: &HostedHostGroupSpec,
) -> Result<(), ValidationError> {
    if group.nodes.is_empty() {
        return Err(ValidationError::new(format!(
            "host group '{}' must declare at least one node",
            group_name
        )));
    }

    let mut control_plane = None::<String>;
    for node_name in &group.nodes {
        let node = config.nodes.get(node_name).ok_or_else(|| {
            ValidationError::new(format!(
                "host group '{}' references unknown node '{}'",
                group_name, node_name
            ))
        })?;
        let host = config.hosts.get(&node.host).ok_or_else(|| {
            ValidationError::new(format!(
                "node '{}' references unknown host '{}'",
                node_name, node.host
            ))
        })?;
        let node_control_plane = match &host.connection {
            HostConnection::Local => {
                return Err(ValidationError::new(format!(
                    "host group '{}' references node '{}' on local host '{}'",
                    group_name, node_name, node.host
                )));
            }
            HostConnection::HostedControlPlane { control_plane } => control_plane.clone(),
            HostConnection::Ssh { .. } => {
                return Err(ValidationError::new(format!(
                    "host group '{}' references node '{}' on ssh-managed host '{}'",
                    group_name, node_name, node.host
                )));
            }
        };
        if let Some(current) = &control_plane {
            if current != &node_control_plane {
                return Err(ValidationError::new(format!(
                    "host group '{}' mixes nodes from control planes '{}' and '{}'",
                    group_name, current, node_control_plane
                )));
            }
        } else {
            control_plane = Some(node_control_plane);
        }
    }

    Ok(())
}

fn cluster_local_lane_supported(host: &HostSpec, machine: &MachineSpec) -> bool {
    matches!(host.connection, HostConnection::Local)
        && host.provider == HostProvider::Local
        && host.platform == HostPlatform::Linux
        && machine.substrate == ExecutionSubstrate::Firecracker
        && machine.protection_mode == ProtectionMode::Standard
}

fn validate_cluster(
    config: &PortConfig,
    cluster_name: &str,
    cluster: &ClusterSpec,
) -> Result<(), ValidationError> {
    if cluster.provider != ClusterProvider::Local {
        return Err(ValidationError::new(format!(
            "cluster '{}' requests provider '{}'; only provider 'local' is supported in this slice",
            cluster_name, cluster.provider
        )));
    }
    if cluster.count != 1 {
        return Err(ValidationError::new(format!(
            "cluster '{}' requests count = {}; only count = 1 is supported in this slice",
            cluster_name, cluster.count
        )));
    }
    if cluster.machine.trim().is_empty() {
        return Err(ValidationError::new(format!(
            "cluster '{}' must declare a non-empty machine",
            cluster_name
        )));
    }
    if cluster
        .version
        .as_deref()
        .is_some_and(|version| version.trim().is_empty())
    {
        return Err(ValidationError::new(format!(
            "cluster '{}' version override must not be empty",
            cluster_name
        )));
    }
    if cluster.args.iter().any(|arg| arg.trim().is_empty()) {
        return Err(ValidationError::new(format!(
            "cluster '{}' args must not contain empty values",
            cluster_name
        )));
    }
    if !cluster.bootstrap.stage_root.is_absolute() {
        return Err(ValidationError::new(format!(
            "cluster '{}' bootstrap stage_root '{}' must be an absolute guest path",
            cluster_name,
            cluster.bootstrap.stage_root.display()
        )));
    }
    if cluster.bootstrap.install_script.as_os_str().is_empty() {
        return Err(ValidationError::new(format!(
            "cluster '{}' bootstrap install_script must declare a host path",
            cluster_name
        )));
    }
    if cluster.bootstrap.install_script.file_name().is_none() {
        return Err(ValidationError::new(format!(
            "cluster '{}' bootstrap install_script '{}' must reference a file",
            cluster_name,
            cluster.bootstrap.install_script.display()
        )));
    }
    if cluster.bootstrap.binary.as_os_str().is_empty() {
        return Err(ValidationError::new(format!(
            "cluster '{}' bootstrap binary must declare a host path",
            cluster_name
        )));
    }
    if cluster.bootstrap.binary.file_name().is_none() {
        return Err(ValidationError::new(format!(
            "cluster '{}' bootstrap binary '{}' must reference a file",
            cluster_name,
            cluster.bootstrap.binary.display()
        )));
    }
    if cluster.bootstrap.guest_profile.name.trim().is_empty() {
        return Err(ValidationError::new(format!(
            "cluster '{}' bootstrap guest profile must declare a non-empty name",
            cluster_name
        )));
    }
    if cluster.bootstrap.guest_profile.required_commands.is_empty() {
        return Err(ValidationError::new(format!(
            "cluster '{}' bootstrap guest profile '{}' must declare at least one required command",
            cluster_name, cluster.bootstrap.guest_profile.name
        )));
    }
    if cluster
        .bootstrap
        .guest_profile
        .required_commands
        .iter()
        .any(|command| command.trim().is_empty())
    {
        return Err(ValidationError::new(format!(
            "cluster '{}' bootstrap guest profile '{}' required commands must not contain empty values",
            cluster_name, cluster.bootstrap.guest_profile.name
        )));
    }
    if cluster.lifecycle.health_command.is_empty() {
        return Err(ValidationError::new(format!(
            "cluster '{}' lifecycle must declare a non-empty health_command",
            cluster_name
        )));
    }
    if cluster
        .lifecycle
        .health_command
        .iter()
        .any(|part| part.trim().is_empty())
    {
        return Err(ValidationError::new(format!(
            "cluster '{}' lifecycle health_command must not contain empty values",
            cluster_name
        )));
    }
    if !cluster.lifecycle.kubeconfig_path.is_absolute() {
        return Err(ValidationError::new(format!(
            "cluster '{}' lifecycle kubeconfig_path '{}' must be an absolute guest path",
            cluster_name,
            cluster.lifecycle.kubeconfig_path.display()
        )));
    }
    if cluster.lifecycle.api_forward_target.trim().is_empty() {
        return Err(ValidationError::new(format!(
            "cluster '{}' lifecycle api_forward_target must declare a non-empty guest endpoint",
            cluster_name
        )));
    }

    let machine = config.machines.get(&cluster.machine).ok_or_else(|| {
        ValidationError::new(format!(
            "cluster '{}' references unknown machine '{}'",
            cluster_name, cluster.machine
        ))
    })?;
    let host = config.hosts.get(&machine.host).ok_or_else(|| {
        ValidationError::new(format!(
            "cluster '{}' machine '{}' references unknown host '{}'",
            cluster_name, cluster.machine, machine.host
        ))
    })?;

    if !cluster_local_lane_supported(host, machine) {
        let control = MachineControlContract::for_connection(&host.connection);
        return Err(ValidationError::new(format!(
            "cluster '{}' machine '{}' targets host '{}' provider '{}' through launch route '{}' with substrate '{}' and protection mode '{}'; clusters only support the local Linux Firecracker standard lane in this slice",
            cluster_name,
            cluster.machine,
            machine.host,
            host_provider_label(host.provider),
            control.launch_route,
            execution_substrate_label(machine.substrate),
            protection_mode_label(machine.protection_mode)
        )));
    }
    if machine.rootfs_read_only {
        return Err(ValidationError::new(format!(
            "cluster '{}' machine '{}' must keep the guest rootfs writable for K3s bootstrap in this slice",
            cluster_name, cluster.machine
        )));
    }

    Ok(())
}

fn validate_k3s_cluster(
    config: &PortConfig,
    cluster_name: &str,
    cluster: &K3sClusterSpec,
) -> Result<(), ValidationError> {
    if cluster.control_plane.trim().is_empty() {
        return Err(ValidationError::new(format!(
            "k3s cluster '{}' must declare a non-empty control plane",
            cluster_name
        )));
    }
    if !config.control_planes.contains_key(&cluster.control_plane) {
        return Err(ValidationError::new(format!(
            "k3s cluster '{}' references unknown control plane '{}'",
            cluster_name, cluster.control_plane
        )));
    }
    if cluster.host_group.trim().is_empty() {
        return Err(ValidationError::new(format!(
            "k3s cluster '{}' must declare a non-empty host group",
            cluster_name
        )));
    }
    let group = config.host_groups.get(&cluster.host_group).ok_or_else(|| {
        ValidationError::new(format!(
            "k3s cluster '{}' references unknown host group '{}'",
            cluster_name, cluster.host_group
        ))
    })?;
    let group_control_plane = group_control_plane_name(config, &cluster.host_group, group)?;
    if group_control_plane != cluster.control_plane {
        return Err(ValidationError::new(format!(
            "k3s cluster '{}' binds control plane '{}' but host group '{}' resolves through '{}'",
            cluster_name, cluster.control_plane, cluster.host_group, group_control_plane
        )));
    }
    if cluster.server_machines.is_empty() {
        return Err(ValidationError::new(format!(
            "k3s cluster '{}' must declare at least one control-plane machine",
            cluster_name
        )));
    }
    if cluster.api_endpoint.trim().is_empty() {
        return Err(ValidationError::new(format!(
            "k3s cluster '{}' must declare a non-empty api endpoint",
            cluster_name
        )));
    }
    if cluster
        .version
        .as_deref()
        .is_some_and(|version| version.trim().is_empty())
    {
        return Err(ValidationError::new(format!(
            "k3s cluster '{}' version override must not be empty",
            cluster_name
        )));
    }
    if !cluster.api_endpoint.starts_with("https://") {
        return Err(ValidationError::new(format!(
            "k3s cluster '{}' api endpoint '{}' must start with 'https://'",
            cluster_name, cluster.api_endpoint
        )));
    }
    if cluster.server_args.iter().any(|arg| arg.trim().is_empty()) {
        return Err(ValidationError::new(format!(
            "k3s cluster '{}' server args must not contain empty values",
            cluster_name
        )));
    }
    if cluster.worker_args.iter().any(|arg| arg.trim().is_empty()) {
        return Err(ValidationError::new(format!(
            "k3s cluster '{}' worker args must not contain empty values",
            cluster_name
        )));
    }

    let mut seen = BTreeSet::new();
    let mut server_summaries = Vec::with_capacity(cluster.server_machines.len());
    for server_machine in &cluster.server_machines {
        if server_machine.trim().is_empty() {
            return Err(ValidationError::new(format!(
                "k3s cluster '{}' control-plane machines must not contain empty names",
                cluster_name
            )));
        }
        if !seen.insert(server_machine.clone()) {
            return Err(ValidationError::new(format!(
                "k3s cluster '{}' reuses machine '{}' across K3s node roles",
                cluster_name, server_machine
            )));
        }
        let summary = validate_k3s_cluster_machine(
            config,
            cluster_name,
            &cluster.control_plane,
            &cluster.host_group,
            server_machine,
            "control-plane",
        )?;
        server_summaries.push(summary);
    }

    for worker_machine in &cluster.worker_machines {
        if worker_machine.trim().is_empty() {
            return Err(ValidationError::new(format!(
                "k3s cluster '{}' worker machines must not contain empty names",
                cluster_name
            )));
        }
        if !seen.insert(worker_machine.clone()) {
            return Err(ValidationError::new(format!(
                "k3s cluster '{}' reuses machine '{}' across K3s node roles",
                cluster_name, worker_machine
            )));
        }
        validate_k3s_cluster_machine(
            config,
            cluster_name,
            &cluster.control_plane,
            &cluster.host_group,
            worker_machine,
            "worker",
        )?;
    }

    if cluster.control_plane_scheduler == HostedSchedulerPolicy::Spread {
        let distinct_candidates = server_summaries
            .iter()
            .flat_map(|summary| summary.candidate_nodes.iter().cloned())
            .collect::<BTreeSet<_>>();
        if distinct_candidates.len() < cluster.server_machines.len() {
            return Err(ValidationError::new(format!(
                "k3s cluster '{}' requires {} distinct hosted candidate nodes for control-plane scheduler 'spread', but only {} are available across host group '{}': {}",
                cluster_name,
                cluster.server_machines.len(),
                distinct_candidates.len(),
                cluster.host_group,
                if distinct_candidates.is_empty() {
                    String::from("(none)")
                } else {
                    distinct_candidates
                        .into_iter()
                        .collect::<Vec<_>>()
                        .join(", ")
                }
            )));
        }
    }

    Ok(())
}

fn group_control_plane_name(
    config: &PortConfig,
    group_name: &str,
    group: &HostedHostGroupSpec,
) -> Result<String, ValidationError> {
    let node_name = group.nodes.first().ok_or_else(|| {
        ValidationError::new(format!(
            "host group '{}' must declare at least one node",
            group_name
        ))
    })?;
    let node = config.nodes.get(node_name).ok_or_else(|| {
        ValidationError::new(format!(
            "host group '{}' references unknown node '{}'",
            group_name, node_name
        ))
    })?;
    let host = config.hosts.get(&node.host).ok_or_else(|| {
        ValidationError::new(format!(
            "node '{}' references unknown host '{}'",
            node_name, node.host
        ))
    })?;
    match &host.connection {
        HostConnection::HostedControlPlane { control_plane } => Ok(control_plane.clone()),
        HostConnection::Local => Err(ValidationError::new(format!(
            "host group '{}' references node '{}' on local host '{}'",
            group_name, node_name, node.host
        ))),
        HostConnection::Ssh { .. } => Err(ValidationError::new(format!(
            "host group '{}' references node '{}' on ssh-managed host '{}'",
            group_name, node_name, node.host
        ))),
    }
}

fn validate_k3s_cluster_machine(
    config: &PortConfig,
    cluster_name: &str,
    control_plane: &str,
    host_group: &str,
    machine_name: &str,
    role: &str,
) -> Result<HostedMachineSummaryContract, ValidationError> {
    let machine = config.machines.get(machine_name).ok_or_else(|| {
        ValidationError::new(format!(
            "k3s cluster '{}' references unknown {} machine '{}'",
            cluster_name, role, machine_name
        ))
    })?;
    if !machine.volumes.is_empty() {
        return Err(ValidationError::new(format!(
            "k3s cluster '{}' {} machine '{}' must remain stateless; attached volumes are out of scope",
            cluster_name, role, machine_name
        )));
    }
    if machine.substrate != ExecutionSubstrate::Firecracker {
        return Err(ValidationError::new(format!(
            "k3s cluster '{}' {} machine '{}' must use Firecracker for the first slice",
            cluster_name, role, machine_name
        )));
    }
    if !matches!(
        machine.protection_mode,
        ProtectionMode::Standard | ProtectionMode::Pvm
    ) {
        return Err(ValidationError::new(format!(
            "k3s cluster '{}' {} machine '{}' must use Firecracker protection mode 'standard' or 'pvm'",
            cluster_name, role, machine_name
        )));
    }

    let host = config.hosts.get(&machine.host).ok_or_else(|| {
        ValidationError::new(format!(
            "machine '{}' references unknown host '{}'",
            machine_name, machine.host
        ))
    })?;
    if host.platform != HostPlatform::Linux {
        return Err(ValidationError::new(format!(
            "k3s cluster '{}' {} machine '{}' must target a Linux host",
            cluster_name, role, machine_name
        )));
    }
    match &host.connection {
        HostConnection::HostedControlPlane {
            control_plane: machine_control_plane,
        } => {
            if machine_control_plane != control_plane {
                return Err(ValidationError::new(format!(
                    "k3s cluster '{}' {} machine '{}' resolves through control plane '{}' instead of '{}'",
                    cluster_name, role, machine_name, machine_control_plane, control_plane
                )));
            }
        }
        HostConnection::Local => {
            return Err(ValidationError::new(format!(
                "k3s cluster '{}' {} machine '{}' must target a hosted control plane, not local host '{}'",
                cluster_name, role, machine_name, machine.host
            )));
        }
        HostConnection::Ssh { .. } => {
            return Err(ValidationError::new(format!(
                "k3s cluster '{}' {} machine '{}' must target a hosted control plane, not ssh-managed host '{}'",
                cluster_name, role, machine_name, machine.host
            )));
        }
    }

    let summary = config
        .hosted_machine_summary_contract(machine_name)?
        .ok_or_else(|| {
            ValidationError::new(format!(
                "k3s cluster '{}' {} machine '{}' must resolve to a hosted machine summary",
                cluster_name, role, machine_name
            ))
        })?;
    if !summary.host_groups.iter().any(|group| group == host_group) {
        return Err(ValidationError::new(format!(
            "k3s cluster '{}' {} machine '{}' must belong to host group '{}'; available groups: {}",
            cluster_name,
            role,
            machine_name,
            host_group,
            if summary.host_groups.is_empty() {
                String::from("(none)")
            } else {
                summary.host_groups.join(", ")
            }
        )));
    }
    if summary.candidate_nodes.is_empty() {
        return Err(ValidationError::new(format!(
            "k3s cluster '{}' {} machine '{}' has no hosted placement candidates: {}",
            cluster_name, role, machine_name, summary.placement_detail
        )));
    }

    Ok(summary)
}

fn hosted_node_rejection_reason(
    machine_name: &str,
    node_name: &str,
    machine: &MachineSpec,
    node: &HostedNodeContract,
) -> Result<Option<String>, ValidationError> {
    let architecture = resolve_machine_architecture(machine.architecture).map_err(|error| {
        ValidationError::new(format!(
            "machine '{}' cannot resolve hosted placement architecture: {error}",
            machine_name
        ))
    })?;

    if !node.capabilities.substrates.contains(&machine.substrate) {
        return Ok(Some(format!(
            "substrate '{}' is required but node advertises {}",
            execution_substrate_label(machine.substrate),
            label_list(&node.capabilities.substrates, execution_substrate_label,)
        )));
    }
    if !node.capabilities.architectures.contains(&architecture) {
        return Ok(Some(format!(
            "architecture '{}' is required but node advertises {}",
            machine_architecture_label(architecture),
            label_list(&node.capabilities.architectures, machine_architecture_label,)
        )));
    }
    if !node
        .capabilities
        .protection_modes
        .contains(&machine.protection_mode)
    {
        return Ok(Some(format!(
            "protection mode '{}' is required but node advertises {}",
            protection_mode_label(machine.protection_mode),
            label_list(&node.capabilities.protection_modes, protection_mode_label,)
        )));
    }
    if machine.protection_mode == ProtectionMode::Pvm {
        let Some(lane) = node.capabilities.pvm_lane_for(architecture) else {
            return Ok(Some(format!(
                "pvm-ready state is required but node does not advertise a '{}' PVM lane",
                machine_architecture_label(architecture)
            )));
        };
        if lane.state != PvmCapabilityState::Ready {
            let detail = if lane.host_kit.is_some() {
                format!(
                    "pvm-ready state is required but node advertises {}. Run `port control-plane prepare-pvm-node --control-plane {} --node {} --architecture {}` before launching `{}`; Port does not silently fall back to the standard lane.",
                    hosted_pvm_state_label(lane.state),
                    node.control_plane,
                    node_name,
                    machine_architecture_label(architecture),
                    machine_name
                )
            } else {
                format!(
                    "pvm-ready state is required but node advertises {} without a provider-backed host-kit contract. `{}` stays outside the hosted PVM lane until Port owns a prepared host contract for node '{}'; Port does not silently fall back to the standard lane.",
                    hosted_pvm_state_label(lane.state),
                    machine_name,
                    node_name
                )
            };
            return Ok(Some(detail));
        }
    }

    Ok(None)
}

fn hosted_placement_detail(
    machine_name: &str,
    machine: &MachineSpec,
    host_name: &str,
    provider: HostProvider,
    candidate_nodes: &[String],
    rejected_nodes: &BTreeMap<String, String>,
) -> Result<String, ValidationError> {
    let requirement = hosted_machine_requirement(machine_name, machine, host_name, provider)?;

    let mut detail = format!("machine '{machine_name}' requires {requirement}");
    if candidate_nodes.is_empty() {
        detail.push_str("; no hosted nodes satisfy that requirement");
    } else {
        detail.push_str(&format!(
            "; candidate nodes: {}",
            candidate_nodes.join(", ")
        ));
    }
    if !rejected_nodes.is_empty() {
        let rejected = rejected_nodes
            .iter()
            .map(|(node_name, reason)| format!("{node_name} ({reason})"))
            .collect::<Vec<_>>()
            .join(", ");
        detail.push_str(&format!("; rejected nodes: {rejected}"));
    }

    Ok(detail)
}

fn hosted_machine_requirement(
    machine_name: &str,
    machine: &MachineSpec,
    host_name: &str,
    provider: HostProvider,
) -> Result<String, ValidationError> {
    let architecture = resolve_machine_architecture(machine.architecture).map_err(|error| {
        ValidationError::new(format!(
            "machine '{}' cannot resolve hosted placement architecture: {error}",
            machine_name
        ))
    })?;
    Ok(format!(
        "{} on {} via {} for host '{}' provider '{}'",
        protection_mode_requirement_label(machine.protection_mode),
        machine_architecture_label(architecture),
        execution_substrate_label(machine.substrate),
        host_name,
        host_provider_label(provider),
    ))
}

fn host_provider_label(provider: HostProvider) -> &'static str {
    match provider {
        HostProvider::Local => "local",
        HostProvider::GenericLinux => "generic-linux",
        HostProvider::Aws => "aws",
        HostProvider::Gcp => "gcp",
        HostProvider::Azure => "azure",
    }
}

fn protection_mode_requirement_label(mode: ProtectionMode) -> &'static str {
    match mode {
        ProtectionMode::Standard => "standard protection",
        ProtectionMode::Pvm => "PVM",
    }
}

fn protection_mode_label(mode: ProtectionMode) -> &'static str {
    match mode {
        ProtectionMode::Standard => "standard",
        ProtectionMode::Pvm => "pvm",
    }
}

fn machine_architecture_label(architecture: MachineArchitecture) -> &'static str {
    match architecture {
        MachineArchitecture::Native => "native",
        MachineArchitecture::X86_64 => "x86_64",
        MachineArchitecture::Aarch64 => "aarch64",
    }
}

fn execution_substrate_label(substrate: ExecutionSubstrate) -> &'static str {
    match substrate {
        ExecutionSubstrate::Firecracker => "firecracker",
        ExecutionSubstrate::CloudHypervisor => "cloud-hypervisor",
        ExecutionSubstrate::Avf => "avf",
    }
}

fn hosted_pvm_state_label(state: PvmCapabilityState) -> &'static str {
    match state {
        PvmCapabilityState::Ready => "ready",
        PvmCapabilityState::Planned => "planned",
        PvmCapabilityState::ResearchOnly => "research-only",
    }
}

fn label_list<T: Copy>(items: &[T], label: fn(T) -> &'static str) -> String {
    items
        .iter()
        .copied()
        .map(label)
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        ArtifactReference, ArtifactSelector, ArtifactStore, AvfConsoleTransport,
        AvfExecutionContract, AvfGuestTransport, AvfLaunchOwner, ClusterBootstrapSpec,
        ClusterFlavor, ClusterGuestProfileSpec, ClusterLifecycleSpec, ClusterProvider, ClusterSpec,
        ExecutionSubstrate, FirecrackerPvmLaneContract, GuestCommandVerb, HostConnection,
        HostPlatform, HostProvider, HostedAuthTokenSource, HostedGuestAttachActor,
        HostedGuestAttachHop, HostedGuestProtocolContract, HostedPlacementPolicy,
        HostedSchedulerPolicy, K3sClusterSpec, MachineArchitecture, MachineCommandRoute,
        MachineControlContract, MachineGuestBroker, MachineInventoryOwner, MachineInventoryScope,
        MachineLifecycleOwner, MachineStatusSource, OciRegistryAuth, OciRegistryTransport,
        PortConfig, ProtectionMode, PvmCapabilityState, PvmLaneDecision, ServiceHealthPolicy,
        ServiceHealthcheck, ServiceKind, ServicePolicy, ServiceRestartPolicy,
        hosted_artifact_store_path,
    };

    fn sample_avf_config() -> PortConfig {
        let mut config = PortConfig::sample();
        config.clusters.clear();
        let machine = config
            .machines
            .get_mut("demo")
            .expect("sample machine should exist");
        machine.host = String::from("mac-local");
        machine.substrate = ExecutionSubstrate::Avf;
        machine.architecture = MachineArchitecture::X86_64;
        machine.protection_mode = ProtectionMode::Standard;

        config
    }

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
        assert!(encoded.contains("[control_planes.demo]"));
        assert!(encoded.contains("[nodes.aws-linux-node]"));
        assert!(encoded.contains("[host_groups.remote-linux]"));
        assert!(encoded.contains("scheduler = \"deterministic-first-fit\""));
        assert!(encoded.contains("mode = \"hosted-control-plane\""));
        assert!(encoded.contains("substrate = \"firecracker\""));
        assert!(encoded.contains("substrate = \"cloud-hypervisor\""));
        assert!(encoded.contains("protection_mode = \"standard\""));
        assert!(encoded.contains("architecture = \"native\""));
        assert!(encoded.contains("[[hosts.local.firecracker.pvm_lanes]]"));
        assert!(encoded.contains("decision = \"planned\""));
        assert!(encoded.contains("decision = \"research-only\""));
        assert!(encoded.contains("requires_patched_firecracker = true"));
        assert!(encoded.contains("firecracker_binary_name = \"firecracker-pvm\""));
        assert!(encoded.contains("firecracker_binary_env = \"PORT_PVM_FIRECRACKER_BINARY\""));
        assert!(encoded.contains("[machines.demo.guest]"));
        assert!(encoded.contains("[machines.cloud-aws]"));
        assert!(encoded.contains("[clusters.demo]"));
        assert!(encoded.contains("flavor = \"k3s\""));
        assert!(encoded.contains("provider = \"local\""));
        assert!(encoded.contains("[clusters.demo.bootstrap]"));
        assert!(encoded.contains("stage_root = \"/opt/port/clusters/demo\""));
        assert!(encoded.contains("[clusters.demo.bootstrap.guest_profile]"));
        assert!(encoded.contains("name = \"kube-ready\""));
        assert!(encoded.contains("[clusters.demo.lifecycle]"));
        assert!(encoded.contains("kubeconfig_path = \"/etc/rancher/k3s/k3s.yaml\""));
        assert!(encoded.contains("api_forward_target = \"127.0.0.1:6443\""));
        assert!(encoded.contains("[artifacts.kernels.demo-kernel.reference]"));
        assert!(encoded.contains("[artifacts.kernels.demo-kernel.distribution.push]"));
        assert!(encoded.contains("[artifacts.kernels.demo-kernel.variants]"));
    }

    #[test]
    fn volume_attachment_contract() {
        let mut sample = PortConfig::sample();
        sample
            .machines
            .get_mut("demo")
            .expect("sample machine should exist")
            .volumes
            .push(super::MachineVolumeSpec {
                name: String::from("data"),
                backend: super::MachineVolumeBackend::HostFile,
                persistence: super::MachineVolumePersistence::Persistent,
                path: std::path::PathBuf::from("volumes/demo-data.ext4"),
            });

        sample
            .validate()
            .expect("attached-volume machine contract should validate");

        let encoded = sample.to_toml_string().expect("sample should encode");
        assert!(encoded.contains("[[machines.demo.volumes]]"));
        assert!(encoded.contains("backend = \"host-file\""));
        assert!(encoded.contains("persistence = \"persistent\""));

        let decoded = PortConfig::from_toml_str(&encoded).expect("sample should decode");
        assert_eq!(decoded, sample);
    }

    #[test]
    fn workspace_scratch_runtime_class_round_trips() {
        let mut sample = PortConfig::sample();
        sample.clusters.clear();
        let machine = sample
            .machines
            .get_mut("demo")
            .expect("sample machine should exist");
        machine.architecture = MachineArchitecture::X86_64;
        machine.rootfs_read_only = true;
        machine.rootfs_overlay = Some(super::MachineRootfsOverlaySpec { size_mib: 4096 });
        machine.runtime_class = Some(super::MachineRuntimeClassSpec {
            kind: super::MachineRuntimeClassKind::WorkspaceScratchBuilder,
            trust: super::MachineRuntimeTrustPosture::WorkspaceUntrusted,
            state_isolation: super::MachineRuntimeStateIsolation::WorkspaceWritable,
            writable_roots: vec![
                super::MachineRuntimeWritableRoot::NixStore,
                super::MachineRuntimeWritableRoot::SourceRoot,
                super::MachineRuntimeWritableRoot::TempRoot,
            ],
            declared_inputs: Vec::new(),
            workspace: Some(super::MachineRuntimeWorkspaceBinding {
                workspace: String::from("demo"),
                lane: String::from("scratch"),
            }),
        });

        sample
            .validate()
            .expect("workspace scratch runtime class should validate");

        let encoded = sample.to_toml_string().expect("sample should encode");
        assert!(encoded.contains("[machines.demo.runtime_class]"));
        assert!(encoded.contains("kind = \"workspace-scratch-builder\""));
        assert!(encoded.contains("trust = \"workspace-untrusted\""));
        assert!(encoded.contains("writable_roots = ["));
        assert!(encoded.contains("\"nix-store\""));
        assert!(encoded.contains("\"source-root\""));
        assert!(encoded.contains("\"temp-root\""));
        assert!(encoded.contains("[machines.demo.runtime_class.workspace]"));

        let decoded = PortConfig::from_toml_str(&encoded).expect("sample should decode");
        assert_eq!(decoded, sample);
    }

    #[test]
    fn workspace_scratch_runtime_class_rejects_publish_trust() {
        let mut sample = PortConfig::sample();
        sample.clusters.clear();
        let machine = sample
            .machines
            .get_mut("demo")
            .expect("sample machine should exist");
        machine.architecture = MachineArchitecture::X86_64;
        machine.rootfs_read_only = true;
        machine.rootfs_overlay = Some(super::MachineRootfsOverlaySpec { size_mib: 4096 });
        machine.runtime_class = Some(super::MachineRuntimeClassSpec {
            kind: super::MachineRuntimeClassKind::WorkspaceScratchBuilder,
            trust: super::MachineRuntimeTrustPosture::PromotionTrusted,
            state_isolation: super::MachineRuntimeStateIsolation::WorkspaceWritable,
            writable_roots: vec![
                super::MachineRuntimeWritableRoot::NixStore,
                super::MachineRuntimeWritableRoot::SourceRoot,
                super::MachineRuntimeWritableRoot::TempRoot,
            ],
            declared_inputs: Vec::new(),
            workspace: Some(super::MachineRuntimeWorkspaceBinding {
                workspace: String::from("demo"),
                lane: String::from("scratch"),
            }),
        });

        let error = sample
            .validate()
            .expect_err("scratch builder should reject publish trust");
        assert!(
            error
                .to_string()
                .contains("must stay 'workspace-untrusted' and cannot imply publish trust"),
            "{error}"
        );
    }

    #[test]
    fn workspace_scratch_runtime_class_requires_declared_writable_roots() {
        let mut sample = PortConfig::sample();
        sample.clusters.clear();
        let machine = sample
            .machines
            .get_mut("demo")
            .expect("sample machine should exist");
        machine.architecture = MachineArchitecture::X86_64;
        machine.rootfs_read_only = true;
        machine.rootfs_overlay = Some(super::MachineRootfsOverlaySpec { size_mib: 4096 });
        machine.runtime_class = Some(super::MachineRuntimeClassSpec {
            kind: super::MachineRuntimeClassKind::WorkspaceScratchBuilder,
            trust: super::MachineRuntimeTrustPosture::WorkspaceUntrusted,
            state_isolation: super::MachineRuntimeStateIsolation::WorkspaceWritable,
            writable_roots: vec![
                super::MachineRuntimeWritableRoot::NixStore,
                super::MachineRuntimeWritableRoot::SourceRoot,
            ],
            declared_inputs: Vec::new(),
            workspace: Some(super::MachineRuntimeWorkspaceBinding {
                workspace: String::from("demo"),
                lane: String::from("scratch"),
            }),
        });

        let error = sample
            .validate()
            .expect_err("scratch builder should require the temp root");
        assert!(
            error
                .to_string()
                .contains("must declare writable root 'temp-root'"),
            "{error}"
        );
    }

    #[test]
    fn promotion_runner_runtime_class_round_trips() {
        let mut sample = PortConfig::sample();
        sample.clusters.clear();
        let machine = sample
            .machines
            .get_mut("cloud-aws")
            .expect("sample machine should exist");
        machine.architecture = MachineArchitecture::X86_64;
        machine.rootfs_read_only = true;
        machine.rootfs_overlay = Some(super::MachineRootfsOverlaySpec { size_mib: 4096 });
        machine.runtime_class = Some(super::MachineRuntimeClassSpec {
            kind: super::MachineRuntimeClassKind::BlessedClosurePromotionRunner,
            trust: super::MachineRuntimeTrustPosture::PromotionTrusted,
            state_isolation: super::MachineRuntimeStateIsolation::CleanRoom,
            writable_roots: vec![super::MachineRuntimeWritableRoot::EvidenceRoot],
            declared_inputs: vec![
                super::MachineRuntimeDeclaredInput::SourceBundle,
                super::MachineRuntimeDeclaredInput::RequestedOutputs,
                super::MachineRuntimeDeclaredInput::PolicySnapshot,
            ],
            workspace: None,
        });

        sample
            .validate()
            .expect("promotion runner runtime class should validate");

        let encoded = sample.to_toml_string().expect("sample should encode");
        assert!(encoded.contains("kind = \"blessed-closure-promotion-runner\""));
        assert!(encoded.contains("trust = \"promotion-trusted\""));
        assert!(encoded.contains("state_isolation = \"clean-room\""));
        assert!(encoded.contains("\"source-bundle\""));
        assert!(encoded.contains("\"requested-outputs\""));
        assert!(encoded.contains("\"policy-snapshot\""));

        let decoded = PortConfig::from_toml_str(&encoded).expect("sample should decode");
        assert_eq!(decoded, sample);
    }

    #[test]
    fn promotion_runner_runtime_class_rejects_workspace_binding() {
        let mut sample = PortConfig::sample();
        sample.clusters.clear();
        let machine = sample
            .machines
            .get_mut("cloud-aws")
            .expect("sample machine should exist");
        machine.architecture = MachineArchitecture::X86_64;
        machine.rootfs_read_only = true;
        machine.rootfs_overlay = Some(super::MachineRootfsOverlaySpec { size_mib: 4096 });
        machine.runtime_class = Some(super::MachineRuntimeClassSpec {
            kind: super::MachineRuntimeClassKind::BlessedClosurePromotionRunner,
            trust: super::MachineRuntimeTrustPosture::PromotionTrusted,
            state_isolation: super::MachineRuntimeStateIsolation::CleanRoom,
            writable_roots: vec![super::MachineRuntimeWritableRoot::EvidenceRoot],
            declared_inputs: vec![
                super::MachineRuntimeDeclaredInput::SourceBundle,
                super::MachineRuntimeDeclaredInput::RequestedOutputs,
                super::MachineRuntimeDeclaredInput::PolicySnapshot,
            ],
            workspace: Some(super::MachineRuntimeWorkspaceBinding {
                workspace: String::from("demo"),
                lane: String::from("scratch"),
            }),
        });

        let error = sample
            .validate()
            .expect_err("promotion runner should reject workspace binding");
        assert!(
            error
                .to_string()
                .contains("must not carry a workspace binding or creator-scoped identity"),
            "{error}"
        );
    }

    #[test]
    fn promotion_runner_runtime_class_rejects_scratch_writable_roots() {
        let mut sample = PortConfig::sample();
        sample.clusters.clear();
        let machine = sample
            .machines
            .get_mut("cloud-aws")
            .expect("sample machine should exist");
        machine.architecture = MachineArchitecture::X86_64;
        machine.rootfs_read_only = true;
        machine.rootfs_overlay = Some(super::MachineRootfsOverlaySpec { size_mib: 4096 });
        machine.runtime_class = Some(super::MachineRuntimeClassSpec {
            kind: super::MachineRuntimeClassKind::BlessedClosurePromotionRunner,
            trust: super::MachineRuntimeTrustPosture::PromotionTrusted,
            state_isolation: super::MachineRuntimeStateIsolation::CleanRoom,
            writable_roots: vec![
                super::MachineRuntimeWritableRoot::EvidenceRoot,
                super::MachineRuntimeWritableRoot::TempRoot,
            ],
            declared_inputs: vec![
                super::MachineRuntimeDeclaredInput::SourceBundle,
                super::MachineRuntimeDeclaredInput::RequestedOutputs,
                super::MachineRuntimeDeclaredInput::PolicySnapshot,
            ],
            workspace: None,
        });

        let error = sample
            .validate()
            .expect_err("promotion runner should reject scratch writable roots");
        assert!(
            error
                .to_string()
                .contains("must not reuse scratch writable roots"),
            "{error}"
        );
    }

    #[test]
    fn machine_contract_without_attachments_regression() {
        let sample = PortConfig::sample();

        sample
            .validate()
            .expect("attachment-free sample config should remain valid");
        assert!(
            sample
                .machines
                .get("demo")
                .expect("sample machine should exist")
                .volumes
                .is_empty()
        );

        let encoded = sample.to_toml_string().expect("sample should encode");
        assert!(!encoded.contains("[[machines.demo.volumes]]"));

        let decoded = PortConfig::from_toml_str(&encoded).expect("sample should decode");
        let machine = decoded
            .machines
            .get("demo")
            .expect("sample machine should exist");
        assert!(machine.volumes.is_empty());
        assert_eq!(machine.guest_image, "demo-guest");
        assert!(!machine.rootfs_read_only);
    }

    #[test]
    fn local_cluster_contract() {
        let sample = PortConfig::sample();

        sample
            .validate()
            .expect("sample config with local cluster should validate");

        let cluster = sample
            .clusters
            .get("demo")
            .expect("sample local cluster should exist");
        assert_eq!(
            cluster,
            &ClusterSpec {
                flavor: ClusterFlavor::K3s,
                provider: ClusterProvider::Local,
                count: 1,
                machine: String::from("demo"),
                version: None,
                args: vec![String::from("--disable=traefik")],
                bootstrap: ClusterBootstrapSpec {
                    stage_root: PathBuf::from("/opt/port/clusters/demo"),
                    install_script: PathBuf::from(
                        "examples/bootstrap/demo-k3s/install-k3s-offline.sh",
                    ),
                    binary: PathBuf::from("examples/bootstrap/demo-k3s/k3s"),
                    guest_profile: ClusterGuestProfileSpec {
                        name: String::from("kube-ready"),
                        required_commands: vec![
                            String::from("sh"),
                            String::from("install"),
                            String::from("ln"),
                            String::from("chmod"),
                            String::from("dirname"),
                            String::from("setsid"),
                            String::from("modprobe"),
                        ],
                    },
                },
                lifecycle: ClusterLifecycleSpec {
                    health_command: vec![
                        String::from("opt/port/clusters/demo/bin/k3s"),
                        String::from("kubectl"),
                        String::from("get"),
                        String::from("nodes"),
                        String::from("-o"),
                        String::from("wide"),
                        String::from("--request-timeout=15s"),
                    ],
                    kubeconfig_path: PathBuf::from("/etc/rancher/k3s/k3s.yaml"),
                    api_forward_target: String::from("127.0.0.1:6443"),
                    forwards: Vec::new(),
                },
            }
        );

        let encoded = sample.to_toml_string().expect("sample should encode");
        assert!(encoded.contains("[clusters.demo]"));
        assert!(encoded.contains("count = 1"));
        assert!(encoded.contains("machine = \"demo\""));
        assert!(!encoded.contains("machine = \"demo\"\nversion = "));
        assert!(
            encoded.contains(
                "install_script = \"examples/bootstrap/demo-k3s/install-k3s-offline.sh\""
            )
        );
        assert!(encoded.contains("binary = \"examples/bootstrap/demo-k3s/k3s\""));
        assert!(encoded.contains("health_command = ["));
        assert!(encoded.contains("kubeconfig_path = \"/etc/rancher/k3s/k3s.yaml\""));
        assert!(encoded.contains("api_forward_target = \"127.0.0.1:6443\""));
        assert!(encoded.contains("required_commands = ["));
        assert!(encoded.contains("\"sh\""));
        assert!(encoded.contains("\"install\""));
        assert!(encoded.contains("\"ln\""));
        assert!(encoded.contains("\"chmod\""));

        let decoded = PortConfig::from_toml_str(&encoded).expect("sample should decode");
        assert_eq!(decoded, sample);
    }

    #[test]
    fn local_cluster_contract_rejects_follow_on_provider_and_count_shapes() {
        let mut config = PortConfig::sample();
        config
            .clusters
            .get_mut("demo")
            .expect("sample local cluster should exist")
            .provider = ClusterProvider::Aws;

        let error = config
            .validate()
            .expect_err("aws cluster provider should fail validation");
        assert!(
            error
                .to_string()
                .contains("only provider 'local' is supported in this slice")
        );

        let mut config = PortConfig::sample();
        config
            .clusters
            .get_mut("demo")
            .expect("sample local cluster should exist")
            .count = 2;

        let error = config
            .validate()
            .expect_err("multi-node cluster should fail validation");
        assert!(
            error
                .to_string()
                .contains("only count = 1 is supported in this slice")
        );
    }

    #[test]
    fn local_cluster_contract_rejects_relative_stage_root_and_empty_guest_profile_commands() {
        let mut config = PortConfig::sample();
        config
            .clusters
            .get_mut("demo")
            .expect("sample local cluster should exist")
            .bootstrap
            .stage_root = PathBuf::from("opt/port/clusters/demo");

        let error = config
            .validate()
            .expect_err("relative cluster stage root should fail validation");
        assert!(error.to_string().contains("must be an absolute guest path"));

        let mut config = PortConfig::sample();
        config
            .clusters
            .get_mut("demo")
            .expect("sample local cluster should exist")
            .bootstrap
            .guest_profile
            .required_commands = Vec::new();

        let error = config
            .validate()
            .expect_err("empty guest profile commands should fail validation");
        assert!(
            error
                .to_string()
                .contains("must declare at least one required command")
        );

        let mut config = PortConfig::sample();
        config
            .clusters
            .get_mut("demo")
            .expect("sample local cluster should exist")
            .lifecycle
            .health_command = Vec::new();

        let error = config
            .validate()
            .expect_err("empty lifecycle health command should fail validation");
        assert!(error.to_string().contains("non-empty health_command"));

        let mut config = PortConfig::sample();
        config
            .clusters
            .get_mut("demo")
            .expect("sample local cluster should exist")
            .lifecycle
            .kubeconfig_path = PathBuf::from("etc/rancher/k3s/k3s.yaml");

        let error = config
            .validate()
            .expect_err("relative kubeconfig path should fail validation");
        assert!(error.to_string().contains("must be an absolute guest path"));

        let mut config = PortConfig::sample();
        config
            .clusters
            .get_mut("demo")
            .expect("sample local cluster should exist")
            .lifecycle
            .api_forward_target = String::new();

        let error = config
            .validate()
            .expect_err("empty api forward target should fail validation");
        assert!(error.to_string().contains("non-empty guest endpoint"));
    }

    #[test]
    fn local_cluster_contract_preserves_existing_machine_routes() {
        let config = PortConfig::sample();

        config
            .validate()
            .expect("sample config with local cluster should remain valid");
        assert_eq!(
            config
                .machine_control_contract("demo")
                .expect("local machine contract should resolve"),
            MachineControlContract::local_runtime_root()
        );
        assert_eq!(
            config
                .machine_control_contract("cloud-aws")
                .expect("hosted machine contract should resolve"),
            MachineControlContract::hosted_control_plane()
        );
        assert!(
            config
                .hosted_guest_attach_contract("cloud-aws")
                .expect("hosted guest attach contract should resolve")
                .is_some(),
            "hosted guest contract should remain available with local clusters present"
        );
    }

    #[test]
    fn hosted_k3s_cluster_contract() {
        let mut sample = PortConfig::sample();
        sample.k3s_clusters.insert(
            String::from("demo"),
            K3sClusterSpec {
                control_plane: String::from("demo"),
                host_group: String::from("remote-linux"),
                server_machines: vec![String::from("cloud-generic")],
                worker_machines: vec![String::from("cloud-aws")],
                api_endpoint: String::from("https://demo-k3s.internal:6443"),
                control_plane_scheduler: HostedSchedulerPolicy::Spread,
                version: Some(String::from("v1.35.2+k3s1")),
                server_args: vec![String::from("--disable=traefik")],
                worker_args: Vec::new(),
            },
        );

        sample
            .validate()
            .expect("sample config with hosted k3s cluster should validate");

        let cluster = sample
            .k3s_clusters
            .get("demo")
            .expect("sample hosted k3s cluster should exist");
        assert_eq!(
            cluster,
            &K3sClusterSpec {
                control_plane: String::from("demo"),
                host_group: String::from("remote-linux"),
                server_machines: vec![String::from("cloud-generic")],
                worker_machines: vec![String::from("cloud-aws")],
                api_endpoint: String::from("https://demo-k3s.internal:6443"),
                control_plane_scheduler: HostedSchedulerPolicy::Spread,
                version: Some(String::from("v1.35.2+k3s1")),
                server_args: vec![String::from("--disable=traefik")],
                worker_args: Vec::new(),
            }
        );

        let encoded = sample.to_toml_string().expect("sample should encode");
        assert!(encoded.contains("[k3s_clusters.demo]"));
        assert!(encoded.contains("host_group = \"remote-linux\""));
        assert!(encoded.contains("server_machines = [\"cloud-generic\"]"));
        assert!(encoded.contains("worker_machines = [\"cloud-aws\"]"));
        assert!(encoded.contains("api_endpoint = \"https://demo-k3s.internal:6443\""));
        assert!(encoded.contains("control_plane_scheduler = \"spread\""));
        assert!(encoded.contains("version = \"v1.35.2+k3s1\""));

        let decoded = PortConfig::from_toml_str(&encoded).expect("sample should decode");
        assert_eq!(decoded, sample);
    }

    #[test]
    fn hosted_k3s_spread_scheduler_requires_distinct_candidate_nodes() {
        let mut sample = PortConfig::sample();
        let mut server_b = sample
            .machines
            .get("cloud-aws")
            .expect("cloud-aws should exist")
            .clone();
        server_b.guest.vsock_cid = 62;
        server_b.guest.control_port = 7002;
        server_b.guest.console_log = PathBuf::from("runtime/cloud-aws-b/console.log");
        sample
            .machines
            .insert(String::from("cloud-aws-b"), server_b);
        sample.k3s_clusters.insert(
            String::from("demo"),
            K3sClusterSpec {
                control_plane: String::from("demo"),
                host_group: String::from("aws-builders"),
                server_machines: vec![String::from("cloud-aws"), String::from("cloud-aws-b")],
                worker_machines: Vec::new(),
                api_endpoint: String::from("https://demo-k3s.internal:6443"),
                control_plane_scheduler: HostedSchedulerPolicy::Spread,
                version: Some(String::from("v1.35.2+k3s1")),
                server_args: vec![String::from("--disable=traefik")],
                worker_args: Vec::new(),
            },
        );

        let error = sample
            .validate()
            .expect_err("spread should fail when only one candidate node exists");
        let message = error.to_string();
        assert!(message.contains("control-plane scheduler 'spread'"));
        assert!(message.contains("distinct hosted candidate nodes"));
        assert!(message.contains("aws-builders"));
    }

    #[test]
    fn hosted_k3s_two_control_planes_with_spread_stay_non_ha_topology() {
        let mut sample = PortConfig::sample();
        let mut server_b = sample
            .machines
            .get("cloud-aws")
            .expect("cloud-aws should exist")
            .clone();
        server_b.guest.vsock_cid = 62;
        server_b.guest.control_port = 7002;
        server_b.guest.console_log = PathBuf::from("runtime/cloud-aws-b/console.log");
        sample
            .machines
            .insert(String::from("cloud-aws-b"), server_b);
        sample.k3s_clusters.insert(
            String::from("demo"),
            K3sClusterSpec {
                control_plane: String::from("demo"),
                host_group: String::from("aws-builders"),
                server_machines: vec![String::from("cloud-aws"), String::from("cloud-aws-b")],
                worker_machines: Vec::new(),
                api_endpoint: String::from("https://demo-k3s.internal:6443"),
                control_plane_scheduler: HostedSchedulerPolicy::Spread,
                version: Some(String::from("v1.35.2+k3s1")),
                server_args: vec![String::from("--disable=traefik")],
                worker_args: Vec::new(),
            },
        );

        let cluster = sample
            .k3s_clusters
            .get("demo")
            .expect("demo cluster should exist");
        assert_eq!(
            cluster.ha_topology_posture(),
            super::HostedK3sHaTopologyPosture::NonHaTopology
        );
    }

    #[test]
    fn hosted_k3s_three_control_planes_with_spread_become_ha_eligible_topology() {
        let mut sample = PortConfig::sample();
        for (name, cid, port, log) in [
            ("cloud-aws-b", 62, 7002, "runtime/cloud-aws-b/console.log"),
            ("cloud-aws-c", 63, 7003, "runtime/cloud-aws-c/console.log"),
        ] {
            let mut machine = sample
                .machines
                .get("cloud-aws")
                .expect("cloud-aws should exist")
                .clone();
            machine.guest.vsock_cid = cid;
            machine.guest.control_port = port;
            machine.guest.console_log = PathBuf::from(log);
            sample.machines.insert(String::from(name), machine);
        }
        sample.k3s_clusters.insert(
            String::from("demo"),
            K3sClusterSpec {
                control_plane: String::from("demo"),
                host_group: String::from("aws-builders"),
                server_machines: vec![
                    String::from("cloud-aws"),
                    String::from("cloud-aws-b"),
                    String::from("cloud-aws-c"),
                ],
                worker_machines: Vec::new(),
                api_endpoint: String::from("https://demo-k3s.internal:6443"),
                control_plane_scheduler: HostedSchedulerPolicy::Spread,
                version: Some(String::from("v1.35.2+k3s1")),
                server_args: vec![String::from("--disable=traefik")],
                worker_args: Vec::new(),
            },
        );

        let cluster = sample
            .k3s_clusters
            .get("demo")
            .expect("demo cluster should exist");
        assert_eq!(
            cluster.ha_topology_posture(),
            super::HostedK3sHaTopologyPosture::HaEligibleTopology
        );
    }

    #[test]
    fn hosted_k3s_cluster_accepts_pvm_control_plane_machines() {
        let mut sample = PortConfig::sample();
        sample
            .nodes
            .get_mut("aws-linux-node")
            .expect("aws-linux-node should exist")
            .capabilities
            .pvm_lanes[0]
            .state = PvmCapabilityState::Ready;
        sample
            .machines
            .get_mut("cloud-aws")
            .expect("cloud-aws should exist")
            .protection_mode = ProtectionMode::Pvm;
        sample.k3s_clusters.insert(
            String::from("demo"),
            K3sClusterSpec {
                control_plane: String::from("demo"),
                host_group: String::from("aws-builders"),
                server_machines: vec![String::from("cloud-aws")],
                worker_machines: Vec::new(),
                api_endpoint: String::from("https://demo-k3s.internal:6443"),
                control_plane_scheduler: HostedSchedulerPolicy::Spread,
                version: Some(String::from("v1.35.2+k3s1")),
                server_args: vec![String::from("--disable=traefik")],
                worker_args: Vec::new(),
            },
        );

        sample
            .validate()
            .expect("hosted k3s should accept Firecracker PVM control-plane machines");
    }

    #[test]
    fn hosted_k3s_cluster_contract_regression_existing_routes() {
        let mut config = PortConfig::sample();
        config.k3s_clusters.clear();

        config
            .validate()
            .expect("sample config without hosted k3s clusters should remain valid");
        assert!(config.k3s_clusters.is_empty());

        let encoded = config.to_toml_string().expect("sample should encode");
        assert!(!encoded.contains("[k3s_clusters.demo]"));

        assert_eq!(
            config
                .machine_control_contract("demo")
                .expect("local machine contract should resolve"),
            MachineControlContract::local_runtime_root()
        );
        assert_eq!(
            config
                .machine_control_contract("cloud-aws")
                .expect("hosted machine contract should resolve"),
            MachineControlContract::hosted_control_plane()
        );
        assert!(
            config
                .hosted_guest_attach_contract("cloud-aws")
                .expect("hosted guest attach contract should resolve")
                .is_some(),
            "hosted guest contract should remain available without k3s clusters"
        );

        config.nodes.clear();
        config.host_groups.clear();
        config
            .hosts
            .get_mut("generic-linux")
            .expect("generic-linux host should exist")
            .connection = HostConnection::Ssh {
            destination: String::from("builder.example.internal"),
            user: String::from("ubuntu"),
            port: 2222,
        };

        config
            .validate()
            .expect("ssh route regression config without k3s clusters should validate");
        assert_eq!(
            config
                .machine_control_contract("cloud-generic")
                .expect("ssh-backed machine contract should resolve"),
            MachineControlContract::ssh_managed_remote()
        );
    }

    #[test]
    fn service_policy_defaults_to_never_and_no_healthcheck() {
        let policy = ServicePolicy::default();

        assert_eq!(policy.restart, ServiceRestartPolicy::Never);
        assert_eq!(policy.healthcheck.policy, ServiceHealthPolicy::None);
        assert!(policy.healthcheck.command.is_empty());
        policy
            .validate_for_kind(ServiceKind::Service)
            .expect("default policy should be valid");
    }

    #[test]
    fn service_policy_invalid_combinations_reject_restart_and_health_mismatches() {
        let sandbox_error = ServicePolicy {
            restart: ServiceRestartPolicy::Always,
            healthcheck: ServiceHealthcheck::default(),
        }
        .validate_for_kind(ServiceKind::Sandbox)
        .expect_err("sandbox restart policy should reject");
        assert!(sandbox_error.contains("sandbox"), "{sandbox_error}");
        assert!(
            sandbox_error.contains("restart policy 'never'"),
            "{sandbox_error}"
        );

        let missing_command = ServicePolicy {
            restart: ServiceRestartPolicy::OnFailure,
            healthcheck: ServiceHealthcheck {
                policy: ServiceHealthPolicy::Command,
                command: Vec::new(),
            },
        }
        .validate_for_kind(ServiceKind::Service)
        .expect_err("command health policy should require a command");
        assert!(
            missing_command.contains("health policy 'command'"),
            "{missing_command}"
        );

        let stray_command = ServicePolicy {
            restart: ServiceRestartPolicy::Never,
            healthcheck: ServiceHealthcheck {
                policy: ServiceHealthPolicy::None,
                command: vec![String::from("/bin/true")],
            },
        }
        .validate_for_kind(ServiceKind::Service)
        .expect_err("health command should reject when policy is none");
        assert!(stray_command.contains("health command"), "{stray_command}");
        assert!(
            stray_command.contains("health policy 'none'"),
            "{stray_command}"
        );
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
        assert_eq!(config.hosts["mac-local"].platform, HostPlatform::Macos);
        assert_eq!(
            config.hosts["generic-linux"].provider,
            HostProvider::GenericLinux
        );
        assert_eq!(config.hosts["aws-linux"].provider, HostProvider::Aws);
        assert_eq!(config.hosts["gcp-linux"].provider, HostProvider::Gcp);
        assert_eq!(config.hosts["azure-linux"].provider, HostProvider::Azure);
        assert_eq!(config.machines["cloud-aws"].host, "aws-linux");
        assert_eq!(config.machines["demo-avf"].host, "mac-local");
        assert_eq!(config.machines["demo-ch"].host, "local");
        assert_eq!(
            config.machines["demo"].substrate,
            ExecutionSubstrate::Firecracker
        );
        assert_eq!(
            config.machines["demo-ch"].substrate,
            ExecutionSubstrate::CloudHypervisor
        );
        assert_eq!(
            config.machines["demo-avf"].substrate,
            ExecutionSubstrate::Avf
        );
        assert_eq!(
            config.machines["demo-avf"].architecture,
            MachineArchitecture::Native
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
    fn sample_config_includes_cloud_hypervisor_standard_variants() {
        let config = PortConfig::sample();
        let kernel = &config.artifacts.kernels["demo-kernel"];
        let guest = &config.artifacts.guest_images["demo-guest"];

        assert!(kernel.supports(
            MachineArchitecture::X86_64,
            ExecutionSubstrate::CloudHypervisor,
            ProtectionMode::Standard
        ));
        assert!(kernel.supports(
            MachineArchitecture::Aarch64,
            ExecutionSubstrate::CloudHypervisor,
            ProtectionMode::Standard
        ));
        assert!(guest.supports(
            MachineArchitecture::X86_64,
            ExecutionSubstrate::CloudHypervisor,
            ProtectionMode::Standard
        ));
        assert!(guest.supports(
            MachineArchitecture::Aarch64,
            ExecutionSubstrate::CloudHypervisor,
            ProtectionMode::Standard
        ));
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
        assert_eq!(
            contract.monitor_route,
            MachineCommandRoute::DirectLocalRuntime
        );
        assert_eq!(contract.top_route, MachineCommandRoute::DirectLocalRuntime);
        assert_eq!(
            contract.service_route,
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
        assert_eq!(
            contract.monitor_route,
            MachineCommandRoute::HostedControlPlane
        );
        assert_eq!(contract.top_route, MachineCommandRoute::HostedControlPlane);
        assert_eq!(
            contract.service_route,
            MachineCommandRoute::HostedControlPlane
        );
    }

    #[test]
    fn ssh_host_connection_contract() {
        let mut config = PortConfig::sample();
        config.nodes.clear();
        config.host_groups.clear();
        config
            .hosts
            .get_mut("generic-linux")
            .expect("generic-linux host should exist")
            .connection = HostConnection::Ssh {
            destination: String::from("builder.example.internal"),
            user: String::from("ubuntu"),
            port: 2222,
        };

        let encoded = config.to_toml_string().expect("config should encode");
        assert!(encoded.contains("mode = \"ssh\""));
        assert!(encoded.contains("destination = \"builder.example.internal\""));
        assert!(encoded.contains("user = \"ubuntu\""));
        assert!(encoded.contains("port = 2222"));

        let decoded = PortConfig::from_toml_str(&encoded).expect("config should decode");
        assert_eq!(
            decoded.hosts["generic-linux"].connection,
            HostConnection::Ssh {
                destination: String::from("builder.example.internal"),
                user: String::from("ubuntu"),
                port: 2222,
            }
        );

        let contract = decoded
            .machine_control_contract("cloud-generic")
            .expect("ssh-backed machine contract should resolve");
        assert_eq!(contract, MachineControlContract::ssh_managed_remote());
        assert_eq!(
            contract.inventory_scope,
            MachineInventoryScope::SshRuntimeRoot
        );
        assert_eq!(
            contract.inventory_owner,
            MachineInventoryOwner::SshRemoteRuntime
        );
        assert_eq!(
            contract.lifecycle_owner,
            MachineLifecycleOwner::SshRemotePortRuntime
        );
        assert_eq!(
            contract.guest_broker,
            MachineGuestBroker::SshRemoteRuntimeTransport
        );
        assert_eq!(
            contract.status_source,
            MachineStatusSource::SshRuntimeManifestAndHostProcess
        );
        assert_eq!(contract.launch_route, MachineCommandRoute::SshManagedRemote);
        assert_eq!(contract.status_route, MachineCommandRoute::SshManagedRemote);
        assert_eq!(contract.stop_route, MachineCommandRoute::SshManagedRemote);
    }

    #[test]
    fn hybrid_route_regression_local_and_hosted() {
        let config = PortConfig::sample();

        assert_eq!(
            config
                .machine_control_contract("demo")
                .expect("local contract should resolve"),
            MachineControlContract::local_runtime_root()
        );
        assert_eq!(
            config
                .machine_control_contract("cloud-aws")
                .expect("hosted contract should resolve"),
            MachineControlContract::hosted_control_plane()
        );
        assert_eq!(
            config
                .hosted_api_identity_contract("demo")
                .expect("local identity should resolve"),
            None
        );
        assert!(
            config
                .hosted_api_identity_contract("cloud-aws")
                .expect("hosted identity should resolve")
                .is_some(),
            "hosted machines should continue to resolve hosted API identity"
        );
    }

    #[test]
    fn sample_config_derives_hosted_api_identity_contract() {
        let config = PortConfig::sample();

        let contract = config
            .hosted_api_identity_contract("cloud-aws")
            .expect("cloud aws contract should resolve")
            .expect("cloud aws should target a hosted control plane");

        assert_eq!(contract.control_plane, "demo");
        assert_eq!(contract.endpoint, "https://port.example.internal");
        assert_eq!(contract.audience, "port-hosted-demo");
        assert_eq!(contract.route, MachineCommandRoute::HostedControlPlane);
        assert_eq!(contract.auth.header, "authorization");
        assert!(matches!(
            contract.auth.source,
            HostedAuthTokenSource::Env { variable } if variable == "PORT_DEMO_TOKEN"
        ));
        assert_eq!(
            config.hosts["aws-linux"].connection,
            HostConnection::HostedControlPlane {
                control_plane: String::from("demo")
            }
        );
    }

    #[test]
    fn sample_config_derives_hosted_artifact_identity_contract() {
        let config = PortConfig::sample();

        let contract = config
            .hosted_artifact_identity_contract("https://port.example.internal")
            .expect("hosted artifact contract should resolve");

        assert_eq!(contract.control_plane, "demo");
        assert_eq!(contract.endpoint, "https://port.example.internal");
        assert_eq!(contract.audience, "port-hosted-demo");
        assert_eq!(contract.auth.header, "authorization");
        assert!(matches!(
            contract.auth.source,
            HostedAuthTokenSource::Env { variable } if variable == "PORT_DEMO_TOKEN"
        ));
    }

    #[test]
    fn hosted_artifact_store_path_is_deterministic() {
        let path = hosted_artifact_store_path(
            "demo",
            &ArtifactReference {
                registry: String::from("demo-fs"),
                repository: String::from("port/demo-kernel"),
                version: String::from("v1"),
            },
            ArtifactSelector {
                architecture: MachineArchitecture::X86_64,
                substrate: ExecutionSubstrate::Firecracker,
                protection_mode: ProtectionMode::Standard,
            },
            PathBuf::from("vmlinux"),
        );

        assert_eq!(
            path,
            PathBuf::from(
                ".port/hosted/demo/artifacts/demo-fs/port/demo-kernel/v1/x86_64/firecracker/standard/vmlinux"
            )
        );
    }

    #[test]
    fn oci_registry_contract_derives_deterministic_remote_reference() {
        let selector = ArtifactSelector {
            architecture: MachineArchitecture::X86_64,
            substrate: ExecutionSubstrate::Firecracker,
            protection_mode: ProtectionMode::Standard,
        };
        let reference = ArtifactReference {
            registry: String::from("registry.port.test:5000"),
            repository: String::from("artifacts/demo-kernel"),
            version: String::from("v1"),
        };

        assert_eq!(
            reference.oci_remote_reference(selector),
            "registry.port.test:5000/artifacts/demo-kernel:v1-x86_64-firecracker-standard"
        );

        let encoded = toml::to_string(&ArtifactStore::OciRegistry {
            transport: OciRegistryTransport::PlainHttp,
            auth: OciRegistryAuth::BasicEnv {
                username_variable: String::from("PORT_OCI_USER"),
                password_variable: String::from("PORT_OCI_PASSWORD"),
            },
        })
        .expect("oci registry store should encode");

        assert!(encoded.contains("backend = \"oci-registry\""));
        assert!(encoded.contains("transport = \"plain-http\""));
        assert!(encoded.contains("kind = \"basic-env\""));
        assert!(encoded.contains("username_variable = \"PORT_OCI_USER\""));
        assert!(encoded.contains("password_variable = \"PORT_OCI_PASSWORD\""));
    }

    #[test]
    fn oci_registry_contract_rejects_empty_basic_auth_environment_variables() {
        let mut config = PortConfig::sample();
        config
            .artifacts
            .kernels
            .get_mut("demo-kernel")
            .unwrap()
            .distribution
            .push = ArtifactStore::OciRegistry {
            transport: OciRegistryTransport::PlainHttp,
            auth: OciRegistryAuth::BasicEnv {
                username_variable: String::new(),
                password_variable: String::from("PORT_OCI_PASSWORD"),
            },
        };

        let error = config
            .validate()
            .expect_err("empty OCI auth variable names should fail validation");

        assert!(
            error.to_string().contains("OCI registry backend"),
            "unexpected validation error: {error}"
        );
    }

    #[test]
    fn validate_rejects_duplicate_control_plane_endpoints() {
        let mut config = PortConfig::sample();
        config.control_planes.insert(
            String::from("shadow"),
            super::HostedControlPlaneSpec {
                endpoint: String::from("https://port.example.internal"),
                audience: String::from("shadow-audience"),
                auth: super::HostedAuthTokenContract {
                    scheme: super::HostedAuthScheme::Bearer,
                    header: String::from("authorization"),
                    source: super::HostedAuthTokenSource::Env {
                        variable: String::from("PORT_SHADOW_TOKEN"),
                    },
                },
            },
        );

        let error = config
            .validate()
            .expect_err("duplicate control-plane endpoints should be rejected");

        assert!(
            error
                .to_string()
                .contains("duplicate hosted control-plane endpoint"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn validate_rejects_unknown_control_plane_reference() {
        let mut config = PortConfig::sample();
        config
            .hosts
            .get_mut("aws-linux")
            .expect("aws host")
            .connection = HostConnection::HostedControlPlane {
            control_plane: String::from("missing"),
        };
        config.host_groups.clear();

        let error = config
            .validate()
            .expect_err("missing control plane should fail validation");

        assert!(
            error
                .to_string()
                .contains("references unknown control plane 'missing'")
        );
    }

    #[test]
    fn sample_config_derives_hosted_node_inventory_contract() {
        let config = PortConfig::sample();

        let contract = config
            .hosted_inventory_contract()
            .expect("hosted inventory contract should resolve");

        let aws_node = &contract.nodes["aws-linux-node"];
        assert_eq!(aws_node.host, "aws-linux");
        assert_eq!(
            aws_node.runtime_root,
            PathBuf::from("runtime/hosted/aws-linux-node")
        );
        assert_eq!(aws_node.control_plane, "demo");
        assert_eq!(
            aws_node.inventory_owner,
            MachineInventoryOwner::HostedControlPlane
        );
        assert_eq!(
            aws_node.lifecycle_owner,
            MachineLifecycleOwner::HostedNodeAgent
        );
        assert_eq!(aws_node.capabilities.providers, vec![HostProvider::Aws]);

        let remote_group = &contract.host_groups["remote-linux"];
        assert_eq!(remote_group.control_plane, "demo");
        assert_eq!(
            remote_group.inventory_owner,
            MachineInventoryOwner::HostedControlPlane
        );
        assert_eq!(
            remote_group.placement,
            HostedPlacementPolicy::ExplicitMembership
        );
        assert_eq!(
            remote_group.scheduler,
            HostedSchedulerPolicy::DeterministicFirstFit
        );
        assert!(
            remote_group
                .nodes
                .contains(&String::from("generic-linux-node"))
        );

        let registered = contract
            .hosted_registered_node_contract(
                "demo",
                "aws-linux-node",
                &super::HostedNodeRegistration {
                    endpoint: String::from("http://127.0.0.1:9001"),
                    token: String::from("node-demo-token"),
                    registered_at: 10,
                    refreshed_at: 25,
                    ttl_seconds: 30,
                },
            )
            .expect("registered node contract should resolve");
        assert_eq!(registered.node_name, "aws-linux-node");
        assert_eq!(registered.endpoint, "http://127.0.0.1:9001");
        assert_eq!(registered.freshness.registered_at, 10);
        assert_eq!(registered.freshness.refreshed_at, 25);
        assert_eq!(registered.freshness.fresh_until, 55);
        assert_eq!(registered.node.runtime_root, aws_node.runtime_root);
        assert!(
            registered
                .host_groups
                .contains(&String::from("remote-linux"))
        );
        assert!(
            registered
                .host_groups
                .contains(&String::from("aws-builders"))
        );
    }

    #[test]
    fn hosted_registered_node_contract_rejects_invalid_registration_inputs() {
        let config = PortConfig::sample();
        let contract = config
            .hosted_inventory_contract()
            .expect("hosted inventory contract should resolve");

        let missing_endpoint = contract
            .hosted_registered_node_contract(
                "demo",
                "aws-linux-node",
                &super::HostedNodeRegistration {
                    endpoint: String::from(" "),
                    token: String::from("node-demo-token"),
                    registered_at: 10,
                    refreshed_at: 25,
                    ttl_seconds: 30,
                },
            )
            .expect_err("blank endpoint should fail");
        assert!(
            missing_endpoint
                .to_string()
                .contains("must declare a non-empty endpoint")
        );

        let stale_refresh = contract
            .hosted_registered_node_contract(
                "demo",
                "aws-linux-node",
                &super::HostedNodeRegistration {
                    endpoint: String::from("http://127.0.0.1:9001"),
                    token: String::from("node-demo-token"),
                    registered_at: 50,
                    refreshed_at: 25,
                    ttl_seconds: 30,
                },
            )
            .expect_err("refresh before registration should fail");
        assert!(
            stale_refresh
                .to_string()
                .contains("cannot refresh before its initial registration")
        );
    }

    #[test]
    fn sample_config_derives_hosted_machine_lifecycle_contracts() {
        let config = PortConfig::sample();

        let summary = config
            .hosted_machine_summary_contract("cloud-aws")
            .expect("hosted machine summary should resolve")
            .expect("cloud-aws should be hosted");
        assert_eq!(summary.host_name, "aws-linux");
        assert_eq!(summary.provider, HostProvider::Aws);
        assert_eq!(summary.control_plane, "demo");
        assert_eq!(
            summary.candidate_nodes,
            vec![String::from("aws-linux-node")]
        );
        assert!(summary.placement_detail.contains("host 'aws-linux'"));
        assert!(summary.placement_detail.contains("provider 'aws'"));
        assert!(summary.host_groups.contains(&String::from("remote-linux")));
        assert!(summary.host_groups.contains(&String::from("aws-builders")));
        assert_eq!(
            summary.host_group_policies["remote-linux"],
            HostedSchedulerPolicy::DeterministicFirstFit
        );
        assert_eq!(
            summary.host_group_policies["aws-builders"],
            HostedSchedulerPolicy::DeterministicFirstFit
        );
        assert_eq!(
            summary.control.status_route,
            MachineCommandRoute::HostedControlPlane
        );

        let status = config
            .hosted_machine_status_contract("cloud-aws")
            .expect("hosted machine status should resolve")
            .expect("cloud-aws status should be hosted");
        assert_eq!(
            status.status_source,
            MachineStatusSource::ControlPlaneInventoryAndNodeAgentRuntime
        );
        assert!(status.detail.contains("control-plane inventory"));

        let stop = config
            .hosted_machine_stop_contract("cloud-aws")
            .expect("hosted machine stop should resolve")
            .expect("cloud-aws stop should be hosted");
        assert_eq!(stop.stop_route, MachineCommandRoute::HostedControlPlane);
        assert_eq!(stop.lifecycle_owner, MachineLifecycleOwner::HostedNodeAgent);
        assert!(stop.detail.contains("node agent"));

        let monitor = config
            .hosted_machine_monitor_contract("cloud-aws")
            .expect("hosted machine monitor should resolve")
            .expect("cloud-aws monitor should be hosted");
        assert_eq!(
            monitor.status_source,
            MachineStatusSource::ControlPlaneInventoryAndNodeAgentRuntime
        );
        assert_eq!(
            monitor.monitor_route,
            MachineCommandRoute::HostedControlPlane
        );
        assert_eq!(monitor.top_route, MachineCommandRoute::HostedControlPlane);
        assert_eq!(
            monitor.lifecycle_owner,
            MachineLifecycleOwner::HostedNodeAgent
        );
        assert!(monitor.detail.contains("detached forward"));

        let service = config
            .hosted_service_contract("cloud-aws")
            .expect("hosted service contract should resolve")
            .expect("cloud-aws service should be hosted");
        assert_eq!(
            service.lifecycle_owner,
            MachineLifecycleOwner::HostedNodeAgent
        );
        assert_eq!(
            service.guest_broker,
            MachineGuestBroker::ControlPlaneNodeAgentTunnel
        );
        assert_eq!(
            service.service_route,
            MachineCommandRoute::HostedControlPlane
        );
        assert!(service.detail.contains("canonical service surface"));
    }

    #[test]
    fn hosted_pvm_summary_filters_candidates_to_ready_nodes() {
        let mut config = PortConfig::sample();
        config
            .machines
            .get_mut("cloud-generic")
            .expect("cloud-generic should exist")
            .protection_mode = ProtectionMode::Pvm;

        let summary = config
            .hosted_machine_summary_contract("cloud-generic")
            .expect("hosted pvm summary should resolve")
            .expect("cloud-generic should be hosted");

        assert!(summary.candidate_nodes.is_empty());
        assert!(
            summary.rejected_nodes["generic-linux-node"]
                .contains("without a provider-backed host-kit contract")
        );
        assert!(summary.rejected_nodes["generic-linux-node"].contains("cloud-generic"));
        assert!(summary.placement_detail.contains("generic-linux-node"));
        assert!(summary.placement_detail.contains("planned"));
    }

    #[test]
    fn hosted_pvm_summary_keeps_aws_contract_provider_aware() {
        let mut config = PortConfig::sample();
        config
            .machines
            .get_mut("cloud-aws")
            .expect("cloud-aws should exist")
            .protection_mode = ProtectionMode::Pvm;

        let summary = config
            .hosted_machine_summary_contract("cloud-aws")
            .expect("cloud-aws hosted pvm summary should resolve")
            .expect("cloud-aws should be hosted");

        assert_eq!(summary.host_name, "aws-linux");
        assert_eq!(summary.provider, HostProvider::Aws);
        assert!(summary.candidate_nodes.is_empty());
        assert!(summary.rejected_nodes["aws-linux-node"].contains("prepare-pvm-node"));
        assert!(summary.rejected_nodes["aws-linux-node"].contains("cloud-aws"));
        assert!(summary.rejected_nodes["aws-linux-node"].contains("planned"));
        assert!(summary.placement_detail.contains("aws-linux-node"));
        assert!(summary.placement_detail.contains("provider 'aws'"));
        assert!(!summary.placement_detail.contains("generic-linux-node"));
    }

    #[test]
    fn hosted_standard_summary_tracks_host_and_provider_for_each_demo_lane() {
        let config = PortConfig::sample();

        let generic = config
            .hosted_machine_summary_contract("cloud-generic")
            .expect("cloud-generic summary should resolve")
            .expect("cloud-generic should be hosted");
        assert_eq!(generic.host_name, "generic-linux");
        assert_eq!(generic.provider, HostProvider::GenericLinux);
        assert_eq!(
            generic.candidate_nodes,
            vec![String::from("generic-linux-node")]
        );
        assert!(generic.placement_detail.contains("host 'generic-linux'"));
        assert!(
            generic
                .placement_detail
                .contains("provider 'generic-linux'")
        );

        let gcp = config
            .hosted_machine_summary_contract("cloud-gcp")
            .expect("cloud-gcp summary should resolve")
            .expect("cloud-gcp should be hosted");
        assert_eq!(gcp.host_name, "gcp-linux");
        assert_eq!(gcp.provider, HostProvider::Gcp);
        assert_eq!(gcp.candidate_nodes, vec![String::from("gcp-linux-node")]);
        assert!(gcp.placement_detail.contains("host 'gcp-linux'"));
        assert!(gcp.placement_detail.contains("provider 'gcp'"));
    }

    #[test]
    fn hosted_standard_summary_reports_explicit_rejection_context() {
        let mut config = PortConfig::sample();
        config
            .nodes
            .get_mut("generic-linux-node")
            .expect("generic-linux-node should exist")
            .capabilities
            .protection_modes = vec![ProtectionMode::Pvm];

        let summary = config
            .hosted_machine_summary_contract("cloud-generic")
            .expect("cloud-generic summary should resolve")
            .expect("cloud-generic should be hosted");

        assert!(summary.candidate_nodes.is_empty());
        assert_eq!(
            summary.rejected_nodes["generic-linux-node"],
            "protection mode 'standard' is required but node advertises pvm"
        );
        assert!(summary.placement_detail.contains("generic-linux-node"));
        assert!(
            summary
                .placement_detail
                .contains("provider 'generic-linux'")
        );
        assert!(
            !summary
                .placement_detail
                .contains("run Port on that host directly")
        );
    }

    #[test]
    fn hosted_standard_summary_reports_missing_host_inventory_explicitly() {
        let mut config = PortConfig::sample();
        config.nodes.remove("generic-linux-node");
        config
            .host_groups
            .get_mut("remote-linux")
            .expect("remote-linux host group should exist")
            .nodes
            .retain(|node| node != "generic-linux-node");

        let error = config
            .hosted_machine_summary_contract("cloud-generic")
            .expect_err("missing hosted node inventory should fail resolution");

        assert!(error.to_string().contains(
            "machine 'cloud-generic' targets hosted host 'generic-linux' but no hosted node inventory record matches that host"
        ));
        assert!(!error.to_string().contains("run Port on that host directly"));
    }

    #[test]
    fn sample_config_derives_hosted_guest_attach_contract() {
        let config = PortConfig::sample();

        let contract = config
            .hosted_guest_attach_contract("cloud-aws")
            .expect("hosted guest attach contract should resolve")
            .expect("cloud-aws guest attach should be hosted");

        assert_eq!(contract.machine.control_plane, "demo");
        assert_eq!(
            contract.guest_broker,
            MachineGuestBroker::ControlPlaneNodeAgentTunnel
        );
        assert_eq!(
            contract.guest_route,
            MachineCommandRoute::HostedControlPlane
        );
        assert_eq!(
            contract.command_surface,
            vec![
                GuestCommandVerb::Exec,
                GuestCommandVerb::Copy,
                GuestCommandVerb::Pty,
                GuestCommandVerb::Logs,
                GuestCommandVerb::Forward
            ]
        );
        assert_eq!(
            contract.protocol,
            HostedGuestProtocolContract::PortAgentProtocol
        );
        assert_eq!(
            contract.attach_path,
            vec![
                HostedGuestAttachHop {
                    actor: HostedGuestAttachActor::Cli,
                    role: String::from("initiates a canonical `port guest ...` request"),
                },
                HostedGuestAttachHop {
                    actor: HostedGuestAttachActor::HostedControlPlane,
                    role: String::from("authorizes guest attachment and resolves the owning node",),
                },
                HostedGuestAttachHop {
                    actor: HostedGuestAttachActor::HostedNodeAgent,
                    role: String::from(
                        "opens the host-local guest transport and bridges the byte stream",
                    ),
                },
                HostedGuestAttachHop {
                    actor: HostedGuestAttachActor::GuestAgent,
                    role: String::from(
                        "continues serving the existing guest request and response frames",
                    ),
                },
            ]
        );
        assert!(
            contract
                .detail
                .contains("port guest exec|copy|pty|logs|forward")
        );
    }

    #[test]
    fn validate_rejects_hosted_node_on_local_host() {
        let mut config = PortConfig::sample();
        config.nodes.insert(
            String::from("bad-node"),
            super::HostedNodeSpec {
                host: String::from("local"),
                runtime_root: PathBuf::from("runtime/hosted/bad-node"),
                capabilities: super::HostedNodeCapabilities {
                    providers: vec![HostProvider::Local],
                    platforms: vec![super::HostPlatform::Linux],
                    substrates: vec![ExecutionSubstrate::Firecracker],
                    architectures: vec![MachineArchitecture::X86_64],
                    protection_modes: vec![ProtectionMode::Standard],
                    pvm_lanes: vec![],
                },
                notes: vec![],
            },
        );

        let error = config
            .validate()
            .expect_err("local hosted node should fail validation");

        assert!(
            error
                .to_string()
                .contains("hosted nodes must resolve through a hosted control plane")
        );
    }

    #[test]
    fn validate_rejects_hosted_node_without_runtime_root() {
        let mut config = PortConfig::sample();
        config
            .nodes
            .get_mut("aws-linux-node")
            .expect("aws-linux-node should exist")
            .runtime_root = PathBuf::new();

        let error = config
            .validate()
            .expect_err("missing runtime_root should fail validation");

        assert!(
            error
                .to_string()
                .contains("must declare a non-empty runtime_root")
        );
    }

    #[test]
    fn x86_firecracker_pvm_contract_requires_host_and_artifact_kits() {
        let contract = FirecrackerPvmLaneContract::for_architecture(MachineArchitecture::X86_64);

        assert_eq!(contract.decision, PvmLaneDecision::Planned);
        assert!(contract.host_kit.is_some());
        assert!(contract.artifact_kit.is_some());
        assert!(
            contract
                .host_kit
                .as_ref()
                .expect("x86 host kit should exist")
                .firecracker_binary_name
                == "firecracker-pvm"
        );
        assert_eq!(
            contract
                .host_kit
                .as_ref()
                .expect("x86 host kit should exist")
                .firecracker_binary_env
                .as_deref(),
            Some("PORT_PVM_FIRECRACKER_BINARY")
        );
        assert!(
            contract
                .host_kit
                .as_ref()
                .expect("x86 host kit should exist")
                .host_boot_args
                .contains(&String::from("pti=off"))
        );
        assert!(
            contract
                .validation
                .iter()
                .any(|check| check.name == "host-kernel")
        );
        assert!(
            contract
                .follow_on_work
                .iter()
                .any(|item| item.contains("port doctor"))
        );
    }

    #[test]
    fn arm64_firecracker_pvm_contract_is_research_only() {
        let contract = FirecrackerPvmLaneContract::for_architecture(MachineArchitecture::Aarch64);

        assert_eq!(contract.decision, PvmLaneDecision::ResearchOnly);
        assert!(contract.host_kit.is_none());
        assert!(contract.artifact_kit.is_none());
        assert!(
            contract.validation[0]
                .detail
                .contains("supportable Firecracker/PVM runtime path")
        );
    }

    #[test]
    fn local_firecracker_support_resolves_serialized_pvm_lanes_by_architecture() {
        let config = PortConfig::sample();
        let firecracker = &config.hosts["local"].firecracker;

        assert_eq!(
            firecracker
                .pvm_lane_for(MachineArchitecture::X86_64)
                .expect("x86_64 lane should exist")
                .decision,
            PvmLaneDecision::Planned
        );
        assert_eq!(
            firecracker
                .pvm_lane_for(MachineArchitecture::Aarch64)
                .expect("aarch64 lane should exist")
                .decision,
            PvmLaneDecision::ResearchOnly
        );
        assert_eq!(
            firecracker
                .pvm_lane_for(MachineArchitecture::X86_64)
                .expect("x86_64 lane should exist")
                .capability_state(),
            PvmCapabilityState::Planned
        );
    }

    #[test]
    fn sample_config_derives_hosted_node_pvm_capability_states() {
        let config = PortConfig::sample();

        let inventory = config
            .hosted_inventory_contract()
            .expect("hosted inventory contract should resolve");

        assert_eq!(
            inventory.nodes["aws-linux-node"].capabilities.pvm_lanes[0].state,
            PvmCapabilityState::Planned
        );
        assert_eq!(
            inventory.nodes["generic-linux-node"].capabilities.pvm_lanes[0].state,
            PvmCapabilityState::Planned
        );
        assert_eq!(
            inventory.nodes["gcp-linux-node"].capabilities.pvm_lanes[0].state,
            PvmCapabilityState::Planned
        );
    }

    #[test]
    fn sample_config_derives_hosted_node_pvm_host_kit_contracts() {
        let config = PortConfig::sample();

        let inventory = config
            .hosted_inventory_contract()
            .expect("hosted inventory contract should resolve");

        let aws_lane = &inventory.nodes["aws-linux-node"].capabilities.pvm_lanes[0];
        let aws_host_kit = aws_lane
            .host_kit
            .as_ref()
            .expect("planned hosted PVM lane should declare a host-kit contract");
        assert_eq!(aws_host_kit.host_platform, HostPlatform::Linux);
        assert_eq!(aws_host_kit.host_architecture, MachineArchitecture::X86_64);
        assert_eq!(
            aws_host_kit.firecracker_binary_name,
            String::from("firecracker-pvm")
        );
        assert_eq!(
            aws_host_kit.firecracker_binary_env.as_deref(),
            Some("PORT_PVM_FIRECRACKER_BINARY")
        );
        assert!(
            aws_host_kit
                .host_boot_args
                .contains(&String::from("pti=off"))
        );

        let generic_lane = &inventory.nodes["generic-linux-node"].capabilities.pvm_lanes[0];
        assert!(generic_lane.host_kit.is_none());
    }

    #[test]
    fn sample_config_derives_hosted_node_pvm_host_kit_package_identity() {
        let config = PortConfig::sample();

        let inventory = config
            .hosted_inventory_contract()
            .expect("hosted inventory contract should resolve");

        let aws_lane = &inventory.nodes["aws-linux-node"].capabilities.pvm_lanes[0];
        let aws_host_kit = aws_lane
            .host_kit
            .as_ref()
            .expect("planned hosted PVM lane should declare a host-kit contract");

        assert_eq!(aws_host_kit.package.name, "firecracker-pvm-host-kit");
        assert_eq!(aws_host_kit.package.version, "2026.04");
        assert_eq!(aws_host_kit.package.host_kernel_release, "6.12.0-port-pvm");
        assert_eq!(
            aws_host_kit.package.firecracker_build,
            "v1.13.0-dev+loopholelabs.pvm.7f6c070fa09c"
        );
    }

    #[test]
    fn ready_hosted_pvm_lane_requires_an_explicit_host_kit_contract() {
        let mut config = PortConfig::sample();
        config
            .nodes
            .get_mut("aws-linux-node")
            .expect("aws-linux-node should exist")
            .capabilities
            .pvm_lanes[0]
            .state = PvmCapabilityState::Ready;
        config
            .nodes
            .get_mut("aws-linux-node")
            .expect("aws-linux-node should exist")
            .capabilities
            .pvm_lanes[0]
            .host_kit = None;

        let error = config
            .validate()
            .expect_err("ready hosted PVM lane without host-kit should fail validation");

        assert!(
            error
                .to_string()
                .contains("must declare a host-kit contract")
        );
    }

    #[test]
    fn ready_hosted_pvm_lane_requires_host_kit_package_identity() {
        let mut config = PortConfig::sample();
        config
            .nodes
            .get_mut("aws-linux-node")
            .expect("aws-linux-node should exist")
            .capabilities
            .pvm_lanes[0]
            .state = PvmCapabilityState::Ready;
        let host_kit = config
            .nodes
            .get_mut("aws-linux-node")
            .expect("aws-linux-node should exist")
            .capabilities
            .pvm_lanes[0]
            .host_kit
            .as_mut()
            .expect("ready hosted PVM lane should declare a host-kit contract");
        host_kit.package.version.clear();

        let error = config
            .validate()
            .expect_err("ready hosted PVM lane without a package version should fail validation");

        assert!(error.to_string().contains("package"));
    }

    #[test]
    fn avf_contract_maps_guest_transport_and_console() {
        let contract = AvfExecutionContract::linux_guest();

        assert_eq!(contract.host_platform, super::HostPlatform::Macos);
        assert_eq!(contract.guest_transport, AvfGuestTransport::VirtioSocket);
        assert_eq!(contract.console_transport, AvfConsoleTransport::SerialPort);
        assert!(
            contract
                .supported_host_architectures
                .contains(&MachineArchitecture::Aarch64)
        );
        assert!(
            contract
                .launch_owners
                .contains(&AvfLaunchOwner::LocalPortRuntime)
        );
        assert!(contract.directory_share.supported);
        assert!(contract.directory_share.required_for_rosetta);
        assert!(contract.follow_on_work[0].contains("VZVirtualMachineConfiguration"));
    }

    #[test]
    fn validate_accepts_local_macos_standard_avf_machine_contract() {
        let config = sample_avf_config();

        config
            .validate()
            .expect("local macOS AVF contract should validate");
    }

    #[test]
    fn validate_rejects_non_macos_avf_machine_contract() {
        let mut config = sample_avf_config();
        config
            .hosts
            .get_mut("mac-local")
            .expect("mac-local host should exist")
            .platform = HostPlatform::Linux;

        let error = config
            .validate()
            .expect_err("non-macOS AVF machine should fail validation");

        assert!(
            error
                .to_string()
                .contains("Apple Virtualization Framework requires a macOS host platform")
        );
    }

    #[test]
    fn validate_rejects_hosted_control_plane_avf_machine_contract() {
        let mut config = sample_avf_config();
        config
            .hosts
            .get_mut("mac-local")
            .expect("mac-local host should exist")
            .connection = HostConnection::HostedControlPlane {
            control_plane: String::from("demo"),
        };

        let error = config
            .validate()
            .expect_err("hosted-control-plane AVF machine should fail validation");

        assert!(
            error
                .to_string()
                .contains("AVF local runtime currently requires a local host connection")
        );
    }

    #[test]
    fn validate_rejects_avf_pvm_machine_contract() {
        let mut config = sample_avf_config();
        config
            .machines
            .get_mut("demo")
            .expect("sample machine should exist")
            .protection_mode = ProtectionMode::Pvm;

        let error = config
            .validate()
            .expect_err("AVF/PVM machine should fail validation");

        assert!(
            error
                .to_string()
                .contains("Apple Virtualization Framework does not currently define a PVM lane")
        );
    }

    #[test]
    fn validate_accepts_firecracker_overlay_on_x86_64_read_only_rootfs() {
        let mut config = PortConfig::sample();
        let machine = config
            .machines
            .get_mut("cloud-aws")
            .expect("cloud-aws should exist");
        machine.architecture = MachineArchitecture::X86_64;
        machine.rootfs_read_only = true;
        machine.rootfs_overlay = Some(super::MachineRootfsOverlaySpec { size_mib: 4096 });

        config
            .validate()
            .expect("x86_64 Firecracker overlay contract should validate");
    }

    #[test]
    fn validate_rejects_rootfs_overlay_without_read_only_base() {
        let mut config = PortConfig::sample();
        config
            .machines
            .get_mut("cloud-aws")
            .expect("cloud-aws should exist")
            .rootfs_overlay = Some(super::MachineRootfsOverlaySpec { size_mib: 4096 });

        let error = config
            .validate()
            .expect_err("overlay without read-only rootfs should fail validation");

        assert!(error.to_string().contains("rootfs_read_only = true"));
    }

    #[test]
    fn validate_rejects_rootfs_overlay_on_non_x86_64_firecracker_guest() {
        let mut config = PortConfig::sample();
        let machine = config
            .machines
            .get_mut("cloud-aws")
            .expect("cloud-aws should exist");
        machine.architecture = MachineArchitecture::Aarch64;
        machine.rootfs_read_only = true;
        machine.rootfs_overlay = Some(super::MachineRootfsOverlaySpec { size_mib: 4096 });

        let error = config
            .validate()
            .expect_err("non-x86_64 overlay contract should fail validation");

        assert!(error.to_string().contains("currently requires x86_64"));
    }

    #[test]
    fn checked_in_example_models_all_provider_variants() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/port.toml")
            .canonicalize()
            .expect("example config path should resolve");
        let config = PortConfig::from_path(&path).expect("example config should parse");

        assert_eq!(config.hosts["local"].provider, HostProvider::Local);
        assert_eq!(config.clusters["demo"].provider, ClusterProvider::Local);
        assert_eq!(config.clusters["demo"].machine, "demo");
        assert_eq!(
            config.hosts["generic-linux"].provider,
            HostProvider::GenericLinux
        );
        assert_eq!(config.hosts["aws-linux"].provider, HostProvider::Aws);
        assert_eq!(config.hosts["gcp-linux"].provider, HostProvider::Gcp);
        assert_eq!(config.hosts["azure-linux"].provider, HostProvider::Azure);
        assert_eq!(config.hosts["mac-local"].platform, HostPlatform::Macos);
        assert_eq!(config.machines["cloud-azure"].host, "azure-linux");
        assert_eq!(config.machines["demo-avf"].host, "mac-local");
        assert!(config.control_planes.contains_key("demo"));
        assert!(config.nodes.contains_key("aws-linux-node"));
        assert_eq!(
            config.nodes["aws-linux-node"].runtime_root,
            PathBuf::from("runtime/hosted/aws-linux-node")
        );
        assert_eq!(
            config.nodes["aws-linux-node"].capabilities.pvm_lanes[0].state,
            PvmCapabilityState::Planned
        );
        assert_eq!(
            config.nodes["aws-linux-node"].capabilities.pvm_lanes[0]
                .host_kit
                .as_ref()
                .expect("aws node should declare a host-kit contract")
                .firecracker_binary_name,
            String::from("firecracker-pvm")
        );
        assert_eq!(
            config.host_groups["aws-builders"].scheduler,
            HostedSchedulerPolicy::DeterministicFirstFit
        );
        assert_eq!(
            config.nodes["generic-linux-node"].capabilities.pvm_lanes[0].state,
            PvmCapabilityState::Planned
        );
        assert!(config.host_groups.contains_key("remote-linux"));
        assert_eq!(
            config.hosts["generic-linux"].connection,
            HostConnection::HostedControlPlane {
                control_plane: String::from("demo")
            }
        );
        assert_eq!(
            config.machines["demo"].substrate,
            ExecutionSubstrate::Firecracker
        );
        assert_eq!(
            config.machines["demo-avf"].substrate,
            ExecutionSubstrate::Avf
        );
        assert_eq!(
            config.machines["demo"].protection_mode,
            ProtectionMode::Standard
        );
        assert_eq!(config.hosts["local"].firecracker.pvm_lanes.len(), 2);
        assert_eq!(
            config.hosts["local"].firecracker.pvm_lanes[0].architecture,
            MachineArchitecture::X86_64
        );
        assert_eq!(
            config.hosts["local"].firecracker.pvm_lanes[0].decision,
            PvmLaneDecision::Planned
        );
        assert_eq!(
            config.hosts["local"].firecracker.pvm_lanes[1].architecture,
            MachineArchitecture::Aarch64
        );
        assert_eq!(
            config.hosts["local"].firecracker.pvm_lanes[1].decision,
            PvmLaneDecision::ResearchOnly
        );
        assert_eq!(
            config.artifacts.kernels["demo-kernel"].variants[0]
                .selector
                .architecture,
            MachineArchitecture::X86_64
        );
        assert!(config.artifacts.kernels["demo-kernel"].supports(
            MachineArchitecture::X86_64,
            ExecutionSubstrate::Firecracker,
            ProtectionMode::Pvm
        ));
        assert!(config.artifacts.kernels["demo-kernel"].supports(
            MachineArchitecture::X86_64,
            ExecutionSubstrate::Avf,
            ProtectionMode::Standard
        ));
        assert_eq!(
            config.artifacts.guest_images["demo-guest"].variants[0]
                .selector
                .substrate,
            ExecutionSubstrate::Firecracker
        );
        assert!(config.artifacts.guest_images["demo-guest"].supports(
            MachineArchitecture::X86_64,
            ExecutionSubstrate::Firecracker,
            ProtectionMode::Pvm
        ));
        assert!(config.artifacts.guest_images["demo-guest"].supports(
            MachineArchitecture::Aarch64,
            ExecutionSubstrate::Avf,
            ProtectionMode::Standard
        ));
    }

    #[test]
    fn from_path_tracks_config_directory_as_state_root() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/port.toml")
            .canonicalize()
            .expect("example config path should resolve");
        let config = PortConfig::from_path(&path).expect("example config should parse");

        assert_eq!(config.state_root(), path.parent());
    }

    #[test]
    fn hosted_host_group_scheduler_field_is_required() {
        let encoded = PortConfig::sample()
            .to_toml_string()
            .expect("sample config should encode");
        let invalid = encoded.replacen("scheduler = \"deterministic-first-fit\"\n", "", 1);

        let error = PortConfig::from_toml_str(&invalid)
            .expect_err("missing host-group scheduler should fail parsing");

        assert!(error.to_string().contains("scheduler"));
    }

    #[test]
    fn hosted_host_group_scheduler_value_must_be_known() {
        let encoded = PortConfig::sample()
            .to_toml_string()
            .expect("sample config should encode");
        let invalid = encoded.replacen(
            "scheduler = \"deterministic-first-fit\"",
            "scheduler = \"not-a-policy\"",
            1,
        );

        let error = PortConfig::from_toml_str(&invalid)
            .expect_err("invalid host-group scheduler should fail parsing");

        assert!(error.to_string().contains("not-a-policy"));
    }

    #[test]
    fn artifact_compatibility_supports_x86_pvm_and_rejects_arm64_pvm() {
        let config = PortConfig::sample();
        let kernel = &config.artifacts.kernels["demo-kernel"];
        let guest = &config.artifacts.guest_images["demo-guest"];

        assert!(kernel.supports(
            MachineArchitecture::X86_64,
            ExecutionSubstrate::Firecracker,
            ProtectionMode::Pvm
        ));
        assert!(guest.supports(
            MachineArchitecture::X86_64,
            ExecutionSubstrate::Firecracker,
            ProtectionMode::Pvm
        ));
        assert!(!kernel.supports(
            MachineArchitecture::Aarch64,
            ExecutionSubstrate::Firecracker,
            ProtectionMode::Pvm
        ));
        assert!(guest.supports(
            MachineArchitecture::X86_64,
            ExecutionSubstrate::Firecracker,
            ProtectionMode::Standard
        ));
        assert!(!guest.supports(
            MachineArchitecture::Aarch64,
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
        assert!(
            kernel
                .variant(
                    MachineArchitecture::Aarch64,
                    ExecutionSubstrate::Firecracker,
                    ProtectionMode::Standard
                )
                .is_some()
        );
        assert!(
            kernel
                .variant(
                    MachineArchitecture::X86_64,
                    ExecutionSubstrate::Firecracker,
                    ProtectionMode::Pvm
                )
                .is_some()
        );
    }
}
