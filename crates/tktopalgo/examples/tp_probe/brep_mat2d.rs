//! The BRepMAT2d probe operation over faces read by `tp_shape`, the Rust twin of
//! tools/oracle/tktopalgo_probe_brepmat2d.cxx (the request layout is documented there).
use super::bisector::{join, sample, span};
use super::brep_lib_common::type_name;
use super::common::{Reader, attempt, end, flag, int, pnt2d, real};
use super::shape::tp_shape;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dPoint as _;
use rustycad_tkg2d::geom2d::{
    Geom2dCartesianPoint, Geom2dCurve, Geom2dGeometry, Geom2dTrimmedCurve,
};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tktopalgo::brep_mat2d::{
    BRepMAT2dBisectingLocus, BRepMAT2dExplorer, BRepMAT2dLinkTopoBilo,
};
use rustycad_tktopalgo::mat::{MatBasicElt, MatNode, MatSide};

/// `tp_bm2_curve`: type, basis type, range and three points.
fn curve(c: &Handle<dyn Geom2dCurve>) -> Result<(), Failure> {
    type_name(c);
    let trimmed: Handle<Geom2dTrimmedCurve> = c.cast();
    if let Some(t) = trimmed.get() {
        type_name(&t.basis_curve());
    }
    let (a, b) = (c.first_parameter(), c.last_parameter());
    real(a);
    real(b);
    let (a, b) = span(a, b);
    for k in [0, 2, 4] {
        attempt(|| {
            pnt2d(c.value(sample(a, b, k))?);
            Ok(())
        });
    }
    Ok(())
}

/// `tp_bm2_geometry`: a point (0 and its coordinates) or a curve (1 and the curve).
fn geometry(g: &Handle<dyn Geom2dGeometry>) -> Result<(), Failure> {
    let point: Handle<Geom2dCartesianPoint> = g.cast();
    if let Some(p) = point.get() {
        int(0);
        pnt2d(p.pnt2d());
        return Ok(());
    }
    int(1);
    curve(&g.cast())
}

/// `tp_bm2_explorer`: contours, curves, closed flags and the modified shapes.
fn explorer(e: &mut BRepMAT2dExplorer, face: &Shape) -> Result<(), Failure> {
    int(e.number_of_contours());
    for i in 1..=e.number_of_contours() {
        int(e.number_of_curves(i)?);
        flag(*e.get_is_closed().value(i as usize)?);
        e.init(i);
        while e.more()? {
            curve(&e.value()?)?;
            e.next();
        }
    }
    flag(e.is_modified(face));
    let new_face = topods::face(&e.modified_shape(face))?;
    int(new_face.nb_children());
    for w in Explorer::new(face, ShapeEnum::Wire, ShapeEnum::Shape)? {
        flag(e.is_modified(&w));
        int(e.modified_shape(&w).nb_children());
    }
    for s in Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape)? {
        flag(e.is_modified(&s));
        if e.is_modified(&s) {
            modified_edge(&e.modified_shape(&s), &new_face)?;
        }
    }
    Ok(())
}

/// A new edge of the explorer: orientation, pcurve on the new face and its ends, and
/// whether it has a 3d curve.
fn modified_edge(
    new_edge: &Shape,
    new_face: &rustycad_tkbrep::topods::Face,
) -> Result<(), Failure> {
    let edge = topods::edge(new_edge)?;
    int(edge.orientation() as i32);
    let pcurve = BRepTool::curve_on_face(&edge, new_face)?;
    type_name(&pcurve.pcurve);
    real(pcurve.first);
    real(pcurve.last);
    if let Some(c) = pcurve.pcurve.get() {
        for u in [pcurve.first, pcurve.last] {
            attempt(|| {
                pnt2d(c.value(u)?);
                Ok(())
            });
        }
    }
    flag(!BRepTool::curve(&edge)?.0.is_null());
    Ok(())
}

fn index_node(n: &Handle<MatNode>) -> i32 {
    n.get().map_or(-1, MatNode::index)
}
fn index_elt(e: &Handle<MatBasicElt>) -> i32 {
    e.get().map_or(-1, MatBasicElt::index)
}

/// The arcs of `tp_bm2_graph` with GeomBis and its Reverse flag.
fn graph_arcs(b: &BRepMAT2dBisectingLocus) -> Result<(), Failure> {
    let g = b.graph();
    for i in 1..=g.number_of_arcs() {
        attempt(|| {
            let a = g.arc(i)?;
            int(a.index());
            int(a.geom_index());
            int(index_elt(&a.first_element()));
            int(index_elt(&a.second_element()));
            int(index_node(&a.first_node()));
            int(index_node(&a.second_node()));
            attempt(|| {
                let (bis, reverse) = b.geom_bis(&a)?;
                flag(reverse);
                curve(&bis.value().cast())
            });
            Ok(())
        });
    }
    Ok(())
}

