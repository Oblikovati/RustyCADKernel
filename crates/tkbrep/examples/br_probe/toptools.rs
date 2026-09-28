//! TopTools probe cases, the Rust twin of tools/oracle/tkbrep_probe_toptools.cxx.
use super::common::{
    CapturedConsole, Reader, end, failure, flag, int, masked, out_stream, real, rest, text, word,
    written_bytes,
};
use super::graph::{Graph, chain, desc, graph, make};
use rustycad_tkbrep::topods::{Builder, Shape, ShapeIterator};
use rustycad_tkbrep::toptools::{LocationSet, ShapeSet, ShapeSetData, TopToolsShapeSet, shape_set_io};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::storage::IoStream;
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::toploc::Location;
use std::sync::Arc;

/// `std::string w; IS >> w`: a word of any length.
const ANY_WORD: usize = 1 << 20;

/// `ProbeShapeSet`: hooks that make TopoDS-only shapes and trace their calls.
struct ProbeShapeSet {
    data: ShapeSetData,
    added: i32,
    checked: i32,
}
impl ProbeShapeSet {
    fn new(console: Arc<CapturedConsole>) -> Self {
        Self {
            data: ShapeSetData::with_console(console),
            added: 0,
            checked: 0,
        }
    }
}
impl ShapeSet for ProbeShapeSet {
    fn shape_set_data(&self) -> &ShapeSetData {
        &self.data
    }
    fn shape_set_data_mut(&mut self) -> &mut ShapeSetData {
        &mut self.data
    }
    fn clear(&mut self) {
        shape_set_io::clear(&mut self.data);
        self.added = 0;
        self.checked = 0;
    }
    fn add_geometry(&mut self, shape: &Shape) -> Result<(), Failure> {
        self.added += 1 + shape.nb_children();
        Ok(())
    }
    fn dump_geometry(&self, stream: &mut IoStream) -> Result<(), Failure> {
        stream.insert_bytes(format!("geometry {}\n", self.added).as_bytes());
        Ok(())
    }
    fn write_geometry(&mut self, stream: &mut IoStream, _: &ProgressRange<'_>) -> Result<(), Failure> {
        stream.insert_bytes(format!("Probe {}\n", self.added).as_bytes());
        Ok(())
    }
    fn read_geometry(&mut self, stream: &mut IoStream, _: &ProgressRange<'_>) -> Result<(), Failure> {
        stream.extract_word(ANY_WORD);
        stream.extract_int(&mut self.added);
        Ok(())
    }
    fn dump_shape_geometry(&self, shape: &Shape, stream: &mut IoStream) -> Result<(), Failure> {
        stream.insert_bytes(format!("  children {}\n", shape.nb_children()).as_bytes());
        Ok(())
    }
    fn write_shape_geometry(&self, shape: &Shape, stream: &mut IoStream) -> Result<(), Failure> {
        stream.insert_bytes(format!("g{}", shape.nb_children()).as_bytes());
        Ok(())
    }
    fn read_shape_geometry(&mut self, kind: ShapeEnum, stream: &mut IoStream, shape: &mut Shape) -> Result<(), Failure> {
        stream.extract_word(ANY_WORD);
        *shape = make(kind as i32);
        Ok(())
    }
    fn add_shapes(&mut self, shape: &mut Shape, child: &Shape) -> Result<(), Failure> {
        Builder.add(shape, child)
    }
    fn check(&mut self, _: ShapeEnum, _: &mut Shape) -> Result<(), Failure> {
        self.checked += 1;
        Ok(())
    }
}

fn set_desc(set: &dyn ShapeSet, s: &Shape) {
    if s.is_null() {
        return word("N");
    }
    int(set.index(&s.located(&Location::default(), false).unwrap()));
    int(s.orientation() as i32);
    int(set.locations().index(s.location()));
}

fn set_shapes(set: &dyn ShapeSet) {
    int(set.nb_shapes());
    for i in 1..=set.nb_shapes() {
        let s = set.shape(i).unwrap();
        int(s.shape_type().unwrap() as i32);
        for f in [s.free(), s.modified(), s.checked(), s.orientable(), s.closed(), s.infinite(), s.convex(), s.locked()] {
            flag(f.unwrap());
        }
        let mut it = ShapeIterator::new(&s, false, false);
        while let Ok(child) = it.value() {
            set_desc(set, child);
            it.next();
        }
        word("|");
    }
}

fn set_written(set: &mut dyn ShapeSet, precision: usize) -> Vec<u8> {
    let mut o = out_stream(precision);
    if let Err(e) = set.write(&mut o, &ProgressRange::default()) {
        failure(&e);
    }
    int(o.precision() as i32);
    written_bytes(o)
}

fn shape_records(g: &Graph, set: &ProbeShapeSet) {
    for s in &g.shapes {
        let (mut d, mut w) = (out_stream(6), out_stream(6));
        set.dump_shape(s, &mut d).unwrap();
        set.write_shape(s, &mut w).unwrap();
        let written = written_bytes(w);
        text(&written_bytes(d));
        text(&written);
        let mut back = IoStream::from_bytes(written);
        match set.read_shape(&mut back) {
            Ok(read) => desc(g, &read),
            Err(e) => {
                failure(&e);
                desc(g, &Shape::default());
            }
        }
    }
}

