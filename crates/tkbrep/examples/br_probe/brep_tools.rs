//! The text format, the queries and the in-place tools of BRepTools over a scene, the
//! Rust twin of tools/oracle/tkbrep_probe_breptools.cxx.
use super::brep::{Scene, scene};
use super::common::{
    CapturedConsole, Reader, attempt, end, failure, flag, int, masked, out_stream, real, text,
    word, written_bytes,
};
use super::graph::{Graph, desc};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::{self, BRepToolsShapeSet, BRepWriteOptions};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Face, Shape};
use rustycad_tkbrep::toptools::{IndexedMapOfShape, ShapeSet};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::storage::IoStream;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::bnd::BoundingBox2;

/// `bt_written`: `BRepTools::Write` with the options into a string.
pub fn written(sh: &Shape, tri: bool, normals: bool, version: i32) -> Result<Vec<u8>, Failure> {
    let mut o = out_stream(6);
    let options = BRepWriteOptions {
        with_triangles: tri,
        with_normals: normals,
        version,
    };
    brep_tools::write_with_options(sh, &mut o, options, &ProgressRange::default())?;
    Ok(written_bytes(o))
}

/// `bt_reread`: the console output, then `N` or the type and text of the shape read.
fn reread(w: &[u8]) -> Result<(), Failure> {
    let console = CapturedConsole::shared();
    let mut r = Shape::default();
    let mut i = IoStream::from_bytes(w.to_vec());
    let builder = BRepBuilder::default();
    let range = ProgressRange::default();
    brep_tools::read_with_console(&mut r, &mut i, &builder, &range, console.clone())?;
    text(&console.taken());
    if r.is_null() {
        word("N");
        return Ok(());
    }
    int(r.shape_type()? as i32);
    text(&written(&r, true, true, 3)?);
    Ok(())
}

fn text_shape(sh: &Shape) {
    const OPTIONS: [(i32, bool, bool); 5] = [
        (1, true, false),
        (2, true, false),
        (3, true, true),
        (3, false, false),
        (2, false, true),
    ];
    for (version, tri, normals) in OPTIONS {
        attempt(|| {
            let w = written(sh, tri, normals, version)?;
            text(&w);
            reread(&w)
        });
    }
    attempt(|| {
        let mut d = out_stream(6);
        brep_tools::dump(sh, &mut d)?;
        text(&masked(&written_bytes(d)));
        Ok(())
    });
}

/// `bt_whole_set`: one shape set holding every scene shape, then read back.
fn whole_set(s: &Scene, version: i32) {
    let mut set = BRepToolsShapeSet::new(true, true);
    set.set_format_nb(version);
    for sh in &s.graph.shapes {
        attempt(|| set.add(sh).map(|_| ()));
    }
    let mut o = out_stream(6);
    let result = write_whole_set(s, &mut set, &mut o);
    let bytes = written_bytes(o);
    // BR_TRY(write...; br_text(o.str())): nothing is printed before an exception.
    match result {
        Ok(()) => text(&bytes),
        Err(error) => failure(&error),
    }
    attempt(|| {
        let mut d = out_stream(6);
        set.dump(&mut d)?;
        text(&masked(&written_bytes(d)));
        Ok(())
    });
    read_whole_set(s, &bytes);
}

/// The set text, then the reference of each scene shape.
fn write_whole_set(
    s: &Scene,
    set: &mut BRepToolsShapeSet,
    o: &mut IoStream,
) -> Result<(), Failure> {
    set.write(o, &ProgressRange::default())?;
    for sh in &s.graph.shapes {
        set.write_shape(sh, o)?;
    }
    Ok(())
}

