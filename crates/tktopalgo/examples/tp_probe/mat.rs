//! MAT and MAT2d probe operations: the bisecting locus pipeline of
//! BRepMAT2d_BisectingLocus::Compute, the Rust twin of tools/oracle/tktopalgo_probe_mat.cxx.
use super::bisector::{join, sample, span};
use super::common::{Reader, attempt, end, flag, int, pnt2d, real, text};
use super::curve2d::curve2d;
use rustycad_tkernel::ncollection::{DataMap, Sequence};
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dGeometry, Geom2dTrimmedCurve};
use rustycad_tktopalgo::mat::{
    MatArc, MatBasicElt, MatFusion, MatGraph, MatListOfBisector, MatNode, MatSide, MatZone,
};
use rustycad_tktopalgo::mat2d::{
    Mat2dCircuit, Mat2dConnexion, Mat2dCutCurve, Mat2dFigure, Mat2dLine, Mat2dMat2d, Mat2dTool2d,
};
use std::collections::HashMap;

pub(super) struct FigureInput {
    pub figure: Mat2dFigure,
    pub closed: Sequence<bool>,
}

pub(super) fn read_figure(r: &mut Reader<'_>) -> Result<FigureInput, Failure> {
    let mut input = FigureInput {
        figure: Sequence::default(),
        closed: Sequence::default(),
    };
    let n = r.integer();
    for _ in 0..n {
        input.closed.append(r.flag());
        let m = r.integer();
        let mut line: Mat2dLine = Sequence::default();
        for _ in 0..m {
            line.append(curve2d(r)?.cast());
        }
        input.figure.append(line);
    }
    Ok(input)
}

fn type_name<T: ?Sized + RuntimeTyped>(h: &Handle<T>) {
    text(h.dynamic_type().map_or("", |t| t.name()).as_bytes());
}

