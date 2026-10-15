//! The ChFiKPart functions (`fl_kpart_fcts`, `fl_corner_spine`, `fl_kpart_make`,
//! `fl_kpart_corner`); the twin of tools/oracle/tkfillet_probe_chfikpart.cxx.
use super::blend_func::{geom_guide, geom_surface, surface};
use super::builder0::curve2d;
use super::builder0_solve::common_point;
use super::builder0_topo::curve3d;
use super::common::{Reader, attempt, end, flag, int, pnt, pnt2d, real, xyz};
use rustycad_tkbool::top_ope_brep_ds::TopOpeBRepDSDataStructure;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkfillet::chfi_ds::{ChamfMode, SurfData, SurfDataHandle, lock, shared};
use rustycad_tkfillet::chfikpart::{self, ChFiKPartComputeData};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Axis2, Circle3, Line3, Point2};

/// `fl_kpart_fcts`: see tools/oracle/tkfillet_probe_chfikpart.cxx.
pub fn fcts_case(values: &[f64]) {
    let r = &mut Reader::new(values);
    let (u, f, l, eps) = (r.real(), r.real(), r.real(), r.real());
    let (uv1, uv2, pd, pf) = (r.pnt2d(), r.pnt2d(), r.real(), r.real());
    let (c, gf, gl) = (geom_guide(r), r.real(), r.real());
    let s = geom_surface(r);
    real(chfikpart::in_period(u, f, l, eps));
    attempt(|| {
        curve2d(&chfikpart::pcurve(uv1, uv2, pd, pf)?.cast());
        Ok(())
    });
    attempt(|| {
        let (cg, sg) = (
            GeomAdaptorCurve::with_range(&c, gf, gl)?,
            GeomAdaptorSurface::with_surface(&s)?,
        );
        curve2d(&chfikpart::proj_pc(&cg, &sg)?);
        Ok(())
    });
    end();
}

/// `fl_corner_spine`: see tools/oracle/tkfillet_probe_chfikpart.cxx.
pub fn corner_spine_case(values: &[f64]) {
    let r = &mut Reader::new(values);
    let (s1, s2) = (surface(r), surface(r));
    let (p1s1, p1s2, p2s2, rad) = (r.pnt2d(), r.pnt2d(), r.pnt2d(), r.real());
    attempt(|| {
        let (cyl, circ, first, last) =
            chfikpart::corner_spine((&s1, &s2), (p1s1, p1s2, p2s2), rad)?;
        let pos = cyl.position();
        pnt(pos.location());
        xyz(pos.direction().xyz());
        xyz(pos.x_direction().xyz());
        xyz(pos.y_direction().xyz());
        real(cyl.radius());
        let cpos = circ.position();
        pnt(cpos.location());
        xyz(cpos.direction().xyz());
        xyz(cpos.x_direction().xyz());
        real(circ.radius());
        real(first);
        real(last);
        Ok(())
    });
    end();
}

/// The orientation of a probe integer.
fn orientation(r: &mut Reader<'_>) -> Orientation {
    [
        Orientation::Forward,
        Orientation::Reversed,
        Orientation::Internal,
        Orientation::External,
    ][r.integer() as usize]
}

/// `fl_kpart_surface`: the kind and three points of a surface of the data structure.
fn kpart_surface(s: &Handle<dyn GeomSurface>) {
    flag(s.is_null());
    let Some(surf) = s.get() else {
        return;
    };
    int(GeomAdaptorSurface::with_surface(s)
        .and_then(|a| a.get_type())
        .map_or(-1, |t| t as i32));
    attempt(|| {
        pnt(surf.value(0.2, 0.3)?);
        pnt(surf.value(1.1, 0.7)?);
        pnt(surf.value(2.0, -0.4)?);
        Ok(())
    });
}

/// `fl_kpart_data`: the surface data a ChFiKPart function wrote and its geometry.
fn kpart_data(ds: &TopOpeBRepDSDataStructure, d: &SurfDataHandle) -> Result<(), Failure> {
    let d = lock(d);
    int(ds.nb_surfaces());
    int(ds.nb_curves());
    int(d.surf());
    int(d.orientation() as i32);
    if d.surf() > 0 {
        kpart_surface(ds.surface(d.surf()).surface());
        real(ds.surface(d.surf()).tolerance());
    }
    for i in 1..=2 {
        let fi = d.interference(i);
        int(fi.line_index());
        if fi.line_index() > 0 {
            curve3d(ds.curve(fi.line_index()).curve());
            real(ds.curve(fi.line_index()).tolerance());
        }
        int(fi.transition() as i32);
        real(fi.first_parameter());
        real(fi.last_parameter());
        curve2d(fi.pcurve_on_face());
        curve2d(fi.pcurve_on_surf());
    }
    for first in [true, false] {
        for i in 1..=2 {
            common_point(d.vertex(first, i))?;
            pnt2d(d.get_2d_point(first, i));
        }
    }
    Ok(())
}

