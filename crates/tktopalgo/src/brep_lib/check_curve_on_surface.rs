// Rust translation of BRepLib_CheckCurveOnSurface.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepLib_CheckCurveOnSurface`: the greatest distance between the 3d curve of an edge
//! and its curves on a face.
use rustycad_tkbrep::brep::{BRepTool, EdgePCurve};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dCurveOnSurface, Adaptor3dSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkgeombase::geom_lib::CheckCurveOnSurface;

/// `BRepLib_CheckCurveOnSurface` (docs/topalgo-design.md section 7): the
/// `GeomLib_CheckCurveOnSurface` of an edge on a face; on a seam both curves on surface
/// are checked, the second only when the first succeeds.
///
/// ```
/// use rustycad_tktopalgo::brep_lib::BRepLibCheckCurveOnSurface;
/// let mut check = BRepLibCheckCurveOnSurface::new();
/// check.perform();
/// assert!(!check.is_done());
/// assert_eq!(check.error_status(), 1);
/// ```
#[derive(Clone)]
pub struct BRepLibCheckCurveOnSurface {
    my_c_on_surf_geom: CheckCurveOnSurface,
    my_adaptor_curve_on_surface: Handle<Adaptor3dCurveOnSurface>,
    my_adaptor_curve_on_surface2: Handle<Adaptor3dCurveOnSurface>,
    my_is_parallel: bool,
}

impl Default for BRepLibCheckCurveOnSurface {
    fn default() -> Self {
        Self::new()
    }
}

impl BRepLibCheckCurveOnSurface {
    /// `BRepLib_CheckCurveOnSurface()`: nothing to check;
    /// `BRepLibCheckCurveOnSurface::new()`.
    pub fn new() -> Self {
        Self {
            my_c_on_surf_geom: CheckCurveOnSurface::new(),
            my_adaptor_curve_on_surface: Handle::default(),
            my_adaptor_curve_on_surface2: Handle::default(),
            my_is_parallel: false,
        }
    }

    /// `BRepLib_CheckCurveOnSurface(theEdge, theFace)`: `Init`;
    /// `BRepLibCheckCurveOnSurface::with_edge_face(&edge, &face)?`.
    pub fn with_edge_face(edge: &Edge, face: &Face) -> Result<Self, Failure> {
        let mut check = Self::new();
        check.init(edge, face)?;
        Ok(check)
    }

    /// `Init(theEdge, theFace)`: resets the check, then, for a non-degenerated
    /// geometric edge on a face, adapts the edge curve and its curves on the face (the
    /// curves on surface stay those of an earlier Init otherwise); `check.init(&e, &f)?`.
    pub fn init(&mut self, edge: &Edge, face: &Face) -> Result<(), Failure> {
        self.my_c_on_surf_geom.init_empty();
        if edge.is_null() || face.is_null() {
            return Ok(());
        }
        if BRepTool::degenerated(edge)? || !BRepTool::is_geometric_edge(edge)? {
            return Ok(());
        }
        // 3D curve initialization
        let curve3d: Handle<dyn Adaptor3dCurve> =
            Handle::new(BRepAdaptorCurve::with_edge(edge)?).cast();
        // Surface initialization
        let pcurve = BRepTool::curve_on_face(edge, face)?;
        let geom_surface = BRepTool::surface(face)?;
        // 2D curves initialization
        let curve2d = pcurve_adaptor(&pcurve)?;
        let surface: Handle<dyn Adaptor3dSurface> =
            Handle::new(GeomAdaptorSurface::with_surface(&geom_surface)?).cast();
        self.my_adaptor_curve_on_surface =
            Handle::new(Adaptor3dCurveOnSurface::with_curve(&curve2d, &surface)?);
        if BRepTool::is_closed_on_face(edge, face)? {
            let reversed = Edge::unchecked(edge.reversed());
            let curve2d = pcurve_adaptor(&BRepTool::curve_on_face(&reversed, face)?)?;
            self.my_adaptor_curve_on_surface2 =
                Handle::new(Adaptor3dCurveOnSurface::with_curve(&curve2d, &surface)?);
        }
        self.my_c_on_surf_geom
            .init(&curve3d, precision::p_confusion());
        Ok(())
    }

    /// `Perform()`: the greatest distance on the first curve on surface, then, when it
    /// succeeds and the edge is a seam, on the second one; `check.perform()`.
    pub fn perform(&mut self) {
        // Compute the max distance
        let first = self.my_adaptor_curve_on_surface.clone();
        self.compute(&first);
        if self.error_status() != 0 {
            return;
        }
        if self.my_adaptor_curve_on_surface2.is_some() {
            // compute max distance for myAdaptorCurveOnSurface2
            // (for the second curve on closed surface)
            let second = self.my_adaptor_curve_on_surface2.clone();
            self.compute(&second);
        }
    }

    /// The protected `Compute(theCurveOnSurface)`: the geometric check in the current
    /// parallel mode.
    fn compute(&mut self, curve_on_surface: &Handle<Adaptor3dCurveOnSurface>) {
        self.my_c_on_surf_geom.set_parallel(self.my_is_parallel);
        self.my_c_on_surf_geom.perform(curve_on_surface);
    }

    /// `IsDone()`: `ErrorStatus() == 0`; `check.is_done()`.
    pub fn is_done(&self) -> bool {
        self.my_c_on_surf_geom.error_status() == 0
    }

    /// `SetParallel(theIsParallel)`; `check.set_parallel(true)`.
    pub fn set_parallel(&mut self, is_parallel: bool) {
        self.my_is_parallel = is_parallel;
    }

    /// `IsParallel()`; `check.is_parallel()`.
    pub fn is_parallel(&self) -> bool {
        self.my_is_parallel
    }

    /// `ErrorStatus()`: 0 on success (see `GeomLib_CheckCurveOnSurface`);
    /// `check.error_status()`.
    pub fn error_status(&self) -> i32 {
        self.my_c_on_surf_geom.error_status()
    }

    /// `MaxDistance()`; `check.max_distance()`.
    pub fn max_distance(&self) -> f64 {
        self.my_c_on_surf_geom.max_distance()
    }

    /// `MaxParameter()`: where the greatest distance is reached; `check.max_parameter()`.
    pub fn max_parameter(&self) -> f64 {
        self.my_c_on_surf_geom.max_parameter()
    }
}

/// The adaptor of a p-curve over its range (Standard_NullObject for a null p-curve).
fn pcurve_adaptor(pcurve: &EdgePCurve) -> Result<Handle<dyn Adaptor2dCurve2d>, Failure> {
    let adaptor = Geom2dAdaptorCurve::with_range(&pcurve.pcurve, pcurve.first, pcurve.last)?;
    Ok(Handle::new(adaptor).cast())
}
