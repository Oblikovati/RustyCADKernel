// Rust translation of the GeomFill_SectionLaw members of
// ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_NSections.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The queries of `GeomFill_NSections`, read from its surface, and the tests of a
//! constant or conical law on two sections.
use super::nsections::GeomFillNSections;
use super::section_curve::section_tolerance;
use super::section_law::{GeomFillSectionLaw, SectionD1, SectionD2};
use super::sweep_failures::{live, sweep_failure};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::put;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{
    GeomBSplineSurface, GeomCircle, GeomCurve, GeomSurface, GeomTrimmedCurve,
};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkgeombase::gcpnts::AbscissaPoint;
use rustycad_tkmath::geom_abs::{CurveType, Shape};
use rustycad_tkmath::gp::{Axis1, Circle3, Direction3, Line3, Point3, Vector3, Xyz};

fn out_of_range(member: &str, index: usize, len: usize) -> Failure {
    sweep_failure(
        "Standard_OutOfRange",
        &format!("GeomFill_NSections::{member}: index {index}; expected 1..={len}"),
    )
}

impl GeomFillNSections {
    fn section(&self, member: &str, index: usize) -> Result<&Handle<dyn GeomCurve>, Failure> {
        self.my_sections
            .get(index - 1)
            .ok_or_else(|| out_of_range(member, index, self.my_sections.len()))
    }

    fn param(&self, member: &str, index: usize) -> Result<f64, Failure> {
        self.my_params
            .get(index.wrapping_sub(1))
            .copied()
            .ok_or_else(|| out_of_range(member, index, self.my_params.len()))
    }

    /// The adaptors of the two sections of a two-section law, or `None`.
    fn two_sections(
        &self,
        member: &str,
    ) -> Result<Option<(GeomAdaptorCurve, GeomAdaptorCurve)>, Failure> {
        if self.my_sections.len() != 2 {
            return Ok(None);
        }
        let ac1 = GeomAdaptorCurve::with_curve(self.section(member, 1)?)?;
        let ac2 = GeomAdaptorCurve::with_curve(self.section(member, 2)?)?;
        Ok(Some((ac1, ac2)))
    }

    fn with_surface<T>(&self, absent: T, read: impl FnOnce(&GeomBSplineSurface) -> T) -> T {
        let surface = self.surface();
        surface.get().map_or(absent, read)
    }

    /// Two circles of the same axis (up to 1e-4), radius and location, or with the
    /// second centre on the axis of the first.
    fn same_circles(c1: &Circle3, c2: &Circle3) -> Result<bool, Failure> {
        let tol = 1.0e-7;
        let samedir = c1.axis().is_parallel(c2.axis(), 1.0e-4);
        let samerad = (c1.radius() - c2.radius()).abs() < tol;
        let mut samepos = c1.location().distance(&c2.location()) < tol;
        if !samepos {
            let v = Vector3::between(c1.location(), c2.location());
            let d = Axis1::new(c1.location(), Direction3::from_vector(v)?);
            samepos = c1.axis().is_parallel(d, 1.0e-4);
        }
        Ok(samedir && samerad && samepos)
    }

    /// Two segments of the same direction and length at the same place (possibly
    /// reversed).
    fn same_segments(ac1: &GeomAdaptorCurve, ac2: &GeomAdaptorCurve) -> Result<bool, Failure> {
        let (l1, l2) = (ac1.line()?, ac2.line()?);
        let tol = 1.0e-7;
        let samedir = l1.direction().is_parallel(l2.direction(), 1.0e-4);
        let p11 = ac1.value(ac1.first_parameter()?)?;
        let p12 = ac1.value(ac1.last_parameter()?)?;
        let p21 = ac2.value(ac2.first_parameter()?)?;
        let p22 = ac2.value(ac2.last_parameter()?)?;
        let samelength = (p11.distance(&p12) - p21.distance(&p22)).abs() < tol;
        let samepos = (p11.distance(&p21) < tol && p12.distance(&p22) < tol)
            || (p12.distance(&p21) < tol && p11.distance(&p22) < tol);
        Ok(samedir && samelength && samepos)
    }

    /// The circle of section `index` moved back by the inverse of its
    /// transformation, when the law has transformations.
    fn untransformed_circle(
        &self,
        ac: &GeomAdaptorCurve,
        index: usize,
    ) -> Result<Circle3, Failure> {
        let mut c = ac.circle()?;
        if !self.my_trsfs.is_empty() {
            let trsf = self
                .my_trsfs
                .get(index - 1)
                .ok_or_else(|| out_of_range("IsConicalLaw", index, self.my_trsfs.len()))?;
            c.transform(trsf.inverted()?)?;
        }
        Ok(c)
    }
}

