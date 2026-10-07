//! DRAW variables: `Draw::Set`, `Draw::Get`, `Draw::GetExisting`, `Draw::Drawables` and
//! the `tracevar` trace that ties a Tcl variable to a drawable.
//!
//! A drawable variable is a Tcl variable whose value is its own name, carrying a write
//! and unset trace whose client data is the drawable (here: its id in the session map).
//! Overwriting or unsetting the variable removes the drawable from the map and the
//! display, unless it is protected: then the write is refused ("variable is protected")
//! and the value is restored.
use super::drawable::DrawableHandle;
use super::number::DrawNumber;
use super::session::DrawSession;
use rustycad_tcl::{InterpHandle, TraceEvent, TraceTag, sys};
use std::rc::{Rc, Weak};

/// The trace class standing for the C++ `tracevar` procedure pointer.
const TRACE_CLASS: &str = "Draw::tracevar";
/// The events `tracevar` watches.
const TRACE_EVENTS: i32 = sys::TCL_TRACE_UNSETS | sys::TCL_TRACE_WRITES;

impl DrawSession {
    /// `Draw::Set(name, D)`: displayed when autodisplay is on.
    pub fn set_drawable_auto(&self, name: &str, drawable: Option<DrawableHandle>) {
        self.set_drawable(name, drawable, self.autodisplay.get());
    }

    /// `Draw::Set(name, D, Disp)`: stores `drawable` in variable `name` (`None` clears
    /// it) and displays or erases it. `.` designates the drawable itself: it is only
    /// re-displayed. A protected variable is left alone with a message.
    pub fn set_drawable(&self, name: &str, drawable: Option<DrawableHandle>, display: bool) {
        if name == "." {
            self.redisplay(drawable, display);
            return;
        }
        if let Some(old) = self.traced_drawable(name)
            && old.protected()
        {
            self.console().write_out("variable is protected\n");
            return;
        }
        let Some(interp) = self.interp_handle() else {
            return;
        };
        // Unsetting fires `tracevar`, which forgets the previous drawable.
        let _ = interp.unset_var(name, 0);
        let Some(drawable) = drawable else {
            return;
        };
        self.attach(&interp, name, &drawable);
        self.show(&drawable, display);
    }

    /// `Draw::Set(name, double)`: updates a numeric variable or creates one (never
    /// displayed).
    pub fn set_number(&self, name: &str, value: f64) {
        if let Some(number) = self
            .get_existing(name)
            .and_then(|d| super::drawable::downcast_handle::<DrawNumber>(&d))
        {
            number.set_value(value);
            return;
        }
        self.set_drawable(name, Some(Rc::new(DrawNumber::new(value))), false);
    }

    /// `Draw::Set(name, const char*)`: a plain Tcl variable.
    pub fn set_string(&self, name: &str, value: &str) {
        if let Some(interp) = self.interp_handle() {
            let _ = interp.set_var(name, value, 0);
        }
    }

    /// `Draw::Get(name)`: the drawable of variable `name`; `.` asks the user to pick a
    /// displayed object (in batch mode nothing can be picked).
    pub fn get(&self, name: &str) -> Option<DrawableHandle> {
        if name != "." {
            return self.traced_drawable(name);
        }
        self.pick_drawable()
    }

    /// `Draw::getDrawable(name, allowPick)`: [`Self::get`] when picking is allowed,
    /// [`Self::get_existing`] otherwise.
    pub fn get_drawable(&self, name: &str, allow_pick: bool) -> Option<DrawableHandle> {
        if allow_pick {
            self.get(name)
        } else {
            self.get_existing(name)
        }
    }

    /// `Draw::GetExisting(name)`: as [`Self::get`] without picking.
    pub fn get_existing(&self, name: &str) -> Option<DrawableHandle> {
        if name == "." {
            None
        } else {
            self.traced_drawable(name)
        }
    }

    /// `Draw::Get(name, double&)`: the value of a numeric variable.
    pub fn get_number(&self, name: &str) -> Option<f64> {
        let drawable = self.get_existing(name)?;
        drawable.downcast_ref::<DrawNumber>().map(DrawNumber::value)
    }

    /// `Draw::Drawables()`: the drawables held by variables, in creation order (the C++
    /// map iterates in pointer-hash order, which is not observable in batch mode).
    pub fn drawables(&self) -> Vec<DrawableHandle> {
        self.drawables.borrow().values().cloned().collect()
    }

    /// The interpreter of the session; `None` only if someone detached it (C++ would
    /// dereference a null interpreter).
    pub(crate) fn interp_handle(&self) -> Option<Rc<rustycad_tcl::Interp>> {
        self.interpretor.interp().ok()
    }

    /// `Tcl_VarTraceInfo(..., tracevar, NULL)` then the map lookup of `getDrawable`.
    fn traced_drawable(&self, name: &str) -> Option<DrawableHandle> {
        let key = self
            .interp_handle()?
            .var_trace_key(name, TRACE_EVENTS, TRACE_CLASS)
            .ok()??;
        self.drawables.borrow().get(&key).cloned()
    }

