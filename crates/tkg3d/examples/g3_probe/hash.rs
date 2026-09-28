//! The GeomHash hashers of the TKG3d probe; the counterpart of tools/oracle/tkg3d_probe_hash.cxx.
use super::common::{built_curve, built_surface, end, flag};
use super::print::Reader;
use rustycad_tkernel::ncollection::HashPolicy;
use rustycad_tkernel::standard::{Handle, RuntimeTyped};
use rustycad_tkg3d::geom_hash::{
    GeomHashAxisPlacement, GeomHashCurveHasher, GeomHashDirectionHasher, GeomHashPointHasher,
    GeomHashPolygon2DHasher, GeomHashPolygon3DHasher, GeomHashPolygonOnTriHasher,
    GeomHashSurfaceHasher, GeomHashTriangulationHasher, GeomHashVectorHasher, PolygonOnTriHashKey,
};
use rustycad_tkmath::gp::{Axis2, Point2};
use rustycad_tkmath::poly::{
    Polygon2D, Polygon3D, PolygonOnTriangulation, Triangle, Triangulation,
};

fn hash_value(value: usize) {
    print!("h{value} ");
}

/// The hasher tolerances: a negative CompTolerance selects the default constructor.
fn hasher<H: Default>(r: &mut Reader, make: fn(f64, f64) -> H) -> H {
    let (comp, tol) = (r.real(), r.real());
    tolerant(comp, tol, make)
}
fn tolerant<H: Default>(comp: f64, tol: f64, make: fn(f64, f64) -> H) -> H {
    if comp < 0.0 {
        return H::default();
    }
    make(comp, tol)
}

/// Prints every hash, then equality for every ordered pair; each case ends its line.
fn hash_all<K>(hasher: &impl HashPolicy<K>, keys: &[K]) {
    for key in keys {
        hash_value(hasher.hash(key));
    }
    for a in keys {
        for b in keys {
            flag(hasher.equal(a, b));
        }
    }
}

/// Items `0 <object> | 1 (null) | 2 index`; `None` after a reported construction failure.
fn items<T: ?Sized + RuntimeTyped>(
    r: &mut Reader,
    build: impl Fn(&mut Reader) -> Option<Handle<T>>,
) -> Option<Vec<Handle<T>>> {
    let n = r.integer();
    let mut handles: Vec<Handle<T>> = Vec::new();
    for _ in 0..n {
        let handle = match r.integer() {
            0 => build(r)?,
            2 => handles[r.integer() as usize].clone(),
            _ => Handle::default(),
        };
        handles.push(handle);
    }
    Some(handles)
}

pub fn hash_curves_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let h = hasher(&mut r, GeomHashCurveHasher::new);
    if let Some(keys) = items(&mut r, built_curve) {
        hash_all(&h, &keys);
        end();
    }
}

pub fn hash_surfaces_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let h = hasher(&mut r, GeomHashSurfaceHasher::new);
    if let Some(keys) = items(&mut r, built_surface) {
        hash_all(&h, &keys);
        end();
    }
}

pub fn hash_primitives_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (comp, tol) = (r.real(), r.real());
    let points = [r.pnt(), r.pnt()];
    let directions = [r.dir(), r.dir()];
    let vectors = [r.vec(), r.vec()];
    let mut frames: Vec<Axis2> = Vec::new();
    for _ in 0..2 {
        let (p, n, x) = (r.pnt(), r.dir(), r.dir());
        match Axis2::with_x_direction(p, n, x) {
            Ok(frame) => frames.push(frame),
            Err(error) => return println!("{}", error.exception_type()),
        }
    }
    print!("p ");
    hash_all(&tolerant(comp, tol, GeomHashPointHasher::new), &points);
    print!("d ");
    hash_all(
        &tolerant(comp, tol, GeomHashDirectionHasher::new),
        &directions,
    );
    print!("v ");
    hash_all(&tolerant(comp, tol, GeomHashVectorHasher::new), &vectors);
    print!("f ");
    hash_all(&tolerant(comp, tol, GeomHashAxisPlacement::new), &frames);
    end();
}