impl GeomFillSectionLaw for GeomFillNSections {
    fn d0(
        &self,
        v: f64,
        poles: &mut Array1<'_, Point3>,
        weights: &mut Array1<'_, f64>,
    ) -> Result<bool, Failure> {
        self.nsections_d0(v, poles, weights)
    }

    fn d1(&self, v: f64, values: SectionD1<'_, '_>) -> Result<bool, Failure> {
        self.nsections_d1(v, values)
    }

    fn d2(&self, v: f64, values: SectionD2<'_, '_>) -> Result<bool, Failure> {
        self.nsections_d2(v, values)
    }

    /// `BSplineSurface()`: the surface itself (shared).
    fn bspline_surface(&self) -> Result<Handle<GeomBSplineSurface>, Failure> {
        Ok(self.surface())
    }

    /// `SectionShape`: the U shape of the surface. Without a surface the native
    /// outputs are left unchanged; here they are 0.
    fn section_shape(&self) -> Result<(i32, i32, i32), Failure> {
        Ok(self.with_surface((0, 0, 0), |s| {
            (s.nb_u_poles(), s.nb_u_knots(), s.u_degree())
        }))
    }

    fn knots(&self, knots: &mut Array1<'_, f64>) -> Result<(), Failure> {
        self.with_surface((), |s| {
            knots.assign(&s.u_knots());
        });
        Ok(())
    }

    fn mults(&self, mults: &mut Array1<'_, i32>) -> Result<(), Failure> {
        self.with_surface((), |s| {
            mults.assign(&s.u_multiplicities());
        });
        Ok(())
    }

    fn is_rational(&self) -> Result<bool, Failure> {
        Ok(self.with_surface(false, |s| s.is_u_rational()))
    }

    fn is_u_periodic(&self) -> Result<bool, Failure> {
        Ok(self.with_surface(false, |s| s.is_u_periodic()))
    }

    fn is_v_periodic(&self) -> Result<bool, Failure> {
        Ok(self.with_surface(false, |s| s.is_v_periodic()))
    }

    /// `NbIntervals(S)`: the V intervals of the surface (0 without one).
    fn nb_intervals(&self, shape: Shape) -> Result<i32, Failure> {
        let surface = self.surface();
        if surface.is_null() {
            return Ok(0);
        }
        GeomAdaptorSurface::with_surface(&surface.cast::<dyn GeomSurface>())?.nb_v_intervals(shape)
    }

    fn intervals(&self, bounds: &mut Array1<'_, f64>, shape: Shape) -> Result<(), Failure> {
        let surface = self.surface();
        if surface.is_null() {
            return Ok(());
        }
        GeomAdaptorSurface::with_surface(&surface.cast::<dyn GeomSurface>())?
            .v_intervals(bounds, shape)
    }

    /// `SetInterval`: nothing to do.
    fn set_interval(&self, _first: f64, _last: f64) -> Result<(), Failure> {
        Ok(())
    }

    fn get_interval(&self) -> Result<(f64, f64), Failure> {
        Ok((self.v_first, self.v_last))
    }

    fn get_domain(&self) -> Result<(f64, f64), Failure> {
        Ok((self.v_first, self.v_last))
    }

    fn get_tolerance(
        &self,
        bound_tol: f64,
        surf_tol: f64,
        _angle_tol: f64,
        tol3d: &mut Array1<'_, f64>,
    ) -> Result<(), Failure> {
        section_tolerance(bound_tol, surf_tol, tol3d)
    }

    /// `BarycentreOfSurf()`: the mean of 21 x 21 surface points. The native U does
    /// not restart at each row, so the rows walk on past the U bound (kept).
    fn barycentre_of_surf(&self) -> Result<Point3, Failure> {
        let surface = self.surface();
        let Some(s) = surface.get() else {
            return Ok(Point3::new(0.0, 0.0, 0.0));
        };
        let (u0, u1, v0, v1) = s.bounds();
        let (delta_v, delta_u) = ((v1 - v0) / 20.0, (u1 - u0) / 20.0);
        let (mut u, mut v, mut bary) = (u0, v0, Xyz::new(0.0, 0.0, 0.0));
        for _ in 0..=20 {
            for _ in 0..=20 {
                bary += s.value(u, v)?.xyz();
                u += delta_u;
            }
            v += delta_v;
        }
        Ok(Point3::from(bary / f64::from(21 * 21)))
    }

    /// `MaximalSection()`: the longest section.
    fn maximal_section(&self) -> Result<f64, Failure> {
        let mut lmax = 0.0;
        for section in &self.my_sections {
            let l = AbscissaPoint::length(&GeomAdaptorCurve::with_curve(section)?)?;
            if l > lmax {
                lmax = l;
            }
        }
        Ok(lmax)
    }

