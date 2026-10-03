//! Working analyzer delegation for ShapeFix_Wire.
use super::ShapeFixWire;
use crate::shape_analysis::{ShapeAnalysisSurface, ShapeAnalysisWire};
use crate::shape_extend::{ShapeExtend, ShapeExtendStatus};
use crate::shape_fix::FixRoot;
use rustycad_tkbrep::topods::{Face, Wire};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkmath::toploc::Location;

impl ShapeFixWire {
    /// Selects aggregate reorder behavior; `fixer.set_fix_reorder_mode(0)` disables it.
    pub fn set_fix_reorder_mode(&self, mode: i32) {
        self.state.write().fix_reorder_mode = mode;
    }

    /// Selects aggregate small-edge repair; `fixer.set_fix_small_mode(1)` enables it.
    pub fn set_fix_small_mode(&self, mode: i32) {
        self.state.write().fix_small_mode = mode;
    }

    /// Selects aggregate vertex-connection repair; `fixer.set_fix_connected_mode(0)` disables it.
    pub fn set_fix_connected_mode(&self, mode: i32) {
        self.state.write().fix_connected_mode = mode;
    }

    /// Selects aggregate degenerated-edge repair; `fixer.set_fix_degenerated_mode(1)` enables it.
    pub fn set_fix_degenerated_mode(&self, mode: i32) {
        self.state.write().fix_degenerated_mode = mode;
    }

    /// Selects aggregate notched-edge repair; `fixer.set_fix_notched_edges_mode(0)` disables it.
    pub fn set_fix_notched_edges_mode(&self, mode: i32) {
        self.state.write().fix_notched_edges_mode = mode;
    }

    /// Selects aggregate tail repair; `fixer.set_fix_tail_mode(1)` enables it.
    pub fn set_fix_tail_mode(&self, mode: i32) {
        self.state.write().fix_tail_mode = mode;
    }

    /// Sets the native maximum tail angle in radians; `fixer.set_max_tail_angle(0.25)`.
    pub fn set_max_tail_angle(&self, angle: f64) {
        let sine = angle.sin();
        self.state.write().max_tail_angle_sine = sine.max(0.0);
    }

    /// Sets the maximum tail width; `fixer.set_max_tail_width(0.01)`.
    pub fn set_max_tail_width(&self, width: f64) {
        self.state.write().max_tail_width = width;
    }

    /// Selects aggregate self-intersection repair; `fixer.set_fix_self_intersection_mode(0)` disables it.
    pub fn set_fix_self_intersection_mode(&self, mode: i32) {
        self.state.write().fix_self_intersection_mode = mode;
    }

    /// Selects aggregate missing-edge repair; `fixer.set_fix_lacking_mode(1)` enables it.
    pub fn set_fix_lacking_mode(&self, mode: i32) {
        self.state.write().fix_lacking_mode = mode;
    }

    /// Reads the native aggregate small-edge mode; `fixer.fix_small_mode()`.
    pub fn fix_small_mode(&self) -> i32 {
        self.state.read().fix_small_mode
    }

    /// Reads the native aggregate vertex-connection mode; `fixer.fix_connected_mode()`.
    pub fn fix_connected_mode(&self) -> i32 {
        self.state.read().fix_connected_mode
    }

    /// Reads the native aggregate degenerated-edge mode; `fixer.fix_degenerated_mode()`.
    pub fn fix_degenerated_mode(&self) -> i32 {
        self.state.read().fix_degenerated_mode
    }

    /// Reads the native aggregate self-intersection mode; `fixer.fix_self_intersection_mode()`.
    pub fn fix_self_intersection_mode(&self) -> i32 {
        self.state.read().fix_self_intersection_mode
    }

    /// Reads the native aggregate missing-edge mode; `fixer.fix_lacking_mode()`.
    pub fn fix_lacking_mode(&self) -> i32 {
        self.state.read().fix_lacking_mode
    }

    /// Returns whether self-intersection repair removed a segment; `fixer.status_removed_segment()`.
    pub fn status_removed_segment(&self) -> bool {
        self.state.read().removed_segment
    }

