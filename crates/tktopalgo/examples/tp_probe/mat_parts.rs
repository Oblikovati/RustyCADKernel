//! MAT and MAT2d probe operations on the classes one by one, the Rust twin of
//! tools/oracle/tktopalgo_probe_mat_parts.cxx.
use super::common::{Reader, attempt, captured, end, flag, int, real, text};
use super::curve2d::curve2d;
use super::mat::{connexion, read_figure};
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkmath::gp::Point2;
use rustycad_tktopalgo::mat::{
    MatArc, MatBasicElt, MatBisector, MatEdge, MatGraph, MatListOfEdge, MatNode, MatSide, MatZone,
};
use rustycad_tktopalgo::mat2d::{
    Mat2dBiInt, Mat2dConnexion, Mat2dCutCurve, Mat2dMat2d, Mat2dMiniPath, Mat2dTool2d,
};

fn type_name<T: ?Sized + RuntimeTyped>(h: &Handle<T>) {
    text(h.dynamic_type().map_or("", |t| t.name()).as_bytes());
}
fn edge_number(e: &Handle<MatEdge>) -> i32 {
    e.get().map_or(-1, MatEdge::edge_number)
}
fn new_edge(number: i32) -> Handle<MatEdge> {
    let e = Handle::new(MatEdge::new());
    e.set_edge_number(number);
    e
}
fn index_arc(a: &Handle<MatArc>) -> i32 {
    a.get().map_or(-1, MatArc::index)
}
fn index_node(n: &Handle<MatNode>) -> i32 {
    n.get().map_or(-1, MatNode::index)
}
fn index_elt(e: &Handle<MatBasicElt>) -> i32 {
    e.get().map_or(-1, MatBasicElt::index)
}

fn list_state(l: &MatListOfEdge, looped: bool) -> Result<(), Failure> {
    int(l.number());
    int(l.index());
    flag(l.more());
    flag(l.is_empty());
    if l.more() {
        int(edge_number(&l.current()?));
    }
    if !l.is_empty() {
        int(edge_number(&l.first_item()?));
        int(edge_number(&l.last_item()?));
    }
    if looped && l.more() {
        int(edge_number(&l.previous_item()?));
        int(edge_number(&l.next_item()?));
    }
    Ok(())
}

fn list_op(
    l: &MatListOfEdge,
    edges: &[Handle<MatEdge>],
    (op, arg): (i32, i32),
) -> Result<(), Failure> {
    match op {
        0 => l.first(),
        1 => l.last(),
        2 => l.next()?,
        3 => l.previous()?,
        4 => int(edge_number(&l.brackets(arg)?)),
        5 => l.unlink()?,
        6 => l.link_before(&new_edge(arg))?,
        7 => l.link_after(&new_edge(arg))?,
        8 => l.front_add(&new_edge(arg)),
        9 => l.permute()?,
        10 => l.close_loop()?,
        11 => l.init(&edges[(arg - 1) as usize])?,
        12 => l.set_current(&new_edge(arg))?,
        _ => l.back_add(&new_edge(arg)),
    }
    Ok(())
}

/// `n (op arg)*`: the list of edges and its edits.
pub fn mat_list_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let n = r.integer();
        let l: Handle<MatListOfEdge> = Handle::new(MatListOfEdge::new());
        let edges: Vec<Handle<MatEdge>> = (1..=n).map(new_edge).collect();
        edges.iter().for_each(|e| l.back_add(e));
        let mut looped = false;
        list_state(&l, looped)?;
        let ops = r.integer();
        for _ in 0..ops {
            let (op, arg) = (r.integer(), r.integer());
            attempt(|| list_op(&l, &edges, (op, arg)));
            looped |= op == 10;
            list_state(&l, looped)?;
        }
        type_name(&l);
        Ok(())
    });
    end();
}

fn bisector_with_edges(first: i32, second: i32, number: i32) -> Handle<MatBisector> {
    let b = Handle::new(MatBisector::new());
    b.set_first_edge(new_edge(first));
    b.set_second_edge(new_edge(second));
    b.set_bisector_number(number);
    b
}

/// `shift level`: MAT_Bisector, its sons and its Dump.
pub fn mat_bisector_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let (shift, level) = (r.integer(), r.integer());
        let b = bisector_with_edges(1, 2, 7);
        for i in 0..2 {
            b.add_bisector(&bisector_with_edges(3 + i, 5 + i, 8 + i));
        }
        int(b.bisector_number());
        int(b.index_number());
        real(b.first_parameter());
        real(b.second_parameter());
        real(b.sense());
        int(b.first_bisector()?.bisector_number());
        int(b.last_bisector()?.bisector_number());
        captured(|out| b.dump(shift, level, out));
        b.list().first();
        captured(|out| b.dump(shift, level, out));
        captured(|out| b.list().dump(shift, level, out));
        type_name(&b);
        type_name(&b.list());
        Ok(())
    });
    end();
}

