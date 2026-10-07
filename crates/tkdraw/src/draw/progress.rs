//! `Draw_ProgressIndicator`: progress output to the Tcl result, the console and a Tk
//! progress window.
use super::interpretor::DrawInterpretor;
use super::interpretor_link::InterpretorLink;
use rustycad_tkernel::message::{
    ProgressIndicator, ProgressIndicatorHooks, ProgressRange, ProgressScope,
};
use rustycad_tkernel::osd::Osd;
use rustycad_tkernel::precision;
use std::io::Write;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

static DEFAULT_TCL_MODE: AtomicBool = AtomicBool::new(false);
static DEFAULT_CONSOLE_MODE: AtomicBool = AtomicBool::new(false);
static DEFAULT_GRAPH_MODE: AtomicBool = AtomicBool::new(false);
/// `StopIndicator()`: the id of the indicator the user asked to stop (0 for none).
static STOP_INDICATOR: AtomicU64 = AtomicU64::new(0);

/// `DefaultTclMode()`: the Tcl mode of new indicators.
pub fn default_tcl_mode() -> bool {
    DEFAULT_TCL_MODE.load(Ordering::Relaxed)
}
/// `DefaultTclMode() = mode`.
pub fn set_default_tcl_mode(mode: bool) {
    DEFAULT_TCL_MODE.store(mode, Ordering::Relaxed);
}
/// `DefaultConsoleMode()`.
pub fn default_console_mode() -> bool {
    DEFAULT_CONSOLE_MODE.load(Ordering::Relaxed)
}
/// `DefaultConsoleMode() = mode`.
pub fn set_default_console_mode(mode: bool) {
    DEFAULT_CONSOLE_MODE.store(mode, Ordering::Relaxed);
}
/// `DefaultGraphMode()`.
pub fn default_graph_mode() -> bool {
    DEFAULT_GRAPH_MODE.load(Ordering::Relaxed)
}
/// `DefaultGraphMode() = mode`.
pub fn set_default_graph_mode(mode: bool) {
    DEFAULT_GRAPH_MODE.store(mode, Ordering::Relaxed);
}
/// `StopIndicator()`: the id of the indicator to stop (0 for none).
pub fn stop_indicator() -> u64 {
    STOP_INDICATOR.load(Ordering::Relaxed)
}
/// `StopIndicator() = id` (what the `XProgress -stop` command does).
pub fn set_stop_indicator(id: u64) {
    STOP_INDICATOR.store(id, Ordering::Relaxed);
}

/// The mutable members of the C++ class.
#[derive(Debug)]
struct ProgressState {
    tcl_mode: bool,
    console_mode: bool,
    graph_mode: bool,
    shown: bool,
    stopped: bool,
    update_threshold: f64,
    last_position: f64,
    start_time: u64,
}

/// What the indicator and its hooks share.
struct ProgressShared {
    id: u64,
    link: InterpretorLink,
    state: Mutex<ProgressState>,
}

impl ProgressShared {
    fn state(&self) -> MutexGuard<'_, ProgressState> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

/// `Draw_ProgressIndicator`.
///
/// ```
/// use rustycad_tkdraw::draw::{DrawInterpretor, DrawProgressIndicator, RecordingConsole};
/// use std::rc::Rc;
/// let di = DrawInterpretor::new(Rc::new(RecordingConsole::default()));
/// di.init();
/// let progress = DrawProgressIndicator::new(&di, 1.0);
/// progress.set_tcl_mode(true);
/// // Starting shows 0%; dropping the whole range completes it.
/// drop(progress.start());
/// assert_eq!(di.result().unwrap(), "Progress: 0%\nProgress: 100%\n");
/// ```
pub struct DrawProgressIndicator {
    indicator: ProgressIndicator,
    shared: Arc<ProgressShared>,
}

impl DrawProgressIndicator {
    /// `Draw_ProgressIndicator(di, theUpdateThreshold)`: modes from the defaults; a
    /// non-forced update is shown only after `threshold` percent of progress.
    pub fn new(di: &DrawInterpretor, threshold: f64) -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        let state = ProgressState {
            tcl_mode: default_tcl_mode(),
            console_mode: default_console_mode(),
            graph_mode: default_graph_mode(),
            shown: false,
            stopped: false,
            update_threshold: 0.01 * threshold,
            last_position: -1.0,
            start_time: 0,
        };
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let shared = Arc::new(ProgressShared {
            id,
            link: InterpretorLink::new(di),
            state: Mutex::new(state),
        });
        let indicator = ProgressIndicator::new(Box::new(DrawProgressHooks(Arc::clone(&shared))));
        Self { indicator, shared }
    }

