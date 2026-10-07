// Rust port of DataExchange/TKXSBase/IFSelect/IFSelect_Act (.hxx, .cxx) and
// IFSelect_ActFunc.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IFSelect_Act`: an activator of one command executed by a function, and the static
//! group under which the functions are added.
use super::{IFSelectActivator, IFSelectActivatorBase, IFSelectReturnStatus, IFSelectSessionPilot};
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, Handle, StandardTransient},
    tcollection::AsciiString,
};
use std::sync::{OnceLock, PoisonError, RwLock};

/// `IFSelect_ActFunc`: the function of a command, on the session pilot.
pub type IFSelectActFunc =
    fn(&Handle<IFSelectSessionPilot>) -> Result<IFSelectReturnStatus, Failure>;

/// The native statics `thedefgr` and `thedefil`: the group and file of the functions
/// added next.
fn default_group() -> &'static RwLock<(AsciiString, AsciiString)> {
    static GROUP: OnceLock<RwLock<(AsciiString, AsciiString)>> = OnceLock::new();
    GROUP.get_or_init(|| RwLock::new((AsciiString::default(), AsciiString::default())))
}

/// `IFSelect_Act`.
///
/// ```
/// use rustycad_tkxsbase::if_select::{IFSelectAct, IFSelectActivator};
/// let act = IFSelectAct::new("hello", "says hello", None);
/// assert_eq!(act.help(1).as_bytes(), b"says hello");
/// ```
pub struct IFSelectAct {
    transient: StandardTransient,
    base: IFSelectActivatorBase,
    thename: AsciiString,
    thehelp: AsciiString,
    thefunc: Option<IFSelectActFunc>,
}
impl_transient!(IFSelectAct, "IFSelect_Act", dyn IFSelectActivator, transient, interfaces: [dyn IFSelectActivator]);

impl IFSelectAct {
    /// `IFSelect_Act(name, help, func)`: built as its handle, bound for `Add`.
    pub fn new(name: &str, help: &str, func: Option<IFSelectActFunc>) -> Handle<Self> {
        let act = Handle::new(Self {
            transient: StandardTransient::default(),
            base: IFSelectActivatorBase::new(),
            thename: AsciiString::new(name),
            thehelp: AsciiString::new(help),
            thefunc: func,
        });
        act.base.bind_self(&act.clone().cast());
        act
    }
    /// The command name.
    pub fn name(&self) -> &AsciiString {
        &self.thename
    }
    /// `SetGroup(group, file = "")`: the group and file of the functions added next.
    pub fn set_group(group: &str, file: &str) {
        *default_group()
            .write()
            .unwrap_or_else(PoisonError::into_inner) =
            (AsciiString::new(group), AsciiString::new(file));
    }
    /// The group and file set by `SetGroup`.
    fn defaults() -> (AsciiString, AsciiString) {
        default_group()
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
    /// The text of an `AsciiString`.
    fn text(s: &AsciiString) -> String {
        String::from_utf8_lossy(s.as_bytes()).into_owned()
    }
    /// `AddFunc(name, help, func)`: a new command in the current group. Native quirk
    /// kept: the file of the group is not given to the command.
    pub fn add_func(name: &str, help: &str, func: IFSelectActFunc) -> Result<(), Failure> {
        let act = Self::new(name, help, Some(func));
        let (group, _) = Self::defaults();
        if !group.is_empty() {
            act.set_for_group(&Self::text(&group), "");
        }
        act.add(1, name)
    }
    /// `AddFSet(name, help, func)`: a new command creating an item (for `xsnew`), in the
    /// current group and file.
    pub fn add_f_set(name: &str, help: &str, func: IFSelectActFunc) -> Result<(), Failure> {
        let act = Self::new(name, help, Some(func));
        let (group, file) = Self::defaults();
        if !group.is_empty() {
            act.set_for_group(&Self::text(&group), &Self::text(&file));
        }
        act.add_set(1, name)
    }
}

impl IFSelectActivator for IFSelectAct {
    fn activator_base(&self) -> &IFSelectActivatorBase {
        &self.base
    }
    /// `Do(number, pilot)`: the function on the pilot (void without function).
    fn do_command(
        &self,
        _number: i32,
        pilot: &Handle<IFSelectSessionPilot>,
    ) -> Result<IFSelectReturnStatus, Failure> {
        match self.thefunc {
            Some(func) => func(pilot),
            None => Ok(IFSelectReturnStatus::RetVoid),
        }
    }
    /// `Help(number)`: the help given at the construction.
    fn help(&self, _number: i32) -> AsciiString {
        self.thehelp.clone()
    }
}
