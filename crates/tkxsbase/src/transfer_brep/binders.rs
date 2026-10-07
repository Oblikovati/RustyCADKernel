// Rust port of DataExchange/TKXSBase/TransferBRep/TransferBRep_BinderOfShape,
// TransferBRep_ShapeBinder and TransferBRep_ShapeListBinder (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The binders of shape results: one shape (`BinderOfShape`, `ShapeBinder` with its
//! typed accessors) or a list of shapes (`ShapeListBinder`).
use super::TransferBRepShapeInfo;
use crate::transfer::{TransferBinder, TransferBinderBase};
use rustycad_tkbrep::topods::{
    self, CompSolid, Compound, Edge, Face, Shape, Shell, Solid, Vertex, Wire,
};
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, RuntimeType, RuntimeTyped, SharedState, StandardTransient, exceptions},
};
use rustycad_tkg3d::top_abs::ShapeEnum;
use std::ops::Deref;
use std::sync::RwLockWriteGuard;

/// `TransferBRep_BinderOfShape`: one shape result.
///
/// ```
/// use rustycad_tkxsbase::transfer::TransferBinder;
/// use rustycad_tkxsbase::transfer_brep::TransferBRepBinderOfShape;
/// let binder = TransferBRepBinderOfShape::new();
/// assert!(!binder.has_result() && binder.result_type_name() == "TopoDS_Shape");
/// ```
#[derive(Default)]
pub struct TransferBRepBinderOfShape {
    transient: StandardTransient,
    base: TransferBinderBase,
    theres: SharedState<Shape>,
}
impl_transient!(TransferBRepBinderOfShape, "TransferBRep_BinderOfShape", dyn TransferBinder, transient, interfaces: [dyn TransferBinder]);

impl TransferBRepBinderOfShape {
    /// `TransferBRep_BinderOfShape()`: no result.
    pub fn new() -> Self {
        Self::default()
    }
    /// `TransferBRep_BinderOfShape(res)`: the result `res` (present).
    pub fn with_result(res: &Shape) -> Self {
        let binder = Self::default();
        *binder.theres.write() = res.clone();
        // a new binder is never "used": SetResultPresent cannot fail
        let _ = binder.set_result_present();
        binder
    }
    /// `SetResult(res)`: `Transfer_TransferFailure` when the result was already used.
    pub fn set_result(&self, res: &Shape) -> Result<(), Failure> {
        self.set_result_present()?;
        *self.theres.write() = res.clone();
        Ok(())
    }
    /// `Result`.
    pub fn result(&self) -> Shape {
        self.theres.read().clone()
    }
    /// `CResult`: the result, modifiable (the write guard), set present first.
    pub fn c_result(&self) -> Result<RwLockWriteGuard<'_, Shape>, Failure> {
        self.set_result_present()?;
        Ok(self.theres.write())
    }
}

impl TransferBinder for TransferBRepBinderOfShape {
    fn binder_base(&self) -> &TransferBinderBase {
        &self.base
    }
    /// `ResultType`: `TopoDS_TShape` (`TransferBRep_ShapeInfo::Type`).
    fn result_type(&self) -> &'static RuntimeType {
        TransferBRepShapeInfo::type_of(&self.result())
    }
    /// `ResultTypeName`: the name of the typed shape (`TransferBRep_ShapeInfo::TypeName`).
    fn result_type_name(&self) -> String {
        TransferBRepShapeInfo::type_name(&self.result()).to_owned()
    }
}

/// `TransferBRep_ShapeBinder`: a binder of shape with the typed accessors; embeds its
/// `TransferBRep_BinderOfShape` (registered as its RTTI parent).
///
/// ```
/// use rustycad_tkxsbase::transfer_brep::TransferBRepShapeBinder;
/// use rustycad_tkg3d::top_abs::ShapeEnum;
/// assert_eq!(TransferBRepShapeBinder::new().shape_type().unwrap(), ShapeEnum::Shape);
/// ```
#[derive(Default)]
pub struct TransferBRepShapeBinder {
    base: TransferBRepBinderOfShape,
}
impl_transient!(TransferBRepShapeBinder, "TransferBRep_ShapeBinder", TransferBRepBinderOfShape, base, interfaces: [dyn TransferBinder]);

