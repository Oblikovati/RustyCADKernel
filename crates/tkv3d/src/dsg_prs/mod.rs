//! Martensite ports of OCCT dimension and datum presentation builders (DsgPrs at 3d097a0).
//!
//! Each `DsgPrs_*Presentation` class is a unit struct whose static `Add` overloads are named
//! after what distinguishes them (`add`, `add_with_arrow`, `add_arc`...). The package class
//! [`DsgPrs`](crate::dsg_prs::DsgPrs) holds the shared layout computations; native output arguments become returned
//! values or `&mut` structs. A null presentation, drawer or aspect (a native null
//! dereference) is a `Standard_NullObject` failure.
mod xyz_axis;
pub use xyz_axis::DsgPrsXYZAxisPresentation;

mod plane;
pub use plane::{DsgPrsShadedPlanePresentation, DsgPrsXYZPlanePresentation};

mod angle;
mod dimension_line;
mod package;
mod radius;
mod relations;
mod shape_dir;
mod simple;
mod support;
mod symbol;
mod symmetric;

pub use angle::{DsgPrsAnglePresentation, DsgPrsConeAngle};
pub use dimension_line::{
    DsgPrsLengthPresentation, DsgPrsOffsetPresentation, DsgPrsParalPresentation,
};
pub use package::{
    DsgPrs, DsgPrsCurvilinearLength, DsgPrsFacesAngle, DsgPrsFacesAngleInputs, DsgPrsFillet,
    DsgPrsFilletInputs, DsgPrsPlanarLength,
};
pub use radius::{
    DsgPrsDiameterPresentation, DsgPrsEllipseRadiusPresentation, DsgPrsFilletOutputs,
    DsgPrsFilletRadiusPresentation, DsgPrsRadiusPresentation,
};
pub use relations::{
    DsgPrsEqualDistancePresentation, DsgPrsIdenticPresentation, DsgPrsMidPointPresentation,
};
pub use shape_dir::{
    DsgPrsShapeDirPresentation, DsgPrsSymbPresentation, DsgPrsTangentPresentation,
};
pub use simple::{
    DsgPrsChamf2dPresentation, DsgPrsConcentricPresentation, DsgPrsDatumPrs,
    DsgPrsEqualRadiusPresentation, DsgPrsFixPresentation, DsgPrsPerpenPresentation,
};
pub use support::DsgPrsPresentation;
pub use symbol::{DsgPrsArrowSide, DsgPrsSymbolEnds, dsg_prs_compute_symbol};
pub use symmetric::DsgPrsSymmetricPresentation;
