// Rust port of DataExchange/TKXSBase/IFSelect/IFSelect_Activator (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IFSelect_Activator`: the commands of the session pilot. An activator registers
//! command names (with a number and a mode) in a process-wide dictionary, then
//! executes them by number.
use super::{IFSelectReturnStatus, IFSelectSessionPilot};
use rustycad_tkernel::{
    impl_transient_interface,
    ncollection::DataMap,
    standard::{
        Failure, Handle, SharedState, StandardTransient, Transient, WeakHandle, exceptions,
    },
    tcollection::AsciiString,
};
use std::sync::{OnceLock, PoisonError, RwLock, RwLockWriteGuard};

/// The native statics: the dictionary of commands (to their 1-based rank), and by rank
/// the number, the activator and the mode of each command.
#[derive(Default)]
struct ActivatorRegistry {
    thedico: DataMap<AsciiString, i32>,
    thenums: Vec<i32>,
    themodes: Vec<i32>,
    theacts: Vec<Handle<dyn IFSelectActivator>>,
}

/// The process-wide registry (a guard to drop before calling an activator).
fn registry() -> RwLockWriteGuard<'static, ActivatorRegistry> {
    static REGISTRY: OnceLock<RwLock<ActivatorRegistry>> = OnceLock::new();
    REGISTRY
        .get_or_init(|| RwLock::new(ActivatorRegistry::default()))
        .write()
        .unwrap_or_else(PoisonError::into_inner)
}

/// The fields of `IFSelect_Activator` (group and file), with the handle `this` converts
/// to natively.
pub struct IFSelectActivatorBase {
    thegroup: SharedState<AsciiString>,
    thefile: SharedState<AsciiString>,
    this: OnceLock<WeakHandle<dyn IFSelectActivator>>,
}

impl Default for IFSelectActivatorBase {
    fn default() -> Self {
        Self::new()
    }
}

impl IFSelectActivatorBase {
    /// `IFSelect_Activator()` (protected): the group "XSTEP", no file.
    ///
    /// ```
    /// use rustycad_tkxsbase::if_select::IFSelectActivatorBase;
    /// let _base = IFSelectActivatorBase::new();
    /// ```
    pub fn new() -> Self {
        Self {
            thegroup: SharedState::new(AsciiString::new("XSTEP")),
            thefile: SharedState::default(),
            this: OnceLock::new(),
        }
    }
    /// Records the handle of the activator, which `Add` registers.
    pub fn bind_self(&self, actor: &Handle<dyn IFSelectActivator>) {
        let _ = self.this.set(actor.downgrade());
    }
    /// The handle `this` converts to natively; `Standard_NullObject` before `bind_self`.
    pub fn this(&self) -> Result<Handle<dyn IFSelectActivator>, Failure> {
        self.this
            .get()
            .map(WeakHandle::upgrade)
            .filter(|h| h.is_some())
            .ok_or_else(|| {
                exceptions::null_object(
                    "IFSelect_Activator: activator not bound to its handle; expected bind_self",
                )
            })
    }
    /// `Adding(actor, number, command, mode)`: the command registered for `actor` (a
    /// name already known is reported, the new definition replaces it).
    pub fn adding(actor: &Handle<dyn IFSelectActivator>, number: i32, command: &str, mode: i32) {
        let mut reg = registry();
        let key = AsciiString::new(command);
        if reg.thedico.is_bound(&key) {
            println!(
                "****  XSTEP commands, name conflict on {command} first defined remains  ****"
            );
        }
        let rank = reg.thenums.len() as i32 + 1;
        reg.thedico.bind(key, rank);
        reg.thenums.push(number);
        reg.theacts.push(actor.clone());
        reg.themodes.push(mode);
    }
    /// `Remove(command)`: the command name forgotten.
    pub fn remove(command: &str) {
        registry().thedico.unbind(&AsciiString::new(command));
    }
    /// `Select(command, number, actor)`: the number and the activator of a command,
    /// `None` when unknown.
    pub fn select(command: &str) -> Option<(i32, Handle<dyn IFSelectActivator>)> {
        let reg = registry();
        let rank = *reg.thedico.seek(&AsciiString::new(command))? as usize;
        Some((reg.thenums[rank - 1], reg.theacts[rank - 1].clone()))
    }
    /// `Mode(command)`: the mode of a command, -1 when unknown.
    pub fn mode(command: &str) -> i32 {
        let reg = registry();
        reg.thedico
            .seek(&AsciiString::new(command))
            .map_or(-1, |rank| reg.themodes[*rank as usize - 1])
    }
    /// `Commands(mode = -1, command = "")`: the commands starting with `command`, in the
    /// native dictionary order: of the mode `mode`, or (for a negative mode) all of them
    /// when `command` is empty. Native quirk kept: for a negative mode and a non empty
    /// `command`, the group of the activator must also be `command`.
    pub fn commands(mode: i32, command: &str) -> Vec<AsciiString> {
        let entries: Vec<(AsciiString, i32, i32, Handle<dyn IFSelectActivator>)> = {
            let reg = registry();
            reg.thedico
                .items()
                .map(|(k, r)| {
                    (
                        k.clone(),
                        *r,
                        reg.themodes[*r as usize - 1],
                        reg.theacts[*r as usize - 1].clone(),
                    )
                })
                .collect()
        };
        let mut list = Vec::new();
        for (key, _, kmode, acti) in entries {
            if !key.as_bytes().starts_with(command.as_bytes()) {
                continue;
            }
            let keep = if mode < 0 {
                acti.get().is_some_and(|a| {
                    command.is_empty() || a.group().as_bytes() == command.as_bytes()
                })
            } else {
                kmode == mode
            };
            if keep {
                list.push(key);
            }
        }
        list
    }
}

