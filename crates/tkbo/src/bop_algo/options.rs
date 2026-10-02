// Port of BOPAlgo_Options.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! Shared options and alert reporting for Boolean algorithms.
use super::AlertUserBreak;
use rustycad_tkernel::message::{AlertHandle, Gravity, MsgFile, ProgressScope, Report};
use rustycad_tkernel::ncollection::BaseAllocator;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::RuntimeType;
use rustycad_tkernel::tcollection::AsciiString;
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Once};

// OCCT's process default affects newly constructed objects only.
static GLOBAL_RUN_PARALLEL: AtomicBool = AtomicBool::new(false);

/// `BOPAlgo_Options`: configuration survives `clear`, while reports are shared on clone.
///
/// ```
/// use rustycad_tkbo::bop_algo::Options;
/// let mut options = Options::new();
/// options.set_fuzzy_value(0.01);
/// options.clear();
/// assert_eq!(options.fuzzy_value(), 0.01);
/// ```
#[derive(Clone)]
pub struct Options {
    allocator: Option<BaseAllocator>,
    report: Arc<Report>,
    run_parallel: bool,
    fuzzy_value: f64,
    use_obb: bool,
}

fn load_boolean_messages() {
    static LOAD: Once = Once::new();
    LOAD.call_once(|| {
        let catalog = MsgFile::global();
        if !catalog.has_msg(&AsciiString::new("BOPAlgo_LOAD_CHECKER")) {
            catalog
                .load_from_string(include_bytes!("messages.msg"), -1)
                .expect("pinned BOPAlgo messages.msg must contain a valid message catalog");
        }
    });
}

impl Default for Options {
    fn default() -> Self {
        Self::new()
    }
}

impl Options {
    /// Uses the common allocator and current parallel default; `Options::new()`.
    pub fn new() -> Self {
        Self::with_allocator(Some(BaseAllocator::common()))
    }

    /// Preserves the supplied allocator, including a null handle; `Options::with_allocator(None)`.
    pub fn with_allocator(allocator: Option<BaseAllocator>) -> Self {
        load_boolean_messages();
        Self {
            allocator,
            report: Arc::new(Report::new()),
            run_parallel: Self::get_parallel_mode(),
            fuzzy_value: precision::confusion(),
            use_obb: false,
        }
    }

    /// Returns the allocator identity; `Options::new().allocator().is_some()`.
    pub fn allocator(&self) -> Option<&BaseAllocator> {
        self.allocator.as_ref()
    }

    /// Clears every severity without resetting configuration; `options.clear()`.
    pub fn clear(&self) {
        self.report.clear();
    }

    /// Records a failure alert; `options.add_error(&alert)`.
    pub fn add_error(&self, alert: &AlertHandle) {
        self.report.add_alert(Gravity::Fail, alert);
    }

    /// Records a warning alert; `options.add_warning(&alert)`.
    pub fn add_warning(&self, alert: &AlertHandle) {
        self.report.add_alert(Gravity::Warning, alert);
    }

    /// Tests whether failure alerts exist; `assert!(!Options::new().has_errors())`.
    pub fn has_errors(&self) -> bool {
        !self.report.get_alerts(Gravity::Fail).is_empty()
    }

    /// Tests the runtime type among failures; `options.has_error(alert.dynamic_type())`.
    pub fn has_error(&self, kind: &RuntimeType) -> bool {
        self.report.has_alert_with_gravity(kind, Gravity::Fail)
    }

    /// Tests whether warning alerts exist; `assert!(!Options::new().has_warnings())`.
    pub fn has_warnings(&self) -> bool {
        !self.report.get_alerts(Gravity::Warning).is_empty()
    }

    /// Tests the runtime type among warnings; `options.has_warning(alert.dynamic_type())`.
    pub fn has_warning(&self, kind: &RuntimeType) -> bool {
        self.report.has_alert_with_gravity(kind, Gravity::Warning)
    }

    /// Shares access to the native report identity; `options.report().clear()`.
    pub fn report(&self) -> &Arc<Report> {
        &self.report
    }

    /// Writes failure message keys; `options.dump_errors(&mut Vec::new())?`.
    pub fn dump_errors(&self, output: &mut dyn Write) -> io::Result<()> {
        self.report.dump_gravity(output, Gravity::Fail)
    }

    /// Writes warning message keys; `options.dump_warnings(&mut Vec::new())?`.
    pub fn dump_warnings(&self, output: &mut dyn Write) -> io::Result<()> {
        self.report.dump_gravity(output, Gravity::Warning)
    }

    /// Removes warning alerts only; `options.clear_warnings()`.
    pub fn clear_warnings(&self) {
        self.report.clear_gravity(Gravity::Warning);
    }

    /// Reads the process default for new options; `Options::get_parallel_mode()`.
    pub fn get_parallel_mode() -> bool {
        GLOBAL_RUN_PARALLEL.load(Ordering::SeqCst)
    }

    /// Changes the process default for subsequent constructors; `Options::set_parallel_mode(true)`.
    pub fn set_parallel_mode(enabled: bool) {
        GLOBAL_RUN_PARALLEL.store(enabled, Ordering::SeqCst);
    }

    /// Overrides parallel execution for this instance; `options.set_run_parallel(true)`.
    pub fn set_run_parallel(&mut self, enabled: bool) {
        self.run_parallel = enabled;
    }

    /// Reads this instance's parallel mode; `Options::new().run_parallel()`.
    pub fn run_parallel(&self) -> bool {
        self.run_parallel
    }

    /// Applies OCCT's lower bound, preserving NaN as `std::max` does; `options.set_fuzzy_value(0.0)`.
    pub fn set_fuzzy_value(&mut self, tolerance: f64) {
        self.fuzzy_value = if tolerance < precision::confusion() {
            precision::confusion()
        } else {
            tolerance
        };
    }

    /// Returns the additional tolerance; `Options::new().fuzzy_value()`.
    pub fn fuzzy_value(&self) -> f64 {
        self.fuzzy_value
    }

    /// Enables oriented bounding boxes; `options.set_use_obb(true)`.
    pub fn set_use_obb(&mut self, enabled: bool) {
        self.use_obb = enabled;
    }

    /// Reads the oriented bounding box flag; `assert!(!Options::new().use_obb())`.
    pub fn use_obb(&self) -> bool {
        self.use_obb
    }

    /// Records the cancellation alert when a scope requests interruption.
    /// Example: `assert!(!Options::new().user_break(&ProgressScope::default()))`.
    pub fn user_break(&self, scope: &ProgressScope<'_>) -> bool {
        if !scope.user_break() {
            return false;
        }
        self.add_error(&(Arc::new(AlertUserBreak) as AlertHandle));
        true
    }
}
