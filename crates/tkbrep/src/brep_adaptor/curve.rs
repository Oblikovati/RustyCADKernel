// Rust translation of ModelingData/TKBRep/BRepAdaptor/BRepAdaptor_Curve.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepAdaptor_Curve`: an edge seen as a 3D curve.
use super::forward::inherited_curve_members;
use crate::brep::BRepTool;
use crate::brep::errors::null_object;
use crate::topods::{Edge, Face};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle, SharedState};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dCurveOnSurface, Adaptor3dSurface};
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBezierCurve, GeomCurveD1, GeomCurveD2, GeomCurveD3, GeomOffsetCurve,
    GeomSurface,
};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorSurface, GeomAdaptorTransformedCurve};
use rustycad_tkmath::geom_abs::{CurveType, Shape};
use rustycad_tkmath::gp::{
    Circle3, Ellipse3, Hyperbola3, Line3, Parabola3, Point3, Transform3, Vector3,
};

/// An edge seen as a 3D curve (`BRepAdaptor_Curve`): its 3D curve, or its first curve
/// on a surface when it has none, or its curve on a given face, transformed by the
/// edge location.
///
/// The native class derives from `GeomAdaptor_TransformedCurve`; the Rust type embeds
/// that adaptor as its parent field and forwards every inherited member to it. The
/// members take `&self`, the edge is kept behind a lock (docs/geom-design.md).
///
/// ```
/// use rustycad_tkbrep::brep::BRepBuilder;
/// use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
/// use rustycad_tkbrep::topods::Edge;
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
/// use rustycad_tkg3d::geom::{GeomCurve, GeomLine};
/// use rustycad_tkmath::gp::{Axis1, Point3};
/// let line: Handle<dyn GeomCurve> = Handle::new(GeomLine::new(Axis1::default())).cast();
/// let mut edge = Edge::default();
/// let builder = BRepBuilder::default();
/// builder.make_edge_with_curve(&mut edge, &line, 1e-7)?;
/// builder.range(&edge, 0.0, 2.0, false)?;
/// let curve = BRepAdaptorCurve::with_edge(&edge)?;
/// assert_eq!(curve.value(2.0)?, Point3::new(0.0, 0.0, 2.0));
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Default)]
pub struct BRepAdaptorCurve {
    parent: GeomAdaptorTransformedCurve,
    edge: SharedState<Edge>,
}
impl_transient!(
    BRepAdaptorCurve,
    "BRepAdaptor_Curve",
    GeomAdaptorTransformedCurve,
    parent,
    interfaces: [dyn Adaptor3dCurve]
);

impl BRepAdaptorCurve {
    /// `BRepAdaptor_Curve(E)`: the 3D curve of `edge`, or its first curve on a surface;
    /// Standard_NullObject when it has neither; `BRepAdaptorCurve::with_edge(&edge)?`.
    pub fn with_edge(edge: &Edge) -> Result<Self, Failure> {
        let adaptor = Self::default();
        adaptor.initialize(edge)?;
        Ok(adaptor)
    }

    /// `BRepAdaptor_Curve(E, F)`: the curve on `face` of `edge`;
    /// `BRepAdaptorCurve::with_edge_on_face(&edge, &face)?`.
    pub fn with_edge_on_face(edge: &Edge, face: &Face) -> Result<Self, Failure> {
        let adaptor = Self::default();
        adaptor.initialize_on_face(edge, face)?;
        Ok(adaptor)
    }

    /// The embedded parent adaptor (the `GeomAdaptor_TransformedCurve` base).
    pub fn transformed_curve(&self) -> &GeomAdaptorTransformedCurve {
        &self.parent
    }

    /// `Reset()`: unloads the curve, the curve on surface, the edge and the
    /// transformation; `adaptor.reset()`.
    pub fn reset(&self) {
        if let Some(curve) = self.parent.change_curve().get() {
            curve.reset();
        }
        self.parent.load_curve_on_surface(&Handle::default());
        *self.edge.write() = Edge::default();
        self.parent.set_trsf(Transform3::default());
    }

    /// `Initialize(E)`: loads the 3D curve of `edge` on its range, else its first curve on
    /// a surface; Standard_NullObject ("No geometry") when it has neither, after the
    /// edge is stored and the curve on surface dropped; `adaptor.initialize(&edge)?`.
    pub fn initialize(&self, edge: &Edge) -> Result<(), Failure> {
        self.parent.load_curve_on_surface(&Handle::default());
        *self.edge.write() = edge.clone();
        let found = BRepTool::curve_with_location(edge)?;
        let location = if found.curve.is_null() {
            self.load_first_curve_on_surface(edge)?
        } else {
            self.parent
                .load_range(&found.curve, found.first, found.last)?;
            found.location
        };
        self.parent.set_trsf(location.transformation());
        Ok(())
    }

