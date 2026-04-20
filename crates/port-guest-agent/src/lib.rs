use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::net::{Shutdown, TcpStream};
use std::os::unix::net::{UnixListener, UnixStream};
#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;
use std::path::{Component, Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::{Arc, Mutex, TryLockError, mpsc};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, anyhow, bail};
use port_agent_protocol::{
    CopyRequest, ExecRequest, ExecResult, ForwardEndpoint, ForwardRequest, GuestOperation,
    LogsRequest, LogsResult, ManagedServiceKind, ManagedServiceOperation, ManagedServiceRequest,
    ManagedServiceResult, ManagedServiceRuntimeState, ManagedServiceStatus, OperationResult,
    PtyRequest, PtyResult, RequestEnvelope, ResponseEnvelope, StreamKind, StreamOutputChannel,
    StreamRequestFrame, StreamResponseFrame, parse_forward_endpoint, read_frame, write_frame,
};
use port_model::{ServiceHealthPolicy, ServiceHealthState, ServicePolicy, ServiceRestartPolicy};
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use serde::{Deserialize, Serialize};
use vsock::{VMADDR_CID_ANY, VsockListener, VsockStream};

#[cfg(not(test))]
const MANAGED_PROCESS_RECONCILE_INTERVAL: Duration = Duration::from_secs(5);
#[cfg(test)]
const MANAGED_PROCESS_RECONCILE_INTERVAL: Duration = Duration::from_millis(50);

#[cfg(not(test))]
const MANAGED_PROCESS_HEALTH_RESTART_GRACE_PERIOD: Duration = Duration::from_secs(120);
#[cfg(test)]
const MANAGED_PROCESS_HEALTH_RESTART_GRACE_PERIOD: Duration = Duration::from_millis(250);

const MANAGED_PROCESS_UNHEALTHY_RESTART_THRESHOLD: u32 = 3;
#[cfg(not(test))]
const MANAGED_PROCESS_HEALTH_COMMAND_TIMEOUT: Duration = Duration::from_secs(30);
#[cfg(test)]
const MANAGED_PROCESS_HEALTH_COMMAND_TIMEOUT: Duration = Duration::from_millis(500);
#[cfg(not(test))]
const MANAGED_PROCESS_HEALTH_COMMAND_POLL_INTERVAL: Duration = Duration::from_millis(100);
#[cfg(test)]
const MANAGED_PROCESS_HEALTH_COMMAND_POLL_INTERVAL: Duration = Duration::from_millis(10);

/// Upper bound on the time a healthy guest-agent takes to answer a
/// `GuestOperation::Ping` request on its main dispatch path. Node-agent probes
/// size their timeout against this, and the test suite asserts handler latency
/// stays under it.
pub const PING_RESPONSE_BUDGET: Duration = Duration::from_millis(100);
const MANAGED_PROCESS_EVIDENCE_RETENTION_LIMIT: usize = 5;
#[cfg(not(test))]
const MANAGED_PROCESS_EVIDENCE_LOG_SETTLE_INTERVAL: Duration = Duration::from_millis(100);
#[cfg(test)]
const MANAGED_PROCESS_EVIDENCE_LOG_SETTLE_INTERVAL: Duration = Duration::from_millis(20);

#[derive(Debug, Default)]
struct ManagedProcessSupervisor {
    processes: BTreeMap<String, ManagedProcessHandle>,
}

#[derive(Debug)]
struct ManagedProcessHandle {
    record: ManagedProcessRecord,
    command: Vec<String>,
    env: BTreeMap<String, String>,
    cwd: PathBuf,
    policy: ServicePolicy,
    child: Child,
    started_at: Instant,
    consecutive_unhealthy_checks: u32,
}

#[derive(Debug)]
enum ManagedServiceHealthEvaluation {
    State(ServiceHealthState, Option<String>),
    ManagedProcessExited(ExitStatus),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ManagedProcessRecord {
    name: String,
    kind: ManagedServiceKind,
    state: ManagedServiceRuntimeState,
    #[serde(default)]
    restart_count: u32,
    pid: Option<u32>,
    exit_code: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    last_exit_code: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    last_exit_detail: Option<String>,
    #[serde(default)]
    health_state: ServiceHealthState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    health_detail: Option<String>,
    stdout_path: String,
    stderr_path: String,
    detail: String,
}

#[derive(Debug, Clone)]
pub struct AgentService {
    root: PathBuf,
    supervisor: Arc<Mutex<ManagedProcessSupervisor>>,
}

impl AgentService {
    #[must_use]
    pub fn new(root: PathBuf) -> Self {
        let supervisor = Arc::new(Mutex::new(ManagedProcessSupervisor::default()));
        spawn_managed_process_reconciler(root.clone(), Arc::clone(&supervisor));
        Self { root, supervisor }
    }

    pub fn handle(&self, request: RequestEnvelope) -> ResponseEnvelope {
        let id = request.id;
        match self.handle_non_streaming_operation(request.operation) {
            Ok((exit_code, result)) => ResponseEnvelope::Completed {
                id,
                exit_code,
                result,
            },
            Err(error) => ResponseEnvelope::Failed {
                id,
                message: error.to_string(),
            },
        }
    }

    fn handle_non_streaming_operation(
        &self,
        operation: GuestOperation,
    ) -> Result<(i32, OperationResult)> {
        match operation {
            GuestOperation::Exec(request) => self.exec(request),
            GuestOperation::Pty(request) => self.pty(request),
            GuestOperation::Logs(request) => self.logs(request),
            GuestOperation::ManagedService(request) => self.managed_service(request),
            GuestOperation::Ping => Ok((0, OperationResult::Pong)),
            GuestOperation::Copy(_) | GuestOperation::Forward(_) => {
                bail!("operation requires a streaming guest-agent connection")
            }
        }
    }

    fn exec(&self, request: ExecRequest) -> Result<(i32, OperationResult)> {
        let (program, args) = request
            .command
            .split_first()
            .ok_or_else(|| anyhow!("exec request requires a command"))?;

        let cwd = request
            .cwd
            .as_deref()
            .map(|path| self.resolve_guest_path(path))
            .transpose()?
            .unwrap_or_else(|| self.root.clone());

        let output = std::process::Command::new(program)
            .args(args)
            .current_dir(cwd)
            .envs(request.env)
            .output()
            .with_context(|| format!("failed to spawn '{}'", program))?;

        let exit_code = output.status.code().unwrap_or(1);
        let result = OperationResult::Exec(ExecResult {
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        });

        Ok((exit_code, result))
    }

    fn pty(&self, request: PtyRequest) -> Result<(i32, OperationResult)> {
        let (program, args) = request
            .command
            .split_first()
            .ok_or_else(|| anyhow!("pty request requires a command"))?;
        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows: request.rows,
                cols: request.cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .context("failed to allocate PTY")?;
        let mut builder = CommandBuilder::new(program);
        builder.args(args);
        builder.cwd(&self.root);
        let mut child = pair
            .slave
            .spawn_command(builder)
            .context("failed to spawn PTY command")?;
        drop(pair.slave);

        let mut reader = pair
            .master
            .try_clone_reader()
            .context("failed to clone PTY reader")?;
        let status = child.wait().context("failed to wait for PTY child")?;
        let mut transcript = String::new();
        reader
            .read_to_string(&mut transcript)
            .context("failed to read PTY transcript")?;

        let exit_code =
            i32::try_from(status.exit_code()).context("PTY child exit code overflowed i32")?;
        let result = OperationResult::Pty(PtyResult { transcript });
        Ok((exit_code, result))
    }

    fn logs(&self, request: LogsRequest) -> Result<(i32, OperationResult)> {
        let path = self.resolve_guest_path(&request.path)?;
        let contents = fs::read_to_string(&path)
            .with_context(|| format!("failed to read log '{}'", path.display()))?;
        let contents = if let Some(tail_lines) = request.tail_lines {
            tail(&contents, tail_lines as usize)
        } else {
            contents
        };

        let result = OperationResult::Logs(LogsResult { contents });
        Ok((0, result))
    }

    fn managed_service(&self, request: ManagedServiceRequest) -> Result<(i32, OperationResult)> {
        match request.operation {
            ManagedServiceOperation::Start {
                name,
                kind,
                command,
                env,
                cwd,
                policy,
            } => self.start_managed_service(name, kind, command, env, cwd, policy),
            ManagedServiceOperation::List => self.list_managed_services(),
            ManagedServiceOperation::Status { name } => self.status_managed_service(&name),
            ManagedServiceOperation::Stop { name } => self.stop_managed_service(&name),
        }
    }

    fn start_managed_service(
        &self,
        name: String,
        kind: ManagedServiceKind,
        command: Vec<String>,
        env: BTreeMap<String, String>,
        cwd: Option<String>,
        policy: ServicePolicy,
    ) -> Result<(i32, OperationResult)> {
        validate_managed_service_name(&name)?;
        let cwd = cwd
            .as_deref()
            .map(|path| self.resolve_guest_path(path))
            .transpose()?
            .unwrap_or_else(|| self.root.clone());
        let mut child = spawn_managed_process(&self.root, &name, &command, &env, &cwd)?;
        let stdout_relative = managed_service_stdout_relative_path(&name);
        let stderr_relative = managed_service_stderr_relative_path(&name);

        let mut record = ManagedProcessRecord {
            name: name.clone(),
            kind,
            state: ManagedServiceRuntimeState::Running,
            restart_count: 0,
            pid: Some(child.id()),
            exit_code: None,
            last_exit_code: None,
            last_exit_detail: None,
            health_state: ServiceHealthState::Unknown,
            health_detail: None,
            stdout_path: guest_visible_path(&stdout_relative),
            stderr_path: guest_visible_path(&stderr_relative),
            detail: String::from("managed process is running"),
        };
        if let Some(status) = child
            .try_wait()
            .context("failed to inspect managed service state after spawn")?
        {
            record.pid = None;
            record.exit_code = status.code();
            record.state = if status.success() {
                ManagedServiceRuntimeState::Exited
            } else {
                ManagedServiceRuntimeState::Failed
            };
            record.detail = format!(
                "managed process exited immediately with code {}",
                record.exit_code.unwrap_or(1)
            );
        }
        self.persist_managed_process_record(&record)?;

        let mut supervisor = self
            .supervisor
            .lock()
            .map_err(|_| anyhow!("managed process supervisor lock was poisoned"))?;
        if let Some(existing) = supervisor.processes.get_mut(&name) {
            refresh_managed_process_handle_liveness(&self.root, existing)?;
            if existing.record.state == ManagedServiceRuntimeState::Running {
                bail!("managed service '{}' is already running", name);
            }
        }
        if record.state == ManagedServiceRuntimeState::Running {
            let mut handle = ManagedProcessHandle {
                record,
                command,
                env,
                cwd,
                policy,
                child,
                started_at: Instant::now(),
                consecutive_unhealthy_checks: 0,
            };
            refresh_managed_process_handle_liveness(&self.root, &mut handle)?;
            let status = managed_service_status(&handle.record);
            supervisor.processes.insert(name.clone(), handle);
            Ok((
                0,
                OperationResult::ManagedService(ManagedServiceResult::Status(status)),
            ))
        } else {
            let status = managed_service_status(&record);
            supervisor.processes.remove(&name);
            Ok((
                0,
                OperationResult::ManagedService(ManagedServiceResult::Status(status)),
            ))
        }
    }

    fn list_managed_services(&self) -> Result<(i32, OperationResult)> {
        let services = self.managed_service_statuses()?;
        Ok((
            0,
            OperationResult::ManagedService(ManagedServiceResult::List { services }),
        ))
    }

    fn status_managed_service(&self, name: &str) -> Result<(i32, OperationResult)> {
        validate_managed_service_name(name)?;
        let status = self.managed_service_status_by_name(name)?;
        Ok((
            0,
            OperationResult::ManagedService(ManagedServiceResult::Status(status)),
        ))
    }

    fn stop_managed_service(&self, name: &str) -> Result<(i32, OperationResult)> {
        validate_managed_service_name(name)?;
        let mut supervisor = self
            .supervisor
            .lock()
            .map_err(|_| anyhow!("managed process supervisor lock was poisoned"))?;
        let Some(handle) = supervisor.processes.get_mut(name) else {
            drop(supervisor);
            let status = self.managed_service_status_by_name(name)?;
            return Ok((
                0,
                OperationResult::ManagedService(ManagedServiceResult::Status(status)),
            ));
        };

        refresh_managed_process_handle_liveness(&self.root, handle)?;
        if handle.record.state != ManagedServiceRuntimeState::Running {
            let status = managed_service_status(&handle.record);
            return Ok((
                0,
                OperationResult::ManagedService(ManagedServiceResult::Status(status)),
            ));
        }

        terminate_child(&mut handle.child)?;
        let exit_status = wait_for_child_exit(&mut handle.child)?;
        handle.record.state = ManagedServiceRuntimeState::Stopped;
        handle.record.pid = None;
        handle.record.exit_code = exit_status.code();
        handle.record.last_exit_code = exit_status.code();
        handle.record.last_exit_detail = Some(String::from("managed process stopped"));
        handle.record.health_state = ServiceHealthState::Unknown;
        handle.record.health_detail = None;
        handle.record.detail = String::from("managed process stopped");
        self.persist_managed_process_record(&handle.record)?;
        let status = managed_service_status(&handle.record);
        supervisor.processes.remove(name);
        Ok((
            0,
            OperationResult::ManagedService(ManagedServiceResult::Status(status)),
        ))
    }

