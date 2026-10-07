// Rust port of ApplicationFramework/TKCAF/TDataXtd/TDataXtd.hxx, TDataXtd.cxx,
// TDataXtd_ConstraintEnum.hxx and TDataXtd_GeometryEnum.hxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TDataXtd`: the package functions and the enumerations of the extended attributes.
use super::{
    TDataXtdAxis, TDataXtdConstraint, TDataXtdGeometry, TDataXtdPatternStd, TDataXtdPlacement,
    TDataXtdPlane, TDataXtdPoint, TDataXtdPosition, TDataXtdShape,
};
use rustycad_tkernel::standard::{Guid, Handle};
use rustycad_tklcaf::tdf::{TdfAttribute, TdfDerivedAttribute};
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};

/// `TDataXtd_GeometryEnum`, in native order.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub enum TDataXtdGeometryEnum {
    /// `TDataXtd_ANY_GEOM`.
    #[default]
    AnyGeom,
    /// `TDataXtd_POINT`.
    Point,
    /// `TDataXtd_LINE`.
    Line,
    /// `TDataXtd_CIRCLE`.
    Circle,
    /// `TDataXtd_ELLIPSE`.
    Ellipse,
    /// `TDataXtd_SPLINE`.
    Spline,
    /// `TDataXtd_PLANE`.
    Plane,
    /// `TDataXtd_CYLINDER`.
    Cylinder,
}

/// `TDataXtd_ConstraintEnum`, in native order (2d planar constraints, then placement
/// constraints).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub enum TDataXtdConstraintEnum {
    /// `TDataXtd_RADIUS`.
    #[default]
    Radius,
    /// `TDataXtd_DIAMETER`.
    Diameter,
    /// `TDataXtd_MINOR_RADIUS`.
    MinorRadius,
    /// `TDataXtd_MAJOR_RADIUS`.
    MajorRadius,
    /// `TDataXtd_TANGENT`.
    Tangent,
    /// `TDataXtd_PARALLEL`.
    Parallel,
    /// `TDataXtd_PERPENDICULAR`.
    Perpendicular,
    /// `TDataXtd_CONCENTRIC`.
    Concentric,
    /// `TDataXtd_COINCIDENT`.
    Coincident,
    /// `TDataXtd_DISTANCE`.
    Distance,
    /// `TDataXtd_ANGLE`.
    Angle,
    /// `TDataXtd_EQUAL_RADIUS`.
    EqualRadius,
    /// `TDataXtd_SYMMETRY`.
    Symmetry,
    /// `TDataXtd_MIDPOINT`.
    Midpoint,
    /// `TDataXtd_EQUAL_DISTANCE`.
    EqualDistance,
    /// `TDataXtd_FIX`.
    Fix,
    /// `TDataXtd_RIGID`.
    Rigid,
    /// `TDataXtd_FROM`.
    From,
    /// `TDataXtd_AXIS`.
    Axis,
    /// `TDataXtd_MATE`.
    Mate,
    /// `TDataXtd_ALIGN_FACES`.
    AlignFaces,
    /// `TDataXtd_ALIGN_AXES`.
    AlignAxes,
    /// `TDataXtd_AXES_ANGLE`.
    AxesAngle,
    /// `TDataXtd_FACES_ANGLE`.
    FacesAngle,
    /// `TDataXtd_ROUND`.
    Round,
    /// `TDataXtd_OFFSET`.
    Offset,
}

/// The constraint names of `TDataXtd::Print`, in enumeration order.
const CONSTRAINT_NAMES: [&str; 26] = [
    "RADIUS",
    "DIAMETER",
    "MINOR_RADIUS",
    "MAJOR_RADIUS",
    "TANGENT",
    "PARALLEL",
    "PERPENDICULAR",
    "CONCENTRIC",
    "COINCIDENT",
    "DISTANCE",
    "ANGLE",
    "EQUAL_RADIUS",
    "SYMMETRY",
    "MIDPOINT",
    "EQUAL_DISTANCE",
    "FIX",
    "RIGID",
    "FROM",
    "AXIS",
    "MATE",
    "ALIGN_FACES",
    "ALIGN_AXES",
    "AXES_ANGLE",
    "FACES_ANGLE",
    "ROUND",
    "OFFSET",
];

