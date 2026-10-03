//! ShapeAnalysis_Wire state, ordering and connection checks; other native checks remain unported.
use super::{ShapeAnalysisEdge, ShapeAnalysisSurface, ShapeAnalysisWireOrder};
use crate::shape_extend::{ShapeExtend, ShapeExtendStatus, ShapeExtendWireData};
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topods::{Face, Wire},
};
use rustycad_tkernel::{
    impl_transient, precision,
    standard::{Failure, Handle, SharedState, StandardTransient},
};
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkmath::toploc::Location;
mod closed;
mod connected;
mod curve_gaps;
mod degenerated;
mod gaps;
mod lacking;
mod order;
mod shape_connect;
mod state;
struct WireAnalysisFields {
    wire: Handle<ShapeExtendWireData>,
    face: Face,
    surface: Handle<ShapeAnalysisSurface>,
    precision: f64,
    status: i32,
    status_order: i32,
    status_connected: i32,
    status_gaps3d: i32,
    status_gaps2d: i32,
    status_curve_gaps: i32,
    status_degenerated: i32,
    status_closed: i32,
    status_lacking: i32,
    min_3d: f64,
    max_3d: f64,
    min_2d: f64,
    max_2d: f64,
}
/// Native wire analyzer foundation; `ShapeAnalysisWire::from_wire(&wire, &face, 1e-7)?`.
/// Implements state initialization, ordering and connection checks.
pub struct ShapeAnalysisWire {
    parent: StandardTransient,
    fields: SharedState<WireAnalysisFields>,
}
impl_transient!(
    ShapeAnalysisWire,
    "ShapeAnalysis_Wire",
    StandardTransient,
    parent
);
impl Default for ShapeAnalysisWire {
    fn default() -> Self {
        Self {
            parent: StandardTransient::default(),
            fields: SharedState::new(WireAnalysisFields {
                wire: Handle::default(),
                face: Face::default(),
                surface: Handle::default(),
                precision: precision::confusion(),
                status: 0,
                status_order: 0,
                status_connected: 0,
                status_gaps3d: 0,
                status_gaps2d: 0,
                status_curve_gaps: 0,
                status_degenerated: 0,
                status_closed: 0,
                status_lacking: 0,
                min_3d: 0.,
                max_3d: 0.,
                min_2d: 0.,
                max_2d: 0.,
            }),
        }
    }
}
impl ShapeAnalysisWire {
    /// Unloaded native analyzer; `ShapeAnalysisWire::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// Load wire topology, face and precision; `ShapeAnalysisWire::from_wire(&wire, &face, 1e-7)?`.
    pub fn from_wire(wire: &Wire, face: &Face, precision: f64) -> Result<Self, Failure> {
        let analyzer = Self::new();
        analyzer.init_wire(wire, face, precision)?;
        Ok(analyzer)
    }
    /// Share the native wire-data object; `ShapeAnalysisWire::from_wire_data(&wire, &face, 1e-7)?`.
    pub fn from_wire_data(
        wire: &Handle<ShapeExtendWireData>,
        face: &Face,
        precision: f64,
    ) -> Result<Self, Failure> {
        let analyzer = Self::new();
        analyzer.init_wire_data(wire, face, precision)?;
        Ok(analyzer)
    }
    /// Reset wire and statuses, then set face and precision; `analyzer.init_wire(&wire, &face, 1e-7)?`.
    pub fn init_wire(&self, wire: &Wire, face: &Face, precision: f64) -> Result<(), Failure> {
        let wire = Handle::new(ShapeExtendWireData::from_wire(wire, true, true)?);
        self.init_wire_data(&wire, face, precision)
    }
    /// Reset wire and statuses while retaining the shared data object; `analyzer.init_wire_data(&wire, &face, 1e-7)?`.
    pub fn init_wire_data(
        &self,
        wire: &Handle<ShapeExtendWireData>,
        face: &Face,
        precision: f64,
    ) -> Result<(), Failure> {
        self.load_wire_data(wire);
        self.set_face(face)?;
        self.set_precision(precision);
        Ok(())
    }
    /// Replace wire, leaving face and precision intact; `analyzer.load_wire(&wire)?`.
    pub fn load_wire(&self, wire: &Wire) -> Result<(), Failure> {
        self.clear_statuses();
        let wire = Handle::new(ShapeExtendWireData::from_wire(wire, true, true)?);
        self.fields.write().wire = wire;
        Ok(())
    }
    /// Share a replacement wire and reset statuses; `analyzer.load_wire_data(&wire)`.
    pub fn load_wire_data(&self, wire: &Handle<ShapeExtendWireData>) {
        self.clear_statuses();
        self.fields.write().wire = wire.clone();
    }
    /// Store face and create its surface analyzer; null faces retain the prior analyzer.
    /// Example: `analyzer.set_face(&face)?`.
    pub fn set_face(&self, face: &Face) -> Result<(), Failure> {
        self.fields.write().face = face.clone();
        if face.is_null() {
            return Ok(());
        }
        let surface = BRepTool::surface(face)?;
        if !surface.is_null() {
            self.fields.write().surface = Handle::new(ShapeAnalysisSurface::new(&surface)?);
        }
        Ok(())
    }
    /// Store explicitly supplied face and surface analyzer; `analyzer.set_face_with_analysis(&face, &surface)`.
    pub fn set_face_with_analysis(&self, face: &Face, surface: &Handle<ShapeAnalysisSurface>) {
        let mut fields = self.fields.write();
        fields.face = face.clone();
        fields.surface = surface.clone();
    }
    /// Replace only the surface analyzer; `analyzer.set_surface_analysis(&surface)`.
    pub fn set_surface_analysis(&self, surface: &Handle<ShapeAnalysisSurface>) {
        self.fields.write().surface = surface.clone();
    }
    /// Create a face around an unlocated surface; `analyzer.set_surface(&surface)?`.
    pub fn set_surface(&self, surface: &Handle<dyn GeomSurface>) -> Result<(), Failure> {
        self.set_located_surface(surface, &Location::default())
    }
    /// Create a face around a located surface; `analyzer.set_located_surface(&surface, &location)?`.
    pub fn set_located_surface(
        &self,
        surface: &Handle<dyn GeomSurface>,
        location: &Location,
    ) -> Result<(), Failure> {
        let mut face = Face::default();
        BRepBuilder::default().make_face_on_located_surface(
            &mut face,
            surface,
            location,
            precision::confusion(),
        )?;
        self.set_face(&face)
    }
}
