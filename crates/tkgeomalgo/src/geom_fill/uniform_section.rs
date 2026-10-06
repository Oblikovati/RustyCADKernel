// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_UniformSection.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomFill_UniformSection`: a constant section.
use super::frame_math::point_from_xyz;
use super::section_curve::{SectionCurve, section_tolerance};
use super::section_law::{GeomFillSectionLaw, SectionD1, SectionD2};
use super::sweep_failures::live;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkg2d::geom2d::arrays::one_based;
use rustycad_tkg3d::geom::{GeomBSplineSurface, GeomCurve};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point3, Vector3};

/// `GeomFill_UniformSection`: the same B-spline section at every parameter of
/// `[First, Last]`. A B-spline input is shared, not copied, as natively.
/// `let law = GeomFillUniformSection::new(&circle, 0.0, 1.0)?;`.
pub struct GeomFillUniformSection {
    parent: StandardTransient,
    first: f64,
    last: f64,
    curve: SectionCurve,
}
impl_transient!(
    GeomFillUniformSection,
    "GeomFill_UniformSection",
    dyn GeomFillSectionLaw,
    parent,
    interfaces: [dyn GeomFillSectionLaw]
);

impl GeomFillUniformSection {
    /// `GeomFill_UniformSection(C, FirstParameter = 0, LastParameter = 1)`.
    pub fn new(
        c: &Handle<dyn GeomCurve>,
        first_parameter: f64,
        last_parameter: f64,
    ) -> Result<Self, Failure> {
        Ok(Self {
            parent: StandardTransient::default(),
            first: first_parameter,
            last: last_parameter,
            curve: SectionCurve::new(c)?,
        })
    }
}

impl GeomFillSectionLaw for GeomFillUniformSection {
    fn d0(
        &self,
        _param: f64,
        poles: &mut Array1<'_, Point3>,
        weights: &mut Array1<'_, f64>,
    ) -> Result<bool, Failure> {
        self.curve.assign_poles(poles, weights);
        Ok(true)
    }
    fn d1(&self, _param: f64, v: SectionD1<'_, '_>) -> Result<bool, Failure> {
        self.curve.assign_poles(v.poles, v.weights);
        v.d_poles.init(Vector3::new(0.0, 0.0, 0.0));
        v.d_weights.init(0.0);
        Ok(true)
    }
    fn d2(&self, _param: f64, v: SectionD2<'_, '_>) -> Result<bool, Failure> {
        self.curve.assign_poles(v.d1.poles, v.d1.weights);
        v.d1.d_poles.init(Vector3::new(0.0, 0.0, 0.0));
        v.d1.d_weights.init(0.0);
        v.d2_poles.init(Vector3::new(0.0, 0.0, 0.0));
        v.d2_weights.init(0.0);
        Ok(true)
    }
    /// The surface swept by the section along V in `[First, Last]` (degree 1).
    fn bspline_surface(&self) -> Result<Handle<GeomBSplineSurface>, Failure> {
        self.surface()
    }
    fn section_shape(&self) -> Result<(i32, i32, i32), Failure> {
        Ok(self.curve.section_shape())
    }
    fn knots(&self, knots: &mut Array1<'_, f64>) -> Result<(), Failure> {
        self.curve.knots(knots);
        Ok(())
    }
    fn mults(&self, mults: &mut Array1<'_, i32>) -> Result<(), Failure> {
        self.curve.mults(mults);
        Ok(())
    }
    fn is_rational(&self) -> Result<bool, Failure> {
        Ok(self.curve.curve().is_rational())
    }
    fn is_u_periodic(&self) -> Result<bool, Failure> {
        Ok(self.curve.curve().is_periodic())
    }
    /// Always true.
    fn is_v_periodic(&self) -> Result<bool, Failure> {
        Ok(true)
    }
    /// Always 1.
    fn nb_intervals(&self, _shape: Shape) -> Result<i32, Failure> {
        Ok(1)
    }
    fn intervals(&self, bounds: &mut Array1<'_, f64>, _shape: Shape) -> Result<(), Failure> {
        let (lower, upper) = (bounds.lower(), bounds.upper());
        rustycad_tkg2d::geom2d::arrays::put(bounds, lower, self.first)?;
        rustycad_tkg2d::geom2d::arrays::put(bounds, upper, self.last)
    }
    /// Nothing: the law is constant.
    fn set_interval(&self, _first: f64, _last: f64) -> Result<(), Failure> {
        Ok(())
    }
    fn get_interval(&self) -> Result<(f64, f64), Failure> {
        Ok((self.first, self.last))
    }
    fn get_domain(&self) -> Result<(f64, f64), Failure> {
        Ok((self.first, self.last))
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
    /// The mean of 21 points of the section.
    fn barycentre_of_surf(&self) -> Result<Point3, Failure> {
        let mut bary = self.curve.point_sum()?;
        bary /= 21.0;
        Ok(point_from_xyz(bary))
    }
    /// The length of the section.
    fn maximal_section(&self) -> Result<f64, Failure> {
        self.curve.length()
    }
    fn get_minimal_weight(&self, weights: &mut Array1<'_, f64>) -> Result<(), Failure> {
        self.curve.curve().weights_array(|w| {
            weights.assign(w);
        });
        Ok(())
    }
    /// Always true with `Error = 0`.
    fn is_constant(&self, error: &mut f64) -> Result<bool, Failure> {
        *error = 0.0;
        Ok(true)
    }
    /// A copy of the section curve.
    fn constant_section(&self) -> Result<Handle<dyn GeomCurve>, Failure> {
        let section = live(
            &self.curve.my_section,
            "GeomFill_UniformSection::ConstantSection",
            "mySection",
        )?;
        Ok(section.copy()?.cast())
    }
}

impl GeomFillUniformSection {
    /// The body of `BSplineSurface`.
    fn surface(&self) -> Result<Handle<GeomBSplineSurface>, Failure> {
        let c = self.curve.curve();
        let nb_poles = c.nb_poles();
        let mut poles: Array2<'static, Point3> =
            Array2::with_bounds(1, nb_poles, 1, 2).map_err(|e| {
                super::sweep_failures::sweep_failure("Standard_RangeError", &e.to_string())
            })?;
        for ii in 1..=nb_poles {
            let pole = c.pole(ii)?;
            poles[(ii, 1)] = pole;
            poles[(ii, 2)] = pole;
        }
        let mut v_knots: Array1<'static, f64> = one_based(2)?;
        v_knots[1] = self.first;
        v_knots[2] = self.last;
        let mut v_mults: Array1<'static, i32> = one_based(2)?;
        v_mults.init(2);
        let (u_knots, u_mults) = (c.knots(), c.multiplicities());
        let bs = GeomBSplineSurface::new(
            &poles,
            (&u_knots, &u_mults),
            (&v_knots, &v_mults),
            (c.degree(), 1),
            (c.is_periodic(), false),
        )?;
        Ok(Handle::new(bs))
    }
}