    /// Registers `drawable` under `name`: map entry, variable value, trace.
    fn attach(&self, interp: &InterpHandle, name: &str, drawable: &DrawableHandle) {
        let key = drawable.base().id();
        self.drawables.borrow_mut().insert(key, Rc::clone(drawable));
        let value = interp.set_var(name, name, 0).ok().flatten();
        drawable.set_name(value.as_deref());
        let tag = TraceTag {
            class: TRACE_CLASS,
            key,
        };
        let body = trace_body(self.self_reference.clone(), key);
        let _ = interp.trace_var(name, TRACE_EVENTS, tag, body);
    }

    /// Displays a new value (unless already visible) or erases it.
    fn show(&self, drawable: &DrawableHandle, display: bool) {
        let mut viewer = self.viewer();
        if display && !drawable.visible() {
            viewer.display(drawable);
        } else if !display && drawable.visible() {
            viewer.remove_drawable(drawable);
        }
    }

    /// The `.` case of `Draw::Set`.
    fn redisplay(&self, drawable: Option<DrawableHandle>, display: bool) {
        let Some(drawable) = drawable else {
            return;
        };
        let mut viewer = self.viewer();
        viewer.remove_drawable(&drawable);
        if display {
            viewer.display(&drawable);
        }
    }

    /// The pick of `getDrawable`: waits for a click, then picks within 5 pixels.
    fn pick_drawable(&self) -> Option<DrawableHandle> {
        self.console().write_out("Pick an object\n");
        let mut errors = String::new();
        let selection = self.viewer().select(true, &mut errors);
        self.console().write_err(&errors);
        let mut last = self.last_pick.borrow_mut();
        if let Some(selection) = selection {
            (last.view, last.x, last.y, last.button) =
                (selection.view, selection.x, selection.y, selection.button);
        }
        let (_, picked) = self.viewer().pick(last.view, last.x, last.y, 5, 0);
        if let Some(name) = picked.as_ref().and_then(|drawable| drawable.name()) {
            last.name = name;
        }
        picked
    }

    /// `tracevar`: the variable `name` holding drawable `key` is written or unset.
    fn on_variable_event(&self, name: &str, key: u64) -> Option<String> {
        // Protects against a map emptied before the interpreter is deleted.
        if self.drawables.borrow().is_empty() {
            return None;
        }
        let interp = self.interp_handle()?;
        let tag = TraceTag {
            class: TRACE_CLASS,
            key,
        };
        // The C++ code dereferences its client data even after the drawable left the
        // map; a drawable no longer in the map only loses its trace here.
        let drawable = self.drawables.borrow().get(&key).cloned();
        let Some(drawable) = drawable else {
            let _ = interp.untrace_var(name, 0, tag);
            return None;
        };
        if drawable.protected() {
            let value = interp.set_var(name, name, 0).ok().flatten();
            drawable.set_name(value.as_deref());
            return Some("variable is protected".to_string());
        }
        self.forget_drawable(&drawable);
        let _ = interp.untrace_var(name, 0, tag);
        self.drawables.borrow_mut().remove(&key);
        None
    }

    /// Erases a drawable whose variable is gone, scheduling the repaint of its views.
    fn forget_drawable(&self, drawable: &DrawableHandle) {
        if !drawable.visible() {
            return;
        }
        self.viewer().remove_drawable(drawable);
        if drawable.is_3d() {
            self.repaint_3d.set(true);
        } else {
            self.repaint_2d.set(true);
        }
    }
}

/// The trace body of one drawable variable.
fn trace_body(session: Weak<DrawSession>, key: u64) -> Box<dyn rustycad_tcl::VarTraceProc> {
    Box::new(
        move |_: &InterpHandle, event: &TraceEvent| -> Option<String> {
            session.upgrade()?.on_variable_event(&event.part1, key)
        },
    )
}

#[cfg(test)]
mod tests {
    use crate::draw::{DrawNumber, DrawSession, DrawViewer, DrawableHandle, RecordingConsole};
    use std::rc::Rc;

    fn session() -> (Rc<DrawSession>, Rc<RecordingConsole>) {
        let console = Rc::new(RecordingConsole::default());
        (
            DrawSession::new(console.clone(), DrawViewer::batch()),
            console,
        )
    }

    #[test]
    fn a_drawable_variable_holds_its_name_and_is_found_again() {
        let (session, _) = session();
        let number: DrawableHandle = Rc::new(DrawNumber::new(4.0));
        session.set_drawable("v", Some(number.clone()), false);
        assert!(Rc::ptr_eq(&session.get("v").unwrap(), &number));
        assert_eq!(number.name().as_deref(), Some("v"));
        assert!(session.get_drawable("v", false).is_some());
        assert!(session.get_drawable(".", false).is_none());
        assert_eq!(session.drawables().len(), 1);
    }

    #[test]
    fn overwriting_a_variable_forgets_its_drawable_unless_protected() {
        let (session, console) = session();
        session.set_number("p", 1.0);
        session.get("p").unwrap().set_protected(true);
        session.set_number("p", 2.0);
        assert_eq!(session.get_number("p"), Some(2.0));
        session.set_drawable("p", None, false);
        assert_eq!(console.take_output(), "variable is protected\n");
        session.get("p").unwrap().set_protected(false);
        session.set_string("p", "text");
        assert!(session.get("p").is_none());
        assert!(session.drawables().is_empty());
    }
}
