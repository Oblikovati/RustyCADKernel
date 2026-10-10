//! `StdPrs_ToolPoint` and `StdPrs_Point` at OCCT 3d097a0 use live Geom_Point coordinates.
use crate::prs3d::{Prs3dPoint, Prs3dPointTool};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::GeomPoint;
/// Coordinate adapter for geometric point presentations; `StdPrsToolPoint::coord(&point)`.
pub struct StdPrsToolPoint;
impl StdPrsToolPoint {
    /// Read the live component; `let (x, y, z) = StdPrsToolPoint::coord(&point)`.
    pub fn coord(point: &Handle<dyn GeomPoint>) -> (f64, f64, f64) {
        (point.x(), point.y(), point.z())
    }
}
impl Prs3dPointTool<Handle<dyn GeomPoint>> for StdPrsToolPoint {
    fn coord(point: &Handle<dyn GeomPoint>) -> (f64, f64, f64) {
        Self::coord(point)
    }
}
/// Standard geometric point presentation; `StdPrsPoint::add(&prs, &point, &drawer)?`.
pub type StdPrsPoint = Prs3dPoint<Handle<dyn GeomPoint>, StdPrsToolPoint>;
