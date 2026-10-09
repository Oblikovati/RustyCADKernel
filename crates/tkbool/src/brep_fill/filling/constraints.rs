// Rust translation of ModelingAlgorithms/TKBool/BRepFill/BRepFill_CurveConstraint.hxx/.cxx,
// BRepFill_EdgeFaceAndOrder.hxx/.cxx and BRepFill_FaceAndOrder.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The constraints of a filling: the curve constraints of the plate (on a face or on a
//! 3d curve) and the edge/face/order records of the boundary and the free constraints.
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dCurveOnSurface};
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkgeomalgo::geom_plate::GeomPlateCurveConstraint;
use rustycad_tkmath::geom_abs::Shape as Continuity;

/// The curve constraints of a filling (`BRepFill_CurveConstraint`, a
/// GeomPlate_CurveConstraint whose curve may lie on a face); Rust has no class
/// inheritance, so the constructors build the plate constraint directly;
/// `let c = BRepFillCurveConstraint::with_curve(&curve, 0, 10, 1e-4)?;`.
pub struct BRepFillCurveConstraint;

impl BRepFillCurveConstraint {
    /// `BRepFill_CurveConstraint(Boundary, Order, NPt, TolDist, TolAng, TolCurv)`
    /// (natively defaulted to 10, 1e-4, 0.01 and 0.1): the constraint of order -1..=2 on
    /// the curve on a surface (a geometric surface, or the located surface of a face);
    /// Standard_Failure for another order or a null boundary.
    pub fn on_surface(
        boundary: &Handle<Adaptor3dCurveOnSurface>,
        order: i32,
        npt: i32,
        tols: [f64; 3],
    ) -> Result<Handle<GeomPlateCurveConstraint>, Failure> {
        if !(-1..=2).contains(&order) {
            return Err(Failure::new(&format!(
                "BRepFill : The continuity is not G0 G1 or G2; got order {order}"
            )));
        }
        let Some(frontiere) = boundary.get() else {
            return Err(Failure::new(
                "BRepFill_CurveConstraint : Curve must be on a Surface",
            ));
        };
        let surf: Handle<dyn GeomSurface> = surface_of(frontiere)?;
        Ok(Handle::new(GeomPlateCurveConstraint::from_derived_state(
            boundary, order, npt, tols, &surf,
        )?))
    }

    /// `BRepFill_CurveConstraint(Boundary, Tang, NPt, TolDist)` (natively defaulted to
    /// 10 points and 1e-4): the plate constraint on a 3d curve (orders G0 and -1), with
    /// the default angular and curvature tolerances.
    pub fn with_curve(
        boundary: &Handle<dyn Adaptor3dCurve>,
        tang: i32,
        npt: i32,
        tol_dist: f64,
    ) -> Result<Handle<GeomPlateCurveConstraint>, Failure> {
        Ok(Handle::new(GeomPlateCurveConstraint::new(
            boundary, tang, npt, tol_dist, 0.01, 0.1,
        )?))
    }
}

/// The geometric surface of a curve on surface: that of a GeomAdaptor_Surface, else
/// the located surface of the face of a BRepAdaptor_Surface.
fn surface_of(frontiere: &Adaptor3dCurveOnSurface) -> Result<Handle<dyn GeomSurface>, Failure> {
    let gs1: Handle<GeomAdaptorSurface> = frontiere.get_surface().cast();
    if let Some(gs1) = gs1.get() {
        return Ok(gs1.surface());
    }
    let bs1: Handle<BRepAdaptorSurface> = frontiere.get_surface().cast();
    let bs1 = bs1.get().ok_or_else(|| {
        Failure::with_type("Standard_NullObject", "BRepFill_CurveConstraint: surface adaptor neither GeomAdaptor_Surface nor BRepAdaptor_Surface; expected one of them", "BRepFill_CurveConstraint")
    })?;
    BRepTool::surface(&bs1.face())
}

/// An edge, its support face (null for none) and the continuity order of a filling
/// constraint (`BRepFill_EdgeFaceAndOrder`);
/// `BRepFillEdgeFaceAndOrder::with_edge(&edge, &Face::default(), Continuity::C0)`.
#[derive(Clone, Debug)]
pub struct BRepFillEdgeFaceAndOrder {
    pub(crate) my_edge: Edge,
    pub(crate) my_face: Face,
    pub(crate) my_order: Continuity,
}

impl BRepFillEdgeFaceAndOrder {
    /// `BRepFill_EdgeFaceAndOrder()`: null shapes and C0 (the order is uninitialized
    /// natively); `BRepFillEdgeFaceAndOrder::new()`.
    pub fn new() -> Self {
        Self::with_edge(&Edge::default(), &Face::default(), Continuity::C0)
    }

    /// `BRepFill_EdgeFaceAndOrder(anEdge, aFace, anOrder)`;
    /// `BRepFillEdgeFaceAndOrder::with_edge(&edge, &face, Continuity::G1)`.
    pub fn with_edge(an_edge: &Edge, a_face: &Face, an_order: Continuity) -> Self {
        Self {
            my_edge: an_edge.clone(),
            my_face: a_face.clone(),
            my_order: an_order,
        }
    }
}

/// A support face and the continuity order of a free filling constraint
/// (`BRepFill_FaceAndOrder`); `BRepFillFaceAndOrder::with_face(&face, Continuity::G1)`.
#[derive(Clone, Debug)]
pub struct BRepFillFaceAndOrder {
    pub(crate) my_face: Face,
    pub(crate) my_order: Continuity,
}

impl BRepFillFaceAndOrder {
    /// `BRepFill_FaceAndOrder()`: a null face and C0 (the order is uninitialized
    /// natively); `BRepFillFaceAndOrder::new()`.
    pub fn new() -> Self {
        Self::with_face(&Face::default(), Continuity::C0)
    }

    /// `BRepFill_FaceAndOrder(aFace, anOrder)`;
    /// `BRepFillFaceAndOrder::with_face(&face, Continuity::G1)`.
    pub fn with_face(a_face: &Face, an_order: Continuity) -> Self {
        Self {
            my_face: a_face.clone(),
            my_order: an_order,
        }
    }
}

impl Default for BRepFillEdgeFaceAndOrder {
    /// The empty record; `BRepFillEdgeFaceAndOrder::default()`.
    fn default() -> Self {
        Self::new()
    }
}

impl Default for BRepFillFaceAndOrder {
    /// The empty record; `BRepFillFaceAndOrder::default()`.
    fn default() -> Self {
        Self::new()
    }
}
