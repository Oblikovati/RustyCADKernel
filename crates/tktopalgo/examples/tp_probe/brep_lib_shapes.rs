//! The shape records and the grid mesher of the BRepLib package probes, the Rust twin of
//! tools/oracle/tktopalgo_probe_breplib_shapes.cxx (the layout is documented there).
use super::brep_lib_common::geom_curve;
use super::common::{Reader, int, real};
use super::face::{surface, tp_face};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{Compound, Edge, Face, Shape};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::Point2;
use rustycad_tkmath::poly::{PolygonOnTriangulation, Triangle, Triangulation, mesh_purpose};
use rustycad_tkmath::toploc::Location;
use rustycad_tktopalgo::brep_lib::{BRepLibMakeEdge, BRepLibMakeShell};

/// `tp_shape_record`.
pub fn shape_record(r: &mut Reader<'_>) -> Result<Shape, Failure> {
    let builder = BRepBuilder::default();
    match r.integer() {
        0 => Ok(tp_face(r)?.face.into_shape()),
        1 => {
            let s = surface(r)?;
            if r.flag() {
                let (u1, u2, v1, v2) = (r.real(), r.real(), r.real(), r.real());
                let m = BRepLibMakeShell::with_bounds(&s, (u1, u2, v1, v2), r.flag())?;
                return Ok(m.shell()?.into_shape());
            }
            Ok(BRepLibMakeShell::with_surface(&s, r.flag())?
                .shell()?
                .into_shape())
        }
        2 => {
            let mut c = Compound::default();
            builder.make_compound(&mut c);
            let n = r.integer();
            for _ in 0..n {
                let child = shape_record(r)?;
                builder.add(&mut c, &child)?;
            }
            Ok(c.into_shape())
        }
        _ => {
            let c = geom_curve(r)?;
            let (p1, p2, t1, t2) = (r.real(), r.real(), r.real(), r.real());
            let m = BRepLibMakeEdge::from_curve_params(&c, p1, p2)?;
            if !m.is_done() {
                return Ok(Shape::default());
            }
            if !m.vertex1()?.is_null() {
                builder.update_vertex_tolerance(m.vertex1()?, t1)?;
            }
            if !m.vertex2()?.is_null() {
                builder.update_vertex_tolerance(m.vertex2()?, t2)?;
            }
            Ok(m.edge()?.into_shape())
        }
    }
}

/// `tp_face_box`: the box of the ends of the curves on surface of the face.
fn face_box(f: &Face) -> Result<[f64; 4], Failure> {
    let mut b = [f64::MAX, -f64::MAX, f64::MAX, -f64::MAX];
    let mut x = Explorer::new(f, ShapeEnum::Edge, ShapeEnum::Shape)?;
    while x.more() {
        let e = Edge::unchecked(x.current().clone());
        x.next()?;
        let found = BRepTool::curve_on_face(&e, f)?;
        let Some(c) = found.pcurve.get() else {
            continue;
        };
        for t in [found.first, found.last] {
            if precision::is_infinite(t) {
                continue;
            }
            let p: Point2 = c.value(t)?;
            b = [
                std_min(b[0], p.x()),
                std_max(b[1], p.x()),
                std_min(b[2], p.y()),
                std_max(b[3], p.y()),
            ];
        }
    }
    Ok(b)
}

/// `tp_side_polygon`: the polygon of the curve of an edge on a side of the grid box.
fn side_polygon(
    e: &Edge,
    f: &Face,
    n: i32,
    b: &[f64; 4],
) -> Result<Option<Handle<PolygonOnTriangulation>>, Failure> {
    let found = BRepTool::curve_on_face(e, f)?;
    let Some(c) = found.pcurve.get() else {
        return Ok(None);
    };
    let (a, bb) = (found.first, found.last);
    if precision::is_infinite(a) || precision::is_infinite(bb) {
        return Ok(None);
    }
    let (m, pa, pb) = (c.value(0.5 * (a + bb))?, c.value(a)?, c.value(bb)?);
    let tol = 1e-9;
    let side = if (m.x() - b[0]).abs() < tol {
        0
    } else if (m.x() - b[1]).abs() < tol {
        1
    } else if (m.y() - b[2]).abs() < tol {
        2
    } else if (m.y() - b[3]).abs() < tol {
        3
    } else {
        return Ok(None);
    };
    let along_v = side < 2;
    let increasing = if along_v {
        pa.y() < pb.y()
    } else {
        pa.x() < pb.x()
    };
    let p = PolygonOnTriangulation::new(n + 1, false)?;
    for k in 0..=n {
        let s = if increasing { k } else { n - k };
        let i = if along_v {
            if side == 0 { 0 } else { n }
        } else {
            s
        };
        let j = if along_v {
            s
        } else if side == 2 {
            0
        } else {
            n
        };
        p.set_node(k + 1, i * (n + 1) + j + 1)?;
    }
    Ok(Some(Handle::new(p)))
}