impl Deref for TransferBRepShapeBinder {
    type Target = TransferBRepBinderOfShape;
    fn deref(&self) -> &TransferBRepBinderOfShape {
        &self.base
    }
}

impl TransferBinder for TransferBRepShapeBinder {
    fn binder_base(&self) -> &TransferBinderBase {
        self.base.binder_base()
    }
    fn result_type(&self) -> &'static RuntimeType {
        self.base.result_type()
    }
    fn result_type_name(&self) -> String {
        self.base.result_type_name()
    }
}

impl TransferBRepShapeBinder {
    /// `TransferBRep_ShapeBinder()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `TransferBRep_ShapeBinder(res)`.
    pub fn with_result(res: &Shape) -> Self {
        Self {
            base: TransferBRepBinderOfShape::with_result(res),
        }
    }
    /// `ShapeType`: `TopAbs_SHAPE` without result.
    pub fn shape_type(&self) -> Result<ShapeEnum, Failure> {
        if !self.has_result() {
            return Ok(ShapeEnum::Shape);
        }
        self.result().shape_type()
    }
    /// `Vertex`: `Standard_TypeMismatch` for another type.
    pub fn vertex(&self) -> Result<Vertex, Failure> {
        topods::vertex(&self.result())
    }
    /// `Edge`.
    pub fn edge(&self) -> Result<Edge, Failure> {
        topods::edge(&self.result())
    }
    /// `Wire`.
    pub fn wire(&self) -> Result<Wire, Failure> {
        topods::wire(&self.result())
    }
    /// `Face`.
    pub fn face(&self) -> Result<Face, Failure> {
        topods::face(&self.result())
    }
    /// `Shell`.
    pub fn shell(&self) -> Result<Shell, Failure> {
        topods::shell(&self.result())
    }
    /// `Solid`.
    pub fn solid(&self) -> Result<Solid, Failure> {
        topods::solid(&self.result())
    }
    /// `CompSolid`.
    pub fn comp_solid(&self) -> Result<CompSolid, Failure> {
        topods::comp_solid(&self.result())
    }
    /// `Compound`.
    pub fn compound(&self) -> Result<Compound, Failure> {
        topods::compound(&self.result())
    }
}

/// `TransferBRep_ShapeListBinder`: a list of shape results (`None` is the native null
/// list of the constructor from a null sequence).
///
/// ```
/// use rustycad_tkxsbase::transfer_brep::TransferBRepShapeListBinder;
/// use rustycad_tkbrep::topods::Shape;
/// let list = TransferBRepShapeListBinder::new();
/// list.add_result(&Shape::default()).unwrap();
/// assert_eq!(list.nb_shapes().unwrap(), 1);
/// ```
pub struct TransferBRepShapeListBinder {
    transient: StandardTransient,
    base: TransferBinderBase,
    theres: SharedState<Option<Vec<Shape>>>,
}
impl_transient!(TransferBRepShapeListBinder, "TransferBRep_ShapeListBinder", dyn TransferBinder, transient, interfaces: [dyn TransferBinder]);

impl Default for TransferBRepShapeListBinder {
    fn default() -> Self {
        Self::new()
    }
}

/// The native null dereference of a null list.
fn null_list(member: &str) -> Failure {
    exceptions::null_object(&format!(
        "TransferBRep_ShapeListBinder::{member}: null list; expected a list of shapes"
    ))
}

