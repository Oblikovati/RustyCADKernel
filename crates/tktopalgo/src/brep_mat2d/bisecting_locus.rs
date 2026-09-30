// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepMAT2d/BRepMAT2d_BisectingLocus.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepMAT2d_BisectingLocus`: the bisecting locus of the contours of an explorer, as a
//! MAT graph with its geometry (the bisectors, the nodes and the basic elements).
use super::explorer::BRepMAT2dExplorer;
use crate::bisector::Bisec;
use crate::mat::{MatArc, MatBasicElt, MatFusion, MatGraph, MatListOfBisector, MatNode, MatSide};
use crate::mat2d::{
    Mat2dBiInt, Mat2dCircuit, Mat2dCutCurve, Mat2dFigure, Mat2dLine, Mat2dMat2d, Mat2dTool2d,
};
use rustycad_tkernel::ncollection::{DataMap, Sequence};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dGeometry};
use rustycad_tkmath::geom_abs::JoinType;
use rustycad_tkmath::gp::Point2;
use std::collections::HashMap;

/// The number of sections of each curve of the explorer, by (contour, curve)
/// (`NCollection_DataMap<MAT2d_BiInt, int>`, only searched by key).
type SectionCounts = HashMap<Mat2dBiInt, i32>;

/// `BRepMAT2d_BisectingLocus` (docs/topalgo-design.md section 14): the bisecting locus of
/// the contours of a [`BRepMAT2dExplorer`], computed on one side of one contour. The
/// curves are cut at their curvature extrema and inflections, linked into a MAT2d
/// circuit, and the MAT2d algorithm builds the MAT graph; with several contours the
/// basic elements shared by two contours are merged.
///
/// ```
/// use rustycad_tktopalgo::brep_mat2d::{BRepMAT2dBisectingLocus, BRepMAT2dExplorer};
/// use rustycad_tktopalgo::mat::MatSide;
/// use rustycad_tkmath::geom_abs::JoinType;
/// let mut locus = BRepMAT2dBisectingLocus::new();
/// locus.compute(&mut BRepMAT2dExplorer::new(), 1, MatSide::Left, JoinType::Arc, false)?;
/// assert!(!locus.is_done());
/// assert_eq!(locus.number_of_contours(), 0);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct BRepMAT2dBisectingLocus {
    the_graph: Handle<MatGraph>,
    the_tool: Mat2dTool2d,
    is_done: bool,
    nb_sect: SectionCounts,
    nb_contours: i32,
}

impl Default for BRepMAT2dBisectingLocus {
    fn default() -> Self {
        Self::new()
    }
}

impl BRepMAT2dBisectingLocus {
    /// `BRepMAT2d_BisectingLocus()`: not done, no graph, no contour;
    /// `BRepMAT2dBisectingLocus::new()`.
    pub fn new() -> Self {
        Self {
            the_graph: Handle::default(),
            the_tool: Mat2dTool2d::new(),
            is_done: false,
            nb_sect: SectionCounts::new(),
            nb_contours: 0,
        }
    }

    /// `Compute(anExplo, LineIndex, aSide, aJoinType, IsOpenResult)` (defaults 1,
    /// `MAT_Left`, `GeomAbs_Arc`, false): the locus of the contours of `explo`, from the
    /// line `index_line`, on `side`. Without contours it returns with a new empty graph
    /// and `IsDone` unchanged; when the MAT algorithm fails `IsDone` is false and the
    /// graph stays empty. The errors of MAT2d and MAT are returned;
    /// `locus.compute(&mut explorer, 1, MatSide::Left, JoinType::Arc, false)?`.
    pub fn compute(
        &mut self,
        explo: &mut BRepMAT2dExplorer,
        index_line: i32,
        side: MatSide,
        join_type: JoinType,
        is_open_result: bool,
    ) -> Result<(), Failure> {
        let mut the_mat = Mat2dMat2d::new(is_open_result);
        self.nb_sect.clear();
        self.the_graph = Handle::new(MatGraph::new());
        self.nb_contours = explo.number_of_contours();
        if self.nb_contours == 0 {
            return Ok(());
        }
        self.init_tool(explo, index_line, side, (join_type, is_open_result))?;
        match is_open_result {
            true => the_mat.create_mat_open(&mut self.the_tool)?,
            false => the_mat.create_mat(&mut self.the_tool)?,
        }
        self.is_done = the_mat.is_done();
        if !self.is_done {
            return Ok(());
        }
        self.build_graph(&the_mat)?;
        if explo.number_of_contours() > 1 {
            self.fuse_contours(explo.number_of_contours())?;
        }
        Ok(())
    }