    /// The first curve on a surface of `edge`, loaded as the curve on surface; returns
    /// its location.
    fn load_first_curve_on_surface(
        &self,
        edge: &Edge,
    ) -> Result<rustycad_tkmath::toploc::Location, Failure> {
        let found = BRepTool::first_curve_on_surface(edge)?;
        if found.pcurve.is_null() {
            return Err(null_object(
                "BRepAdaptor_Curve::No geometry: the edge has no 3D curve and no curve on surface",
            ));
        }
        let con_surf = curve_on_surface(&found.pcurve, &found.surface, found.first, found.last)?;
        self.parent.load_curve_on_surface(&con_surf);
        Ok(found.location)
    }

    /// `Initialize(E, F)`: loads the curve on `face` of `edge` (its p-curve on the face's
    /// surface), transformed by the face location; `adaptor.initialize_on_face(&e, &f)?`.
    pub fn initialize_on_face(&self, edge: &Edge, face: &Face) -> Result<(), Failure> {
        self.parent.load_curve_on_surface(&Handle::default());
        *self.edge.write() = edge.clone();
        let (surface, location) = BRepTool::surface_with_location(face)?;
        let found = BRepTool::curve_on_face(edge, face)?;
        let con_surf = curve_on_surface(&found.pcurve, &surface, found.first, found.last)?;
        self.parent.load_curve_on_surface(&con_surf);
        self.parent.set_trsf(location.transformation());
        Ok(())
    }

    /// `Edge()`; `adaptor.edge()`.
    pub fn edge(&self) -> Edge {
        self.edge.read().clone()
    }

    /// `Tolerance()`: the tolerance of the edge; `adaptor.tolerance()?`.
    pub fn tolerance(&self) -> Result<f64, Failure> {
        BRepTool::edge_tolerance(&self.edge())
    }

    /// The native copy constructor: the parent adaptor copied (the 3D adaptor by value,
    /// the curve on surface shared), and the edge.
    fn copy_with(&self, parent: GeomAdaptorTransformedCurve) -> Self {
        Self {
            parent,
            edge: SharedState::new(self.edge()),
        }
    }
}

/// `new Adaptor3d_CurveOnSurface()` loaded with the p-curve on `[first, last]` and the
/// untrimmed surface, as the two `Initialize` members build it.
fn curve_on_surface(
    pcurve: &Handle<dyn Geom2dCurve>,
    surface: &Handle<dyn GeomSurface>,
    first: f64,
    last: f64,
) -> Result<Handle<Adaptor3dCurveOnSurface>, Failure> {
    let adapted_surface = GeomAdaptorSurface::default();
    adapted_surface.load(surface)?;
    let adapted_curve = Geom2dAdaptorCurve::default();
    adapted_curve.load_range(pcurve, first, last)?;
    let con_surf = Adaptor3dCurveOnSurface::default();
    let curve: Handle<dyn Adaptor2dCurve2d> = Handle::new(adapted_curve).cast();
    let surface: Handle<dyn Adaptor3dSurface> = Handle::new(adapted_surface).cast();
    con_surf.load(&curve, &surface)?;
    Ok(Handle::new(con_surf))
}

impl Adaptor3dCurve for BRepAdaptorCurve {
    /// `ShallowCopy()`: shallow copies of the 3D adaptor and of the curve on surface,
    /// the transformation and the edge.
    fn shallow_copy(&self) -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
        Ok(Handle::new(self.copy_with(self.parent.shallow_adaptor()?)).cast())
    }

    /// `Trim(First, Last, Tol)`: a copy of this adaptor (edge and transformation kept)
    /// whose 3D curve is reloaded on `[first, last]`, or whose curve on surface is
    /// trimmed.
    fn trim(
        &self,
        first: f64,
        last: f64,
        tolerance: f64,
    ) -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
        let parent = GeomAdaptorTransformedCurve::default();
        if self.parent.is_3d_curve() {
            parent.load_range(&self.parent.geom_curve(), first, last)?;
        } else {
            let trimmed = self
                .parent
                .curve_on_surface()?
                .trim(first, last, tolerance)?;
            parent.load_curve_on_surface(&trimmed.cast());
        }
        parent.set_trsf(self.parent.trsf());
        Ok(Handle::new(self.copy_with(parent)).cast())
    }

    inherited_curve_members!();
}
