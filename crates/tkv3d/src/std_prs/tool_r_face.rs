//! Native oriented p-curve traversal for Martensite isoline trimming; OCCT 3d097a0.
use super::StdPrsIsolineSurfaceSource;
use rustycad_tkbrep::{
    brep::BRepTool,
    topexp::Explorer,
    topods::{Face, edge},
};
use rustycad_tkernel::standard::{Failure, Handle, SharedState};
use rustycad_tkg2d::{adaptor2d::Adaptor2dCurve2d, geom2d_adaptor::Geom2dAdaptorCurve};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
/// Traverse forward/reversed p-curves with one live curve adaptor; `tool.init()?`.
pub struct StdPrsToolRFace {
    face: Face,
    explorer: SharedState<Explorer>,
    curve: Geom2dAdaptorCurve,
    has_null_curves: SharedState<bool>,
}
impl Default for StdPrsToolRFace {
    fn default() -> Self {
        Self {
            face: Face::default(),
            explorer: SharedState::new(Explorer::default()),
            curve: Geom2dAdaptorCurve::default(),
            has_null_curves: SharedState::new(false),
        }
    }
}
impl StdPrsToolRFace {
    /// Construct uninitialized traversal; `StdPrsToolRFace::new().more() == false`.
    pub fn new() -> Self {
        // Init is separate natively, so construction must not discover missing p-curves yet.
        Self::default()
    }
    /// Retain the face with forward orientation; `StdPrsToolRFace::with_surface(&surface)?`.
    pub fn with_surface<
        S: StdPrsIsolineSurfaceSource + rustycad_tkernel::standard::RuntimeTyped + ?Sized,
    >(
        surface: &Handle<S>,
    ) -> Result<Self, Failure> {
        let mut face = surface.isoline_surface_base().face();
        face.set_orientation(Orientation::Forward);
        Ok(Self {
            face,
            ..Self::default()
        })
    }
    /// Restart without clearing the sticky invalid flag; `tool.init()?`.
    pub fn init(&self) -> Result<(), Failure> {
        self.explorer
            .write()
            .init(&self.face, ShapeEnum::Edge, ShapeEnum::Shape)?;
        self.seek_oriented_curve()
    }
    /// Move once, then skip invalid/unoriented edges; `tool.next()?`.
    #[allow(clippy::should_implement_trait)]
    pub fn next(&self) -> Result<(), Failure> {
        self.explorer.write().next()?;
        self.seek_oriented_curve()
    }
    /// Report actual explorer state; `while tool.more() { tool.next()?; }`.
    pub fn more(&self) -> bool {
        let explorer = self.explorer.read();
        explorer.more()
    }
    /// Native hatching is always oriented; `assert!(tool.is_oriented())`.
    pub fn is_oriented(&self) -> bool {
        // Boundary orientation controls inside/outside; never silently use unordered hatching.
        true
    }
    /// Read the cumulative current orientation; `tool.orientation()`.
    pub fn orientation(&self) -> Orientation {
        let explorer = self.explorer.read();
        explorer.current().orientation()
    }
    /// Borrow the real adaptor, observing later loads/reset; `let curve = tool.value(); tool.next()?`.
    pub fn value(&self) -> &dyn Adaptor2dCurve2d {
        // Native Value refers to myCurve rather than an adaptor snapshot, including after Init/Next.
        &self.curve
    }
    /// Report whether any missing p-curve was skipped; `tool.is_invalid_geometry()`.
    pub fn is_invalid_geometry(&self) -> bool {
        let skipped = self.has_null_curves.read();
        *skipped
    }
}
impl StdPrsToolRFace {
    fn seek_oriented_curve(&self) -> Result<(), Failure> {
        while self.more() {
            if self.load_current_curve()? {
                return Ok(());
            }
            self.explorer.write().next()?;
        }
        self.curve.reset();
        Ok(())
    }
    fn load_current_curve(&self) -> Result<bool, Failure> {
        if !matches!(
            self.orientation(),
            Orientation::Forward | Orientation::Reversed
        ) {
            return Ok(false);
        }
        let current = edge(self.explorer.read().current())?.clone();
        let pcurve = BRepTool::curve_on_face(&current, &self.face)?;
        if pcurve.pcurve.is_null() {
            *self.has_null_curves.write() = true;
            return Ok(false);
        }
        self.curve
            .load_range(&pcurve.pcurve, pcurve.first, pcurve.last)?;
        Ok(true)
    }
}