/// The spine of `fl_guide`: a line (kind 0) or a circle.
fn guide_spine(r: &mut Reader<'_>) -> (Option<Line3>, Option<Circle3>) {
    let kind = r.integer();
    let (p, n, x) = (r.pnt(), r.dir(), r.dir());
    let a = r.real();
    if kind == 0 {
        return (Some(Line3::new(p, n)), None);
    }
    let frame = Axis2::with_x_direction(p, n, x).expect("frame");
    (None, Some(Circle3::new(frame, a).expect("circle")))
}

/// The scalar inputs of `fl_kpart_make`.
struct MakeArgs {
    op: i32,
    ors: (Orientation, Orientation),
    ofpl: Orientation,
    first: f64,
    ab: (f64, f64),
    range: (f64, f64),
    plandab: bool,
    dison: bool,
}

/// The chamfer mode of a `fl_kpart_make` operation.
fn chamf_mode(op: i32) -> ChamfMode {
    match op {
        2 => ChamfMode::ConstThroatChamfer,
        3 => ChamfMode::ConstThroatWithPenetrationChamfer,
        _ => ChamfMode::ClassicChamfer,
    }
}

/// The two surfaces of `fl_kpart_make`.
type Pair<'a> = (&'a dyn Adaptor3dSurface, &'a dyn Adaptor3dSurface);

/// `fl_kpart_make` between two planes.
fn make_pln_pln(
    ds: &mut TopOpeBRepDSDataStructure,
    d: &SurfDataHandle,
    (s1, s2): Pair<'_>,
    l: &Line3,
    m: &MakeArgs,
) -> Result<bool, Failure> {
    let (p1, p2) = (s1.plane()?, s2.plane()?);
    match m.op {
        0 => chfikpart::make_fillet_pln_pln(ds, d, (&p1, &p2), m.ors, m.ab.0, l, m.first, m.ofpl),
        1..=3 => chfikpart::make_chamfer_pln_pln(
            ds,
            d,
            chamf_mode(m.op),
            (&p1, &p2),
            m.ors,
            m.ab,
            l,
            m.first,
            m.ofpl,
        ),
        _ => chfikpart::make_ch_asym_pln_pln(
            ds,
            d,
            (&p1, &p2),
            m.ors,
            m.ab,
            l,
            m.first,
            m.ofpl,
            m.dison,
        ),
    }
}

/// `fl_kpart_make` between a plane and a cylinder along a line.
fn make_pln_cyl_line(
    ds: &mut TopOpeBRepDSDataStructure,
    d: &SurfDataHandle,
    (s1, s2): Pair<'_>,
    l: &Line3,
    m: &MakeArgs,
) -> Result<bool, Failure> {
    let (p, c) = (s1.plane()?, s2.cylinder()?);
    match m.op {
        0 => chfikpart::make_fillet_pln_cyl_line(
            ds,
            d,
            (&p, &c),
            m.range,
            m.ors,
            m.ab.0,
            l,
            m.first,
            m.ofpl,
            m.plandab,
        ),
        1..=3 => chfikpart::make_chamfer_pln_cyl_line(
            ds,
            d,
            chamf_mode(m.op),
            (&p, &c),
            m.ors,
            m.ab,
            l,
            m.first,
            m.ofpl,
            m.plandab,
        ),
        _ => chfikpart::make_ch_asym_pln_cyl_line(
            ds,
            d,
            (&p, &c),
            m.ors,
            m.ab,
            l,
            m.first,
            m.ofpl,
            m.plandab,
            m.dison,
        ),
    }
}

/// `fl_kpart_make` between a plane and a cylinder along a circle.
fn make_pln_cyl_circle(
    ds: &mut TopOpeBRepDSDataStructure,
    d: &SurfDataHandle,
    (s1, s2): Pair<'_>,
    c: &Circle3,
    m: &MakeArgs,
) -> Result<bool, Failure> {
    let (p, cyl) = (s1.plane()?, s2.cylinder()?);
    match m.op {
        0 => chfikpart::make_fillet_pln_cyl_circle(
            ds,
            d,
            (&p, &cyl),
            m.range,
            m.ors,
            m.ab.0,
            c,
            m.first,
            m.ofpl,
            m.plandab,
        ),
        1..=3 => chfikpart::make_chamfer_pln_cyl_circle(
            ds,
            d,
            chamf_mode(m.op),
            (&p, &cyl),
            m.range,
            m.ors,
            m.ab,
            c,
            m.first,
            m.ofpl,
            m.plandab,
        ),
        _ => chfikpart::make_ch_asym_pln_cyl_circle(
            ds,
            d,
            (&p, &cyl),
            m.range,
            m.ors,
            m.ab,
            c,
            m.first,
            m.ofpl,
            m.plandab,
            m.dison,
        ),
    }
}

