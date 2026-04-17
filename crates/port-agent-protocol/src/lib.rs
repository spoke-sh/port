use std::collections::BTreeMap;
use std::io::{BufRead, Write};
use std::path::PathBuf;

use port_model::{ServiceHealthState, ServicePolicy};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestEnvelope {
    pub id: u64,
    pub operation: GuestOperation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum GuestOperation {
    Exec(ExecRequest),
    Copy(CopyRequest),
    Pty(PtyRequest),
    Logs(LogsRequest),
    Forward(ForwardRequest),
    ManagedService(ManagedServiceRequest),
    Ping,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecRequest {
    pub command: Vec<String>,
    pub cwd: Option<String>,
    pub env: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CopyRequest {
    pub source: String,
    pub destination: String,
    pub direction: CopyDirection,
    pub size_bytes: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CopyDirection {
    HostToGuest,
    GuestToHost,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PtyRequest {
    pub command: Vec<String>,
    pub cols: u16,
    pub rows: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogsRequest {
    pub path: String,
    pub follow: bool,
    pub tail_lines: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForwardRequest {
    pub listen: String,
    pub target: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ManagedServiceKind {
    Service,
    Sandbox,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManagedServiceRequest {
    pub operation: ManagedServiceOperation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "verb", rename_all = "kebab-case")]
pub enum ManagedServiceOperation {
    Start {
        name: String,
        kind: ManagedServiceKind,
        command: Vec<String>,
        env: BTreeMap<String, String>,
        cwd: Option<String>,
        policy: ServicePolicy,
    },
    List,
    Status {
        name: String,
    },
    Stop {
        name: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ManagedServiceRuntimeState {
    Stored,
    Starting,
    Running,
    Exited,
    Stopped,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManagedServiceStatus {
    pub name: String,
    pub kind: ManagedServiceKind,
    pub state: ManagedServiceRuntimeState,
    #[serde(default)]
    pub restart_count: u32,
    pub pid: Option<u32>,
    pub exit_code: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_exit_code: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_exit_detail: Option<String>,
    #[serde(default)]
    pub health_state: ServiceHealthState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub health_detail: Option<String>,
    pub stdout_path: Option<String>,
    pub stderr_path: Option<String>,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ForwardEndpoint {
    Tcp(String),
    Unix(PathBuf),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "kebab-case")]
pub enum ResponseEnvelope {
    Accepted {
        id: u64,
        stream: StreamKind,
        size_bytes: Option<u64>,
    },
    Completed {
        id: u64,
        exit_code: i32,
        result: OperationResult,
    },
    Failed {
        id: u64,
        message: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum OperationResult {
    Exec(ExecResult),
    Copy(CopyResult),
    Pty(PtyResult),
    Logs(LogsResult),
    Forward(ForwardResult),
    ManagedService(ManagedServiceResult),
    Pong,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecResult {
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CopyResult {
    pub bytes_copied: u64,
    pub path: String,
    pub direction: CopyDirection,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PtyResult {
    pub transcript: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogsResult {
    pub contents: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForwardResult {
    pub listen: String,
    pub target: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "result", rename_all = "kebab-case")]
pub enum ManagedServiceResult {
    Status(ManagedServiceStatus),
    List { services: Vec<ManagedServiceStatus> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StreamKind {
    Bytes,
    Pty,
    Logs,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StreamSessionContract {
    pub kind: StreamKind,
    pub input: StreamInputMode,
    pub output: Vec<StreamOutputChannel>,
    pub termination: StreamTerminationMode,
}

impl StreamSessionContract {
    #[must_use]
    pub fn pty() -> Self {
        Self {
            kind: StreamKind::Pty,
            input: StreamInputMode::Pty,
            output: vec![StreamOutputChannel::Stdout, StreamOutputChannel::Stderr],
            termination: StreamTerminationMode::ExplicitExit,
        }
    }

    #[must_use]
    pub fn logs_follow() -> Self {
        Self {
            kind: StreamKind::Logs,
            input: StreamInputMode::None,
            output: vec![StreamOutputChannel::Logs],
            termination: StreamTerminationMode::ExplicitEof,
        }
    }

    #[must_use]
    pub fn byte_stream() -> Self {
        Self {
            kind: StreamKind::Bytes,
            input: StreamInputMode::Bytes,
            output: vec![StreamOutputChannel::Bytes],
            termination: StreamTerminationMode::ExplicitEof,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StreamInputMode {
    None,
    Bytes,
    Pty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StreamOutputChannel {
    Bytes,
    Stdout,
    Stderr,
    Logs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StreamTerminationMode {
    ExplicitEof,
    ExplicitExit,
    ExplicitError,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum StreamRequestFrame {
    Input { data: String },
    Close,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum StreamResponseFrame {
    Data {
        channel: StreamOutputChannel,
        data: String,
    },
    Exit {
        exit_code: i32,
    },
    Eof,
    Error {
        message: String,
    },
}

#[derive(Debug)]
pub enum ProtocolError {
    Io(std::io::Error),
    Encode(serde_json::Error),
    Decode(serde_json::Error),
    EmptyFrame,
    InvalidForwardEndpoint(String),
}

impl std::fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(source) => write!(f, "I/O error: {source}"),
            Self::Encode(source) => write!(f, "encode error: {source}"),
            Self::Decode(source) => write!(f, "decode error: {source}"),
            Self::EmptyFrame => f.write_str("received an empty protocol frame"),
            Self::InvalidForwardEndpoint(message) => {
                write!(f, "invalid forward endpoint: {message}")
            }
        }
    }
}

impl std::error::Error for ProtocolError {}

impl From<std::io::Error> for ProtocolError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

pub fn parse_forward_endpoint(input: &str) -> Result<ForwardEndpoint, ProtocolError> {
    if let Some(path) = input.strip_prefix("unix://") {
        return parse_unix_path(path);
    }
    if let Some(path) = input.strip_prefix("unix:") {
        return parse_unix_path(path);
    }

    if input.trim().is_empty() {
        return Err(ProtocolError::InvalidForwardEndpoint(String::from(
            "endpoint must not be empty",
        )));
    }

    Ok(ForwardEndpoint::Tcp(input.to_string()))
}

pub fn render_forward_endpoint(endpoint: &ForwardEndpoint) -> String {
    match endpoint {
        ForwardEndpoint::Tcp(address) => address.clone(),
        ForwardEndpoint::Unix(path) => format!("unix:{}", path.display()),
    }
}

fn parse_unix_path(path: &str) -> Result<ForwardEndpoint, ProtocolError> {
    let path = if path.starts_with('/') {
        PathBuf::from(path)
    } else {
        PathBuf::from("/").join(path)
    };
    if path.as_os_str().is_empty() {
        return Err(ProtocolError::InvalidForwardEndpoint(String::from(
            "unix endpoint path must not be empty",
        )));
    }
    Ok(ForwardEndpoint::Unix(path))
}

pub fn write_frame<W: Write, T: Serialize>(writer: &mut W, value: &T) -> Result<(), ProtocolError> {
    serde_json::to_writer(&mut *writer, value).map_err(ProtocolError::Encode)?;
    writer.write_all(b"\n")?;
    writer.flush()?;
    Ok(())
}

pub fn read_frame<R: BufRead, T: DeserializeOwned>(reader: &mut R) -> Result<T, ProtocolError> {
    let mut line = String::new();
    let bytes = reader.read_line(&mut line)?;
    if bytes == 0 || line.trim().is_empty() {
        return Err(ProtocolError::EmptyFrame);
    }

    serde_json::from_str(&line).map_err(ProtocolError::Decode)
}

#[cfg(test)]
mod tests {
    use super::{
        CopyDirection, CopyRequest, ExecRequest, ExecResult, ForwardEndpoint, ForwardResult,
        GuestOperation, ManagedServiceKind, ManagedServiceOperation, ManagedServiceRequest,
        ManagedServiceResult, ManagedServiceRuntimeState, ManagedServiceStatus, OperationResult,
        RequestEnvelope, ResponseEnvelope, StreamInputMode, StreamKind, StreamOutputChannel,
        StreamRequestFrame, StreamResponseFrame, StreamSessionContract, StreamTerminationMode,
        parse_forward_endpoint, read_frame, render_forward_endpoint, write_frame,
    };
    use port_model::{ServiceHealthState, ServicePolicy};
    use std::collections::BTreeMap;
    use std::io::Cursor;
    use std::path::PathBuf;

    #[test]
    fn request_round_trips_through_json() {
        let request = RequestEnvelope {
            id: 7,
            operation: GuestOperation::Exec(ExecRequest {
                command: vec![
                    String::from("/bin/sh"),
                    String::from("-lc"),
                    String::from("id"),
                ],
                cwd: Some(String::from("/workspace")),
                env: BTreeMap::from([(String::from("TERM"), String::from("xterm-256color"))]),
            }),
        };

        let encoded = serde_json::to_string(&request).expect("request should encode");
        let decoded: RequestEnvelope =
            serde_json::from_str(&encoded).expect("request should decode");

        assert_eq!(decoded, request);
    }

    #[test]
    fn ping_request_round_trips_through_json() {
        let request = RequestEnvelope {
            id: 99,
            operation: GuestOperation::Ping,
        };

        let encoded = serde_json::to_string(&request).expect("ping request should encode");
        assert!(encoded.contains("\"ping\""));

        let decoded: RequestEnvelope =
            serde_json::from_str(&encoded).expect("ping request should decode");
        assert_eq!(decoded, request);
    }

    #[test]
    fn pong_response_round_trips_through_json() {
        let response = ResponseEnvelope::Completed {
            id: 99,
            exit_code: 0,
            result: OperationResult::Pong,
        };

        let encoded = serde_json::to_string(&response).expect("pong response should encode");
        assert!(encoded.contains("\"pong\""));

        let decoded: ResponseEnvelope =
            serde_json::from_str(&encoded).expect("pong response should decode");
        assert_eq!(decoded, response);
    }

    #[test]
    fn copy_request_serializes_direction_and_paths() {
        let request = RequestEnvelope {
            id: 11,
            operation: GuestOperation::Copy(CopyRequest {
                source: String::from("./local.txt"),
                destination: String::from("/tmp/remote.txt"),
                direction: CopyDirection::HostToGuest,
                size_bytes: Some(7),
            }),
        };

        let encoded = serde_json::to_string(&request).expect("request should encode");

        assert!(encoded.contains("\"copy\""));
        assert!(encoded.contains("\"host-to-guest\""));
        assert!(encoded.contains("/tmp/remote.txt"));
    }

    #[test]
    fn responses_capture_operation_results() {
        let accepted = ResponseEnvelope::Accepted {
            id: 1,
            stream: StreamKind::Pty,
            size_bytes: None,
        };
        let completed = ResponseEnvelope::Completed {
            id: 1,
            exit_code: 0,
            result: OperationResult::Exec(ExecResult {
                stdout: String::from("hello"),
                stderr: String::new(),
            }),
        };

        let accepted_json = serde_json::to_string(&accepted).expect("accepted should encode");
        let completed_json = serde_json::to_string(&completed).expect("completed should encode");

        assert!(accepted_json.contains("\"pty\""));
        assert!(completed_json.contains("\"stdout\":\"hello\""));
    }

    #[test]
    fn newline_framing_round_trips_requests_and_responses() {
        let request = RequestEnvelope {
            id: 21,
            operation: GuestOperation::Exec(ExecRequest {
                command: vec![String::from("/bin/echo"), String::from("ok")],
                cwd: None,
                env: BTreeMap::new(),
            }),
        };
        let response = ResponseEnvelope::Completed {
            id: 21,
            exit_code: 0,
            result: OperationResult::Forward(ForwardResult {
                listen: String::from("127.0.0.1:3000"),
                target: String::from("127.0.0.1:4000"),
            }),
        };

        let mut request_buf = Vec::new();
        write_frame(&mut request_buf, &request).expect("request should frame");
        let decoded_request: RequestEnvelope =
            read_frame(&mut Cursor::new(request_buf)).expect("request should decode");

        let mut response_buf = Vec::new();
        write_frame(&mut response_buf, &response).expect("response should frame");
        let decoded_response: ResponseEnvelope =
            read_frame(&mut Cursor::new(response_buf)).expect("response should decode");

        assert_eq!(decoded_request, request);
        assert_eq!(decoded_response, response);
    }

    #[test]
    fn parse_forward_endpoint_supports_tcp_and_unix() {
        assert_eq!(
            parse_forward_endpoint("127.0.0.1:8080").expect("tcp endpoint should parse"),
            ForwardEndpoint::Tcp(String::from("127.0.0.1:8080"))
        );
        assert_eq!(
            parse_forward_endpoint("unix:/tmp/port.sock").expect("unix endpoint should parse"),
            ForwardEndpoint::Unix(PathBuf::from("/tmp/port.sock"))
        );
        assert_eq!(
            parse_forward_endpoint("unix://tmp/port.sock")
                .expect("unix endpoint with double slash should parse"),
            ForwardEndpoint::Unix(PathBuf::from("/tmp/port.sock"))
        );
    }

    #[test]
    fn render_forward_endpoint_preserves_scheme_for_unix() {
        assert_eq!(
            render_forward_endpoint(&ForwardEndpoint::Tcp(String::from("127.0.0.1:8080"))),
            "127.0.0.1:8080"
        );
        assert_eq!(
            render_forward_endpoint(&ForwardEndpoint::Unix(PathBuf::from("/tmp/port.sock"))),
            "unix:/tmp/port.sock"
        );
    }

    #[test]
    fn stream_session_contracts_encode_expected_lifecycle() {
        let pty = StreamSessionContract::pty();
        assert_eq!(pty.kind, StreamKind::Pty);
        assert_eq!(pty.input, StreamInputMode::Pty);
        assert_eq!(
            pty.output,
            vec![StreamOutputChannel::Stdout, StreamOutputChannel::Stderr]
        );
        assert_eq!(pty.termination, StreamTerminationMode::ExplicitExit);

        let logs = StreamSessionContract::logs_follow();
        assert_eq!(logs.kind, StreamKind::Logs);
        assert_eq!(logs.input, StreamInputMode::None);
        assert_eq!(logs.output, vec![StreamOutputChannel::Logs]);
        assert_eq!(logs.termination, StreamTerminationMode::ExplicitEof);

        let bytes = StreamSessionContract::byte_stream();
        assert_eq!(bytes.kind, StreamKind::Bytes);
        assert_eq!(bytes.input, StreamInputMode::Bytes);
        assert_eq!(bytes.output, vec![StreamOutputChannel::Bytes]);
        assert_eq!(bytes.termination, StreamTerminationMode::ExplicitEof);
    }

    #[test]
    fn stream_session_contract_round_trips_through_json() {
        let contract = StreamSessionContract::pty();
        let encoded = serde_json::to_string(&contract).expect("contract should encode");
        let decoded: StreamSessionContract =
            serde_json::from_str(&encoded).expect("contract should decode");
        assert_eq!(decoded, contract);
    }

    #[test]
    fn stream_frames_round_trip_through_json() {
        let request = StreamRequestFrame::Input {
            data: String::from("ls\n"),
        };
        let request_encoded = serde_json::to_string(&request).expect("request should encode");
        let decoded_request: StreamRequestFrame =
            serde_json::from_str(&request_encoded).expect("request should decode");
        assert_eq!(decoded_request, request);

        let response = StreamResponseFrame::Data {
            channel: StreamOutputChannel::Stdout,
            data: String::from("pty-ok"),
        };
        let response_encoded = serde_json::to_string(&response).expect("response should encode");
        let decoded_response: StreamResponseFrame =
            serde_json::from_str(&response_encoded).expect("response should decode");
        assert_eq!(decoded_response, response);
    }

    #[test]
    fn managed_service_operations_round_trip_through_json() {
        let request = RequestEnvelope {
            id: 34,
            operation: GuestOperation::ManagedService(ManagedServiceRequest {
                operation: ManagedServiceOperation::Start {
                    name: String::from("buildbox"),
                    kind: ManagedServiceKind::Sandbox,
                    command: vec![
                        String::from("/bin/sh"),
                        String::from("-lc"),
                        String::from("make test"),
                    ],
                    env: BTreeMap::from([(String::from("API_TOKEN"), String::from("demo"))]),
                    cwd: Some(String::from("/workspace")),
                    policy: ServicePolicy::default(),
                },
            }),
        };
        let result = ResponseEnvelope::Completed {
            id: 34,
            exit_code: 0,
            result: OperationResult::ManagedService(ManagedServiceResult::Status(
                ManagedServiceStatus {
                    name: String::from("buildbox"),
                    kind: ManagedServiceKind::Sandbox,
                    state: ManagedServiceRuntimeState::Running,
                    restart_count: 1,
                    pid: Some(4242),
                    exit_code: None,
                    last_exit_code: Some(23),
                    last_exit_detail: Some(String::from("managed process exited with code 23")),
                    health_state: ServiceHealthState::Healthy,
                    health_detail: None,
                    stdout_path: Some(String::from("/run/port/services/buildbox.stdout.log")),
                    stderr_path: Some(String::from("/run/port/services/buildbox.stderr.log")),
                    detail: String::from("managed sandbox is running"),
                },
            )),
        };

        let encoded_request = serde_json::to_string(&request).expect("request should encode");
        let decoded_request: RequestEnvelope =
            serde_json::from_str(&encoded_request).expect("request should decode");
        assert_eq!(decoded_request, request);

        let encoded_result = serde_json::to_string(&result).expect("result should encode");
        let decoded_result: ResponseEnvelope =
            serde_json::from_str(&encoded_result).expect("result should decode");
        assert_eq!(decoded_result, result);
    }
}
