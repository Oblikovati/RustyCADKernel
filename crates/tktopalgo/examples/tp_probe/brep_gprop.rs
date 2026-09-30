//! BRepGProp probe operations (the package functions and the derived frameworks) over
//! shapes read by `tp_shape`, the Rust twin of tools/oracle/tktopalgo_probe_brepgprop.cxx.
use super::common::{Reader, attempt, end, flag, int, pnt, real, xyz};
use super::shape::tp_shape;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::{Axis1, Direction3, Plane3, Point3, Transform3, Vector3};
use rustycad_tkmath::poly::mesh_purpose;
use rustycad_tkmath::toploc::Location;
use rustycad_tktopalgo::brep_gprop::{
    BRepGProp, BRepGPropCinert, BRepGPropEdgeTool, BRepGPropMeshCinert, BRepGPropMeshProps,
    GPropsSums, MeshObjType, VolumeGKFlags,
};

/// `tp_gprops`: mass, centre, matrix, static moments, the moment about a fixed axis, then
/// the principal properties (or their exception).
pub fn gprops(p: &GProps) {
    real(p.mass());
    pnt(p.centre_of_mass());
    let m = p.matrix_of_inertia();
    for i in 1..=3 {
        for j in 1..=3 {
            real(m.value(i, j).expect("matrix index"));
        }
    }
    let (ix, iy, iz) = p.static_moments();
    [ix, iy, iz].into_iter().for_each(real);
    let axis = Axis1::new(
        Point3::new(1., 2., 3.),
        Direction3::new(1., 1., 1.).expect("axis"),
    );
    real(p.moment_of_inertia(axis));
    attempt(|| {
        let pp = p.principal_properties()?;
        let (a, b, c) = pp.moments();
        [a, b, c].into_iter().for_each(real);
        for v in [
            pp.first_axis_of_inertia(),
            pp.second_axis_of_inertia(),
            pp.third_axis_of_inertia(),
        ] {
            xyz(*v.xyz());
        }
        let (a, b, c) = pp.radius_of_gyration();
        [a, b, c].into_iter().for_each(real);
        flag(pp.has_symmetry_axis());
        flag(pp.has_symmetry_point());
        Ok(())
    });
}

/// `tp_plane`: `P N` of a plane.
pub fn plane(r: &mut Reader<'_>) -> Plane3 {
    let p = r.pnt();
    Plane3::new(p, r.dir()).expect("probe plane")
}

/// `shape mode flags...`: the package functions; the error, then the framework.
pub fn package_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let s = tp_shape(&mut r)?;
        let mut p = GProps::new();
        package_call(&mut r, &s.shape, &mut p)?;
        gprops(&p);
        Ok(())
    });
    end();
}

fn package_call(
    r: &mut Reader<'_>,
    shape: &rustycad_tkbrep::topods::Shape,
    p: &mut GProps,
) -> Result<(), Failure> {
    match r.integer() {
        0 => BRepGProp::linear_properties(shape, p, r.flag(), r.flag()),
        1 => BRepGProp::surface_properties(shape, p, r.flag(), r.flag()),
        2 => {
            let e = r.real();
            real(BRepGProp::surface_properties_eps(shape, p, e, r.flag())?);
            Ok(())
        }
        3 => BRepGProp::volume_properties(shape, p, r.flag(), r.flag(), r.flag()),
        4 => {
            let e = r.real();
            let closed = r.flag();
            real(BRepGProp::volume_properties_eps(
                shape,
                p,
                e,
                closed,
                r.flag(),
            )?);
            Ok(())
        }
        mode => {
            let pl = (mode == 6).then(|| plane(r));
            let e = r.real();
            let closed = r.flag();
            let flags = VolumeGKFlags {
                is_use_span: r.flag(),
                cg_flag: r.flag(),
                i_flag: r.flag(),
                skip_shared: r.flag(),
            };
            real(match pl {
                Some(pl) => BRepGProp::volume_properties_gk_plane(shape, p, pl, e, closed, flags)?,
                None => BRepGProp::volume_properties_gk(shape, p, e, closed, flags)?,
            });
            Ok(())
        }
    }
}

