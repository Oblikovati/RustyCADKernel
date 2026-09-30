// Rust translation of ModelingAlgorithms/TKTopAlgo/Bisector/Bisector_Inter.hxx/.cxx (the constructors and Perform) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Bisector_Inter`: the intersection of two bisectors, cut into their intervals
//! of continuity.
mod single;

use super::{Bisec, BisectorCurve};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkgeomalgo::int_res2d::{Domain, Intersection};
use rustycad_tkmath::gp::{Direction2, Point2};
use std::ops::{Deref, DerefMut};

/// `Bisector_Inter`: an `IntRes2d_Intersection` of two [`Bisec`]s on two domains.
/// Each bisector is cut into its intervals of continuity (an extension becomes a
/// segment); pairs of neighbouring curve-curve bisectors are intersected on their
/// common guide curve. The points and segments are read through `Deref`.
/// ```
/// use rustycad_tktopalgo::bisector::BisectorInter;
/// let inter = BisectorInter::new();
/// assert!(!inter.is_done());
/// ```
#[derive(Clone, Debug, Default)]
pub struct BisectorInter {
    base: Intersection,
}

impl Deref for BisectorInter {
    type Target = Intersection;
    fn deref(&self) -> &Intersection {
        &self.base
    }
}

impl DerefMut for BisectorInter {
    fn deref_mut(&mut self) -> &mut Intersection {
        &mut self.base
    }
}

/// The two tolerances and the flag of a `Perform`.
#[derive(Clone, Copy)]
pub(crate) struct InterTolerances {
    pub tol_conf: f64,
    pub tol: f64,
    pub comun_element: bool,
}

/// One part of a bisector: its curve on one interval of continuity (the bisector
/// itself, or the segment of an extension) and the domain of that interval. The
/// slots of the intervals outside the domain stay null, as natively.
#[derive(Clone, Default)]
struct Part {
    curve: Handle<dyn Geom2dCurve>,
    domain: Domain,
}

impl BisectorInter {
    /// `Bisector_Inter()`: not done; `BisectorInter::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Bisector_Inter(C1, D1, C2, D2, TolConf, Tol, ComunElement)`: [`BisectorInter::perform`]
    /// on a new object; `BisectorInter::with_bisectors(&b1, &d1, &b2, &d2, 1e-7, 1e-7, true)?`.
    #[allow(clippy::too_many_arguments)] // The native signature.
    pub fn with_bisectors(
        c1: &Bisec,
        d1: &Domain,
        c2: &Bisec,
        d2: &Domain,
        tol_conf: f64,
        tol: f64,
        comun_element: bool,
    ) -> Result<Self, Failure> {
        let mut inter: Self = Self::new();
        inter.perform(c1, d1, c2, d2, tol_conf, tol, comun_element)?;
        Ok(inter)
    }
    /// `Perform(C1, D1, C2, D2, TolConf, Tol, ComunElement)`: intersects the parts of
    /// the two bisectors that meet the domains, pair by pair, and appends the
    /// results. `comun_element` tells that neighbouring curve-curve bisectors share
    /// an element. Natively the pairs read the first slots of the part arrays, which
    /// hold null curves when an interval before the domain was skipped (a crash); the
    /// port raises Standard_NullObject there;
    /// `inter.perform(&b1, &d1, &b2, &d2, 1e-7, 1e-7, true)?`.
    #[allow(clippy::too_many_arguments)] // The native signature.
    pub fn perform(
        &mut self,
        c1: &Bisec,
        d1: &Domain,
        c2: &Bisec,
        d2: &Domain,
        tol_conf: f64,
        tol: f64,
        comun_element: bool,
    ) -> Result<(), Failure> {
        let bis1: Handle<dyn BisectorCurve> = c1.value().basis_curve().cast();
        let bis2: Handle<dyn BisectorCurve> = c2.value().basis_curve().cast();
        let nb1: i32 = bis1.nb_intervals();
        let (parts1, count1) = cut_parts(&bis1, d1, nb1)?;
        // The native test of an extension at the end of the second bisector
        // compares its interval index with the interval count of the FIRST one.
        let (parts2, count2) = cut_parts(&bis2, d2, nb1)?;
        let tolerances = InterTolerances {
            tol_conf,
            tol,
            comun_element,
        };
        for part1 in &parts1[1..=count1] {
            for part2 in &parts2[1..=count2] {
                self.single_perform(part1, part2, tolerances)?;
            }
        }
        Ok(())
    }
}

// The domain bounds of a bisector: its parameters, or the infinite reals.
fn domain_bounds(d: &Domain) -> Result<(f64, f64), Failure> {
    let min_domain: f64 = if d.has_first_point() {
        d.first_parameter()?
    } else {
        f64::MIN
    };
    let max_domain: f64 = if d.has_last_point() {
        d.last_parameter()?
    } else {
        f64::MAX
    };
    Ok((min_domain, max_domain))
}

// The parts of a bisector on its intervals that meet the domain `d`, in slots
// `1..=NbIntervals` (slot 0 unused), and how many there are. `end_count` is the
// interval count the extension-at-end test compares with.
fn cut_parts(
    bis: &Handle<dyn BisectorCurve>,
    d: &Domain,
    end_count: i32,
) -> Result<(Vec<Part>, usize), Failure> {
    let nb: i32 = bis.nb_intervals();
    let (min_domain, max_domain) = domain_bounds(d)?;
    let mut parts: Vec<Part> = vec![Part::default(); nb.max(0) as usize + 1];
    let mut count: usize = 0;
    for ib in 1..=nb {
        let (mut u_min, mut u_max) = (bis.interval_first(ib)?, bis.interval_last(ib)?);
        if !(u_max > min_domain && u_min < max_domain) {
            continue;
        }
        u_min = std_max(u_min, min_domain);
        u_max = std_min(u_max, max_domain);
        let (p_min, p_max) = (bis.value(u_min)?, bis.value(u_max)?);
        let slot: &mut Part = &mut parts[ib as usize];
        slot.domain.set_values(
            p_min,
            u_min,
            d.first_tolerance()?,
            p_max,
            u_max,
            d.last_tolerance()?,
        );
        let extension: bool =
            (ib == 1 && bis.is_extend_at_start()) || (ib == end_count && bis.is_extend_at_end());
        slot.curve = if extension {
            construct_segment(p_min, p_max, u_min)?
        } else {
            bis.cast()
        };
        count += 1;
    }
    Ok((parts, count))
}

// `ConstructSegment(PMin, PMax, UMin, UMax)`: the line through the two points,
// parameterized so that `u_min` is at `p_min`.
fn construct_segment(
    p_min: Point2,
    p_max: Point2,
    u_min: f64,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let dir = Direction2::new(p_max.x() - p_min.x(), p_max.y() - p_min.y())?;
    let origin = Point2::new(p_min.x() - u_min * dir.x(), p_min.y() - u_min * dir.y());
    Ok(Handle::new(Geom2dLine::from_point(origin, dir)).cast())
}
