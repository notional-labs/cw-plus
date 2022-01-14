use std::num::TryFromIntError;
use std::string::FromUtf8Error;
use thiserror::Error;

use cosmwasm_std::StdError;
use cw0::PaymentError;

pub use eyre::{Report, Result};

/// Never is a placeholder to ensure we don't return any errors
#[derive(Error, Debug)]
pub enum Never {}

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Payment(#[from] PaymentError),

    #[error("Channel doesn't exist: {id}")]
    NoSuchChannel { id: String },

    #[error("Didn't send any funds")]
    NoFunds {},

    #[error("Amount larger than 2**64, not supported by ics20 packets")]
    AmountOverflow {},

    #[error("Only supports channel with ibc version ics20-1, got {version}")]
    InvalidIbcVersion { version: String },

    #[error("Only supports unordered channel")]
    OnlyOrderedChannel {},

    #[error("Insufficient funds to redeem voucher on channel")]
    InsufficientFunds {},

    #[error("Only accepts tokens that originate on this chain, not native tokens of remote chain")]
    NoForeignTokens {},

    #[error("Parsed port from denom ({port}) doesn't match packet")]
    FromOtherPort { port: String },

    #[error("Parsed channel from denom ({channel}) doesn't match packet")]
    FromOtherChannel { channel: String },

    #[error("Cannot migrate from different contract type: {previous_contract}")]
    CannotMigrate { previous_contract: String },

    #[error("Got a submessage reply with unknown id: {id}")]
    UnknownReplyId { id: u64 },
}

impl From<FromUtf8Error> for ContractError {
    fn from(_: FromUtf8Error) -> Self {
        ContractError::Std(StdError::invalid_utf8("parsing denom key"))
    }
}

impl From<TryFromIntError> for ContractError {
    fn from(_: TryFromIntError) -> Self {
        ContractError::AmountOverflow {}
    }
}

/// Kinds of errors.
#[derive(Clone, Debug, Error, PartialEq)]
pub enum Error {
    /// Invalid account.
    #[error("invalid account ID: {id:?}")]
    AccountId {
        /// Malformed account ID
        id: String,
    },

    /// Cryptographic errors.
    #[error("cryptographic error")]
    Crypto,

    /// Invalid decimal value.
    #[error("invalid decimal value: {value:?}")]
    Decimal {
        /// Invalid decimal value
        value: String,
    },

    /// Invalid denomination.
    #[error("invalid denomination: {name:?}")]
    Denom {
        /// Invalid name
        name: String,
    },

    /// Protobuf is missing a field.
    #[error("missing proto field: {name:?}")]
    MissingField {
        /// Name of the missing field
        name: &'static str,
    },

    /// Unexpected message type.
    #[error("unexpected Msg type: {found:?}, expected {expected:?}")]
    MsgType {
        /// Expected type URL.
        expected: &'static str,

        /// Actual type URL found in the [`prost_types::Any`] message.
        found: String,
    },

    #[error("invalid proto enum value: {name:?}, value: {found_value:?}")]
    InvalidEnumValue {
        /// Name of the enum field
        name: &'static str,

        /// Actual value of the field found
        found_value: i32,
    },
    
}


impl Eq for Error {}
