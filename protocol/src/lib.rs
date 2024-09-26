use serde::{Deserialize, Serialize};

pub const SERVER_PORT: &str = "9000";

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

// event server -> client
// command client -> server and response -> client
#[derive(Debug, Serialize, Deserialize)]
pub enum SignalEnum {
    Start,
    UserCommand(UserCommand),
    UserResponse(UserMessage),
    TankCommand(TankCommand),
    TankMessage(TankMessage),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum UserCommand {
    Login,
    IceOffer(TankId, String),
    SdpOffer(TankId, String),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum UserMessage {
    LoginResponse(UserId),
    CameraListGetSuccess(Vec<TankId>),
    SdpAnswer(TankId, String),
    IceOfferAnswer(TankId, String),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TankCommand {
    Login,
    NewCamera(TankId),
    SdpAnswer(UserId, String),
    IceAnswer(UserId, String),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TankMessage {
    LoginResponse(TankId),
    SdpConnectionOffer(UserId, String),
    IceConnectionOffer(UserId, String),
}

impl SignalEnum {
    pub fn is_login(&self) -> bool {
        match self {
            SignalEnum::UserCommand(cmd) => matches!(cmd, UserCommand::Login),
            SignalEnum::TankCommand(cmd) => matches!(cmd, TankCommand::Login),
            _ => false,
        }
    }
    pub fn is_operator(&self) -> bool {
        matches!(self, SignalEnum::UserCommand(_))
    }
    pub fn is_tank(&self) -> bool {
        matches!(self, SignalEnum::TankCommand(_))
    }
}
