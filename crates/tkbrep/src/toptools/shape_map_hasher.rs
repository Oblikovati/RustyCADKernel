// Rust translation of ModelingData/TKBRep/TopTools/TopTools_ShapeMapHasher.hxx and
// TopTools_FormatVersion.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The shape hasher of the TopTools maps and the TopTools format versions.
use crate::topods::Shape;
use rustycad_tkernel::ncollection::{HashPolicy, IndexedDataMap, IndexedMap, List, Map};

/// `TopTools_ShapeMapHasher`: hashes a shape with `std::hash<TopoDS_Shape>` and
/// compares with `IsSame`, so that the orientation is ignored.
///
/// ```
/// use rustycad_tkbrep::topods::Shape;
/// use rustycad_tkbrep::toptools::ShapeMapHasher;
/// use rustycad_tkernel::ncollection::HashPolicy;
/// let null = Shape::default();
/// assert!(ShapeMapHasher.equal(&null, &null.reversed()));
/// assert_eq!(ShapeMapHasher.hash(&null), null.hash_code());
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct ShapeMapHasher;

impl HashPolicy<Shape> for ShapeMapHasher {
    /// `operator()(theShape)`.
    fn hash(&self, key: &Shape) -> usize {
        key.hash_code()
    }
    /// `operator()(S1, S2)`: `S1.IsSame(S2)`.
    fn equal(&self, first: &Shape, second: &Shape) -> bool {
        first.is_same(second)
    }
}

/// `NCollection_IndexedMap<TopoDS_Shape, TopTools_ShapeMapHasher>`.
pub type IndexedMapOfShape = IndexedMap<Shape, ShapeMapHasher>;
/// `NCollection_Map<TopoDS_Shape, TopTools_ShapeMapHasher>`.
pub type MapOfShape = Map<Shape, ShapeMapHasher>;
/// `NCollection_IndexedDataMap<TopoDS_Shape, NCollection_List<TopoDS_Shape>, TopTools_ShapeMapHasher>`.
pub type IndexedDataMapOfShapeListOfShape = IndexedDataMap<Shape, List<Shape>, ShapeMapHasher>;

/// `TopTools_FormatVersion`: the versions of the TopTools ASCII format.
pub mod format_version {
    /// `TopTools_FormatVersion_VERSION_1`: no CurveOnSurface UV points; reading calls `Check`.
    pub const VERSION_1: i32 = 1;
    /// `TopTools_FormatVersion_VERSION_2`: stores the CurveOnSurface UV points.
    pub const VERSION_2: i32 = 2;
    /// `TopTools_FormatVersion_VERSION_3`: stores per-vertex normals of triangulation-only faces.
    pub const VERSION_3: i32 = 3;
    /// `TopTools_FormatVersion_CURRENT`.
    pub const CURRENT: i32 = VERSION_3;
    /// `TopTools_FormatVersion_LOWER`.
    pub const LOWER: i32 = VERSION_1;
    /// `TopTools_FormatVersion_UPPER`.
    pub const UPPER: i32 = VERSION_3;
}
