//! Working analyzer delegation for ShapeFix_Wire.
use super::ShapeFixWire;
use crate::shape_analysis::{ShapeAnalysisSurface, ShapeAnalysisWire};
use crate::shape_fix::FixRoot;
use rustycad_tkbrep::topods::{Face, Wire};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkmath::toploc::Location;

impl ShapeFixWire {
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
