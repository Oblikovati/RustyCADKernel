//! The state that DRAW keeps in C++ globals (`theCommands`, `dout`, `Draw_Batch`,
//! `Draw_Spying`, `Draw_Chrono`, the variable map, the parse flag, the last pick, the
//! progress indicator, the restore factories, the command-group guards), gathered in one
//! [`DrawSession`] that is passed to every command.
//!
//! Commands hold a weak reference to the session (the interpreter owns the commands, so
//! a strong one would be a cycle). The `Draw` static functions are methods here.
use super::console::DrawConsole;
use super::drawable::DrawableHandle;
use super::expression::{ExpressionContext, evaluate};
use super::interpretor::{DrawInterpretor, IntegerParser};
use super::istream::DrawIStream;
use super::plugins::{DrawPluginFactory, DrawPluginLibraries};
use super::progress::DrawProgressIndicator;
use super::viewer::DrawViewer;
use rustycad_tkernel::osd::{NativeCpuClock, PerfRegistry};
use rustycad_tkernel::resource::ResourceHost;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::units::Units;
use rustycad_tkernel::units_api::UnitsApi;
use std::any::{Any, TypeId};
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::File;
use std::rc::{Rc, Weak};
use std::sync::Arc;

/// A restore factory (`Draw_Drawable3D::FactoryFunction_t`). The session stands for
/// the package globals a factory reads (`DrawTrSurf::Parameters()`, ...).
pub type DrawableFactory = fn(&DrawSession, &mut DrawIStream) -> Option<DrawableHandle>;

/// A command body with access to the session: `int f(Draw_Interpretor&, int, const char**)`
/// plus the globals it reads.
pub type SessionCommand = fn(&DrawSession, &DrawInterpretor, &[&str]) -> Result<i32, Failure>;

/// The last interactive pick (`p_id`, `p_X`, `p_Y`, `p_b`, `p_Name`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LastPick {
    /// The view.
    pub view: i32,
    /// Pointer x in view coordinates.
    pub x: i32,
    /// Pointer y in view coordinates.
    pub y: i32,
    /// The button.
    pub button: i32,
    /// The name of the picked variable.
    pub name: String,
}

/// The DRAW session: interpreter, viewer, variables and global flags.
pub struct DrawSession {
    pub(crate) interpretor: DrawInterpretor,
    pub(crate) viewer: RefCell<DrawViewer>,
    pub(crate) drawables: RefCell<BTreeMap<u64, DrawableHandle>>,
    pub(crate) autodisplay: Cell<bool>,
    pub(crate) repaint_2d: Cell<bool>,
    pub(crate) repaint_3d: Cell<bool>,
    pub(crate) parse_failed: Cell<bool>,
    pub(crate) spy_file: RefCell<Option<File>>,
    pub(crate) chrono: Cell<bool>,
    /// `Draw_BlackBackGround`, toggled by `color` without arguments.
    pub(crate) black_background: Cell<bool>,
    /// The PostScript font name and size of `hardcopy` (`dfont`).
    pub(crate) font: RefCell<(String, String)>,
    /// `Draw_BeforeCommand`/`Draw_AfterCommand` are set (by the variable commands).
    pub(crate) command_hooks: Cell<bool>,
    pub(crate) virtual_windows: Cell<bool>,
    pub(crate) last_pick: RefCell<LastPick>,
    pub(crate) progress: RefCell<Option<Rc<DrawProgressIndicator>>>,
    pub(crate) factories: RefCell<HashMap<String, DrawableFactory>>,
    pub(crate) plugin_libraries: RefCell<DrawPluginLibraries>,
    pub(crate) loaded_plugins: RefCell<HashMap<String, DrawPluginFactory>>,
    pub(crate) units_api: RefCell<UnitsApi>,
    perf_registry: PerfRegistry,
    pub(crate) loaded_groups: RefCell<HashSet<&'static str>>,
    pub(crate) self_reference: Weak<DrawSession>,
    /// The static state of the packages above Draw (`DrawTrSurf::Parameters()`, ...),
    /// one value per type.
    package_states: RefCell<HashMap<TypeId, Rc<dyn Any>>>,
    /// The `Draw::Atoi` closure the interpreter uses for `Draw_ExitOnCatch`; owned here
    /// because the interpreter only keeps a weak reference.
    _integer_parser: Rc<IntegerParser>,
}

