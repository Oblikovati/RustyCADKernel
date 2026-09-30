// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepCheck/BRepCheck_Result.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepCheck_Result`: the abstract checker of one shape, with the status lists of the
//! shape itself and of each context it was checked in (docs/topalgo-design.md section 6).
use super::errors::no_such_object;
use super::package::BRepCheck;
use super::status::{BRepCheckStatus, ListOfStatus};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::impl_transient_interface;
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::standard::{Failure, SharedState, StandardTransient, Transient};
use std::sync::Arc;

/// `Handle(NCollection_Shared<NCollection_List<BRepCheck_Status>>)`: one status list,
/// shared by the map and by the member that appends to it after releasing the lock.
pub(crate) type SharedStatusList = Arc<SharedState<ListOfStatus>>;

/// A fresh empty shared list (`new NCollection_Shared<...>()`).
pub(crate) fn new_status_list() -> SharedStatusList {
    Arc::new(SharedState::new(ListOfStatus::default()))
}

/// `BRepCheck::Add` on a shared list.
pub(crate) fn add_status(list: &SharedStatusList, status: BRepCheckStatus) {
    BRepCheck::add(&mut list.write(), status);
}

/// Appends `NoError` to a list left empty by the checks (the native closing
/// `if (lst.IsEmpty()) lst.Append(BRepCheck_NoError)`).
pub(crate) fn close_status_list(list: &SharedStatusList) {
    let mut guard = list.write();
    if guard.is_empty() {
        guard.append(BRepCheckStatus::NoError);
    }
}

/// The protected fields of `BRepCheck_Result` and its private context iterator.
struct ResultFields {
    shape: Shape,
    min: bool,
    blind: bool,
    is_parallel: bool,
    map: DataMap<Shape, SharedStatusList, ShapeMapHasher>,
    // `myIter`: the bindings the iterator visits, in the map's order, and its position.
    context: Vec<(Shape, SharedStatusList)>,
    position: usize,
}

/// The protected data of `BRepCheck_Result` (`myShape`, `myMin`, `myBlind`,
/// `myIsParallel`, `myMap`) with the context iterator, embedded by every checker.
///
/// ```
/// use rustycad_tkbrep::topods::Shape;
/// use rustycad_tktopalgo::brep_check::ResultBase;
/// let base = ResultBase::new();
/// assert!(base.shape().is_null() && !base.is_minimum());
/// ```
pub struct ResultBase {
    fields: SharedState<ResultFields>,
}

impl Default for ResultBase {
    fn default() -> Self {
        Self::new()
    }
}

impl ResultBase {
    /// The protected `BRepCheck_Result()`: no shape, `myMin` and `myBlind` false;
    /// `ResultBase::new()`.
    pub fn new() -> Self {
        Self {
            fields: SharedState::new(ResultFields {
                shape: Shape::default(),
                min: false,
                blind: false,
                is_parallel: false,
                map: DataMap::default(),
                context: Vec::new(),
                position: 0,
            }),
        }
    }

    /// `myShape`; `base.shape()`.
    pub fn shape(&self) -> Shape {
        self.fields.read().shape.clone()
    }

    /// `myMin`; `base.is_minimum()`.
    pub fn is_minimum(&self) -> bool {
        self.fields.read().min
    }

    /// Sets `myMin`; `base.set_minimum(true)`.
    pub fn set_minimum(&self, value: bool) {
        self.fields.write().min = value;
    }

    /// `myBlind`; `base.is_blind()`.
    pub fn is_blind(&self) -> bool {
        self.fields.read().blind
    }

    /// Sets `myBlind`; `base.set_blind(true)`.
    pub fn set_blind(&self, value: bool) {
        self.fields.write().blind = value;
    }

    /// `myIsParallel`; `base.is_parallel()`.
    pub fn is_parallel(&self) -> bool {
        self.fields.read().is_parallel
    }

