//! BinTools probe cases, part 3: the Rust twin of
//! tools/oracle/tkbrep_probe_bintools_shape.cxx (scenes, given bytes, truncations).
use super::bintools::{attempt, bytes_token, failure, failure_parts, out_stream, read_bytes};
use super::bintools_sets::read_geometry_set;
use super::brep::{Geometry, Scene, read_geometry};
use super::common::{CapturedConsole, Reader, end, int, real, text, word, written_bytes};
use super::graph::{self, Graph};
use rustycad_tkbrep::bintools::{
    self, BinCurve2dSet, BinCurveSet, BinLocationSet, BinShapeReader, BinShapeSet, BinShapeSetBase,
    BinShapeWriter, BinSurfaceSet, WriteOptions, format_version,
};
use rustycad_tkbrep::topods::{Shape, ShapeIterator};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::ncollection::{Array1, HArray1OfReal};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::storage::IoStream;
use rustycad_tkmath::gp::Point2;
use rustycad_tkmath::poly::Polygon3D;
use rustycad_tkmath::toploc::Location;

/// `bt_tweak`.
fn tweak(g: &mut Geometry, r: &mut Reader<'_>) -> Result<(), Failure> {
    let (code, i) = (r.integer(), r.integer() as usize);
    if code == 1 {
        return g.triangulations[i].compute_normals();
    }
    if code == 2 {
        let t = &g.triangulations[i];
        t.add_uv_nodes()?;
        for k in 1..=t.nb_nodes() {
            let p = t.node(k)?;
            t.set_uv_node(k, Point2::new(p.x(), p.y()))?;
        }
        return Ok(());
    }
    let d = r.real();
    match code {
        3 => g.triangulations[i].set_deflection(d),
        4 => {
            let old = g.polygons3d[i].clone();
            let params: Vec<f64> = (1..=old.nb_nodes()).map(|k| f64::from(k) * d).collect();
            let p = Polygon3D::from_nodes_and_parameters(
                &old.nodes(),
                &Array1::from_shared(&params, 1),
            )?;
            p.set_deflection(d);
            g.polygons3d[i] = Handle::new(p);
        }
        _ => {
            let n = &g.nodes[i];
            n.set_deflection(d);
            let params = HArray1OfReal::with_bounds(1, n.nb_nodes())?;
            for k in 1..=n.nb_nodes() {
                params.set_value(k, f64::from(k) * d)?;
            }
            n.set_parameters(Handle::new(params))?;
        }
    }
    Ok(())
}

/// `bt_scene`.
fn scene(r: &mut Reader<'_>) -> Scene {
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
        match tweak(&mut s.geometry, r) {
            Ok(()) => word("-"),
            Err(e) => word(e.exception_type()),
        }
    }
    for _ in 0..r.integer() {
        super::brep_commands::command(&mut s, r);
    }
    s
}

/// `bt_walk`.
fn walk_with(s: &Shape, ids: &mut Vec<usize>) {
    if s.is_null() {
        word("N");
        return;
    }
    let address = s.tshape_address();
    let id = ids.iter().position(|a| *a == address).unwrap_or_else(|| {
        ids.push(address);
        ids.len() - 1
    });
    int(s.shape_type().unwrap() as i32);
    int(s.orientation() as i32);
    int(id as i32);
    let flags = [
        s.free(),
        s.modified(),
        s.checked(),
        s.orientable(),
        s.closed(),
        s.infinite(),
        s.convex(),
        s.locked(),
    ];
    int(flags.iter().enumerate().fold(0, |bits, (bit, f)| {
        bits | (i32::from(*f.as_ref().unwrap()) << bit)
    }));
    let mut n = 0;
    let mut l = s.location().clone();
    while !l.is_identity() {
        n += 1;
        l = l.next_location().clone();
    }
    int(n);
    if n > 0 {
        let t = s.location().transformation();
        for i in 1..=3 {
            for j in 1..=4 {
                real(t.value(i, j).unwrap());
            }
        }
    }
    int(s.nb_children());
    let mut it = ShapeIterator::new(s, false, false);
    while it.more() {
        walk_with(it.value().unwrap(), ids);
        it.next();
    }
}
fn walk(s: &Shape) {
    walk_with(s, &mut Vec::new());
}

fn options(wt: bool, wn: bool, version: i32) -> WriteOptions {
    WriteOptions {
        with_triangles: wt,
        with_normals: wn,
        version,
    }
}

/// `bt_write_set`.
fn write_set(s: &Shape, wt: bool, wn: bool, version: i32) -> Vec<u8> {
    let mut o = out_stream();
    attempt(|| {
        bintools::write(
            s,
            &mut o,
            options(wt, wn, version),
            &ProgressRange::default(),
        )
    });
    written_bytes(o)
}

/// `bt_read_set`.
fn read_set(bytes: Vec<u8>) -> Shape {
    let mut is = IoStream::from_bytes(bytes);
    let mut s = Shape::default();
    let console = CapturedConsole::shared();
    let result =
        bintools::read_with_console(&mut s, &mut is, &ProgressRange::default(), console.clone());
    text(&console.taken());
    if let Err(e) = result {
        failure(&e);
    }
    int(i32::from(is.rdstate()));
    walk(&s);
    s
}

