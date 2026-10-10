//! Real line geometry and isolated units for CPU, native differential and GPU fixtures.
#[path = "../../../tkernel/tests/units_support/mod.rs"]
mod units_support;
use rustycad_tkernel::{
    standard::{Handle, SharedState},
    units_api::UnitsApi,
};
use rustycad_tkg3d::geom::{GeomCartesianPoint, GeomLine, GeomPoint};
use rustycad_tkmath::gp::{Axis1, Direction3, Point3};
use rustycad_tkv3d::ais::AisLine;
use std::sync::Arc;
pub fn units() -> Arc<SharedState<UnitsApi>> {
    Arc::new(SharedState::new(units_support::api()))
}
pub fn endpoints() -> (Handle<dyn GeomPoint>, Handle<dyn GeomPoint>) {
    (
        Handle::new(GeomCartesianPoint::new(-3., -2., 0.)).cast(),
        Handle::new(GeomCartesianPoint::new(3., 2., 0.)).cast(),
    )
}
pub fn component() -> Handle<GeomLine> {
    Handle::new(GeomLine::new(Axis1::new(
        Point3::new(-3., -2., 0.),
        Direction3::new(3., 2., 0.).unwrap(),
    )))
}
pub fn object(segment: bool, units: Arc<SharedState<UnitsApi>>) -> Handle<AisLine> {
    if segment {
        let (a, b) = endpoints();
        return AisLine::from_points(&a, &b, units);
    }
    AisLine::new(&component(), units)
}