impl DrawSession {
    /// A session with a fresh Tcl interpreter (`theCommands.Init()`) and `viewer`
    /// (batch or windowed). Tcl's own `init.tcl` is not run here (`Tcl_Init` is part of
    /// the application start, see `appli`).
    ///
    /// ```
    /// use rustycad_tkdraw::draw::{DrawSession, DrawViewer, RecordingConsole};
    /// use std::rc::Rc;
    /// let session = DrawSession::new(Rc::new(RecordingConsole::default()), DrawViewer::batch());
    /// session.set_number("a", 2.5);
    /// assert_eq!(session.get_number("a"), Some(2.5));
    /// ```
    pub fn new(console: Rc<dyn DrawConsole>, viewer: DrawViewer) -> Rc<Self> {
        let interpretor = DrawInterpretor::new(console);
        interpretor.init();
        Rc::new_cyclic(|weak: &Weak<DrawSession>| {
            let parser_session = weak.clone();
            let integer_parser: Rc<IntegerParser> = Rc::new(move |text: &str| {
                parser_session
                    .upgrade()
                    .map_or(0, |session| session.atoi(text))
            });
            interpretor.set_integer_parser(Rc::downgrade(&integer_parser));
            Self {
                interpretor,
                viewer: RefCell::new(viewer),
                drawables: RefCell::new(BTreeMap::new()),
                autodisplay: Cell::new(true),
                repaint_2d: Cell::new(false),
                repaint_3d: Cell::new(false),
                parse_failed: Cell::new(true),
                spy_file: RefCell::new(None),
                chrono: Cell::new(false),
                black_background: Cell::new(true),
                font: RefCell::new(("Helvetica".to_string(), "150".to_string())),
                command_hooks: Cell::new(false),
                virtual_windows: Cell::new(false),
                last_pick: RefCell::new(LastPick::default()),
                progress: RefCell::new(None),
                factories: RefCell::new(HashMap::new()),
                plugin_libraries: RefCell::new(DrawPluginLibraries::default()),
                loaded_plugins: RefCell::new(HashMap::new()),
                units_api: RefCell::new(UnitsApi::new(
                    Arc::new(Units::new()),
                    ResourceHost::native(),
                )),
                perf_registry: PerfRegistry::new(Rc::new(NativeCpuClock)),
                loaded_groups: RefCell::new(HashSet::new()),
                self_reference: weak.clone(),
                package_states: RefCell::new(HashMap::new()),
                _integer_parser: integer_parser,
            }
        })
    }

    /// `Draw::GetInterpretor`.
    pub fn interpretor(&self) -> &DrawInterpretor {
        &self.interpretor
    }

