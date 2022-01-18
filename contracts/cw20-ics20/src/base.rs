//! Base functionality.

use crate::{proto, Decimal, Error, ErrorReport, Result};
use eyre::WrapErr;
use serde::{de, de::Error as _, ser, Deserialize, Serialize};
use std::{fmt, str::FromStr};
use subtle_encoding::bech32;
use std::convert::TryFrom;
use std::convert::TryInto;

/// Account identifiers
#[derive(Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct AccountId {
    /// Account ID encoded as Bech32
    bech32: String,

    /// Length of the human-readable prefix of the address
    hrp_length: usize,
}

impl AccountId {
    /// Create an [`AccountId`] with the given human-readable prefix and
    /// public key hash.
    pub fn new(prefix: &str, bytes: [u8; tendermint::account::LENGTH]) -> Result<Self> {
        let id = bech32::encode(prefix, &bytes);

        // TODO(tarcieri): ensure this is the proper validation for an account prefix
        if prefix.chars().all(|c| matches!(c, 'a'..='z')) {
            Ok(Self {
                bech32: id,
                hrp_length: prefix.len(),
            })
        } else {
            Err(Error::AccountId { id })
                .wrap_err("expected prefix to be lowercase alphabetical characters only")
        }
    }

    /// Get the human-readable prefix of this account.
    pub fn prefix(&self) -> &str {
        &self.bech32[..self.hrp_length]
    }

    /// Decode an account ID from Bech32 to an inner byte value.
    pub fn to_bytes(&self) -> [u8; tendermint::account::LENGTH] {
        bech32::decode(&self.bech32)
            .ok()
            .and_then(|result| result.1.try_into().ok())
            .expect("malformed Bech32 AccountId")
    }
}

impl AsRef<str> for AccountId {
    fn as_ref(&self) -> &str {
        &self.bech32
    }
}

impl fmt::Debug for AccountId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("AccountId").field(&self.as_ref()).finish()
    }
}

impl fmt::Display for AccountId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_ref())
    }
}

impl FromStr for AccountId {
    type Err = ErrorReport;

    fn from_str(s: &str) -> Result<Self> {
        let (hrp, bytes) = bech32::decode(s).wrap_err("failed to decode bech32")?;

        if bytes.len() == tendermint::account::LENGTH {
            Ok(Self {
                bech32: s.to_owned(),
                hrp_length: hrp.len(),
            })
        } else {
            Err(Error::AccountId { id: s.to_owned() }).wrap_err_with(|| {
                format!(
                    "account ID should be at least {} bytes long, but was {} bytes long",
                    tendermint::account::LENGTH,
                    bytes.len()
                )
            })
        }
    }
}

impl From<AccountId> for tendermint::account::Id {
    fn from(id: AccountId) -> tendermint::account::Id {
        tendermint::account::Id::from(&id)
    }
}

impl From<&AccountId> for tendermint::account::Id {
    fn from(id: &AccountId) -> tendermint::account::Id {
        tendermint::account::Id::new(id.to_bytes())
    }
}

impl<'de> Deserialize<'de> for AccountId {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer)?
            .parse()
            .map_err(D::Error::custom)
    }
}

impl Serialize for AccountId {
    fn serialize<S: ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.bech32.serialize(serializer)
    }
}

/// Coin defines a token with a denomination and an amount.
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Coin {
    /// Denomination
    pub denom: String,

    /// Amount
    pub amount: String,
}

impl TryFrom<proto::cosmos::base::v1beta1::Coin> for Coin {
    type Error = ErrorReport;

    fn try_from(proto: proto::cosmos::base::v1beta1::Coin) -> Result<Coin> {
        Coin::try_from(&proto)
    }
}

impl TryFrom<&proto::cosmos::base::v1beta1::Coin> for Coin {
    type Error = ErrorReport;

    fn try_from(proto: &proto::cosmos::base::v1beta1::Coin) -> Result<Coin> {
        Ok(Coin {
            denom: proto.denom.parse()?,
            amount: proto.amount.parse()?,
        })
    }
}

impl From<Coin> for proto::cosmos::base::v1beta1::Coin {
    fn from(coin: Coin) -> proto::cosmos::base::v1beta1::Coin {
        proto::cosmos::base::v1beta1::Coin::from(&coin)
    }
}

impl From<&Coin> for proto::cosmos::base::v1beta1::Coin {
    fn from(coin: &Coin) -> proto::cosmos::base::v1beta1::Coin {
        proto::cosmos::base::v1beta1::Coin {
            denom: coin.denom.to_string(),
            amount: coin.amount.to_string(),
        }
    }
}

/// Denomination.
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Denom(String);

impl AsRef<str> for Denom {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl fmt::Display for Denom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_ref())
    }
}

impl FromStr for Denom {
    type Err = ErrorReport;

    fn from_str(s: &str) -> Result<Self> {
        // TODO(tarcieri): ensure this is the proper validation for a denom name
        if s.chars().all(|c| matches!(c, 'a'..='z')) {
            Ok(Denom(s.to_owned()))
        } else {
            Err(Error::Denom { name: s.to_owned() }.into())
        }
    }
}

/// Coin defines a token with a denomination and an amount.
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct SwapAmountInRoute {
    /// Denomination
    pub pool_id: u64,

    /// Amount
    pub token_out_denom: String,
}

impl TryFrom<proto::osmosis::gamm::v1beta1::SwapAmountInRoute> for SwapAmountInRoute {
    type Error = ErrorReport;

    fn try_from(proto: proto::osmosis::gamm::v1beta1::SwapAmountInRoute) -> Result<SwapAmountInRoute> {
        SwapAmountInRoute::try_from(&proto)
    }
}

impl TryFrom<&proto::osmosis::gamm::v1beta1::SwapAmountInRoute> for SwapAmountInRoute {
    type Error = ErrorReport;

    fn try_from(proto: &proto::osmosis::gamm::v1beta1::SwapAmountInRoute) -> Result<SwapAmountInRoute> {
        Ok(SwapAmountInRoute {
            pool_id: proto.pool_id,
            token_out_denom: proto.token_out_denom.parse()?,
        })
    }
}

impl From<SwapAmountInRoute> for proto::osmosis::gamm::v1beta1::SwapAmountInRoute {
    fn from(swap_amount_in_route: SwapAmountInRoute) -> proto::osmosis::gamm::v1beta1::SwapAmountInRoute {
        proto::osmosis::gamm::v1beta1::SwapAmountInRoute::from(&swap_amount_in_route)
    }
}

impl From<&SwapAmountInRoute> for proto::osmosis::gamm::v1beta1::SwapAmountInRoute {
    fn from(swap_amount_in_route: &SwapAmountInRoute) -> proto::osmosis::gamm::v1beta1::SwapAmountInRoute {
        proto::osmosis::gamm::v1beta1::SwapAmountInRoute {
            pool_id: swap_amount_in_route.pool_id,
            token_out_denom: swap_amount_in_route.token_out_denom.to_string(),
        }
    }
}