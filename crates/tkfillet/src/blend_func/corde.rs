// Rust translation of BlendFunc_Corde.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The chord of a chamfer: the point of a surface in the plane normal to the guide at a
//! given distance from the guide point.
use super::chamfer_plane::ChamferPlane;
use super::linalg::{mset, vector, vget, vset};
use super::package::live_surface;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkmath::gp::{Point3, Vector2, Vector3};
use rustycad_tkmath::math::{Gauss, Matrix, Vector};

/// `BlendFunc_Corde`: the equations, of variables `(U, V)` on the surface, of the point at
/// the distance `dis` from the guide point in the plane normal to the guide:
/// `F(1) = nplan.P + theD` and `F(2) = |P - Pgui|^2 - dis^2`. The guide frame is set by
/// `SetParam`; `IsSolution` computes the tangent along the guide.
///
/// ```ignore
/// let mut corde = Corde::new(&surface, &guide);
/// corde.set_dist(0.5);
/// corde.set_param(1.0)?;
/// ```
pub struct Corde {
    surf: Handle<dyn Adaptor3dSurface>,
    guide: Handle<dyn Adaptor3dCurve>,
    pts: Point3,
    dis: f64,
    plane: ChamferPlane,
    tgs: Vector3,
    tg2d: Vector2,
    istangent: bool,
}

impl Corde {
    /// `BlendFunc_Corde(S, CG)`; `Corde::new(&surface, &guide)`.
    pub fn new(s: &Handle<dyn Adaptor3dSurface>, cg: &Handle<dyn Adaptor3dCurve>) -> Self {
        Self {
            surf: s.clone(),
            guide: cg.clone(),
            pts: Point3::default(),
            dis: 0.0,
            plane: ChamferPlane::default(),
            tgs: Vector3::default(),
            tg2d: Vector2::default(),
            istangent: false,
        }
    }

    /// `SetDist(Dist)`: the distance to the guide point; `corde.set_dist(0.5)`.
    pub fn set_dist(&mut self, dist: f64) {
        self.dis = dist;
    }

    /// `SetParam(Param)`: the guide point, the plane normal and its offset at `param`;
    /// `corde.set_param(1.0)?`.
    pub fn set_param(&mut self, param: f64) -> Result<(), Failure> {
        self.plane.set(&self.guide, param)
    }

    /// `Value(X, F)`: the two equations at `(U, V) = (X(1), X(2))`; `corde.value(&x, &mut f)?`.
    pub fn value(&mut self, x: &Vector<'_>, f: &mut Vector<'_>) -> Result<bool, Failure> {
        self.pts = live_surface(&self.surf)?
            .d1(vget(x, 1)?, vget(x, 2)?)?
            .point;
        vset(f, 1, self.plane.offset(self.pts))?;
        let vref = Vector3::between(self.plane.ptgui, self.pts);
        vset(f, 2, vref.square_magnitude() - self.dis * self.dis)?;
        Ok(true)
    }

    /// `Derivatives(X, D)`: the 2 x 2 Jacobian; `corde.derivatives(&x, &mut d)?`.
    pub fn derivatives(&mut self, x: &Vector<'_>, d: &mut Matrix<'_>) -> Result<bool, Failure> {
        let d1 = live_surface(&self.surf)?.d1(vget(x, 1)?, vget(x, 2)?)?;
        self.pts = d1.point;
        mset(d, 1, 1, self.plane.nplan.dot(d1.d1u))?;
        mset(d, 1, 2, self.plane.nplan.dot(d1.d1v))?;
        mset(
            d,
            2,
            1,
            2. * Vector3::between(self.plane.ptgui, self.pts).dot(d1.d1u),
        )?;
        mset(
            d,
            2,
            2,
            2. * Vector3::between(self.plane.ptgui, self.pts).dot(d1.d1v),
        )?;
        Ok(true)
    }

    /// `PointOnS()`: the last surface point evaluated.
    pub fn point_on_s(&self) -> Point3 {
        self.pts
    }

    /// `PointOnGuide()`: the guide point of the last `SetParam`.
    pub fn point_on_guide(&self) -> Point3 {
        self.plane.ptgui
    }

    /// `NPlan()`: the normal of the guide plane of the last `SetParam`.
    pub fn n_plan(&self) -> Vector3 {
        self.plane.nplan
    }

    /// `IsTangencyPoint()`: whether the last `IsSolution` could not compute the tangent.
    pub fn is_tangency_point(&self) -> bool {
        self.istangent
    }

    /// `TangentOnS()`: Standard_DomainError on a tangency point; `corde.tangent_on_s()?`.
    pub fn tangent_on_s(&self) -> Result<Vector3, Failure> {
        self.when_not_tangent(self.tgs, "BlendFunc_Corde::TangentOnS")
    }

    /// `Tangent2dOnS()`: Standard_DomainError on a tangency point.
    pub fn tangent2d_on_s(&self) -> Result<Vector2, Failure> {
        self.when_not_tangent(self.tg2d, "BlendFunc_Corde::Tangent2dOnS")
    }

