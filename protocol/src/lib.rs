use serde::{Deserialize, Serialize};

pub const SERVER_PORT: &str = "9000";

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub struct SessionID(String);

impl SessionID {
    pub fn new(inner: String) -> Self {
        SessionID(inner)
    }
    pub fn inner(self) -> String {
        self.0
    }
}

impl From<&str> for SessionID {
    fn from(session_id: &str) -> Self {
        SessionID(session_id.to_string())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub struct UserId(String);

impl UserId {
    pub fn new(inner: String) -> Self {
        UserId(inner)
    }
    pub fn inner(self) -> String {
        self.0
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub struct TankId(String);

impl TankId {
    pub fn new(inner: String) -> Self {
        Self(inner)
    }
    pub fn inner(self) -> String {
        self.0
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub enum ProtoId {
    Tank(TankId),
    User(UserId),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SignalEnum {
    OperatorCommand(OperatorCommand),
    UserResponse(UserResponse),
    TankCommand(TankCommand),
    TankResponse(TankResponse),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum OperatorCommand {
    Login,
    ConnectTo(TankId, String),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum UserResponse {
    LoginResponse(UserId),
    CameraListGetSuccess(Vec<TankId>),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TankCommand {
    Login,
    NewCamera(TankId),
}
#[derive(Debug, Serialize, Deserialize)]
pub enum TankResponse {
    LoginResponse(TankId),
    SessionAsk(String),
}

impl SignalEnum {
    pub fn is_login(&self) -> bool {
        match self {
            SignalEnum::OperatorCommand(cmd) => matches!(cmd, OperatorCommand::Login),
            SignalEnum::TankCommand(cmd) => matches!(cmd, TankCommand::Login),
            _ => false,
        }
    }
    pub fn is_operator(&self) -> bool {
        matches!(self, SignalEnum::OperatorCommand(_))
    }
    pub fn is_tank(&self) -> bool {
        matches!(self, SignalEnum::TankCommand(_))
    }
}
