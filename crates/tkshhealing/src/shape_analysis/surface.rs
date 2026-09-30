// Rust translation of ShapeAnalysis_Surface at OCCT
// 3d097a0328e71b826377d4814ab05ec3c3d23871.
mod boxes;
mod closure;
mod degenerate;
mod degenerate_sequence;
mod iso_projection;
mod isos;
mod newton;
mod next_projection;
mod projection;
mod singularities;
pub use singularities::SurfaceSingularity;

use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface, SurfaceBounds};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::{Point2, Point3};

#[derive(Clone)]
struct SurfaceAnalysisFields {
    surface: Handle<dyn GeomSurface>,
    adaptor: Handle<GeomAdaptorSurface>,
    bounds: SurfaceBounds,
    gap: f64,
    boxes: [BoundingBox3; 4],
    boxes_computed: bool,
    closure: [f64; 2],
    extension: [f64; 2],
    singularities: Option<Vec<SurfaceSingularity>>,
    bound_isos: Option<[Handle<dyn GeomCurve>; 4]>,
}

/// Surface analysis with native identity-sensitive initialization and cached boundary isos.
///
/// ```
/// use rustycad_tkshhealing::shape_analysis::ShapeAnalysisSurface;
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkg3d::geom::GeomPlane;
/// use rustycad_tkmath::gp::Axis3;
/// let surface = Handle::new(GeomPlane::new(Axis3::default())).cast();
/// let analysis = ShapeAnalysisSurface::new(&surface)?;
/// let point = analysis.value(0.0, 0.0)?;
/// # Ok::<(),rustycad_tkernel::standard::Failure>(())
/// ```
pub struct ShapeAnalysisSurface {
    parent: StandardTransient,
    fields: SharedState<SurfaceAnalysisFields>,
    extrema: SharedState<Option<rustycad_tkgeombase::extrema::ExtPS<'static>>>,
}
impl_transient!(
    ShapeAnalysisSurface,
    "ShapeAnalysis_Surface",
    StandardTransient,
    parent
);

impl ShapeAnalysisSurface {
    /// Creates the analyzer; `ShapeAnalysisSurface::new(&surface)?`.
    pub fn new(surface: &Handle<dyn GeomSurface>) -> Result<Self, Failure> {
        let fields = SurfaceAnalysisFields::new(surface)?;
        Ok(Self {
            parent: StandardTransient::default(),
            fields: SharedState::new(fields),
            extrema: SharedState::new(None),
        })
    }

    /// Resets caches only when surface identity changes; `analysis.init(&surface)?`.
    pub fn init(&self, surface: &Handle<dyn GeomSurface>) -> Result<(), Failure> {
        if self.surface() == *surface {
            return Ok(());
        }
        // Bug 33895: a null initialization preserves the previous surface.
        if surface.is_null() {
            return Ok(());
        }
        let mut replacement = SurfaceAnalysisFields::new(surface)?;
        // Native Init invalidates computation, but retains accumulated box contents.
        replacement.boxes = self.fields.read().boxes.clone();
        replacement.extension = self.fields.read().extension;
        *self.fields.write() = replacement;
        *self.extrema.write() = None;
        Ok(())
    }

    /// Copies native cached singularities and adaptor without recomputation; `analysis.init_from(&other)?`.
    pub fn init_from(&self, other: &ShapeAnalysisSurface) -> Result<(), Failure> {
        self.init(&other.surface())?;
        let copied = other.fields.snapshot();
        let mut fields = self.fields.write();
        fields.adaptor = copied.adaptor;
        fields.singularities = copied.singularities;
        Ok(())
    }

    /// Returns the analyzed surface; `let surface = analysis.surface();`.
    pub fn surface(&self) -> Handle<dyn GeomSurface> {
        self.fields.read().surface.clone()
    }

    /// Returns the native eagerly created adaptor; `let adaptor = analysis.adaptor3d();`.
    pub fn adaptor3d(&self) -> Handle<GeomAdaptorSurface> {
        self.fields.read().adaptor.clone()
    }

    /// Returns the existing adaptor; `let adaptor = analysis.true_adaptor3d();`.
    pub fn true_adaptor3d(&self) -> Handle<GeomAdaptorSurface> {
        self.adaptor3d()
    }

    /// Returns the last analysis distance; `let gap = analysis.gap();`.
    pub fn gap(&self) -> f64 {
        self.fields.read().gap
    }

    /// Evaluates using the adaptor; `let point = analysis.value(0.0, 0.0)?`.
    pub fn value(&self, u: f64, v: f64) -> Result<Point3, Failure> {
        self.adaptor3d().value(u, v)
    }

    /// Evaluates a parameter point; `let point = analysis.value_uv(Point2::new(0.0, 0.0))?`.
    pub fn value_uv(&self, uv: Point2) -> Result<Point3, Failure> {
        self.value(uv.x(), uv.y())
    }

    /// Returns buffered bounds; `let bounds = analysis.bounds();`.
    pub fn bounds(&self) -> SurfaceBounds {
        self.fields.read().bounds
    }

    /// Changes buffered bounds without invalidating native caches; `analysis.set_domain((0.,1.,0.,1.));`.
    pub fn set_domain(&self, bounds: SurfaceBounds) {
        self.fields.write().bounds = bounds;
    }
}

impl SurfaceAnalysisFields {
    fn new(surface: &Handle<dyn GeomSurface>) -> Result<Self, Failure> {
        let live = surface.get().ok_or_else(|| {
            Failure::with_message(
                "Standard_NullObject",
                "ShapeAnalysis_Surface: null surface; expected a Geom_Surface",
            )
        })?;
        Ok(Self {
            surface: surface.clone(),
            adaptor: Handle::new(GeomAdaptorSurface::with_surface(surface)?),
            bounds: live.bounds(),
            gap: 0.0,
            boxes: std::array::from_fn(|_| BoundingBox3::default()),
            boxes_computed: false,
            closure: [-1.; 2],
            extension: [0.01; 2],
            singularities: None,
            bound_isos: None,
        })
    }
}