/// `fl_kpart_make` between a plane and a cone along a circle.
fn make_pln_con(
    ds: &mut TopOpeBRepDSDataStructure,
    d: &SurfDataHandle,
    (s1, s2): Pair<'_>,
    c: &Circle3,
    m: &MakeArgs,
) -> Result<bool, Failure> {
    let (p, con) = (s1.plane()?, s2.cone()?);
    match m.op {
        0 => chfikpart::make_fillet_pln_con(
            ds,
            d,
            (&p, &con),
            m.range,
            m.ors,
            m.ab.0,
            c,
            m.first,
            m.ofpl,
            m.plandab,
        ),
        1..=3 => chfikpart::make_chamfer_pln_con(
            ds,
            d,
            chamf_mode(m.op),
            (&p, &con),
            m.range,
            m.ors,
            m.ab,
            c,
            m.first,
            m.ofpl,
            m.plandab,
        ),
        _ => chfikpart::make_ch_asym_pln_con(
            ds,
            d,
            (&p, &con),
            m.range,
            m.ors,
            m.ab,
            c,
            m.first,
            m.ofpl,
            m.plandab,
            m.dison,
        ),
    }
}

/// The dispatch of `fl_kpart_make` on the second surface and the spine.
fn make_any(
    ds: &mut TopOpeBRepDSDataStructure,
    d: &SurfDataHandle,
    s: Pair<'_>,
    (line, circle): &(Option<Line3>, Option<Circle3>),
    m: &MakeArgs,
) -> Result<bool, Failure> {
    match (s.1.get_type()?, line, circle) {
        (SurfaceType::Plane, Some(l), _) => make_pln_pln(ds, d, s, l, m),
        (SurfaceType::Cylinder, Some(l), _) => make_pln_cyl_line(ds, d, s, l, m),
        (SurfaceType::Cylinder, _, Some(c)) => make_pln_cyl_circle(ds, d, s, c, m),
        (_, _, Some(c)) => make_pln_con(ds, d, s, c, m),
        _ => Err(Failure::new("probe: no circle")),
    }
}

/// `fl_kpart_make`: see tools/oracle/tkfillet_probe_chfikpart.cxx.
pub fn make_case(values: &[f64]) {
    let r = &mut Reader::new(values);
    let op = r.integer();
    let (s1, s2) = (surface(r), surface(r));
    let ors = (orientation(r), orientation(r));
    let ofpl = orientation(r);
    let spine = guide_spine(r);
    let (first, a, b, fu, lu) = (r.real(), r.real(), r.real(), r.real(), r.real());
    let (plandab, dison) = (r.flag(), r.flag());
    let m = MakeArgs {
        op,
        ors,
        ofpl,
        first,
        ab: (a, b),
        range: (fu, lu),
        plandab,
        dison,
    };
    let mut ds = TopOpeBRepDSDataStructure::new();
    let d = shared(SurfData::new());
    attempt(|| {
        let s = (s1.get().expect("surface"), s2.get().expect("surface"));
        flag(make_any(&mut ds, &d, s, &spine, &m)?);
        kpart_data(&ds, &d)
    });
    end();
}

/// `fl_kpart_corner`: see tools/oracle/tkfillet_probe_chfikpart.cxx.
pub fn corner_case(values: &[f64]) {
    let r = &mut Reader::new(values);
    let kind = r.integer();
    let (s1, s2, s3) = (surface(r), surface(r), surface(r));
    let o: Vec<Orientation> = (0..5).map(|_| orientation(r)).collect();
    let (r1, r2) = (r.real(), r.real());
    let p: [Point2; 4] = [r.pnt2d(), r.pnt2d(), r.pnt2d(), r.pnt2d()];
    let mut ds = TopOpeBRepDSDataStructure::new();
    let d = shared(SurfData::new());
    attempt(|| {
        let done = match kind {
            0 => ChFiKPartComputeData::compute_corner_torus(
                &mut ds,
                &d,
                (&s1, &s2),
                (o[0], o[2], o[3]),
                (r1, r2),
                p,
            )?,
            1 => ChFiKPartComputeData::compute_corner_sphere(
                &mut ds,
                &d,
                (&s1, &s2),
                (o[0], o[2]),
                r1,
                (p[0], p[1], p[2]),
            )?,
            _ => ChFiKPartComputeData::compute_corner_rotule(
                &mut ds,
                &d,
                (&s1, &s2, &s3),
                (o[0], o[2], o[3], o[4]),
                r1,
            )?,
        };
        flag(done);
        kpart_data(&ds, &d)
    });
    end();
}
