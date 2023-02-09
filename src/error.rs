use std::net;
use thiserror::Error;

pub type Result<T = ()> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("config error")]
    Config(#[from] config::ConfigError),
    #[error("custom error")]
    Custom(String),
    #[error("io error")]
    IO(#[from] std::io::Error),
    #[error("crypto error")]
    CryptoError(#[from] helium_crypto::Error),
    #[error("encode error")]
    Encode(#[from] EncodeError),
    #[error("decode error")]
    Decode(#[from] DecodeError),
    #[error("service error: {0}")]
    Service(#[from] ServiceError),
    #[error("semtech udp error")]
    Semtech(#[from] Box<semtech_udp::server_runtime::Error>),
    #[error("beacon error")]
    Beacon(#[from] beacon::Error),
    #[error("gateway error: {0}")]
    Gateway(#[from] crate::gateway::GatewayError),
    #[error("region error")]
    Region(#[from] RegionError),
    #[error("system time")]
    SystemTime(#[from] std::time::SystemTimeError),
}

#[derive(Error, Debug)]
pub enum EncodeError {
    #[error("protobuf encode")]
    Prost(#[from] prost::EncodeError),
}

#[derive(Error, Debug)]
pub enum DecodeError {
    #[error("uri decode")]
    Uri(#[from] http::uri::InvalidUri),
    #[error("keypair uri: {0}")]
    KeypairUri(String),
    #[error("json decode")]
    Json(#[from] serde_json::Error),
    #[error("base64 decode")]
    Base64(#[from] base64::DecodeError),
    #[error("network address decode")]
    Addr(#[from] net::AddrParseError),
    #[error("protobuf decode")]
    Prost(#[from] prost::DecodeError),
    #[error("lorawan decode")]
    LoraWan(#[from] lorawan::LoraWanError),
    #[error("semtech decode")]
    Semtech(#[from] semtech_udp::data_rate::ParseError),
    #[error("packet crc")]
    InvalidCrc,
    #[error("unexpected transaction in envelope")]
    InvalidEnvelope,
    #[error("no rx1 window in downlink packet")]
    NoRx1Window,
    #[error("no datarate found in packet")]
    NoDataRate,
    #[error("packet is not a beacon")]
    NotBeacon,
    #[error("invalid beacon datarate: {0}")]
    InvalidBeaconDataRate(String),
}

#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("service {0:?}")]
    Service(#[from] helium_proto::services::Error),
    #[error("rpc {0:?}")]
    Rpc(#[from] tonic::Status),
    #[error("stream closed")]
    Stream,
    #[error("channel closed")]
    Channel,
    #[error("no service")]
    NoService,
    #[error("Unable to connect to local server. Check that `helium_gateway` is running.")]
    LocalClientConnect(helium_proto::services::Error),
}

#[derive(Debug, Error)]
pub enum RegionError {
    #[error("no region params found or active")]
    NoRegionParams,
    #[error("no region tx power defined in region params")]
    NoRegionTxPower,
}

macro_rules! from_err {
    ($to_type:ty, $from_type:ty) => {
        impl From<$from_type> for Error {
            fn from(v: $from_type) -> Self {
                Self::from(<$to_type>::from(v))
            }
        }
    };
}

// Service Errors
from_err!(ServiceError, helium_proto::services::Error);
from_err!(ServiceError, tonic::Status);

impl<T> From<tokio::sync::mpsc::error::SendError<T>> for Error {
    fn from(_err: tokio::sync::mpsc::error::SendError<T>) -> Self {
        Self::Service(ServiceError::Stream)
    }
}

// Encode Errors
from_err!(EncodeError, prost::EncodeError);

// Decode Errors
from_err!(DecodeError, http::uri::InvalidUri);
from_err!(DecodeError, base64::DecodeError);
from_err!(DecodeError, serde_json::Error);
from_err!(DecodeError, net::AddrParseError);
from_err!(DecodeError, prost::DecodeError);
from_err!(DecodeError, lorawan::LoraWanError);
from_err!(DecodeError, semtech_udp::data_rate::ParseError);

impl DecodeError {
    pub fn invalid_envelope() -> Error {
        Error::Decode(DecodeError::InvalidEnvelope)
    }

    pub fn invalid_crc() -> Error {
        Error::Decode(DecodeError::InvalidCrc)
    }

    pub fn prost_decode(msg: &'static str) -> Error {
        Error::Decode(prost::DecodeError::new(msg).into())
    }

    pub fn keypair_uri<T: ToString>(msg: T) -> Error {
        Error::Decode(DecodeError::KeypairUri(msg.to_string()))
    }

    pub fn no_rx1_window() -> Error {
        Error::Decode(DecodeError::NoRx1Window)
    }

    pub fn no_data_rate() -> Error {
        Error::Decode(DecodeError::NoDataRate)
    }

    pub fn invalid_beacon_data_rate(datarate: String) -> Error {
        Error::Decode(DecodeError::InvalidBeaconDataRate(datarate))
    }

    pub fn not_beacon() -> Error {
        Error::Decode(DecodeError::NotBeacon)
    }
}

impl RegionError {
    pub fn no_region_params() -> Error {
        Error::Region(RegionError::NoRegionParams)
    }

    pub fn no_region_tx_power() -> Error {
        Error::Region(RegionError::NoRegionTxPower)
    }
}

impl Error {
    /// Use as for custom or rare errors that don't quite deserve their own
    /// error
    pub fn custom<T: ToString>(msg: T) -> Error {
        Error::Custom(msg.to_string())
    }

    pub fn channel() -> Error {
        Error::Service(ServiceError::Channel)
    }

    pub fn no_service() -> Error {
        Error::Service(ServiceError::NoService)
    }

    pub fn local_client_connect(e: helium_proto::services::Error) -> Error {
        Error::Service(ServiceError::LocalClientConnect(e))
    }
}
