use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{
    attr, entry_point, from_binary, to_binary, Binary, ContractResult, DepsMut, Env,
    IbcBasicResponse, IbcChannel, IbcChannelCloseMsg, IbcChannelConnectMsg, IbcChannelOpenMsg,
    IbcEndpoint, IbcOrder, IbcPacket, IbcPacketAckMsg, IbcPacketReceiveMsg, IbcPacketTimeoutMsg,
    IbcReceiveResponse, Reply, Response, StdResult, Uint128,  Addr, Attribute,
};

use core::convert::AsRef;
use std::convert::TryFrom;
use hex::ToHex;


use sha2::{Sha256, Digest};

use subtle_encoding::bech32;

use crate::tx::{CosmosTx, Msg};

use crate::contract::try_send;
use crate::tx::{MsgSwapExactAmountIn, MsgJoinSwapExternAmountIn};
use crate::{proto};
use crate::error::{ContractError, Never};
use crate::state::{ChannelInfo, CHANNEL_INFO, CHANNEL_STATE, DENOM, CONNECTION_TO_IBC_DENOM,
    TRANSFER_ACTION, SWAP_ACTION, JOIN_POOL_ACTION,};

pub const ICS20_VERSION: &str = "ics20-1";
pub const ICS20_ORDERING: IbcOrder = IbcOrder::Unordered;

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct Ics20Packet {
    /// amount of tokens to transfer is encoded as a string, but limited to u64 max
    pub amount: Uint128,
    /// the token denomination to be transferred
    pub denom: String,
    /// the recipient address on the destination chain
    pub receiver: String,
    /// the sender address
    pub sender: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct InterchainAccountPacketData {
    pub r#type: i32,

    pub data: Vec<u8>,

    pub memo: String,
}

impl InterchainAccountPacketData {
    pub fn new<T: Into<String>>(r#type: i32, data: Vec<u8>, memo: &str) -> Self {
        InterchainAccountPacketData {
            r#type: r#type.into(),
            data: data,
            memo: memo.to_string(),
        }
    }
}

impl Ics20Packet {
    pub fn new<T: Into<String>>(amount: Uint128, denom: T, sender: &str, receiver: &str) -> Self {
        Ics20Packet {
            denom: denom.into(),
            amount,
            sender: sender.to_string(),
            receiver: receiver.to_string(),
        }
    }

    pub fn validate(&self) -> Result<(), ContractError> {
        if self.amount.u128() > (u64::MAX as u128) {
            Err(ContractError::AmountOverflow {})
        } else {
            Ok(())
        }
    }
}

/// This is a generic ICS acknowledgement format.
/// Proto defined here: https://github.com/cosmos/cosmos-sdk/blob/v0.42.0/proto/ibc/core/channel/v1/channel.proto#L141-L147
/// This is compatible with the JSON serialization
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Ics20Ack {
    Result(Binary),
    Error(String),
}

// create a serialized success message
fn ack_success() -> Binary {
    let res = Ics20Ack::Result(b"1".into());
    to_binary(&res).unwrap()
}

// create a serialized error message
fn ack_fail(err: String) -> Binary {
    let res = Ics20Ack::Error(err);
    to_binary(&res).unwrap()
}

const SEND_TOKEN_ID: u64 = 1337;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, reply: Reply) -> Result<Response, ContractError> {
    if reply.id != SEND_TOKEN_ID {
        return Err(ContractError::UnknownReplyId { id: reply.id });
    }
    let res = match reply.result {
        ContractResult::Ok(_) => Response::new(),
        ContractResult::Err(err) => {
            // encode an acknowledgement error
            Response::new().set_data(ack_fail(err))
        }
    };
    Ok(res)
}

#[cfg_attr(not(feature = "library"), entry_point)]
/// enforces ordering and versioning constraints
pub fn ibc_channel_open(
    _deps: DepsMut,
    _env: Env,
    msg: IbcChannelOpenMsg,
) -> Result<(), ContractError> {

    enforce_order_and_version(msg.channel(), msg.counterparty_version())?;
    Ok(())
}


