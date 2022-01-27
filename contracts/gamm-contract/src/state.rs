use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Addr, Uint128};

use cosmwasm_std::{IbcEndpoint};
use cw_storage_plus::{Item, Map};

pub const CONFIG: Item<Config> = Item::new("ics20_config");

// static info on one channel that doesn't change
pub const CHANNEL_INFO: Map<&str, ChannelInfo> = Map::new("channel_info");

// indexed by (channel_id, denom) maintaining the balance of the channel in that currency
pub const CHANNEL_STATE: Map<(&str, &str), ChannelState> = Map::new("channel_state");

pub const BALANCES: Map<&Addr, Uint128> = Map::new("balance");

pub const CONNECTION_TO_IBC_DENOM: Map<&str, String> = Map::new("connection_to_ibc_denom");

pub const DENOM: &str = "test";
pub const TRANSFER_ACTION: &str = "transfer";
pub const SWAP_ACTION: &str = "swap";
pub const JOIN_POOL_ACTION: &str = "join_pool";

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct ChannelState {
    pub outstanding: Uint128,
    pub total_sent: Uint128,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct Config {
    pub default_timeout: u64,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct ChannelInfo {
    /// id of this channel
    pub id: String,
    /// the remote channel/port we connect to
    pub counterparty_endpoint: IbcEndpoint,
    /// the connection this exists on (you can use to query client/consensus info)
    pub connection_id: String,

    pub ica_addr: String,
}