    /// Reads aggregate notched-edge status; `fixer.status_notches(ShapeExtendStatus::Done)`.
    pub fn status_notches(&self, status: ShapeExtendStatus) -> bool {
        ShapeExtend::decode_status(self.state.read().notches_status, status)
    }

    /// Reads aggregate tail status; `fixer.status_fix_tails(ShapeExtendStatus::Done)`.
    pub fn status_fix_tails(&self, status: ShapeExtendStatus) -> bool {
        ShapeExtend::decode_status(self.state.read().tails_status, status)
    }

    /// Sets whether high-level `Perform` invokes `FixEdgeCurves`; `fixer.set_fix_edge_curves_mode(0)`.
    pub fn set_fix_edge_curves_mode(&self, mode: i32) {
        self.state.write().edge_curves_mode = mode;
    }

    /// Reads the high-level `FixEdgeCurves` mode; `assert_eq!(fixer.fix_edge_curves_mode(), -1)`.
    pub fn fix_edge_curves_mode(&self) -> i32 {
        self.state.read().edge_curves_mode
    }

    /// Sets whether reversed p-curves are repaired; `fixer.set_fix_reversed_2d_mode(1)`.
    pub fn set_fix_reversed_2d_mode(&self, mode: i32) {
        self.state.write().reversed_2d_mode = mode;
    }

    /// Reads the reversed p-curve repair mode; `fixer.fix_reversed_2d_mode()`.
    pub fn fix_reversed_2d_mode(&self) -> i32 {
        self.state.read().reversed_2d_mode
    }

    /// Sets whether invalid face p-curves may be removed; `fixer.set_fix_remove_pcurve_mode(1)`.
    pub fn set_fix_remove_pcurve_mode(&self, mode: i32) {
        self.state.write().remove_pcurve_mode = mode;
    }

    /// Reads the p-curve removal mode; `fixer.fix_remove_pcurve_mode()`.
    pub fn fix_remove_pcurve_mode(&self) -> i32 {
        self.state.read().remove_pcurve_mode
    }

    /// Sets whether missing p-curves are projected; `fixer.set_fix_add_pcurve_mode(1)`.
    pub fn set_fix_add_pcurve_mode(&self, mode: i32) {
        self.state.write().add_pcurve_mode = mode;
    }

    /// Reads the p-curve creation mode; `fixer.fix_add_pcurve_mode()`.
    pub fn fix_add_pcurve_mode(&self) -> i32 {
        self.state.read().add_pcurve_mode
    }

    /// Sets whether invalid 3D curves may be removed; `fixer.set_fix_remove_curve_3d_mode(1)`.
    pub fn set_fix_remove_curve_3d_mode(&self, mode: i32) {
        self.state.write().remove_curve_3d_mode = mode;
    }

    /// Reads the 3D curve removal mode; `fixer.fix_remove_curve_3d_mode()`.
    pub fn fix_remove_curve_3d_mode(&self) -> i32 {
        self.state.read().remove_curve_3d_mode
    }

    /// Sets whether missing 3D curves are built; `fixer.set_fix_add_curve_3d_mode(1)`.
    pub fn set_fix_add_curve_3d_mode(&self, mode: i32) {
        self.state.write().add_curve_3d_mode = mode;
    }

    /// Reads the 3D curve creation mode; `fixer.fix_add_curve_3d_mode()`.
    pub fn fix_add_curve_3d_mode(&self) -> i32 {
        self.state.read().add_curve_3d_mode
    }

    /// Sets whether seam p-curves are repaired; `fixer.set_fix_seam_mode(1)`.
    pub fn set_fix_seam_mode(&self, mode: i32) {
        self.state.write().seam_mode = mode;
    }

    /// Reads the seam repair mode; `fixer.fix_seam_mode()`.
    pub fn fix_seam_mode(&self) -> i32 {
        self.state.read().seam_mode
    }

    /// Sets whether periodic p-curves are shifted; `fixer.set_fix_shifted_mode(0)`.
    pub fn set_fix_shifted_mode(&self, mode: i32) {
        self.state.write().shifted_mode = mode;
    }

