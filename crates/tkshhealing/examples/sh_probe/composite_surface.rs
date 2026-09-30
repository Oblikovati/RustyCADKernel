//! ShapeExtend_CompositeSurface probe operations, the Rust twin of
//! tools/oracle/tkshhealing_probe_composite.cxx (which documents the request).
use super::common::{Reader, attempt, end, flag, int, pnt, pnt2d, real, show, xy, xyz};
use super::complex_curve::trsf;
use super::shapes::{TpShapes, tp_shapes};
use rustycad_tkernel::ncollection::{Array1, HArray2};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomGeometry, GeomSurface};
use rustycad_tkmath::gp::{Point2, Vector3};
use rustycad_tkshhealing::shape_extend::{
    JointValues, ShapeExtendCompositeSurface, ShapeExtendParametrisation, SurfaceGrid,
};

type Surface = ShapeExtendCompositeSurface;

fn param(value: i32) -> ShapeExtendParametrisation {
    [
        ShapeExtendParametrisation::Natural,
        ShapeExtendParametrisation::Uniform,
        ShapeExtendParametrisation::Unitary,
    ][value as usize]
}

/// `sh_cs_joints`: `lower n (value)...`.
fn joints(r: &mut Reader<'_>) -> Array1<'static, f64> {
    let (lower, n) = (r.integer(), r.integer());
    let mut a = Array1::with_bounds(lower, lower + n - 1).expect("probe joint bounds");
    for i in lower..lower + n {
        a.set_value(i, r.real()).expect("probe joint");
    }
    a
}

fn surface_index(t: &TpShapes, s: &Handle<dyn GeomSurface>) -> i32 {
    if let Some(i) = t.surfaces.iter().position(|candidate| candidate == s) {
        return i as i32;
    }
    if s.is_null() { -1 } else { -2 }
}

fn array(a: &JointValues) {
    let Some(a) = a.get() else {
        int(-1);
        return;
    };
    let a = a.array1();
    int(a.lower());
    int(a.length());
    a.as_slice().iter().for_each(|&v| real(v));
}

fn state(cs: &Surface) -> Result<(), Failure> {
    int(cs.nb_u_patches()?);
    int(cs.nb_v_patches()?);
    array(&cs.u_joint_values());
    array(&cs.v_joint_values());
    let (u1, u2, v1, v2) = cs.bounds();
    [u1, u2, v1, v2].into_iter().for_each(real);
    flag(cs.is_u_closed()?);
    flag(cs.is_v_closed()?);
    flag(cs.is_u_periodic());
    flag(cs.is_v_periodic());
    int(cs.continuity() as i32);
    [
        cs.is_cn_u(0)?,
        cs.is_cn_u(1)?,
        cs.is_cn_v(-1)?,
        cs.is_cn_v(2)?,
    ]
    .into_iter()
    .for_each(flag);
    real(cs.u_reversed_parameter(0.3)?);
    real(cs.v_reversed_parameter(-0.7)?);
    flag(cs.u_iso(0.5)?.is_null());
    flag(cs.v_iso(0.5)?.is_null());
    show(cs.u_joint_value(0), real);
    show(
        cs.nb_v_patches().and_then(|n| cs.v_joint_value(n + 2)),
        real,
    );
    show(cs.patch(0, 1), |p| flag(p.is_null()));
    Ok(())
}

fn vec(v: Vector3) {
    xyz(*v.xyz());
}

fn cell(cs: &Surface, i: i32, j: i32, uv: Point2) {
    attempt(|| {
        let l = cs.global_to_local(i, j, uv)?;
        pnt2d(l);
        pnt2d(cs.local_to_global(i, j, l)?);
        real(cs.u_global_to_local(i, j, uv.x())?);
        real(cs.v_global_to_local(i, j, uv.y())?);
        real(cs.u_local_to_global(i, j, l.x())?);
        real(cs.v_local_to_global(i, j, l.y())?);
        Ok(())
    });
    attempt(|| {
        let (changed, u_fact, t) = cs.global_to_local_transformation(i, j)?;
        flag(changed);
        real(u_fact);
        int(t.form() as i32);
        xy(t.apply_coordinates(uv.xy()));
        Ok(())
    });
}

fn evaluators(cs: &Surface, u: f64, v: f64) {
    show(cs.value(u, v), pnt);
    attempt(|| {
        let d = cs.eval_d1(u, v)?;
        pnt(d.point);
        [d.d1u, d.d1v].into_iter().for_each(vec);
        Ok(())
    });
    attempt(|| {
        let d = cs.eval_d2(u, v)?;
        [d.d2u, d.d2v, d.d2uv].into_iter().for_each(vec);
        Ok(())
    });
    attempt(|| {
        let d = cs.eval_d3(u, v)?;
        [d.d3u, d.d3v, d.d3uuv, d.d3uvv].into_iter().for_each(vec);
        Ok(())
    });
    show(cs.eval_dn(u, v, 1, 1), vec);
    show(cs.eval_dn(u, v, 0, 0), vec);
    show(cs.value_at(Point2::new(u, v)), pnt);
}

fn point(t: &TpShapes, cs: &Surface, u: f64, v: f64) {
    let (mut i, mut j) = (0, 0);
    attempt(|| {
        (i, j) = cs.locate_uv_point(Point2::new(u, v))?;
        int(i);
        int(j);
        Ok(())
    });
    show(cs.patch_at(u, v), |p| int(surface_index(t, &p)));
    cell(cs, i, j, Point2::new(u, v));
    evaluators(cs, u, v);
}

fn points(t: &TpShapes, cs: &Surface, pts: &[f64]) {
    pts.as_chunks::<2>()
        .0
        .iter()
        .for_each(|&[u, v]| point(t, cs, u, v));
}

fn build(r: &mut Reader<'_>, grid: &SurfaceGrid) -> Result<Surface, Failure> {
    match r.integer() {
        0 => Surface::with_grid(grid, param(r.integer())),
        2 => {
            let cs = Surface::new();
            flag(cs.init(grid, param(r.integer()))?);
            Ok(cs)
        }
        ctor => {
            let (uj, vj) = (joints(r), joints(r));
            if ctor == 1 {
                return Surface::with_joints(grid, &uj, &vj);
            }
            let cs = Surface::new();
            flag(cs.init_with_joints(grid, &uj, &vj)?);
            Ok(cs)
        }
    }
}

fn empty_checks(transform: rustycad_tkmath::gp::Transform3) -> Result<(), Failure> {
    let empty = Surface::new();
    flag(empty.patches().is_null());
    flag(empty.u_joint_values().is_null());
    let copy = empty.copy()?.cast::<Surface>();
    flag(copy.patches().is_null());
    empty.transform(transform)?;
    empty.set_u_first_value(1.0)?;
    flag(empty.init(&Handle::default(), ShapeExtendParametrisation::Natural)?);
    Ok(())
}

fn run(r: &mut Reader<'_>, t: &TpShapes) -> Result<(), Failure> {
    let (nu, nv) = (r.integer(), r.integer());
    let grid = HArray2::with_value(1, nu, 1, nv, Handle::<dyn GeomSurface>::default())
        .expect("probe grid");
    for i in 1..=nu {
        for j in 1..=nv {
            grid.change_array2()
                .set_value(i, j, t.surfaces[r.integer() as usize].clone())
                .expect("probe grid");
        }
    }
    let grid: SurfaceGrid = Handle::new(grid);
    let cs = build(r, &grid)?;
    let pts = r.reals();
    let prec = r.real();
    let transform = trsf(r);
    let (uj, vj) = (joints(r), joints(r));
    state(&cs)?;
    points(t, &cs, &pts);
    flag(cs.set_u_joint_values(&uj)?);
    flag(cs.set_v_joint_values(&vj)?);
    state(&cs)?;
    cs.set_u_first_value(-1.5)?;
    cs.set_v_first_value(2.0)?;
    array(&cs.u_joint_values());
    points(t, &cs, &pts);
    flag(cs.check_connectivity(prec)?);
    flag(cs.is_u_closed()?);
    flag(cs.is_v_closed()?);
    let copy = cs.copy()?.cast::<Surface>();
    state(&copy)?;
    points(t, &copy, &pts);
    cs.transform(transform)?;
    points(t, &cs, &pts);
    points(t, &copy, &pts);
    empty_checks(transform)
}

/// `sh_composite program nu nv (surface)... ctor ... npts (u v)... prec trsf newjoints`.
pub fn composite_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let t = tp_shapes(&mut r).expect("probe shape program");
    attempt(|| run(&mut r, &t));
    end();
}