    fn resolve_guest_path(&self, input: impl AsRef<Path>) -> Result<PathBuf> {
        let input = input.as_ref();
        let mut relative = PathBuf::new();
        for component in input.components() {
            match component {
                Component::RootDir | Component::CurDir => {}
                Component::ParentDir => bail!(
                    "parent path segments are not allowed: '{}'",
                    input.display()
                ),
                Component::Normal(part) => relative.push(part),
                Component::Prefix(_) => {
                    bail!("path prefixes are not supported: '{}'", input.display())
                }
            }
        }

        Ok(self.root.join(relative))
    }

    fn managed_service_root(&self) -> PathBuf {
        self.root.join("run/port/services")
    }

    fn managed_service_runtime_dir(&self) -> PathBuf {
        self.managed_service_root().join("runtime")
    }

    fn managed_service_status_by_name(&self, name: &str) -> Result<ManagedServiceStatus> {
        let statuses = self.managed_service_statuses()?;
        statuses
            .into_iter()
            .find(|status| status.name == name)
            .ok_or_else(|| anyhow!("managed service '{}' does not exist", name))
    }

    fn managed_service_statuses(&self) -> Result<Vec<ManagedServiceStatus>> {
        let mut supervisor = match self.supervisor.try_lock() {
            Ok(supervisor) => supervisor,
            Err(TryLockError::WouldBlock) => return self.load_managed_process_records(),
            Err(TryLockError::Poisoned(_)) => {
                bail!("managed process supervisor lock was poisoned")
            }
        };
        let mut live = BTreeMap::new();
        for (name, handle) in &mut supervisor.processes {
            refresh_managed_process_handle_liveness(&self.root, handle)?;
            live.insert(name.clone(), managed_service_status(&handle.record));
        }
        supervisor
            .processes
            .retain(|_, handle| handle.record.state == ManagedServiceRuntimeState::Running);
        drop(supervisor);

        let mut statuses = self.load_managed_process_records()?;
        statuses.retain(|status| !live.contains_key(&status.name));
        statuses.extend(live.into_values());
        statuses.sort_by(|left, right| left.name.cmp(&right.name));
        Ok(statuses)
    }

    fn load_managed_process_records(&self) -> Result<Vec<ManagedServiceStatus>> {
        let runtime_dir = self.managed_service_runtime_dir();
        if !runtime_dir.exists() {
            return Ok(Vec::new());
        }

        let mut statuses = Vec::new();
        for entry in fs::read_dir(&runtime_dir)
            .with_context(|| format!("failed to read '{}'", runtime_dir.display()))?
        {
            let entry =
                entry.with_context(|| format!("failed to inspect '{}'", runtime_dir.display()))?;
            if !entry
                .file_type()
                .with_context(|| format!("failed to inspect '{}'", entry.path().display()))?
                .is_file()
            {
                continue;
            }
            let file = File::open(entry.path())
                .with_context(|| format!("failed to open '{}'", entry.path().display()))?;
            let mut record: ManagedProcessRecord = serde_json::from_reader(file)
                .with_context(|| format!("failed to decode '{}'", entry.path().display()))?;
            reconcile_detached_managed_process_record(&mut record);
            self.persist_managed_process_record(&record)?;
            statuses.push(managed_service_status(&record));
        }
        Ok(statuses)
    }

    fn persist_managed_process_record(&self, record: &ManagedProcessRecord) -> Result<()> {
        write_managed_process_record(&self.root, record)
    }
}

fn validate_managed_service_name(name: &str) -> Result<()> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        bail!("managed service name must not be empty");
    }
    if trimmed.contains('/') || trimmed.contains("..") {
        bail!("managed service name must not contain path traversal or '/' segments");
    }
    Ok(())
}

fn managed_service_stdout_relative_path(name: &str) -> PathBuf {
    PathBuf::from("run/port/services").join(format!("{name}.stdout.log"))
}

fn managed_service_stderr_relative_path(name: &str) -> PathBuf {
    PathBuf::from("run/port/services").join(format!("{name}.stderr.log"))
}

fn managed_service_evidence_relative_root() -> PathBuf {
    PathBuf::from("run/port/service-evidence")
}

fn managed_service_evidence_relative_dir(
    name: &str,
    captured_at_unix_ms: u128,
    restart_attempt: u32,
) -> PathBuf {
    managed_service_evidence_relative_root()
        .join(name)
        .join(format!("{captured_at_unix_ms}-restart-{restart_attempt}"))
}

fn guest_visible_path(relative: &Path) -> String {
    format!("/{}", relative.display())
}

fn managed_service_evidence_candidates(name: &str) -> Vec<PathBuf> {
    let mut candidates = vec![
        managed_service_stdout_relative_path(name),
        managed_service_stderr_relative_path(name),
    ];
    if matches!(name, "k3s-agent" | "k3s-server") {
        candidates.push(PathBuf::from(
            "var/lib/rancher/k3s/agent/containerd/containerd.log",
        ));
        candidates.push(PathBuf::from("var/log/port-agent.log"));
    }
    candidates
}

fn append_evidence_detail(detail: &str, evidence_path: &str) -> String {
    format!("{detail}; evidence captured at {evidence_path}")
}

fn write_evidence_file(root: &Path, relative: &Path, contents: impl AsRef<[u8]>) -> Result<()> {
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create '{}'", parent.display()))?;
    }
    fs::write(&path, contents)
        .with_context(|| format!("failed to write evidence file '{}'", path.display()))
}

fn copy_evidence_file(root: &Path, relative: &Path, evidence_relative_dir: &Path) -> Result<()> {
    let source = root.join(relative);
    if !source.exists() {
        return Ok(());
    }
    let destination = root.join(evidence_relative_dir).join(relative);
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create '{}'", parent.display()))?;
    }
    fs::copy(&source, &destination).with_context(|| {
        format!(
            "failed to copy evidence file '{}' to '{}'",
            source.display(),
            destination.display()
        )
    })?;
    Ok(())
}

fn prune_managed_process_evidence(root: &Path, name: &str) -> Result<()> {
    let service_root = root
        .join(managed_service_evidence_relative_root())
        .join(name);
    if !service_root.exists() {
        return Ok(());
    }

    let mut entries = fs::read_dir(&service_root)
        .with_context(|| format!("failed to read '{}'", service_root.display()))?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            entry
                .file_type()
                .ok()
                .filter(|file_type| file_type.is_dir())
                .map(|_| entry)
        })
        .collect::<Vec<_>>();
    entries.sort_by_key(|entry| entry.file_name());
    while entries.len() > MANAGED_PROCESS_EVIDENCE_RETENTION_LIMIT {
        let entry = entries.remove(0);
        fs::remove_dir_all(entry.path()).with_context(|| {
            format!(
                "failed to prune managed service evidence '{}'",
                entry.path().display()
            )
        })?;
    }
    Ok(())
}

fn capture_managed_process_evidence(
    root: &Path,
    record: &ManagedProcessRecord,
    reason: &str,
    restart_attempt: u32,
) -> Result<String> {
    let captured_at_unix_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let evidence_relative_dir =
        managed_service_evidence_relative_dir(&record.name, captured_at_unix_ms, restart_attempt);
    let evidence_guest_path = guest_visible_path(&evidence_relative_dir);
    fs::create_dir_all(root.join(&evidence_relative_dir)).with_context(|| {
        format!(
            "failed to create managed service evidence directory '{}'",
            root.join(&evidence_relative_dir).display()
        )
    })?;

    let metadata = format!(
        concat!(
            "name: {}\n",
            "restart_attempt: {}\n",
            "captured_at_unix_ms: {}\n",
            "reason: {}\n",
            "state: {:?}\n",
            "exit_code: {:?}\n",
            "last_exit_code: {:?}\n",
            "health_state: {:?}\n",
            "health_detail: {}\n",
            "detail: {}\n"
        ),
        record.name,
        restart_attempt,
        captured_at_unix_ms,
        reason,
        record.state,
        record.exit_code,
        record.last_exit_code,
        record.health_state,
        record.health_detail.as_deref().unwrap_or(""),
        record.detail
    );
    write_evidence_file(
        root,
        &evidence_relative_dir.join("metadata.txt"),
        metadata.as_bytes(),
    )?;

    let runtime_relative = evidence_relative_dir
        .join("run/port/services/runtime")
        .join(format!("{}.json", record.name));
    let runtime_json = serde_json::to_vec_pretty(record)
        .context("failed to encode managed service runtime record for evidence capture")?;
    write_evidence_file(
        root,
        &runtime_relative,
        format!("{}\n", String::from_utf8_lossy(&runtime_json)).as_bytes(),
    )?;

    for candidate in managed_service_evidence_candidates(&record.name) {
        copy_evidence_file(root, &candidate, &evidence_relative_dir)?;
    }
    prune_managed_process_evidence(root, &record.name)?;
    Ok(evidence_guest_path)
}

fn capture_managed_process_evidence_best_effort(
    root: &Path,
    record: &ManagedProcessRecord,
    reason: &str,
    restart_attempt: u32,
) -> Option<String> {
    match capture_managed_process_evidence(root, record, reason, restart_attempt) {
        Ok(path) => Some(path),
        Err(error) => {
            eprintln!(
                "port-guest-agent failed to capture evidence for managed service '{}': {error}",
                record.name
            );
            None
        }
    }
}

fn settle_managed_process_logs_after_exit() {
    thread::sleep(MANAGED_PROCESS_EVIDENCE_LOG_SETTLE_INTERVAL);
}

fn managed_service_status(record: &ManagedProcessRecord) -> ManagedServiceStatus {
    ManagedServiceStatus {
        name: record.name.clone(),
        kind: record.kind,
        state: record.state,
        restart_count: record.restart_count,
        pid: record.pid,
        exit_code: record.exit_code,
        last_exit_code: record.last_exit_code,
        last_exit_detail: record.last_exit_detail.clone(),
        health_state: record.health_state,
        health_detail: record.health_detail.clone(),
        stdout_path: Some(record.stdout_path.clone()),
        stderr_path: Some(record.stderr_path.clone()),
        detail: record.detail.clone(),
    }
}

fn reconcile_detached_managed_process_record(record: &mut ManagedProcessRecord) {
    if record.state != ManagedServiceRuntimeState::Running {
        return;
    }

    record.detail = match record.pid {
        Some(pid) => match managed_process_pid_is_live(pid) {
            Ok(true) => {
                record.health_state = ServiceHealthState::Unknown;
                record.health_detail = None;
                format!(
                    "managed process pid {pid} is live but guest-agent does not hold a supervisor handle"
                )
            }
            Ok(false) => {
                record.state = ManagedServiceRuntimeState::Failed;
                record.pid = None;
                record.exit_code = None;
                record.health_state = ServiceHealthState::Unknown;
                record.health_detail = None;
                let detail = format!(
                    "recorded managed process pid {pid} is no longer live and guest-agent does not hold a supervisor handle"
                );
                record.last_exit_detail = Some(detail.clone());
                detail
            }
            Err(error) => format!("failed to verify recorded managed process pid {pid}: {error}"),
        },
        None => {
            record.state = ManagedServiceRuntimeState::Failed;
            record.exit_code = None;
            record.health_state = ServiceHealthState::Unknown;
            record.health_detail = None;
            let detail = String::from(
                "managed process record claims running but does not record a live pid",
            );
            record.last_exit_detail = Some(detail.clone());
            detail
        }
    };
}

fn spawn_managed_process_reconciler(
    root: PathBuf,
    supervisor: Arc<Mutex<ManagedProcessSupervisor>>,
) {
    thread::spawn(move || {
        loop {
            thread::sleep(MANAGED_PROCESS_RECONCILE_INTERVAL);
            let mut supervisor = match supervisor.lock() {
                Ok(supervisor) => supervisor,
                Err(_) => {
                    eprintln!("port-guest-agent managed process supervisor lock was poisoned");
                    continue;
                }
            };
            for handle in supervisor.processes.values_mut() {
                if let Err(error) = refresh_managed_process_handle(&root, handle) {
                    eprintln!(
                        "port-guest-agent failed to reconcile managed service '{}': {error}",
                        handle.record.name
                    );
                }
            }
            supervisor
                .processes
                .retain(|_, handle| handle.record.state == ManagedServiceRuntimeState::Running);
        }
    });
}

