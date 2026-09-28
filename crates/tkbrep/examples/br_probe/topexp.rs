//! TopExp probe cases, the Rust twin of tools/oracle/tkbrep_probe_topexp.cxx.
use super::common::{Reader, end, flag, int, word};
use super::graph::{Graph, desc, graph};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{Edge, Shape, Wire};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, IndexedMapOfShape, MapOfShape};
use rustycad_tkg3d::top_abs::ShapeEnum;

fn kind(k: i32) -> ShapeEnum {
    ShapeEnum::ALL[k as usize]
}

fn explore(g: &Graph, root: &Shape, find: i32, avoid: i32) {
    let mut e = Explorer::new(root, kind(find), kind(avoid)).unwrap();
    let mut n = 0;
    while e.more() && n < 1000 {
        desc(g, e.current());
        int(e.depth());
        e.next().unwrap();
        n += 1;
    }
    word("|");
    desc(g, e.current());
    int(e.depth());
    e.next().unwrap();
    flag(e.more());
    desc(g, e.value());
    int(e.depth());
    e.re_init().unwrap();
    flag(e.more());
    desc(g, e.current());
    desc(g, e.explored_shape());
    e.clear();
    flag(e.more());
    int(e.depth());
}

/// `br_explore script root`.
pub fn explore_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let g = graph(&mut r);
    let root = g.shape(r.integer());
    for find in 0..=8 {
        for avoid in 0..=8 {
            explore(&g, &root, find, avoid);
        }
    }
    end();
}

fn indexed(g: &Graph, m: &IndexedMapOfShape) {
    int(m.extent() as i32);
    for i in 1..=m.extent() {
        desc(g, &m[i]);
    }
}

fn ancestors(g: &Graph, m: &IndexedDataMapOfShapeListOfShape) {
    int(m.extent() as i32);
    for i in 1..=m.extent() {
        desc(g, m.find_key(i).unwrap());
        int(m[i].extent());
        for a in &m[i] {
            desc(g, a);
        }
    }
}

/// `br_maps script root`.
pub fn maps_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let g = graph(&mut r);
    let root = g.shape(r.integer());
    let mut all = IndexedMapOfShape::default();
    for t in 0..=8 {
        let mut m = IndexedMapOfShape::default();
        topexp::map_shapes_of_type(&root, kind(t), &mut m).unwrap();
        indexed(&g, &m);
        topexp::map_shapes_of_type(&root, kind(t), &mut all).unwrap();
    }
    indexed(&g, &all);
    for c in 0..4 {
        let mut m = IndexedMapOfShape::default();
        topexp::map_shapes(&root, &mut m, c & 1 != 0, c & 2 != 0);
        indexed(&g, &m);
        let mut plain = MapOfShape::default();
        topexp::map_shapes_into_map(&root, &mut plain, c & 1 != 0, c & 2 != 0);
        int(plain.extent() as i32);
        for s in &g.shapes {
            flag(plain.contains(s));
        }
    }
    for ts in 0..=7 {
        for ta in 0..=7 {
            let (mut m, mut u, mut uo) = Default::default();
            topexp::map_shapes_and_ancestors(&root, kind(ts), kind(ta), &mut m).unwrap();
            ancestors(&g, &m);
            topexp::map_shapes_and_unique_ancestors(&root, kind(ts), kind(ta), &mut u, false)
                .unwrap();
            ancestors(&g, &u);
            topexp::map_shapes_and_unique_ancestors(&root, kind(ts), kind(ta), &mut uo, true)
                .unwrap();
            ancestors(&g, &uo);
        }
    }
    end();
}

/// `br_vertices script`.
pub fn vertices_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let g = graph(&mut r);
    for s in g.shapes.iter().filter(|s| !s.is_null()) {
        let e = Edge::unchecked(s.clone());
        for cum in [false, true] {
            desc(&g, &topexp::first_vertex(&e, cum));
            desc(&g, &topexp::last_vertex(&e, cum));
            let (f, l) = topexp::vertices(&e, cum);
            desc(&g, &f);
            desc(&g, &l);
        }
        for t in g.shapes.iter().filter(|t| !t.is_null()) {
            let found = topexp::common_vertex(&e, &Edge::unchecked(t.clone()));
            flag(found.is_some());
            desc(&g, &found.map(Shape::from).unwrap_or_default());
        }
        if s.shape_type().unwrap() == ShapeEnum::Wire {
            let (f, l) = topexp::wire_vertices(&Wire::unchecked(s.clone()));
            desc(&g, &f);
            desc(&g, &l);
        }
    }
    end();
}
