//! HLRBRep_Curve.cxx (the projected 2D curve: evaluation, conics, poles), OCCT
//! 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::{BCurveTool, CLProps, Curve};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::real::epsilon;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::{at, put};
use rustycad_tkg3d::geom::GeomBSplineCurve;
use rustycad_tkgeombase::proj_lib::{project_circle_on_plane, project_ellipse_on_plane};
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{
    self, Axis2d, Axis3, Circle2, Direction2, Direction3, Ellipse2, Hyperbola2, Line2, Parabola2,
    Plane3, Point2, Point3, Vector2,
};

impl Curve {
    /// Tangent(AtStart, P, D): the 2D point and tangent at the start (or the end) of
    /// the 3D range, through the local properties of order 2 (when the first
    /// derivative is null, the tangent comes from the higher ones);
    /// StdFail_UndefinedDerivative when no tangent is defined;
    /// `let (p, d) = curve.tangent(true)?;`.
    pub fn tangent(&self, at_start: bool) -> Result<(Point2, Direction2), Failure> {
        let u = if at_start {
            BCurveTool::first_parameter(self.curve())?
        } else {
            BCurveTool::last_parameter(self.curve())?
        };
        let mut p = Point2::default();
        self.d0(u, &mut p)?;
        let mut clp = CLProps::new(self, 2, epsilon(1.))?;
        clp.set_parameter(u)?;
        if !clp.is_tangent_defined()? {
            return Err(Failure::with_type(
                "StdFail_UndefinedDerivative",
                &format!(
                    "HLRBRep_Curve::Tangent: no tangent at parameter {u}; expected a defined tangent"
                ),
                "",
            ));
        }
        Ok((p, clp.tangent()?))
    }

    /// D0(U, P): the projected point of parameter `u` (a 3D parameter). The
    /// projector's standard view of type 0 leaves `p` unchanged, as natively;
    /// `curve.d0(0.5, &mut p)?`.
    pub fn d0(&self, u: f64, p: &mut Point2) -> Result<(), Failure> {
        let p3d = BCurveTool::d0(self.curve(), u)?;
        self.projector()?.project(p3d, p);
        Ok(())
    }

    /// Value(U): the projected point (from the origin, see [`Curve::d0`]);
    /// `curve.value(0.5)?`.
    pub fn value(&self, u: f64) -> Result<Point2, Failure> {
        let mut p = Point2::default();
        self.d0(u, &mut p)?;
        Ok(p)
    }

    /// D1(U, P, V): the projected point and first derivative (the perspective
    /// derivative formula in perspective); `curve.d1(0.5, &mut p, &mut v)?`.
    pub fn d1(&self, u: f64, p: &mut Point2, v: &mut Vector2) -> Result<(), Failure> {
        let (mut p3d, mut v13d) = BCurveTool::d1(self.curve(), u)?;
        let projector = self.projector()?;
        if !projector.perspective() {
            // OCC155
            (*p, *v) = projector.project_d1(p3d, v13d);
            return Ok(());
        }
        p3d.transform(projector.transformation());
        v13d.transform(projector.transformation());
        let f = projector.focus()?;
        let r = 1. - p3d.z() / f;
        let e = v13d.z() / (f * r * r);
        p.set_coordinates(p3d.x() / r, p3d.y() / r);
        v.set_coordinates(v13d.x() / r + p3d.x() * e, v13d.y() / r + p3d.y() * e);
        Ok(())
    }

    /// D2(U, P, V1, V2): the projected point and two derivatives;
    /// `curve.d2(0.5, &mut p, &mut v1, &mut v2)?`.
    pub fn d2(
        &self,
        u: f64,
        p: &mut Point2,
        v1: &mut Vector2,
        v2: &mut Vector2,
    ) -> Result<(), Failure> {
        let (mut p3d, mut v13d, mut v23d) = BCurveTool::d2(self.curve(), u)?;
        let projector = self.projector()?;
        p3d.transform(projector.transformation());
        v13d.transform(projector.transformation());
        v23d.transform(projector.transformation());
        if !projector.perspective() {
            p.set_coordinates(p3d.x(), p3d.y());
            v1.set_coordinates(v13d.x(), v13d.y());
            v2.set_coordinates(v23d.x(), v23d.y());
            return Ok(());
        }
        let f = projector.focus()?;
        let r = 1. - p3d.z() / f;
        let q = f * r * r;
        let e = v13d.z() / q;
        let c = e * v13d.z() / (f * r);
        p.set_coordinates(p3d.x() / r, p3d.y() / r);
        v1.set_coordinates(v13d.x() / r + p3d.x() * e, v13d.y() / r + p3d.y() * e);
        v2.set_coordinates(
            v23d.x() / r + 2. * v13d.x() * e + p3d.x() * v23d.z() / q + 2. * p3d.x() * c,
            v23d.y() / r + 2. * v13d.y() * e + p3d.y() * v23d.z() / q + 2. * p3d.y() * c,
        );
        Ok(())
    }

