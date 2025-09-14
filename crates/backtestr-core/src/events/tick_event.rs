use backtestr_data::Tick;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickEvent {
    pub tick: Tick,
    pub received_at: i64,
    pub processing_sequence: u64,
}

impl TickEvent {
    pub fn new(tick: Tick, sequence: u64) -> Self {
        Self {
            tick,
            received_at: chrono::Utc::now().timestamp_millis(),
            processing_sequence: sequence,
        }
    }

    pub fn from_tick(tick: Tick) -> Self {
        Self::new(tick, 0)
    }

    pub fn with_sequence(mut self, sequence: u64) -> Self {
        self.processing_sequence = sequence;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tick_event_creation() {
        let tick = Tick::new_with_millis("EURUSD".to_string(), 1704067230000, 1.0920, 1.0922);
        let event = TickEvent::from_tick(tick.clone());

        assert_eq!(event.tick.symbol, "EURUSD");
        assert_eq!(event.processing_sequence, 0);
        assert!(event.received_at > 0);
    }

    #[test]
    fn test_tick_event_with_sequence() {
        let tick = Tick::new_with_millis("EURUSD".to_string(), 1704067230000, 1.0920, 1.0922);
        let event = TickEvent::from_tick(tick).with_sequence(42);

        assert_eq!(event.processing_sequence, 42);
    }
}
