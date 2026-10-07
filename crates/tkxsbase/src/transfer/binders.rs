// Rust port of DataExchange/TKXSBase/Transfer/Transfer_SimpleBinderOfTransient,
// Transfer_VoidBinder, Transfer_MultipleBinder and Transfer_TransientListBinder (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The concrete binders of transients: one result, no result, several results.
use super::{TransferBinder, TransferBinderBase};
use rustycad_tkernel::{
    impl_transient,
    standard::{
        Failure, Handle, RuntimeType, RuntimeTyped, SharedState, StandardTransient, exceptions,
    },
};

/// `Transfer_SimpleBinderOfTransient`: one transient result.
///
/// ```
/// use rustycad_tkxsbase::transfer::{TransferBinder, TransferSimpleBinderOfTransient};
/// use rustycad_tkernel::standard::{Handle, StandardTransient};
/// let binder = TransferSimpleBinderOfTransient::new();
/// binder.set_result(&Handle::new(StandardTransient::default())).unwrap();
/// assert!(binder.has_result());
/// assert_eq!(binder.result_type_name(), "Standard_Transient");
/// ```
#[derive(Default)]
pub struct TransferSimpleBinderOfTransient {
    transient: StandardTransient,
    base: TransferBinderBase,
    theres: SharedState<Handle<StandardTransient>>,
}
impl_transient!(TransferSimpleBinderOfTransient, "Transfer_SimpleBinderOfTransient", dyn TransferBinder, transient, interfaces: [dyn TransferBinder]);

impl TransferSimpleBinderOfTransient {
    /// `Transfer_SimpleBinderOfTransient()`: no result.
    pub fn new() -> Self {
        Self::default()
    }
    /// `SetResult(res)`: `Transfer_TransferFailure` when the result was already used.
    pub fn set_result(&self, res: &Handle<StandardTransient>) -> Result<(), Failure> {
        self.set_result_present()?;
        *self.theres.write() = res.clone();
        Ok(())
    }
    /// `Result`.
    pub fn result(&self) -> Handle<StandardTransient> {
        self.theres.read().clone()
    }
    /// `GetTypedResult(bnd, atype, res)`: the first result of the chain of `bnd` (simple
    /// binders only) kind of `atype`.
    pub fn get_typed_result(
        bnd: &Handle<dyn TransferBinder>,
        atype: &RuntimeType,
        res: &mut Handle<StandardTransient>,
    ) -> bool {
        let mut current = bnd.clone();
        while let Some(live) = current.get() {
            let simple = current.cast::<TransferSimpleBinderOfTransient>();
            let next = live.next_result();
            if let Some(binder) = simple.get() {
                let rs = binder.result();
                if rs.is_some() && rs.is_kind(atype) {
                    *res = rs;
                    return true;
                }
            }
            current = next;
        }
        false
    }
}

impl TransferBinder for TransferSimpleBinderOfTransient {
    fn binder_base(&self) -> &TransferBinderBase {
        &self.base
    }
    /// `ResultType`: the type of the result, `Standard_Transient` without one.
    fn result_type(&self) -> &'static RuntimeType {
        let res = self.result();
        match res.dynamic_type().filter(|_| self.has_result()) {
            Some(kind) => kind,
            None => StandardTransient::static_type(),
        }
    }
    /// `ResultTypeName`: "(void)" without result.
    fn result_type_name(&self) -> String {
        let res = self.result();
        match res.dynamic_type().filter(|_| self.has_result()) {
            Some(kind) => kind.name().to_owned(),
            None => String::from("(void)"),
        }
    }
}

/// `Transfer_VoidBinder`: a binder without result (for the checks).
#[derive(Default)]
pub struct TransferVoidBinder {
    transient: StandardTransient,
    base: TransferBinderBase,
}
impl_transient!(TransferVoidBinder, "Transfer_VoidBinder", dyn TransferBinder, transient, interfaces: [dyn TransferBinder]);

impl TransferVoidBinder {
    /// `Transfer_VoidBinder()`.
    pub fn new() -> Self {
        Self::default()
    }
}

impl TransferBinder for TransferVoidBinder {
    fn binder_base(&self) -> &TransferBinderBase {
        &self.base
    }
    /// `ResultType`: its own dynamic type.
    fn result_type(&self) -> &'static RuntimeType {
        Self::static_type()
    }
    /// `ResultTypeName`: "(void)".
    fn result_type_name(&self) -> String {
        String::from("(void)")
    }
}

/// `Transfer_MultipleBinder`: a list of results (created at the first one).
#[derive(Default)]
pub struct TransferMultipleBinder {
    transient: StandardTransient,
    base: TransferBinderBase,
    themulres: SharedState<Option<Vec<Handle<StandardTransient>>>>,
}
impl_transient!(TransferMultipleBinder, "Transfer_MultipleBinder", dyn TransferBinder, transient, interfaces: [dyn TransferBinder]);