    /// The id that `XProgress -stop` refers to (the C++ `this` pointer).
    pub fn id(&self) -> u64 {
        self.shared.id
    }

    /// `Start`: resets and returns the range covering the whole scale.
    pub fn start(&self) -> ProgressRange<'_> {
        self.indicator.start()
    }

    /// The base indicator.
    pub fn indicator(&self) -> &ProgressIndicator {
        &self.indicator
    }

    /// `SetTclMode`.
    pub fn set_tcl_mode(&self, mode: bool) {
        self.shared.state().tcl_mode = mode;
    }
    /// `GetTclMode`.
    pub fn tcl_mode(&self) -> bool {
        self.shared.state().tcl_mode
    }
    /// `SetConsoleMode`.
    pub fn set_console_mode(&self, mode: bool) {
        self.shared.state().console_mode = mode;
    }
    /// `GetConsoleMode`.
    pub fn console_mode(&self) -> bool {
        self.shared.state().console_mode
    }
    /// `SetGraphMode`.
    pub fn set_graph_mode(&self, mode: bool) {
        self.shared.state().graph_mode = mode;
    }
    /// `GetGraphMode`.
    pub fn graph_mode(&self) -> bool {
        self.shared.state().graph_mode
    }

    /// `Reset`: closes the progress window if shown and clears the break flag.
    pub fn reset(&self) {
        reset_shared(&self.shared);
    }

    /// `UserBreak`.
    pub fn user_break(&self) -> bool {
        user_break_shared(&self.shared)
    }
}

impl Drop for DrawProgressIndicator {
    /// `~Draw_ProgressIndicator` calls `Reset`.
    fn drop(&mut self) {
        reset_shared(&self.shared);
    }
}

/// The hooks the base indicator calls.
struct DrawProgressHooks(Arc<ProgressShared>);

impl ProgressIndicatorHooks for DrawProgressHooks {
    fn show(&self, scope: &ProgressScope<'_>, position: f64, force: bool) {
        show_progress(&self.0, scope, position, force);
    }

    fn user_break(&self) -> bool {
        user_break_shared(&self.0)
    }

    fn reset(&self) {
        reset_shared(&self.0);
    }
}

/// The C++ `Reset` after the base reset.
fn reset_shared(shared: &ProgressShared) {
    let was_shown = std::mem::replace(&mut shared.state().shown, false);
    if was_shown {
        // `Eval` resets the result, so it is saved and appended back.
        let _ = shared.link.with_interpretor(|di| {
            let saved = di.result().unwrap_or_default();
            let _ = di.eval("destroy .xprogress");
            let _ = di.append(saved);
        });
    }
    let mut state = shared.state();
    (state.stopped, state.last_position, state.start_time) = (false, -1.0, 0);
}

/// The C++ `UserBreak`: the stop request of `XProgress`, or Ctrl-Break.
fn user_break_shared(shared: &ProgressShared) -> bool {
    if stop_indicator() == shared.id {
        shared.state().stopped = true;
        let _ = shared
            .link
            .with_interpretor(|di| di.eval("XProgress -stop 0"));
    } else if Osd::control_break().is_err() {
        shared.state().stopped = true;
    }
    shared.state().stopped
}