fn refresh_managed_process_handle_liveness(
    root: &Path,
    handle: &mut ManagedProcessHandle,
) -> Result<()> {
    if handle.record.state != ManagedServiceRuntimeState::Running {
        write_managed_process_record(root, &handle.record)?;
        return Ok(());
    }

    if let Some(status) = handle
        .child
        .try_wait()
        .context("failed to inspect managed process state")?
    {
        handle_managed_process_exit(root, handle, status)?;
    }

    write_managed_process_record(root, &handle.record)
}

fn refresh_managed_process_handle(root: &Path, handle: &mut ManagedProcessHandle) -> Result<()> {
    refresh_managed_process_handle_liveness(root, handle)?;

    if handle.record.state == ManagedServiceRuntimeState::Running {
        match evaluate_managed_service_health(handle)? {
            ManagedServiceHealthEvaluation::State(health_state, health_detail) => {
                handle.record.health_state = health_state;
                handle.record.health_detail = health_detail;
                match health_state {
                    ServiceHealthState::Healthy | ServiceHealthState::Unknown => {
                        handle.consecutive_unhealthy_checks = 0;
                    }
                    ServiceHealthState::Unhealthy => {
                        handle.consecutive_unhealthy_checks =
                            handle.consecutive_unhealthy_checks.saturating_add(1);
                    }
                }
                if should_restart_unhealthy_managed_service(handle) {
                    let health_detail = handle
                        .record
                        .health_detail
                        .clone()
                        .unwrap_or_else(|| String::from("health check reported unhealthy"));
                    terminate_child(&mut handle.child)?;
                    let exit_status = wait_for_child_exit(&mut handle.child)?;
                    handle.record.pid = None;
                    handle.record.exit_code = exit_status.code();
                    handle.record.last_exit_code = exit_status.code();
                    let mut restart_detail = format!(
                        "managed process restarted after health check failure: {health_detail}"
                    );
                    settle_managed_process_logs_after_exit();
                    if let Some(evidence_path) = capture_managed_process_evidence_best_effort(
                        root,
                        &handle.record,
                        &restart_detail,
                        handle.record.restart_count.saturating_add(1),
                    ) {
                        restart_detail = append_evidence_detail(&restart_detail, &evidence_path);
                    }
                    handle.record.last_exit_detail = Some(restart_detail.clone());
                    restart_managed_process(root, handle, restart_detail)?;
                }
            }
            ManagedServiceHealthEvaluation::ManagedProcessExited(status) => {
                handle_managed_process_exit(root, handle, status)?;
            }
        }
    }
    write_managed_process_record(root, &handle.record)
}

fn handle_managed_process_exit(
    root: &Path,
    handle: &mut ManagedProcessHandle,
    status: ExitStatus,
) -> Result<()> {
    handle.record.pid = None;
    handle.record.exit_code = status.code();
    handle.record.state = if status.success() {
        ManagedServiceRuntimeState::Exited
    } else {
        ManagedServiceRuntimeState::Failed
    };
    let mut exit_detail = managed_process_exit_detail(&status);
    let mut restart_detail = managed_process_restart_detail(&status);
    handle.record.last_exit_code = handle.record.exit_code;
    handle.record.health_state = ServiceHealthState::Unknown;
    handle.record.health_detail = None;
    handle.record.detail = exit_detail.clone();
    if should_restart_managed_service(handle.policy.restart, handle.record.state) {
        settle_managed_process_logs_after_exit();
        if let Some(evidence_path) = capture_managed_process_evidence_best_effort(
            root,
            &handle.record,
            &exit_detail,
            handle.record.restart_count.saturating_add(1),
        ) {
            exit_detail = append_evidence_detail(&exit_detail, &evidence_path);
            restart_detail = append_evidence_detail(&restart_detail, &evidence_path);
        }
        handle.record.last_exit_detail = Some(exit_detail.clone());
        handle.record.detail = exit_detail;
        restart_managed_process(root, handle, restart_detail)?;
    } else {
        handle.record.last_exit_detail = Some(exit_detail.clone());
        handle.record.detail = exit_detail;
    }
    Ok(())
}

fn should_restart_managed_service(
    policy: ServiceRestartPolicy,
    state: ManagedServiceRuntimeState,
) -> bool {
    match policy {
        ServiceRestartPolicy::Never => false,
        ServiceRestartPolicy::OnFailure => matches!(state, ManagedServiceRuntimeState::Failed),
        ServiceRestartPolicy::Always => {
            matches!(
                state,
                ManagedServiceRuntimeState::Exited | ManagedServiceRuntimeState::Failed
            )
        }
    }
}

fn should_restart_unhealthy_managed_service(handle: &ManagedProcessHandle) -> bool {
    matches!(handle.policy.restart, ServiceRestartPolicy::Always)
        && handle.policy.healthcheck.restart_on_unhealthy
        && matches!(handle.record.health_state, ServiceHealthState::Unhealthy)
        && handle.started_at.elapsed() >= MANAGED_PROCESS_HEALTH_RESTART_GRACE_PERIOD
        && handle.consecutive_unhealthy_checks >= MANAGED_PROCESS_UNHEALTHY_RESTART_THRESHOLD
}

fn restart_managed_process(
    root: &Path,
    handle: &mut ManagedProcessHandle,
    running_detail: String,
) -> Result<()> {
    handle.record.restart_count += 1;
    match spawn_managed_process(
        root,
        &handle.record.name,
        &handle.command,
        &handle.env,
        &handle.cwd,
    ) {
        Ok(mut child) => {
            if let Some(status) = child
                .try_wait()
                .context("failed to inspect managed service state after restart")?
            {
                handle.record.pid = None;
                handle.record.exit_code = status.code();
                handle.record.state = if status.success() {
                    ManagedServiceRuntimeState::Exited
                } else {
                    ManagedServiceRuntimeState::Failed
                };
                handle.record.last_exit_code = handle.record.exit_code;
                handle.record.last_exit_detail = Some(format!(
                    "managed process exited immediately after restart with code {}",
                    handle.record.exit_code.unwrap_or(1)
                ));
                handle.record.detail = handle
                    .record
                    .last_exit_detail
                    .clone()
                    .unwrap_or_else(|| String::from("managed process restart failed"));
            } else {
                handle.record.state = ManagedServiceRuntimeState::Running;
                handle.record.pid = Some(child.id());
                handle.record.exit_code = None;
                handle.record.health_state = ServiceHealthState::Unknown;
                handle.record.health_detail = None;
                handle.record.detail = running_detail;
                handle.child = child;
                handle.started_at = Instant::now();
                handle.consecutive_unhealthy_checks = 0;
            }
        }
        Err(error) => {
            handle.record.state = ManagedServiceRuntimeState::Failed;
            handle.record.detail = format!(
                "managed process restart attempt {} failed: {error}",
                handle.record.restart_count
            );
        }
    }
    Ok(())
}

fn evaluate_managed_service_health(
    handle: &mut ManagedProcessHandle,
) -> Result<ManagedServiceHealthEvaluation> {
    match handle.policy.healthcheck.policy {
        ServiceHealthPolicy::None => Ok(ManagedServiceHealthEvaluation::State(
            ServiceHealthState::Unknown,
            None,
        )),
        ServiceHealthPolicy::Command => {
            let (program, args) = handle
                .policy
                .healthcheck
                .command
                .split_first()
                .ok_or_else(|| anyhow!("managed service health check requires a command"))?;
            let mut child = Command::new(program)
                .args(args)
                .current_dir(&handle.cwd)
                .envs(&handle.env)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn();
            match child {
                Ok(ref mut child) => {
                    let started = Instant::now();
                    loop {
                        if let Some(status) = handle.child.try_wait().context(
                            "failed to inspect managed process state during health check",
                        )? {
                            reap_health_command_best_effort(child);
                            return Ok(ManagedServiceHealthEvaluation::ManagedProcessExited(
                                status,
                            ));
                        }
                        if let Some(status) = child
                            .try_wait()
                            .context("failed to inspect managed service health command state")?
                        {
                            return if status.success() {
                                Ok(ManagedServiceHealthEvaluation::State(
                                    ServiceHealthState::Healthy,
                                    None,
                                ))
                            } else {
                                Ok(ManagedServiceHealthEvaluation::State(
                                    ServiceHealthState::Unhealthy,
                                    Some(format!(
                                        "health command exited with code {}",
                                        status.code().unwrap_or(1)
                                    )),
                                ))
                            };
                        }
                        if started.elapsed() >= MANAGED_PROCESS_HEALTH_COMMAND_TIMEOUT {
                            reap_health_command_best_effort(child);
                            return Ok(ManagedServiceHealthEvaluation::State(
                                ServiceHealthState::Unhealthy,
                                Some(format!(
                                    "health command timed out after {:?}",
                                    MANAGED_PROCESS_HEALTH_COMMAND_TIMEOUT
                                )),
                            ));
                        }
                        thread::sleep(MANAGED_PROCESS_HEALTH_COMMAND_POLL_INTERVAL);
                    }
                }
                Err(error) => Ok(ManagedServiceHealthEvaluation::State(
                    ServiceHealthState::Unhealthy,
                    Some(format!("health command failed: {error}")),
                )),
            }
        }
    }
}

fn reap_health_command_best_effort(child: &mut Child) {
    if matches!(child.try_wait(), Ok(Some(_))) {
        return;
    }
    let _ = child.kill();
    let _ = child.wait();
}

fn managed_process_exit_detail(status: &ExitStatus) -> String {
    if let Some(code) = status.code() {
        return format!("managed process exited with code {code}");
    }
    managed_process_signal_detail(status, "managed process terminated")
        .unwrap_or_else(|| String::from("managed process terminated without an exit code"))
}

fn managed_process_restart_detail(status: &ExitStatus) -> String {
    if let Some(code) = status.code() {
        return format!("managed process restarted after exit code {code}");
    }
    managed_process_signal_detail(status, "managed process restarted after termination")
        .unwrap_or_else(|| String::from("managed process restarted after abnormal termination"))
}

#[cfg(unix)]
fn managed_process_signal_detail(status: &ExitStatus, prefix: &str) -> Option<String> {
    let signal = status.signal()?;
    let mut detail = format!("{prefix} by signal {signal}");
    if let Some(name) = managed_process_signal_name(signal) {
        detail.push_str(&format!(" ({name})"));
    }
    if status.core_dumped() {
        detail.push_str(" (core dumped)");
    }
    Some(detail)
}

#[cfg(not(unix))]
fn managed_process_signal_detail(_status: &ExitStatus, _prefix: &str) -> Option<String> {
    None
}

#[cfg(unix)]
fn managed_process_signal_name(signal: i32) -> Option<&'static str> {
    match signal {
        libc::SIGABRT => Some("SIGABRT"),
        libc::SIGALRM => Some("SIGALRM"),
        libc::SIGBUS => Some("SIGBUS"),
        libc::SIGFPE => Some("SIGFPE"),
        libc::SIGHUP => Some("SIGHUP"),
        libc::SIGILL => Some("SIGILL"),
        libc::SIGINT => Some("SIGINT"),
        libc::SIGKILL => Some("SIGKILL"),
        libc::SIGPIPE => Some("SIGPIPE"),
        libc::SIGQUIT => Some("SIGQUIT"),
        libc::SIGSEGV => Some("SIGSEGV"),
        libc::SIGTERM => Some("SIGTERM"),
        libc::SIGTRAP => Some("SIGTRAP"),
        _ => None,
    }
}