    /// D3(U, P, V1, V2, V3): not implemented natively, the outputs stay unchanged;
    /// `curve.d3(0.5, &mut p, &mut v1, &mut v2, &mut v3)`.
    pub fn d3(
        &self,
        _u: f64,
        _p: &mut Point2,
        _v1: &mut Vector2,
        _v2: &mut Vector2,
        _v3: &mut Vector2,
    ) {
    }

    /// DN(U, N): not implemented natively, a null vector; `curve.dn(0.5, 2)`.
    pub fn dn(&self, _u: f64, _n: i32) -> Vector2 {
        Vector2::default()
    }

    /// Line(): the projected line through the point and derivative at parameter 0
    /// (Standard_ConstructionError for a null derivative); `curve.line()?`.
    pub fn line(&self) -> Result<Line2, Failure> {
        let (mut p, mut v) = (Point2::default(), Vector2::default());
        self.d1(0., &mut p, &mut v)?;
        Ok(Line2::new(p, Direction2::from_vector(v)?))
    }

    /// Circle(): the circle transformed to the viewing frame and projected on XOY;
    /// `curve.circle()?`.
    pub fn circle(&self) -> Result<Circle2, Failure> {
        let mut c = BCurveTool::circle(self.curve())?;
        c.transform(self.projector()?.transformation())?;
        project_circle_on_plane(&view_plane(), &c)
    }

    /// Ellipse(): the projected ellipse; a 3D ellipse is transformed and projected,
    /// a circle seen obliquely gives the ellipse of axis ratio `|D2 xy|`;
    /// `curve.ellipse()?`.
    pub fn ellipse(&self) -> Result<Ellipse2, Failure> {
        let transformation = self.projector()?.transformation();
        if BCurveTool::get_type(self.curve())? == CurveType::Ellipse {
            let mut e = BCurveTool::ellipse(self.curve())?;
            e.transform(transformation)?;
            return project_ellipse_on_plane(&view_plane(), &e);
        }
        // this is a circle
        let mut c = BCurveTool::circle(self.curve())?;
        c.transform(transformation)?;
        let d1: Direction3 = c.axis().direction();
        let d3 = d1.crossed(gp::dz())?;
        let d2 = d1.crossed(d3)?;
        let rap = (d2.x() * d2.x() + d2.y() * d2.y()).sqrt();
        let d = Direction2::new(d1.y(), -d1.x())?;
        let p = Point2::new(c.location().x(), c.location().y());
        let mut el = Ellipse2::from_x_axis(Axis2d::new(p, d), c.radius(), c.radius() * rap, true)?;
        if d1.z() < 0. {
            el.reverse();
        }
        Ok(el)
    }

    /// Hyperbola(): not implemented natively, a default hyperbola;
    /// `curve.hyperbola()`.
    pub fn hyperbola(&self) -> Hyperbola2 {
        Hyperbola2::default()
    }

    /// Parabola(): not implemented natively, a default parabola; `curve.parabola()`.
    pub fn parabola(&self) -> Parabola2 {
        Parabola2::default()
    }

    /// Poles(TP): the poles of the 3D B-spline (or else the Bezier) in the indices
    /// of `tp`, transformed to the viewing frame; `curve.poles(&mut tp)?`.
    pub fn poles(&self, tp: &mut Array1<'_, Point2>) -> Result<(), Failure> {
        let source = if BCurveTool::get_type(self.curve())? == CurveType::BSplineCurve {
            BCurveTool::bspline(self.curve())?.poles()
        } else {
            BCurveTool::bezier(self.curve())?.poles()
        };
        let tp3 = read_range(&source, tp.lower(), tp.upper())?;
        self.write_projected(&tp3, tp)
    }