/// `bt_write_refs`.
fn write_refs(shapes: &[Shape], wt: bool, wn: bool) -> Vec<u8> {
    let mut w = BinShapeWriter::new();
    w.set_with_triangles(wt);
    w.set_with_normals(wn);
    let mut o = out_stream();
    for s in shapes {
        attempt(|| w.write_shape(s, &mut o));
    }
    written_bytes(o)
}

/// `bt_read_refs`.
fn read_refs(bytes: Vec<u8>, count: usize, skip: i64) -> Vec<Shape> {
    let mut rd = BinShapeReader::new();
    let mut is = IoStream::from_bytes(bytes);
    is.seekg(skip);
    let mut shapes = Vec::new();
    for _ in 0..count {
        let mut s = Shape::default();
        attempt(|| rd.read_shape(&mut is, &mut s));
        int(i32::from(is.rdstate()));
        walk(&s);
        shapes.push(s);
    }
    shapes
}

/// `br_bin_shape`.
pub fn shape_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (wt, wn, version) = (r.flag(), r.flag(), r.integer());
    let sc = scene(&mut r);
    let root = sc.graph.shape(r.integer());
    let bytes = write_set(&root, wt, wn, version);
    bytes_token(&bytes);
    let back = read_set(bytes);
    bytes_token(&write_set(&back, wt, wn, version));
    let refs = write_refs(&[root.clone(), root.clone()], wt, wn);
    bytes_token(&refs);
    bytes_token(&write_refs(&read_refs(refs, 2, 0), wt, wn));
    let mut set = BinShapeSet::new();
    set.set_with_triangles(wt);
    set.set_with_normals(wn);
    set.set_format_nb(version);
    int(set.format_nb());
    attempt(|| set.add(&root).map(int));
    int(set.nb_shapes());
    int(set.locations().nb_locations());
    for s in &sc.graph.shapes {
        int(set.index(s));
    }
    end();
}

/// `br_bin_read`.
pub fn read_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let reader = r.integer();
    let bytes = read_bytes(&mut r);
    match reader {
        0 => {
            let s = read_set(bytes);
            bytes_token(&write_set(&s, true, false, format_version::CURRENT));
        }
        1 => {
            let (skip, count) = (i64::from(r.integer()), r.integer() as usize);
            bytes_token(&write_refs(&read_refs(bytes, count, skip), true, false));
        }
        2 => read_geometry_set::<BinCurveSet>(bytes),
        3 => read_geometry_set::<BinCurve2dSet>(bytes),
        4 => read_geometry_set::<BinSurfaceSet>(bytes),
        5 => read_location_set(bytes),
        _ => read_shape_set(bytes),
    }
    end();
}

fn read_location_set(bytes: Vec<u8>) {
    let mut set = BinLocationSet::new();
    let mut is = IoStream::from_bytes(bytes);
    attempt(|| set.read(&mut is));
    int(i32::from(is.rdstate()));
    int(set.nb_locations());
    let mut o = out_stream();
    attempt(|| set.write(&mut o));
    bytes_token(&written_bytes(o));
}

fn read_shape_set(bytes: Vec<u8>) {
    let console = CapturedConsole::shared();
    let mut set = BinShapeSet::with_console(console.clone());
    let mut is = IoStream::from_bytes(bytes);
    let result = set.read(&mut is, &ProgressRange::default());
    text(&console.taken());
    if let Err(e) = result {
        failure(&e);
    }
    int(i32::from(is.rdstate()));
    int(set.format_nb());
    int(set.nb_shapes());
    let mut o = out_stream();
    attempt(|| set.write(&mut o, &ProgressRange::default()));
    bytes_token(&written_bytes(o));
}

/// `bt_undefined_cut`.
fn undefined_cut(bytes: &[u8], cut: usize) -> bool {
    for tag in [&b"Locations "[..], b"Curve2ds ", b"Curves ", b"Surfaces "] {
        let Some(at) = bytes.windows(tag.len()).position(|w| w == tag) else {
            continue;
        };
        let word_end = at + tag.len() - 1;
        if cut >= word_end && cut <= word_end + 1 {
            return true;
        }
    }
    false
}

/// `br_bin_truncate`.
pub fn truncate_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (wt, wn, version, step) = (r.flag(), r.flag(), r.integer(), r.integer() as usize);
    let sc = scene(&mut r);
    let root = sc.graph.shape(r.integer());
    let bytes = write_set(&root, wt, wn, version);
    int(bytes.len() as i32);
    let mut cut = 0;
    while cut < bytes.len() {
        if !undefined_cut(&bytes, cut) {
            read_prefix(&bytes[..cut]);
        }
        cut += step;
    }
    end();
}

fn read_prefix(prefix: &[u8]) {
    let mut is = IoStream::from_bytes(prefix.to_vec());
    let mut s = Shape::default();
    let console = CapturedConsole::shared();
    match bintools::read_with_console(&mut s, &mut is, &ProgressRange::default(), console.clone()) {
        Ok(()) => int(if s.is_null() {
            -1
        } else {
            s.shape_type().unwrap() as i32
        }),
        Err(e) => failure_parts(e.exception_type(), e.message()),
    }
    text(&console.taken());
    int(i32::from(is.rdstate()));
}
