// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_FunctionGuide.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomFill_FunctionGuide`: the system `G(w) - S(u, v) = 0` between the guide `G`
//! and the surface of revolution `S` of the section placed at a path point.
use super::section_law::GeomFillSectionLaw;
use super::sweep_failures::live;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::one_based;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomCurve, GeomSurface, GeomSurfaceOfRevolution, GeomTrimmedCurve,
};
use rustycad_tkgeombase::failure_capture::FailureCapture;
use rustycad_tkmath::gp::{self, Axis1, Axis3, Direction3, Point3, Transform3, Vector3, Xyz};
use rustycad_tkmath::math::{FunctionSet, FunctionSetWithDerivatives, Matrix, Vector};

/// `GeomFill_FunctionGuide`: a `math_FunctionSetWithDerivatives` of `(w, u, v)`.
///
/// A callback exception is kept and the callback answers false;
/// [`GeomFillFunctionGuide::take_failure`] raises it after the solver. `Deriv2T` is
/// declared natively but never defined, so it has no port.
/// `let mut f = GeomFillFunctionGuide::new(&law, &guide, 0.0)?; f.set_param(0.0, p, d, x)?;`.
pub struct GeomFillFunctionGuide {
    the_guide: Handle<dyn Adaptor3dCurve>,
    the_law: Handle<dyn GeomFillSectionLaw>,
    isconst: bool,
    the_curve: Handle<dyn GeomCurve>,
    the_const: Handle<dyn GeomCurve>,
    the_surface: Handle<dyn GeomSurface>,
    first: f64,
    last: f64,
    the_u_on_s: f64,
    centre: Xyz,
    dir: Xyz,
    capture: FailureCapture,
}

fn coords(v: Vector3) -> [f64; 3] {
    [v.x(), v.y(), v.z()]
}

impl GeomFillFunctionGuide {
    /// `GeomFill_FunctionGuide(S, Guide, ParamOnLaw = 0)`: a constant section is kept
    /// with its bounds, another one is rebuilt at `param_on_law` by `SetParam`.
    pub fn new(
        s: &Handle<dyn GeomFillSectionLaw>,
        guide: &Handle<dyn Adaptor3dCurve>,
        param_on_law: f64,
    ) -> Result<Self, Failure> {
        let law = live(s, "GeomFill_FunctionGuide", "TheLaw")?;
        let mut function = Self {
            the_guide: guide.clone(),
            the_law: s.clone(),
            isconst: false,
            the_curve: Handle::default(),
            the_const: Handle::default(),
            the_surface: Handle::default(),
            first: 0.0,
            last: 0.0,
            the_u_on_s: param_on_law,
            centre: Xyz::default(),
            dir: Xyz::default(),
            capture: FailureCapture::default(),
        };
        let mut tol = precision::confusion();
        if law.is_constant(&mut tol)? {
            function.isconst = true;
            function.the_const = law.constant_section()?;
            let c = live(&function.the_const, "GeomFill_FunctionGuide", "TheConst")?;
            function.first = c.first_parameter();
            function.last = c.last_parameter();
        }
        Ok(function)
    }

    /// The exception a solver callback met, if any; `f.take_failure()?`.
    pub fn take_failure(&mut self) -> Result<(), Failure> {
        self.capture.take()
    }

    /// The section B-spline at `TheUonS` of a varying law.
    fn law_section(&self) -> Result<Handle<dyn GeomCurve>, Failure> {
        let law = live(&self.the_law, "GeomFill_FunctionGuide::SetParam", "TheLaw")?;
        let (nb_poles, nb_knots, deg) = law.section_shape()?;
        let mut mult: Array1<'static, i32> = one_based(nb_knots)?;
        law.mults(&mut mult)?;
        let mut knots: Array1<'static, f64> = one_based(nb_knots)?;
        law.knots(&mut knots)?;
        let mut poles = one_based(nb_poles)?;
        let mut weights: Array1<'static, f64> = one_based(nb_poles)?;
        law.d0(self.the_u_on_s, &mut poles, &mut weights)?;
        let curve = if law.is_rational()? {
            GeomBSplineCurve::with_weights(
                &poles,
                &weights,
                &knots,
                &mult,
                deg,
                law.is_u_periodic()?,
            )?
        } else {
            GeomBSplineCurve::new(&poles, &knots, &mult, deg, law.is_u_periodic()?)?
        };
        Ok(Handle::new(curve).cast())
    }

