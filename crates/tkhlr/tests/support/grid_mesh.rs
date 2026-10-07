//! The deterministic mesher of the polyhedral HLR probe (`hb_mesh` in
//! tools/oracle/tkhlr_probe_hlrbrep_poly.cxx), shared by the probe and the tests: a
//! grid on the UV rectangle of each face, a fan on a disk.
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCircle, Geom2dConic};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::Point2;
use rustycad_tkmath::poly::{PolygonOnTriangulation, Triangle, Triangulation};

fn arr<T: Default + Clone>(values: &[T]) -> Array1<'static, T> {
    let mut a = Array1::with_bounds(1, values.len() as i32).expect("probe array");
    for (k, v) in values.iter().enumerate() {
        *a.change_value(k as i32 + 1).expect("probe array") = v.clone();
    }
    a
}

/// `hb_grid_polygon`.
fn grid_polygon(
    a: Point2,
    z: Point2,
    (f0, l0): (f64, f64),
    us: [f64; 4],
    nu: i32,
    nv: i32,
) -> Handle<PolygonOnTriangulation> {
    let eps = 1e-9;
    let vert = (a.x() - z.x()).abs() < eps;
    let n = if vert { nv } else { nu };
    let at_min = if vert {
        (a.x() - us[0]).abs() < eps
    } else {
        (a.y() - us[2]).abs() < eps
    };
    let up = if vert { a.y() < z.y() } else { a.x() < z.x() };
    let (mut nodes, mut pars) = (Vec::new(), Vec::new());
    for k in 0..=n {
        let s = if up { k } else { n - k };
        let (i, j) = if vert {
            (if at_min { 0 } else { nu }, s)
        } else {
            (s, if at_min { 0 } else { nv })
        };
        nodes.push(1 + j * (nu + 1) + i);
        let c = if vert {
            us[2] + (us[3] - us[2]) * f64::from(j) / f64::from(nv)
        } else {
            us[0] + (us[1] - us[0]) * f64::from(i) / f64::from(nu)
        };
        let frac = if vert {
            (c - a.y()) / (z.y() - a.y())
        } else {
            (c - a.x()) / (z.x() - a.x())
        };
        pars.push(f0 + (l0 - f0) * frac);
    }
    Handle::new(PolygonOnTriangulation::from_nodes_and_parameters(
        &arr(&nodes),
        &arr(&pars),
    ))
}

fn pcurve_ends(e: &Edge, f: &Face) -> Result<(Point2, Point2, f64, f64), Failure> {
    let pc = BRepTool::curve_on_face(e, f)?;
    let c = pc.pcurve.get().ok_or_else(|| Failure::new("no pcurve"))?;
    Ok((c.value(pc.first)?, c.value(pc.last)?, pc.first, pc.last))
}

/// `hb_mesh_grid`.
fn mesh_grid(f: &Face, nu: i32, nv: i32) -> Result<(), Failure> {
    let (s, loc) = BRepTool::surface_with_location(f)?;
    let s = s.get().ok_or_else(|| Failure::new("no surface"))?;
    let (u1, u2, v1, v2) = brep_tools::uv_bounds(f)?;
    let us = [u1, u2, v1, v2];
    let t = Triangulation::with_sizes((nu + 1) * (nv + 1), 2 * nu * nv, true, false)?;
    for j in 0..=nv {
        for i in 0..=nu {
            let u = us[0] + (us[1] - us[0]) * f64::from(i) / f64::from(nu);
            let v = us[2] + (us[3] - us[2]) * f64::from(j) / f64::from(nv);
            let k = 1 + j * (nu + 1) + i;
            t.set_node(k, s.value(u, v)?)?;
            t.set_uv_node(k, Point2::new(u, v))?;
        }
    }
    let mut k = 1;
    for j in 0..nv {
        for i in 0..nu {
            let a = 1 + j * (nu + 1) + i;
            let (b, c, d) = (a + 1, a + 1 + nu + 1, a + nu + 1);
            t.set_triangle(k, Triangle::new(a, b, c))?;
            t.set_triangle(k + 1, Triangle::new(a, c, d))?;
            k += 2;
        }
    }
    let t = Handle::new(t);
    let bb = BRepBuilder::default();
    bb.update_face_triangulation(f, &t, true)?;
    let mut done: Vec<Edge> = Vec::new();
    for es in Explorer::new(&Shape::from(f.clone()), ShapeEnum::Edge, ShapeEnum::Shape)? {
        let e = topods::edge(&es)?;
        if done.iter().any(|d| d.is_same(&e)) {
            continue;
        }
        done.push(e.clone());
        if BRepTool::is_closed_on_face(&e, f)? {
            let mut p = Vec::new();
            for o in [Orientation::Forward, Orientation::Reversed] {
                let mut eo = e.clone();
                eo.set_orientation(o);
                let (a, z, f0, l0) = pcurve_ends(&eo, f)?;
                p.push(grid_polygon(a, z, (f0, l0), us, nu, nv));
            }
            bb.update_edge_polygons_on_triangulation(&e, (&p[0], &p[1]), &t, &loc)?;
        } else {
            let (a, z, f0, l0) = pcurve_ends(&e, f)?;
            bb.update_edge_polygon_on_triangulation(
                &e,
                &grid_polygon(a, z, (f0, l0), us, nu, nv),
                &t,
                &loc,
            )?;
        }
    }
    Ok(())
}

