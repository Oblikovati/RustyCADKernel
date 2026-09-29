//! IntSurf probe operations, the Rust twin of tools/oracle/tkgeomalgo_probe_intsurf.cxx.
use super::common::{Reader, end, flag, int, pnt, real, show, xyz};
use rustycad_tkgeomalgo::int_surf::{
    IntSurfPntOn2S, IntSurfQuadric, IntSurfQuadricTool, IntSurfTransition, make_transition,
};
use rustycad_tkmath::gp::{Axis3, Cone3, Cylinder3, Plane3, Point3, Sphere3, Torus3, Vector3};

/// Prints a transition: type, then the tangent flag, situation and opposite flag or the
/// exception each accessor raises.
fn transition(t: &IntSurfTransition) {
    int(t.transition_type() as i32);
    show(t.is_tangent(), flag);
    show(t.situation(), |s| int(s as i32));
    show(t.is_opposite(), flag);
}

/// `kind loc dir xdir r1 r2`: the quadric of ga_intsurf_quadric.
fn quadric(r: &mut Reader) -> IntSurfQuadric {
    let kind = r.integer();
    let (loc, dir, xdir) = (r.pnt(), r.dir(), r.dir());
    let (r1, r2) = (r.real(), r.real());
    let ax3 = Axis3::new(loc, dir, xdir).expect("probe frame");
    match kind {
        0 => IntSurfQuadric::from_plane(&Plane3::from_position(ax3)),
        1 => IntSurfQuadric::from_cylinder(&Cylinder3::new(ax3, r1).expect("probe cylinder")),
        2 => IntSurfQuadric::from_sphere(&Sphere3::new(ax3, r1).expect("probe sphere")),
        3 => IntSurfQuadric::from_cone(&Cone3::new(ax3, r2, r1).expect("probe cone")),
        4 => IntSurfQuadric::from_torus(&Torus3::new(ax3, r1, r2).expect("probe torus")),
        _ => IntSurfQuadric::new(),
    }
}

/// Implicit members at a point: distance, gradient, ValAndGrad, Normale(P), tolerance.
pub fn quadric_point_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let q = quadric(&mut r);
    let p = r.pnt();
    real(q.distance(p));
    xyz(*q.gradient(p).xyz());
    let (dist, grad) = q
        .val_and_grad(p)
        .unwrap_or((-7.0, Vector3::new(-1.0, -2.0, -3.0)));
    real(dist);
    xyz(*grad.xyz());
    show(q.normale_at(p), |n| xyz(*n.xyz()));
    real(IntSurfQuadricTool::tolerance(&q));
    int(q.type_quadric() as i32);
    end();
}

/// Parametric members at (u, v): Value, D1, DN, Normale(U, V), Parameters.
pub fn quadric_uv_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let q = quadric(&mut r);
    let (u, v) = (r.real(), r.real());
    pnt(q.value(u, v));
    let unchanged = (
        Point3::new(-1.0, -2.0, -3.0),
        Vector3::new(-4.0, -5.0, -6.0),
        Vector3::new(-7.0, -8.0, -9.0),
    );
    let (p, d1u, d1v) = q.d1(u, v).unwrap_or(unchanged);
    pnt(p);
    xyz(*d1u.xyz());
    xyz(*d1v.xyz());
    for (nu, nv) in [(1, 0), (0, 1), (1, 1), (2, 0)] {
        xyz(*q.dn(u, v, nu, nv).xyz());
    }
    show(q.normale(u, v), |n| xyz(*n.xyz()));
    let (pu, pv) = q.parameters(q.value(u, v)).unwrap_or((-11.0, -12.0));
    real(pu);
    real(pv);
    end();
}

/// `tg1 tg2 normal`: IntSurf::MakeTransition on two fresh transitions.
pub fn transition_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (t1, t2, n) = (r.xyz(), r.xyz(), r.dir());
    let (mut first, mut second) = (IntSurfTransition::new(), IntSurfTransition::new());
    make_transition(
        Vector3::from(t1),
        Vector3::from(t2),
        n,
        &mut first,
        &mut second,
    );
    transition(&first);
    transition(&second);
    end();
}

/// `p1 u1 v1 u2 v2 p2 u1 v1 u2 v2 tol3d tol2d`: IntSurf_PntOn2S::IsSame.
pub fn same_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let mut points = [IntSurfPntOn2S::new(); 2];
    for point in &mut points {
        let pt = r.pnt();
        let (u1, v1, u2, v2) = (r.real(), r.real(), r.real(), r.real());
        point.set_value_all(pt, u1, v1, u2, v2);
    }
    let (tol3, tol2) = (r.real(), r.real());
    flag(points[0].is_same(&points[1], tol3, tol2));
    end();
}
