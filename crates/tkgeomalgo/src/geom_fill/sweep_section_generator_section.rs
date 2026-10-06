// Rust translation of the two Section members of
// ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_SweepSectionGenerator.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The sections of `GeomFill_SweepSectionGenerator`: a transformed (and, between
//! two sections, blended) first section, or the arc between the two rails; and the
//! derivatives of the first and last sections, which turn around the centre of
//! curvature of the path.
use super::sweep_failures::{live, sweep_failure};
use super::sweep_section_generator::GeomFillSweepSectionGenerator;
use crate::app_blend::{AppBlendSection, TheSectionGenerator};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::real::epsilon;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::put;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCircle, GeomCurve, GeomTrimmedCurve};
use rustycad_tkgeombase::geom_convert::curve_to_bspline_curve;
use rustycad_tkmath::convert::ParameterisationType;
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Axis2, Direction3, Point3, Vector3};

fn index_failure(array: &str, index: i32, e: impl std::fmt::Display) -> Failure {
    sweep_failure(
        "Standard_OutOfRange",
        &format!(
            "GeomFill_SweepSectionGenerator::Section: {array}({index}): {e}; expected an index in the array bounds"
        ),
    )
}

/// The parameter on a rail at the path fraction `alpha` (the projection of the
/// path point on an unbounded straight rail).
fn rail_parameter(rail: &dyn Adaptor3dCurve, alpha: f64, ppath: Point3) -> Result<f64, Failure> {
    let (f, l) = (rail.first_parameter()?, rail.last_parameter()?);
    let u = (1.0 - alpha) * f + alpha * l;
    let unbounded = precision::is_infinite(f) || precision::is_infinite(l);
    if rail.get_type()? == CurveType::Line && unbounded {
        return elclib::parameter(&rail.line()?, ppath);
    }
    Ok(u)
}

impl GeomFillSweepSectionGenerator {
    /// `Section(P, Poles, Poles2d, Weigths)`: the poles and weights of section
    /// `p` assigned to the section arrays; `generator.section(2, &mut section)?`.
    pub fn section(&self, p: i32, section: &mut AppBlendSection) -> Result<(), Failure> {
        if self.my_type != 0 {
            return self.moved_section(p, section);
        }
        self.rail_section(p, section)
    }

    fn moved_section(&self, p: i32, section: &mut AppBlendSection) -> Result<(), Failure> {
        let first = self.first_sect("GeomFill_SweepSectionGenerator::Section")?;
        section.poles.assign(&first.poles());
        first.weights_array(|w| {
            section.weights.assign(w);
        });
        if p <= 1 {
            return Ok(());
        }
        let cumul_tr = self
            .my_trsfs
            .get((p - 2) as usize)
            .copied()
            .ok_or_else(|| index_failure("myTrsfs", p - 1, "outside the sequence"))?;
        if self.my_type == 3 || self.my_type == 6 {
            self.blend_sections(p, first, section)?;
        }
        for i in 1..=section.poles.length() {
            let pole = section
                .poles
                .change_value(i)
                .map_err(|e| index_failure("Poles", i, e))?;
            pole.transform(cumul_tr);
        }
        Ok(())
    }

    /// Section `p` between the first and last sections, weighted by its index.
    fn blend_sections(
        &self,
        p: i32,
        first: &GeomBSplineCurve,
        section: &mut AppBlendSection,
    ) -> Result<(), Failure> {
        let last = live(
            &self.my_last_sect,
            "GeomFill_SweepSectionGenerator::Section",
            "myLastSect",
        )?;
        let n = self.my_nb_sections;
        let (a, b, d) = (f64::from(n - p), f64::from(p - 1), f64::from(n - 1));
        for i in 1..=first.nb_poles() {
            let xyz = a * first.pole(i)?.xyz() + b * last.pole(i)?.xyz();
            put(&mut section.poles, i, Point3::from(xyz / d))?;
            let weight = a * first.weight(i)? + b * last.weight(i)?;
            put(&mut section.weights, i, weight / d)?;
        }
        Ok(())
    }

    /// The path point of section `p` and the points of the rails at the same
    /// fraction of their ranges.
    fn rail_points(&self, p: i32) -> Result<[Point3; 3], Failure> {
        let member = "GeomFill_SweepSectionGenerator::Section";
        let adp_path = live(&self.my_adp_path, member, "myAdpPath")?;
        let coef = (f64::from(p) - 1.0) / (f64::from(self.my_nb_sections) - 1.0);
        let (pf, pl) = (adp_path.first_parameter()?, adp_path.last_parameter()?);
        let u = (1.0 - coef) * pf + coef * pl;
        let ppath = adp_path.value(u)?;
        let mut alpha = u - pf;
        alpha /= pl - pf;
        let first_rail = live(&self.my_adp_first_sect, member, "myAdpFirstSect")?;
        let p1 = first_rail.value(rail_parameter(first_rail, alpha, ppath)?)?;
        let last_rail = live(&self.my_adp_last_sect, member, "myAdpLastSect")?;
        let p2 = last_rail.value(rail_parameter(last_rail, alpha, ppath)?)?;
        Ok([ppath, p1, p2])
    }

