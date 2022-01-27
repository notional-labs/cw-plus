use crate::{prost_ext::MessageExt, proto, ErrorReport, Result};
use prost_types::Any;
use core::convert::TryFrom;

#[derive(Clone, Debug, PartialEq)]
pub struct CosmosTx {
    pub messages: Vec<Any>,
}

impl CosmosTx {
    pub fn new<I>(
        messages: I,
    ) -> Self
    where
        I: IntoIterator<Item = Any>,
    {
        CosmosTx { 
            messages: messages.into_iter().map(Into::into).collect(),
        }
    }

    pub fn into_proto(self) -> proto::ibc::applications::interchain_accounts::v1::CosmosTx {
        self.into()
    }

    /// Encode this type using Protocol Buffers.
    pub fn into_bytes(self) -> Result<Vec<u8>> {
        self.into_proto().to_bytes()
    }

}

impl From<CosmosTx> for proto::ibc::applications::interchain_accounts::v1::CosmosTx {
    fn from(cosmos_tx: CosmosTx) -> proto::ibc::applications::interchain_accounts::v1::CosmosTx {
        proto::ibc::applications::interchain_accounts::v1::CosmosTx {
            messages: cosmos_tx.messages.into_iter().map(Into::into).collect(),
        }
    }
}

impl TryFrom<proto::ibc::applications::interchain_accounts::v1::CosmosTx> for CosmosTx {
    type Error = ErrorReport;

    fn try_from(proto: proto::ibc::applications::interchain_accounts::v1::CosmosTx) -> Result<CosmosTx> {
        Ok(CosmosTx {
            messages: proto.messages.into_iter().map(Into::into).collect(),
        })
    }
}