fn spawn_managed_process(
    root: &Path,
    name: &str,
    command: &[String],
    env: &BTreeMap<String, String>,
    cwd: &Path,
) -> Result<Child> {
    let (program, args) = command
        .split_first()
        .ok_or_else(|| anyhow!("managed service start requires a command"))?;
    let log_dir = root.join("run/port/services");
    let runtime_dir = log_dir.join("runtime");
    fs::create_dir_all(&log_dir)
        .with_context(|| format!("failed to create '{}'", log_dir.display()))?;
    fs::create_dir_all(&runtime_dir)
        .with_context(|| format!("failed to create '{}'", runtime_dir.display()))?;

    let stdout_relative = managed_service_stdout_relative_path(name);
    let stderr_relative = managed_service_stderr_relative_path(name);
    let stdout_path = root.join(&stdout_relative);
    let stderr_path = root.join(&stderr_relative);
    if stdout_path.exists() {
        fs::remove_file(&stdout_path)
            .with_context(|| format!("failed to remove '{}'", stdout_path.display()))?;
    }
    if stderr_path.exists() {
        fs::remove_file(&stderr_path)
            .with_context(|| format!("failed to remove '{}'", stderr_path.display()))?;
    }
    let stdout_writer = File::create(&stdout_path)
        .with_context(|| format!("failed to create '{}'", stdout_path.display()))?;
    let stderr_writer = File::create(&stderr_path)
        .with_context(|| format!("failed to create '{}'", stderr_path.display()))?;

    let redactions = env
        .values()
        .filter(|value| !value.is_empty())
        .cloned()
        .collect::<Vec<_>>();

    let mut child = Command::new(program)
        .args(args)
        .current_dir(cwd)
        .envs(env)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("failed to spawn managed service '{name}'"))?;

    let stdout_reader = child
        .stdout
        .take()
        .context("managed service stdout was not piped")?;
    let stderr_reader = child
        .stderr
        .take()
        .context("managed service stderr was not piped")?;
    spawn_redacted_log_pump(stdout_reader, stdout_writer, redactions.clone());
    spawn_redacted_log_pump(stderr_reader, stderr_writer, redactions);
    Ok(child)
}

fn write_managed_process_record(root: &Path, record: &ManagedProcessRecord) -> Result<()> {
    let runtime_dir = root.join("run/port/services/runtime");
    fs::create_dir_all(&runtime_dir)
        .with_context(|| format!("failed to create '{}'", runtime_dir.display()))?;
    let path = runtime_dir.join(format!("{}.json", record.name));
    let temp_path = runtime_dir.join(format!("{}.json.tmp", record.name));
    let bytes = serde_json::to_vec_pretty(record)
        .with_context(|| format!("failed to encode '{}'", path.display()))?;
    fs::write(&temp_path, format!("{}\n", String::from_utf8_lossy(&bytes)))
        .with_context(|| format!("failed to write '{}'", temp_path.display()))?;
    fs::rename(&temp_path, &path).with_context(|| {
        format!(
            "failed to move managed process record '{}' into place",
            path.display()
        )
    })
}

fn spawn_redacted_log_pump<R>(reader: R, writer: File, redactions: Vec<String>)
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let _ = copy_redacted_output(reader, writer, redactions);
    });
}

fn copy_redacted_output<R>(reader: R, mut writer: File, redactions: Vec<String>) -> Result<()>
where
    R: Read,
{
    let mut reader = BufReader::new(reader);
    let mut buffer = Vec::new();
    loop {
        buffer.clear();
        let bytes_read = reader
            .read_until(b'\n', &mut buffer)
            .context("failed to read managed process output")?;
        if bytes_read == 0 {
            break;
        }
        let mut chunk = String::from_utf8_lossy(&buffer).into_owned();
        for secret in &redactions {
            if !secret.is_empty() {
                chunk = chunk.replace(secret, "[redacted]");
            }
        }
        writer
            .write_all(chunk.as_bytes())
            .context("failed to write managed process log output")?;
        writer
            .flush()
            .context("failed to flush managed process log output")?;
    }
    Ok(())
}

fn terminate_child(child: &mut Child) -> Result<()> {
    let pid = child.id() as i32;
    // SAFETY: `kill(pid, SIGTERM)` targets the managed child process id only.
    let status = unsafe { libc::kill(pid, libc::SIGTERM) };
    if status != 0 {
        let error = std::io::Error::last_os_error();
        bail!(
            "failed to signal managed process pid {}: {}",
            child.id(),
            error
        );
    }
    Ok(())
}

fn wait_for_child_exit(child: &mut Child) -> Result<std::process::ExitStatus> {
    for _ in 0..100 {
        if let Some(status) = child
            .try_wait()
            .context("failed to wait for managed process exit")?
        {
            return Ok(status);
        }
        thread::sleep(Duration::from_millis(20));
    }
    child.kill().context("failed to kill managed process")?;
    child.wait().context("failed to reap managed process")
}

fn managed_process_pid_is_live(pid: u32) -> Result<bool> {
    if !managed_process_pid_exists(pid)? {
        return Ok(false);
    }

    Ok(!matches!(managed_process_state_code(pid)?, Some('Z')))
}

fn managed_process_pid_exists(pid: u32) -> Result<bool> {
    // SAFETY: `kill(pid, 0)` is the standard existence probe for a process id.
    let status = unsafe { libc::kill(pid as i32, 0) };
    if status == 0 {
        return Ok(true);
    }

    let error = std::io::Error::last_os_error();
    match error.raw_os_error() {
        Some(libc::ESRCH) => Ok(false),
        Some(libc::EPERM) => Ok(true),
        _ => Err(anyhow!("failed to probe pid {pid}: {error}")),
    }
}

#[cfg(target_os = "linux")]
fn managed_process_state_code(pid: u32) -> Result<Option<char>> {
    let status_path = PathBuf::from("/proc").join(pid.to_string()).join("status");
    if !status_path.exists() {
        return Ok(None);
    }

    let status = fs::read_to_string(&status_path)
        .with_context(|| format!("failed to read process status '{}'", status_path.display()))?;
    Ok(status
        .lines()
        .find_map(|line| line.strip_prefix("State:"))
        .and_then(|state| state.trim().chars().next()))
}

#[cfg(not(target_os = "linux"))]
fn managed_process_state_code(_pid: u32) -> Result<Option<char>> {
    Ok(None)
}

trait AgentStream: Read + std::io::Write + Send + 'static {
    fn try_clone_stream(&self) -> std::io::Result<Self>
    where
        Self: Sized;

    fn shutdown_write(&self) -> std::io::Result<()>;
}

enum ForwardTargetStream {
    Tcp(TcpStream),
    Unix(UnixStream),
}

impl ForwardTargetStream {
    fn try_clone_stream(&self) -> std::io::Result<Self> {
        match self {
            Self::Tcp(stream) => stream.try_clone().map(Self::Tcp),
            Self::Unix(stream) => stream.try_clone().map(Self::Unix),
        }
    }

    fn shutdown_write(&self) -> std::io::Result<()> {
        match self {
            Self::Tcp(stream) => stream.shutdown(Shutdown::Write),
            Self::Unix(stream) => stream.shutdown(Shutdown::Write),
        }
    }
}

impl Read for ForwardTargetStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Self::Tcp(stream) => stream.read(buf),
            Self::Unix(stream) => stream.read(buf),
        }
    }
}

impl std::io::Write for ForwardTargetStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Self::Tcp(stream) => stream.write(buf),
            Self::Unix(stream) => stream.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Self::Tcp(stream) => stream.flush(),
            Self::Unix(stream) => stream.flush(),
        }
    }
}

impl AgentStream for std::os::unix::net::UnixStream {
    fn try_clone_stream(&self) -> std::io::Result<Self> {
        self.try_clone()
    }

    fn shutdown_write(&self) -> std::io::Result<()> {
        self.shutdown(Shutdown::Write)
    }
}

impl AgentStream for VsockStream {
    fn try_clone_stream(&self) -> std::io::Result<Self> {
        self.try_clone()
    }

    fn shutdown_write(&self) -> std::io::Result<()> {
        self.shutdown(Shutdown::Write)
    }
}

pub fn serve(socket_path: &Path, root: PathBuf) -> Result<()> {
    serve_with_vsock(socket_path, root, None)
}

pub fn serve_with_vsock(socket_path: &Path, root: PathBuf, vsock_port: Option<u32>) -> Result<()> {
    let service = AgentService::new(root);
    let unix_listener = bind_unix_listener(socket_path)?;
    let vsock_listener = vsock_port
        .map(|port| {
            VsockListener::bind_with_cid_port(VMADDR_CID_ANY, port).with_context(|| {
                format!("failed to bind guest-agent vsock listener on port {port}")
            })
        })
        .transpose()?;

    if let Some(vsock_listener) = vsock_listener {
        let service = service.clone();
        thread::spawn(move || {
            if let Err(error) = serve_vsock_listener(vsock_listener, &service) {
                eprintln!("port-guest-agent vsock listener exited: {error}");
            }
        });
    }

    serve_unix_listener(unix_listener, &service)
}

fn bind_unix_listener(socket_path: &Path) -> Result<UnixListener> {
    if socket_path.exists() {
        fs::remove_file(socket_path)
            .with_context(|| format!("failed to remove '{}'", socket_path.display()))?;
    }
    if let Some(parent) = socket_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create '{}'", parent.display()))?;
    }

    UnixListener::bind(socket_path)
        .with_context(|| format!("failed to bind '{}'", socket_path.display()))
}

fn serve_unix_listener(listener: UnixListener, service: &AgentService) -> Result<()> {
    for stream in listener.incoming() {
        let stream = stream.context("failed to accept guest-agent connection")?;
        let service = service.clone();
        thread::spawn(move || {
            if let Err(error) = handle_protocol_stream(stream, &service) {
                eprintln!("port-guest-agent Unix transport connection failed: {error}");
            }
        });
    }

    Ok(())
}

fn serve_vsock_listener(listener: VsockListener, service: &AgentService) -> Result<()> {
    for stream in listener.incoming() {
        let stream = stream.context("failed to accept guest-agent vsock connection")?;
        let service = service.clone();
        thread::spawn(move || {
            if let Err(error) = handle_protocol_stream(stream, &service) {
                eprintln!("port-guest-agent vsock transport connection failed: {error}");
            }
        });
    }

    Ok(())
}

fn handle_protocol_stream<S>(stream: S, service: &AgentService) -> Result<()>
where
    S: AgentStream,
{
    let reader_stream = stream
        .try_clone_stream()
        .context("failed to clone guest-agent stream")?;
    let mut reader = BufReader::new(reader_stream);
    let request: RequestEnvelope =
        read_frame(&mut reader).map_err(|error| anyhow!("protocol error: {error}"))?;
    match request.operation {
        GuestOperation::Copy(copy) => service.copy_stream(request.id, copy, &mut reader, stream),
        GuestOperation::Forward(forward) => service.forward_stream(request.id, forward, stream),
        GuestOperation::Pty(pty) => service.pty_stream(request.id, pty, stream),
        GuestOperation::Logs(logs) if logs.follow => service.logs_stream(request.id, logs, stream),
        operation => {
            let response = service.handle(RequestEnvelope {
                id: request.id,
                operation,
            });
            let mut writer = BufWriter::new(stream);
            write_frame(&mut writer, &response).map_err(|error| anyhow!("protocol error: {error}"))
        }
    }
}

impl AgentService {
    fn copy_stream<R, W>(
        &self,
        id: u64,
        request: CopyRequest,
        reader: &mut BufReader<R>,
        writer_stream: W,
    ) -> Result<()>
    where
        R: Read,
        W: std::io::Write,
    {
        let mut writer = BufWriter::new(writer_stream);
        match request.direction {
            port_agent_protocol::CopyDirection::HostToGuest => {
                let destination = match self.resolve_guest_path(&request.destination) {
                    Ok(path) => path,
                    Err(error) => return write_failed_response(&mut writer, id, error),
                };
                if let Some(parent) = destination.parent() {
                    if let Err(error) = fs::create_dir_all(parent)
                        .with_context(|| format!("failed to create '{}'", parent.display()))
                    {
                        return write_failed_response(&mut writer, id, error);
                    }
                }
                let Some(size_bytes) = request.size_bytes else {
                    return write_failed_response(
                        &mut writer,
                        id,
                        anyhow!("host-to-guest copy requires size_bytes"),
                    );
                };
                let mut destination_file = match fs::File::create(&destination)
                    .with_context(|| format!("failed to create '{}'", destination.display()))
                {
                    Ok(file) => file,
                    Err(error) => return write_failed_response(&mut writer, id, error),
                };
                write_frame(
                    &mut writer,
                    &ResponseEnvelope::Accepted {
                        id,
                        stream: StreamKind::Bytes,
                        size_bytes: None,
                    },
                )
                .map_err(|error| anyhow!("protocol error: {error}"))?;
                let mut limited = reader.take(size_bytes);
                let bytes_copied = std::io::copy(&mut limited, &mut destination_file)
                    .with_context(|| format!("failed to write '{}'", destination.display()))?;
                if bytes_copied != size_bytes {
                    bail!(
                        "expected {size_bytes} bytes for host-to-guest copy, received {bytes_copied}"
                    );
                }

                write_frame(
                    &mut writer,
                    &ResponseEnvelope::Completed {
                        id,
                        exit_code: 0,
                        result: OperationResult::Copy(port_agent_protocol::CopyResult {
                            bytes_copied,
                            path: request.destination,
                            direction: request.direction,
                        }),
                    },
                )
                .map_err(|error| anyhow!("protocol error: {error}"))?;
            }
            port_agent_protocol::CopyDirection::GuestToHost => {
                let source = match self.resolve_guest_path(&request.source) {
                    Ok(path) => path,
                    Err(error) => return write_failed_response(&mut writer, id, error),
                };
                let mut source_file = match fs::File::open(&source)
                    .with_context(|| format!("failed to open '{}'", source.display()))
                {
                    Ok(file) => file,
                    Err(error) => return write_failed_response(&mut writer, id, error),
                };
                let size_bytes = match source_file
                    .metadata()
                    .with_context(|| format!("failed to stat '{}'", source.display()))
                {
                    Ok(metadata) => metadata.len(),
                    Err(error) => return write_failed_response(&mut writer, id, error),
                };
                write_frame(
                    &mut writer,
                    &ResponseEnvelope::Accepted {
                        id,
                        stream: StreamKind::Bytes,
                        size_bytes: Some(size_bytes),
                    },
                )
                .map_err(|error| anyhow!("protocol error: {error}"))?;
                let bytes_copied = std::io::copy(&mut source_file, &mut writer)
                    .with_context(|| format!("failed to stream '{}'", source.display()))?;
                if bytes_copied != size_bytes {
                    bail!(
                        "expected to stream {size_bytes} bytes for guest-to-host copy, wrote {bytes_copied}"
                    );
                }
                write_frame(
                    &mut writer,
                    &ResponseEnvelope::Completed {
                        id,
                        exit_code: 0,
                        result: OperationResult::Copy(port_agent_protocol::CopyResult {
                            bytes_copied,
                            path: request.destination,
                            direction: request.direction,
                        }),
                    },
                )
                .map_err(|error| anyhow!("protocol error: {error}"))?;
            }
        }

        Ok(())
    }

