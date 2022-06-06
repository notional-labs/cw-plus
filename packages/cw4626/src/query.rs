use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Cw4626QueryMsg {
    // This message to return the total amount of underlying tokens held by the vault.
    TotalHoldings {},

    // This message to return the total amount of underlying tokens held in the vault for owner.
    BalanceOfUnderlying { owner: Addr },

    // Returns the address of the token the vault uses for accounting, depositing, and withdrawing.
    Underlying {},

    // Returns the total number of unredeemed vault shares in circulation.
    TotalSupply {},

    // Returns the total amount of vault shares the _owner currently has.
    BalanceOf {owner: Addr},

    // The amount of underlying tokens one baseUnit of vault shares is redeemable for.
    ExchangeRate {},

    // The decimal scalar for vault shares and operations involving exchangeRate.
    BaseUnit {},
}