/// The geometry names of `TDataXtd::Print`, in enumeration order.
const GEOMETRY_NAMES: [&str; 8] = [
    "ANY_GEOM", "POINT", "LINE", "CIRCLE", "ELLIPSE", "SPLINE", "PLANE", "CYLINDER",
];

/// `TDataXtd`: package functions.
///
/// ```
/// use rustycad_tkcaf::tdata_xtd::{TDataXtd, TDataXtdConstraintEnum};
/// let mut text = Vec::new();
/// TDataXtd::print_constraint(TDataXtdConstraintEnum::AlignFaces, &mut text).unwrap();
/// assert_eq!(text, b"ALIGN_FACES");
/// ```
pub struct TDataXtd;

impl TDataXtd {
    /// `IDList(anIDList)`: the IDs of the TDataXtd attributes (not Presentation or
    /// Triangulation), in the native order.
    pub fn id_list(ids: &mut Vec<Guid>) {
        ids.extend([
            TDataXtdAxis::get_id(),
            TDataXtdConstraint::get_id(),
            TDataXtdGeometry::get_id(),
            TDataXtdPatternStd::get_id(),
            TDataXtdPlacement::get_id(),
            TDataXtdPoint::get_id(),
            TDataXtdPlane::get_id(),
            TDataXtdPosition::get_id(),
            TDataXtdShape::get_id(),
        ]);
    }
    /// `Print(GEO, S)`: the name of a geometry type.
    pub fn print_geometry(
        geometry: TDataXtdGeometryEnum,
        stream: &mut dyn io::Write,
    ) -> io::Result<()> {
        write!(stream, "{}", GEOMETRY_NAMES[geometry as usize])
    }
    /// `Print(CTR, S)`: the name of a constraint type.
    pub fn print_constraint(
        constraint: TDataXtdConstraintEnum,
        stream: &mut dyn io::Write,
    ) -> io::Result<()> {
        write!(stream, "{}", CONSTRAINT_NAMES[constraint as usize])
    }
    /// The `IMPLEMENT_DERIVED_ATTRIBUTE` registrations of TKCAF (Shape, Axis, Point, Plane,
    /// Placement, in the native link order). Natively the library load registers them; the
    /// port registers them once, when the first of these attributes is created.
    pub fn register_derived_attributes() {
        // A flag, not a `Once`: the registry builds its prototypes with these constructors,
        // which call back here while the registration may still be running.
        static REGISTERED: AtomicBool = AtomicBool::new(false);
        if REGISTERED.swap(true, Ordering::SeqCst) {
            return;
        }
        let creators: [fn() -> Handle<dyn TdfAttribute>; 5] = [
            || Handle::new(TDataXtdShape::new()).cast(),
            || Handle::new(TDataXtdAxis::new()).cast(),
            || Handle::new(TDataXtdPoint::new()).cast(),
            || Handle::new(TDataXtdPlane::new()).cast(),
            || Handle::new(TDataXtdPlacement::new()).cast(),
        ];
        for create in creators {
            TdfDerivedAttribute::register(create, None, None);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_names_follow_the_enumerations() {
        let mut text = Vec::new();
        TDataXtd::print_geometry(TDataXtdGeometryEnum::Cylinder, &mut text).unwrap();
        TDataXtd::print_constraint(TDataXtdConstraintEnum::Offset, &mut text).unwrap();
        TDataXtd::print_constraint(TDataXtdConstraintEnum::Radius, &mut text).unwrap();
        assert_eq!(text, b"CYLINDEROFFSETRADIUS");
        let mut ids = Vec::new();
        TDataXtd::id_list(&mut ids);
        assert_eq!(ids.len(), 9);
        assert_eq!(ids[0], TDataXtdAxis::get_id());
    }
}