pub fn get_ica_address(controller_port_id: &String) -> String{
        // create a Sha256 object
    let mut hasher = Sha256::new();

    // first hash ica module acc addr
    hasher.update([103, 215, 116, 116, 202, 142, 58, 88, 18, 222, 50, 58, 40, 215, 198, 218, 107, 62, 79, 41]);

    let ica_module_acc_addr_hash = hasher.finalize();

    let new_hasher = Sha256::new();


    let out = new_hasher.chain_update(ica_module_acc_addr_hash).chain_update(controller_port_id.as_bytes()).finalize();


    // let mut x = vec![0;32];

    // x[..32].clone_from_slice(&out.as_slice());

    let ica_addr = bech32::encode("cosmos", &out.as_slice());



    return ica_addr;
}

pub fn get_escrow_address(channel_id: String, action: &str) -> Addr {
    return Addr::unchecked(channel_id + "/" + action);
}

pub fn get_ibc_denom(dest_port: String, dest_chan: &str) -> String {
    let denom_path = dest_port + "/" + dest_chan + "/" + DENOM;
    let mut hasher = Sha256::new();
    hasher.update(denom_path.as_bytes());

    let denom_path_hash = hasher.finalize();
    // let mut hash_bz = vec![0;32];

    // hash_bz[..32].clone_from_slice(&denom_path_hash.as_slice());

    let mut s = String::with_capacity(2 * denom_path_hash.len());
    denom_path_hash.write_hex(&mut s).expect("Failed to write");
    return "ibc/".to_string() + &s.to_uppercase();
}

