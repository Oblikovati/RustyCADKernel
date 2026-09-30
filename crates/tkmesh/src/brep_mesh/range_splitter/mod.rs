//! Native parametric ranges and physical-length scaling (OCCT 3d097a03).
use crate::{imesh_data::FaceHandle, imesh_tools::MeshParameters};
use rustycad_tkbrep::{brep::BRepTool, brep_adaptor::BRepAdaptorSurface};
use rustycad_tkernel::{
    ncollection::{List, Shared},
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkmath::gp::{Point2, Point3};
mod cone;
mod interface;
pub use interface::RangeSplitter;
mod sampling;
pub use cone::ConeRangeSplitter;
/// Allocator-backed shared surface nodes; `SurfaceNodes::default()`.
pub type SurfaceNodes = Handle<Shared<List<Point2>>>;
/// Default surface range tool; `DefaultRangeSplitter::default()`.
pub struct DefaultRangeSplitter {
    /// Selected face; writable for native derived splitter implementations.
    pub face: FaceHandle,
    /// U domain retained by native derived splitters.
    pub range_u: (f64, f64),
    /// V domain retained by native derived splitters.
    pub range_v: (f64, f64),
    /// Physical scaling overridden by derived splitter hooks.
    pub delta: (f64, f64),
    /// Parametric tolerances overridden by derived splitter hooks.
    pub tolerance: (f64, f64),
    /// Validity retained across Reset, as in OCCT 3d097a03.
    pub valid: bool,
}
impl Default for DefaultRangeSplitter {
    fn default() -> Self {
        Self {
            face: FaceHandle::default(),
            range_u: (0.0, 0.0),
            range_v: (0.0, 0.0),
            delta: (0.0, 0.0),
            tolerance: (0.0, 0.0),
            valid: true,
        }
    }
}
impl DefaultRangeSplitter {
    /// Begin collecting a face border; `splitter.reset(&face, &parameters)`.
    pub fn reset(&mut self, face: &FaceHandle, _parameters: &MeshParameters) {
        self.face = face.clone();
        self.range_u = (1.0e100, -1.0e100);
        self.range_v = self.range_u;
        self.delta = (1.0, 1.0);
        self.tolerance = (precision::confusion(), precision::confusion());
    }
    /// Extend the discrete bounds; `splitter.add_point(Point2::new(0.0,1.0))`.
    pub fn add_point(&mut self, point: Point2) {
        self.range_u = (
            std_min(point.x(), self.range_u.0),
            std_max(point.x(), self.range_u.1),
        );
        self.range_v = (
            std_min(point.y(), self.range_v.0),
            std_max(point.y(), self.range_v.1),
        );
    }
    /// Clamp overlapping domains and compute scaling; `splitter.adjust_range()?`.
    pub fn adjust_range(&mut self) -> Result<(), Failure> {
        RangeSplitter::adjust_range(self)
    }
    fn adjust_lengths(&mut self) -> Result<Option<(f64, f64)>, Failure> {
        let surface = self.get_surface();
        if !self.adjust_bounds(&surface)? {
            self.valid = false;
            return Ok(None);
        }
        let length_u = sampling::sample_length(&surface, self.range_u, self.range_v, true)?;
        let length_v = sampling::sample_length(&surface, self.range_v, self.range_u, false)?;
        self.valid = length_u > precision::p_confusion() && length_v > precision::p_confusion();
        Ok(self.valid.then_some((length_u, length_v)))
    }
    /// Report the last adjustment's validity; `splitter.is_valid()`.
    pub fn is_valid(&self) -> bool {
        self.valid
    }
    /// Convert parametric coordinates to the physical face basis or back; `splitter.scale(point,true)`.
    pub fn scale(&self, point: Point2, to_face_basis: bool) -> Point2 {
        if to_face_basis {
            return Point2::new(
                (point.x() - self.range_u.0) / self.delta.0,
                (point.y() - self.range_v.0) / self.delta.1,
            );
        }
        Point2::new(
            point.x() * self.delta.0 + self.range_u.0,
            point.y() * self.delta.1 + self.range_v.0,
        )
    }
    /// The default tool creates no interior nodes; `splitter.generate_surface_nodes(&parameters)`.
    pub fn generate_surface_nodes(
        &self,
        _parameters: &MeshParameters,
    ) -> Result<SurfaceNodes, Failure> {
        Ok(SurfaceNodes::default())
    }
    /// Evaluate the original surface; `splitter.point(Point2::new(0.0,1.0))?`.
    pub fn point(&self, point: Point2) -> Result<Point3, Failure> {
        self.get_surface().value(point.x(), point.y())
    }
    /// Retain the selected discrete face; `splitter.get_dface()`.
    pub fn get_dface(&self) -> &FaceHandle {
        &self.face
    }
    /// Obtain its live surface adaptor; `splitter.get_surface()`.
    pub fn get_surface(&self) -> Handle<BRepAdaptorSurface> {
        self.face.get_surface()
    }
    /// Return accumulated U bounds; `splitter.get_range_u()`.
    pub fn get_range_u(&self) -> (f64, f64) {
        self.range_u
    }
    /// Return accumulated V bounds; `splitter.get_range_v()`.
    pub fn get_range_v(&self) -> (f64, f64) {
        self.range_v
    }
    /// Return physical scaling; `splitter.get_delta()`.
    pub fn get_delta(&self) -> (f64, f64) {
        self.delta
    }
    /// Return parametric tolerances; `splitter.get_tolerance_uv()`.
    pub fn get_tolerance_uv(&self) -> (f64, f64) {
        self.tolerance
    }
    fn adjust_bounds(&mut self, surface: &BRepAdaptorSurface) -> Result<bool, Failure> {
        sampling::update_range(
            surface.first_u_parameter()?,
            surface.last_u_parameter()?,
            surface.is_u_periodic()?,
            &mut self.range_u,
        );
        if self.range_u.1 < self.range_u.0 {
            return Ok(false);
        }
        sampling::update_range(
            surface.first_v_parameter()?,
            surface.last_v_parameter()?,
            surface.is_v_periodic()?,
            &mut self.range_v,
        );
        Ok(self.range_v.1.partial_cmp(&self.range_v.0) != Some(std::cmp::Ordering::Less))
    }
    fn compute_tolerance(&mut self, _length_u: f64, _length_v: f64) -> Result<(), Failure> {
        let tolerance = BRepTool::face_tolerance(&self.face.get_face()?)?;
        let surface = self.get_surface();
        self.tolerance.0 = std_max(
            std_min(1.0e-5, surface.u_resolution(tolerance)? * 1.1),
            1.0e-7 * (self.range_u.1 - self.range_u.0),
        );
        self.tolerance.1 = std_max(
            std_min(1.0e-5, surface.v_resolution(tolerance)? * 1.1),
            1.0e-7 * (self.range_v.1 - self.range_v.0),
        );
        Ok(())
    }
    fn compute_delta(&mut self, length_u: f64, length_v: f64) {
        self.delta.0 = (self.range_u.1 - self.range_u.0)
            / if length_u < self.tolerance.0 {
                1.0
            } else {
                length_u
            };
        self.delta.1 = (self.range_v.1 - self.range_v.0)
            / if length_v < self.tolerance.1 {
                1.0
            } else {
                length_v
            };
    }
}