/// `br_shapeset format script root`.
pub fn shapeset_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let format = r.integer();
    let g = graph(&mut r);
    let root = g.shape(r.integer());
    let mut set = ProbeShapeSet::new(CapturedConsole::shared());
    set.set_format_nb(format);
    int(set.format_nb());
    int(set.add(&root).unwrap());
    for s in &g.shapes {
        int(set.add(s).unwrap());
    }
    int(set.nb_shapes());
    int(set.added);
    for s in &g.shapes {
        int(set.index(s));
        int(set.locations().index(s.location()));
    }
    for i in 0..=set.nb_shapes() + 1 {
        match set.shape(i) {
            Ok(s) => desc(&g, &s),
            Err(e) => failure(&e),
        }
    }
    for i in -1..=g.locs.len() as i32 + 2 {
        chain(&g, &set.locations().location(i));
    }
    let mut o = out_stream(6);
    set.dump_extent(&mut o).unwrap();
    text(&written_bytes(o));
    let mut t = AsciiString::new("x");
    set.dump_extent_string(&mut t).unwrap();
    text(t.as_bytes());
    let mut o = out_stream(6);
    if let Err(e) = set.dump(&mut o) {
        failure(&e);
    }
    text(&masked(&written_bytes(o)));
    shape_records(&g, &set);
    read_back(&mut set);
    end();
}

fn read_back(set: &mut ProbeShapeSet) {
    let written = set_written(set, 6);
    text(&written);
    let console = CapturedConsole::shared();
    let mut back = ProbeShapeSet::new(console.clone());
    let mut input = IoStream::from_bytes(written);
    let result = back.read(&mut input, &ProgressRange::default());
    let mut printed = console.taken();
    if let Err(e) = result {
        printed.extend_from_slice(format!("{} ", e.exception_type()).as_bytes());
    }
    text(&printed);
    int(i32::from(input.rdstate()));
    int(back.format_nb());
    int(back.added);
    int(back.checked);
    set_shapes(&back);
    text(&set_written(&mut back, 3));
    back.clear();
    int(back.nb_shapes());
    int(back.added);
}

/// `br_setread base format bytes`.
pub fn setread_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let base = r.flag();
    let format = r.integer();
    let mut input = IoStream::from_bytes(r.bytes());
    let console = CapturedConsole::shared();
    let mut probe = ProbeShapeSet::new(console.clone());
    let mut plain = TopToolsShapeSet::with_console(console.clone());
    let set: &mut dyn ShapeSet = if base { &mut plain } else { &mut probe };
    set.set_format_nb(format);
    let result = set.read(&mut input, &ProgressRange::default());
    let mut printed = console.taken();
    if let Err(e) = result {
        printed.extend_from_slice(format!("{} ", e.exception_type()).as_bytes());
    }
    text(&printed);
    int(i32::from(input.rdstate()));
    int(set.format_nb());
    let set: &dyn ShapeSet = if base { &plain } else { &probe };
    int(probe.checked);
    set_shapes(set);
    let mut o = out_stream(6);
    set.dump_extent(&mut o).unwrap();
    text(&written_bytes(o));
    text(&rest(&mut input));
    end();
}

fn locations(set: &LocationSet, count: i32) {
    for i in -1..=count + 1 {
        let l = set.location(i);
        int(set.index(&l));
        let t = l.transformation();
        for row in 1..=3 {
            for col in 1..=4 {
                real(t.value(row, col).unwrap());
            }
        }
        let mut items = Vec::new();
        let mut c = l.clone();
        while !c.is_identity() {
            let datum = Location::from_datum(c.first_datum().unwrap().clone());
            items.push((set.index(&datum), c.first_power().unwrap()));
            c = c.next_location().clone();
        }
        int(items.len() as i32);
        for (d, p) in items {
            int(d);
            int(p);
        }
    }
}

fn locations_written(set: &LocationSet, precision: usize) -> Vec<u8> {
    let mut o = out_stream(precision);
    set.write(&mut o, &ProgressRange::default());
    int(o.precision() as i32);
    written_bytes(o)
}

fn count(set: &LocationSet) -> i32 {
    let mut count = 0;
    while !set.location(count + 1).is_identity() {
        count += 1;
    }
    count
}

/// `br_locset script`.
pub fn locset_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let g = graph(&mut r);
    let mut set = LocationSet::new();
    for l in &g.locs {
        int(set.add(l));
    }
    for l in &g.locs {
        int(set.index(l));
    }
    let n = count(&set);
    int(n);
    locations(&set, n);
    let mut o = out_stream(6);
    set.dump(&mut o);
    text(&written_bytes(o));
    let written = locations_written(&set, 6);
    text(&written);
    let mut back = LocationSet::new();
    let mut input = IoStream::from_bytes(written);
    if let Err(e) = back.read(&mut input, &ProgressRange::default()) {
        failure(&e);
    }
    int(i32::from(input.rdstate()));
    locations(&back, n);
    text(&locations_written(&back, 4));
    back.clear();
    flag(back.location(1).is_identity());
    end();
}

/// `br_locread bytes`.
pub fn locread_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let mut input = IoStream::from_bytes(r.bytes());
    let console = CapturedConsole::shared();
    let mut set = LocationSet::with_console(console.clone());
    let result = set.read(&mut input, &ProgressRange::default());
    let mut printed = console.taken();
    if let Err(e) = result {
        printed.extend_from_slice(format!("{} ", e.exception_type()).as_bytes());
    }
    text(&printed);
    int(i32::from(input.rdstate()));
    let n = count(&set);
    int(n);
    locations(&set, n);
    let mut o = out_stream(6);
    set.dump(&mut o);
    text(&written_bytes(o));
    end();
}
