// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_BoundWithSurf.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomFill_BoundWithSurf`: a boundary on a curve on a surface, whose normals are
//! the surface normals.
use super::bound_parameter::BoundParameter;
use super::{GeomFillBoundary, GeomFillBoundaryTolerances};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::errors;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dCurveOnSurface, Adaptor3dSurface};
use rustycad_tkg3d::geom::GeomSurfaceD2;
use rustycad_tkmath::gp::{Point2, Point3, Vector2, Vector3};

/// A boundary on a curve on a surface (`GeomFill_BoundWithSurf`): its normals give
/// the filling a tangency constraint.
///
/// `let bound = GeomFillBoundWithSurf::new(&curve_on_surface, 1e-4, 1e-2); bound.norm(0.5)?`
pub struct GeomFillBoundWithSurf {
    parent: StandardTransient,
    tolerances: GeomFillBoundaryTolerances,
    my_con_s: Adaptor3dCurveOnSurface,
    my_par: BoundParameter,
}
impl_transient!(
    GeomFillBoundWithSurf,
    "GeomFill_BoundWithSurf",
    dyn GeomFillBoundary,
    parent,
    interfaces: [dyn GeomFillBoundary]
);

impl GeomFillBoundWithSurf {
    /// `GeomFill_BoundWithSurf(CurveOnSurf, Tol3d, Tolang)`: the boundary keeps a copy
    /// of the adaptor (sharing its curve and surface);
    /// `GeomFillBoundWithSurf::new(&curve_on_surface, 1e-4, 1e-2)`.
    pub fn new(curve_on_surf: &Adaptor3dCurveOnSurface, tol3d: f64, tolang: f64) -> Self {
        Self {
            parent: StandardTransient::default(),
            tolerances: GeomFillBoundaryTolerances::new(tol3d, tolang),
            my_con_s: Adaptor3dCurveOnSurface::copy_of(curve_on_surf),
            my_par: BoundParameter::default(),
        }
    }
    /// `myConS.GetCurve()`, or the Standard_NullObject of a null one.
    fn curve2d(&self) -> Result<Handle<dyn Adaptor2dCurve2d>, Failure> {
        let curve = self.my_con_s.get_curve();
        if curve.is_null() {
            return Err(errors::null_object("GeomFill_BoundWithSurf: no 2d curve"));
        }
        Ok(curve)
    }
    /// `myConS.GetSurface()`, or the Standard_NullObject of a null one.
    fn surface(&self) -> Result<Handle<dyn Adaptor3dSurface>, Failure> {
        let surface = self.my_con_s.get_surface();
        if surface.is_null() {
            return Err(errors::null_object("GeomFill_BoundWithSurf: no surface"));
        }
        Ok(surface)
    }
}

/// The derivative of the unit normal from the second derivatives of the surface
/// along the 2d direction `(dx, dy)` (the regular branch of `D1Norm`).
fn normal_derivative(n: Vector3, s: &GeomSurfaceD2, (dx, dy): (f64, f64)) -> Vector3 {
    let (su, sv) = (s.d1u, s.d1v);
    let (nsuu, nsuv, nsvv) = (n.dot(s.d2u), n.dot(s.d2uv), n.dot(s.d2v));
    let (susu, susv, svsv) = (su.dot(su), su.dot(sv), sv.dot(sv));
    let deno: f64 = (susu * svsv) - (susv * susv);
    let a: f64 = (-nsuu * svsv + nsuv * susv) / deno;
    let b: f64 = (nsuu * susv - nsuv * susu) / deno;
    let c: f64 = (-nsuv * svsv + nsvv * susv) / deno;
    let d: f64 = (nsuv * susv - nsvv * susu) / deno;
    let temp1: Vector3 = su * a + sv * b;
    let temp2: Vector3 = su * c + sv * d;
    temp1 * dx + temp2 * dy
}

/// `(susu * svsv) - (susv * susv)`, the Gram determinant of the tangents.
fn gram(s: &GeomSurfaceD2) -> f64 {
    let (su, sv) = (s.d1u, s.d1v);
    (su.dot(su) * sv.dot(sv)) - (su.dot(sv) * su.dot(sv))
}

impl GeomFillBoundary for GeomFillBoundWithSurf {
    fn value(&self, u: f64) -> Result<Point3, Failure> {
        let x: f64 = self.my_par.value(u)?;
        self.my_con_s.value(x)
    }
    fn d1(&self, u: f64) -> Result<(Point3, Vector3), Failure> {
        let (x, dx) = self.my_par.d1(u)?;
        let (p, v) = self.my_con_s.d1(x)?;
        Ok((p, v * dx))
    }
    /// Always true (the constraint surface gives the normals).
    fn has_normals(&self) -> bool {
        true
    }
    /// The unit normal of the surface at the 2d point of `myPar(U)`; a degenerate
    /// surface point raises gp_VectorWithNullMagnitude (Normalize).
    fn norm(&self, u: f64) -> Result<Vector3, Failure> {
        let w: f64 = self.my_par.value(u)?;
        let uv: Point2 = self.curve2d()?.value(w)?;
        let s = self.surface()?.d1(uv.x(), uv.y())?;
        let mut su: Vector3 = s.d1u;
        su.cross(s.d1v);
        su.normalize()?;
        Ok(su)
    }
    /// The normal and its derivative along the boundary. Quirk kept: when the Gram
    /// determinant is below 1e-16 the derivative is `(Norm(U + 1e-12) - N) * 1e-12`
    /// (a product where a difference quotient divides).
    fn d1_norm(&self, u: f64) -> Result<(Vector3, Vector3), Failure> {
        let (w, dw) = self.my_par.d1(u)?;
        let (p2d, v2d): (Point2, Vector2) = self.curve2d()?.d1(w)?;
        let v2d: Vector2 = v2d * dw;
        let s: GeomSurfaceD2 = self.surface()?.d2(p2d.x(), p2d.y())?;
        let mut n: Vector3 = s.d1u.crossed(s.d1v);
        n.normalize()?;
        if gram(&s).abs() < 1.0e-16 {
            let temp: Vector3 = self.norm(u + 1.0e-12)?;
            let dn: Vector3 = (n * -1. + temp) * 1.0e-12;
            return Ok((n, dn));
        }
        Ok((n, normal_derivative(n, &s, (v2d.x(), v2d.y()))))
    }
    fn reparametrize(
        &self,
        first: f64,
        last: f64,
        has_df: bool,
        has_dl: bool,
        df: f64,
        dl: f64,
        rev: bool,
    ) -> Result<(), Failure> {
        self.my_par
            .reparametrize(&self.my_con_s, first, last, (has_df, has_dl), (df, dl), rev)
    }
    fn bounds(&self) -> Result<(f64, f64), Failure> {
        self.my_par.bounds(|| {
            Ok((
                self.my_con_s.first_parameter()?,
                self.my_con_s.last_parameter()?,
            ))
        })
    }
    fn is_degenerated(&self) -> bool {
        false
    }
    fn tolerances(&self) -> &GeomFillBoundaryTolerances {
        &self.tolerances
    }
}