    fn when_not_tangent<T>(&self, value: T, member: &str) -> Result<T, Failure> {
        if self.istangent {
            return Err(Failure::with_message("Standard_DomainError", member));
        }
        Ok(value)
    }

    /// `DerFguide(Sol, DerF)`: the derivative of the equations along the guide at `sol`;
    /// `let derf = corde.der_f_guide(&sol)?;`.
    pub fn der_f_guide(&mut self, sol: &Vector<'_>) -> Result<Vector2, Failure> {
        self.pts = live_surface(&self.surf)?
            .d1(vget(sol, 1)?, vget(sol, 2)?)?
            .point;
        let dnplan = self.plane.dnplan();
        let temp = Vector3::from(self.pts.xyz() - self.plane.ptgui.xyz());
        Ok(Vector2::new(
            dnplan.dot(temp) - self.plane.nplan.dot(self.plane.d1gui),
            -2. * self.plane.d1gui.dot(temp),
        ))
    }

    /// `IsSolution(Sol, Tol)`: whether `|F(1)| <= tol` and `|F(2)| <= tol^2`; for a
    /// solution, the tangent along the guide by Gauss (a failed decomposition marks a
    /// tangency point); `corde.is_solution(&sol, 1e-7)?`.
    pub fn is_solution(&mut self, sol: &Vector<'_>, tol: f64) -> Result<bool, Failure> {
        let mut valsol = vector(2)?;
        let mut gradsol = super::linalg::matrix(2, 2)?;
        self.value(sol, &mut valsol)?;
        self.derivatives(sol, &mut gradsol)?;
        if vget(&valsol, 1)?.abs() > tol || vget(&valsol, 2)?.abs() > tol * tol {
            return Ok(false);
        }
        let d1 = live_surface(&self.surf)?.d1(vget(sol, 1)?, vget(sol, 2)?)?;
        self.pts = d1.point;
        let dnplan = self.plane.dnplan();
        let temp = Vector3::from(self.pts.xyz() - self.plane.ptgui.xyz());
        let mut secmember = vector(2)?;
        vset(
            &mut secmember,
            1,
            self.plane.nplan.dot(self.plane.d1gui) - dnplan.dot(temp),
        )?;
        vset(&mut secmember, 2, 2. * self.plane.d1gui.dot(temp))?;
        // gradsol * der = secmember, der(1) = dU/dW, der(2) = dV/dW, W the guide parameter.
        let resol = Gauss::new(&gradsol)?;
        self.istangent = !resol.is_done();
        if resol.is_done() {
            resol.solve(&mut secmember)?;
            let (du, dv) = (vget(&secmember, 1)?, vget(&secmember, 2)?);
            self.tgs.set_linear_form(du, d1.d1u, dv, d1.d1v);
            self.tg2d = Vector2::new(du, dv);
        }
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustycad_tkg3d::geom::{GeomCurve, GeomLine, GeomPlane, GeomSurface};
    use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
    use rustycad_tkmath::gp::{Axis1, Axis3, Direction3};

    /// The plane z = 0 and the X axis as guide.
    fn plane_along_x() -> Corde {
        let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
        let surface: Handle<dyn Adaptor3dSurface> =
            Handle::new(GeomAdaptorSurface::with_surface(&plane).unwrap()).cast();
        let x_axis = Axis1::new(Point3::default(), Direction3::new(1., 0., 0.).unwrap());
        let line: Handle<dyn GeomCurve> = Handle::new(GeomLine::new(x_axis)).cast();
        let guide: Handle<dyn Adaptor3dCurve> =
            Handle::new(GeomAdaptorCurve::with_curve(&line).unwrap()).cast();
        Corde::new(&surface, &guide)
    }

    #[test]
    fn the_point_at_the_distance_is_a_solution() {
        let mut corde = plane_along_x();
        corde.set_dist(0.5);
        corde.set_param(2.0).unwrap();
        let mut sol = vector(2).unwrap();
        vset(&mut sol, 1, 2.0).unwrap();
        vset(&mut sol, 2, 0.5).unwrap();
        assert!(corde.is_solution(&sol, 1e-9).unwrap());
        assert!(!corde.is_tangency_point());
        assert_eq!(corde.point_on_s(), Point3::new(2.0, 0.5, 0.0));
        // Moving along the guide moves the chord point with it.
        assert_eq!(corde.tangent2d_on_s().unwrap(), Vector2::new(1.0, 0.0));
    }

    #[test]
    fn off_solution_points_are_refused() {
        let mut corde = plane_along_x();
        corde.set_dist(0.5);
        corde.set_param(0.0).unwrap();
        let mut sol = vector(2).unwrap();
        vset(&mut sol, 2, 0.6).unwrap();
        assert!(!corde.is_solution(&sol, 1e-3).unwrap());
        let derf = corde.der_f_guide(&sol).unwrap();
        assert_eq!(derf, Vector2::new(-1.0, 0.0));
    }
}
