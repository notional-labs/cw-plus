#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, DepsMut, Env, IbcMsg, IbcQuery, MessageInfo, Order,
    PortIdResponse, Response, StdResult, Uint128,
};

use cw2::{get_contract_version, set_contract_version};

use crate::amount::Amount;
use crate::error::ContractError;
use crate::ibc::{Ics20Packet,InterchainAccountPacketData, get_escrow_address, write_memo};
use crate::msg::{
    ChannelResponse, ExecuteMsg, InitMsg, ListChannelsResponse, MigrateMsg, PortResponse, QueryMsg,
    TransferMsg, SwapMsg, JoinPoolMsg, BalanceResponse, IbcDenomResponse
};
use crate::tx::{
    MsgSwapExactAmountIn,MsgJoinSwapExternAmountIn,MsgSend,Msg, CosmosTx,
};
use crate::base::{Coin,SwapAmountInRoute,};
use crate::state::{Config, DENOM, CHANNEL_INFO, CHANNEL_STATE, CONFIG, BALANCES, CONNECTION_TO_IBC_DENOM, TRANSFER_ACTION, SWAP_ACTION, JOIN_POOL_ACTION};
use cw0::{one_coin};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw20-ics20";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InitMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let cfg = Config {
        default_timeout: msg.default_timeout,
    };
    CONFIG.save(deps.storage, &cfg)?;
    for balance in msg.balances {
        BALANCES.save(deps.storage, &Addr::unchecked(balance.address), &balance.amount.into())?;
    }
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Transfer(msg) => {
            let coin = one_coin(&info)?;
            execute_transfer(deps, env, msg, Amount::Native(coin), info.sender)
        },
        ExecuteMsg::Swap(msg) => {
            let coin = one_coin(&info)?;
            execute_ibc_swap(deps, env, msg, Amount::Native(coin), info.sender)
        },
        ExecuteMsg::JoinPool(msg) => {
            let coin = one_coin(&info)?;
            execute_ibc_join(deps, env, msg, Amount::Native(coin), info.sender)
        }

    }
}

pub fn execute_ibc_join(
    deps: DepsMut,
    env: Env,
    msg: JoinPoolMsg,
    amount: Amount,
    sender: Addr,
) -> Result<Response, ContractError> {
    if amount.is_empty() {
        return Err(ContractError::NoFunds {});
    }
    // ensure the requested channel is registered
    // FIXME: add a .has method to map to make this faster
    let chan_info = CHANNEL_INFO.may_load(deps.storage, &msg.channel)?; 
    
    if chan_info.is_none() {
        return Err(ContractError::NoSuchChannel { id: msg.channel });
    }

    let chan_info_unwraped = chan_info.unwrap();
    // let connection_id =chan_info_unwraped.connection_id;

    // let ibc_denom = CONNECTION_TO_IBC_DENOM.may_load(deps.storage, &connection_id).unwrap().unwrap();

    let gamm_denom = "gamm/pool/".to_string() + &msg.pool_id.to_string();

    let ica_addr = chan_info_unwraped.ica_addr;

    if ica_addr == "" {
        return Err(ContractError::NotAnIcaChan {channel_id: msg.channel})
    }

    // delta from user is in seconds
    let timeout_delta = match msg.timeout {
        Some(t) => t,
        None => CONFIG.load(deps.storage)?.default_timeout,
    };
    // timeout is in nanoseconds
    let timeout = env.block.time.plus_seconds(timeout_delta);

    let escrow_addr = get_escrow_address(msg.channel.to_string(), JOIN_POOL_ACTION);
    let in_amount_u128 : u128 = msg.in_amount.parse::<u128>().unwrap();
    try_send(deps, &sender, &escrow_addr, in_amount_u128.into()).unwrap();

    let token_in = Coin{
        denom: msg.in_denom.to_string(),
        amount: msg.in_amount.to_string(),
    };

    let join_msg = MsgJoinSwapExternAmountIn{
        sender: ica_addr.to_string(),
        token_in: token_in,
        share_out_min_amount: msg.share_out_exact_amount.to_string(),
        pool_id: msg.pool_id,
    }.to_any().unwrap();

    let token_out = Coin{
        denom: gamm_denom,
        amount: msg.share_out_exact_amount.to_string(),
    };


    let send_msg = MsgSend{
        from_address: ica_addr.to_string(),
        to_address: msg.remote_address.to_string(),
        amount: vec![token_out],
    }.to_any().unwrap();

    let cosmos_tx = CosmosTx::new([join_msg, send_msg]).into_bytes().unwrap();

    let packet = InterchainAccountPacketData{
        r#type: 1,
        data: cosmos_tx,
        memo: write_memo(&sender, JOIN_POOL_ACTION),
    };
    // prepare message
    let msg = IbcMsg::SendPacket {
        channel_id: msg.channel,
        data: to_binary(&packet)?,
        timeout: timeout.into(),
    };

    // Note: we update local state when we get ack - do not count this transfer towards anything until acked
    // similar event messages like ibctransfer module

    // send response
    let res = Response::new()
        .add_message(msg);
    Ok(res)
}

