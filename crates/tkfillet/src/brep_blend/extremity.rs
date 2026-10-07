// Rust translation of BRepBlend_Extremity.hxx/.lxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepBlend_Extremity`: an end of a blend line on a surface or a restriction.
use super::BRepBlendPointOnRst;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg3d::adaptor3d::Adaptor3dHVertex;
use rustycad_tkgeomalgo::int_surf::IntSurfTransition;
use rustycad_tkmath::gp::{Point3, Vector3};

/// `BRepBlend_Extremity`: the point, its parameters (on a surface `(U, V)`, on a curve
/// `W` in `U`), the parameter on the guide, the tolerance, the vertex it may be, an
/// optional tangent, and the restriction arcs it lies on.
///
/// ```
/// use rustycad_tkfillet::brep_blend::BRepBlendExtremity;
/// use rustycad_tkmath::gp::Point3;
/// let end = BRepBlendExtremity::on_curve(Point3::new(1.0, 2.0, 3.0), 0.5, 2.0, 1e-7);
/// assert_eq!((end.parameter(), end.parameter_on_guide()), (0.5, 2.0));
/// assert!(end.tangent().is_err());
/// ```
#[derive(Clone, Default)]
pub struct BRepBlendExtremity {
    vtx: Handle<dyn Adaptor3dHVertex>,
    seqpt: Vec<BRepBlendPointOnRst>,
    pt: Point3,
    tang: Vector3,
    param: f64,
    u: f64,
    v: f64,
    tol: f64,
    isvtx: bool,
    hastang: bool,
}

impl BRepBlendExtremity {
    /// `BRepBlend_Extremity()`: the origin, no vertex, no tangent.
    pub fn new() -> Self {
        Self::default()
    }

    /// `BRepBlend_Extremity(P, U, V, Param, Tol)`: a point of a surface;
    /// `BRepBlendExtremity::on_surface(p, u, v, param, 1e-7)`.
    pub fn on_surface(p: Point3, u: f64, v: f64, param: f64, tol: f64) -> Self {
        Self {
            pt: p,
            u,
            v,
            param,
            tol,
            ..Self::default()
        }
    }

    /// `BRepBlend_Extremity(P, U, V, Param, Tol, Vtx)`: a vertex of a surface;
    /// `BRepBlendExtremity::on_vertex(p, u, v, param, 1e-7, &vertex)`.
    pub fn on_vertex(
        p: Point3,
        u: f64,
        v: f64,
        param: f64,
        tol: f64,
        vtx: &Handle<dyn Adaptor3dHVertex>,
    ) -> Self {
        Self {
            vtx: vtx.clone(),
            isvtx: true,
            ..Self::on_surface(p, u, v, param, tol)
        }
    }

    /// `BRepBlend_Extremity(P, W, Param, Tol)`: a point of a curve, `W` kept in `U`;
    /// `BRepBlendExtremity::on_curve(p, w, param, 1e-7)`.
    pub fn on_curve(p: Point3, w: f64, param: f64, tol: f64) -> Self {
        Self::on_surface(p, w, 0.0, param, tol)
    }

    /// `SetValue(P, U, V, Param, Tol)`: a point of a surface; the arcs are cleared, the
    /// vertex flag reset, the tangent kept.
    pub fn set_value(&mut self, p: Point3, u: f64, v: f64, param: f64, tol: f64) {
        (self.pt, self.u, self.v, self.param, self.tol) = (p, u, v, param, tol);
        self.isvtx = false;
        self.seqpt.clear();
    }

    /// `SetValue(P, U, V, Param, Tol, Vtx)`: a vertex of a surface.
    pub fn set_value_on_vertex(
        &mut self,
        p: Point3,
        (u, v): (f64, f64),
        param: f64,
        tol: f64,
        vtx: &Handle<dyn Adaptor3dHVertex>,
    ) {
        self.set_value(p, u, v, param, tol);
        self.isvtx = true;
        self.vtx = vtx.clone();
    }