    /// `GetMinimalWeight(Weights)`: the smallest weight of each U row, or 1.
    fn get_minimal_weight(&self, weights: &mut Array1<'_, f64>) -> Result<(), Failure> {
        let surface = self.surface();
        let Some(s) = surface.get() else {
            return Ok(());
        };
        let w_surf = match (s.is_u_rational(), s.weights()) {
            (true, Some(w_surf)) => w_surf,
            _ => {
                weights.init(1.0);
                return Ok(());
            }
        };
        for i in 1..=s.nb_u_poles() {
            let mut min = w_surf[(i, 1)];
            for j in 2..=s.nb_v_poles() {
                if min > w_surf[(i, j)] {
                    min = w_surf[(i, j)];
                }
            }
            put(weights, i, min)?;
        }
        Ok(())
    }

    /// `IsConstant(Error)`: two equal circles or segments.
    fn is_constant(&self, error: &mut f64) -> Result<bool, Failure> {
        let mut isconst = false;
        if let Some((ac1, ac2)) = self.two_sections("IsConstant")? {
            let c_type = ac1.get_type()?;
            if ac2.get_type()? == c_type {
                isconst = match c_type {
                    CurveType::Circle => Self::same_circles(&ac1.circle()?, &ac2.circle()?)?,
                    CurveType::Line => Self::same_segments(&ac1, &ac2)?,
                    _ => false,
                };
            }
        }
        *error = 0.0;
        Ok(isconst)
    }

    /// `ConstantSection()`: a copy of the first section.
    fn constant_section(&self) -> Result<Handle<dyn GeomCurve>, Failure> {
        let member = "ConstantSection";
        Ok(live(
            self.section(member, 1)?,
            "GeomFill_NSections::ConstantSection",
            "mySections(1)",
        )?
        .copy()?
        .cast())
    }

    /// `IsConicalLaw(Error)`: two coaxial circles (before their transformations)
    /// on the same parameter range.
    fn is_conical_law(&self, error: &mut f64) -> Result<bool, Failure> {
        let mut isconic = false;
        if let Some((ac1, ac2)) = self.two_sections("IsConicalLaw")?
            && ac1.get_type()? == CurveType::Circle
            && ac2.get_type()? == CurveType::Circle
        {
            let c1 = self.untransformed_circle(&ac1, 1)?;
            let c2 = self.untransformed_circle(&ac2, 2)?;
            isconic = c1.axis().is_parallel(c2.axis(), 1.0e-4)
                && Line3::from_axis(c1.axis()).distance(c2.location()) < 1.0e-7
                && same_range(&ac1, &ac2)?;
        }
        *error = 0.0;
        Ok(isconic)
    }

    /// `CirclSection(V)`: the circle of the first section with the radius
    /// interpolated between the end sections; StdFail_NotDone for another law.
    fn circl_section(&self, v: f64) -> Result<Handle<dyn GeomCurve>, Failure> {
        let mut err = 0.0;
        if !self.is_conical_law(&mut err)? {
            return Err(sweep_failure("StdFail_NotDone", "The Law is not Conical!"));
        }
        let ac1 = GeomAdaptorCurve::with_curve(self.section("CirclSection", 1)?)?;
        let last = self.my_sections.len();
        let ac2 = GeomAdaptorCurve::with_curve(self.section("CirclSection", last)?)?;
        let (mut c1, c2) = (ac1.circle()?, ac2.circle()?);
        let p1 = self.param("CirclSection", 1)?;
        let p2 = self.param("CirclSection", self.my_params.len())?;
        let radius = (c2.radius() - c1.radius()) * (v - p1) / (p2 - p1) + c1.radius();
        c1.set_radius(radius)?;
        let c: Handle<dyn GeomCurve> = Handle::new(GeomCircle::from_circ(c1)).cast();
        let (a_par_f, a_par_l) = (ac1.first_parameter()?, ac1.last_parameter()?);
        let a_period = if ac1.is_periodic()? {
            ac1.period()?
        } else {
            0.0
        };
        if a_period == 0.0 || (a_par_l - a_par_f - a_period).abs() > precision::p_confusion() {
            return Ok(
                Handle::new(GeomTrimmedCurve::new(&c, a_par_f, a_par_l, true, true)?).cast(),
            );
        }
        Ok(c)
    }
}

/// Whether two adaptors have the same parameter range within PConfusion().
fn same_range(ac1: &GeomAdaptorCurve, ac2: &GeomAdaptorCurve) -> Result<bool, Failure> {
    let tol = precision::p_confusion();
    Ok(
        (ac1.first_parameter()? - ac2.first_parameter()?).abs() <= tol
            && (ac1.last_parameter()? - ac2.last_parameter()?).abs() <= tol,
    )
}
