// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_Generator.hxx/.cxx/.lxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomFill_Generator`: the ruled B-spline surface through profiled sections.
use super::GeomFillProfiler;
use super::filling::put2;
use super::profiler_unify::curve_at;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::{at, one_based, put};
use rustycad_tkg2d::geom2d::errors;
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomBSplineSurface, GeomSurface};
use rustycad_tkmath::gp::Point3;
use std::ops::{Deref, DerefMut};

/// The surface through the sections of a profile (`GeomFill_Generator`): U follows
/// the common section profile, V is linear between consecutive sections (degree 1,
/// knots `0, 1, ..., n - 1`). The `GeomFill_Profiler` is reached through `Deref`.
///
/// `let mut g = GeomFillGenerator::new(); g.add_curve(&c1)?; g.add_curve(&c2)?; g.perform(1e-9)?; g.surface()`
#[derive(Clone, Default)]
pub struct GeomFillGenerator {
    profiler: GeomFillProfiler,
    my_surface: Handle<GeomBSplineSurface>,
}

impl Deref for GeomFillGenerator {
    type Target = GeomFillProfiler;
    fn deref(&self) -> &GeomFillProfiler {
        &self.profiler
    }
}

impl DerefMut for GeomFillGenerator {
    fn deref_mut(&mut self) -> &mut GeomFillProfiler {
        &mut self.profiler
    }
}

/// The pole and weight grids of the surface, sections in columns.
struct Grids {
    poles: Array2<'static, Point3>,
    weights: Array2<'static, f64>,
    v_knots: Array1<'static, f64>,
}

impl GeomFillGenerator {
    /// An empty generator (`GeomFill_Generator()`); `GeomFillGenerator::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Perform(PTol)`: the profiler's `Perform`, then the surface of degree 1 in V,
    /// periodic in U when every section is periodic; `generator.perform(1e-9)?`.
    pub fn perform(&mut self, ptol: f64) -> Result<(), Failure> {
        self.profiler.perform(ptol)?;
        let nb_u_knots: i32 = self.nb_knots()?;
        let nb_v_knots: i32 = self.my_sequence.len() as i32;
        let is_u_periodic: bool = self.is_periodic();
        let mut v_mults: Array1<'static, i32> = one_based(nb_v_knots)?;
        v_mults.init(1);
        put(&mut v_mults, 1, 2)?;
        put(&mut v_mults, nb_v_knots, 2)?;
        let mut u_knots: Array1<'static, f64> = one_based(nb_u_knots)?;
        let mut u_mults: Array1<'static, i32> = one_based(nb_u_knots)?;
        self.knots_and_mults(&mut u_knots, &mut u_mults)?;
        let grids: Grids = self.section_grids()?;
        let surface = GeomBSplineSurface::with_weights(
            &grids.poles,
            &grids.weights,
            (&u_knots, &u_mults),
            (&grids.v_knots, &v_mults),
            (self.degree()?, 1),
            (is_u_periodic, false),
        )?;
        self.my_surface = Handle::new(surface);
        Ok(())
    }
    /// The poles and weights of each section as a column, and the V knot `j - 1` of
    /// section `j`.
    fn section_grids(&self) -> Result<Grids, Failure> {
        let (nb_u_poles, nb_v_poles) = (self.nb_poles()?, self.my_sequence.len() as i32);
        let table = |e| errors::array_bounds(e);
        let mut grids = Grids {
            poles: Array2::with_bounds(1, nb_u_poles, 1, nb_v_poles).map_err(table)?,
            weights: Array2::with_bounds(1, nb_u_poles, 1, nb_v_poles).map_err(table)?,
            v_knots: one_based(nb_v_poles)?,
        };
        for j in 1..=nb_v_poles {
            let section: &GeomBSplineCurve = curve_at(&self.my_sequence, j)?;
            let poles: Array1<'static, Point3> = section.poles();
            put(&mut grids.v_knots, j, f64::from(j - 1))?;
            section.weights_array(|weights| -> Result<(), Failure> {
                for i in 1..=nb_u_poles {
                    put2(&mut grids.poles, i, j, at(&poles, i)?)?;
                    put2(&mut grids.weights, i, j, at(weights, i)?)?;
                }
                Ok(())
            })?;
        }
        Ok(grids)
    }
    /// `Surface()`: the surface of the last `Perform`, null before;
    /// `generator.surface()`.
    pub fn surface(&self) -> Handle<dyn GeomSurface> {
        self.my_surface.cast::<dyn GeomSurface>()
    }
}