impl TransferBRepShapeListBinder {
    /// `TransferBRep_ShapeListBinder()`: an empty list.
    pub fn new() -> Self {
        Self::with_list(Some(Vec::new()))
    }
    /// `TransferBRep_ShapeListBinder(list)`: `None` is a null list.
    pub fn with_list(list: Option<Vec<Shape>>) -> Self {
        Self {
            transient: StandardTransient::default(),
            base: TransferBinderBase::default(),
            theres: SharedState::new(list),
        }
    }
    /// `AddResult(res)`.
    pub fn add_result(&self, res: &Shape) -> Result<(), Failure> {
        self.theres
            .write()
            .as_mut()
            .ok_or_else(|| null_list("AddResult"))?
            .push(res.clone());
        Ok(())
    }
    /// `Result`: a copy of the list (`None` when null).
    pub fn result(&self) -> Option<Vec<Shape>> {
        self.theres.read().clone()
    }
    /// `SetResult(num, res)`: `Standard_OutOfRange` out of the list.
    pub fn set_result(&self, num: i32, res: &Shape) -> Result<(), Failure> {
        let mut st = self.theres.write();
        let list = st.as_mut().ok_or_else(|| null_list("SetResult"))?;
        let len = list.len();
        let slot = usize::try_from(num - 1).ok().and_then(|i| list.get_mut(i));
        *slot.ok_or_else(|| {
            exceptions::out_of_range(&format!(
                "TransferBRep_ShapeListBinder::SetResult: {num}; expected [1, {len}]"
            ))
        })? = res.clone();
        Ok(())
    }
    /// `NbShapes`.
    pub fn nb_shapes(&self) -> Result<i32, Failure> {
        Ok(self
            .theres
            .read()
            .as_ref()
            .ok_or_else(|| null_list("NbShapes"))?
            .len() as i32)
    }
    /// `Shape(num)`: `Standard_OutOfRange` out of the list.
    pub fn shape(&self, num: i32) -> Result<Shape, Failure> {
        let st = self.theres.read();
        let list = st.as_ref().ok_or_else(|| null_list("Shape"))?;
        let found = usize::try_from(num - 1).ok().and_then(|i| list.get(i));
        found.cloned().ok_or_else(|| {
            exceptions::out_of_range(&format!(
                "TransferBRep_ShapeListBinder::Shape: {num}; expected [1, {}]",
                list.len()
            ))
        })
    }
    /// `ShapeType(num)`.
    pub fn shape_type(&self, num: i32) -> Result<ShapeEnum, Failure> {
        self.shape(num)?.shape_type()
    }
    /// `Vertex(num)`.
    pub fn vertex(&self, num: i32) -> Result<Vertex, Failure> {
        topods::vertex(&self.shape(num)?)
    }
    /// `Edge(num)`.
    pub fn edge(&self, num: i32) -> Result<Edge, Failure> {
        topods::edge(&self.shape(num)?)
    }
    /// `Wire(num)`.
    pub fn wire(&self, num: i32) -> Result<Wire, Failure> {
        topods::wire(&self.shape(num)?)
    }
    /// `Face(num)`.
    pub fn face(&self, num: i32) -> Result<Face, Failure> {
        topods::face(&self.shape(num)?)
    }
    /// `Shell(num)`.
    pub fn shell(&self, num: i32) -> Result<Shell, Failure> {
        topods::shell(&self.shape(num)?)
    }
    /// `Solid(num)`.
    pub fn solid(&self, num: i32) -> Result<Solid, Failure> {
        topods::solid(&self.shape(num)?)
    }
    /// `CompSolid(num)`.
    pub fn comp_solid(&self, num: i32) -> Result<CompSolid, Failure> {
        topods::comp_solid(&self.shape(num)?)
    }
    /// `Compound(num)`.
    pub fn compound(&self, num: i32) -> Result<Compound, Failure> {
        topods::compound(&self.shape(num)?)
    }
}

impl TransferBinder for TransferBRepShapeListBinder {
    fn binder_base(&self) -> &TransferBinderBase {
        &self.base
    }
    /// `IsMultiple`: more than one shape (a null list is natively dereferenced: false).
    fn is_multiple(&self) -> bool {
        self.nb_shapes().is_ok_and(|n| n > 1)
    }
    /// `ResultType`: `TransferBRep_ShapeListBinder` itself.
    fn result_type(&self) -> &'static RuntimeType {
        Self::static_type()
    }
    /// `ResultTypeName`: "list(TopoDS_Shape)".
    fn result_type_name(&self) -> String {
        String::from("list(TopoDS_Shape)")
    }
}