/// `shape edge x y z`: the edge tool, Cinert(C, loc) and a performed default Cinert.
pub fn cinert_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let s = tp_shape(&mut r)?;
        let e = s.edges[r.integer() as usize].clone();
        let loc = r.pnt();
        let c = BRepAdaptorCurve::with_edge(&e)?;
        let f = BRepGPropEdgeTool::first_parameter(&c)?;
        let l = BRepGPropEdgeTool::last_parameter(&c)?;
        real(f);
        real(l);
        int(BRepGPropEdgeTool::integration_order(&c)?);
        let n = BRepGPropEdgeTool::nb_intervals(&c, Continuity::CN)?;
        int(n);
        let mut t = Array1::with_bounds(1, n + 1).map_err(|e| e.into_failure("probe"))?;
        BRepGPropEdgeTool::intervals(&c, &mut t, Continuity::CN)?;
        t.iter().copied().for_each(real);
        pnt(BRepGPropEdgeTool::value(&c, 0.5 * (f + l))?);
        let (p, v) = BRepGPropEdgeTool::d1(&c, 0.5 * (f + l))?;
        pnt(p);
        xyz(*v.xyz());
        let a = BRepGPropCinert::with_curve(&c, loc)?;
        gprops(&a);
        let mut b = BRepGPropCinert::new();
        b.set_location(loc);
        b.perform(&c)?;
        gprops(&b);
        Ok(())
    });
    end();
}

/// `shape edge x y z`: PreparePolygon(E) and the MeshCinert of its points.
pub fn mesh_cinert_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let s = tp_shape(&mut r)?;
        let e = s.edges[r.integer() as usize].clone();
        let loc = r.pnt();
        let nodes = BRepGPropMeshCinert::prepare_polygon(&e)?;
        flag(nodes.is_none());
        if let Some(nodes) = &nodes {
            int(nodes.lower());
            int(nodes.length());
            nodes.iter().copied().for_each(pnt);
            let mut m = BRepGPropMeshCinert::new();
            m.set_location(loc);
            m.perform(nodes)?;
            gprops(&m);
        }
        let mut again = None;
        BRepGPropMeshCinert::prepare_polygon_into(&e, &mut again)?;
        flag(again.is_none());
        Ok(())
    });
    end();
}

/// `lower n (x y z)... x y z`: MeshCinert::Perform on nodes indexed from `lower`.
pub fn mesh_nodes_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (lower, n) = (r.integer(), r.integer());
    let mut nodes = Array1::with_bounds(lower, lower + n - 1).expect("probe nodes");
    for i in 0..n {
        nodes[lower + i] = r.pnt();
    }
    let loc = r.pnt();
    attempt(|| {
        let mut m = BRepGPropMeshCinert::new();
        m.set_location(loc);
        m.perform(&nodes)?;
        gprops(&m);
        Ok(())
    });
    end();
}

/// `tp_bgp_trsf`: a translation, rotation, scale, point mirror or scaled rotation.
fn trsf(r: &mut Reader<'_>) -> Result<Transform3, Failure> {
    let mut t = Transform3::default();
    match r.integer() {
        0 => t.set_translation(Vector3::from(r.xyz())),
        1 => {
            let (p, d) = (r.pnt(), r.dir());
            t.set_rotation(Axis1::new(p, d), r.real())?;
        }
        2 => {
            let p = r.pnt();
            t.set_scale(p, r.real())?;
        }
        3 => t.set_mirror_point(r.pnt()),
        _ => {
            let (p, d) = (r.pnt(), r.dir());
            t.set_rotation(Axis1::new(p, d), r.real())?;
            let mut scale = Transform3::default();
            scale.set_scale(p, r.real())?;
            t.pre_multiply(scale);
        }
    }
    Ok(t)
}

/// `shape face type ori located [trsf] x y z`: MeshProps on the face triangulation.
pub fn mesh_props_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let s = tp_shape(&mut r)?;
        let f = s.faces[r.integer() as usize].clone();
        let kind = if r.integer() == 0 {
            MeshObjType::Vinert
        } else {
            MeshObjType::Sinert
        };
        let mut m = BRepGPropMeshProps::new(kind);
        let o = super::face::orientation(&mut r);
        let located = r.flag();
        let l = if located {
            Location::new(trsf(&mut r)?)
        } else {
            Location::default()
        };
        m.set_location(r.pnt());
        int(m.get_mesh_obj_type() as i32);
        let (mesh, _) = BRepTool::triangulation(&f, mesh_purpose::NONE)?;
        if located {
            m.perform_located(&mesh, &l, o)?;
        } else {
            m.perform(&mesh, o)?;
        }
        gprops(&m);
        Ok(())
    });
    end();
}

/// `p1 p2 p3 apex volume n (l1 l2 w)...`: CalculateProps into zeroed sums.
pub fn mesh_calc_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (a, b, c, apex) = (r.pnt(), r.pnt(), r.pnt(), r.pnt());
    let volume = r.flag();
    let n = r.integer();
    let rule: Vec<f64> = (0..3 * n).map(|_| r.real()).collect();
    let mut sums: GPropsSums = [0.0; 10];
    attempt(|| {
        BRepGPropMeshProps::calculate_props((a, b, c), apex, volume, &mut sums, n, &rule)?;
        sums.iter().copied().for_each(real);
        Ok(())
    });
    end();
}