    /// `SetParam(Param, Centre, Dir, XDir)`: the surface of revolution of the
    /// section placed in the frame `(Centre, Dir, XDir)`.
    pub fn set_param(&mut self, _param: f64, c: Point3, d: Xyz, dx: Xyz) -> Result<(), Failure> {
        self.centre = c.xyz();
        self.dir = d;
        let rep = Axis3::new(gp::origin(), gp::dz(), gp::dx())?;
        let b2 = Direction3::from_xyz(dx)?;
        let rep_triedre = Axis3::new(c, Direction3::from_xyz(d)?, b2)?;
        let mut transfo = Transform3::default();
        transfo.set_transformation_between(rep_triedre, rep);
        self.the_curve = if self.isconst {
            let copy: Handle<dyn GeomCurve> = live(
                &self.the_const,
                "GeomFill_FunctionGuide::SetParam",
                "TheConst",
            )?
            .copy()?
            .cast();
            Handle::new(GeomTrimmedCurve::new(
                &copy, self.first, self.last, true, true,
            )?)
            .cast()
        } else {
            self.law_section()?
        };
        let axe = Axis1::new(c, Direction3::from_xyz(self.dir)?);
        live(
            &self.the_curve,
            "GeomFill_FunctionGuide::SetParam",
            "TheCurve",
        )?
        .transform(transfo)?;
        self.the_surface = Handle::new(GeomSurfaceOfRevolution::new(&self.the_curve, axe)?).cast();
        Ok(())
    }

    fn guide(&self) -> Result<&dyn Adaptor3dCurve, Failure> {
        live(&self.the_guide, "GeomFill_FunctionGuide", "TheGuide")
    }

    fn surface(&self) -> Result<&dyn GeomSurface, Failure> {
        live(&self.the_surface, "GeomFill_FunctionGuide", "TheSurface")
    }

    /// `Value(X, F)`: `G(X(1)) - S(X(2), X(3))`.
    pub fn try_value(&self, x: &Vector<'_>, f: &mut Vector<'_>) -> Result<bool, Failure> {
        let p = self.guide()?.d0(*x.value(1)?)?;
        let p1 = self.surface()?.d0(*x.value(2)?, *x.value(3)?)?;
        *f.change_value(1)? = p.x() - p1.x();
        *f.change_value(2)? = p.y() - p1.y();
        *f.change_value(3)? = p.z() - p1.z();
        Ok(true)
    }

    /// `Derivatives(X, D)`: `[G', -S_u, -S_v]`.
    pub fn try_derivatives(&self, x: &Vector<'_>, d: &mut Matrix<'_>) -> Result<bool, Failure> {
        let (_, dp) = self.guide()?.d1(*x.value(1)?)?;
        let s = self.surface()?.d1(*x.value(2)?, *x.value(3)?)?;
        let (dp, du, dv) = (coords(dp), coords(s.d1u), coords(s.d1v));
        for i in 0..3 {
            let row = i as i32 + 1;
            *d.change_value(row, 1)? = dp[i];
            *d.change_value(row, 2)? = -du[i];
            *d.change_value(row, 3)? = -dv[i];
        }
        Ok(true)
    }

