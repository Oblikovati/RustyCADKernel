//! Native NCollection_List iteration observes appends and unlinks of other nodes.
use super::{AlertEntry, AlertHandle, Arc, CompositeAlerts, Gravity};

pub(crate) struct AlertCursor {
    current: Option<Arc<AlertEntry>>,
}

impl AlertCursor {
    pub(crate) fn new(composite: &CompositeAlerts, gravity: Gravity) -> Self {
        let current = composite.lists()[gravity.index()].first().cloned();
        Self { current }
    }

    pub(crate) fn current_alert(&self) -> Option<AlertHandle> {
        self.current.as_ref().map(|entry| entry.alert.clone())
    }

    pub(crate) fn advance(&mut self) {
        // Weak links avoid recursively retaining a cleared hierarchy through one iterator.
        self.current = self.current.as_ref().and_then(|entry| entry.next_entry());
    }
}