    /// `dout`, borrowed for modification (never hold it across a Tcl evaluation).
    pub fn viewer(&self) -> std::cell::RefMut<'_, DrawViewer> {
        self.viewer.borrow_mut()
    }

    /// `Draw_Batch`.
    pub fn is_batch(&self) -> bool {
        self.viewer.borrow().is_batch()
    }

    /// The console (`std::cout`).
    pub fn console(&self) -> &Rc<dyn DrawConsole> {
        self.interpretor.console()
    }

    /// `Draw_Chrono`: time every command.
    pub fn chrono(&self) -> bool {
        self.chrono.get()
    }

    /// Sets `Draw_Chrono`.
    pub fn set_chrono(&self, enabled: bool) {
        self.chrono.set(enabled);
    }

    /// `Draw_VirtualWindows`: views use off-screen windows.
    pub fn virtual_windows(&self) -> bool {
        self.virtual_windows.get()
    }

    /// Sets `Draw_VirtualWindows`.
    pub fn set_virtual_windows(&self, enabled: bool) {
        self.virtual_windows.set(enabled);
    }

    /// Registers a command group once per interpreter (the `static bool Done` guards):
    /// true the first time `group` is seen.
    pub(crate) fn first_load(&self, group: &'static str) -> bool {
        self.loaded_groups.borrow_mut().insert(group)
    }

    /// `Add(name, help, __FILE__, function, group)` for a session command.
    pub fn add_command(
        &self,
        name: &str,
        help: &str,
        file: &str,
        command: SessionCommand,
        group: &str,
    ) -> Result<(), Failure> {
        let session = self.self_reference.clone();
        let body = move |di: &DrawInterpretor, args: &[&str]| -> Result<i32, Failure> {
            let session = session.upgrade().ok_or_else(|| {
                Failure::with_message("Standard_ProgramError", "the DRAW session was dropped")
            })?;
            command(&session, di, args)
        };
        self.interpretor
            .add_with_file(name, help, file, Box::new(body), group)
    }

    /// `Draw_Drawable3D::RegisterFactory`.
    pub fn register_factory(&self, type_name: &str, factory: DrawableFactory) {
        self.factories
            .borrow_mut()
            .insert(type_name.to_string(), factory);
    }

    /// `Draw_Drawable3D::Restore`: the drawable read by the factory of `type_name`, or
    /// `None` when no factory is registered.
    pub fn restore(&self, type_name: &str, stream: &mut DrawIStream) -> Option<DrawableHandle> {
        let factory = self.factories.borrow().get(type_name).copied()?;
        factory(self, stream)
    }

    /// The function-local statics of a package above Draw (such as
    /// `DrawTrSurf::Parameters()`): one `T` per session, made on first use.
    ///
    /// ```
    /// use rustycad_tkdraw::draw::{DrawSession, DrawViewer, RecordingConsole};
    /// use std::cell::Cell;
    /// use std::rc::Rc;
    /// let session = DrawSession::new(Rc::new(RecordingConsole::default()), DrawViewer::batch());
    /// session.package_state::<Cell<i32>>().set(3);
    /// assert_eq!(session.package_state::<Cell<i32>>().get(), 3);
    /// ```
    pub fn package_state<T: Default + 'static>(&self) -> Rc<T> {
        let mut states = self.package_states.borrow_mut();
        let state = states
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Rc::new(T::default()));
        // The map key is the TypeId of the value, so the downcast cannot fail.
        Rc::clone(state)
            .downcast::<T>()
            .unwrap_or_else(|_| Rc::new(T::default()))
    }

    /// `Draw::Atof`: the value of a numeric expression (see [`evaluate`]).
    pub fn atof(&self, text: &str) -> f64 {
        evaluate(text, self)
    }

    /// `Draw::Atoi`: `Atof` truncated toward zero (saturating where C++ is undefined).
    pub fn atoi(&self, text: &str) -> i32 {
        self.atof(text) as i32
    }

    /// `Draw::ParseReal`: the value, or `None` when the expression did not parse.
    pub fn parse_real(&self, text: &str) -> Option<f64> {
        let value = self.atof(text);
        if self.parse_failed.get() {
            self.parse_failed.set(false);
            return None;
        }
        Some(value)
    }

    /// `Draw::ParseInteger`: the value when it parses and is a whole `int`.
    pub fn parse_integer(&self, text: &str) -> Option<i32> {
        let value = self.parse_real(text)?;
        let integer = value as i32;
        (integer as f64 == value).then_some(integer)
    }

    /// The performance meters `dperf` reports (the C++ uses the process-wide registry
    /// of `OSD_PerfMeter`; the Rust meters live in an explicit registry).
    pub fn perf_registry(&self) -> &PerfRegistry {
        &self.perf_registry
    }

    /// `Draw::SetProgressBar`.
    pub fn set_progress_bar(&self, progress: Option<Rc<DrawProgressIndicator>>) {
        *self.progress.borrow_mut() = progress;
    }

    /// `Draw::GetProgressBar`.
    pub fn progress_bar(&self) -> Option<Rc<DrawProgressIndicator>> {
        self.progress.borrow().clone()
    }

    /// `Draw::LastPick`.
    pub fn last_pick(&self) -> LastPick {
        self.last_pick.borrow().clone()
    }

    /// `Draw::Repaint`: repaint both kinds of views after the current command.
    pub fn repaint(&self) {
        self.repaint_2d.set(true);
        self.repaint_3d.set(true);
    }

    /// `Draw_RepaintNowIfNecessary`, also the `after` hook of every command.
    pub fn repaint_now_if_necessary(&self) -> Result<(), Failure> {
        if self.repaint_2d.replace(false) {
            self.viewer().repaint_2d()?;
        }
        if self.repaint_3d.replace(false) {
            self.viewer().repaint_3d()?;
        }
        Ok(())
    }

    /// The `before` hook of every command: forget pending repaints.
    pub fn before_command(&self) {
        self.repaint_2d.set(false);
        self.repaint_3d.set(false);
    }
}

impl ExpressionContext for DrawSession {
    fn number_variable(&self, name: &str) -> Option<f64> {
        self.get_number(name)
    }

    fn call_function(&self, script: &str) -> f64 {
        let di = &self.interpretor;
        let saved = di.result().unwrap_or_default();
        if !saved.is_empty() {
            let _ = di.reset();
        }
        let value = if di.eval(script).unwrap_or(1) != 0 {
            self.print_line(&format!("Call of function {script} failed"));
            0.0
        } else {
            rustycad_tkernel::standard::strtod(&di.result().unwrap_or_default()).0
        };
        let _ = di.reset();
        if !saved.is_empty() {
            let _ = di.append(saved);
        }
        value
    }

    fn print_line(&self, text: &str) {
        self.console().write_out(&format!("{text}\n"));
    }

    fn set_parse_failed(&self, failed: bool) {
        self.parse_failed.set(failed);
    }
}