/// `tp_edge_polygon`.
fn edge_polygon(
    e: &Edge,
    f: &Face,
    t: &Handle<Triangulation>,
    n: i32,
    b: &[f64; 4],
) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    let location = Location::default();
    if !BRepTool::is_closed_on_face(e, f)? {
        if let Some(p) = side_polygon(e, f, n, b)? {
            builder.update_edge_polygon_on_triangulation(e, &p, t, &location)?;
        }
        return Ok(());
    }
    let fe = Edge::unchecked(e.oriented(Orientation::Forward));
    let ff = Face::unchecked(f.oriented(Orientation::Forward));
    let p1 = side_polygon(&fe, &ff, n, b)?;
    let p2 = side_polygon(&Edge::unchecked(fe.reversed()), &ff, n, b)?;
    if let (Some(p1), Some(p2)) = (p1, p2) {
        builder.update_edge_polygons_on_triangulation(e, (&p1, &p2), t, &location)?;
    }
    Ok(())
}

/// `tp_mesh_face`.
fn mesh_face(f: &Face, n: i32, uv: bool) -> Result<(), Failure> {
    let s = BRepTool::surface(f)?;
    let Some(surf) = s.get() else { return Ok(()) };
    let b = face_box(f)?;
    if b[0] > b[1] {
        return Ok(());
    }
    let t = Triangulation::with_sizes((n + 1) * (n + 1), 2 * n * n, uv, false)?;
    for i in 0..=n {
        for j in 0..=n {
            let u = b[0] + (b[1] - b[0]) * f64::from(i) / f64::from(n);
            let v = b[2] + (b[3] - b[2]) * f64::from(j) / f64::from(n);
            let k = i * (n + 1) + j + 1;
            t.set_node(k, surf.value(u, v)?)?;
            if uv {
                t.set_uv_node(k, Point2::new(u, v))?;
            }
        }
    }
    let mut tri = 0;
    for i in 0..n {
        for j in 0..n {
            let a = i * (n + 1) + j + 1;
            let (bb, c, d) = (a + n + 1, a + n + 2, a + 1);
            tri += 1;
            t.set_triangle(tri, Triangle::new(a, bb, c))?;
            tri += 1;
            t.set_triangle(tri, Triangle::new(a, c, d))?;
        }
    }
    let t = Handle::new(t);
    BRepBuilder::default().update_face_triangulation(f, &t, true)?;
    let mut x = Explorer::new(f, ShapeEnum::Edge, ShapeEnum::Shape)?;
    while x.more() {
        let e = Edge::unchecked(x.current().clone());
        x.next()?;
        edge_polygon(&e, f, &t, n, &b)?;
    }
    Ok(())
}

/// The faces of a shape in MapShapes order.
pub fn faces_of(sh: &Shape) -> Result<Vec<Face>, Failure> {
    let mut faces = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(sh, ShapeEnum::Face, &mut faces)?;
    (1..=faces.extent())
        .map(|i| Ok(Face::unchecked(faces.find_key(i)?.clone())))
        .collect()
}

/// `tp_mesh`.
pub fn mesh(sh: &Shape, n: i32, uv: bool) -> Result<(), Failure> {
    for f in faces_of(sh)? {
        mesh_face(&f, n, uv)?;
    }
    Ok(())
}

/// `tp_normals`.
pub fn normals(sh: &Shape) -> Result<(), Failure> {
    for f in faces_of(sh)? {
        let (t, _) = BRepTool::triangulation(&f, mesh_purpose::NONE)?;
        let Some(mesh) = t.get() else {
            int(-1);
            continue;
        };
        if !mesh.has_normals() {
            int(0);
            continue;
        }
        int(mesh.nb_nodes());
        for k in 1..=mesh.nb_nodes() {
            let v = mesh.normal_vec3(k)?;
            real(f64::from(v.x()));
            real(f64::from(v.y()));
            real(f64::from(v.z()));
        }
    }
    Ok(())
}