/// The static CutSketch of BRepMAT2d_BisectingLocus.cxx.
fn cut_sketch(
    figure: &mut Mat2dFigure,
    nb_sect: &mut HashMap<(i32, i32), i32>,
) -> Result<(), Failure> {
    let mut cuter = Mat2dCutCurve::new();
    for i in 1..=figure.length() {
        let contour: &mut Mat2dLine = figure.change_value(i as usize)?;
        let mut i_curve_init = 0;
        let mut j: i32 = 1;
        while j <= contour.length() {
            i_curve_init += 1;
            let curve: Handle<dyn Geom2dCurve> = contour.value(j as usize)?.cast();
            cuter.perform(&curve)?;
            let mut nb_section = 1;
            if !cuter.un_modified() {
                let ico = j;
                nb_section = cuter.nb_curves()?;
                for k in 1..=nb_section {
                    contour.insert_after(j as usize, cuter.value(k)?.cast())?;
                    j += 1;
                }
                contour.remove(ico as usize)?;
                j -= 1;
            }
            nb_sect.insert((i, i_curve_init), nb_section);
            j += 1;
        }
    }
    Ok(())
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

pub(super) fn connexion(c: &Mat2dConnexion) {
    int(c.index_first_line());
    int(c.index_second_line());
    int(c.index_item_on_first());
    int(c.index_item_on_second());
    real(c.parameter_on_first());
    real(c.parameter_on_second());
    pnt2d(c.point_on_first());
    pnt2d(c.point_on_second());
    real(c.distance());
}

fn circuit_items(c: &Mat2dCircuit) -> Result<(), Failure> {
    int(c.number_of_items());
    for i in 1..=c.number_of_items() {
        let g: Handle<dyn Geom2dGeometry> = c.value(i)?;
        type_name(&g);
        let cu: Handle<dyn Geom2dCurve> = g.cast();
        if let Some(cu) = cu.get() {
            real(cu.first_parameter());
            real(cu.last_parameter());
        }
        flag(c.connexion_on(i));
        if c.connexion_on(i) {
            connexion(&*c.connexion(i)?);
        }
    }
    Ok(())
}

fn circuit(c: &Mat2dCircuit, nlines: i32) -> Result<(), Failure> {
    circuit_items(c)?;
    for l in 1..=nlines {
        attempt(|| {
            let n = c.line_length(l)?;
            int(n);
            for k in 1..=n {
                attempt(|| {
                    let s = c.ref_to_equi(l, k)?;
                    int(s.len() as i32);
                    s.iter().for_each(|e| int(*e));
                    Ok(())
                });
            }
            Ok(())
        });
    }
    attempt(|| {
        int(c.line_length(nlines + 2)?);
        Ok(())
    });
    Ok(())
}

fn geom_bis(tool: &Mat2dTool2d, index: i32) {
    attempt(|| {
        let t: Handle<Geom2dTrimmedCurve> = tool.geom_bis(index)?.value();
        type_name(&t.basis_curve());
        let (a, b) = (t.first_parameter(), t.last_parameter());
        real(a);
        real(b);
        let (a, b) = span(a, b);
        for k in [0, 2, 4] {
            attempt(|| {
                pnt2d(t.value(sample(a, b, k))?);
                Ok(())
            });
        }
        Ok(())
    });
}

fn graph_arcs(g: &MatGraph, tool: &Mat2dTool2d) {
    for i in 1..=g.number_of_arcs() {
        attempt(|| {
            let a = g.arc(i)?;
            int(a.index());
            int(a.geom_index());
            int(index_elt(&a.first_element()));
            int(index_elt(&a.second_element()));
            int(index_node(&a.first_node()));
            int(index_node(&a.second_node()));
            for side in [MatSide::Left, MatSide::Right] {
                attempt(|| {
                    flag(a.has_neighbour(&a.first_node(), side)?);
                    int(index_arc(&a.neighbour(&a.first_node(), side)?));
                    Ok(())
                });
            }
            geom_bis(tool, a.geom_index());
            Ok(())
        });
    }
}

fn graph_nodes(g: &MatGraph, tool: &Mat2dTool2d) {
    for i in 1..=g.number_of_nodes() {
        attempt(|| {
            let n = g.node(i)?;
            int(n.index());
            int(n.geom_index());
            real(n.distance());
            attempt(|| {
                flag(n.pending_node()?);
                Ok(())
            });
            flag(n.on_basic_elt());
            flag(n.infinite());
            attempt(|| {
                pnt2d(tool.geom_pnt(n.geom_index())?);
                Ok(())
            });
            attempt(|| {
                let mut s: Sequence<Handle<MatArc>> = Sequence::default();
                n.linked_arcs(&mut s)?;
                int(s.length());
                s.iter().for_each(|a| int(index_arc(a)));
                Ok(())
            });
            attempt(|| {
                let mut s: Sequence<Handle<MatBasicElt>> = Sequence::default();
                n.near_elts(&mut s)?;
                int(s.length());
                s.iter().for_each(|e| int(index_elt(e)));
                Ok(())
            });
            Ok(())
        });
    }
}

fn graph_elts(g: &MatGraph) {
    for i in 1..=g.number_of_basic_elts() {
        attempt(|| {
            let e = g.basic_elt(i)?;
            int(e.index());
            int(e.geom_index());
            int(index_arc(&e.start_arc()));
            int(index_arc(&e.end_arc()));
            // A zone that reaches an infinite node turns from StartArc: a null one crashes
            // natively (MAT_Zone::NodeForTurn dereferences it).
            if e.start_arc().is_null() {
                return Ok(());
            }
            attempt(|| {
                let z = MatZone::with_basic_elt(&e)?;
                flag(z.no_empty_zone());
                flag(z.limited());
                int(z.number_of_arcs());
                for k in 1..=z.number_of_arcs() {
                    int(index_arc(&z.arc_on_frontier(k)?));
                }
                Ok(())
            });
            Ok(())
        });
    }
}

fn graph(g: &MatGraph, tool: &Mat2dTool2d) {
    int(g.number_of_arcs());
    int(g.number_of_nodes());
    int(g.number_of_basic_elts());
    int(g.number_of_infinite_nodes());
    graph_arcs(g, tool);
    graph_nodes(g, tool);
    graph_elts(g);
}

/// BRepMAT2d_BisectingLocus::RenumerationAndFusion.
fn fusion(
    graph: &MatGraph,
    tool: &mut Mat2dTool2d,
    (iline, length): (i32, i32),
    index_last: &mut i32,
    new_map: &mut DataMap<i32, Handle<MatBasicElt>>,
) -> Result<(), Failure> {
    for i in 1..=length {
        let s = tool.circuit().ref_to_equi(iline, i)?;
        let ind_first = s[0];
        new_map.bind(*index_last, graph.change_basic_elt(ind_first)?);
        *index_last += 1;
        for &other in &s[1..] {
            let mut merge = MatFusion::default();
            graph.fusion_of_basic_elts(ind_first, other, &mut merge)?;
            flag(merge.merge_arc1);
            flag(merge.merge_arc2);
            if merge.merge_arc1 {
                tool.bisec_fusion(merge.geom_index_arc1, merge.geom_index_arc2)?;
            }
            if merge.merge_arc2 {
                tool.bisec_fusion(merge.geom_index_arc3, merge.geom_index_arc4)?;
            }
        }
    }
    Ok(())
}

fn locus(r: &mut Reader<'_>) -> Result<(), Failure> {
    let mut f = read_figure(r)?;
    let side = if r.integer() == 0 {
        MatSide::Left
    } else {
        MatSide::Right
    };
    let join_type = join(r.integer());
    let open = r.flag();
    let nlines = f.figure.length();
    let mut nb_sect: HashMap<(i32, i32), i32> = HashMap::new();
    cut_sketch(&mut f.figure, &mut nb_sect)?;
    for l in 1..=nlines {
        int(f.figure.value(l as usize)?.length());
        let mut k = 1;
        while let Some(n) = nb_sect.get(&(l, k)) {
            int(*n);
            k += 1;
        }
    }
    let circuit_handle: Handle<Mat2dCircuit> = Handle::new(Mat2dCircuit::new(join_type, open));
    circuit_handle.perform(&mut f.figure, &f.closed, 1, side == MatSide::Left)?;
    circuit(&circuit_handle, nlines)?;
    let mut tool = Mat2dTool2d::new();
    tool.sense(side);
    tool.set_join_type(join_type);
    tool.init_items(&circuit_handle);
    let mut mat = Mat2dMat2d::new(open);
    if open {
        mat.create_mat_open(&mut tool)?;
    } else {
        mat.create_mat(&mut tool)?;
    }
    flag(mat.is_done());
    flag(mat.semi_infinite());
    int(mat.number_of_bisectors());
    if mat.is_done() {
        locus_graph(&mat, &mut tool, nlines)?;
    }
    Ok(())
}

fn locus_graph(mat: &Mat2dMat2d, tool: &mut Mat2dTool2d, nlines: i32) -> Result<(), Failure> {
    let roots: Handle<MatListOfBisector> = Handle::new(MatListOfBisector::new());
    mat.init()?;
    while mat.more()? {
        roots.back_add(&mat.bisector()?);
        mat.next()?;
    }
    int(roots.number());
    let graph_handle: Handle<MatGraph> = Handle::new(MatGraph::new());
    graph_handle.perform(
        mat.semi_infinite(),
        &roots,
        tool.number_of_items()?,
        mat.number_of_bisectors(),
    )?;
    if nlines > 1 {
        let mut new_map: DataMap<i32, Handle<MatBasicElt>> = DataMap::default();
        let mut index_last = 1;
        for l in 1..=nlines {
            let length = tool.circuit().line_length(l)?;
            fusion(
                &graph_handle,
                tool,
                (l, length),
                &mut index_last,
                &mut new_map,
            )?;
        }
        graph_handle.change_basic_elts(&new_map);
        graph_handle.compact_arcs()?;
        graph_handle.compact_nodes()?;
    }
    graph(&graph_handle, tool);
    Ok(())
}

/// `figure side join openresult`: the bisecting locus of the figure.
pub fn mat2d_locus_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| locus(&mut r));
    end();
}