fn poly2d(r: &mut Reader) -> Option<Handle<Polygon2D>> {
    let (deflection, n) = (r.real(), r.integer());
    let polygon = Polygon2D::new(n).expect("probe polygon");
    polygon.change_nodes(|nodes| {
        for i in 1..=n {
            let x = r.real();
            nodes.set_value(i, Point2::new(x, r.real())).unwrap();
        }
    });
    polygon.set_deflection(deflection);
    Some(Handle::new(polygon))
}

fn poly3d(r: &mut Reader) -> Option<Handle<Polygon3D>> {
    let (deflection, with_parameters, n) = (r.real(), r.flag(), r.integer());
    let polygon = Polygon3D::new(n, with_parameters).expect("probe polygon");
    polygon.change_nodes(|nodes| (1..=n).for_each(|i| nodes.set_value(i, r.pnt()).unwrap()));
    if with_parameters {
        let edit = |params: &mut rustycad_tkernel::ncollection::Array1<'static, f64>| {
            (1..=n).for_each(|i| params.set_value(i, r.real()).unwrap())
        };
        polygon.change_parameters(edit).unwrap();
    }
    polygon.set_deflection(deflection);
    Some(Handle::new(polygon))
}

fn on_triangulation(r: &mut Reader) -> Handle<PolygonOnTriangulation> {
    let (deflection, with_parameters, n) = (r.real(), r.flag(), r.integer());
    let polygon = PolygonOnTriangulation::new(n, with_parameters).expect("probe polygon");
    (1..=n).for_each(|i| polygon.set_node(i, r.integer()).unwrap());
    if with_parameters {
        (1..=n).for_each(|i| polygon.set_parameter(i, r.real()).unwrap());
    }
    polygon.set_deflection(deflection);
    Handle::new(polygon)
}

fn triangulation(r: &mut Reader) -> Option<Handle<Triangulation>> {
    let (deflection, uv) = (r.real(), r.flag());
    let (nodes, triangles) = (r.integer(), r.integer());
    let mesh = Triangulation::with_sizes(nodes, triangles, uv, false).expect("probe mesh");
    (1..=nodes).for_each(|i| mesh.set_node(i, r.pnt()).unwrap());
    if uv {
        for i in 1..=nodes {
            let x = r.real();
            mesh.set_uv_node(i, Point2::new(x, r.real())).unwrap();
        }
    }
    for i in 1..=triangles {
        let (a, b) = (r.integer(), r.integer());
        mesh.set_triangle(i, Triangle::new(a, b, r.integer()))
            .unwrap();
    }
    mesh.set_deflection(deflection);
    Some(Handle::new(mesh))
}

pub fn hash_poly2d_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let h = hasher(&mut r, GeomHashPolygon2DHasher::new);
    hash_all(&h, &items(&mut r, poly2d).unwrap());
    end();
}

pub fn hash_poly3d_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let h = hasher(&mut r, GeomHashPolygon3DHasher::new);
    hash_all(&h, &items(&mut r, poly3d).unwrap());
    end();
}

pub fn hash_triangulation_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let h = hasher(&mut r, GeomHashTriangulationHasher::new);
    hash_all(&h, &items(&mut r, triangulation).unwrap());
    end();
}

pub fn hash_ontri_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let h = hasher(&mut r, GeomHashPolygonOnTriHasher::new);
    let n = r.integer();
    let mut keys: Vec<PolygonOnTriHashKey> = Vec::new();
    for _ in 0..n {
        let tri_rep_id = r.real() as u32;
        let poly = match r.integer() {
            0 => on_triangulation(&mut r),
            2 => keys[r.integer() as usize].poly.clone(),
            _ => Handle::default(),
        };
        keys.push(PolygonOnTriHashKey { poly, tri_rep_id });
    }
    hash_all(&h, &keys);
    end();
}