/// Seconds since the epoch (`time(NULL)`).
fn now_seconds() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// The C++ `Show`.
fn show_progress(shared: &ProgressShared, scope: &ProgressScope<'_>, position: f64, force: bool) {
    let (graph, tcl, console) = {
        let mut state = shared.state();
        if !state.graph_mode && !state.tcl_mode && !state.console_mode {
            return;
        }
        if state.start_time == 0 {
            state.start_time = now_seconds();
        }
        let too_small = (position - state.last_position).abs() < state.update_threshold;
        if !force && (1.0 - position) > precision::confusion() && too_small {
            return;
        }
        state.last_position = position;
        (state.graph_mode, state.tcl_mode, state.console_mode)
    };
    let text = progress_text(scope, position);
    if graph && shared.link.is_owner_thread() {
        show_window(shared, scope, position, &text);
    }
    if tcl {
        shared.link.append(&format!("{text}\n"));
    }
    if console {
        // `std::cout`, reachable from any thread.
        let _ = writeln!(std::io::stdout(), "{text}");
    }
}

/// `Progress: N% name: value / max ...` for the named scopes from the root down.
fn progress_text(scope: &ProgressScope<'_>, position: f64) -> String {
    let mut text = format!("Progress: {:.0}%", 100.0 * position);
    let mut chain: Vec<&ProgressScope<'_>> = Vec::new();
    let mut current = Some(scope);
    while let Some(next) = current {
        chain.push(next);
        current = next.parent();
    }
    for scope in chain.iter().rev() {
        let Some(name) = scope.name() else {
            continue;
        };
        text.push_str(&format!(
            " {}: {}",
            String::from_utf8_lossy(name),
            scope_progress(scope)
        ));
    }
    text
}

/// The value of one scope: `finished`, the value of an infinite scope, or `value / max`.
fn scope_progress(scope: &ProgressScope<'_>) -> String {
    let value = scope.value();
    if !scope.is_infinite() {
        return format!("{value:.0} / {:.0}", scope.max_value());
    }
    if precision::is_infinite(value) {
        "finished".to_string()
    } else {
        format!("{value:.0}")
    }
}

/// The graphic progress bar (Tk), with the elapsed and estimated time.
fn show_window(shared: &ProgressShared, scope: &ProgressScope<'_>, position: f64, text: &str) {
    let start_time = shared.state().start_time;
    let mut label = text.to_string();
    if position > 0.01 {
        let elapsed = now_seconds() - start_time;
        label.push_str(&format!(
            "\nElapsed/estimated time: {}/{:.0} sec",
            elapsed as i64,
            elapsed as f64 / position
        ));
    }
    let first = !std::mem::replace(&mut shared.state().shown, true);
    let _ = shared.link.with_interpretor(|di| {
        let saved = di.result().unwrap_or_default();
        if first {
            let _ = di.eval(&create_window_script(shared.id));
        }
        let _ = di.eval(&update_window_script(position, scope.get_portion(), &label));
        let _ = di.append(saved);
    });
}

/// The Tk commands that create the `.xprogress` window.
fn create_window_script(id: u64) -> String {
    format!(
        "toplevel .xprogress -height 100 -width 410;wm title .xprogress \"Progress\";set xprogress_stop 0;\
         canvas .xprogress.bar -width 402 -height 22;\
         .xprogress.bar create rectangle 2 2 2 21 -fill blue -tags progress;\
         .xprogress.bar create rectangle 2 2 2 21 -outline black -tags progress_next;\
         message .xprogress.text -width 400 -text \"Progress 0%\";\
         button .xprogress.stop -text \"Break\" -relief groove -width 9 -command {{XProgress -stop {id:#x}}};\
         pack .xprogress.bar .xprogress.text .xprogress.stop -side top;"
    )
}

/// The Tk commands that move the bar and update the text.
fn update_window_script(position: f64, portion: f64, label: &str) -> String {
    format!(
        ".xprogress.bar coords progress 2 2 {:.0} 21;.xprogress.bar coords progress_next 2 2 {:.0} 21;\
         .xprogress.text configure -text \"{label}\";update",
        1.0 + 400.0 * position,
        1.0 + 400.0 * portion
    )
}

#[cfg(test)]
mod tests;