pub fn execute_ibc_swap(
    deps: DepsMut,
    env: Env,
    msg: SwapMsg,
    amount: Amount,
    sender: Addr,
) -> Result<Response, ContractError> {
    if amount.is_empty() {
        return Err(ContractError::NoFunds {});
    }
    // ensure the requested channel is registered
    // FIXME: add a .has method to map to make this faster
    let chan_info = CHANNEL_INFO.may_load(deps.storage, &msg.channel)?;

    if chan_info.is_none() {
        return Err(ContractError::NoSuchChannel { id: msg.channel });
    }

    let unwraped_chan_info = chan_info.unwrap();
    
    // let connection_id = unwraped_chan_info.connection_id.to_string();

    // let ibc_denom = CONNECTION_TO_IBC_DENOM.may_load(deps.storage, &connection_id)?.unwrap();
    

    let ica_addr = unwraped_chan_info.ica_addr;

    if ica_addr == "" {
        return Err(ContractError::NotAnIcaChan {channel_id: msg.channel})
    }

    // delta from user is in seconds
    let timeout_delta = match msg.timeout {
        Some(t) => t,
        None => CONFIG.load(deps.storage)?.default_timeout,
    };
    // timeout is in nanoseconds
    let timeout = env.block.time.plus_seconds(timeout_delta);

    let escrow_addr = get_escrow_address(msg.channel.to_string(), SWAP_ACTION);
    let in_amount_u128 : u128 = msg.in_amount.parse::<u128>().unwrap();
    try_send(deps, &sender, &escrow_addr, in_amount_u128.into()).unwrap();

    let route = SwapAmountInRoute{
        pool_id : msg.pool_id,
        token_out_denom: msg.out_denom.to_string(),
    };
    let token_in = Coin{
        denom: msg.in_denom.to_string(),
        amount: msg.in_amount.to_string(),
    };


    let swap_msg = MsgSwapExactAmountIn{
        sender: ica_addr.to_string(),
        routes: vec![route],
        token_in: token_in,
        token_out_min_amount: msg.exact_amount_out.to_string(),
    }.to_any().unwrap();

    let token_out = Coin{
        denom: msg.out_denom.to_string(),
        amount: msg.exact_amount_out.to_string(),
    };


    let send_msg = MsgSend{
        from_address: ica_addr.to_string(),
        to_address: msg.remote_address.to_string(),
        amount: vec![token_out],
    }.to_any().unwrap();

    let cosmos_tx = CosmosTx::new([swap_msg, send_msg]).into_bytes().unwrap();

    let packet = InterchainAccountPacketData{
        r#type: 1,
        data: cosmos_tx,
        memo: "".to_string(),
    };
    // prepare message
    let msg = IbcMsg::SendPacket {
        channel_id: msg.channel,
        data: to_binary(&packet)?,
        timeout: timeout.into(),
    };

    // Note: we update local state when we get ack - do not count this transfer towards anything until acked
    // similar event messages like ibctransfer module

    // send response
    let res = Response::new()
        .add_message(msg);
    Ok(res)
}

pub fn try_send(deps: DepsMut, from: &Addr, to: &Addr, amount: Uint128) -> Result<Response, ContractError>{
    BALANCES.update(
        deps.storage,
        from,
        |balance: Option<Uint128>| -> StdResult<_> {
            Ok(balance.unwrap_or_default().checked_sub(amount)?)
        },
    )?;
    BALANCES.update(
        deps.storage,
        to,
        |balance: Option<Uint128>| -> StdResult<_> { Ok(balance.unwrap_or_default() + amount) },
    )?;

    let res = Response::new()
    .add_attribute("action", "transfer")
    .add_attribute("from", from)
    .add_attribute("to", to)
    .add_attribute("amount", amount);
    Ok(res)
}

