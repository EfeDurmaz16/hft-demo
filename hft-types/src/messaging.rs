use crate::{EnrichedTick, Order, OrderBook, TradingSignal};
use serde::{Deserialize, Serialize};

/// Message types for inter-process communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    /// Market tick from simulator to feed handler
    Tick(crate::MarketTick),

    /// Enriched tick from feed handler to strategy
    EnrichedTick(EnrichedTick),

    /// Trading signal from strategy to order gateway
    Signal(TradingSignal),

    /// Order from strategy/gateway
    Order(Order),

    /// Order book update
    OrderBookUpdate(OrderBook),

    /// Heartbeat for connection monitoring
    Heartbeat { sender: String, timestamp: u128 },

    /// System control messages
    Shutdown,
}

impl Message {
    pub fn serialize(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    pub fn deserialize(data: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(data)
    }
}

/// TCP message frame with length prefix
pub struct MessageFrame {
    pub length: u32,
    pub payload: Vec<u8>,
}

impl MessageFrame {
    pub fn new(message: &Message) -> Result<Self, serde_json::Error> {
        let payload = message.serialize()?;
        Ok(Self {
            length: payload.len() as u32,
            payload,
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(4 + self.payload.len());
        bytes.extend_from_slice(&self.length.to_be_bytes());
        bytes.extend_from_slice(&self.payload);
        bytes
    }

    pub fn from_length_and_payload(length: u32, payload: Vec<u8>) -> Self {
        Self { length, payload }
    }

    pub fn parse_message(&self) -> Result<Message, serde_json::Error> {
        Message::deserialize(&self.payload)
    }
}
