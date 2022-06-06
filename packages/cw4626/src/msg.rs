use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128};

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all="snake_case")]
pub enum Cw4626ExecuteMsg {
    /// This message deposits `amount` tokens into the vault and grants ownership to `receiver`.
    Deposit { receiver: Addr, amount: Uint128 },
    /// Mints `shares` Vault shares to `receiver` by depositing exactly `amount` of underlying tokens.
    Mint { receiver: Addr, shares: Uint128 },
    /// This message withdraws `amount` token from the vault and transfers them to `receiver`.
    Withdraw { receiver: Addr, amount: Uint128 },
    /// Redeems a specific number of `shares` for underlying tokens and transfers them to `receiver`.
    Redeem { receiver: Addr, shares: Uint128 },
}
