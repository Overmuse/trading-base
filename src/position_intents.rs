use crate::Error;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum UpdatePolicy {
    Retain,
    RetainLong,
    RetainShort,
    Update,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Amount {
    Dollars(Decimal),
    Shares(Decimal),
    Zero,
}
impl Amount {
    pub fn merge(self, other: Self) -> Result<Self, Error> {
        match (self, other) {
            (Amount::Dollars(x), Amount::Dollars(y)) => Ok(Amount::Dollars(x + y)),
            (Amount::Shares(x), Amount::Shares(y)) => Ok(Amount::Shares(x + y)),
            (Amount::Zero, Amount::Zero) => Ok(Amount::Zero),
            (Amount::Zero, y) => Ok(y),
            (x, Amount::Zero) => Ok(x),
            (x, y) => Err(Error::IncompatibleAmountError(x, y)),
        }
    }

    pub const fn is_zero(&self) -> bool {
        match self {
            Amount::Dollars(x) => x.is_zero(),
            Amount::Shares(x) => x.is_zero(),
            Amount::Zero => true,
        }
    }

    pub const fn is_sign_positive(&self) -> bool {
        match self {
            Amount::Dollars(x) => x.is_sign_positive(),
            Amount::Shares(x) => x.is_sign_positive(),
            Amount::Zero => false,
        }
    }

    pub const fn is_sign_negative(&self) -> bool {
        match self {
            Amount::Dollars(x) => x.is_sign_negative(),
            Amount::Shares(x) => x.is_sign_negative(),
            Amount::Zero => false,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Identifier {
    Ticker(String),
    All,
}

impl<T: ToString> From<T> for Identifier {
    fn from(s: T) -> Self {
        Self::Ticker(s.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct PositionIntentBuilder {
    strategy: String,
    sub_strategy: Option<String>,
    identifier: Identifier,
    amount: Amount,
    update_policy: UpdatePolicy,
    decision_price: Option<Decimal>,
    limit_price: Option<Decimal>,
    stop_price: Option<Decimal>,
    before: Option<DateTime<Utc>>,
    after: Option<DateTime<Utc>>,
}

impl PositionIntentBuilder {
    pub fn sub_strategy(mut self, sub_strategy: impl Into<String>) -> Self {
        self.sub_strategy = Some(sub_strategy.into());
        self
    }

    pub fn decision_price(mut self, decision_price: Decimal) -> Self {
        self.decision_price = Some(decision_price);
        self
    }

    pub fn limit_price(mut self, limit_price: Decimal) -> Self {
        self.limit_price = Some(limit_price);
        self
    }

    pub fn stop_price(mut self, stop_price: Decimal) -> Self {
        self.stop_price = Some(stop_price);
        self
    }

    pub fn before(mut self, before: DateTime<Utc>) -> Self {
        self.before = Some(before);
        self
    }

    pub fn after(mut self, after: DateTime<Utc>) -> Self {
        self.after = Some(after);
        self
    }

    pub fn update_policy(mut self, policy: UpdatePolicy) -> Self {
        self.update_policy = policy;
        self
    }

    pub fn build(self) -> Result<PositionIntent, Error> {
        if let Some((before, after)) = self.before.zip(self.after) {
            if before < after {
                return Err(Error::InvalidBeforeAfter(before, after));
            }
        }
        match (self.identifier.clone(), self.amount.clone()) {
            (Identifier::All, Amount::Dollars(_)) => return Err(Error::InvalidCombination),
            (Identifier::All, Amount::Shares(_)) => return Err(Error::InvalidCombination),
            _ => (),
        }
        Ok(PositionIntent {
            id: Uuid::new_v4(),
            strategy: self.strategy,
            sub_strategy: self.sub_strategy,
            timestamp: Utc::now(),
            identifier: self.identifier,
            amount: self.amount,
            update_policy: self.update_policy,
            decision_price: self.decision_price,
            limit_price: self.limit_price,
            stop_price: self.stop_price,
            before: self.before,
            after: self.after,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PositionIntent {
    pub id: Uuid,
    /// The strategy that is requesting a position. Dollar limits are shared between all positions
    /// of the same strategy.
    pub strategy: String,
    /// Identifier for a specific leg of a position for a strategy. Sub-strategies must still
    /// adhere to the dollar limits of the strategy, but the order-manager will keep track of the
    /// holdings at the sub-strategy level.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_strategy: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub identifier: Identifier,
    pub amount: Amount,
    pub update_policy: UpdatePolicy,
    /// The price at which the decision was made to send a position request. This can be used by
    /// other parts of the app for execution analysis. This field might also be used for
    /// translating between dollars and shares by the order-manager.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision_price: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_price: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_price: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<DateTime<Utc>>,
}

impl PositionIntent {
    pub fn builder(
        strategy: impl Into<String>,
        identifier: impl Into<Identifier>,
        amount: Amount,
    ) -> PositionIntentBuilder {
        PositionIntentBuilder {
            strategy: strategy.into(),
            sub_strategy: None,
            identifier: identifier.into(),
            amount,
            update_policy: UpdatePolicy::Update,
            decision_price: None,
            limit_price: None,
            stop_price: None,
            before: None,
            after: None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::Duration;

    #[test]
    fn can_construct_position_intent() {
        let builder = PositionIntent::builder("A", "AAPL", Amount::Dollars(Decimal::new(1, 0)));
        let _intent = builder
            .sub_strategy("B")
            .decision_price(Decimal::new(2, 0))
            .limit_price(Decimal::new(3, 0))
            .stop_price(Decimal::new(3, 0))
            .update_policy(UpdatePolicy::Retain)
            .before(Utc::now() + Duration::hours(1))
            .after(Utc::now())
            .build()
            .unwrap();
    }

    #[test]
    fn can_serialize_and_deserialize() {
        let builder = PositionIntent::builder("A", "AAPL", Amount::Shares(Decimal::new(1, 0)));
        let intent = builder
            .sub_strategy("B")
            .decision_price(Decimal::new(2, 0))
            .limit_price(Decimal::new(3, 0))
            .stop_price(Decimal::new(3, 0))
            .update_policy(UpdatePolicy::Retain)
            .before(Utc::now() + Duration::hours(1))
            .after(Utc::now())
            .build()
            .unwrap();
        let serialized = serde_json::to_string(&intent).unwrap();
        let deserialized = serde_json::from_str(&serialized).unwrap();
        assert_eq!(intent, deserialized);
    }
}