    /// The arc of `myRadius` from the first rail to the second around the path
    /// point, or the first rail point repeated when the arc is empty.
    fn rail_section(&self, p: i32, section: &mut AppBlendSection) -> Result<(), Failure> {
        let [ppath, p1, p2] = self.rail_points(p)?;
        let (axis, angle) = if p1.distance(&p2) < precision::confusion() {
            (Axis2::default(), 0.0)
        } else {
            let (v1, v2) = (Vector3::between(ppath, p1), Vector3::between(ppath, p2));
            let normal = Direction3::from_vector(v1.crossed(v2))?;
            let axis = Axis2::with_x_direction(ppath, normal, Direction3::from_vector(v1)?)?;
            (axis, elclib::circle_parameter(axis, p2))
        };
        if angle < precision::angular() {
            for i in 1..=section.poles.upper() {
                put(&mut section.poles, i, p1)?;
                put(&mut section.weights, i, 1.0)?;
            }
            return Ok(());
        }
        let bs = self.arc_bspline(axis, angle)?;
        let b = live(&bs, "GeomFill_SweepSectionGenerator::Section", "BS")?;
        section.poles.assign(&b.poles());
        b.weights_array(|w| {
            section.weights.assign(w);
        });
        Ok(())
    }

    fn arc_bspline(&self, axis: Axis2, angle: f64) -> Result<Handle<GeomBSplineCurve>, Failure> {
        let circ: Handle<dyn GeomCurve> =
            Handle::new(GeomCircle::new(axis, self.my_radius)?).cast();
        let ct: Handle<dyn GeomCurve> =
            Handle::new(GeomTrimmedCurve::new(&circ, 0.0, angle, true, true)?).cast();
        let conv = if self.my_polynomial {
            ParameterisationType::Polynomial
        } else {
            ParameterisationType::QuasiAngular
        };
        curve_to_bspline_curve(&ct, conv)
    }

    /// `Section(P, Poles, DPoles, Poles2d, DPoles2d, Weigths, DWeigths)`: the
    /// section, and for the first and last ones the derivatives (a rotation about
    /// the centre of curvature of the path, or a translation on a straight end);
    /// false for another section or the circles between rails;
    /// `generator.section_d1(1, &mut section)?`.
    pub fn section_d1(&self, p: i32, section: &mut AppBlendSection) -> Result<bool, Failure> {
        self.section(p, section)?;
        if self.my_type == 0 {
            return Ok(false);
        }
        let path = self.path("GeomFill_SweepSectionGenerator::Section")?;
        let u = if p == 1 {
            path.first_parameter()
        } else if p == self.my_nb_sections {
            path.last_parameter()
        } else {
            return Ok(false);
        };
        let (pt, d1, d2) = path.d2(u)?;
        let l = d1.magnitude();
        if l < epsilon(1.0) {
            return Ok(false);
        }
        let nb_poles = self
            .first_sect("GeomFill_SweepSectionGenerator::Section")?
            .nb_poles();
        end_derivatives(section, nb_poles, (pt, d1, d2), l)?;
        Ok(true)
    }
}

/// The pole derivatives of an end section: a rotation about the centre of
/// curvature of the path, or the path derivative on a straight end; the weight
/// derivatives are null.
fn end_derivatives(
    section: &mut AppBlendSection,
    nb_poles: i32,
    (pt, d1, d2): (Point3, Vector3, Vector3),
    l: f64,
) -> Result<(), Failure> {
    let t = Vector3::from(Direction3::from_vector(d1)?);
    let d = d2 - t * d2.dot(t);
    let c = d.magnitude() / (l * l);
    if c < epsilon(1.0) {
        for i in 1..=nb_poles {
            put(&mut section.d_poles, i, d1)?;
        }
    } else {
        let n = Vector3::from(Direction3::from_vector(d)?);
        let q = pt.translated(n * (1.0 / c));
        for i in 1..=nb_poles {
            let dp = turning_derivative(&section.poles, i, q, t, n)?;
            put(&mut section.d_poles, i, dp)?;
        }
    }
    for i in 1..=nb_poles {
        put(&mut section.d_weights, i, 0.0)?;
    }
    Ok(())
}

/// `x N - y T` scaled to the distance of pole `i` from `q` in the `(T, N)` plane.
fn turning_derivative(
    poles: &Array1<'_, Point3>,
    i: i32,
    q: Point3,
    t: Vector3,
    n: Vector3,
) -> Result<Vector3, Failure> {
    let pole = *poles.value(i).map_err(|e| index_failure("Poles", i, e))?;
    let v = Vector3::between(q, pole);
    let (x, y) = (v.dot(t), v.dot(n));
    let mut dp = n * x - t * y;
    if dp.magnitude() > epsilon(1.0) {
        dp.normalize()?;
        dp *= (x * x + y * y).sqrt();
    }
    Ok(dp)
}

/// `GeomFill_AppSweep` evaluates the sections through this interface.
impl TheSectionGenerator for GeomFillSweepSectionGenerator {
    type Point = i32;
    fn get_shape(&self) -> Result<[i32; 4], Failure> {
        GeomFillSweepSectionGenerator::get_shape(self)
    }
    fn knots(&self, knots: &mut Array1<'_, f64>) -> Result<(), Failure> {
        GeomFillSweepSectionGenerator::knots(self, knots)
    }
    fn mults(&self, mults: &mut Array1<'_, i32>) -> Result<(), Failure> {
        GeomFillSweepSectionGenerator::mults(self, mults)
    }
    fn section_d1(&mut self, p: &i32, section: &mut AppBlendSection) -> Result<bool, Failure> {
        GeomFillSweepSectionGenerator::section_d1(self, *p, section)
    }
    fn section(&mut self, p: &i32, section: &mut AppBlendSection) -> Result<(), Failure> {
        GeomFillSweepSectionGenerator::section(self, *p, section)
    }
    fn parameter(&self, p: &i32) -> Result<f64, Failure> {
        GeomFillSweepSectionGenerator::parameter(self, *p)
    }
}