struct Piece {
    e1: Handle<MatBasicElt>,
    e2: Handle<MatBasicElt>,
    a1: Handle<MatArc>,
    a2: Handle<MatArc>,
    a3: Handle<MatArc>,
    n: Handle<MatNode>,
    m: Handle<MatNode>,
    o: Handle<MatNode>,
}

fn build_piece() -> Result<Piece, Failure> {
    let e1 = Handle::new(MatBasicElt::new(1));
    let e2 = Handle::new(MatBasicElt::new(2));
    let e3 = Handle::new(MatBasicElt::new(3));
    let a1 = Handle::new(MatArc::new(1, 11, &e1, &e2));
    let a2 = Handle::new(MatArc::new(2, 12, &e2, &e3));
    let a3 = Handle::new(MatArc::new(3, 13, &e3, &e1));
    let n = Handle::new(MatNode::new(5, &a1, 2.5));
    let m = Handle::new(MatNode::new(6, &a1, 0.0));
    let o = Handle::new(MatNode::new(7, &a2, precision::infinite()));
    a1.set_first_node(&n);
    a2.set_first_node(&n);
    a3.set_first_node(&n);
    a1.set_second_node(&m);
    a2.set_second_node(&o);
    a1.set_neighbour(MatSide::Left, &n, &a2)?;
    a2.set_neighbour(MatSide::Left, &n, &a3)?;
    a3.set_neighbour(MatSide::Left, &n, &a1)?;
    a1.set_neighbour(MatSide::Right, &n, &a3)?;
    a2.set_first_arc(MatSide::Right, &a1);
    a3.set_second_arc(MatSide::Left, &a2);
    e1.set_start_arc(&a1);
    e1.set_end_arc(&a3);
    e1.set_geom_index(4);
    e1.set_index(9);
    n.set_index(3);
    a3.set_geom_index(14);
    a3.set_index(4);
    Ok(Piece {
        e1,
        e2,
        a1,
        a2,
        a3,
        n,
        m,
        o,
    })
}

fn piece_links(p: &Piece) -> Result<(), Failure> {
    int(p.e1.index());
    int(p.e1.geom_index());
    int(index_arc(&p.e1.start_arc()));
    int(index_arc(&p.e1.end_arc()));
    int(p.a3.index());
    int(p.a3.geom_index());
    int(index_elt(&p.a1.first_element()));
    int(index_elt(&p.a1.second_element()));
    int(index_node(&p.a1.the_other_node(&p.n)?));
    int(index_node(&p.a1.the_other_node(&p.m)?));
    attempt(|| p.a1.the_other_node(&p.o).map(|_| ()));
    for x in [&p.n, &p.m, &p.o] {
        for s in [MatSide::Left, MatSide::Right] {
            attempt(|| {
                flag(p.a1.has_neighbour(x, s)?);
                Ok(())
            });
            attempt(|| {
                int(index_arc(&p.a1.neighbour(x, s)?));
                Ok(())
            });
        }
    }
    attempt(|| p.a1.set_neighbour(MatSide::Left, &p.o, &p.a2));
    Ok(())
}

fn piece_nodes(p: &Piece) -> Result<(), Failure> {
    int(p.n.index());
    int(p.n.geom_index());
    real(p.n.distance());
    flag(p.n.pending_node()?);
    flag(p.m.pending_node()?);
    flag(p.n.on_basic_elt());
    flag(p.m.on_basic_elt());
    flag(p.o.infinite());
    let mut arcs: Sequence<Handle<MatArc>> = Sequence::default();
    p.n.linked_arcs(&mut arcs)?;
    int(arcs.length());
    arcs.iter().for_each(|a| int(index_arc(a)));
    let mut elts: Sequence<Handle<MatBasicElt>> = Sequence::default();
    p.n.near_elts(&mut elts)?;
    int(elts.length());
    elts.iter().for_each(|e| int(index_elt(e)));
    p.m.set_linked_arc(&p.a2);
    attempt(|| {
        flag(p.m.pending_node()?);
        Ok(())
    });
    Ok(())
}