pub fn execute_transfer(
    deps: DepsMut,
    env: Env,
    msg: TransferMsg,
    amount: Amount,
    sender: Addr,
) -> Result<Response, ContractError> {
    if amount.is_empty() {
        return Err(ContractError::NoFunds {});
    }
    
    // FIXME: add a .has method to map to make this faster
    let chan_info = CHANNEL_INFO.may_load(deps.storage, &msg.channel)?; 

    if chan_info.is_none() {
        return Err(ContractError::NoSuchChannel { id: msg.channel });
    }

    // delta from user is in seconds
    let timeout_delta = match msg.timeout {
        Some(t) => t,
        None => CONFIG.load(deps.storage)?.default_timeout,
    };
    // timeout is in nanoseconds
    let timeout = env.block.time.plus_seconds(timeout_delta);

    let escrow_addr = get_escrow_address(msg.channel.to_string(), TRANSFER_ACTION);

    try_send(deps, &sender, &escrow_addr, msg.amount.into()).unwrap();

    // build ics20 packet
    let packet = Ics20Packet::new(
        msg.amount.into(),
        DENOM,
        sender.as_ref(),
        &msg.remote_address,
    );
    packet.validate()?;

    // prepare message
    let msg = IbcMsg::SendPacket {
        channel_id: msg.channel,
        data: to_binary(&packet)?,
        timeout: timeout.into(),
    };

    // Note: we update local state when we get ack - do not count this transfer towards anything until acked
    // similar event messages like ibctransfer module

    // send response
    let res = Response::new()
        .add_message(msg)
        .add_attribute("action", "transfer")
        .add_attribute("sender", &packet.sender)
        .add_attribute("receiver", &packet.receiver)
        .add_attribute("denom", &packet.denom)
        .add_attribute("amount", &packet.amount.to_string());
    Ok(res)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    let version = get_contract_version(deps.storage)?;
    if version.contract != CONTRACT_NAME {
        return Err(ContractError::CannotMigrate {
            previous_contract: version.contract,
        });
    }
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Port {} => to_binary(&query_port(deps)?),
        QueryMsg::ListChannels {} => to_binary(&query_list(deps)?),
        QueryMsg::Channel { id } => to_binary(&query_channel(deps, id)?),
        QueryMsg::Balance{ address} => to_binary(&query_balance(deps, address)?),
        QueryMsg::IbcDenom{ connection_id } => to_binary(&query_ibc_denom(deps, connection_id)?)
    }
}

fn query_ibc_denom(deps: Deps, connection_id: String) -> StdResult<IbcDenomResponse> {
    let ibc_denom = CONNECTION_TO_IBC_DENOM.load(deps.storage, &connection_id)?;
    Ok(IbcDenomResponse {ibc_denom: ibc_denom})
}

fn query_balance(deps: Deps, address: String) -> StdResult<BalanceResponse> {
    let amount = BALANCES.load(deps.storage, &Addr::unchecked(address))?;
    Ok(BalanceResponse { amount: amount.into() })
}

fn query_port(deps: Deps) -> StdResult<PortResponse> {
    let query = IbcQuery::PortId {}.into();
    let PortIdResponse { port_id } = deps.querier.query(&query)?;
    Ok(PortResponse { port_id })
}

fn query_list(deps: Deps) -> StdResult<ListChannelsResponse> {
    let channels: StdResult<Vec<_>> = CHANNEL_INFO
        .range(deps.storage, None, None, Order::Ascending)
        .map(|r| r.map(|(_, v)| v))
        .collect();
    Ok(ListChannelsResponse {
        channels: channels?,
    })
}

// make public for ibc tests
pub fn query_channel(deps: Deps, id: String) -> StdResult<ChannelResponse> {
    let info = CHANNEL_INFO.load(deps.storage, &id)?;
    // this returns Vec<(outstanding, total)>
    let state: StdResult<Vec<_>> = CHANNEL_STATE
        .prefix(&id)
        .range(deps.storage, None, None, Order::Ascending)
        .map(|r| {
            let (k, v) = r?;
            let denom = String::from_utf8(k)?;
            let outstanding = Amount::from_parts(denom.clone(), v.outstanding);
            let total = Amount::from_parts(denom, v.total_sent);
            Ok((outstanding, total))
        })
        .collect();
    // we want (Vec<outstanding>, Vec<total>)
    let (balances, total_sent) = state?.into_iter().unzip();

    Ok(ChannelResponse {
        info,
        balances,
        total_sent,
    })
}