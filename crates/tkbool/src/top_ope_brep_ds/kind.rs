// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepDS/TopOpeBRepDS_Kind.hxx,
// TopOpeBRepDS_Config.hxx and TopOpeBRepDS_CheckStatus.hxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The enumerations of the topological data structure: the kinds of its entities, the
//! orientation of same domain shapes and the status of a check.

/// The kind of an entity of the data structure (`TopOpeBRepDS_Kind`): a geometry
/// (point, curve, surface) or a shape type; `TopOpeBRepDSKind::Edge as i32 == 4`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(i32)]
pub enum TopOpeBRepDSKind {
    /// `TopOpeBRepDS_POINT` (also the value of the zeroed storage natively).
    #[default]
    Point,
    /// `TopOpeBRepDS_CURVE`.
    Curve,
    /// `TopOpeBRepDS_SURFACE`.
    Surface,
    /// `TopOpeBRepDS_VERTEX`.
    Vertex,
    /// `TopOpeBRepDS_EDGE`.
    Edge,
    /// `TopOpeBRepDS_WIRE`.
    Wire,
    /// `TopOpeBRepDS_FACE`.
    Face,
    /// `TopOpeBRepDS_SHELL`.
    Shell,
    /// `TopOpeBRepDS_SOLID`.
    Solid,
    /// `TopOpeBRepDS_COMPSOLID`.
    CompSolid,
    /// `TopOpeBRepDS_COMPOUND`.
    Compound,
    /// `TopOpeBRepDS_UNKNOWN`.
    Unknown,
}

/// The orientation of a shape relative to its same domain reference
/// (`TopOpeBRepDS_Config`); `TopOpeBRepDSConfig::SameOriented as i32 == 1`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum TopOpeBRepDSConfig {
    /// `TopOpeBRepDS_UNSHGEOMETRY`: no shared geometry.
    #[default]
    UnshGeometry,
    /// `TopOpeBRepDS_SAMEORIENTED`.
    SameOriented,
    /// `TopOpeBRepDS_DIFFORIENTED`.
    DiffOriented,
}

/// The status of a data structure check (`TopOpeBRepDS_CheckStatus`);
/// `TopOpeBRepDSCheckStatus::Ok as i32 == 0`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum TopOpeBRepDSCheckStatus {
    /// `TopOpeBRepDS_OK`.
    #[default]
    Ok,
    /// `TopOpeBRepDS_NOK`.
    Nok,
}
