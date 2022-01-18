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

    pub token_in: Coin,

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
            token_in: TryFrom::try_from(proto.token_in.clone().unwrap())?,
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
            token_in: From::from(proto::cosmos::base::v1beta1::Coin::from(&msg.token_in)),
            token_out_min_amount: msg.token_out_min_amount.to_string(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct MsgJoinPool {
    /// Sender's address.
    pub sender: String,

    pub pool_id: u64,

    pub share_out_amount: String,

    pub token_in_maxs: Vec<Coin>,
}

impl Msg for MsgJoinPool {
    type Proto = proto::osmosis::gamm::v1beta1::MsgJoinPool;
}

impl TryFrom<proto::osmosis::gamm::v1beta1::MsgJoinPool> for MsgJoinPool {
    type Error = ErrorReport;

    fn try_from(proto: proto::osmosis::gamm::v1beta1::MsgJoinPool) -> Result<MsgJoinPool> {
        MsgJoinPool::try_from(&proto)
    }
}

impl TryFrom<&proto::osmosis::gamm::v1beta1::MsgJoinPool> for MsgJoinPool {
    type Error = ErrorReport;

    fn try_from(proto: &proto::osmosis::gamm::v1beta1::MsgJoinPool) -> Result<MsgJoinPool> {
        Ok(MsgJoinPool {
            sender: proto.sender.parse()?,
            pool_id: proto.pool_id,
            share_out_amount: proto.share_out_amount.parse()?,
            token_in_maxs: proto
                .token_in_maxs
                .iter()
                .map(TryFrom::try_from)
                .collect::<Result<_, _>>()?,
        })
    }
}

impl From<MsgJoinPool> for proto::osmosis::gamm::v1beta1::MsgJoinPool {
    fn from(msg: MsgJoinPool) -> proto::osmosis::gamm::v1beta1::MsgJoinPool {
        proto::osmosis::gamm::v1beta1::MsgJoinPool::from(&msg)
    }
}

impl From<&MsgJoinPool> for proto::osmosis::gamm::v1beta1::MsgJoinPool {
    fn from(msg: &MsgJoinPool) -> proto::osmosis::gamm::v1beta1::MsgJoinPool {
        proto::osmosis::gamm::v1beta1::MsgJoinPool {
            sender: msg.sender.to_string(),
            pool_id: msg.pool_id,
            share_out_amount: msg.share_out_amount.to_string(),
            token_in_maxs: msg
                .token_in_maxs
                .iter()
                .map(Into::into)
                .collect()
        }
    }
}