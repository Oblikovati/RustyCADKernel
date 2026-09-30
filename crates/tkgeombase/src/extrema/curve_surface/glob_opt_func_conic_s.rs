// Rust translation of Extrema_GlobOptFuncConicS.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Extrema_GlobOptFuncConicS`: `F(u, v)`, the square distance from `S(u, v)` to a
//! trimmed conic (its projection when inside the trimming, and its end points),
//! false outside the surface bounds or for another curve.
use crate::extrema::failures::null_object;
use crate::failure_capture::FailureCapture;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::std_min;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Circle3, Ellipse3, Hyperbola3, Line3, Parabola3, Point3};
use rustycad_tkmath::math::{MultipleVarFunction, Vector};
use std::f64::consts::PI;

/// The conic of a function, `None` for another curve type (`myCType`).
#[derive(Clone, Copy, Debug)]
enum Conic {
    Line(Line3),
    Circle(Circle3),
    Ellipse(Ellipse3),
    Hyperbola(Hyperbola3),
    Parabola(Parabola3),
}

impl Conic {
    // `ElCLib::Parameter(conic, P)`.
    fn parameter(&self, point: Point3) -> Result<f64, Failure> {
        match self {
            Conic::Line(c) => elclib::parameter(c, point),
            Conic::Circle(c) => elclib::parameter(c, point),
            Conic::Ellipse(c) => elclib::parameter(c, point),
            Conic::Hyperbola(c) => elclib::parameter(c, point),
            Conic::Parabola(c) => elclib::parameter(c, point),
        }
    }
    fn is_closed(&self) -> bool {
        matches!(self, Conic::Circle(_) | Conic::Ellipse(_))
    }
}

/// The trimmed conic of LoadConic.
struct LoadedConic<'a> {
    c: &'a dyn Adaptor3dCurve,
    conic: Option<Conic>,
    cpf: Point3,
    cpl: Point3,
    tf: f64,
    tl: f64,
}

/// `Extrema_GlobOptFuncConicS`; `GlobOptFuncConicS::with_curve(&curve, &surface)?`.
pub struct GlobOptFuncConicS<'a> {
    s: &'a dyn Adaptor3dSurface,
    loaded: Option<LoadedConic<'a>>,
    uf: f64,
    ul: f64,
    vf: f64,
    vl: f64,
    failure: FailureCapture,
}

