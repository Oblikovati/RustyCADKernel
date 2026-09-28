//! TopoDS probe cases, the Rust twin of tools/oracle/tkbrep_probe_topods.cxx.
use super::common::{Reader, end, failure, flag, int, long, masked, text, word};
use super::graph::{Graph, desc, graph};
use rustycad_tkbrep::topods::{
    self, AlertAttribute, AlertWithShape, HShape, Shape, ShapeIterator, TShape,
};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::message::{
    Alert, AlertHandle, Attribute, Gravity, Messenger, PrinterToReport, Report,
};
use rustycad_tkernel::ncollection::HashPolicy;
use rustycad_tkernel::standard::RuntimeTyped;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkmath::toploc::Location;
use std::sync::Arc;

/// The (datum address, power) items of a location: real addresses, or synthetic ones
/// (0x100 per datum id, from 2).
fn items(g: &Graph, l: &Location, real: bool) -> Vec<(usize, i32)> {
    let mut items = Vec::new();
    let mut c = l.clone();
    while !c.is_identity() {
        let datum = c.first_datum().unwrap();
        let address = if real {
            datum.as_ptr() as usize
        } else {
            0x100 * (g.datum_id(datum) + 2) as usize
        };
        items.push((address, c.first_power().unwrap()));
        c = c.next_location().clone();
    }
    items
}

fn hash(g: &Graph, s: &Shape) {
    let location_hash = Location::hash_items(&items(g, s.location(), true));
    let real = Shape::hash_from_parts(s.tshape_address(), location_hash);
    flag(s.hash_code() == real && ShapeMapHasher.hash(s) == real);
    flag(s.location().hash_code() == location_hash);
    let address = if s.is_null() {
        0
    } else {
        0x1000 * (g.tshape_id(s) + 1) as usize
    };
    let synthetic = Location::hash_items(&items(g, s.location(), false));
    long(Shape::hash_from_parts(address, synthetic) as u64);
}

fn flags(s: &Shape) {
    let Some(t) = s.tshape().get() else {
        word("N");
        return int(s.nb_children());
    };
    int(t.shape_type() as i32);
    for f in [
        t.free(),
        t.modified(),
        t.checked(),
        t.orientable(),
        t.closed(),
        t.infinite(),
        t.convex(),
        t.locked(),
    ] {
        flag(f);
    }
    int(s.nb_children());
    int(t.nb_children());
    word(s.tshape().dynamic_type().unwrap().name());
    flag(s.tshape().is_kind(<dyn TShape as RuntimeTyped>::static_type()));
    let copy = t.empty_copy();
    word(copy.dynamic_type().unwrap().name());
    int(copy.nb_children());
    int(copy.shape_type() as i32);
}

fn iterate(g: &Graph, s: &Shape, ori: bool, loc: bool) {
    let mut it = ShapeIterator::new(s, ori, loc);
    let mut n = 0;
    while it.more() && n < 1000 {
        desc(g, it.value().unwrap());
        it.next();
        n += 1;
    }
    word("|");
    if let Err(e) = it.value() {
        failure(&e);
    }
    it.next();
    flag(it.more());
}

fn casts(s: &Shape) {
    let results = [
        topods::vertex(s).map(|_| ()),
        topods::edge(s).map(|_| ()),
        topods::wire(s).map(|_| ()),
        topods::face(s).map(|_| ()),
        topods::shell(s).map(|_| ()),
        topods::solid(s).map(|_| ()),
        topods::comp_solid(s).map(|_| ()),
        topods::compound(s).map(|_| ()),
    ];
    for result in results {
        match result {
            Ok(()) => word("-"),
            Err(e) => failure(&e),
        }
    }
}

/// `br_topods script`.
pub fn topods_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let g = graph(&mut r);
    for s in &g.shapes {
        desc(&g, s);
        flags(s);
        for c in 0..4 {
            iterate(&g, s, c & 1 != 0, c & 2 != 0);
        }
        hash(&g, s);
        casts(s);
    }
    for a in &g.shapes {
        for b in &g.shapes {
            let code = i32::from(a.is_partner(b))
                + 2 * i32::from(a.is_same(b))
                + 4 * i32::from(a.is_equal(b))
                + 8 * i32::from(a != b);
            int(code);
        }
    }
    end();
}

fn dumped(s: &Shape, depth: i32) -> Vec<u8> {
    let mut o = DumpStream::default();
    s.dump_json(&mut o, depth).unwrap();
    masked(o.bytes())
}

fn alerts(s: &Shape) {
    let a = Arc::new(AlertWithShape::new(s));
    let handle: AlertHandle = a.clone();
    flag(a.supports_merge());
    flag(a.merge(&handle));
    flag(a.get_shape().is_equal(s));
    a.set_shape(&Shape::default());
    flag(a.get_shape().is_null());
    word(a.dynamic_type().name());
    let t = AlertAttribute::new(s, &AsciiString::new("probe"));
    flag(t.get_shape().is_equal(s));
    text(t.get_name().as_bytes());
    text(&masked(&t.stream()));
    let mut o = DumpStream::default();
    t.dump_json(&mut o, -1);
    text(&masked(o.bytes()));
    word(Attribute::dynamic_type(&t).name());
}

/// `br_dump script`.
pub fn dump_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let g = graph(&mut r);
    for s in &g.shapes {
        for depth in -1..=2 {
            text(&dumped(s, depth));
        }
        if let Some(t) = s.tshape().get() {
            let mut o = DumpStream::default();
            t.dump_json(&mut o, -1).unwrap();
            text(&masked(o.bytes()));
        }
        let h = HShape::new(s);
        flag(h.shape().is_equal(s));
        h.change_shape(|shape| shape.reverse());
        int(h.shape().orientation() as i32);
        h.set_shape(&Shape::default());
        flag(h.shape().is_null());
        alerts(s);
    }
    let report = Arc::new(Report::new());
    let printer = Arc::new(PrinterToReport::new());
    printer.set_report(Some(report.clone()));
    let messenger = Messenger::with_printer(printer);
    for s in &g.shapes {
        AlertAttribute::send(&messenger, s);
    }
    int(report.get_alerts(Gravity::Info).len() as i32);
    end();
}
