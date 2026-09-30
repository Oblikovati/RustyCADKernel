//! Safe non-owning point associations and the native shared collection aliases.
use crate::{
    imesh_data::{EdgePtr, MeshPCurve},
    imesh_tools::MeshParameters,
};
use rustycad_tkernel::{
    ncollection::{Array1, DynamicArray, EBTree, Map, Shared},
    standard::{Failure, Handle, SharedState, WeakHandle},
};
use rustycad_tkmath::{bnd::BoundingBox2, gp::Point2};
/// Live const-reference parameters with an explicit Rust owner; `Handle::new(Shared::new(SharedState::new(MeshParameters::default())))`.
pub type FaceCheckerParameters = Handle<Shared<SharedState<MeshParameters>>>;
/// Native non-owning pointer to a pcurve sample slot; `CurvePointReference::new(curve.downgrade(),0)`.
#[derive(Clone, PartialEq, Eq)]
pub struct CurvePointReference {
    curve: WeakHandle<dyn MeshPCurve>,
    index: i32,
}
impl CurvePointReference {
    /// Retain the sample association without retaining its curve; `CurvePointReference::new(curve.downgrade(),0)`.
    pub fn new(curve: WeakHandle<dyn MeshPCurve>, index: i32) -> Self {
        Self { curve, index }
    }
    /// Read the current sample value; `reference.get()?`.
    pub fn get(&self) -> Result<Point2, Failure> {
        self.curve.upgrade().get_point(self.index)
    }
    /// Write through the point association; `reference.set(Point2::default())?`.
    pub fn set(&self, point: Point2) -> Result<(), Failure> {
        self.curve.upgrade().set_point(self.index, point)
    }
}
/// Native FaceChecker::Segment with nullable, live point references; `FaceSegment::default()`.
#[derive(Clone, Default)]
pub struct FaceSegment {
    /// Native non-owning edge pointer.
    pub edge_ptr: EdgePtr,
    /// First native point pointer, null in the default constructor.
    pub point1: Option<CurvePointReference>,
    /// Second native point pointer, null in the default constructor.
    pub point2: Option<CurvePointReference>,
}
impl FaceSegment {
    /// Associate an edge and its endpoint slots; `FaceSegment::new(edge,Some(first),Some(last))`.
    pub fn new(
        edge_ptr: EdgePtr,
        point1: Option<CurvePointReference>,
        point2: Option<CurvePointReference>,
    ) -> Self {
        Self {
            edge_ptr,
            point1,
            point2,
        }
    }
    pub(super) fn endpoints(&self) -> Result<(Point2, Point2), Failure> {
        let first = self.point1.as_ref().ok_or_else(|| {
            Failure::with_type(
                "Standard_NullObject",
                "Point1 is null; expected a live pcurve point",
                "",
            )
        })?;
        let second = self.point2.as_ref().ok_or_else(|| {
            Failure::with_type(
                "Standard_NullObject",
                "Point2 is null; expected a live pcurve point",
                "",
            )
        })?;
        Ok((first.get()?, second.get()?))
    }
    pub(super) fn second_point(&self) -> Result<Point2, Failure> {
        self.point2
            .as_ref()
            .ok_or_else(|| {
                Failure::with_type(
                    "Standard_NullObject",
                    "Point2 is null; expected a live pcurve point",
                    "",
                )
            })?
            .get()
    }
    pub(super) fn bounds(&self) -> Result<BoundingBox2, Failure> {
        let (first, last) = self.endpoints()?;
        let mut bounds = BoundingBox2::default();
        bounds.add_point(first);
        bounds.add_point(last);
        bounds.enlarge(rustycad_tkernel::precision::confusion());
        Ok(bounds)
    }
}
/// Native shared discrete segments; `FaceSegments::default()`.
pub type FaceSegments = Shared<SharedState<DynamicArray<FaceSegment>>>;
/// Native zero-based shared array of segment vectors; `FaceSegmentArray::default()`.
pub type FaceSegmentArray = Shared<SharedState<Array1<'static, Handle<FaceSegments>>>>;
/// Native shared editable planar tree; `FaceBoundsTree::default()`.
pub type FaceBoundsTree = Shared<SharedState<EBTree<i32, BoundingBox2>>>;
/// Native zero-based array of planar trees; `FaceBoundsTreeArray::default()`.
pub type FaceBoundsTreeArray = Shared<SharedState<Array1<'static, Handle<FaceBoundsTree>>>>;
/// Shared native pointer set; `FaceEdgeSet::default()`.
pub type FaceEdgeSet = Shared<SharedState<Map<EdgePtr>>>;
/// Native zero-based array of nullable intersection sets; `FaceEdgeSetArray::default()`.
pub type FaceEdgeSetArray = Shared<SharedState<Array1<'static, Handle<FaceEdgeSet>>>>;