    /// The first half of `Init(S)`: the shape set, the flags cleared, the map emptied.
    fn reset(&self, shape: &Shape) {
        let mut fields = self.fields.write();
        fields.shape = shape.clone();
        fields.min = false;
        fields.blind = false;
        fields.map.clear(false);
    }

    /// `*myMap.Bound(myShape, new list)`: a fresh list bound to the shape, replacing an
    /// earlier one; `let list = base.bind_own_list();`.
    pub(crate) fn bind_own_list(&self) -> SharedStatusList {
        let mut fields = self.fields.write();
        let list = new_status_list();
        let key = fields.shape.clone();
        fields.map.bound(key, list.clone());
        list
    }

    /// The InContext prologue: `None` when `context` already has a list, else a fresh
    /// list bound to it (under the native lock).
    pub(crate) fn bind_context_list(&self, context: &Shape) -> Option<SharedStatusList> {
        let mut fields = self.fields.write();
        if fields.map.is_bound(context) {
            return None;
        }
        let list = new_status_list();
        fields.map.bound(context.clone(), list.clone());
        Some(list)
    }

    /// `myMap(myShape)`: Standard_NoSuchObject when the shape has no list.
    pub(crate) fn own_list(&self) -> Result<SharedStatusList, Failure> {
        let fields = self.fields.read();
        fields.map.find(&fields.shape).cloned()
    }

    /// `myMap.Find(S)`: Standard_NoSuchObject when `shape` has no list.
    pub(crate) fn list_of(&self, shape: &Shape) -> Result<SharedStatusList, Failure> {
        self.fields.read().map.find(shape).cloned()
    }

    /// `SetFailStatus(S)`: `BRepCheck_CheckFail` added to the list of `shape`, bound
    /// first when missing.
    fn set_fail_status(&self, shape: &Shape) {
        let list = {
            let mut fields = self.fields.write();
            if let Some(found) = fields.map.seek(shape) {
                found.clone()
            } else {
                let list = new_status_list();
                fields.map.bind(shape.clone(), list.clone());
                list
            }
        };
        add_status(&list, BRepCheckStatus::CheckFail);
    }

    /// `InitContextIterator()`: the bindings in map order, the shape's own one skipped
    /// when it comes first; Standard_NoSuchObject on an empty map (`myIter.Key()`).
    fn init_context_iterator(&self) -> Result<(), Failure> {
        let mut fields = self.fields.write();
        let mut context = Vec::with_capacity(fields.map.len());
        let mut cursor = fields.map.cursor();
        while cursor.more() {
            context.push((cursor.key()?.clone(), cursor.value()?.clone()));
            cursor.next();
        }
        let Some((first, _)) = context.first() else {
            return Err(no_such_object(
                "NCollection_DataMap::Iterator::Key: no binding for the context iterator",
            ));
        };
        fields.position = usize::from(first.is_same(&fields.shape));
        fields.context = context;
        Ok(())
    }

    /// `NextShapeInContext()`: the next binding, skipping the shape's own one.
    fn next_shape_in_context(&self) {
        let mut fields = self.fields.write();
        fields.position += 1;
        let position = fields.position;
        if position < fields.context.len() && fields.context[position].0.is_same(&fields.shape) {
            fields.position += 1;
        }
    }

    /// The current binding of the context iterator; Standard_NoSuchObject past the end.
    fn current_context(&self) -> Result<(Shape, SharedStatusList), Failure> {
        let fields = self.fields.read();
        fields.context.get(fields.position).cloned().ok_or_else(|| {
            no_such_object(&format!(
                "BRepCheck_Result: context iterator at {} of {} bindings",
                fields.position,
                fields.context.len()
            ))
        })
    }
}

/// `BRepCheck_Result`: the checker of one shape (docs/topalgo-design.md section 6).
/// `Minimum` checks the shape itself and is run by `Init`; `InContext` checks it as a
/// sub-shape of another shape and records the statuses under that shape; `Blind` is
/// the (empty) check of the shape without context.
///
/// ```
/// use rustycad_tkbrep::topods::Vertex;
/// use rustycad_tktopalgo::brep_check::{BRepCheckResult, BRepCheckVertex};
/// let checker = BRepCheckVertex::new(&Vertex::default()).unwrap();
/// assert!(checker.is_minimum());
/// ```
pub trait BRepCheckResult: Transient {
    /// The embedded `BRepCheck_Result` data.
    fn result_base(&self) -> &ResultBase;

