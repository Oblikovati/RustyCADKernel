//! The polyhedron operation of the IntCurveSurface probe (`ics_polyhedron`), split from
//! int_curve_surface.rs for size; the Rust twin of `ga_ics_polyhedron*` in
//! tools/oracle/tkgeomalgo_probe_intcurvesurface.cxx.
use super::common::{Reader, attempt, end, flag, int, pnt, real, xyz};
use super::ics_geometry::{params, surface};
use super::intf_tool::box3d_print;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkgeomalgo::int_curve_surface::{ThePolyhedronOfHInter, ThePolyhedronToolOfHInter};
use rustycad_tkmath::gp::Point3;

/// `ga_ics_polyhedron`: a polyhedron `mode ...`.
pub fn polyhedron(
    r: &mut Reader,
    s: &Adaptor3dSurfaceHandle,
) -> Result<ThePolyhedronOfHInter, Failure> {
    if r.integer() == 0 {
        let (nu, nv) = (r.integer(), r.integer());
        let (u1, v1, u2, v2) = (r.real(), r.real(), r.real(), r.real());
        return ThePolyhedronOfHInter::new(s, nu, nv, u1, v1, u2, v2);
    }
    let (up, vp) = (params(r), params(r));
    ThePolyhedronOfHInter::with_parameters(s, &up, &vp)
}

fn polyhedron_points(p: &ThePolyhedronOfHInter) -> Result<(), Failure> {
    let (_, nv) = p.size();
    let n = p.nb_points();
    for i in 1..=n {
        let (q, u, v) = p.point_with_parameters(i)?;
        pnt(q);
        real(u);
        real(v);
        pnt(p.point(i)?);
        pnt(p.point(i)?);
        let (u, v) = p.parameters(i)?;
        real(u);
        real(v);
        for j in [i + 1, i - 1, i + nv + 1, i - nv - 1, i + 2] {
            if (1..=n).contains(&j) {
                flag(p.is_on_bound(i, j));
            }
        }
    }
    Ok(())
}

fn polyhedron_triangles(p: &ThePolyhedronOfHInter, probe: Point3) -> Result<(), Failure> {
    let nt = p.nb_triangles();
    for t in 1..=nt {
        let [a, b, c] = p.triangle(t);
        [a, b, c].into_iter().for_each(int);
        box3d_print(&p.components_bounding()[t]);
        let (nrm, d) = p.plane_equation(t)?;
        xyz(nrm);
        real(d);
        let g = Point3::from((p.point(a)?.xyz() + p.point(b)?.xyz() + p.point(c)?.xyz()) / 3.0);
        flag(p.contain(t, g)?);
        flag(p.contain(t, probe)?);
    }
    for t in 0..=nt {
        let [a, b, c] = p.triangle(t.max(1));
        for pivot in [a, b, c] {
            for pedge in [0, a, b, c] {
                if pedge != pivot {
                    let (ret, tc, o) = p.tri_connex(t, pivot, pedge);
                    [ret, tc, o].into_iter().for_each(int);
                    let (ret, tc, o) = ThePolyhedronToolOfHInter::tri_connex(p, t, pivot, pedge);
                    [ret, tc, o].into_iter().for_each(int);
                }
            }
        }
    }
    Ok(())
}

fn polyhedron_summary(p: &ThePolyhedronOfHInter) -> Result<(), Failure> {
    type T = ThePolyhedronToolOfHInter;
    let (nu, nv) = p.size();
    [nu, nv, p.nb_triangles(), p.nb_points()]
        .into_iter()
        .for_each(int);
    real(p.deflection_over_estimation());
    real(p.get_border_deflection());
    box3d_print(p.bounding());
    int(T::nb_triangles(p));
    real(T::deflection_over_estimation(p));
    real(T::get_border_deflection(p));
    box3d_print(T::bounding(p));
    int(p.components_bounding().length());
    int(T::components_bounding(p).length());
    pnt(T::point(p, 1)?);
    T::triangle(p, 1).into_iter().for_each(int);
    flag(T::is_on_bound(p, 1, 2));
    Ok(())
}

fn polyhedron_members(r: &mut Reader, p: &mut ThePolyhedronOfHInter) -> Result<(), Failure> {
    let detail = r.flag();
    let probe = r.pnt();
    polyhedron_summary(p)?;
    if detail {
        polyhedron_points(p)?;
        polyhedron_triangles(p, probe)?;
    }
    let f = [r.flag(), r.flag(), r.flag(), r.flag()];
    p.u_min_singularity(f[0]);
    p.u_max_singularity(f[1]);
    p.v_min_singularity(f[2]);
    p.v_max_singularity(f[3]);
    let has = [
        p.has_u_min_singularity(),
        p.has_u_max_singularity(),
        p.has_v_min_singularity(),
        p.has_v_max_singularity(),
    ];
    has.into_iter().for_each(flag);
    p.set_point(Point3::new(9.0, 9.0, 9.0), 1, 1, 9.0, 9.0);
    pnt(p.point(1)?);
    p.dump();
    ThePolyhedronToolOfHInter::dump(p);
    p.set_deflection_over_estimation(r.real());
    real(p.deflection_over_estimation());
    box3d_print(p.bounding());
    p.fill_bounding()?;
    box3d_print(&p.components_bounding()[1]);
    box3d_print(&p.components_bounding()[p.nb_triangles()]);
    p.destroy();
    Ok(())
}

/// `surface polyhedron detail probe flags(4) flec`: every polyhedron and tool member.
pub fn polyhedron_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let s = surface(&mut r);
    attempt(|| {
        let mut p = polyhedron(&mut r, &s)?;
        polyhedron_members(&mut r, &mut p)
    });
    end();
}