    fn forward_stream<S>(&self, id: u64, request: ForwardRequest, stream: S) -> Result<()>
    where
        S: AgentStream,
    {
        let mut control_writer = BufWriter::new(
            stream
                .try_clone_stream()
                .context("failed to clone guest transport stream for forward ack")?,
        );
        let mut outbound = match connect_forward_target(&request.target)
            .with_context(|| format!("failed to connect to '{}'", request.target))
        {
            Ok(stream) => stream,
            Err(error) => return write_failed_response(&mut control_writer, id, error),
        };
        write_frame(
            &mut control_writer,
            &ResponseEnvelope::Accepted {
                id,
                stream: StreamKind::Bytes,
                size_bytes: None,
            },
        )
        .map_err(|error| anyhow!("protocol error: {error}"))?;
        drop(control_writer);

        let mut inbound_read = stream
            .try_clone_stream()
            .context("failed to clone guest transport stream for forward read")?;
        let mut inbound_write = stream;
        let mut outbound_read = outbound
            .try_clone_stream()
            .context("failed to clone target stream")?;

        let first = thread::spawn(move || {
            let result = std::io::copy(&mut inbound_read, &mut outbound);
            let _ = outbound.shutdown_write();
            result
        });
        let second = thread::spawn(move || {
            let result = std::io::copy(&mut outbound_read, &mut inbound_write);
            let _ = inbound_write.shutdown_write();
            result
        });

        let _ = first.join();
        let _ = second.join();
        Ok(())
    }

    fn pty_stream<S>(&self, id: u64, request: PtyRequest, stream: S) -> Result<()>
    where
        S: AgentStream,
    {
        let (program, args) = request
            .command
            .split_first()
            .ok_or_else(|| anyhow!("pty request requires a command"))?;
        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows: request.rows,
                cols: request.cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .context("failed to allocate PTY")?;
        let mut builder = CommandBuilder::new(program);
        builder.args(args);
        builder.cwd(&self.root);
        let mut child = pair
            .slave
            .spawn_command(builder)
            .context("failed to spawn PTY command")?;
        drop(pair.slave);

        let mut pty_reader = pair
            .master
            .try_clone_reader()
            .context("failed to clone PTY reader")?;
        let mut pty_writer = pair
            .master
            .take_writer()
            .context("failed to acquire PTY writer")?;

        let input_stream = stream
            .try_clone_stream()
            .context("failed to clone guest transport stream for PTY input")?;
        let _input_thread = thread::spawn(move || {
            let mut reader = BufReader::new(input_stream);
            let result: Result<()> = loop {
                match read_frame::<_, StreamRequestFrame>(&mut reader) {
                    Ok(StreamRequestFrame::Input { data }) => {
                        pty_writer
                            .write_all(data.as_bytes())
                            .and_then(|_| pty_writer.flush())
                            .context("failed to write PTY input")?;
                    }
                    Ok(StreamRequestFrame::Close) => break Ok(()),
                    Err(_) => break Ok(()),
                }
            };
            result
        });

        let mut writer = BufWriter::new(stream);
        write_frame(
            &mut writer,
            &ResponseEnvelope::Accepted {
                id,
                stream: StreamKind::Pty,
                size_bytes: None,
            },
        )
        .map_err(|error| anyhow!("protocol error: {error}"))?;

        let result = (|| -> Result<()> {
            let mut buffer = [0_u8; 4096];
            loop {
                let bytes_read = pty_reader
                    .read(&mut buffer)
                    .context("failed to read PTY output")?;
                if bytes_read == 0 {
                    break;
                }
                write_stream_frame(
                    &mut writer,
                    StreamResponseFrame::Data {
                        channel: StreamOutputChannel::Stdout,
                        data: String::from_utf8_lossy(&buffer[..bytes_read]).into_owned(),
                    },
                )?;
            }

            let status = child.wait().context("failed to wait for PTY child")?;
            let exit_code =
                i32::try_from(status.exit_code()).context("PTY child exit code overflowed i32")?;
            write_stream_frame(&mut writer, StreamResponseFrame::Exit { exit_code })?;
            Ok(())
        })();

        if let Err(error) = result {
            let _ = write_stream_frame(
                &mut writer,
                StreamResponseFrame::Error {
                    message: error.to_string(),
                },
            );
            return Err(error);
        }

        Ok(())
    }

    fn logs_stream<S>(&self, id: u64, request: LogsRequest, stream: S) -> Result<()>
    where
        S: AgentStream,
    {
        let path = self.resolve_guest_path(&request.path)?;
        let control_stream = stream
            .try_clone_stream()
            .context("failed to clone guest transport stream for log control")?;
        let (cancel_tx, cancel_rx) = mpsc::channel();
        thread::spawn(move || {
            let mut reader = BufReader::new(control_stream);
            let graceful = loop {
                match read_frame::<_, StreamRequestFrame>(&mut reader) {
                    Ok(StreamRequestFrame::Close) => break true,
                    Ok(StreamRequestFrame::Input { .. }) => {}
                    Err(_) => break false,
                }
            };
            let _ = cancel_tx.send(graceful);
        });

        let mut writer = BufWriter::new(stream);
        write_frame(
            &mut writer,
            &ResponseEnvelope::Accepted {
                id,
                stream: StreamKind::Logs,
                size_bytes: None,
            },
        )
        .map_err(|error| anyhow!("protocol error: {error}"))?;

        let initial = fs::read_to_string(&path)
            .with_context(|| format!("failed to read log '{}'", path.display()))?;
        let initial_chunk = if let Some(tail_lines) = request.tail_lines {
            tail(&initial, tail_lines as usize)
        } else {
            initial.clone()
        };
        let mut last_len = initial.len();
        if !initial_chunk.is_empty() {
            write_stream_frame(
                &mut writer,
                StreamResponseFrame::Data {
                    channel: StreamOutputChannel::Logs,
                    data: initial_chunk,
                },
            )?;
        }

        loop {
            if let Ok(graceful_close) = cancel_rx.try_recv() {
                if graceful_close {
                    write_stream_frame(&mut writer, StreamResponseFrame::Eof)?;
                }
                return Ok(());
            }

            let contents = fs::read_to_string(&path)
                .with_context(|| format!("failed to read log '{}'", path.display()))?;
            let bytes = contents.as_bytes();
            if bytes.len() < last_len {
                last_len = 0;
            }
            if bytes.len() > last_len {
                let chunk = String::from_utf8_lossy(&bytes[last_len..]).into_owned();
                last_len = bytes.len();
                if !chunk.is_empty() {
                    write_stream_frame(
                        &mut writer,
                        StreamResponseFrame::Data {
                            channel: StreamOutputChannel::Logs,
                            data: chunk,
                        },
                    )?;
                }
            }

            thread::sleep(Duration::from_millis(50));
        }
    }
}

fn write_failed_response<W>(writer: &mut BufWriter<W>, id: u64, error: anyhow::Error) -> Result<()>
where
    W: std::io::Write,
{
    write_frame(
        writer,
        &ResponseEnvelope::Failed {
            id,
            message: error.to_string(),
        },
    )
    .map_err(|frame_error| anyhow!("protocol error: {frame_error}"))
}

fn write_stream_frame<W>(writer: &mut BufWriter<W>, frame: StreamResponseFrame) -> Result<()>
where
    W: std::io::Write,
{
    write_frame(writer, &frame).map_err(|frame_error| anyhow!("protocol error: {frame_error}"))
}

fn connect_forward_target(target: &str) -> Result<ForwardTargetStream> {
    match parse_forward_endpoint(target).map_err(|error| anyhow!(error.to_string()))? {
        ForwardEndpoint::Tcp(address) => TcpStream::connect(&address)
            .map(ForwardTargetStream::Tcp)
            .with_context(|| format!("failed to connect to TCP target '{address}'")),
        ForwardEndpoint::Unix(path) => UnixStream::connect(&path)
            .map(ForwardTargetStream::Unix)
            .with_context(|| format!("failed to connect to Unix target '{}'", path.display())),
    }
}

