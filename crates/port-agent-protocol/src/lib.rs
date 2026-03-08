use std::collections::BTreeMap;
use std::io::{BufRead, Write};
use std::path::PathBuf;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StreamKind {
    Bytes,
    Pty,
    Logs,
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
        GuestOperation, OperationResult, RequestEnvelope, ResponseEnvelope, StreamKind,
        parse_forward_endpoint, read_frame, render_forward_endpoint, write_frame,
    };
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
}