/// `hb_mesh_fan`.
fn mesh_fan(f: &Face, e: &Edge, n: i32) -> Result<(), Failure> {
    let (s, loc) = BRepTool::surface_with_location(f)?;
    let s = s.get().ok_or_else(|| Failure::new("no surface"))?;
    let pc = BRepTool::curve_on_face(e, f)?;
    let (f0, l0) = (pc.first, pc.last);
    let circle = pc.pcurve.cast::<Geom2dCircle>();
    let c = circle
        .get()
        .ok_or_else(|| Failure::new("not a circle"))?
        .location();
    let curve = pc.pcurve.get().ok_or_else(|| Failure::new("no pcurve"))?;
    let t = Triangulation::with_sizes(n + 1, n, true, false)?;
    t.set_node(1, s.value(c.x(), c.y())?)?;
    t.set_uv_node(1, c)?;
    let mut pars = Vec::new();
    for k in 0..n {
        let tk = f0 + (l0 - f0) * f64::from(k) / f64::from(n);
        let uv = curve.value(tk)?;
        pars.push(tk);
        t.set_node(k + 2, s.value(uv.x(), uv.y())?)?;
        t.set_uv_node(k + 2, uv)?;
    }
    pars.push(l0);
    let (a, b) = (t.uv_node(2)?, t.uv_node(3)?);
    let flip = (a.x() - c.x()) * (b.y() - c.y()) - (a.y() - c.y()) * (b.x() - c.x()) < 0.;
    for k in 0..n {
        let (i, j) = (k + 2, if k + 1 < n { k + 3 } else { 2 });
        t.set_triangle(
            k + 1,
            if flip {
                Triangle::new(1, j, i)
            } else {
                Triangle::new(1, i, j)
            },
        )?;
    }
    let mut nodes: Vec<i32> = (0..n).map(|k| k + 2).collect();
    nodes.push(2);
    let t = Handle::new(t);
    let bb = BRepBuilder::default();
    bb.update_face_triangulation(f, &t, true)?;
    let p = Handle::new(PolygonOnTriangulation::from_nodes_and_parameters(
        &arr(&nodes),
        &arr(&pars),
    ));
    bb.update_edge_polygon_on_triangulation(e, &p, &t, &loc)
}

/// `hb_mesh` of tools/oracle/tkhlr_probe_hlrbrep_poly.cxx: a fan on a plane bounded by
/// one edge, else an `n` x `n` grid on a plane and an `n` x `m` grid on other surfaces,
/// with the polygons of the edges on the triangulations; `mesh(&shape, 8, 2)?`.
pub fn mesh(sh: &Shape, n: i32, m: i32) -> Result<(), Failure> {
    for fs in Explorer::new(sh, ShapeEnum::Face, ShapeEnum::Shape)? {
        let f = topods::face(&fs)?;
        let es: Vec<Shape> = Explorer::new(&fs, ShapeEnum::Edge, ShapeEnum::Shape)?.collect();
        let plane = GeomAdaptorSurface::with_surface(&BRepTool::surface(&f)?)?.get_type()?
            == SurfaceType::Plane;
        if plane && es.len() == 1 {
            mesh_fan(&f, &topods::edge(&es[0])?, n)?;
        } else {
            mesh_grid(&f, n, if plane { n } else { m })?;
        }
    }
    Ok(())
}
