// Rust port of DataExchange/TKXSBase/Interface/Interface_CheckIterator (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Interface_CheckIterator`: a list of checks with the numbers of their entities. The
//! copies share the lists and the cursor (natively three handles).
use super::{
    InterfaceCheck, InterfaceCheckStatus, InterfaceIntVal, InterfaceInterfaceModel,
    InterfaceModelEntities,
};
use rustycad_tkernel::{
    standard::{Failure, Handle, SharedState, StandardTransient, exceptions},
    tcollection::{AsciiString, HAsciiString},
};
use std::sync::{Arc, OnceLock};

/// The shared lists: the checks and their numbers.
#[derive(Default)]
struct CheckLists {
    thelist: Vec<Handle<InterfaceCheck>>,
    thenums: Vec<i32>,
}

fn null_check() -> Handle<InterfaceCheck> {
    static NULL_CHECK: OnceLock<Handle<InterfaceCheck>> = OnceLock::new();
    NULL_CHECK
        .get_or_init(|| Handle::new(InterfaceCheck::new()))
        .clone()
}

/// `Interface_CheckIterator`.
///
/// ```
/// use rustycad_tkxsbase::interface::{InterfaceCheck, InterfaceCheckIterator, InterfaceCheckStatus};
/// use rustycad_tkernel::standard::Handle;
/// let mut list = InterfaceCheckIterator::with_name("load");
/// let check = Handle::new(InterfaceCheck::new());
/// check.add_warning_text("w", "");
/// list.add(&check, 3);
/// assert_eq!(list.status(), InterfaceCheckStatus::CheckWarning);
/// assert!(list.check(3) == check);
/// ```
#[derive(Clone)]
pub struct InterfaceCheckIterator {
    lists: Arc<SharedState<CheckLists>>,
    themod: Handle<dyn InterfaceInterfaceModel>,
    thename: AsciiString,
    thecurr: Handle<InterfaceIntVal>,
}

impl Default for InterfaceCheckIterator {
    fn default() -> Self {
        Self::new()
    }
}

