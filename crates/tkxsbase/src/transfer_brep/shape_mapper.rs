// Rust port of DataExchange/TKXSBase/TransferBRep/TransferBRep_ShapeMapper (.hxx, _0.cxx),
// TransferBRep_ShapeInfo and TransferBRep_TransferResultInfo (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TransferBRep_ShapeMapper`: a finder on a shape (hashed and compared as
//! `TopTools_ShapeMapHasher` does); `TransferBRep_ShapeInfo`: the type of a shape result;
//! `TransferBRep_TransferResultInfo`: counts of transfer results.
use crate::transfer::{TransferFinder, TransferFinderBase};
use rustycad_tkbrep::{
    topods::{Shape, TShape},
    toptools::ShapeMapHasher,
};
use rustycad_tkernel::{
    impl_transient,
    ncollection::HashPolicy,
    standard::{Handle, RuntimeType, RuntimeTyped, SharedState, StandardTransient},
};
use rustycad_tkg3d::top_abs::ShapeEnum;
use std::sync::RwLockWriteGuard;

/// `TransferBRep_ShapeInfo`: the type of a shape result.
pub struct TransferBRepShapeInfo;

impl TransferBRepShapeInfo {
    /// `Type(ent)`: `TopoDS_TShape` whatever the shape.
    pub fn type_of(_ent: &Shape) -> &'static RuntimeType {
        <dyn TShape as RuntimeTyped>::static_type()
    }
    /// `TypeName(ent)`: the name of the typed shape ("TopoDS_Shape" for null or `SHAPE`).
    pub fn type_name(ent: &Shape) -> &'static str {
        match ent.shape_type() {
            Ok(ShapeEnum::Vertex) => "TopoDS_Vertex",
            Ok(ShapeEnum::Edge) => "TopoDS_Edge",
            Ok(ShapeEnum::Wire) => "TopoDS_Wire",
            Ok(ShapeEnum::Face) => "TopoDS_Face",
            Ok(ShapeEnum::Shell) => "TopoDS_Shell",
            Ok(ShapeEnum::Solid) => "TopoDS_Solid",
            Ok(ShapeEnum::CompSolid) => "TopoDS_CompSolid",
            Ok(ShapeEnum::Compound) => "TopoDS_Compound",
            _ => "TopoDS_Shape",
        }
    }
}

/// `TransferBRep_ShapeMapper`.
///
/// ```
/// use rustycad_tkxsbase::transfer::TransferFinder;
/// use rustycad_tkxsbase::transfer_brep::TransferBRepShapeMapper;
/// use rustycad_tkbrep::topods::Shape;
/// let mapper = TransferBRepShapeMapper::new(&Shape::default());
/// assert_eq!(mapper.value_type_name(), "TopoDS_Shape");
/// ```
pub struct TransferBRepShapeMapper {
    transient: StandardTransient,
    base: TransferFinderBase,
    theval: Shape,
}
impl_transient!(TransferBRepShapeMapper, "TransferBRep_ShapeMapper", dyn TransferFinder, transient, interfaces: [dyn TransferFinder]);

impl TransferBRepShapeMapper {
    /// `TransferBRep_ShapeMapper(akey)`: the hash code of `TopTools_ShapeMapHasher`.
    pub fn new(akey: &Shape) -> Self {
        let mapper = Self {
            transient: StandardTransient::default(),
            base: TransferFinderBase::default(),
            theval: akey.clone(),
        };
        mapper.set_hash_code(ShapeMapHasher.hash(akey));
        mapper
    }
    /// `Value`.
    pub fn value(&self) -> &Shape {
        &self.theval
    }
}

impl TransferFinder for TransferBRepShapeMapper {
    fn finder_base(&self) -> &TransferFinderBase {
        &self.base
    }
    /// `Equates(other)`: same hash code, same type, same shape (`IsSame`).
    fn equates(&self, other: &Handle<dyn TransferFinder>) -> bool {
        let Some(live) = other.get() else {
            return false;
        };
        if self.get_hash_code() != live.get_hash_code()
            || !std::ptr::eq(live.dynamic_type(), Self::static_type())
        {
            return false;
        }
        other
            .cast::<TransferBRepShapeMapper>()
            .get()
            .is_some_and(|m| ShapeMapHasher.equal(&self.theval, &m.theval))
    }
    /// `ValueType`: `TransferBRep_ShapeInfo::Type`.
    fn value_type(&self) -> &'static RuntimeType {
        TransferBRepShapeInfo::type_of(&self.theval)
    }
    /// `ValueTypeName`: `TransferBRep_ShapeInfo::TypeName`.
    fn value_type_name(&self) -> String {
        TransferBRepShapeInfo::type_name(&self.theval).to_owned()
    }
}

/// `TransferBRep_TransferResultInfo`: the counts of results with or without warnings and
/// fails, and of no results; each accessor is the native modifiable reference (a write
/// guard).
///
/// ```
/// use rustycad_tkxsbase::transfer_brep::TransferBRepTransferResultInfo;
/// let info = TransferBRepTransferResultInfo::new();
/// *info.result() += 2;
/// info.clear();
/// assert_eq!(*info.result(), 0);
/// ```
#[derive(Default)]
pub struct TransferBRepTransferResultInfo {
    transient: StandardTransient,
    counts: [SharedState<i32>; 8],
}
impl_transient!(
    TransferBRepTransferResultInfo,
    "TransferBRep_TransferResultInfo",
    StandardTransient,
    transient
);

impl TransferBRepTransferResultInfo {
    /// `TransferBRep_TransferResultInfo()`: every count 0.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Clear`: every count 0.
    pub fn clear(&self) {
        self.counts.iter().for_each(|c| *c.write() = 0);
    }
    /// `Result`.
    pub fn result(&self) -> RwLockWriteGuard<'_, i32> {
        self.counts[0].write()
    }
    /// `ResultWarning`.
    pub fn result_warning(&self) -> RwLockWriteGuard<'_, i32> {
        self.counts[1].write()
    }
    /// `ResultFail`.
    pub fn result_fail(&self) -> RwLockWriteGuard<'_, i32> {
        self.counts[2].write()
    }
    /// `ResultWarningFail`.
    pub fn result_warning_fail(&self) -> RwLockWriteGuard<'_, i32> {
        self.counts[3].write()
    }
    /// `NoResult`.
    pub fn no_result(&self) -> RwLockWriteGuard<'_, i32> {
        self.counts[4].write()
    }
    /// `NoResultWarning`.
    pub fn no_result_warning(&self) -> RwLockWriteGuard<'_, i32> {
        self.counts[5].write()
    }
    /// `NoResultFail`.
    pub fn no_result_fail(&self) -> RwLockWriteGuard<'_, i32> {
        self.counts[6].write()
    }
    /// `NoResultWarningFail`.
    pub fn no_result_warning_fail(&self) -> RwLockWriteGuard<'_, i32> {
        self.counts[7].write()
    }
    /// The eight counts in the order of the accessors (a snapshot).
    pub(super) fn snapshot(&self) -> [i32; 8] {
        std::array::from_fn(|i| *self.counts[i].read())
    }
    /// Adds one to the count of rank `rank` (the order of the accessors).
    pub(super) fn bump(&self, rank: usize) {
        *self.counts[rank].write() += 1;
    }
}
