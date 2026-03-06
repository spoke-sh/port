use std::collections::BTreeMap;

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "kebab-case")]
pub enum ResponseEnvelope {
    Accepted {
        id: u64,
        stream: StreamKind,
    },
    Completed {
        id: u64,
        exit_code: i32,
        message: String,
    },
    Failed {
        id: u64,
        message: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StreamKind {
    Bytes,
    Pty,
    Logs,
}

#[cfg(test)]
mod tests {
    use super::{
        CopyDirection, CopyRequest, ExecRequest, GuestOperation, RequestEnvelope, ResponseEnvelope,
        StreamKind,
    };
    use std::collections::BTreeMap;

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
            }),
        };

        let encoded = serde_json::to_string(&request).expect("request should encode");

        assert!(encoded.contains("\"copy\""));
        assert!(encoded.contains("\"host-to-guest\""));
        assert!(encoded.contains("/tmp/remote.txt"));
    }

    #[test]
    fn responses_capture_stream_and_completion() {
        let accepted = ResponseEnvelope::Accepted {
            id: 1,
            stream: StreamKind::Pty,
        };
        let completed = ResponseEnvelope::Completed {
            id: 1,
            exit_code: 0,
            message: String::from("shell exited"),
        };

        let accepted_json = serde_json::to_string(&accepted).expect("accepted should encode");
        let completed_json = serde_json::to_string(&completed).expect("completed should encode");

        assert!(accepted_json.contains("\"pty\""));
        assert!(completed_json.contains("\"exit_code\":0"));
    }
}
