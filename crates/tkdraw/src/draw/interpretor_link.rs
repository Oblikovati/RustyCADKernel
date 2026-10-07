//! A `Send + Sync` reference to an interpreter, for the message printer and the
//! progress indicator, whose interfaces (`Message_Printer`, `Message_ProgressIndicator`)
//! may be called from any thread.
//!
//! A Tcl interpreter belongs to the thread that created it. The C++ `Draw_Printer` and
//! `Draw_ProgressIndicator` keep a raw `Draw_Interpretor*` and write to it from whatever
//! thread calls them, a data race when that is a worker thread. Here the link resolves
//! the interpreter through a thread-local registry, so it is reachable only on its own
//! thread; text sent from another thread is queued and appended, in order, the next time
//! the link is used on the interpreter's thread (or when it is dropped there).
use super::interpretor::{DrawInterpretor, InterpretorCore};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Weak;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread::ThreadId;

thread_local! {
    /// The interpreters of this thread that links refer to, by link key.
    static INTERPRETORS: RefCell<HashMap<u64, Weak<InterpretorCore>>> = RefCell::new(HashMap::new());
}

/// A thread-checked handle on one interpreter.
#[derive(Debug)]
pub struct InterpretorLink {
    key: u64,
    owner: ThreadId,
    pending: Mutex<Vec<String>>,
}

impl InterpretorLink {
    /// Links `di` (on its own thread).
    ///
    /// ```
    /// use rustycad_tkdraw::draw::{DrawInterpretor, InterpretorLink, RecordingConsole};
    /// use std::rc::Rc;
    /// let di = DrawInterpretor::new(Rc::new(RecordingConsole::default()));
    /// di.init();
    /// let link = InterpretorLink::new(&di);
    /// link.append("from the link");
    /// assert_eq!(di.result().unwrap(), "from the link");
    /// ```
    pub fn new(di: &DrawInterpretor) -> Self {
        static NEXT_KEY: AtomicU64 = AtomicU64::new(1);
        let key = NEXT_KEY.fetch_add(1, Ordering::Relaxed);
        INTERPRETORS.with(|registry| registry.borrow_mut().insert(key, di.downgrade()));
        Self {
            key,
            owner: std::thread::current().id(),
            pending: Mutex::new(Vec::new()),
        }
    }

    /// Runs `action` with the interpreter when called on its thread and the interpreter
    /// still exists; text queued from other threads is appended first.
    pub fn with_interpretor<R>(&self, action: impl FnOnce(&DrawInterpretor) -> R) -> Option<R> {
        if std::thread::current().id() != self.owner {
            return None;
        }
        let core = INTERPRETORS
            .with(|registry| registry.borrow().get(&self.key).and_then(Weak::upgrade))?;
        let di = DrawInterpretor::from_core(core);
        self.flush_pending(&di);
        Some(action(&di))
    }

    /// `*myDraw << text`: appends to the interpreter result, or queues the text when
    /// called from another thread.
    pub fn append(&self, text: &str) {
        let appended = self.with_interpretor(|di| {
            let _ = di.append(text);
        });
        if appended.is_none() && std::thread::current().id() != self.owner {
            self.lock_pending().push(text.to_string());
        }
    }

    /// True on the interpreter's thread (`myGuiThreadId == OSD_Thread::Current()`).
    pub fn is_owner_thread(&self) -> bool {
        std::thread::current().id() == self.owner
    }

    fn lock_pending(&self) -> std::sync::MutexGuard<'_, Vec<String>> {
        self.pending
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn flush_pending(&self, di: &DrawInterpretor) {
        let pending = std::mem::take(&mut *self.lock_pending());
        for text in pending {
            let _ = di.append(&text);
        }
    }
}

impl Drop for InterpretorLink {
    fn drop(&mut self) {
        // Appends what other threads queued; on a foreign thread the registry entry
        // (a weak reference) stays in the owner's thread-local map until that thread ends.
        let _ = self.with_interpretor(|_| ());
        let _ = INTERPRETORS.try_with(|registry| registry.borrow_mut().remove(&self.key));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::draw::RecordingConsole;
    use std::rc::Rc;
    use std::sync::Arc;

    #[test]
    fn text_from_another_thread_waits_for_the_owner() {
        let di = DrawInterpretor::new(Rc::new(RecordingConsole::default()));
        di.init();
        let link = Arc::new(InterpretorLink::new(&di));
        let remote = Arc::clone(&link);
        std::thread::spawn(move || remote.append("remote "))
            .join()
            .unwrap();
        assert_eq!(di.result().unwrap(), "");
        link.append("local");
        assert_eq!(di.result().unwrap(), "remote local");
    }

    #[test]
    fn a_dropped_interpreter_is_not_reached() {
        let link = {
            let di = DrawInterpretor::new(Rc::new(RecordingConsole::default()));
            di.init();
            InterpretorLink::new(&di)
        };
        assert!(link.with_interpretor(|_| ()).is_none());
    }
}