    /// The figure of the explorer cut into sections, its circuit from the line
    /// `index_line`, and the tool initialized on that circuit.
    fn init_tool(
        &mut self,
        explo: &mut BRepMAT2dExplorer,
        index_line: i32,
        side: MatSide,
        (join_type, is_open_result): (JoinType, bool),
    ) -> Result<(), Failure> {
        let mut figure = read_figure(explo)?;
        cut_sketch(&mut figure, &mut self.nb_sect)?;
        let circuit = Handle::new(Mat2dCircuit::new(join_type, is_open_result));
        let left: bool = side == MatSide::Left;
        circuit.perform(&mut figure, explo.get_is_closed(), index_line, left)?;
        self.the_tool.sense(side);
        self.the_tool.set_join_type(join_type);
        self.the_tool.init_items(&circuit);
        Ok(())
    }

    /// The graph of the roots of the MAT trees.
    fn build_graph(&self, the_mat: &Mat2dMat2d) -> Result<(), Failure> {
        let roots: Handle<MatListOfBisector> = Handle::new(MatListOfBisector::new());
        the_mat.init()?;
        while the_mat.more()? {
            roots.back_add(&the_mat.bisector()?);
            the_mat.next()?;
        }
        self.graph_ref()?.perform(
            the_mat.semi_infinite(),
            &roots,
            self.the_tool.number_of_items()?,
            the_mat.number_of_bisectors(),
        )
    }

    /// The fusion of the basic elements duplicated on several lines: the new map of
    /// basic elements, ordered along the lines, then the compacted arcs and nodes.
    fn fuse_contours(&mut self, nb_lines: i32) -> Result<(), Failure> {
        let mut new_map: DataMap<i32, Handle<MatBasicElt>> = DataMap::default();
        let mut index_last: i32 = 1;
        for i in 1..=nb_lines {
            let length = self.circuit()?.line_length(i)?;
            self.renumeration_and_fusion(i, length, &mut index_last, &mut new_map)?;
        }
        let graph = self.graph_ref()?;
        graph.change_basic_elts(&new_map);
        graph.compact_arcs()?;
        graph.compact_nodes()
    }

    /// `RenumerationAndFusion` (private): the first basic element of each item of the
    /// line goes into `new_map`; the others equal to it are merged into it, with their
    /// merged arcs' bisectors.
    fn renumeration_and_fusion(
        &mut self,
        i_line: i32,
        length_line: i32,
        index_last: &mut i32,
        new_map: &mut DataMap<i32, Handle<MatBasicElt>>,
    ) -> Result<(), Failure> {
        let graph = self.the_graph.clone();
        let graph = graph.get().ok_or_else(null_graph)?;
        for i in 1..=length_line {
            let equi = self.circuit()?.ref_to_equi(i_line, i)?;
            let ind_first = *equi.first().ok_or_else(|| empty_equi(i_line, i))?;
            new_map.bind(*index_last, graph.change_basic_elt(ind_first)?);
            *index_last += 1;
            for &other in &equi[1..] {
                let mut merge = MatFusion::default();
                graph.fusion_of_basic_elts(ind_first, other, &mut merge)?;
                if merge.merge_arc1 {
                    self.the_tool
                        .bisec_fusion(merge.geom_index_arc1, merge.geom_index_arc2)?;
                }
                if merge.merge_arc2 {
                    self.the_tool
                        .bisec_fusion(merge.geom_index_arc3, merge.geom_index_arc4)?;
                }
            }
        }
        Ok(())
    }

    /// `IsDone()`; `locus.is_done()`.
    pub fn is_done(&self) -> bool {
        self.is_done
    }

    /// `Graph()`: the graph (null before the first Compute); `locus.graph()`.
    pub fn graph(&self) -> Handle<MatGraph> {
        self.the_graph.clone()
    }

    /// `NumberOfContours()`: the contours of the last explorer; `locus.number_of_contours()`.
    pub fn number_of_contours(&self) -> i32 {
        self.nb_contours
    }

    /// `NumberOfElts(IndLine)`: the basic elements of a line of the circuit
    /// (Standard_NullObject before a Compute that built one);
    /// `locus.number_of_elts(1)?`.
    pub fn number_of_elts(&self, ind_line: i32) -> Result<i32, Failure> {
        self.circuit()?.line_length(ind_line)
    }

    /// `NumberOfSections(IndLine, Index)`: the pieces the curve `Index` of the contour
    /// `IndLine` was cut into; Standard_NoSuchObject for an unknown curve;
    /// `locus.number_of_sections(1, 1)?`.
    pub fn number_of_sections(&self, ind_line: i32, index: i32) -> Result<i32, Failure> {
        self.nb_sect
            .get(&Mat2dBiInt::new(ind_line, index))
            .copied()
            .ok_or_else(|| {
                Failure::with_type(
                    "Standard_NoSuchObject",
                    &format!("NCollection_DataMap::operator(): no curve ({ind_line}, {index}); expected a curve of the explorer"),
                    "",
                )
            })
    }

    /// `BasicElt(IndLine, Index)`: the basic element `Index` of the line `IndLine`
    /// (numbered after the elements of the lines before it);
    /// `locus.basic_elt(1, 1)?`.
    pub fn basic_elt(&self, ind_line: i32, index: i32) -> Result<Handle<MatBasicElt>, Failure> {
        let mut ind: i32 = index;
        for i in 1..ind_line {
            ind += self.circuit()?.line_length(i)?;
        }
        self.graph_ref()?.basic_elt(ind)
    }

