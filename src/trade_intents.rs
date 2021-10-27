use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(tag = "order_type", rename_all = "snake_case")]
pub enum OrderType {
    Market,
    Limit {
        limit_price: Decimal,
    },
    Stop {
        stop_price: Decimal,
    },
    StopLimit {
        stop_price: Decimal,
        limit_price: Decimal,
    },
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum TimeInForce {
    #[serde(rename = "gtc")]
    GoodTilCanceled,
    #[serde(rename = "day")]
    Day,
    #[serde(rename = "ioc")]
    ImmediateOrCancel,
    #[serde(rename = "fok")]
    FillOrKill,
    #[serde(rename = "opg")]
    Open,
    #[serde(rename = "cls")]
    Close,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct TradeIntent {
    pub id: Uuid,
    pub ticker: String,
    pub qty: isize,
    #[serde(flatten)]
    pub order_type: OrderType,
    pub time_in_force: TimeInForce,
}

impl TradeIntent {
    pub fn new(ticker: impl Into<String>, qty: isize) -> Self {
        Self {
            id: Uuid::new_v4(),
            ticker: ticker.into(),
            qty,
            order_type: OrderType::Market,
            time_in_force: TimeInForce::Day,
        }
    }

    pub fn id(mut self, id: Uuid) -> Self {
        self.id = id;
        self
    }

    pub fn order_type(mut self, order_type: OrderType) -> Self {
        self.order_type = order_type;
        self
    }

    pub fn time_in_force(mut self, time_in_force: TimeInForce) -> Self {
        self.time_in_force = time_in_force;
        self
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum TradeMessage {
    New { intent: TradeIntent },
    Cancel { id: Uuid },
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_serialize_and_deserialize() {
        let intent = TradeIntent::new("AAPL", 10)
            .id(Uuid::new_v4())
            .order_type(OrderType::StopLimit {
                stop_price: Decimal::new(100, 0),
                limit_price: Decimal::new(101, 0),
            })
            .time_in_force(TimeInForce::ImmediateOrCancel);
        let new_message = TradeMessage::New {
            intent: intent.clone(),
        };
        let cancel_message = TradeMessage::Cancel { id: intent.id };
        let new_serialized = serde_json::to_string(&new_message).unwrap();
        let new_deserialized = serde_json::from_str(&new_serialized).unwrap();
        let cancel_serialized = serde_json::to_string(&cancel_message).unwrap();
        let cancel_deserialized = serde_json::from_str(&cancel_serialized).unwrap();
        assert_eq!(new_message, new_deserialized);
        assert_eq!(cancel_message, cancel_deserialized);
    }
}
