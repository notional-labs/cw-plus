use cw20_ics20::tx::{MsgSwapExactAmountIn, Msg};
use cw20_ics20::Coin;
use cw20_ics20::tx::CosmosTx;
use cw20_ics20::SwapAmountInRoute;



use std::{str};

/// Chain ID to use for tests

/// RPC port

/// Expected account number

/// Bech32 prefix for an account
/// Denom name
const DENOM: &str = "test";

/// Example memo

fn main() {
    let test_routes = SwapAmountInRoute {
        pool_id: 1u8.into(),
        token_out_denom: DENOM.parse().unwrap(),
    };
    let amount = Coin {
        amount: 1u8.into(),
        denom: DENOM.parse().unwrap(),
    };
    let msg_send = MsgSwapExactAmountIn {
        sender: "test".to_string(),
        routes: vec![test_routes.clone()],
        token_in: amount.clone(),
        token_out_min_amount: "1".to_string(),
    }
    .to_any()
    .unwrap();

    let cosmos_tx = CosmosTx::new(vec![msg_send]);
    let bz = cosmos_tx.into_bytes();
    println!("dasf{:?}", bz);
}