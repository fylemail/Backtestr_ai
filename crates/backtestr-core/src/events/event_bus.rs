use super::bar_completion::BarCompletionEvent;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

type EventCallback = Arc<dyn Fn(&BarCompletionEvent) + Send + Sync>;

pub struct EventBus {
    subscribers: Arc<Mutex<HashMap<String, Vec<EventCallback>>>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            subscribers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn subscribe<F>(&self, event_type: &str, callback: F) -> SubscriptionHandle
    where
        F: Fn(&BarCompletionEvent) + Send + Sync + 'static,
    {
        let mut subs = self.subscribers.lock().unwrap();
        let callbacks = subs.entry(event_type.to_string()).or_default();
        let callback_arc = Arc::new(callback);
        callbacks.push(callback_arc.clone());

        SubscriptionHandle {
            event_type: event_type.to_string(),
            callback_id: callbacks.len() - 1,
        }
    }

    pub fn subscribe_all<F>(&self, callback: F) -> SubscriptionHandle
    where
        F: Fn(&BarCompletionEvent) + Send + Sync + 'static,
    {
        self.subscribe("*", callback)
    }

    pub fn publish(&self, event: BarCompletionEvent) {
        let event_type = event.timeframe_name();
        let subs = self.subscribers.lock().unwrap();

        // Call specific subscribers
        if let Some(callbacks) = subs.get(event_type) {
            for callback in callbacks {
                callback(&event);
            }
        }

        // Call wildcard subscribers
        if let Some(callbacks) = subs.get("*") {
            for callback in callbacks {
                callback(&event);
            }
        }
    }

    pub fn unsubscribe(&self, handle: SubscriptionHandle) {
        let mut subs = self.subscribers.lock().unwrap();
        if let Some(callbacks) = subs.get_mut(&handle.event_type) {
            if handle.callback_id < callbacks.len() {
                callbacks.remove(handle.callback_id);
            }
        }
    }

    pub fn clear_subscribers(&self, event_type: &str) {
        let mut subs = self.subscribers.lock().unwrap();
        subs.remove(event_type);
    }

    pub fn clear_all_subscribers(&self) {
        let mut subs = self.subscribers.lock().unwrap();
        subs.clear();
    }

    pub fn subscriber_count(&self, event_type: &str) -> usize {
        let subs = self.subscribers.lock().unwrap();
        subs.get(event_type).map(|v| v.len()).unwrap_or(0)
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for EventBus {
    fn clone(&self) -> Self {
        Self {
            subscribers: Arc::clone(&self.subscribers),
        }
    }
}

#[derive(Debug)]
pub struct SubscriptionHandle {
    event_type: String,
    callback_id: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use backtestr_data::models::Bar;
    use backtestr_data::timeframe::Timeframe;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_event_subscription() {
        let event_bus = EventBus::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);

        event_bus.subscribe("1M", move |_event| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        let bar = Bar::new(
            "EURUSD".to_string(),
            Timeframe::M1,
            1704067200000,
            1704067260000,
            1.0920,
            1.0925,
            1.0915,
            1.0922,
        );

        event_bus.publish(BarCompletionEvent::MinuteBar(bar));

        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_wildcard_subscription() {
        let event_bus = EventBus::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);

        event_bus.subscribe_all(move |_event| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        let bar = Bar::new(
            "EURUSD".to_string(),
            Timeframe::M5,
            1704067200000,
            1704067500000,
            1.0920,
            1.0925,
            1.0915,
            1.0922,
        );

        event_bus.publish(BarCompletionEvent::FiveMinuteBar(bar.clone()));
        event_bus.publish(BarCompletionEvent::FifteenMinuteBar(bar));

        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_multiple_subscribers() {
        let event_bus = EventBus::new();
        let counter1 = Arc::new(AtomicUsize::new(0));
        let counter2 = Arc::new(AtomicUsize::new(0));

        let counter1_clone = Arc::clone(&counter1);
        let counter2_clone = Arc::clone(&counter2);

        event_bus.subscribe("1M", move |_event| {
            counter1_clone.fetch_add(1, Ordering::SeqCst);
        });

        event_bus.subscribe("1M", move |_event| {
            counter2_clone.fetch_add(2, Ordering::SeqCst);
        });

        let bar = Bar::new(
            "EURUSD".to_string(),
            Timeframe::M1,
            1704067200000,
            1704067260000,
            1.0920,
            1.0925,
            1.0915,
            1.0922,
        );

        event_bus.publish(BarCompletionEvent::MinuteBar(bar));

        assert_eq!(counter1.load(Ordering::SeqCst), 1);
        assert_eq!(counter2.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_subscriber_count() {
        let event_bus = EventBus::new();

        event_bus.subscribe("1M", |_event| {});
        event_bus.subscribe("1M", |_event| {});
        event_bus.subscribe("5M", |_event| {});

        assert_eq!(event_bus.subscriber_count("1M"), 2);
        assert_eq!(event_bus.subscriber_count("5M"), 1);
        assert_eq!(event_bus.subscriber_count("1H"), 0);
    }
}