/// `-`: a hand-made piece of MAT graph.
pub fn mat_arc_case(values: &[f64]) {
    let _ = values;
    attempt(|| {
        let p = build_piece()?;
        piece_links(&p)?;
        piece_nodes(&p)?;
        let z = Handle::new(MatZone::new());
        flag(z.no_empty_zone());
        flag(z.limited());
        attempt(|| z.arc_on_frontier(1).map(|_| ()));
        let z2 = MatZone::with_basic_elt(&p.e2)?;
        int(z2.number_of_arcs());
        flag(z2.limited());
        let g = Handle::new(MatGraph::new());
        int(g.number_of_arcs());
        attempt(|| g.arc(1).map(|_| ()));
        attempt(|| g.node(1).map(|_| ()));
        attempt(|| g.basic_elt(1).map(|_| ()));
        type_name(&p.a1);
        type_name(&p.n);
        type_name(&p.e1);
        type_name(&z);
        type_name(&g);
        Ok(())
    });
    end();
}

fn read_connexion(r: &mut Reader<'_>) -> Mat2dConnexion {
    let (l1, l2, i1, i2) = (r.integer(), r.integer(), r.integer(), r.integer());
    let (d, u1, u2) = (r.real(), r.real(), r.real());
    let (p1, p2) = (r.pnt2d(), r.pnt2d());
    Mat2dConnexion::with_values(l1, l2, i1, i2, d, u1, u2, p1, p2)
}

/// Two connexions and a sense: MAT2d_Connexion and MAT2d_BiInt.
pub fn mat2d_connexion_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let a = Handle::new(read_connexion(&mut r));
        let b = read_connexion(&mut r);
        let sense = r.real();
        flag(a.is_after(&b, sense));
        flag(b.is_after(&a, sense));
        flag(a.is_after(&a, sense));
        connexion(&a.reverse());
        captured(|out| {
            a.dump(0, 1, out);
            Ok(())
        });
        let e = Mat2dConnexion::new();
        connexion(&e);
        e.set_index_first_line(3);
        e.set_index_second_line(4);
        e.set_index_item_on_first(5);
        e.set_index_item_on_second(6);
        e.set_parameter_on_first(0.5);
        e.set_parameter_on_second(0.25);
        e.set_point_on_first(Point2::new(1., 2.));
        e.set_point_on_second(Point2::new(3., 4.));
        e.set_distance(7.);
        connexion(&e);
        let mut k = Mat2dBiInt::new(1, 2);
        k.set_first_index(3);
        k.set_second_index(4);
        int(k.first_index());
        int(k.second_index());
        flag(k.is_equal(&Mat2dBiInt::new(3, 4)));
        flag(k == Mat2dBiInt::new(4, 3));
        type_name(&a);
        Ok(())
    });
    end();
}

/// `curve`: MAT2d_CutCurve.
pub fn mat2d_cut_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let c: Handle<dyn Geom2dCurve> = curve2d(&mut r)?;
        let cut = Mat2dCutCurve::with_curve(&c)?;
        flag(cut.un_modified());
        attempt(|| {
            let n = cut.nb_curves()?;
            int(n);
            for i in 1..=n {
                let t = cut.value(i)?;
                real(t.first_parameter());
                real(t.last_parameter());
            }
            Ok(())
        });
        attempt(|| cut.value(0).map(|_| ()));
        let empty = Mat2dCutCurve::new();
        flag(empty.un_modified());
        attempt(|| empty.value(1).map(|_| ()));
        Ok(())
    });
    end();
}

/// `figure indstart sense`: MAT2d_MiniPath.
pub fn mat2d_minipath_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let f = read_figure(&mut r)?;
        let (ind, sense) = (r.integer(), r.flag());
        let mut path = Mat2dMiniPath::new();
        path.perform(&f.figure, ind, sense)?;
        int(path.path().length());
        path.path().iter().for_each(|c| connexion(c));
        for l in 1..=f.figure.length() {
            flag(path.is_root(l));
            flag(path.is_connexions_from(l));
            if path.is_connexions_from(l) {
                int(path.connexions_from(l)?.length());
            }
            attempt(|| {
                connexion(&*path.father(l)?);
                Ok(())
            });
        }
        let count = f.figure.length() + 1;
        attempt(|| path.connexions_from(count).map(|_| ()));
        let mut empty = Mat2dMiniPath::new();
        attempt(|| empty.run_on_connexions());
        Ok(())
    });
    end();
}

/// `-`: MAT2d_Tool2d without circuit and a MAT2d_Mat2d not computed.
pub fn mat2d_tool_case(values: &[f64]) {
    let _ = values;
    attempt(|| {
        let tool = Mat2dTool2d::new();
        real(tool.tolerance_of_confusion());
        attempt(|| tool.geom_pnt(1).map(|_| ()));
        attempt(|| tool.geom_vec(1).map(|_| ()));
        attempt(|| tool.geom_bis(1).map(|_| ()));
        attempt(|| tool.dump(1, 0));
        let mat = Mat2dMat2d::new(false);
        flag(mat.is_done());
        flag(mat.semi_infinite());
        int(mat.number_of_bisectors());
        Ok(())
    });
    end();
}