pub fn is_ica_port(controller_port_id: &String) -> bool{
    let split = controller_port_id.split(".");
    let vec = split.collect::<Vec<&str>>();
    if vec.len() != 4 {
        return false;
    } else {
        return true;
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
/// record the channel in CHANNEL_INFO
pub fn ibc_channel_connect(
    deps: DepsMut,
    _env: Env,
    msg: IbcChannelConnectMsg,
) -> Result<IbcBasicResponse, ContractError> {
    // we need to check the counter party version in try and ack (sometimes here)
    enforce_order_and_version(msg.channel(), msg.counterparty_version())?;

    let channel: IbcChannel = msg.channel().clone();
    let port_id = channel.endpoint.port_id;
    let info;

    if is_ica_port(&port_id) {
        info = ChannelInfo {
            id: channel.endpoint.channel_id,
            counterparty_endpoint: channel.counterparty_endpoint,
            connection_id: channel.connection_id,
            ica_addr: get_ica_address(&port_id)
        };
    
    } else {
        info = ChannelInfo {
            id: channel.endpoint.channel_id,
            counterparty_endpoint: channel.counterparty_endpoint.clone(),
            connection_id: channel.connection_id.to_string(),
            ica_addr: "".to_string()
        };

        let ibc_denom = CONNECTION_TO_IBC_DENOM.may_load(deps.storage, &*channel.connection_id.to_owned())?;
        if ibc_denom.is_none() {
            let ibc_denom = get_ibc_denom(info.counterparty_endpoint.port_id.to_string(), &channel.counterparty_endpoint.channel_id.clone());
            CONNECTION_TO_IBC_DENOM.save(deps.storage, &channel.connection_id, &ibc_denom)?;        
        }
      }
    CHANNEL_INFO.save(deps.storage, &info.id, &info)?;

    Ok(IbcBasicResponse::default())
}

fn enforce_order_and_version(
    _channel: &IbcChannel,
    _counterparty_version: Option<&str>,
) -> Result<(), ContractError> {
    // if channel.version != ICS20_VERSION {
    //     return Err(ContractError::InvalidIbcVersion {
    //         version: channel.version.clone(),
    //     });
    // }
    // if let Some(version) = counterparty_version {
    //     if version != ICS20_VERSION {
    //         return Err(ContractError::InvalidIbcVersion {
    //             version: version.to_string(),
    //         });
    //     }
    // }
    // if channel.order != ICS20_ORDERING {
    //     return Err(ContractError::OnlyOrderedChannel {});
    // }
    Ok(())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_channel_close(
    _deps: DepsMut,
    _env: Env,
    _channel: IbcChannelCloseMsg,
) -> Result<IbcBasicResponse, ContractError> {
    // TODO: what to do here?
    // we will have locked funds that need to be returned somehow
    unimplemented!();
}

#[cfg_attr(not(feature = "library"), entry_point)]
/// Check to see if we have any balance here
/// We should not return an error if possible, but rather an acknowledgement of failure
pub fn ibc_packet_receive(
    deps: DepsMut,
    _env: Env,
    msg: IbcPacketReceiveMsg,
) -> Result<IbcReceiveResponse, Never> {
    let packet = msg.packet;

    let res = match do_ibc_packet_receive(deps, &packet) {
        Ok(msg) => {


            // build attributes first so we don't have to clone msg below
            // similar event messages like ibctransfer module

            // This cannot fail as we parse it in do_ibc_packet_receive. Best to pass the data somehow?
            let denom = parse_voucher_denom(&msg.denom, &packet.src).unwrap();

            let attributes = vec![
                attr("action", "receive"),
                attr("sender", &msg.sender),
                attr("receiver", &msg.receiver),
                attr("denom", denom),
                attr("amount", msg.amount),
                attr("success", "true"),
            ];

            IbcReceiveResponse::new()
                .set_ack(ack_success())
                .add_attributes(attributes)
        }
        Err(err) => IbcReceiveResponse::new()
            .set_ack(ack_fail(err.to_string()))
            .add_attributes(vec![
                attr("action", "receive"),
                attr("success", "false"),
                attr("error", err.to_string()),
            ]),
    };

    // if we have funds, now send the tokens to the requested recipient
    Ok(res)
}

pub fn write_memo(signer: &Addr, action: &str) -> String {
    return signer.as_ref().to_string() + "/" + action
}


fn get_signer_and_action_from_memo(memo: String) -> Result<(String,String), ContractError> {
    let split = memo.split("/");
    let vec: Vec<&str> = split.collect();
    if vec.len() != 2 {
        return Err(ContractError::InvalidPacket{})
    }
    Ok((vec[0].to_string(), vec[1].to_string()))
}

// Returns local denom if the denom is an encoded voucher from the expected endpoint
// Otherwise, error
fn parse_voucher_denom<'a>(
    voucher_denom: &'a str,
    remote_endpoint: &IbcEndpoint,
) -> Result<&'a str, ContractError> {
    let split_denom: Vec<&str> = voucher_denom.splitn(3, '/').collect();
    if split_denom.len() != 3 {
        return Err(ContractError::NoForeignTokens {});
    }
    // a few more sanity checks
    if split_denom[0] != remote_endpoint.port_id {
        return Err(ContractError::FromOtherPort {
            port: split_denom[0].into(),
        });
    }
    if split_denom[1] != remote_endpoint.channel_id {
        return Err(ContractError::FromOtherChannel {
            channel: split_denom[1].into(),
        });
    }

    Ok(split_denom[2])
}

// this does the work of ibc_packet_receive, we wrap it to turn errors into acknowledgements
fn do_ibc_packet_receive(deps: DepsMut, packet: &IbcPacket) -> Result<Ics20Packet, ContractError> {
    let msg: Ics20Packet = from_binary(&packet.data)?;
    let channel = packet.dest.channel_id.clone();

    // If the token originated on the remote chain, it looks like "ucosm".
    // If it originated on our chain, it looks like "port/channel/ucosm".
    let denom = parse_voucher_denom(&msg.denom, &packet.src)?;
    if denom != DENOM {
        return Err(ContractError::InvalidCoin{});
    }

    let amount = msg.amount;
    CHANNEL_STATE.update(
        deps.storage,
        (&channel, denom),
        |orig| -> Result<_, ContractError> {
            // this will return error if we don't have the funds there to cover the request (or no denom registered)
            let mut cur = orig.ok_or(ContractError::InsufficientFunds {})?;
            cur.outstanding = cur
                .outstanding
                .checked_sub(amount)
                .or(Err(ContractError::InsufficientFunds {}))?;
            Ok(cur)
        },
    )?;
    let escrow_address = get_escrow_address(packet.dest.channel_id.to_string(),TRANSFER_ACTION);

    try_send(deps, &escrow_address, &Addr::unchecked(msg.receiver.to_string()), msg.amount).unwrap();

    Ok(msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
/// check if success or failure and update balance, or return funds
pub fn ibc_packet_ack(
    deps: DepsMut,
    _env: Env,
    msg: IbcPacketAckMsg,
) -> Result<IbcBasicResponse, ContractError> {
    // TODO: trap error like in receive?
    let ics20msg: Ics20Ack = from_binary(&msg.acknowledgement.data)?;
    match ics20msg {
        Ics20Ack::Result(_) => on_packet_success(deps, msg.original_packet),
        Ics20Ack::Error(err) => on_packet_failure(deps, msg.original_packet, err),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
/// return fund to original sender (same as failure in ibc_packet_ack)
pub fn ibc_packet_timeout(
    deps: DepsMut,
    _env: Env,
    msg: IbcPacketTimeoutMsg,
) -> Result<IbcBasicResponse, ContractError> {
    // TODO: trap error like in receive?
    let packet = msg.packet;
    on_packet_failure(deps, packet, "timeout".to_string())
}

// update the balance stored on this (channel, denom) index
fn on_packet_success(deps: DepsMut, packet: IbcPacket) -> Result<IbcBasicResponse, ContractError> {
    
    let mut attributes : Vec<Attribute> = vec![];
    
    if !is_ica_port(&packet.src.port_id) {
        let msg: Ics20Packet = from_binary(&packet.data)?;
        // similar event messages like ibctransfer module
        attributes = vec![
            attr("action", "acknowledge"),
            attr("sender", &msg.sender),
            attr("receiver", &msg.receiver),
            attr("denom", &msg.denom),
            attr("amount", msg.amount),
            attr("success", "true"),
        ];
    
        let channel = packet.src.channel_id;
        let denom = msg.denom;
        let amount = msg.amount;
        CHANNEL_STATE.update(deps.storage, (&channel, &denom), |orig| -> StdResult<_> {
            let mut state = orig.unwrap_or_default();
            state.outstanding += amount;
            state.total_sent += amount;
            Ok(state)
        })?;
    } 
  
    Ok(IbcBasicResponse::new().add_attributes(attributes))

}

// return the tokens to sender
fn on_packet_failure(
    deps: DepsMut,
    packet: IbcPacket,
    err: String,
) -> Result<IbcBasicResponse, ContractError> {
    let port_id = packet.src.port_id;
    let mut attributes : Vec<Attribute> = [].to_vec();
 
    if is_ica_port(&port_id) {
        let msg: InterchainAccountPacketData = from_binary(&packet.data)?;
        let cosmos_tx_bz = msg.data;
        let cosmos_tx_proto : proto::ibc::applications::interchain_accounts::v1::CosmosTx;
        cosmos_tx_proto =  prost::Message::decode(&*cosmos_tx_bz).unwrap();
        let cosmos_tx: CosmosTx = TryFrom::try_from(cosmos_tx_proto).unwrap();
        let (signer, action) = get_signer_and_action_from_memo(msg.memo).unwrap();
        let refund_addr = Addr::unchecked(signer);
        if action == "swap" {
            let swap_msg: MsgSwapExactAmountIn = Msg::from_any(&cosmos_tx.messages[0]).unwrap();
            let mut refund_amount = swap_msg.token_in.amount.parse::<u128>().unwrap();
            if swap_msg.token_in.denom != DENOM {
                refund_amount = 0;
            }
            let escrow_addr = get_escrow_address(packet.src.channel_id, SWAP_ACTION);
            try_send(deps, &escrow_addr, &refund_addr, refund_amount.into()).unwrap();

        } else if action == "join_pool" {
            let join_pool_msg: MsgJoinSwapExternAmountIn = Msg::from_any(&cosmos_tx.messages[0]).unwrap();
            let mut refund_amount = join_pool_msg.token_in.amount.parse::<u128>().unwrap();
            if join_pool_msg.token_in.denom != DENOM {
                refund_amount = 0;
            }
            let escrow_addr = get_escrow_address(packet.src.channel_id, JOIN_POOL_ACTION);
            try_send(deps, &escrow_addr, &refund_addr, refund_amount.into()).unwrap();

        } 
    } else {
        let msg: Ics20Packet = from_binary(&packet.data)?;

        // similar event messages like ibctransfer module
        attributes = vec![
            attr("action", "acknowledge"),
            attr("sender", &msg.sender),
            attr("receiver", &msg.receiver),
            attr("denom", &msg.denom),
            attr("amount", &msg.amount.to_string()),
            attr("success", "false"),
            attr("error", err),
        ];

        let mut refund_amount = msg.amount.into();
        if msg.denom != DENOM {
            refund_amount = 0;
        }
        let escrow_addr = get_escrow_address(packet.src.channel_id, TRANSFER_ACTION);
        let refund_addr = Addr::unchecked(msg.sender);

        try_send(deps, &escrow_addr, &refund_addr, refund_amount.into()).unwrap();

    }

    // refund
    Ok(IbcBasicResponse::new()
    .add_attributes(attributes))

}