use chrono::{DateTime, Utc};
use thiserror::Error;

mod position_intents;
pub use position_intents::{
    Amount, Identifier, PositionIntent, PositionIntentBuilder, UpdatePolicy,
};
mod trade_intents;
pub use trade_intents::{OrderType, TimeInForce, TradeIntent, TradeMessage};

#[derive(Error, Clone, Debug)]
pub enum Error {
    #[error(
        "Non-`Zero` `AmountSpec`s of different type cannot be merged.\nLeft: {0:?}, Right: {1:?}"
    )]
    IncompatibleAmountError(Amount, Amount),
    #[error("Cannot create PositionIntent with `before` < `after`. \nBefore: {0}, After: {1}")]
    InvalidBeforeAfter(DateTime<Utc>, DateTime<Utc>),
    #[error("Identifier `All` can only be used with the `Dollars` and `Shares` `Amount`s")]
    InvalidCombination,
}
