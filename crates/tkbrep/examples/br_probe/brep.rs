//! The geometry registry and the BRep_Builder script, the Rust twin of
//! tools/oracle/tkbrep_probe_brep.cxx.
use super::common::{Reader, failure, word};
use super::graph::{self, Graph};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Vertex};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};
use rustycad_tkg2d::geom2d::{Geom2dCircle, Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{
    GeomBezierCurve, GeomCircle, GeomCurve, GeomCylindricalSurface, GeomLine, GeomOffsetSurface,
    GeomPlane, GeomRectangularTrimmedSurface, GeomSurface, GeomTrimmedCurve,
};
use rustycad_tkmath::gp::{Axis1, Axis2, Axis2d, Axis3, Direction2, Direction3, Point2, Point3};
use rustycad_tkmath::poly::{
    Polygon2D, Polygon3D, PolygonOnTriangulation, Triangle, Triangulation,
};
use rustycad_tkmath::toploc::Location;

/// The geometry registries (`BrGeometry`).
#[derive(Default)]
pub struct Geometry {
    pub curves: Vec<Handle<dyn GeomCurve>>,
    pub surfaces: Vec<Handle<dyn GeomSurface>>,
    pub pcurves: Vec<Handle<dyn Geom2dCurve>>,
    pub polygons3d: Vec<Handle<Polygon3D>>,
    pub polygons2d: Vec<Handle<Polygon2D>>,
    pub triangulations: Vec<Handle<Triangulation>>,
    pub nodes: Vec<Handle<PolygonOnTriangulation>>,
}

/// `br_at`: a registry handle, null out of range.
pub fn at<T: ?Sized + RuntimeTyped>(v: &[Handle<T>], i: i32) -> Handle<T> {
    usize::try_from(i)
        .ok()
        .and_then(|i| v.get(i).cloned())
        .unwrap_or_default()
}

/// `br_id`: the registry index, -1 for null, -2 for a handle not in the registry.
pub fn id<T: ?Sized + RuntimeTyped, U: ?Sized + RuntimeTyped>(
    v: &[Handle<T>],
    h: &Handle<U>,
) -> i32 {
    if h.is_null() {
        return -1;
    }
    v.iter()
        .position(|x| x.object_address() == h.object_address())
        .map_or(-2, |i| i as i32)
}

pub fn point(r: &mut Reader<'_>) -> Point3 {
    let (x, y) = (r.real(), r.real());
    Point3::new(x, y, r.real())
}
fn direction(r: &mut Reader<'_>) -> Result<Direction3, Failure> {
    let (x, y) = (r.real(), r.real());
    Direction3::new(x, y, r.real())
}
pub fn point2d(r: &mut Reader<'_>) -> Point2 {
    let x = r.real();
    Point2::new(x, r.real())
}

fn curve_kind(
    g: &Geometry,
    r: &mut Reader<'_>,
    kind: i32,
) -> Result<Handle<dyn GeomCurve>, Failure> {
    Ok(match kind {
        0 => {
            let p = point(r);
            Handle::new(GeomLine::new(Axis1::new(p, direction(r)?))).into_cast()
        }
        1 => {
            let p = point(r);
            let n = direction(r)?;
            Handle::new(GeomCircle::new(Axis2::new(p, n)?, r.real())?).into_cast()
        }
        2 => {
            let poles: Vec<Point3> = (0..r.integer()).map(|_| point(r)).collect();
            Handle::new(GeomBezierCurve::new(&Array1::from_shared(&poles, 1))?).into_cast()
        }
        _ => {
            let (i, u1) = (r.integer(), r.real());
            Handle::new(GeomTrimmedCurve::new(
                &at(&g.curves, i),
                u1,
                r.real(),
                true,
                true,
            )?)
            .into_cast()
        }
    })
}

fn surface_kind(
    g: &Geometry,
    r: &mut Reader<'_>,
    kind: i32,
) -> Result<Handle<dyn GeomSurface>, Failure> {
    Ok(match kind {
        10 => {
            let p = point(r);
            Handle::new(GeomPlane::new(Axis3::from_normal(p, direction(r)?)?)).into_cast()
        }
        11 => {
            let p = point(r);
            let n = direction(r)?;
            Handle::new(GeomCylindricalSurface::new(
                Axis3::from_normal(p, n)?,
                r.real(),
            )?)
            .into_cast()
        }
        12 => {
            let (i, u1, u2, v1) = (r.integer(), r.real(), r.real(), r.real());
            let s = GeomRectangularTrimmedSurface::new(
                &at(&g.surfaces, i),
                (u1, u2),
                (v1, r.real()),
                (true, true),
            )?;
            Handle::new(s).into_cast()
        }
        _ => {
            let i = r.integer();
            Handle::new(GeomOffsetSurface::new(
                &at(&g.surfaces, i),
                r.real(),
                false,
            )?)
            .into_cast()
        }
    })
}

fn pcurve_kind(r: &mut Reader<'_>, kind: i32) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let p = point2d(r);
    if kind == 20 {
        let x = r.real();
        let d = Direction2::new(x, r.real())?;
        return Ok(Handle::new(Geom2dLine::new(Axis2d::new(p, d))).into_cast());
    }
    let axis = Axis2d::new(p, Direction2::new(1.0, 0.0)?);
    Ok(Handle::new(Geom2dCircle::from_axis(axis, r.real(), true)?).into_cast())
}

/// Pushes the result of a geometry constructor, printing `-` or its exception.
fn record<T: ?Sized + RuntimeTyped>(list: &mut Vec<Handle<T>>, made: Result<Handle<T>, Failure>) {
    match made {
        Ok(h) => {
            word("-");
            list.push(h);
        }
        Err(e) => {
            failure(&e);
            list.push(Handle::default());
        }
    }
}

fn read_triangulation(r: &mut Reader<'_>) -> Handle<Triangulation> {
    let nodes: Vec<Point3> = (0..r.integer()).map(|_| point(r)).collect();
    let triangles: Vec<Triangle> = (0..r.integer())
        .map(|_| {
            let (a, b) = (r.integer(), r.integer());
            Triangle::new(a, b, r.integer())
        })
        .collect();
    let t = Triangulation::from_nodes(
        &Array1::from_shared(&nodes, 1),
        &Array1::from_shared(&triangles, 1),
    )
    .unwrap();
    t.set_mesh_purpose(r.integer() as u32);
    Handle::new(t)
}

fn mesh_kind(g: &mut Geometry, r: &mut Reader<'_>, kind: i32) {
    match kind {
        30 => {
            let p: Vec<Point3> = (0..r.integer()).map(|_| point(r)).collect();
            g.polygons3d
                .push(Handle::new(Polygon3D::from_nodes(&Array1::from_shared(
                    &p, 1,
                ))));
        }
        31 => {
            let p: Vec<Point2> = (0..r.integer()).map(|_| point2d(r)).collect();
            g.polygons2d
                .push(Handle::new(Polygon2D::from_nodes(&Array1::from_shared(
                    &p, 1,
                ))));
        }
        32 => g.triangulations.push(read_triangulation(r)),
        _ => {
            let p: Vec<i32> = (0..r.integer()).map(|_| r.integer()).collect();
            g.nodes.push(Handle::new(PolygonOnTriangulation::from_nodes(
                &Array1::from_shared(&p, 1),
            )));
        }
    }
    word("-");
}

/// `br_read_geometry`.
pub fn read_geometry(g: &mut Geometry, r: &mut Reader<'_>) {
    let kind = r.integer();
    if kind < 10 {
        let made = curve_kind(g, r, kind);
        record(&mut g.curves, made);
    } else if kind < 20 {
        let made = surface_kind(g, r, kind);
        record(&mut g.surfaces, made);
    } else if kind < 30 {
        let made = pcurve_kind(r, kind);
        record(&mut g.pcurves, made);
    } else {
        mesh_kind(g, r, kind);
    }
}

/// `BrScene`: the shape graph and the geometry registries.
pub struct Scene {
    pub graph: Graph,
    pub geometry: Geometry,
}

impl Scene {
    pub fn shape(&self, r: &mut Reader<'_>) -> Shape {
        self.graph.shapes[r.integer() as usize].clone()
    }
    pub fn v(&self, r: &mut Reader<'_>) -> Result<Vertex, Failure> {
        topods::vertex(&self.shape(r))
    }
    pub fn e(&self, r: &mut Reader<'_>) -> Result<Edge, Failure> {
        topods::edge(&self.shape(r))
    }
    pub fn f(&self, r: &mut Reader<'_>) -> Result<Face, Failure> {
        topods::face(&self.shape(r))
    }
    pub fn loc(&self, r: &mut Reader<'_>) -> Location {
        self.graph.loc(r.integer())
    }
    pub fn push(&mut self, shape: impl Into<Shape>) {
        self.graph.shapes.push(shape.into());
    }
}

/// `br_scene`: the locations, the geometry and the commands of a BRep request.
pub fn scene(r: &mut Reader<'_>) -> Scene {
    let mut s = Scene {
        graph: Graph {
            locs: vec![Location::default()],
            datums: Vec::new(),
            shapes: Vec::new(),
        },
        geometry: Geometry::default(),
    };
    for _ in 0..r.integer() {
        graph::read_location(&mut s.graph, r);
    }
    for _ in 0..r.integer() {
        read_geometry(&mut s.geometry, r);
    }
    for _ in 0..r.integer() {
        super::brep_commands::command(&mut s, r);
    }
    s
}