    /// `SetValue(P, W, Param, Tol)`: a point of a curve; `V` is kept, as natively.
    pub fn set_value_on_curve(&mut self, p: Point3, w: f64, param: f64, tol: f64) {
        (self.pt, self.u, self.param, self.tol) = (p, w, param, tol);
        self.isvtx = false;
        self.seqpt.clear();
    }

    /// `SetVertex(V)`: marks the extremity as the vertex `v`.
    pub fn set_vertex(&mut self, v: &Handle<dyn Adaptor3dHVertex>) {
        self.isvtx = true;
        self.vtx = v.clone();
    }

    /// `AddArc(A, Param, TLine, TArc)`: one more restriction arc through the extremity.
    pub fn add_arc(
        &mut self,
        a: &Handle<dyn Adaptor2dCurve2d>,
        param: f64,
        t_line: IntSurfTransition,
        t_arc: IntSurfTransition,
    ) {
        self.seqpt
            .push(BRepBlendPointOnRst::with_arc(a, param, t_line, t_arc));
    }

    /// `Value()`.
    pub fn value(&self) -> Point3 {
        self.pt
    }

    /// `SetTangent(Tangent)`.
    pub fn set_tangent(&mut self, tangent: Vector3) {
        self.hastang = true;
        self.tang = tangent;
    }

    /// `HasTangent()`.
    pub fn has_tangent(&self) -> bool {
        self.hastang
    }

    /// `Tangent()`: Standard_DomainError without a tangent.
    pub fn tangent(&self) -> Result<Vector3, Failure> {
        if !self.hastang {
            return Err(Failure::with_message(
                "Standard_DomainError",
                "BRepBlend_Extremity::Tangent: no tangent set",
            ));
        }
        Ok(self.tang)
    }

    /// `Parameters(U, V)`.
    pub fn parameters(&self) -> (f64, f64) {
        (self.u, self.v)
    }

    /// `Tolerance()`.
    pub fn tolerance(&self) -> f64 {
        self.tol
    }

    /// `IsVertex()`.
    pub fn is_vertex(&self) -> bool {
        self.isvtx
    }

    /// `Vertex()`: Standard_DomainError when the extremity is not a vertex.
    pub fn vertex(&self) -> Result<&Handle<dyn Adaptor3dHVertex>, Failure> {
        if !self.isvtx {
            return Err(Failure::with_message(
                "Standard_DomainError",
                "BRepBlend_Extremity::Vertex: not a vertex",
            ));
        }
        Ok(&self.vtx)
    }

    /// `NbPointOnRst()`.
    pub fn nb_point_on_rst(&self) -> i32 {
        self.seqpt.len() as i32
    }

    /// `PointOnRst(Index)`: 1-based; Standard_OutOfRange outside `[1, NbPointOnRst]`.
    pub fn point_on_rst(&self, index: i32) -> Result<&BRepBlendPointOnRst, Failure> {
        usize::try_from(index - 1)
            .ok()
            .and_then(|i| self.seqpt.get(i))
            .ok_or_else(|| {
                Failure::out_of_range(&format!(
                    "BRepBlend_Extremity::PointOnRst index {index}; expected one in [1, {}]",
                    self.seqpt.len()
                ))
            })
    }

    /// `Parameter()`: the parameter on a curve (`U`).
    pub fn parameter(&self) -> f64 {
        self.u
    }

    /// `ParameterOnGuide()`.
    pub fn parameter_on_guide(&self) -> f64 {
        self.param
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_value_keeps_the_tangent_and_clears_the_arcs() {
        let mut end = BRepBlendExtremity::on_surface(Point3::default(), 1.0, 2.0, 3.0, 1e-7);
        end.set_tangent(Vector3::new(0.0, 0.0, 1.0));
        end.add_arc(
            &Handle::default(),
            0.5,
            IntSurfTransition::default(),
            IntSurfTransition::default(),
        );
        assert_eq!(end.nb_point_on_rst(), 1);
        end.set_value_on_curve(Point3::default(), 7.0, 3.0, 1e-7);
        assert_eq!((end.nb_point_on_rst(), end.parameters()), (0, (7.0, 2.0)));
        assert!(end.has_tangent());
        assert!(end.point_on_rst(1).is_err());
        assert!(end.vertex().is_err());
    }
}
