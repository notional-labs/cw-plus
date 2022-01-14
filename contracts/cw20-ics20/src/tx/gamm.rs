//! Bank module support
//!
//! <https://docs.cosmos.network/master/modules/bank/>

use crate::{proto, tx::Msg, AccountId, Coin, SwapAmountInRoute, ErrorReport, Result};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct MsgSwapExactAmountIn {
    /// Sender's address.
    pub sender: String,

    pub routes: Vec<SwapAmountInRoute>,

    pub token_in: Vec<Coin>,

    pub token_out_min_amount: String,
}

impl Msg for MsgSwapExactAmountIn {
    type Proto = proto::osmosis::gamm::v1beta1::MsgSwapExactAmountIn;
}

impl TryFrom<proto::osmosis::gamm::v1beta1::MsgSwapExactAmountIn> for MsgSwapExactAmountIn {
    type Error = ErrorReport;

    fn try_from(proto: proto::osmosis::gamm::v1beta1::MsgSwapExactAmountIn) -> Result<MsgSwapExactAmountIn> {
        MsgSwapExactAmountIn::try_from(&proto)
    }
}

impl TryFrom<&proto::osmosis::gamm::v1beta1::MsgSwapExactAmountIn> for MsgSwapExactAmountIn {
    type Error = ErrorReport;

    fn try_from(proto: &proto::osmosis::gamm::v1beta1::MsgSwapExactAmountIn) -> Result<MsgSwapExactAmountIn> {
        Ok(MsgSwapExactAmountIn {
            sender: proto.sender.parse()?,
            routes: proto
                .routes
                .iter()
                .map(TryFrom::try_from)
                .collect::<Result<_, _>>()?,
            token_in: proto
                .token_in
                .iter()
                .map(TryFrom::try_from)
                .collect::<Result<_, _>>()?,
            token_out_min_amount: proto.token_out_min_amount,
        })
    }
}

impl From<MsgSwapExactAmountIn> for proto::osmosis::gamm::v1beta1::MsgSwapExactAmountIn {
    fn from(coin: MsgSwapExactAmountIn) -> proto::osmosis::gamm::v1beta1::MsgSwapExactAmountIn {
        proto::osmosis::gamm::v1beta1::MsgSwapExactAmountIn::from(&coin)
    }
}

impl From<&MsgSwapExactAmountIn> for proto::osmosis::gamm::v1beta1::MsgSwapExactAmountIn {
    fn from(msg: &MsgSwapExactAmountIn) -> proto::osmosis::gamm::v1beta1::MsgSwapExactAmountIn {
        proto::osmosis::gamm::v1beta1::MsgSwapExactAmountIn {
            sender: msg.sender,
            routes: msg.routes.iter().map(Into::into).collect(),
            token_in: msg.token_in.iter().map(Into::into).collect(),
            token_out_min_amount: msg.token_out_min_amount,
        }
    }
}

/// Coin defines a token with a denomination and an amount.
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct SwapAmountInRoute {
    /// Denomination
    pub pool_id: u64,

    /// Amount
    pub token_out_denom: Denom,
}