    /// Poles(aCurve, TP): the poles of `a_curve`, transformed to the viewing frame;
    /// `curve.poles_of(&bspline, &mut tp)?`.
    pub fn poles_of(
        &self,
        a_curve: &Handle<GeomBSplineCurve>,
        tp: &mut Array1<'_, Point2>,
    ) -> Result<(), Failure> {
        let tp3 = read_range(&a_curve.poles(), tp.lower(), tp.upper())?;
        self.write_projected(&tp3, tp)
    }

    /// PolesAndWeights(TP, TW): the poles (see [`Curve::poles`]) and the weights in
    /// the indices of `tp`; `curve.poles_and_weights(&mut tp, &mut tw)?`.
    pub fn poles_and_weights(
        &self,
        tp: &mut Array1<'_, Point2>,
        tw: &mut Array1<'_, f64>,
    ) -> Result<(), Failure> {
        let (lower, upper) = (tp.lower(), tp.upper());
        let tp3 = if BCurveTool::get_type(self.curve())? == CurveType::BSplineCurve {
            let spline = BCurveTool::bspline(self.curve())?;
            let poles = read_range(&spline.poles(), lower, upper)?;
            spline.weights_array(|weights| write_range(weights, tw, lower, upper))?;
            poles
        } else {
            let spline = BCurveTool::bezier(self.curve())?;
            let poles = read_range(&spline.poles(), lower, upper)?;
            spline.weights_array(|weights| write_range(weights, tw, lower, upper))?;
            poles
        };
        self.write_projected(&tp3, tp)
    }

    /// PolesAndWeights(aCurve, TP, TW): as [`Curve::poles_and_weights`] on
    /// `a_curve`; `curve.poles_and_weights_of(&bspline, &mut tp, &mut tw)?`.
    pub fn poles_and_weights_of(
        &self,
        a_curve: &Handle<GeomBSplineCurve>,
        tp: &mut Array1<'_, Point2>,
        tw: &mut Array1<'_, f64>,
    ) -> Result<(), Failure> {
        let (lower, upper) = (tp.lower(), tp.upper());
        let tp3 = read_range(&a_curve.poles(), lower, upper)?;
        a_curve.weights_array(|weights| write_range(weights, tw, lower, upper))?;
        self.write_projected(&tp3, tp)
    }

    /// Knots(kn): the knots of a 3D B-spline in the indices of `kn` (other curves
    /// leave it unchanged); `curve.knots(&mut kn)?`.
    pub fn knots(&self, kn: &mut Array1<'_, f64>) -> Result<(), Failure> {
        if BCurveTool::get_type(self.curve())? != CurveType::BSplineCurve {
            return Ok(());
        }
        let knots = BCurveTool::bspline(self.curve())?.knots();
        write_range(&knots, kn, kn.lower(), kn.upper())
    }

    /// Multiplicities(mu): the multiplicities of a 3D B-spline in the indices of
    /// `mu` (other curves leave it unchanged); `curve.multiplicities(&mut mu)?`.
    pub fn multiplicities(&self, mu: &mut Array1<'_, i32>) -> Result<(), Failure> {
        if BCurveTool::get_type(self.curve())? != CurveType::BSplineCurve {
            return Ok(());
        }
        let mults = BCurveTool::bspline(self.curve())?.multiplicities();
        write_range(&mults, mu, mu.lower(), mu.upper())
    }

    // `myProj->Transform(TP3(i)); TP(i).SetCoord(X, Y)` over the range.
    fn write_projected(&self, tp3: &[Point3], tp: &mut Array1<'_, Point2>) -> Result<(), Failure> {
        let projector = self.projector()?;
        for (index, pole) in (tp.lower()..).zip(tp3) {
            let mut p = *pole;
            projector.transform_point(&mut p);
            put(tp, index, Point2::new(p.x(), p.y()))?;
        }
        Ok(())
    }
}

// `gp_Pln(gp::XOY())`, the view plane of the projected conics.
fn view_plane() -> Plane3 {
    Plane3::from_position(Axis3::from_axis2(gp::xoy()))
}

// The native local copy `TP3(i) = aSrcPoles(i)` over `lower..=upper`: an index
// outside the source raises before the output is touched.
fn read_range<T: Copy>(source: &Array1<'_, T>, lower: i32, upper: i32) -> Result<Vec<T>, Failure> {
    (lower..=upper).map(|index| at(source, index)).collect()
}

fn write_range<T: Copy>(
    source: &Array1<'_, T>,
    target: &mut Array1<'_, T>,
    lower: i32,
    upper: i32,
) -> Result<(), Failure> {
    for index in lower..=upper {
        put(target, index, at(source, index)?)?;
    }
    Ok(())
}
