//! Engine output sink trait.
//!
//! The `EngineSink` trait abstracts how the engine delivers events to clients.
//! Implementations decide how to render or transport events:
//! - `CliSink` (in koda-cli): renders to terminal
//! - Future `AcpSink`: serializes over WebSocket
//! - `TestSink`: collects events for assertions

use super::event::{ApprovalDecision, EngineEvent};

/// Trait for consuming engine events.
///
/// Implementors decide how to render or transport events:
/// - `CliSink`: renders to terminal via `display::` and `markdown::`
/// - Future `AcpSink`: serializes over WebSocket
/// - `TestSink`: collects events for assertions
pub trait EngineSink: Send + Sync {
    /// Emit an engine event to the client.
    fn emit(&self, event: EngineEvent);

    /// Request approval from the user for a tool action.
    ///
    /// This is a blocking request/response: the engine pauses until the
    /// client decides. In CLI mode, this shows an interactive select widget.
    /// In server mode, this sends a WebSocket message and awaits the response.
    fn request_approval(
        &self,
        tool_name: &str,
        detail: &str,
        preview: Option<&str>,
        whitelist_hint: Option<&str>,
    ) -> ApprovalDecision;
}

/// A sink that collects events into a Vec for testing.
#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct TestSink {
    events: std::sync::Mutex<Vec<EngineEvent>>,
}

#[allow(dead_code)]
impl TestSink {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get all collected events.
    pub fn events(&self) -> Vec<EngineEvent> {
        self.events.lock().unwrap().clone()
    }

    /// Get the count of collected events.
    pub fn len(&self) -> usize {
        self.events.lock().unwrap().len()
    }

    /// Check if no events were collected.
    pub fn is_empty(&self) -> bool {
        self.events.lock().unwrap().is_empty()
    }
}

impl EngineSink for TestSink {
    fn emit(&self, event: EngineEvent) {
        self.events.lock().unwrap().push(event);
    }

    fn request_approval(
        &self,
        _tool_name: &str,
        _detail: &str,
        _preview: Option<&str>,
        _whitelist_hint: Option<&str>,
    ) -> ApprovalDecision {
        ApprovalDecision::Approve
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sink_collects_events() {
        let sink = TestSink::new();
        assert!(sink.is_empty());

        sink.emit(EngineEvent::ResponseStart);
        sink.emit(EngineEvent::TextDelta {
            text: "hello".into(),
        });
        sink.emit(EngineEvent::TextDone);

        assert_eq!(sink.len(), 3);
        let events = sink.events();
        assert!(matches!(events[0], EngineEvent::ResponseStart));
        assert!(matches!(&events[1], EngineEvent::TextDelta { text } if text == "hello"));
        assert!(matches!(events[2], EngineEvent::TextDone));
    }

    #[test]
    fn test_sink_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<TestSink>();
    }

    #[test]
    fn test_trait_object_works() {
        let sink: Box<dyn EngineSink> = Box::new(TestSink::new());
        sink.emit(EngineEvent::Info {
            message: "test".into(),
        });
    }
}