impl<'a> GlobOptFuncConicS<'a> {
    /// `Extrema_GlobOptFuncConicS(S, theUf, theUl, theVf, theVl)`: no conic until
    /// LoadConic (the native members then read an unset pointer);
    /// `GlobOptFuncConicS::with_bounds(&surface, (0., 1.), (0., 1.))`.
    pub fn with_bounds(
        surface: &'a dyn Adaptor3dSurface,
        (uf, ul): (f64, f64),
        (vf, vl): (f64, f64),
    ) -> Self {
        Self {
            s: surface,
            loaded: None,
            uf,
            ul,
            vf,
            vl,
            failure: FailureCapture::default(),
        }
    }
    /// `Extrema_GlobOptFuncConicS(S)`: the surface bounds, no conic;
    /// `GlobOptFuncConicS::new(&surface)?`.
    pub fn new(surface: &'a dyn Adaptor3dSurface) -> Result<Self, Failure> {
        let u = (surface.first_u_parameter()?, surface.last_u_parameter()?);
        let v = (surface.first_v_parameter()?, surface.last_v_parameter()?);
        Ok(Self::with_bounds(surface, u, v))
    }
    /// `Extrema_GlobOptFuncConicS(C, S)`: the surface bounds and the whole conic;
    /// `GlobOptFuncConicS::with_curve(&curve, &surface)?`.
    pub fn with_curve(
        curve: &'a dyn Adaptor3dCurve,
        surface: &'a dyn Adaptor3dSurface,
    ) -> Result<Self, Failure> {
        let mut function: Self = Self::new(surface)?;
        function.load_conic(curve, curve.first_parameter()?, curve.last_parameter()?)?;
        Ok(function)
    }
    /// `LoadConic(C, theTf, theTl)`: the range (brought into `[0, 2 PI]` for a
    /// periodic curve), its end points and the conic;
    /// `function.load_conic(&curve, 0.0, 1.0)?`.
    pub fn load_conic(
        &mut self,
        curve: &'a dyn Adaptor3dCurve,
        tf: f64,
        tl: f64,
    ) -> Result<(), Failure> {
        let (mut tf, mut tl) = (tf, tl);
        if curve.is_periodic()? {
            let t_max: f64 = 2. * PI + precision::p_confusion();
            if tf > t_max || tf < -precision::p_confusion() || (tl - tf).abs() > t_max {
                let precision_value: f64 = std_min((tl - tf).abs() / 2., precision::p_confusion());
                (tf, tl) = elclib::adjust_periodic(0., 2. * PI, precision_value, tf, tl);
            }
        }
        let (cpf, cpl) = (curve.value(tf)?, curve.value(tl)?);
        let conic: Option<Conic> = match curve.get_type()? {
            CurveType::Line => Some(Conic::Line(curve.line()?)),
            CurveType::Circle => Some(Conic::Circle(curve.circle()?)),
            CurveType::Ellipse => Some(Conic::Ellipse(curve.ellipse()?)),
            CurveType::Hyperbola => Some(Conic::Hyperbola(curve.hyperbola()?)),
            CurveType::Parabola => Some(Conic::Parabola(curve.parabola()?)),
            _ => None,
        };
        self.loaded = Some(LoadedConic {
            c: curve,
            conic,
            cpf,
            cpl,
            tf,
            tl,
        });
        Ok(())
    }
    /// `NbVariables()`: 2; `function.nb_variables()`.
    pub fn nb_variables(&self) -> i32 {
        2
    }
    fn loaded(&self) -> Result<&LoadedConic<'a>, Failure> {
        self.loaded
            .as_ref()
            .ok_or_else(|| null_object("Extrema_GlobOptFuncConicS - no conic loaded"))
    }
    // The conic parameter of the projection of `ps`, shifted by 2 PI for a closed
    // conic trimmed beyond 2 PI; None for another curve.
    fn projection(&self, ps: Point3) -> Result<Option<f64>, Failure> {
        let loaded = self.loaded()?;
        let Some(conic) = loaded.conic else {
            return Ok(None);
        };
        let mut ct: f64 = conic.parameter(ps)?;
        if conic.is_closed() && loaded.tl > 2. * PI + precision::p_confusion() {
            ct += 2. * PI;
        }
        Ok(Some(ct))
    }
    // `value(su, sv, F)`.
    fn value_at(&self, su: f64, sv: f64) -> Result<f64, Failure> {
        let ps: Point3 = self.s.value(su, sv)?;
        let Some(ct) = self.projection(ps)? else {
            return Ok(precision::infinite());
        };
        let loaded = self.loaded()?;
        let mut f: f64 = f64::MAX;
        if ct >= loaded.tf && ct <= loaded.tl {
            f = std_min(f, ps.square_distance(&loaded.c.value(ct)?));
        }
        f = std_min(f, ps.square_distance(&loaded.cpf));
        Ok(std_min(f, ps.square_distance(&loaded.cpl)))
    }
    /// `Value(X, F)`: false outside the bounds or for an infinite value;
    /// `function.value(&x, &mut f)?`.
    pub fn value(&self, x: &Vector<'_>, f: &mut f64) -> Result<bool, Failure> {
        let start: i32 = x.lower();
        let (su, sv) = (x[start], x[start + 1]);
        if !(su >= self.uf && su <= self.ul && sv >= self.vf && sv <= self.vl) {
            return Ok(false);
        }
        *f = self.value_at(su, sv)?;
        Ok(!precision::is_infinite(*f))
    }
    /// `ConicParameter(theUV)`: the conic parameter of the point of the trimmed conic
    /// nearest to `S(u, v)` (an end when it is nearer); `function.conic_parameter(&uv)?`.
    pub fn conic_parameter(&self, uv: &Vector<'_>) -> Result<f64, Failure> {
        let ps: Point3 = self.s.value(uv[1], uv[2])?;
        let loaded = self.loaded()?;
        let Some(mut ct) = self.projection(ps)? else {
            return Ok(loaded.tf);
        };
        let mut f: f64 = f64::MAX;
        if ct >= loaded.tf && ct <= loaded.tl {
            f = std_min(f, ps.square_distance(&loaded.c.value(ct)?));
        }
        let fext: f64 = ps.square_distance(&loaded.cpf);
        if fext < f {
            f = fext;
            ct = loaded.tf;
        }
        if ps.square_distance(&loaded.cpl) < f {
            ct = loaded.tl;
        }
        Ok(ct)
    }
    /// The first failure a math callback kept since the last call;
    /// `function.take_failure()?`.
    pub fn take_failure(&mut self) -> Result<(), Failure> {
        self.failure.take()
    }
}

impl MultipleVarFunction for GlobOptFuncConicS<'_> {
    fn variables(&self) -> i32 {
        2
    }
    fn value(&mut self, point: &Vector<'_>, value: &mut f64) -> bool {
        let result = GlobOptFuncConicS::value(self, point, value);
        self.failure.keep(result).unwrap_or(false)
    }
}
