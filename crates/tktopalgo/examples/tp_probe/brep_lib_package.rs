//! BRepLib package-function probe operations, the Rust twin of
//! tools/oracle/tktopalgo_probe_breplib_package.cxx (the request layout is documented
//! there).
use super::brep_lib_common::{geom_curve, shape, type_name, vertices};
use super::brep_lib_shapes::{faces_of, mesh, normals, shape_record};
use super::common::{Reader, attempt, end, flag, int, pnt, real, show};
use super::face::surface;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{Edge, Face, Shape};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, IndexedMapOfShape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::Point3;
use rustycad_tkmath::poly::{Connect, mesh_purpose};
use rustycad_tktopalgo::brep_lib::{
    self, BRepLibCheckCurveOnSurface, BRepLibMakeShell, BRepLibToolTriangulatedShape, RangeEnd,
    ValidRange,
};

fn sub_shapes(s: &Shape, kind: ShapeEnum) -> Result<IndexedMapOfShape, Failure> {
    let mut m = IndexedMapOfShape::default();
    if !s.is_null() {
        topexp::map_shapes_of_type(s, kind, &mut m)?;
    }
    Ok(m)
}
fn edges_of(s: &Shape) -> Result<Vec<Edge>, Failure> {
    let m = sub_shapes(s, ShapeEnum::Edge)?;
    (1..=m.extent())
        .map(|i| Ok(Edge::unchecked(m.find_key(i)?.clone())))
        .collect()
}
fn edge_ancestors(s: &Shape) -> Result<IndexedDataMapOfShapeListOfShape, Failure> {
    let mut m = IndexedDataMapOfShapeListOfShape::default();
    if !s.is_null() {
        topexp::map_shapes_and_ancestors(s, ShapeEnum::Edge, ShapeEnum::Face, &mut m)?;
    }
    Ok(m)
}
/// `tp_edge_faces`.
fn edge_faces(e: &Edge, anc: &List<Shape>) -> Result<Option<(Face, Face)>, Failure> {
    let mut f1 = Face::default();
    for f in anc.iter() {
        if f1.is_null() {
            f1 = Face::unchecked(f.clone());
        } else if !f1.is_same(f) {
            return Ok(Some((f1, Face::unchecked(f.clone()))));
        }
    }
    if f1.is_null() {
        return Ok(None);
    }
    let mut x = Explorer::new(&f1, ShapeEnum::Edge, ShapeEnum::Shape)?;
    while x.more() {
        if e.is_same(x.current()) && e.orientation() != x.current().orientation() {
            return Ok(Some((f1.clone(), f1)));
        }
        x.next()?;
    }
    Ok(None)
}
/// Runs a request body with the native `try`: the exception type ends the line.
fn guarded(values: &[f64], body: impl FnOnce(&mut Reader<'_>) -> Result<(), Failure>) {
    let mut r = Reader::new(values);
    attempt(|| body(&mut r));
    end();
}
fn continuity_of(r: &mut Reader<'_>) -> Continuity {
    const ALL: [Continuity; 7] = [
        Continuity::C0,
        Continuity::G1,
        Continuity::C1,
        Continuity::G2,
        Continuity::C2,
        Continuity::C3,
        Continuity::CN,
    ];
    ALL[r.integer() as usize]
}

/// `blp_build3d`.
pub fn build3d_case(values: &[f64]) {
    guarded(values, |r| {
        let s = shape_record(r)?;
        let (mode, tol, cont) = (r.integer(), r.real(), continuity_of(r));
        let (deg, seg) = (r.integer(), r.integer());
        match mode {
            0 => flag(brep_lib::build_curves3d(&s, tol, cont, deg, seg)?),
            1 => flag(brep_lib::build_curves3d_default(&s)?),
            _ => {
                for e in edges_of(&s)? {
                    show(brep_lib::build_curve3d(&e, tol, cont, deg, seg), flag);
                }
            }
        }
        shape(&s)
    });
}
/// `blp_samerange`.
pub fn samerange_case(values: &[f64]) {
    guarded(values, |r| {
        let s = shape_record(r)?;
        let (conf, tol) = (r.real(), r.real());
        for e in edges_of(&s)? {
            attempt(|| {
                flag(brep_lib::check_same_range(&e, conf)?);
                brep_lib::same_range(&e, tol)?;
                flag(brep_lib::check_same_range(&e, conf)?);
                Ok(())
            });
        }
        shape(&s)
    });
}
/// `blp_edgetol`.
pub fn edgetol_case(values: &[f64]) {
    guarded(values, |r| {
        let s = shape_record(r)?;
        let (mode, mn, mx) = (r.integer(), r.real(), r.real());
        if mode == 0 {
            flag(brep_lib::update_edge_tolerance(&s, mn, mx)?);
        } else {
            for e in edges_of(&s)? {
                show(brep_lib::update_edge_tol(&e, mn, mx), flag);
            }
        }
        shape(&s)
    });
}
/// `tp_edge_continuities`.
fn edge_continuities(s: &Shape) -> Result<(), Failure> {
    let m = edge_ancestors(s)?;
    for i in 1..=m.extent() {
        let e = Edge::unchecked(m.find_key(i)?.clone());
        let Some((f1, f2)) = edge_faces(&e, m.find_from_index(i)?)? else {
            int(-1);
            continue;
        };
        show(
            BRepTool::continuity_on_faces(&e, &f1, &f2).map(|c| c as i32),
            int,
        );
    }
    Ok(())
}
/// `blp_regularity`.
pub fn regularity_case(values: &[f64]) {
    guarded(values, |r| {
        let s = shape_record(r)?;
        let (mode, tol, n) = (r.integer(), r.real(), r.integer());
        let edges = sub_shapes(&s, ShapeEnum::Edge)?;
        let mut list: List<Shape> = List::default();
        for _ in 0..n {
            list.append(edges.find_key(r.integer() as usize)?.clone());
        }
        match mode {
            0 => brep_lib::encode_regularity(&s, tol)?,
            1 => brep_lib::encode_regularity_of_edges(&s, &list, tol)?,
            _ => {
                let m = edge_ancestors(&s)?;
                for i in 1..=m.extent() {
                    let e = Edge::unchecked(m.find_key(i)?.clone());
                    if let Some((f1, f2)) = edge_faces(&e, m.find_from_index(i)?)? {
                        attempt(|| brep_lib::encode_regularity_of_edge(&e, &f1, &f2, tol));
                    }
                }
            }
        }
        edge_continuities(&s)
    });
}
/// `blp_continuity`.
pub fn continuity_case(values: &[f64]) {
    guarded(values, |r| {
        let s = shape_record(r)?;
        let tol = r.real();
        let m = edge_ancestors(&s)?;
        for i in 1..=m.extent() {
            let e = Edge::unchecked(m.find_key(i)?.clone());
            let Some((f1, f2)) = edge_faces(&e, m.find_from_index(i)?)? else {
                int(-1);
                continue;
            };
            show(
                brep_lib::continuity_of_faces(&e, &f1, &f2, tol).map(|c| c as i32),
                int,
            );
        }
        Ok(())
    });
}
/// `blp_inner`.
pub fn inner_case(values: &[f64]) {
    guarded(values, |r| {
        let s = shape_record(r)?;
        brep_lib::update_inner_tolerances(&s)?;
        shape(&s)
    });
}
/// `blp_sort`.
pub fn sort_case(values: &[f64]) {
    guarded(values, |r| {
        let s = shape_record(r)?;
        let faces = sub_shapes(&s, ShapeEnum::Face)?;
        for lf in [brep_lib::sort_faces(&s)?, brep_lib::reverse_sort_faces(&s)?] {
            int(lf.extent());
            for f in lf.iter() {
                int(faces.find_index(f) as i32);
            }
        }
        Ok(())
    });
}
/// `blp_bounding`.
pub fn bounding_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let vs = vertices(&mut r)?;
        let n = r.integer();
        let mut list: List<Shape> = List::default();
        for _ in 0..n {
            list.append(vs[r.integer() as usize].clone().into_shape());
        }
        let (c, tol) =
            brep_lib::bounding_vertex(&list)?.unwrap_or((Point3::new(-7., -7., -7.), -7.));
        pnt(c);
        real(tol);
        Ok(())
    });
    end();
}
/// The flag and the outputs of FindValidRange, -7 where it writes none.
fn valid_range(found: &ValidRange) {
    flag(found.is_valid);
    real(found.first.unwrap_or(-7.));
    real(found.last.unwrap_or(-7.));
}
/// `blp_validrange`.
pub fn validrange_case(values: &[f64]) {
    guarded(values, |r| {
        let s = shape_record(r)?;
        for e in edges_of(&s)? {
            attempt(|| {
                valid_range(&brep_lib::find_valid_range_of_edge(&e)?);
                Ok(())
            });
        }
        Ok(())
    });
}
/// `blp_validrange2`.
pub fn validrange2_case(values: &[f64]) {
    guarded(values, |r| {
        let c = geom_curve(r)?;
        let (a, b) = (r.real(), r.real());
        let ac = GeomAdaptorCurve::with_range(&c, a, b)?;
        let tol_e = r.real();
        let v1 = RangeEnd {
            parameter: r.real(),
            point: r.pnt(),
            tolerance: r.real(),
        };
        let v2 = RangeEnd {
            parameter: r.real(),
            point: r.pnt(),
            tolerance: r.real(),
        };
        valid_range(&brep_lib::find_valid_range(&ac, tol_e, v1, v2)?);
        Ok(())
    });
}
/// `blp_pcurveplane`.
pub fn pcurveplane_case(values: &[f64]) {
    guarded(values, |r| {
        let s = shape_record(r)?;
        let mode = r.integer();
        for f in faces_of(&s)? {
            let edges = sub_shapes(&f, ShapeEnum::Edge)?;
            if mode == 2 {
                let list: Vec<Shape> = (1..=edges.extent())
                    .map(|k| edges.find_key(k).cloned())
                    .collect::<Result<_, _>>()?;
                attempt(|| brep_lib::build_pcurve_for_edges_on_plane(list.iter(), &f));
                continue;
            }
            for k in 1..=edges.extent() {
                let e = Edge::unchecked(edges.find_key(k)?.clone());
                if mode == 1 {
                    attempt(|| brep_lib::build_pcurve_for_edge_on_plane(&e, &f));
                    continue;
                }
                attempt(|| {
                    let (c, upd) = brep_lib::pcurve_for_edge_on_plane(&e, &f)?;
                    flag(upd);
                    type_name(&c);
                    Ok(())
                });
            }
        }
        shape(&s)
    });
}
/// `blp_ccos`.
pub fn ccos_case(values: &[f64]) {
    guarded(values, |r| {
        let s = shape_record(r)?;
        let parallel = r.flag();
        let m = edge_ancestors(&s)?;
        let mut reused = BRepLibCheckCurveOnSurface::new();
        for i in 1..=m.extent() {
            let e = Edge::unchecked(m.find_key(i)?.clone());
            for f in m.find_from_index(i)?.iter() {
                let f = Face::unchecked(f.clone());
                attempt(|| {
                    let mut c = BRepLibCheckCurveOnSurface::with_edge_face(&e, &f)?;
                    c.set_parallel(parallel);
                    flag(c.is_parallel());
                    c.perform();
                    flag(c.is_done());
                    int(c.error_status());
                    real(c.max_distance());
                    real(c.max_parameter());
                    Ok(())
                });
                attempt(|| {
                    reused.init(&e, &f)?;
                    reused.perform();
                    int(reused.error_status());
                    real(reused.max_distance());
                    Ok(())
                });
            }
        }
        Ok(())
    });
}
/// `blp_shell`.
pub fn shell_case(values: &[f64]) {
    guarded(values, |r| {
        let s = surface(r)?;
        let bounded = r.flag();
        let mut b = (0., 0., 0., 0.);
        if bounded {
            b = (r.real(), r.real(), r.real(), r.real());
        }
        let (segment, mode) = (r.flag(), r.integer());
        let m = if mode == 1 {
            let mut m = BRepLibMakeShell::new();
            flag(m.is_done());
            int(m.error() as i32);
            if !bounded {
                b = s.get().map(|g| g.bounds()).unwrap_or_default();
            }
            m.init(&s, b, segment)?;
            m
        } else if bounded {
            BRepLibMakeShell::with_bounds(&s, b, segment)?
        } else {
            BRepLibMakeShell::with_surface(&s, segment)?
        };
        flag(m.is_done());
        int(m.error() as i32);
        shape(&m.shell()?.into_shape())
    });
}
/// `blp_normals`.
pub fn normals_case(values: &[f64]) {
    guarded(values, |r| {
        let s = shape_record(r)?;
        let (n, uv, ang, force) = (r.integer(), r.flag(), r.real(), r.flag());
        mesh(&s, n, uv)?;
        show(brep_lib::ensure_normal_consistency(&s, ang, force), flag);
        normals(&s)
    });
}
/// `blp_deflection`.
pub fn deflection_case(values: &[f64]) {
    guarded(values, |r| {
        let s = shape_record(r)?;
        let n = r.integer();
        mesh(&s, n, r.flag())?;
        brep_lib::update_deflection(&s)?;
        for f in faces_of(&s)? {
            let (t, _) = BRepTool::triangulation(&f, mesh_purpose::NONE)?;
            match t.get() {
                Some(mesh) => real(mesh.deflection()),
                None => int(-1),
            }
        }
        Ok(())
    });
}
/// `blp_trinormals`.
pub fn trinormals_case(values: &[f64]) {
    guarded(values, |r| {
        let s = shape_record(r)?;
        let (n, uv, connect) = (r.integer(), r.flag(), r.flag());
        mesh(&s, n, uv)?;
        let mut shared = Connect::default();
        for f in faces_of(&s)? {
            let (t, _) = BRepTool::triangulation(&f, mesh_purpose::NONE)?;
            if connect {
                attempt(|| {
                    BRepLibToolTriangulatedShape::compute_normals_with_connect(&f, &t, &mut shared)
                });
            } else {
                attempt(|| BRepLibToolTriangulatedShape::compute_normals(&f, &t));
            }
        }
        normals(&s)
    });
}