    /// `InContext(ContextShape)`: checks the shape as a sub-shape of `context`.
    fn in_context(&self, context: &Shape) -> Result<(), Failure>;

    /// `Minimum()`: the checks of the shape itself, done once.
    fn minimum(&self) -> Result<(), Failure>;

    /// `Blind()`: marks the shape checked without context (no check natively).
    fn blind(&self);

    /// `Init(S)`: the shape set, the statuses cleared, then `Minimum()`;
    /// `checker.init(&shape)?`.
    fn init(&self, shape: &Shape) -> Result<(), Failure> {
        self.result_base().reset(shape);
        self.minimum()
    }

    /// `SetFailStatus(S)`: records `BRepCheck_CheckFail` for `shape`;
    /// `checker.set_fail_status(&shape)`.
    fn set_fail_status(&self, shape: &Shape) {
        self.result_base().set_fail_status(shape);
    }

    /// `Status()`: the statuses of the shape itself; Standard_NoSuchObject when there
    /// is none; `checker.status()?`.
    fn status(&self) -> Result<ListOfStatus, Failure> {
        Ok(self.result_base().own_list()?.read().clone())
    }

    /// `IsMinimum()`; `checker.is_minimum()`.
    fn is_minimum(&self) -> bool {
        self.result_base().is_minimum()
    }

    /// `IsBlind()`; `checker.is_blind()`.
    fn is_blind(&self) -> bool {
        self.result_base().is_blind()
    }

    /// `InitContextIterator()`: starts on the first context shape;
    /// `checker.init_context_iterator()?`.
    fn init_context_iterator(&self) -> Result<(), Failure> {
        self.result_base().init_context_iterator()
    }

    /// `MoreShapeInContext()`; `checker.more_shape_in_context()`.
    fn more_shape_in_context(&self) -> bool {
        self.result_base().current_context().is_ok()
    }

    /// `ContextualShape()`: the current context shape; `checker.contextual_shape()?`.
    fn contextual_shape(&self) -> Result<Shape, Failure> {
        Ok(self.result_base().current_context()?.0)
    }

    /// `StatusOnShape()`: the statuses in the current context shape;
    /// `checker.status_on_shape()?`.
    fn status_on_shape(&self) -> Result<ListOfStatus, Failure> {
        Ok(self.result_base().current_context()?.1.read().clone())
    }

    /// `NextShapeInContext()`; `checker.next_shape_in_context()`.
    fn next_shape_in_context(&self) {
        self.result_base().next_shape_in_context();
    }

    /// `SetParallel(theIsParallel)`; `checker.set_parallel(true)`.
    fn set_parallel(&self, is_parallel: bool) {
        self.result_base().fields.write().is_parallel = is_parallel;
    }

    /// `IsParallel()`; `checker.is_parallel()`.
    fn is_parallel(&self) -> bool {
        self.result_base().is_parallel()
    }

    /// `IsStatusOnShape(theShape)`: `shape` has a status list;
    /// `checker.is_status_on_shape(&shape)`.
    fn is_status_on_shape(&self, shape: &Shape) -> bool {
        self.result_base().fields.read().map.is_bound(shape)
    }

    /// `StatusOnShape(theShape)`: the statuses recorded for `shape`;
    /// Standard_NoSuchObject when there are none; `checker.status_on_shape_of(&shape)?`.
    fn status_on_shape_of(&self, shape: &Shape) -> Result<ListOfStatus, Failure> {
        Ok(self.result_base().list_of(shape)?.read().clone())
    }
}

impl_transient_interface!(dyn BRepCheckResult, "BRepCheck_Result", StandardTransient);