/// The reading half of `bt_whole_set`.
fn read_whole_set(s: &Scene, bytes: &[u8]) {
    let console = CapturedConsole::shared();
    let mut set =
        BRepToolsShapeSet::with_console(BRepBuilder::default(), true, false, console.clone());
    let mut i = IoStream::from_bytes(bytes.to_vec());
    let (mut nb, mut nulls) = (-1, Vec::new());
    let result = (|| -> Result<(), Failure> {
        set.read(&mut i, &ProgressRange::default())?;
        nb = set.nb_shapes();
        for _ in &s.graph.shapes {
            nulls.push(set.read_shape(&mut i)?.is_null());
        }
        Ok(())
    })();
    text(&console.taken());
    int(nb);
    nulls.into_iter().for_each(flag);
    if let Err(error) = result {
        failure(&error);
    }
}

/// `br_tools_text script`.
pub fn text_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let s = scene(&mut r);
    for sh in &s.graph.shapes {
        text_shape(sh);
    }
    for version in 1..=3 {
        whole_set(&s, version);
    }
    end();
}

fn uv((a, b, c, d): (f64, f64, f64, f64)) {
    [a, b, c, d].into_iter().for_each(real);
}
fn bnd_box(b: &BoundingBox2) {
    flag(b.is_void());
    attempt(|| {
        let [a, c, d, e] = b.get()?;
        uv((a, c, d, e));
        Ok(())
    });
}

/// `bt_face_queries`.
fn face_queries(g: &Graph, f: &Face) {
    attempt(|| brep_tools::uv_bounds(f).map(uv));
    boxed(|x| brep_tools::add_uv_bounds(f, x));
    attempt(|| brep_tools::outer_wire(f).map(|w| desc(g, &w)));
    attempt(|| {
        let (u, v) = brep_tools::detect_closedness(f)?;
        flag(u);
        flag(v);
        Ok(())
    });
    for w in Explorer::new(f, ShapeEnum::Wire, ShapeEnum::Shape).unwrap() {
        let wire = topods::wire(&w).unwrap();
        attempt(|| brep_tools::uv_bounds_of_wire(f, &wire).map(uv));
        boxed(|x| brep_tools::add_uv_bounds_of_wire(f, &wire, x));
    }
    for e in Explorer::new(f, ShapeEnum::Edge, ShapeEnum::Shape).unwrap() {
        let edge = topods::edge(&e).unwrap();
        attempt(|| brep_tools::uv_bounds_of_edge(f, &edge).map(uv));
        boxed(|x| brep_tools::add_uv_bounds_of_edge(f, &edge, x));
        attempt(|| brep_tools::is_really_closed(&edge, f).map(flag));
        attempt(|| brep_tools::ori_edge_in_face(&edge, f).map(|o| int(o as i32)));
    }
}

/// `Bnd_Box2d x; BR_TRY(add(x); bt_box(x))`.
fn boxed(add: impl FnOnce(&mut BoundingBox2) -> Result<(), Failure>) {
    let mut x = BoundingBox2::default();
    attempt(|| {
        add(&mut x)?;
        bnd_box(&x);
        Ok(())
    });
}

/// `bt_shape_queries`.
fn shape_queries(g: &Graph, sh: &Shape) {
    let mut edges = IndexedMapOfShape::default();
    attempt(|| {
        brep_tools::map_3d_edges(sh, &mut edges)?;
        int(edges.extent() as i32);
        for i in 1..=edges.extent() {
            desc(g, edges.find_key(i)?);
        }
        Ok(())
    });
    let mut bad = Vec::new();
    brep_tools::check_locations(sh, &mut bad);
    int(bad.len() as i32);
    bad.iter().for_each(|b| desc(g, b));
    attempt(|| brep_tools::triangulation(sh, 0.1, false).map(flag));
    attempt(|| brep_tools::triangulation(sh, 1e-3, true).map(flag));
}