impl TransferMultipleBinder {
    /// `Transfer_MultipleBinder()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `AddResult(res)`: appends a result.
    pub fn add_transient_result(&self, res: &Handle<StandardTransient>) {
        self.themulres
            .write()
            .get_or_insert_with(Vec::new)
            .push(res.clone());
    }
    /// `NbResults`.
    pub fn nb_results(&self) -> i32 {
        self.themulres.read().as_ref().map_or(0, |l| l.len() as i32)
    }
    /// `ResultValue(num)`: `Standard_OutOfRange` out of 1..NbResults.
    pub fn result_value(&self, num: i32) -> Result<Handle<StandardTransient>, Failure> {
        let list = self.themulres.read();
        let found = list
            .as_ref()
            .and_then(|l| usize::try_from(num - 1).ok().and_then(|i| l.get(i)));
        found.cloned().ok_or_else(|| {
            exceptions::out_of_range(&format!(
                "Transfer_MultipleBinder::ResultValue: {num}; expected [1, {}]",
                self.nb_results()
            ))
        })
    }
    /// `MultipleResult`: a copy of the results (empty when none).
    pub fn multiple_result(&self) -> Vec<Handle<StandardTransient>> {
        self.themulres.read().clone().unwrap_or_default()
    }
    /// `SetMultipleResult(mulres)`.
    pub fn set_multiple_result(&self, mulres: Option<Vec<Handle<StandardTransient>>>) {
        *self.themulres.write() = mulres;
    }
}

impl TransferBinder for TransferMultipleBinder {
    fn binder_base(&self) -> &TransferBinderBase {
        &self.base
    }
    /// `ResultType`: `Standard_Transient`.
    fn result_type(&self) -> &'static RuntimeType {
        StandardTransient::static_type()
    }
    /// `ResultTypeName`: "(list)".
    fn result_type_name(&self) -> String {
        String::from("(list)")
    }
    /// `IsMultiple`: other than one result.
    fn is_multiple(&self) -> bool {
        self.themulres.read().as_ref().is_some_and(|l| l.len() != 1)
    }
}

/// `Transfer_TransientListBinder`: a list of transient results.
#[derive(Default)]
pub struct TransferTransientListBinder {
    transient: StandardTransient,
    base: TransferBinderBase,
    theres: SharedState<Vec<Handle<StandardTransient>>>,
}
impl_transient!(TransferTransientListBinder, "Transfer_TransientListBinder", dyn TransferBinder, transient, interfaces: [dyn TransferBinder]);

impl TransferTransientListBinder {
    /// `Transfer_TransientListBinder()`: an empty list.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Transfer_TransientListBinder(list)`.
    pub fn with_list(list: &[Handle<StandardTransient>]) -> Self {
        let binder = Self::default();
        *binder.theres.write() = list.to_vec();
        binder
    }
    /// `AddResult(res)`.
    pub fn add_transient_result(&self, res: &Handle<StandardTransient>) {
        self.theres.write().push(res.clone());
    }
    /// `Result`: a copy of the list.
    pub fn result(&self) -> Vec<Handle<StandardTransient>> {
        self.theres.read().clone()
    }
    /// `SetResult(num, res)`: `Standard_OutOfRange` out of the list.
    pub fn set_result(&self, num: i32, res: &Handle<StandardTransient>) -> Result<(), Failure> {
        let mut list = self.theres.write();
        let len = list.len();
        let slot = usize::try_from(num - 1).ok().and_then(|i| list.get_mut(i));
        *slot.ok_or_else(|| {
            exceptions::out_of_range(&format!(
                "Transfer_TransientListBinder::SetResult: {num}; expected [1, {len}]"
            ))
        })? = res.clone();
        Ok(())
    }
    /// `NbTransients`.
    pub fn nb_transients(&self) -> i32 {
        self.theres.read().len() as i32
    }
    /// `Transient(num)`: `Standard_OutOfRange` out of the list.
    pub fn transient(&self, num: i32) -> Result<Handle<StandardTransient>, Failure> {
        let list = self.theres.read();
        let found = usize::try_from(num - 1).ok().and_then(|i| list.get(i));
        found.cloned().ok_or_else(|| {
            exceptions::out_of_range(&format!(
                "Transfer_TransientListBinder::Transient: {num}; expected [1, {}]",
                list.len()
            ))
        })
    }
}

impl TransferBinder for TransferTransientListBinder {
    fn binder_base(&self) -> &TransferBinderBase {
        &self.base
    }
    /// `ResultType`: `Transfer_TransientListBinder` itself.
    fn result_type(&self) -> &'static RuntimeType {
        Self::static_type()
    }
    /// `ResultTypeName`: "list(Standard_Transient)".
    fn result_type_name(&self) -> String {
        String::from("list(Standard_Transient)")
    }
    /// `IsMultiple`: more than one transient.
    fn is_multiple(&self) -> bool {
        self.nb_transients() > 1
    }
}