    /// `GeomElt(aBasicElt)`: the curve or point of a basic element;
    /// `locus.geom_elt_of_basic_elt(&element)?`.
    pub fn geom_elt_of_basic_elt(
        &self,
        basic_elt: &Handle<MatBasicElt>,
    ) -> Result<Handle<dyn Geom2dGeometry>, Failure> {
        let element = basic_elt.get().ok_or_else(|| null_argument("aBasicElt"))?;
        self.the_tool.geom_elt(element.geom_index())
    }

    /// `GeomElt(aNode)`: the point of a node; `locus.geom_elt_of_node(&node)?`.
    pub fn geom_elt_of_node(&self, node: &Handle<MatNode>) -> Result<Point2, Failure> {
        let node = node.get().ok_or_else(|| null_argument("aNode"))?;
        self.the_tool.geom_pnt(node.geom_index())
    }

    /// `GeomBis(anArc, Reverse)`: the bisector of an arc and whether it runs from the
    /// second node of the arc (`Reverse`): true when it starts at infinity, or when it
    /// is finite and its last point is closer to the first node;
    /// `let (bisec, reverse) = locus.geom_bis(&arc)?;`.
    pub fn geom_bis(&self, arc: &Handle<MatArc>) -> Result<(Bisec, bool), Failure> {
        let arc = arc.get().ok_or_else(|| null_argument("anArc"))?;
        let bisec: &Bisec = self.the_tool.geom_bis(arc.geom_index())?;
        let bis: Handle<dyn Geom2dCurve> = bisec.value().cast();
        let curve = bis.get().ok_or_else(|| null_argument("the bisector"))?;
        let mut reverse = false;
        if curve.first_parameter() <= -precision::infinite() {
            reverse = true;
        } else if curve.last_parameter() < precision::infinite() {
            let pf = curve.value(curve.first_parameter())?;
            let pl = curve.value(curve.last_parameter())?;
            let p_node = self.geom_elt_of_node(&arc.first_node())?;
            reverse = p_node.square_distance(pf) > p_node.square_distance(pl);
        }
        Ok((bisec.clone(), reverse))
    }

    /// `theTool.Circuit()`: Standard_NullObject before a Compute that built one (a
    /// crash natively).
    fn circuit(&self) -> Result<Handle<Mat2dCircuit>, Failure> {
        let circuit = self.the_tool.circuit();
        match circuit.is_null() {
            true => Err(null_argument("the circuit of the tool")),
            false => Ok(circuit),
        }
    }

    /// The graph, Standard_NullObject before the first Compute (a crash natively).
    fn graph_ref(&self) -> Result<&MatGraph, Failure> {
        self.the_graph.get().ok_or_else(null_graph)
    }
}

/// The figure of the explorer: one line per contour with its curves in order.
fn read_figure(explo: &mut BRepMAT2dExplorer) -> Result<Mat2dFigure, Failure> {
    let mut figure: Mat2dFigure = Sequence::default();
    for i in 1..=explo.number_of_contours() {
        let mut line: Mat2dLine = Sequence::default();
        explo.init(i);
        while explo.more()? {
            line.append(explo.value()?.cast());
            explo.next();
        }
        figure.append(line);
    }
    Ok(figure)
}

/// `CutSketch`: each curve of each line replaced by its pieces cut by MAT2d_CutCurve,
/// and the number of pieces recorded by (line, initial curve index).
fn cut_sketch(figure: &mut Mat2dFigure, nb_sect: &mut SectionCounts) -> Result<(), Failure> {
    let mut cuter = Mat2dCutCurve::new();
    for i in 1..=figure.length() {
        let contour: &mut Mat2dLine = figure.change_value(i as usize)?;
        let mut i_curve_init: i32 = 0;
        let mut j: usize = 1;
        while j <= contour.len() {
            i_curve_init += 1;
            cuter.perform(&contour.value(j)?.cast())?;
            let mut nb_section: i32 = 1;
            if !cuter.un_modified() {
                let ico = j;
                nb_section = cuter.nb_curves()?;
                for k in 1..=nb_section {
                    contour.insert_after(j, cuter.value(k)?.cast())?;
                    j += 1;
                }
                contour.remove(ico)?;
                j -= 1;
            }
            nb_sect.insert(Mat2dBiInt::new(i, i_curve_init), nb_section);
            j += 1;
        }
    }
    Ok(())
}

fn null_graph() -> Failure {
    null_argument("the graph (Compute not called)")
}

fn null_argument(what: &str) -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        &format!("BRepMAT2d_BisectingLocus: null {what}; expected a computed object"),
        "",
    )
}

fn empty_equi(line: i32, item: i32) -> Failure {
    Failure::with_type(
        "Standard_OutOfRange",
        &format!("MAT2d_Circuit::RefToEqui({line}, {item}): empty; expected one index at least"),
        "",
    )
}