impl InterfaceCheckIterator {
    /// `Interface_CheckIterator()`: empty.
    pub fn new() -> Self {
        Self::with_name("")
    }
    /// `Interface_CheckIterator(name)`.
    pub fn with_name(name: &str) -> Self {
        let mut list = Self {
            lists: Arc::default(),
            themod: Handle::default(),
            thename: AsciiString::new(name),
            thecurr: Handle::default(),
        };
        list.clear();
        list
    }
    /// `SetName(name)`.
    pub fn set_name(&mut self, name: &str) {
        self.thename = AsciiString::new(name);
    }
    /// `Name`.
    pub fn name(&self) -> &AsciiString {
        &self.thename
    }
    /// `SetModel(model)`.
    pub fn set_model(&mut self, model: &Handle<dyn InterfaceInterfaceModel>) {
        self.themod = model.clone();
    }
    /// `Model`.
    pub fn model(&self) -> Handle<dyn InterfaceInterfaceModel> {
        self.themod.clone()
    }
    /// `Clear`: new empty lists and cursor.
    pub fn clear(&mut self) {
        self.lists = Arc::default();
        let cursor = InterfaceIntVal::new();
        *cursor.c_value() = 1;
        self.thecurr = Handle::new(cursor);
    }
    /// `Merge(other)`: takes the model and adds the checks of `other`.
    pub fn merge(&mut self, other: &InterfaceCheckIterator) {
        self.themod = other.model();
        for (num, ach) in other.items() {
            self.add(&ach, num);
        }
    }
    /// `Add(ach, num = 0)`: adds a check with warnings or fails; its number comes from its
    /// entity when `num` is not positive (-1 when unknown). A check for the number of the
    /// last addition (or before it) is merged into the recorded one.
    pub fn add(&mut self, ach: &Handle<InterfaceCheck>, num: i32) {
        let Some(check) = ach.get() else { return };
        if check.nb_warnings() + check.nb_fails() == 0 {
            return;
        }
        let nm = self.number_of(check, num);
        let last = self.thecurr.value();
        let mut lists = self.lists.write();
        if nm >= 0 && nm <= -last {
            let numpos = lists.thenums.iter().rposition(|n| *n == nm);
            if let Some(position) = numpos {
                let recorded = lists.thelist[position].clone();
                drop(lists);
                recorded.get_messages(check);
                return;
            }
            lists.thelist.push(ach.clone());
            lists.thenums.push(nm);
            return;
        }
        lists.thelist.push(ach.clone());
        lists.thenums.push(nm);
        *self.thecurr.c_value() = -nm;
    }
    fn number_of(&self, check: &InterfaceCheck, num: i32) -> i32 {
        if num > 0 || !check.has_entity() {
            return num;
        }
        let Some(model) = self.themod.get() else {
            return -1;
        };
        let nm = model.number(&check.entity());
        if nm <= 0 { -1 } else { nm }
    }
    /// `Check(num)`: the check of a number (a shared empty check when none).
    pub fn check(&self, num: i32) -> Handle<InterfaceCheck> {
        let lists = self.lists.read();
        let found = lists.thenums.iter().position(|n| *n == num);
        found.map_or_else(null_check, |i| lists.thelist[i].clone())
    }
    /// `Check(ent)`: by its number in the model, else by the entity of the checks.
    pub fn check_entity(&self, ent: &Handle<StandardTransient>) -> Handle<InterfaceCheck> {
        let num = self.themod.get().map_or(-1, |m| m.number(ent));
        if num > 0 {
            return self.check(num);
        }
        let lists = self.lists.read();
        let found = lists.thelist.iter().find(|c| c.entity() == *ent);
        found.cloned().unwrap_or_else(null_check)
    }
    /// `CCheck(num)`: the check of a number, created (empty) when none.
    pub fn c_check(&mut self, num: i32) -> Handle<InterfaceCheck> {
        let mut lists = self.lists.write();
        if let Some(i) = lists.thenums.iter().position(|n| *n == num) {
            return lists.thelist[i].clone();
        }
        let ach = Handle::new(InterfaceCheck::new());
        lists.thelist.push(ach.clone());
        lists.thenums.push(num);
        ach
    }
    /// `CCheck(ent)`: by number or entity, created (with number -1 or the entity number)
    /// when none.
    pub fn c_check_entity(&mut self, ent: &Handle<StandardTransient>) -> Handle<InterfaceCheck> {
        let num = self.themod.get().map_or(-1, |m| m.number(ent));
        if num > 0 {
            return self.c_check(num);
        }
        let mut lists = self.lists.write();
        if let Some(found) = lists.thelist.iter().find(|c| c.entity() == *ent) {
            return found.clone();
        }
        let ach = Handle::new(InterfaceCheck::new());
        lists.thelist.push(ach.clone());
        lists.thenums.push(num);
        ach
    }
    /// `IsEmpty(failsonly)`.
    pub fn is_empty(&self, failsonly: bool) -> bool {
        let lists = self.lists.read();
        if lists.thelist.is_empty() {
            return true;
        }
        failsonly && !lists.thelist.iter().any(|c| c.has_failed())
    }
    /// `Status`: fail as soon as a check failed, else warning when a check warns.
    pub fn status(&self) -> InterfaceCheckStatus {
        let mut stat = InterfaceCheckStatus::CheckOK;
        for ach in self.lists.read().thelist.iter() {
            if ach.has_failed() {
                return InterfaceCheckStatus::CheckFail;
            }
            if ach.nb_warnings() > 0 {
                stat = InterfaceCheckStatus::CheckWarning;
            }
        }
        stat
    }
    /// `Complies(status)`.
    pub fn complies(&self, stat: InterfaceCheckStatus) -> bool {
        let mut res = stat == InterfaceCheckStatus::CheckNoFail;
        for ach in self.lists.read().thelist.iter() {
            let (nbf, nbw) = (ach.nb_fails(), ach.nb_warnings());
            match stat {
                InterfaceCheckStatus::CheckOK if nbf + nbw > 0 => return false,
                InterfaceCheckStatus::CheckWarning if nbf > 0 => return false,
                InterfaceCheckStatus::CheckWarning if nbw > 0 => res = true,
                InterfaceCheckStatus::CheckFail if nbf > 0 => return true,
                InterfaceCheckStatus::CheckAny => return true,
                InterfaceCheckStatus::CheckMessage if nbf + nbw > 0 => return true,
                InterfaceCheckStatus::CheckNoFail if nbf > 0 => return false,
                _ => {}
            }
        }
        res
    }
    /// `Extract(status)`: the checks of a status, in a new iterator.
    pub fn extract(&self, stat: InterfaceCheckStatus) -> InterfaceCheckIterator {
        self.extract_where(|ach| status_takes(stat, ach.nb_fails(), ach.nb_warnings()))
    }
    /// `Extract(mess, incl, status)`: the checks with a matching message.
    pub fn extract_message(
        &self,
        mess: &str,
        incl: i32,
        stat: InterfaceCheckStatus,
    ) -> Result<InterfaceCheckIterator, Failure> {
        let text = Handle::new(HAsciiString::new(mess));
        let mut failure = None;
        let res = self.extract_where(|ach| match ach.complies_message(&text, incl, stat) {
            Ok(found) => found,
            Err(error) => {
                failure.get_or_insert(error);
                false
            }
        });
        failure.map_or(Ok(res), Err)
    }
    /// `Remove(mess, incl, status)`: removes the matching messages from every check.
    pub fn remove(
        &mut self,
        mess: &str,
        incl: i32,
        stat: InterfaceCheckStatus,
    ) -> Result<bool, Failure> {
        let text = Handle::new(HAsciiString::new(mess));
        let checks = self.lists.read().thelist.clone();
        let mut res = false;
        for ach in checks {
            res |= ach.remove(&text, incl, stat)?;
        }
        Ok(res)
    }
    /// `Checkeds(failsonly, global)`: the entities of the checks (the model for the global
    /// check); null without model. Native quirk kept: a check without warning is skipped
    /// even when it has fails.
    pub fn checkeds(
        &self,
        failsonly: bool,
        global: bool,
    ) -> Result<Option<Vec<Handle<StandardTransient>>>, Failure> {
        let Some(model) = self.themod.get() else {
            return Ok(None);
        };
        let mut list = Vec::new();
        for (num, chk) in self.items() {
            if (failsonly && !chk.has_failed()) || chk.nb_warnings() == 0 {
                continue;
            }
            if num == 0 && global {
                list.push(self.themod.cast());
            } else if num > 0 {
                list.push(model.value(num)?);
            }
        }
        Ok(Some(list))
    }
    /// `Start`.
    pub fn start(&self) {
        *self.cursor().c_value() = 1;
    }
    /// `More`: a negative cursor (the last added number) restarts.
    pub fn more(&self) -> bool {
        let cursor = self.cursor();
        if cursor.value() < 0 {
            *cursor.c_value() = 1;
        }
        cursor.value() <= self.lists.read().thelist.len() as i32
    }
    /// `Next`.
    pub fn next(&self) {
        let cursor = self.cursor();
        let mut value = cursor.c_value();
        if *value < 0 {
            *value = 1;
        }
        *value += 1;
    }
    /// `Value`: `Standard_NoSuchObject` after the end.
    pub fn value(&self) -> Result<Handle<InterfaceCheck>, Failure> {
        let index = self.current_index()?;
        Ok(self.lists.read().thelist[index].clone())
    }
    /// `Number`: `Standard_NoSuchObject` after the end.
    pub fn number(&self) -> Result<i32, Failure> {
        let index = self.current_index()?;
        Ok(self.lists.read().thenums[index])
    }
    /// `Destroy`: forgets the cursor (the native destructor's work).
    pub fn destroy(&mut self) {
        self.thecurr.nullify();
    }
    /// The (number, check) pairs in order (what a `Start`/`More`/`Next` loop visits).
    pub fn items(&self) -> Vec<(i32, Handle<InterfaceCheck>)> {
        let lists = self.lists.read();
        lists
            .thenums
            .iter()
            .copied()
            .zip(lists.thelist.iter().cloned())
            .collect()
    }
    /// The shared checks and numbers (for the printing members).
    pub(super) fn snapshot(&self) -> (Vec<Handle<InterfaceCheck>>, Vec<i32>) {
        let lists = self.lists.read();
        (lists.thelist.clone(), lists.thenums.clone())
    }
    fn extract_where(
        &self,
        mut take: impl FnMut(&InterfaceCheck) -> bool,
    ) -> InterfaceCheckIterator {
        let mut res = InterfaceCheckIterator::with_name("");
        res.set_model(&self.themod);
        res.thename = self.thename.clone();
        for (num, ach) in self.items() {
            if take(&ach) {
                res.add(&ach, num);
            }
        }
        res
    }
    fn cursor(&self) -> Handle<InterfaceIntVal> {
        // A destroyed iterator has no cursor natively (a crash); it gets a new one.
        if self.thecurr.is_null() {
            return Handle::new(InterfaceIntVal::new());
        }
        self.thecurr.clone()
    }
    fn current_index(&self) -> Result<usize, Failure> {
        let current = self.cursor().value();
        let length = self.lists.read().thelist.len();
        let no_such = || {
            exceptions::no_such_object(&format!(
                "Interface Check Iterator : Value: rank {current}; expected [1, {length}]"
            ))
        };
        let index = usize::try_from(current - 1).map_err(|_| no_such())?;
        if index >= length {
            return Err(no_such());
        }
        Ok(index)
    }
}

/// Whether `Extract(status)` takes a check with these counts.
fn status_takes(stat: InterfaceCheckStatus, nbf: i32, nbw: i32) -> bool {
    match stat {
        InterfaceCheckStatus::CheckOK => nbf + nbw == 0,
        InterfaceCheckStatus::CheckWarning => nbf == 0 && nbw > 0,
        InterfaceCheckStatus::CheckFail => nbf > 0,
        InterfaceCheckStatus::CheckAny => true,
        InterfaceCheckStatus::CheckMessage => nbf + nbw > 0,
        InterfaceCheckStatus::CheckNoFail => nbf == 0,
    }
}
