//! Bank module support
//!
//! <https://docs.cosmos.network/master/modules/bank/>

use crate::{proto, tx::Msg, Coin, SwapAmountInRoute, ErrorReport, Result};
use core::convert::TryFrom;

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
            token_out_min_amount: proto.token_out_min_amount.parse()?,
        })
    }
}

impl From<MsgSwapExactAmountIn> for proto::osmosis::gamm::v1beta1::MsgSwapExactAmountIn {
    fn from(msg: MsgSwapExactAmountIn) -> proto::osmosis::gamm::v1beta1::MsgSwapExactAmountIn {
        proto::osmosis::gamm::v1beta1::MsgSwapExactAmountIn::from(&msg)
    }
}

impl From<&MsgSwapExactAmountIn> for proto::osmosis::gamm::v1beta1::MsgSwapExactAmountIn {
    fn from(msg: &MsgSwapExactAmountIn) -> proto::osmosis::gamm::v1beta1::MsgSwapExactAmountIn {
        proto::osmosis::gamm::v1beta1::MsgSwapExactAmountIn {
            sender: msg.sender.to_string(),
            routes: msg.routes.iter().map(Into::into).collect(),
            token_in: msg.token_in.iter().map(Into::into).collect(),
            token_out_min_amount: msg.token_out_min_amount.to_string(),
        }
    }
}