/// `IFSelect_Activator`, abstract: `Do` and `Help` are required; the protected
/// constructor is [`IFSelectActivatorBase::new`], whose `bind_self` an implementation
/// calls once it has its handle. The non-virtual members are default methods (do not
/// override them).
pub trait IFSelectActivator: Transient {
    /// The embedded fields.
    fn activator_base(&self) -> &IFSelectActivatorBase;
    /// `Do(number, pilot)`: executes the command `number` with the words of `pilot`.
    fn do_command(
        &self,
        number: i32,
        pilot: &Handle<IFSelectSessionPilot>,
    ) -> Result<IFSelectReturnStatus, Failure>;
    /// `Help(number)`: the help of the command `number`.
    fn help(&self, number: i32) -> AsciiString;
    /// `Add(number, command)`: registers `command` (mode 0, a plain command) for this
    /// activator.
    fn add(&self, number: i32, command: &str) -> Result<(), Failure> {
        IFSelectActivatorBase::adding(&self.activator_base().this()?, number, command, 0);
        Ok(())
    }
    /// `AddSet(number, command)`: registers `command` (mode 1, creating an item, for
    /// `xsnew`) for this activator.
    fn add_set(&self, number: i32, command: &str) -> Result<(), Failure> {
        IFSelectActivatorBase::adding(&self.activator_base().this()?, number, command, 1);
        Ok(())
    }
    /// `Group`.
    fn group(&self) -> AsciiString {
        self.activator_base().thegroup.read().clone()
    }
    /// `File`.
    fn file(&self) -> AsciiString {
        self.activator_base().thefile.read().clone()
    }
    /// `SetForGroup(group, file = "")`.
    fn set_for_group(&self, group: &str, file: &str) {
        let base = self.activator_base();
        *base.thegroup.write() = AsciiString::new(group);
        *base.thefile.write() = AsciiString::new(file);
    }
}
impl_transient_interface!(
    dyn IFSelectActivator,
    "IFSelect_Activator",
    StandardTransient
);