/// The nodes and basic elements of `tp_bm2_graph`.
fn graph_nodes_and_elts(b: &BRepMAT2dBisectingLocus) {
    let g = b.graph();
    for i in 1..=g.number_of_nodes() {
        attempt(|| {
            let n = g.node(i)?;
            int(n.index());
            real(n.distance());
            flag(n.infinite());
            attempt(|| {
                pnt2d(b.geom_elt_of_node(&n)?);
                Ok(())
            });
            Ok(())
        });
    }
    for i in 1..=g.number_of_basic_elts() {
        attempt(|| {
            let x = g.basic_elt(i)?;
            int(x.index());
            int(x.geom_index());
            attempt(|| geometry(&b.geom_elt_of_basic_elt(&x)?));
            Ok(())
        });
    }
}

/// `tp_bm2_graph`: the counts, arcs, nodes, basic elements, then each line.
fn graph(b: &BRepMAT2dBisectingLocus, e: &BRepMAT2dExplorer) -> Result<(), Failure> {
    let g = b.graph();
    int(g.number_of_arcs());
    int(g.number_of_nodes());
    int(g.number_of_basic_elts());
    int(g.number_of_infinite_nodes());
    graph_arcs(b)?;
    graph_nodes_and_elts(b);
    for l in 1..=b.number_of_contours() {
        attempt(|| {
            let n = b.number_of_elts(l)?;
            int(n);
            for i in 1..=n {
                attempt(|| {
                    int(index_elt(&b.basic_elt(l, i)?));
                    Ok(())
                });
            }
            Ok(())
        });
        for k in 1..=e.number_of_curves(l)? + 1 {
            attempt(|| {
                int(b.number_of_sections(l, k)?);
                Ok(())
            });
        }
    }
    Ok(())
}

/// `tp_bm2_link`: the basic elements of every edge and vertex and their generating shapes.
fn link(link: &BRepMAT2dLinkTopoBilo, face: &Shape) -> Result<(), Failure> {
    let mut link = link.clone();
    let mut m = IndexedMapOfShape::default();
    topexp::map_shapes(face, &mut m, true, true);
    for i in 1..=m.extent() {
        let s = m.find_key(i)?.clone();
        if !matches!(s.shape_type()?, ShapeEnum::Edge | ShapeEnum::Vertex) {
            continue;
        }
        link.init(&s);
        let mut n = 0;
        while link.more()? {
            let x = link.value()?;
            int(x.index());
            n += 1;
            attempt(|| {
                let gs = link.generating_shape(&x)?;
                int(m.find_index(&gs) as i32);
                int(gs.orientation() as i32);
                Ok(())
            });
            link.next();
        }
        int(n);
    }
    Ok(())
}

/// Sets the Closed flags of the wires of the face, in explorer order.
fn close_wires(r: &mut Reader<'_>, face: &Shape) -> Result<(), Failure> {
    let mut wires = Explorer::new(face, ShapeEnum::Wire, ShapeEnum::Shape)?;
    for _ in 0..r.integer() {
        let closed = r.flag();
        if wires.more() {
            wires.current().set_closed(closed)?;
            wires.next()?;
        }
    }
    Ok(())
}

/// `bm2_locus shape nw (closed)... side join open line link`.
fn locus(r: &mut Reader<'_>) -> Result<(), Failure> {
    let s = tp_shape(r)?;
    close_wires(r, &s.shape)?;
    let side = if r.integer() == 0 {
        MatSide::Left
    } else {
        MatSide::Right
    };
    let join_type = join(r.integer());
    let (open, line, with_link) = (r.flag(), r.integer(), r.flag());
    let mut e = BRepMAT2dExplorer::with_face(&topods::face(&s.shape)?)?;
    explorer(&mut e, &s.shape)?;
    let mut b = BRepMAT2dBisectingLocus::new();
    attempt(|| b.compute(&mut e, line, side, join_type, open));
    flag(b.is_done());
    int(b.number_of_contours());
    if !b.is_done() {
        return Ok(());
    }
    graph(&b, &e)?;
    if with_link {
        attempt(|| {
            let mut l = BRepMAT2dLinkTopoBilo::with_locus(&e, &b)?;
            l.init(&Shape::default());
            link(&l, &s.shape)
        });
    }
    Ok(())
}

/// `bm2_locus`: see the native file.
pub fn locus_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| locus(&mut r));
    end();
}