    /// Reads the shifted p-curve mode; `fixer.fix_shifted_mode()`.
    pub fn fix_shifted_mode(&self) -> i32 {
        self.state.read().shifted_mode
    }

    /// Sets whether 2D/3D parameterization is repaired; `fixer.set_fix_same_parameter_mode(1)`.
    pub fn set_fix_same_parameter_mode(&self, mode: i32) {
        self.state.write().same_parameter_mode = mode;
    }

    /// Reads the same-parameter mode; `fixer.fix_same_parameter_mode()`.
    pub fn fix_same_parameter_mode(&self) -> i32 {
        self.state.read().same_parameter_mode
    }

    /// Sets whether vertex tolerances are recomputed; `fixer.set_fix_vertex_tolerance_mode(0)`.
    pub fn set_fix_vertex_tolerance_mode(&self, mode: i32) {
        self.state.write().vertex_tolerance_mode = mode;
    }

    /// Reads the vertex-tolerance mode; `fixer.fix_vertex_tolerance_mode()`.
    pub fn fix_vertex_tolerance_mode(&self) -> i32 {
        self.state.read().vertex_tolerance_mode
    }

    /// Loads a wire, supporting face, and both root/analyzer precision in native order.
    /// Example: `fixer.init_wire_face(&wire, &face, 1e-6)?`.
    pub fn init_wire_face(&self, wire: &Wire, face: &Face, precision: f64) -> Result<(), Failure> {
        self.load(wire)?;
        self.set_face(face)?;
        self.set_precision(precision);
        Ok(())
    }

    /// Creates a configured fixer with native maximum tolerance initialization.
    /// Example: `ShapeFixWire::from_wire_face(&wire, &face, 1e-6)?`.
    pub fn from_wire_face(wire: &Wire, face: &Face, precision: f64) -> Result<Self, Failure> {
        let fixer = Self::new();
        fixer.set_max_tolerance(precision);
        fixer.init_wire_face(wire, face, precision)?;
        Ok(fixer)
    }

    /// Returns the shared analyzer, including a null handle supplied through Init.
    /// Example: `let analyzer = fixer.working_analyzer()`.
    pub fn working_analyzer(&self) -> Handle<ShapeAnalysisWire> {
        self.state.read().analyzer.clone()
    }

    /// Counts loaded edges, or returns zero for an unloaded edge list.
    /// Example: `assert_eq!(ShapeFixWire::new().nb_edges(), 0)`.
    pub fn nb_edges(&self) -> i32 {
        self.working_analyzer().nb_edges()
    }

    /// Returns the working supporting face.
    /// Example: `let face = fixer.face()`.
    pub fn face(&self) -> Face {
        self.working_analyzer().face()
    }

    /// Controls whether aggregate repairs include the last-to-first connection.
    /// Example: `fixer.set_closed_mode(false)`.
    pub fn set_closed_mode(&self, enabled: bool) {
        self.state.write().closed_mode = enabled;
    }

    /// Shares a supporting face and explicit surface analyzer.
    /// Example: `fixer.set_face_with_analysis(&face, &surface)`.
    pub fn set_face_with_analysis(&self, face: &Face, surface: &Handle<ShapeAnalysisSurface>) {
        self.working_analyzer()
            .set_face_with_analysis(face, surface);
    }

    /// Replaces only the surface analyzer, retaining the working face.
    /// Example: `fixer.set_surface_analysis(&surface)`.
    pub fn set_surface_analysis(&self, surface: &Handle<ShapeAnalysisSurface>) {
        self.working_analyzer().set_surface_analysis(surface);
    }

    /// Builds a supporting face from an unlocated surface.
    /// Example: `fixer.set_surface(&surface)?`.
    pub fn set_surface(&self, surface: &Handle<dyn GeomSurface>) -> Result<(), Failure> {
        self.working_analyzer().set_surface(surface)
    }

    /// Builds a supporting face from a surface at the specified placement.
    /// Example: `fixer.set_located_surface(&surface, &location)?`.
    pub fn set_located_surface(
        &self,
        surface: &Handle<dyn GeomSurface>,
        location: &Location,
    ) -> Result<(), Failure> {
        self.working_analyzer()
            .set_located_surface(surface, location)
    }
}