/// `br_tools_query script`.
pub fn query_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let s = scene(&mut r);
    let g = &s.graph;
    for sh in g.shapes.iter().filter(|sh| !sh.is_null()) {
        if sh.shape_type().unwrap() == ShapeEnum::Face {
            face_queries(g, &topods::face(sh).unwrap());
        }
        shape_queries(g, sh);
    }
    for a in g.shapes.iter().filter(|sh| !sh.is_null()) {
        for b in g.shapes.iter().filter(|sh| !sh.is_null()) {
            compare_pair(a, b);
        }
    }
    end();
}

/// The Compare of two scene shapes of the same vertex or edge type.
fn compare_pair(a: &Shape, b: &Shape) {
    let kind = a.shape_type().unwrap();
    if kind != b.shape_type().unwrap() {
        return;
    }
    if kind == ShapeEnum::Vertex {
        let (va, vb) = (topods::vertex(a).unwrap(), topods::vertex(b).unwrap());
        attempt(|| brep_tools::compare_vertices(&va, &vb).map(flag));
    }
    if kind == ShapeEnum::Edge {
        let (ea, eb) = (topods::edge(a).unwrap(), topods::edge(b).unwrap());
        flag(brep_tools::compare_edges(&ea, &eb));
    }
}

/// `bt_eval_tolerances`.
fn eval_tolerances(sh: &Shape) -> Result<(), Failure> {
    for f in Explorer::new(sh, ShapeEnum::Face, ShapeEnum::Shape)? {
        let face = topods::face(&f)?;
        for e in Explorer::new(&face, ShapeEnum::Edge, ShapeEnum::Shape)? {
            let edge = topods::edge(&e)?;
            let (c3, first, last) = BRepTool::curve(&edge)?;
            let c2 = BRepTool::curve_on_face(&edge, &face)?.pcurve;
            let sf = BRepTool::surface(&face)?;
            if c3.is_null() || c2.is_null() {
                continue;
            }
            attempt(|| {
                brep_tools::eval_and_update_tol(&edge, &c3, &c2, &sf, first, last).map(real)
            });
        }
    }
    Ok(())
}

/// `bt_triangulations`.
fn triangulations(sh: &Shape) {
    attempt(|| brep_tools::load_all_triangulations(sh, None).map(flag));
    attempt(|| brep_tools::unload_all_triangulations(sh).map(flag));
    attempt(|| brep_tools::load_triangulation(sh, -1, false, None).map(flag));
    attempt(|| brep_tools::unload_triangulation(sh, 0).map(flag));
    attempt(|| brep_tools::activate_triangulation(sh, 1, false).map(flag));
    attempt(|| brep_tools::activate_triangulation(sh, 1, true).map(flag));
    attempt(|| brep_tools::activate_triangulation(sh, 0, true).map(flag));
    attempt(|| brep_tools::load_triangulation(sh, -2, false, None).map(flag));
}

/// `bt_mutate`.
fn mutate(sh: &mut Shape, code: i32) -> Result<(), Failure> {
    match code {
        0 => brep_tools::update(sh),
        1 => brep_tools::clean(sh, false),
        2 => brep_tools::clean(sh, true),
        3 => brep_tools::clean_geometry(sh),
        4 => brep_tools::remove_unused_pcurves(sh),
        5 => brep_tools::remove_internals(sh, false),
        6 => brep_tools::remove_internals(sh, true),
        7 if sh.shape_type()? == ShapeEnum::Face => {
            brep_tools::update_face_uv_points(&topods::face(sh)?)
        }
        7 => Ok(()),
        8 => eval_tolerances(sh),
        _ => {
            triangulations(sh);
            Ok(())
        }
    }
}

/// `br_tools_mutate script code`.
pub fn mutate_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let mut s = scene(&mut r);
    let code = r.integer();
    for sh in s.graph.shapes.iter_mut().filter(|sh| !sh.is_null()) {
        attempt(|| {
            mutate(sh, code)?;
            word("-");
            Ok(())
        });
    }
    for sh in &s.graph.shapes {
        attempt(|| written(sh, true, true, 3).map(|w| text(&w)));
    }
    end();
}