fn tail(contents: &str, lines: usize) -> String {
    let mut collected = contents.lines().map(str::to_owned).collect::<Vec<_>>();
    if collected.len() > lines {
        collected = collected.split_off(collected.len() - lines);
    }
    let mut result = collected.join("\n");
    if contents.ends_with('\n') && !result.is_empty() {
        result.push('\n');
    }
    result
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::fs;
    use std::io::{BufReader, Read, Write};
    use std::net::{Shutdown, TcpListener};
    use std::os::unix::net::UnixStream;
    use std::path::Path;
    use std::process::{Command, Stdio};
    use std::thread;
    use std::time::Duration;
    use std::time::Instant;

    use port_agent_protocol::{
        CopyDirection, CopyRequest, ExecRequest, ForwardRequest, GuestOperation, LogsRequest,
        ManagedServiceKind, ManagedServiceOperation, ManagedServiceRequest, ManagedServiceResult,
        ManagedServiceRuntimeState, OperationResult, PtyRequest, RequestEnvelope, ResponseEnvelope,
        StreamKind, StreamRequestFrame, StreamResponseFrame, read_frame, write_frame,
    };
    use port_model::{
        ServiceHealthPolicy, ServiceHealthState, ServiceHealthcheck, ServicePolicy,
        ServiceRestartPolicy,
    };
    use tempfile::tempdir;

    use super::{
        AgentService, MANAGED_PROCESS_HEALTH_RESTART_GRACE_PERIOD,
        MANAGED_PROCESS_RECONCILE_INTERVAL, handle_protocol_stream, serve_with_vsock,
    };

    fn wait_for(condition: impl Fn() -> bool) {
        for _ in 0..100 {
            if condition() {
                return;
            }
            thread::sleep(Duration::from_millis(20));
        }
        panic!("condition did not become true in time");
    }

    fn read_to_string(path: &Path) -> String {
        fs::read_to_string(path).unwrap_or_default()
    }

    fn wait_for_background(condition: impl Fn() -> bool) {
        wait_for_background_for(MANAGED_PROCESS_RECONCILE_INTERVAL * 12, condition);
    }

    fn wait_for_background_for(timeout: Duration, condition: impl Fn() -> bool) {
        let deadline = std::time::Instant::now()
            .checked_add(timeout)
            .expect("deadline should compute");
        while std::time::Instant::now() < deadline {
            if condition() {
                return;
            }
            thread::sleep(Duration::from_millis(50));
        }
        panic!("background condition did not become true in time");
    }

    #[cfg(target_os = "linux")]
    fn wait_for_process_state(pid: u32, expected: char) {
        wait_for(|| {
            matches!(
                super::managed_process_state_code(pid),
                Ok(Some(state)) if state == expected
            )
        });
    }

    #[test]
    fn service_answers_ping_with_pong_under_response_budget() {
        let temp = tempdir().expect("tempdir should exist");
        let service = AgentService::new(temp.path().to_path_buf());

        let start = Instant::now();
        let response = service.handle(RequestEnvelope {
            id: 42,
            operation: GuestOperation::Ping,
        });
        let elapsed = start.elapsed();

        match response {
            ResponseEnvelope::Completed {
                id,
                exit_code,
                result,
            } => {
                assert_eq!(id, 42);
                assert_eq!(exit_code, 0);
                assert!(matches!(result, OperationResult::Pong));
            }
            other => panic!("unexpected ping response: {other:?}"),
        }
        assert!(
            elapsed < super::PING_RESPONSE_BUDGET,
            "ping handler latency {elapsed:?} exceeded PING_RESPONSE_BUDGET {:?}",
            super::PING_RESPONSE_BUDGET
        );
    }

    #[test]
    fn ping_leaves_running_managed_service_untouched() {
        let temp = tempdir().expect("tempdir should exist");
        let service = AgentService::new(temp.path().to_path_buf());

        // Start a long-running managed service.
        let start = service.handle(RequestEnvelope {
            id: 1,
            operation: GuestOperation::ManagedService(ManagedServiceRequest {
                operation: ManagedServiceOperation::Start {
                    name: String::from("ping-witness"),
                    kind: ManagedServiceKind::Service,
                    command: vec![
                        String::from("/bin/sh"),
                        String::from("-lc"),
                        String::from("sleep 30"),
                    ],
                    env: Default::default(),
                    cwd: None,
                    policy: ServicePolicy::default(),
                },
            }),
        });
        let pid_before = match start {
            ResponseEnvelope::Completed {
                result: OperationResult::ManagedService(ManagedServiceResult::Status(status)),
                ..
            } => status.pid.expect("managed service should have a pid"),
            other => panic!("unexpected start response: {other:?}"),
        };

        // Ping.
        let ping = service.handle(RequestEnvelope {
            id: 2,
            operation: GuestOperation::Ping,
        });
        assert!(matches!(
            ping,
            ResponseEnvelope::Completed {
                result: OperationResult::Pong,
                ..
            }
        ));

        // The managed service should still be the same process.
        let status = service.handle(RequestEnvelope {
            id: 3,
            operation: GuestOperation::ManagedService(ManagedServiceRequest {
                operation: ManagedServiceOperation::Status {
                    name: String::from("ping-witness"),
                },
            }),
        });
        let pid_after = match status {
            ResponseEnvelope::Completed {
                result: OperationResult::ManagedService(ManagedServiceResult::Status(status)),
                ..
            } => status.pid.expect("managed service should still have a pid"),
            other => panic!("unexpected status response: {other:?}"),
        };
        assert_eq!(
            pid_before, pid_after,
            "ping must not disturb running managed services"
        );

        // Clean up.
        let _ = service.handle(RequestEnvelope {
            id: 4,
            operation: GuestOperation::ManagedService(ManagedServiceRequest {
                operation: ManagedServiceOperation::Stop {
                    name: String::from("ping-witness"),
                },
            }),
        });
    }

    #[test]
    fn service_handles_exec_pty_and_logs() {
        let temp = tempdir().expect("tempdir should exist");
        let guest_root = temp.path().join("guest");
        fs::create_dir_all(guest_root.join("var/log")).expect("guest root should exist");
        fs::write(guest_root.join("var/log/app.log"), "line-1\nline-2\n").expect("log file");
        let service = AgentService::new(guest_root.clone());

        let exec = service.handle(RequestEnvelope {
            id: 1,
            operation: GuestOperation::Exec(ExecRequest {
                command: vec![
                    String::from("/bin/sh"),
                    String::from("-lc"),
                    String::from("printf exec-ok"),
                ],
                cwd: None,
                env: Default::default(),
            }),
        });
        assert!(matches!(exec, ResponseEnvelope::Completed { .. }));

        let pty = service.handle(RequestEnvelope {
            id: 2,
            operation: GuestOperation::Pty(PtyRequest {
                command: vec![
                    String::from("/bin/sh"),
                    String::from("-lc"),
                    String::from("printf pty-ok"),
                ],
                cols: 80,
                rows: 24,
            }),
        });
        let pty_text = match pty {
            ResponseEnvelope::Completed { result, .. } => format!("{result:?}"),
            other => panic!("unexpected PTY response: {other:?}"),
        };
        assert!(pty_text.contains("pty-ok"));

        let logs = service.handle(RequestEnvelope {
            id: 3,
            operation: GuestOperation::Logs(LogsRequest {
                path: String::from("/var/log/app.log"),
                follow: false,
                tail_lines: Some(1),
            }),
        });
        let logs_text = match logs {
            ResponseEnvelope::Completed { result, .. } => format!("{result:?}"),
            other => panic!("unexpected logs response: {other:?}"),
        };
        assert!(logs_text.contains("line-2"));
    }

    #[test]
    fn connection_streams_copy_and_forward() {
        let temp = tempdir().expect("tempdir should exist");
        let guest_root = temp.path().join("guest");
        fs::create_dir_all(guest_root.join("workspace")).expect("guest root should exist");
        let host_source = temp.path().join("host.txt");
        fs::write(&host_source, "copy-ok").expect("host file");
        let service = AgentService::new(guest_root.clone());

        let (mut client, server) = UnixStream::pair().expect("stream pair");
        let service_for_upload = service.clone();
        let upload_thread = thread::spawn(move || {
            handle_protocol_stream(server, &service_for_upload).expect("upload should succeed")
        });
        let upload_reader_stream = client.try_clone().expect("upload stream should clone");
        let mut upload_reader = BufReader::new(upload_reader_stream);
        write_frame(
            &mut client,
            &RequestEnvelope {
                id: 4,
                operation: GuestOperation::Copy(CopyRequest {
                    source: host_source.display().to_string(),
                    destination: String::from("/workspace/copied.txt"),
                    direction: CopyDirection::HostToGuest,
                    size_bytes: Some(7),
                }),
            },
        )
        .expect("upload request should write");
        let upload_response: ResponseEnvelope = read_frame(&mut upload_reader).expect("upload ack");
        assert!(matches!(
            upload_response,
            ResponseEnvelope::Accepted {
                stream: StreamKind::Bytes,
                ..
            }
        ));
        client
            .write_all(b"copy-ok")
            .expect("upload bytes should write");
        client.flush().expect("upload bytes should flush");
        let upload_complete: ResponseEnvelope =
            read_frame(&mut upload_reader).expect("upload completion");
        assert!(matches!(
            upload_complete,
            ResponseEnvelope::Completed { .. }
        ));
        assert_eq!(
            fs::read_to_string(guest_root.join("workspace/copied.txt")).expect("copied file"),
            "copy-ok"
        );
        upload_thread.join().expect("upload thread should finish");

        let (mut client, server) = UnixStream::pair().expect("stream pair");
        let service_for_download = service.clone();
        let download_thread = thread::spawn(move || {
            handle_protocol_stream(server, &service_for_download).expect("download should succeed")
        });
        let download_reader_stream = client.try_clone().expect("download stream should clone");
        let mut download_reader = BufReader::new(download_reader_stream);
        write_frame(
            &mut client,
            &RequestEnvelope {
                id: 5,
                operation: GuestOperation::Copy(CopyRequest {
                    source: String::from("/workspace/copied.txt"),
                    destination: temp.path().join("downloaded.txt").display().to_string(),
                    direction: CopyDirection::GuestToHost,
                    size_bytes: None,
                }),
            },
        )
        .expect("download request should write");
        let download_response: ResponseEnvelope =
            read_frame(&mut download_reader).expect("download ack");
        let ResponseEnvelope::Accepted {
            size_bytes: Some(size_bytes),
            ..
        } = download_response
        else {
            panic!("unexpected download response: {download_response:?}");
        };
        let mut bytes = Vec::new();
        download_reader
            .by_ref()
            .take(size_bytes)
            .read_to_end(&mut bytes)
            .expect("download bytes should read");
        assert_eq!(bytes, b"copy-ok");
        let download_complete: ResponseEnvelope =
            read_frame(&mut download_reader).expect("download completion");
        assert!(matches!(
            download_complete,
            ResponseEnvelope::Completed { .. }
        ));
        download_thread
            .join()
            .expect("download thread should finish");

        let target = TcpListener::bind("127.0.0.1:0").expect("target listener");
        let target_addr = target.local_addr().expect("target addr");
        thread::spawn(move || {
            let (mut stream, _) = target.accept().expect("accept target");
            let mut buf = [0_u8; 32];
            let len = stream.read(&mut buf).expect("read target");
            stream.write_all(&buf[..len]).expect("write target");
        });
        let (mut client, server) = UnixStream::pair().expect("stream pair");
        let service_for_forward = service.clone();
        let forward_thread = thread::spawn(move || {
            handle_protocol_stream(server, &service_for_forward).expect("forward should succeed")
        });
        let forward_reader_stream = client.try_clone().expect("forward stream should clone");
        let mut forward_reader = BufReader::new(forward_reader_stream);
        write_frame(
            &mut client,
            &RequestEnvelope {
                id: 6,
                operation: GuestOperation::Forward(ForwardRequest {
                    listen: String::new(),
                    target: target_addr.to_string(),
                }),
            },
        )
        .expect("forward request should write");
        let forward_response: ResponseEnvelope =
            read_frame(&mut forward_reader).expect("forward ack");
        assert!(matches!(
            forward_response,
            ResponseEnvelope::Accepted {
                stream: StreamKind::Bytes,
                ..
            }
        ));
        client.write_all(b"forward-ok").expect("write forwarded");
        client
            .shutdown(Shutdown::Write)
            .expect("shutdown forward write");
        let mut buf = [0_u8; 32];
        let len = forward_reader.read(&mut buf).expect("read forwarded");
        assert_eq!(&buf[..len], b"forward-ok");
        forward_thread.join().expect("forward thread should finish");
    }

    #[test]
    fn connection_streams_pty_and_followed_logs() {
        let temp = tempdir().expect("tempdir should exist");
        let guest_root = temp.path().join("guest");
        fs::create_dir_all(guest_root.join("var/log")).expect("guest root should exist");
        fs::write(guest_root.join("var/log/app.log"), "line-1\n").expect("log file");
        let service = AgentService::new(guest_root.clone());

        let (mut client, server) = UnixStream::pair().expect("stream pair");
        let service_for_pty = service.clone();
        let pty_thread = thread::spawn(move || {
            handle_protocol_stream(server, &service_for_pty).expect("pty should succeed")
        });
        let pty_reader_stream = client.try_clone().expect("pty stream should clone");
        let mut pty_reader = BufReader::new(pty_reader_stream);
        write_frame(
            &mut client,
            &RequestEnvelope {
                id: 7,
                operation: GuestOperation::Pty(PtyRequest {
                    command: vec![
                        String::from("/bin/sh"),
                        String::from("-lc"),
                        String::from("printf pty-stream-ok"),
                    ],
                    cols: 80,
                    rows: 24,
                }),
            },
        )
        .expect("pty request should write");
        let pty_response: ResponseEnvelope = read_frame(&mut pty_reader).expect("pty ack");
        assert!(matches!(
            pty_response,
            ResponseEnvelope::Accepted {
                stream: StreamKind::Pty,
                ..
            }
        ));
        let mut transcript = String::new();
        loop {
            match read_frame(&mut pty_reader).expect("pty stream frame should decode") {
                StreamResponseFrame::Data { data, .. } => transcript.push_str(&data),
                StreamResponseFrame::Exit { exit_code } => {
                    assert_eq!(exit_code, 0);
                    break;
                }
                other => panic!("unexpected PTY stream frame: {other:?}"),
            }
        }
        assert!(transcript.contains("pty-stream-ok"));
        pty_thread.join().expect("pty thread should finish");

        let (mut client, server) = UnixStream::pair().expect("stream pair");
        let service_for_logs = service.clone();
        let logs_thread = thread::spawn(move || {
            handle_protocol_stream(server, &service_for_logs).expect("logs follow should succeed")
        });
        let logs_reader_stream = client.try_clone().expect("logs stream should clone");
        let mut logs_reader = BufReader::new(logs_reader_stream);
        write_frame(
            &mut client,
            &RequestEnvelope {
                id: 8,
                operation: GuestOperation::Logs(LogsRequest {
                    path: String::from("/var/log/app.log"),
                    follow: true,
                    tail_lines: None,
                }),
            },
        )
        .expect("logs request should write");
        let logs_response: ResponseEnvelope = read_frame(&mut logs_reader).expect("logs ack");
        assert!(matches!(
            logs_response,
            ResponseEnvelope::Accepted {
                stream: StreamKind::Logs,
                ..
            }
        ));
        let first: StreamResponseFrame = read_frame(&mut logs_reader).expect("first log frame");
        let StreamResponseFrame::Data { data, .. } = first else {
            panic!("unexpected first log frame: {first:?}");
        };
        assert_eq!(data, "line-1\n");
        fs::write(guest_root.join("var/log/app.log"), "line-1\nline-2\n").expect("log append");
        let second: StreamResponseFrame = read_frame(&mut logs_reader).expect("second log frame");
        let StreamResponseFrame::Data { data, .. } = second else {
            panic!("unexpected second log frame: {second:?}");
        };
        assert_eq!(data, "line-2\n");
        write_frame(&mut client, &StreamRequestFrame::Close).expect("logs close should write");
        let eof: StreamResponseFrame = read_frame(&mut logs_reader).expect("logs eof");
        assert!(matches!(eof, StreamResponseFrame::Eof));
        logs_thread.join().expect("logs thread should finish");
    }

    #[test]
    fn service_manages_process_lifecycle_and_redacts_secret_output() {
        let temp = tempdir().expect("tempdir should exist");
        let guest_root = temp.path().join("guest");
        fs::create_dir_all(guest_root.join("workspace")).expect("workspace should exist");
        let service = AgentService::new(guest_root.clone());

        let start = service.handle(RequestEnvelope {
            id: 9,
            operation: GuestOperation::ManagedService(ManagedServiceRequest {
                operation: ManagedServiceOperation::Start {
                    name: String::from("buildbox"),
                    kind: ManagedServiceKind::Sandbox,
                    command: vec![
                        String::from("/bin/sh"),
                        String::from("-lc"),
                        String::from(
                            "printf '%s\\n' \"$PWD\"; printf '%s\\n' \"$API_TOKEN\" >&2; trap 'exit 0' TERM; while :; do sleep 1; done",
                        ),
                    ],
                    env: BTreeMap::from([(String::from("API_TOKEN"), String::from("s3cr3t"))]),
                    cwd: Some(String::from("/workspace")),
                    policy: ServicePolicy::default(),
                },
            }),
        });
        let start_status = match start {
            ResponseEnvelope::Completed {
                exit_code: 0,
                result: OperationResult::ManagedService(ManagedServiceResult::Status(status)),
                ..
            } => status,
            other => panic!("unexpected start response: {other:?}"),
        };
        assert_eq!(start_status.name, "buildbox");
        assert_eq!(start_status.kind, ManagedServiceKind::Sandbox);
        assert_eq!(start_status.state, ManagedServiceRuntimeState::Running);
        assert_eq!(start_status.restart_count, 0);
        assert!(start_status.pid.is_some());
        assert_eq!(start_status.last_exit_code, None);
        assert_eq!(start_status.last_exit_detail, None);
        assert_eq!(start_status.health_state, ServiceHealthState::Unknown);
        assert_eq!(start_status.health_detail, None);
        assert_eq!(
            start_status.stdout_path.as_deref(),
            Some("/run/port/services/buildbox.stdout.log")
        );
        assert_eq!(
            start_status.stderr_path.as_deref(),
            Some("/run/port/services/buildbox.stderr.log")
        );

        let stdout_log = guest_root.join("run/port/services/buildbox.stdout.log");
        let stderr_log = guest_root.join("run/port/services/buildbox.stderr.log");
        let runtime_record = guest_root.join("run/port/services/runtime/buildbox.json");
        wait_for(|| read_to_string(&stdout_log).contains("/workspace"));
        wait_for(|| read_to_string(&stderr_log).contains("[redacted]"));
        wait_for(|| runtime_record.exists());

        assert!(!read_to_string(&stderr_log).contains("s3cr3t"));
        assert!(!read_to_string(&runtime_record).contains("s3cr3t"));

        let status = service.handle(RequestEnvelope {
            id: 10,
            operation: GuestOperation::ManagedService(ManagedServiceRequest {
                operation: ManagedServiceOperation::Status {
                    name: String::from("buildbox"),
                },
            }),
        });
        let status = match status {
            ResponseEnvelope::Completed {
                exit_code: 0,
                result: OperationResult::ManagedService(ManagedServiceResult::Status(status)),
                ..
            } => status,
            other => panic!("unexpected status response: {other:?}"),
        };
        assert_eq!(status.state, ManagedServiceRuntimeState::Running);
        assert_eq!(status.health_state, ServiceHealthState::Unknown);
        assert!(status.detail.contains("running"));

        let list = service.handle(RequestEnvelope {
            id: 11,
            operation: GuestOperation::ManagedService(ManagedServiceRequest {
                operation: ManagedServiceOperation::List,
            }),
        });
        let services = match list {
            ResponseEnvelope::Completed {
                exit_code: 0,
                result: OperationResult::ManagedService(ManagedServiceResult::List { services }),
                ..
            } => services,
            other => panic!("unexpected list response: {other:?}"),
        };
        assert_eq!(services.len(), 1);
        assert_eq!(services[0].name, "buildbox");
        assert_eq!(services[0].state, ManagedServiceRuntimeState::Running);
        assert_eq!(services[0].health_state, ServiceHealthState::Unknown);

        let exec = service.handle(RequestEnvelope {
            id: 12,
            operation: GuestOperation::Exec(ExecRequest {
                command: vec![
                    String::from("/bin/sh"),
                    String::from("-lc"),
                    String::from("printf exec-ok"),
                ],
                cwd: None,
                env: BTreeMap::new(),
            }),
        });
        let exec_result = match exec {
            ResponseEnvelope::Completed {
                exit_code: 0,
                result: OperationResult::Exec(result),
                ..
            } => result,
            other => panic!("unexpected exec response: {other:?}"),
        };
        assert_eq!(exec_result.stdout, "exec-ok");

        let stop = service.handle(RequestEnvelope {
            id: 13,
            operation: GuestOperation::ManagedService(ManagedServiceRequest {
                operation: ManagedServiceOperation::Stop {
                    name: String::from("buildbox"),
                },
            }),
        });
        let stop_status = match stop {
            ResponseEnvelope::Completed {
                exit_code: 0,
                result: OperationResult::ManagedService(ManagedServiceResult::Status(status)),
                ..
            } => status,
            other => panic!("unexpected stop response: {other:?}"),
        };
        assert_eq!(stop_status.state, ManagedServiceRuntimeState::Stopped);
        assert_eq!(stop_status.exit_code, Some(0));
        assert_eq!(stop_status.pid, None);
        assert_eq!(stop_status.last_exit_code, Some(0));
        assert_eq!(
            stop_status.last_exit_detail.as_deref(),
            Some("managed process stopped")
        );

        wait_for(|| read_to_string(&runtime_record).contains("\"state\": \"stopped\""));
        assert!(!read_to_string(&runtime_record).contains("s3cr3t"));
    }

    #[test]
    fn background_supervisor_restarts_failed_service_without_status_polling() {
        let temp = tempdir().expect("tempdir should exist");
        let guest_root = temp.path().join("guest");
        fs::create_dir_all(guest_root.join("workspace")).expect("workspace should exist");
        let service = AgentService::new(guest_root.clone());

        let start = service.handle(RequestEnvelope {
            id: 14,
            operation: GuestOperation::ManagedService(ManagedServiceRequest {
                operation: ManagedServiceOperation::Start {
                    name: String::from("restartbox"),
                    kind: ManagedServiceKind::Service,
                    command: vec![
                        String::from("/bin/sh"),
                        String::from("-lc"),
                        String::from(
                            "count_file=restarts; count=$(cat \"$count_file\" 2>/dev/null || echo 0); count=$((count + 1)); printf '%s' \"$count\" > \"$count_file\"; if [ \"$count\" -eq 1 ]; then sleep 0.2; exit 23; fi; trap 'exit 0' TERM; while :; do sleep 1; done",
                        ),
                    ],
                    env: BTreeMap::new(),
                    cwd: Some(String::from("/workspace")),
                    policy: ServicePolicy {
                        restart: ServiceRestartPolicy::OnFailure,
                        healthcheck: ServiceHealthcheck::default(),
                    },
                },
            }),
        });
        assert!(matches!(
            start,
            ResponseEnvelope::Completed {
                exit_code: 0,
                result: OperationResult::ManagedService(ManagedServiceResult::Status(_)),
                ..
            }
        ));

        let runtime_record = guest_root.join("run/port/services/runtime/restartbox.json");
        wait_for_background(|| {
            let record = read_to_string(&runtime_record);
            record.contains("\"state\": \"running\"") && record.contains("\"restart_count\": 1")
        });

        let status = service.handle(RequestEnvelope {
            id: 15,
            operation: GuestOperation::ManagedService(ManagedServiceRequest {
                operation: ManagedServiceOperation::Status {
                    name: String::from("restartbox"),
                },
            }),
        });
        let status = match status {
            ResponseEnvelope::Completed {
                exit_code: 0,
                result: OperationResult::ManagedService(ManagedServiceResult::Status(status)),
                ..
            } => status,
            other => panic!("unexpected restart status response: {other:?}"),
        };
        assert_eq!(status.state, ManagedServiceRuntimeState::Running);
        assert_eq!(status.restart_count, 1);
        assert_eq!(status.last_exit_code, Some(23));
    }

    #[test]
    fn background_supervisor_allows_always_service_to_recover_during_health_grace_period() {
        let temp = tempdir().expect("tempdir should exist");
        let guest_root = temp.path().join("guest");
        fs::create_dir_all(guest_root.join("workspace")).expect("workspace should exist");
        let service = AgentService::new(guest_root.clone());

        let start = service.handle(RequestEnvelope {
            id: 16,
            operation: GuestOperation::ManagedService(ManagedServiceRequest {
                operation: ManagedServiceOperation::Start {
                    name: String::from("healthbox"),
                    kind: ManagedServiceKind::Service,
                    command: vec![
                        String::from("/bin/sh"),
                        String::from("-lc"),
                        String::from(
                            "count_file=health-restarts; count=$(cat \"$count_file\" 2>/dev/null || echo 0); count=$((count + 1)); printf '%s' \"$count\" > \"$count_file\"; printf 'pre-restart-%s\\n' \"$count\" >&2; trap 'exit 0' TERM; while :; do sleep 1; done",
                        ),
                    ],
                    env: BTreeMap::new(),
                    cwd: Some(String::from("/workspace")),
                    policy: ServicePolicy {
                        restart: ServiceRestartPolicy::Always,
                        healthcheck: ServiceHealthcheck {
                            policy: ServiceHealthPolicy::Command,
                            command: vec![
                                String::from("/bin/sh"),
                                String::from("-lc"),
                                String::from("test -f healthy"),
                            ],
                            restart_on_unhealthy: false,
                        },
                    },
                },
            }),
        });
        assert!(matches!(
            start,
            ResponseEnvelope::Completed {
                exit_code: 0,
                result: OperationResult::ManagedService(ManagedServiceResult::Status(_)),
                ..
            }
        ));

        let runtime_record = guest_root.join("run/port/services/runtime/healthbox.json");
        wait_for(|| runtime_record.exists());
        thread::sleep(MANAGED_PROCESS_HEALTH_RESTART_GRACE_PERIOD / 2);
        let record = read_to_string(&runtime_record);
        assert!(
            record.contains("\"restart_count\": 0"),
            "service restarted during grace period: {record}"
        );

        fs::write(guest_root.join("workspace/healthy"), "ok").expect("healthy marker should write");
        wait_for_background_for(
            MANAGED_PROCESS_HEALTH_RESTART_GRACE_PERIOD + (MANAGED_PROCESS_RECONCILE_INTERVAL * 4),
            || {
                let record = read_to_string(&runtime_record);
                record.contains("\"health_state\": \"healthy\"")
            },
        );

        let status = service.handle(RequestEnvelope {
            id: 17,
            operation: GuestOperation::ManagedService(ManagedServiceRequest {
                operation: ManagedServiceOperation::Status {
                    name: String::from("healthbox"),
                },
            }),
        });
        let status = match status {
            ResponseEnvelope::Completed {
                exit_code: 0,
                result: OperationResult::ManagedService(ManagedServiceResult::Status(status)),
                ..
            } => status,
            other => panic!("unexpected health status response: {other:?}"),
        };
        assert_eq!(status.state, ManagedServiceRuntimeState::Running);
        assert_eq!(status.restart_count, 0);
        assert_eq!(status.health_state, ServiceHealthState::Healthy);
    }

    #[test]
    fn background_supervisor_marks_always_service_unhealthy_without_restart_by_default() {
        let temp = tempdir().expect("tempdir should exist");
        let guest_root = temp.path().join("guest");
        fs::create_dir_all(guest_root.join("workspace")).expect("workspace should exist");
        let service = AgentService::new(guest_root.clone());

        let start = service.handle(RequestEnvelope {
            id: 18,
            operation: GuestOperation::ManagedService(ManagedServiceRequest {
                operation: ManagedServiceOperation::Start {
                    name: String::from("healthbox"),
                    kind: ManagedServiceKind::Service,
                    command: vec![
                        String::from("/bin/sh"),
                        String::from("-lc"),
                        String::from(
                            "count_file=health-restarts; count=$(cat \"$count_file\" 2>/dev/null || echo 0); count=$((count + 1)); printf '%s' \"$count\" > \"$count_file\"; trap 'exit 0' TERM; while :; do sleep 1; done",
                        ),
                    ],
                    env: BTreeMap::new(),
                    cwd: Some(String::from("/workspace")),
                    policy: ServicePolicy {
                        restart: ServiceRestartPolicy::Always,
                        healthcheck: ServiceHealthcheck {
                            policy: ServiceHealthPolicy::Command,
                            command: vec![
                                String::from("/bin/sh"),
                                String::from("-lc"),
                                String::from("test -f healthy"),
                            ],
                            restart_on_unhealthy: false,
                        },
                    },
                },
            }),
        });
        assert!(matches!(
            start,
            ResponseEnvelope::Completed {
                exit_code: 0,
                result: OperationResult::ManagedService(ManagedServiceResult::Status(_)),
                ..
            }
        ));

        let runtime_record = guest_root.join("run/port/services/runtime/healthbox.json");
        wait_for_background_for(
            MANAGED_PROCESS_HEALTH_RESTART_GRACE_PERIOD + Duration::from_secs(2),
            || {
                let record = read_to_string(&runtime_record);
                record.contains("\"health_state\": \"unhealthy\"")
                    && record.contains("\"restart_count\": 0")
            },
        );

        let status = service.handle(RequestEnvelope {
            id: 19,
            operation: GuestOperation::ManagedService(ManagedServiceRequest {
                operation: ManagedServiceOperation::Status {
                    name: String::from("healthbox"),
                },
            }),
        });
        let status = match status {
            ResponseEnvelope::Completed {
                exit_code: 0,
                result: OperationResult::ManagedService(ManagedServiceResult::Status(status)),
                ..
            } => status,
            other => panic!("unexpected health status response: {other:?}"),
        };
        assert_eq!(status.state, ManagedServiceRuntimeState::Running);
        assert_eq!(status.restart_count, 0);
        assert_eq!(status.health_state, ServiceHealthState::Unhealthy);
        assert!(
            status.health_detail.as_deref() == Some("health command exited with code 1"),
            "{:?}",
            status.health_detail
        );
        assert!(
            !guest_root
                .join("run/port/service-evidence/healthbox")
                .exists(),
            "health observation should not capture restart evidence without an unhealthy-restart policy"
        );
    }

    #[test]
    fn background_supervisor_can_opt_in_to_restart_after_sustained_health_check_failure() {
        let temp = tempdir().expect("tempdir should exist");
        let guest_root = temp.path().join("guest");
        fs::create_dir_all(guest_root.join("workspace")).expect("workspace should exist");
        let service = AgentService::new(guest_root.clone());

        let start = service.handle(RequestEnvelope {
            id: 20,
            operation: GuestOperation::ManagedService(ManagedServiceRequest {
                operation: ManagedServiceOperation::Start {
                    name: String::from("healthbox"),
                    kind: ManagedServiceKind::Service,
                    command: vec![
                        String::from("/bin/sh"),
                        String::from("-lc"),
                        String::from(
                            "count_file=health-restarts; count=$(cat \"$count_file\" 2>/dev/null || echo 0); count=$((count + 1)); printf '%s' \"$count\" > \"$count_file\"; trap 'exit 0' TERM; while :; do sleep 1; done",
                        ),
                    ],
                    env: BTreeMap::new(),
                    cwd: Some(String::from("/workspace")),
                    policy: ServicePolicy {
                        restart: ServiceRestartPolicy::Always,
                        healthcheck: ServiceHealthcheck {
                            policy: ServiceHealthPolicy::Command,
                            command: vec![
                                String::from("/bin/sh"),
                                String::from("-lc"),
                                String::from("test -f healthy"),
                            ],
                            restart_on_unhealthy: true,
                        },
                    },
                },
            }),
        });
        assert!(matches!(
            start,
            ResponseEnvelope::Completed {
                exit_code: 0,
                result: OperationResult::ManagedService(ManagedServiceResult::Status(_)),
                ..
            }
        ));

        let runtime_record = guest_root.join("run/port/services/runtime/healthbox.json");
        wait_for_background_for(
            MANAGED_PROCESS_HEALTH_RESTART_GRACE_PERIOD + Duration::from_secs(2),
            || {
                let record = read_to_string(&runtime_record);
                record.contains("health check failure") && !record.contains("\"restart_count\": 0")
            },
        );

        let status = service.handle(RequestEnvelope {
            id: 21,
            operation: GuestOperation::ManagedService(ManagedServiceRequest {
                operation: ManagedServiceOperation::Status {
                    name: String::from("healthbox"),
                },
            }),
        });
        let status = match status {
            ResponseEnvelope::Completed {
                exit_code: 0,
                result: OperationResult::ManagedService(ManagedServiceResult::Status(status)),
                ..
            } => status,
            other => panic!("unexpected health status response: {other:?}"),
        };
        assert_eq!(status.state, ManagedServiceRuntimeState::Running);
        assert!(status.restart_count >= 1);
        assert_eq!(status.health_state, ServiceHealthState::Unhealthy);
        assert!(
            status
                .detail
                .contains("/run/port/service-evidence/healthbox/")
        );

        let evidence_root = guest_root.join("run/port/service-evidence/healthbox");
        wait_for_background(|| {
            evidence_root.exists()
                && fs::read_dir(&evidence_root)
                    .ok()
                    .map(|entries| {
                        entries
                            .filter_map(|entry| entry.ok())
                            .any(|entry| entry.path().is_dir())
                    })
                    .unwrap_or(false)
        });

        let mut found_evidence = false;
        for entry in fs::read_dir(&evidence_root).expect("evidence root should exist") {
            let entry = entry.expect("evidence entry should read");
            if !entry
                .file_type()
                .expect("evidence entry type should read")
                .is_dir()
            {
                continue;
            }
            let evidence_dir = entry.path();
            let metadata_path = evidence_dir.join("metadata.txt");
            let runtime_path = evidence_dir.join("run/port/services/runtime/healthbox.json");
            let stderr_path = evidence_dir.join("run/port/services/healthbox.stderr.log");
            if metadata_path.exists()
                && runtime_path.exists()
                && stderr_path.exists()
                && read_to_string(&metadata_path).contains("health check failure")
            {
                found_evidence = true;
                break;
            }
        }
        assert!(
            found_evidence,
            "expected healthbox restart evidence with metadata, runtime record, and stderr snapshot"
        );
    }

    #[cfg(unix)]
    #[test]
    fn managed_process_exit_detail_reports_signal_name_for_crash() {
        let mut child = Command::new("bash")
            .args(["-lc", "kill -SEGV $$"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("managed process should start");
        let status = child.wait().expect("managed process should exit");
        let exit_detail = super::managed_process_exit_detail(&status);
        let restart_detail = super::managed_process_restart_detail(&status);
        assert!(exit_detail.contains("SIGSEGV"), "{exit_detail}");
        assert!(restart_detail.contains("SIGSEGV"), "{restart_detail}");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn persisted_running_record_with_live_pid_downgrades_stale_health_to_unknown() {
        let temp = tempdir().expect("tempdir should exist");
        let guest_root = temp.path().join("guest");
        fs::create_dir_all(guest_root.join("run/port/services/runtime"))
            .expect("runtime dir should exist");

        let mut child = Command::new("bash")
            .args(["-lc", "trap 'exit 0' TERM; while :; do sleep 1; done"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("managed process should start");

        super::write_managed_process_record(
            &guest_root,
            &super::ManagedProcessRecord {
                name: String::from("orphanbox"),
                kind: ManagedServiceKind::Service,
                state: ManagedServiceRuntimeState::Running,
                restart_count: 0,
                pid: Some(child.id()),
                exit_code: None,
                last_exit_code: None,
                last_exit_detail: None,
                health_state: ServiceHealthState::Healthy,
                health_detail: None,
                stdout_path: String::from("/run/port/services/orphanbox.stdout.log"),
                stderr_path: String::from("/run/port/services/orphanbox.stderr.log"),
                detail: String::from("managed process is running"),
            },
        )
        .expect("runtime record should write");

        let service = AgentService::new(guest_root.clone());
        let status = service.handle(RequestEnvelope {
            id: 20,
            operation: GuestOperation::ManagedService(ManagedServiceRequest {
                operation: ManagedServiceOperation::Status {
                    name: String::from("orphanbox"),
                },
            }),
        });
        let status = match status {
            ResponseEnvelope::Completed {
                exit_code: 0,
                result: OperationResult::ManagedService(ManagedServiceResult::Status(status)),
                ..
            } => status,
            other => panic!("unexpected orphan status response: {other:?}"),
        };
        assert_eq!(status.state, ManagedServiceRuntimeState::Running);
        assert_eq!(status.pid, Some(child.id()));
        assert_eq!(status.health_state, ServiceHealthState::Unknown);
        assert_eq!(status.health_detail, None);
        assert!(
            status.detail.contains("does not hold a supervisor handle"),
            "{}",
            status.detail
        );

        let _ = child.kill();
        let _ = child.wait();
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn persisted_running_record_with_zombie_pid_is_not_reported_healthy() {
        let temp = tempdir().expect("tempdir should exist");
        let guest_root = temp.path().join("guest");
        fs::create_dir_all(guest_root.join("run/port/services/runtime"))
            .expect("runtime dir should exist");

        let mut child = Command::new("bash")
            .args(["-lc", "exit 0"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("managed process should start");
        wait_for_process_state(child.id(), 'Z');

        super::write_managed_process_record(
            &guest_root,
            &super::ManagedProcessRecord {
                name: String::from("zombiebox"),
                kind: ManagedServiceKind::Service,
                state: ManagedServiceRuntimeState::Running,
                restart_count: 0,
                pid: Some(child.id()),
                exit_code: None,
                last_exit_code: None,
                last_exit_detail: None,
                health_state: ServiceHealthState::Healthy,
                health_detail: None,
                stdout_path: String::from("/run/port/services/zombiebox.stdout.log"),
                stderr_path: String::from("/run/port/services/zombiebox.stderr.log"),
                detail: String::from("managed process is running"),
            },
        )
        .expect("runtime record should write");

        let service = AgentService::new(guest_root.clone());
        let status = service.handle(RequestEnvelope {
            id: 21,
            operation: GuestOperation::ManagedService(ManagedServiceRequest {
                operation: ManagedServiceOperation::Status {
                    name: String::from("zombiebox"),
                },
            }),
        });
        let status = match status {
            ResponseEnvelope::Completed {
                exit_code: 0,
                result: OperationResult::ManagedService(ManagedServiceResult::Status(status)),
                ..
            } => status,
            other => panic!("unexpected zombie status response: {other:?}"),
        };
        assert_eq!(status.state, ManagedServiceRuntimeState::Failed);
        assert_eq!(status.pid, None);
        assert_eq!(status.health_state, ServiceHealthState::Unknown);
        assert_eq!(status.health_detail, None);
        assert!(
            status.detail.contains("no longer live"),
            "{}",
            status.detail
        );

        let runtime_record = guest_root.join("run/port/services/runtime/zombiebox.json");
        wait_for(|| {
            let record = read_to_string(&runtime_record);
            record.contains("\"state\": \"failed\"")
                && record.contains("no longer live")
                && !record.contains("\"health_state\": \"healthy\"")
        });

        let _ = child.wait();
    }

    #[test]
    fn daemon_serves_requests_over_unix_socket() {
        let temp = tempdir().expect("tempdir should exist");
        let guest_root = temp.path().join("guest");
        fs::create_dir_all(&guest_root).expect("guest root");
        let socket_path = temp.path().join("agent.sock");
        let socket_for_thread = socket_path.clone();
        let root = guest_root.clone();
        thread::spawn(move || {
            serve_with_vsock(&socket_for_thread, root, None).expect("server should run")
        });

        for _ in 0..50 {
            if socket_path.exists() {
                break;
            }
            thread::sleep(Duration::from_millis(20));
        }

        let mut stream = UnixStream::connect(&socket_path).expect("connect socket");
        port_agent_protocol::write_frame(
            &mut stream,
            &RequestEnvelope {
                id: 9,
                operation: GuestOperation::Exec(ExecRequest {
                    command: vec![
                        String::from("/bin/sh"),
                        String::from("-lc"),
                        String::from("printf daemon-ok"),
                    ],
                    cwd: None,
                    env: Default::default(),
                }),
            },
        )
        .expect("write request");
        let mut reader = BufReader::new(stream);
        let response: ResponseEnvelope =
            port_agent_protocol::read_frame(&mut reader).expect("read response");
        let text = format!("{response:?}");
        assert!(text.contains("daemon-ok"));
    }
}
