use crate::events::{BarEvent, TickEvent};
use std::sync::Arc;

pub trait EventHandler: Send + Sync {
    fn on_tick(&self, event: &TickEvent);
    fn on_bar(&self, event: &BarEvent);
}

#[derive(Clone)]
pub struct EventDispatcher {
    handlers: Vec<Arc<dyn EventHandler>>,
    sequence_counter: Arc<std::sync::atomic::AtomicU64>,
}

impl EventDispatcher {
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
            sequence_counter: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }

    pub fn add_handler(&mut self, handler: Arc<dyn EventHandler>) {
        self.handlers.push(handler);
    }

    pub fn dispatch_tick(&self, event: &TickEvent) {
        for handler in &self.handlers {
            handler.on_tick(event);
        }
    }

    pub fn dispatch_bar(&self, event: &BarEvent) {
        for handler in &self.handlers {
            handler.on_bar(event);
        }
    }

    pub fn next_sequence(&self) -> u64 {
        self.sequence_counter
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }

    pub fn clear_handlers(&mut self) {
        self.handlers.clear();
    }

    pub fn handler_count(&self) -> usize {
        self.handlers.len()
    }
}

impl Default for EventDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use backtestr_data::{Bar, Tick, Timeframe};
    use std::sync::{Arc, Mutex};

    struct TestHandler {
        tick_count: Arc<Mutex<u32>>,
        bar_count: Arc<Mutex<u32>>,
    }

    impl TestHandler {
        fn new() -> Self {
            Self {
                tick_count: Arc::new(Mutex::new(0)),
                bar_count: Arc::new(Mutex::new(0)),
            }
        }
    }

    impl EventHandler for TestHandler {
        fn on_tick(&self, _event: &TickEvent) {
            let mut count = self.tick_count.lock().unwrap();
            *count += 1;
        }

        fn on_bar(&self, _event: &BarEvent) {
            let mut count = self.bar_count.lock().unwrap();
            *count += 1;
        }
    }

    #[test]
    fn test_event_dispatcher_creation() {
        let dispatcher = EventDispatcher::new();
        assert_eq!(dispatcher.handler_count(), 0);
    }

    #[test]
    fn test_add_handler() {
        let mut dispatcher = EventDispatcher::new();
        let handler = Arc::new(TestHandler::new());
        dispatcher.add_handler(handler);
        assert_eq!(dispatcher.handler_count(), 1);
    }

    #[test]
    fn test_dispatch_tick_event() {
        let mut dispatcher = EventDispatcher::new();
        let handler = Arc::new(TestHandler::new());
        let tick_count = handler.tick_count.clone();

        dispatcher.add_handler(handler);

        let tick = Tick::new_with_millis("EURUSD".to_string(), 1704067230000, 1.0920, 1.0922);
        let event = TickEvent::from_tick(tick);

        dispatcher.dispatch_tick(&event);

        let count = tick_count.lock().unwrap();
        assert_eq!(*count, 1);
    }

    #[test]
    fn test_dispatch_bar_event() {
        let mut dispatcher = EventDispatcher::new();
        let handler = Arc::new(TestHandler::new());
        let bar_count = handler.bar_count.clone();

        dispatcher.add_handler(handler);

        let bar = Bar::new(
            "EURUSD".to_string(),
            Timeframe::M1,
            1704067200000,
            1704067260000,
            1.0920,
            1.0925,
            1.0918,
            1.0923,
        );
        let event = BarEvent::bar_closed(bar, 1);

        dispatcher.dispatch_bar(&event);

        let count = bar_count.lock().unwrap();
        assert_eq!(*count, 1);
    }

    #[test]
    fn test_sequence_counter() {
        let dispatcher = EventDispatcher::new();

        let seq1 = dispatcher.next_sequence();
        let seq2 = dispatcher.next_sequence();
        let seq3 = dispatcher.next_sequence();

        assert_eq!(seq1, 0);
        assert_eq!(seq2, 1);
        assert_eq!(seq3, 2);
    }

    #[test]
    fn test_clear_handlers() {
        let mut dispatcher = EventDispatcher::new();
        let handler = Arc::new(TestHandler::new());

        dispatcher.add_handler(handler);
        assert_eq!(dispatcher.handler_count(), 1);

        dispatcher.clear_handlers();
        assert_eq!(dispatcher.handler_count(), 0);
    }
}
