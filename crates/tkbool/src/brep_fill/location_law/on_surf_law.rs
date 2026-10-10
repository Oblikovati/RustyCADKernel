// Rust translation of ModelingAlgorithms/TKBool/BRepFill/BRepFill_EdgeOnSurfLaw.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepFill_EdgeOnSurfLaw`: the Darboux frame along the curves on surface of the
//! edges of a path lying on the faces of a shape.
use super::{BRepFillLocationLaw, BRepFillLocationLawBase};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::brep_tools::WireExplorer;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Shape, Wire};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dTrimmedCurve};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dCurveOnSurface, Adaptor3dSurface};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkgeomalgo::geom_fill::{
    GeomFillCurveAndTrihedron, GeomFillDarboux, GeomFillLocationLaw, GeomFillTrihedronLaw,
};

/// The location law of a path on a shape (`BRepFill_EdgeOnSurfLaw`): a Darboux
/// trihedron on the curve on surface of each edge; no result when an edge lies on no
/// face of the shape; `let law = BRepFillEdgeOnSurfLaw::new(&path, &support)?;`.
pub struct BRepFillEdgeOnSurfLaw {
    parent: StandardTransient,
    base: BRepFillLocationLawBase,
    hasresult: bool,
}
impl_transient!(
    BRepFillEdgeOnSurfLaw,
    "BRepFill_EdgeOnSurfLaw",
    dyn BRepFillLocationLaw,
    parent,
    interfaces: [dyn BRepFillLocationLaw]
);

/// A curve on surface, its range and the adaptor of its face.
type FaceCurve = (
    Handle<dyn Geom2dCurve>,
    f64,
    f64,
    Handle<dyn Adaptor3dSurface>,
);

/// The curve on the first face of `surf` (explorer order) that holds the edge, and
/// that face's adaptor; `None` when no face holds it.
fn curve_on_shape(edge: &Edge, surf: &Shape) -> Result<Option<FaceCurve>, Failure> {
    for face in Explorer::new(surf, ShapeEnum::Face, ShapeEnum::Shape)? {
        let face = topods::face(&face)?;
        let found = BRepTool::curve_on_face(edge, &face)?;
        if !found.pcurve.is_null() {
            let adaptor: Handle<dyn Adaptor3dSurface> =
                Handle::new(BRepAdaptorSurface::with_face(&face, true)?).cast();
            return Ok(Some((found.pcurve, found.first, found.last, adaptor)));
        }
    }
    Ok(None)
}

impl BRepFillEdgeOnSurfLaw {
    /// `BRepFill_EdgeOnSurfLaw(Path, Surf)`.
    pub fn new(path: &Wire, surf: &Shape) -> Result<Self, Failure> {
        let mut law = Self {
            parent: StandardTransient::default(),
            base: BRepFillLocationLawBase::new(path)?,
            hasresult: true,
        };
        let trihedron: Handle<dyn GeomFillTrihedronLaw> =
            Handle::new(GeomFillDarboux::new()).cast();
        let model = GeomFillCurveAndTrihedron::new(&trihedron);
        let mut ipath: usize = 0;
        let mut explorer = WireExplorer::with_wire(&law.base.path)?;
        while explorer.more() {
            let edge: Edge = explorer.current().clone();
            explorer.next()?;
            if BRepTool::degenerated(&edge)? {
                continue;
            }
            ipath += 1;
            law.base.edges[ipath - 1] = edge.as_ref().clone();
            let Some((pcurve, first, last, surface)) = curve_on_shape(&edge, surf)? else {
                // Impossible to construct the law.
                law.hasresult = false;
                law.base.laws = None;
                return Ok(law);
            };
            let curve: Handle<dyn Adaptor3dCurve> =
                on_surface(&edge, pcurve, (first, last), &surface)?;
            let copy = model.copy()?;
            if let Some(laws) = law.base.laws.as_mut() {
                laws[ipath - 1] = copy.clone();
            }
            if let Some(copy) = copy.get() {
                copy.set_curve(&curve)?;
            }
        }
        Ok(law)
    }

    /// `HasResult()`: every edge lies on a face of the shape; `law.has_result()`.
    pub fn has_result(&self) -> bool {
        self.hasresult
    }
}

/// The curve on surface of an edge, the 2d curve trimmed and reversed for a REVERSED
/// edge.
fn on_surface(
    edge: &Edge,
    mut pcurve: Handle<dyn Geom2dCurve>,
    range: (f64, f64),
    surface: &Handle<dyn Adaptor3dSurface>,
) -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
    let (mut first, mut last) = range;
    if edge.orientation() == Orientation::Reversed {
        let trimmed = Geom2dTrimmedCurve::new(&pcurve, first, last, true, true)?;
        // Reversed, to avoid spoiling the topology.
        trimmed.reverse()?;
        first = trimmed.first_parameter();
        last = trimmed.last_parameter();
        pcurve = Handle::new(trimmed).cast();
    }
    let ac2d: Handle<dyn Adaptor2dCurve2d> =
        Handle::new(Geom2dAdaptorCurve::with_range(&pcurve, first, last)?).cast();
    Ok(Handle::new(Adaptor3dCurveOnSurface::with_curve(&ac2d, surface)?).cast())
}

impl BRepFillLocationLaw for BRepFillEdgeOnSurfLaw {
    fn location_law_base(&self) -> &BRepFillLocationLawBase {
        &self.base
    }
}
