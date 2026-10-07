// Rust translation of ChFi3d_InPeriod, ChFi3d_Boite, ChFi3d_BoundFac, ChFi3d_BoundSrf and
// the ChFi3d_EnlargeBox overloads without the data structure (ChFi3d_Builder_0.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The parameter ranges and boxes the fillet builders compute before intersecting.
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve, BRepAdaptorSurface};
use rustycad_tkbrep::topods::{self, Edge, Shape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomCurve, GeomRectangularTrimmedSurface, GeomSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::Point2;

/// `ChFi3d_InPeriod(U, UFirst, ULast, Eps)`: `U` moved by whole periods
/// `ULast - UFirst` into the period (within `Eps`), then at least `UFirst`.
///
/// ```
/// use rustycad_tkfillet::chfi3d::in_period;
/// assert_eq!(in_period(7.0, 0.0, 5.0, 1e-9), 2.0);
/// ```
pub fn in_period(u: f64, u_first: f64, u_last: f64, eps: f64) -> f64 {
    let period = u_last - u_first;
    let mut u = u;
    while eps < (u_first - u) {
        u += period;
    }
    while eps > (u_last - u) {
        u -= period;
    }
    if u < u_first { u_first } else { u }
}

/// `ChFi3d_Boite(p1, p2, mu, Mu, mv, Mv)`: `(mu, Mu, mv, Mv)`, the box of two points.
///
/// ```
/// use rustycad_tkfillet::chfi3d::boite;
/// use rustycad_tkmath::gp::Point2;
/// assert_eq!(boite(Point2::new(3.0, 1.0), Point2::new(1.0, 2.0)), (1.0, 3.0, 1.0, 2.0));
/// ```
pub fn boite(p1: Point2, p2: Point2) -> (f64, f64, f64, f64) {
    (
        std_min(p1.x(), p2.x()),
        std_max(p1.x(), p2.x()),
        std_min(p1.y(), p2.y()),
        std_max(p1.y(), p2.y()),
    )
}

/// `ChFi3d_Boite(p1, p2, p3, p4, Du, Dv, mu, Mu, mv, Mv)`: `(Du, Dv, mu, Mu, mv, Mv)`, the
/// box of four points (taken by pairs) and its sizes.
///
/// ```
/// use rustycad_tkfillet::chfi3d::boite4;
/// use rustycad_tkmath::gp::Point2;
/// let p = |x, y| Point2::new(x, y);
/// assert_eq!(boite4([p(0.0, 0.0), p(2.0, 1.0), p(1.0, 3.0), p(-1.0, 0.5)]), (3.0, 3.0, -1.0, 2.0, 0.0, 3.0));
/// ```
pub fn boite4(p: [Point2; 4]) -> (f64, f64, f64, f64, f64, f64) {
    let mu = std_min(std_min(p[0].x(), p[1].x()), std_min(p[2].x(), p[3].x()));
    let big_u = std_max(std_max(p[0].x(), p[1].x()), std_max(p[2].x(), p[3].x()));
    let mv = std_min(std_min(p[0].y(), p[1].y()), std_min(p[2].y(), p[3].y()));
    let big_v = std_max(std_max(p[0].y(), p[1].y()), std_max(p[2].y(), p[3].y()));
    (big_u - mu, big_v - mv, mu, big_u, mv, big_v)
}

/// `std::min(a, b)`: `b < a ? b : a` (the first one on ties and NaN).
fn std_min(a: f64, b: f64) -> f64 {
    if b < a { b } else { a }
}

/// `std::max(a, b)`: `a < b ? b : a` (the first one on ties and NaN).
fn std_max(a: f64, b: f64) -> f64 {
    if a < b { b } else { a }
}

/// The surface adaptors `ChFi3d_BoundSrfImpl` resizes: the original surface and the
/// reload on new bounds keeping the tolerances (and the transformation).
trait BoundedSurface: Adaptor3dSurface {
    /// `ChFi3d_BaseSurface`.
    fn base_surface(&self) -> Handle<dyn GeomSurface>;
    /// `ChFi3d_ApplyBounds`.
    fn apply_bounds(
        &self,
        surface: &Handle<dyn GeomSurface>,
        bounds: (f64, f64, f64, f64),
    ) -> Result<(), Failure>;
}

impl BoundedSurface for GeomAdaptorSurface {
    fn base_surface(&self) -> Handle<dyn GeomSurface> {
        self.surface()
    }

    fn apply_bounds(
        &self,
        surface: &Handle<dyn GeomSurface>,
        bounds: (f64, f64, f64, f64),
    ) -> Result<(), Failure> {
        self.load_range(surface, (bounds, (self.tolerance_u(), self.tolerance_v())))
    }
}

impl BoundedSurface for BRepAdaptorSurface {
    fn base_surface(&self) -> Handle<dyn GeomSurface> {
        self.transformed_surface().geom_surface_original()
    }

    fn apply_bounds(
        &self,
        surface: &Handle<dyn GeomSurface>,
        bounds: (f64, f64, f64, f64),
    ) -> Result<(), Failure> {
        let parent = self.transformed_surface();
        let tolerances = (parent.tolerance_u(), parent.tolerance_v());
        parent.load_range(surface, (bounds, tolerances), parent.trsf())
    }
}

/// The steps `(du, dv)` around the box: equal in 3d, a tenth of the remaining period on
/// a periodic direction.
fn bound_steps(
    adaptor: &impl BoundedSurface,
    periods: (f64, f64),
    (umin, umax, vmin, vmax): (f64, f64, f64, f64),
) -> Result<(f64, f64), Failure> {
    let (mut step_u, mut step_v) = (umax - umin, vmax - vmin);
    let (scale_u, scale_v) = (adaptor.u_resolution(1.0)?, adaptor.v_resolution(1.0)?);
    let (step3d_u, step3d_v) = (step_u / scale_u, step_v / scale_v);
    if step3d_u > step3d_v {
        step_v = step3d_u * scale_v;
    }
    if step3d_v > step3d_u {
        step_u = step3d_v * scale_u;
    }
    if periods.0 > 0.0 {
        step_u = 0.1 * (periods.0 - (umax - umin));
    }
    if periods.1 > 0.0 {
        step_v = 0.1 * (periods.1 - (vmax - vmin));
    }
    Ok((step_u, step_v))
}

/// `ChFi3d_BoundSrfImpl`: the adaptor reloaded on the box enlarged by the steps (kept in
/// the natural bounds of a non-periodic direction when asked).
fn bound_surface(
    adaptor: &impl BoundedSurface,
    bounds: (f64, f64, f64, f64),
    check_natural: bool,
) -> Result<(), Failure> {
    let mut surface = adaptor.base_surface();
    let basis = surface
        .cast::<GeomRectangularTrimmedSurface>()
        .get()
        .map(|t| t.basis_surface());
    if let Some(basis) = basis {
        surface = basis;
    }
    let (u1, u2, v1, v2) = surface.bounds();
    let period_u = if surface.is_u_periodic() {
        surface.u_period()?
    } else {
        0.0
    };
    let period_v = if surface.is_v_periodic() {
        surface.v_period()?
    } else {
        0.0
    };
    let (step_u, step_v) = bound_steps(adaptor, (period_u, period_v), bounds)?;
    let (umin, umax, vmin, vmax) = bounds;
    let (mut uu1, mut uu2, mut vv1, mut vv2) =
        (umin - step_u, umax + step_u, vmin - step_v, vmax + step_v);
    if check_natural && !adaptor.is_u_periodic()? {
        (uu1, uu2) = (std_max(uu1, u1), std_min(uu2, u2));
    }
    if check_natural && !adaptor.is_v_periodic()? {
        (vv1, vv2) = (std_max(vv1, v1), std_min(vv2, v2));
    }
    adaptor.apply_bounds(&surface, (uu1, uu2, vv1, vv2))
}

/// `ChFi3d_BoundFac(S, umin, umax, vmin, vmax, checknaturalbounds)`: the face adaptor
/// limited around the box (enlarged by equal 3d steps); `bound_fac(&s, (0.0, 1.0, 0.0,
/// 1.0), true)?`.
pub fn bound_fac(
    s: &BRepAdaptorSurface,
    bounds: (f64, f64, f64, f64),
    check_natural: bool,
) -> Result<(), Failure> {
    bound_surface(s, bounds, check_natural)
}

/// `ChFi3d_BoundSrf(S, umin, umax, vmin, vmax, checknaturalbounds)`: the surface adaptor
/// limited around the box; `bound_srf(&s, (0.0, 1.0, 0.0, 1.0), true)?`.
pub fn bound_srf(
    s: &GeomAdaptorSurface,
    bounds: (f64, f64, f64, f64),
    check_natural: bool,
) -> Result<(), Failure> {
    bound_surface(s, bounds, check_natural)
}

/// `ChFi3d_EnlargeBox(C, wd, wf, box1, box2)`: the points of the curve at `wd` and `wf`
/// added to the boxes; `enlarge_box_curve(&c, 0.0, 1.0, &mut b1, &mut b2)?`.
pub fn enlarge_box_curve(
    c: &Handle<dyn GeomCurve>,
    wd: f64,
    wf: f64,
    box1: &mut BoundingBox3,
    box2: &mut BoundingBox3,
) -> Result<(), Failure> {
    box1.add_point(c.value(wd)?);
    box2.add_point(c.value(wf)?);
    Ok(())
}

/// `ChFi3d_EnlargeBox(S, PC, wd, wf, box1, box2)`: the points of the curve on surface at
/// `wd` and `wf` added to the boxes; `enlarge_box_pcurve(&s, &pc, 0.0, 1.0, &mut b1,
/// &mut b2)?`.
pub fn enlarge_box_pcurve(
    s: &Handle<dyn Adaptor3dSurface>,
    pc: &Handle<dyn Geom2dCurve>,
    (wd, wf): (f64, f64),
    box1: &mut BoundingBox3,
    box2: &mut BoundingBox3,
) -> Result<(), Failure> {
    let p = pc.value(wd)?;
    box1.add_point(s.value(p.x(), p.y())?);
    let p = pc.value(wf)?;
    box2.add_point(s.value(p.x(), p.y())?);
    Ok(())
}

/// `ChFi3d_EnlargeBox(E, LF, w, box)`: the point of the edge at `w` and of its pcurve on
/// each face of the list added to the box (a null face is skipped);
/// `enlarge_box_edge(&e, &faces, 0.5, &mut b)?`.
pub fn enlarge_box_edge(
    e: &Edge,
    lf: &List<Shape>,
    w: f64,
    bx: &mut BoundingBox3,
) -> Result<(), Failure> {
    let bc = BRepAdaptorCurve::with_edge(e)?;
    bx.add_point(bc.value(w)?);
    for shape in lf.iter() {
        let f = topods::face(shape)?;
        if !f.is_null() {
            bc.initialize_on_face(e, &f)?;
            bx.add_point(bc.value(w)?);
        }
    }
    Ok(())
}