    /// `Values(X, F, D)`.
    pub fn try_values(
        &self,
        x: &Vector<'_>,
        f: &mut Vector<'_>,
        d: &mut Matrix<'_>,
    ) -> Result<bool, Failure> {
        let (p, dp) = self.guide()?.d1(*x.value(1)?)?;
        let s = self.surface()?.d1(*x.value(2)?, *x.value(3)?)?;
        let (pc, p1) = (
            [p.x(), p.y(), p.z()],
            [s.point.x(), s.point.y(), s.point.z()],
        );
        let (dp, du, dv) = (coords(dp), coords(s.d1u), coords(s.d1v));
        for i in 0..3 {
            let row = i as i32 + 1;
            *f.change_value(row)? = pc[i] - p1[i];
            *d.change_value(row, 1)? = dp[i];
            *d.change_value(row, 2)? = -du[i];
            *d.change_value(row, 3)? = -dv[i];
        }
        Ok(true)
    }

    /// `DerivT(X, DCentre, DDir, DFDT)`: `C(X(1)) - dS/dt` (the section curve at
    /// `X(1)`, as the native code evaluates it).
    pub fn deriv_t(
        &self,
        x: &Vector<'_>,
        d_centre: Xyz,
        d_dir: Xyz,
        dfdt: &mut Vector<'_>,
    ) -> Result<bool, Failure> {
        let ds = self.dsdt(*x.value(2)?, *x.value(3)?, d_centre, d_dir)?;
        let p = live(
            &self.the_curve,
            "GeomFill_FunctionGuide::DerivT",
            "TheCurve",
        )?
        .d0(*x.value(1)?)?;
        *dfdt.change_value(1)? = p.x() - ds.x();
        *dfdt.change_value(2)? = p.y() - ds.y();
        *dfdt.change_value(3)? = p.z() - ds.z();
        Ok(true)
    }

    /// `DSDT(U, V, DCentre, DDir, DS)`: the motion of the point `(U, V)` of the
    /// surface of revolution. A varying section prints "Not implemented" on the
    /// standard output first, as natively.
    fn dsdt(&self, u: f64, v: f64, dc: Xyz, d_dir: Xyz) -> Result<Vector3, Failure> {
        let pc = live(&self.the_curve, "GeomFill_FunctionGuide::DSDT", "TheCurve")?.d0(v)?;
        let (mut q, mut dq) = (pc.xyz(), Xyz::new(0.0, 0.0, 0.0));
        if !self.isconst {
            println!("Not implemented");
        }
        q -= self.centre;
        dq -= dc;
        let mut dv_cross_cq = Xyz::default();
        dv_cross_cq.set_sum(d_dir.crossed(q), self.dir.crossed(dq));
        dv_cross_cq *= u.sin();
        let cos_u = u.cos();
        let mut dv_dot_cq = Xyz::default();
        dv_dot_cq.set_linear_form(
            d_dir.dot(q) + self.dir.dot(dq),
            self.dir,
            self.dir.dot(q),
            d_dir,
        );
        dv_dot_cq += dv_cross_cq;
        dq *= cos_u;
        dq += dv_dot_cq;
        dq += dc;
        Ok(Vector3::from(dq))
    }
}

impl FunctionSet for GeomFillFunctionGuide {
    /// Always 3.
    fn variables(&self) -> i32 {
        3
    }
    /// Always 3.
    fn equations(&self) -> i32 {
        3
    }
    fn value(&mut self, point: &Vector<'_>, residual: &mut Vector<'_>) -> bool {
        let result = self.try_value(point, residual);
        self.capture.keep(result).unwrap_or(false)
    }
}

impl FunctionSetWithDerivatives for GeomFillFunctionGuide {
    fn derivatives(&mut self, point: &Vector<'_>, jacobian: &mut Matrix<'_>) -> bool {
        let result = self.try_derivatives(point, jacobian);
        self.capture.keep(result).unwrap_or(false)
    }
    fn values(
        &mut self,
        point: &Vector<'_>,
        residual: &mut Vector<'_>,
        jacobian: &mut Matrix<'_>,
    ) -> bool {
        let result = self.try_values(point, residual, jacobian);
        self.capture.keep(result).unwrap_or(false)
    }
}
