// Rust translation of the section indexing of BRepOffsetAPI_ThruSections::Build
// (ModelingAlgorithms/TKOffset/BRepOffsetAPI/BRepOffsetAPI_ThruSections.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The working sections of `Build` and the face indices of their edges and vertices,
//! from `BRepFill_CompatibleWires` or from the wires as given.
use super::BRepOffsetAPIThruSections;
use rustycad_tkbool::brep_fill::BRepFillCompatibleWires;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_tools::WireExplorer;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Shape, ShapeIterator, Vertex, Wire};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

impl BRepOffsetAPIThruSections {
    /// The `myWCheck` branch: the compatible sections replace the wires and the new
    /// edges are indexed; false (not done, with the status of the compatibility) when
    /// they cannot be made compatible.
    pub(super) fn make_compatible(&mut self) -> Result<bool, Failure> {
        let mut georges = BRepFillCompatibleWires::with_sections(&self.my_wires);
        georges.perform(true)?;
        if !georges.is_done() {
            self.my_status = georges.get_status();
            self.base.set_not_done();
            return Ok(false);
        }
        let working: Vec<Shape> = georges.shape().to_vec();
        self.my_degen1 = georges.is_degenerated_first_section();
        self.my_degen2 = georges.is_degenerated_last_section();
        let first_section = if self.my_degen1 { 1 } else { 0 };
        self.my_nb_edges_in_section += topods::wire(&working[first_section])?.nb_children();
        for (wire, working_section) in self.my_wires.clone().iter().zip(&working) {
            let section = topods::wire(working_section)?;
            self.index_split_edges(wire, &section, &georges)?;
        }
        self.my_wires = working;
        Ok(true)
    }

    /// The indices in the working section of the splits of each edge of a section,
    /// and the signed index of its end vertices kept by the first or last split.
    fn index_split_edges(
        &mut self,
        wire: &Shape,
        section: &Wire,
        georges: &BRepFillCompatibleWires,
    ) -> Result<(), Failure> {
        let mut children = ShapeIterator::new(wire, true, true);
        while children.more() {
            let edge = topods::edge(children.value()?)?;
            let indices = self.split_indices(&edge, section, georges)?;
            self.my_edge_new_indices.bind(edge.into_shape(), indices);
            children.next();
        }
        Ok(())
    }

    /// The working-section indices of the splits of one edge.
    fn split_indices(
        &mut self,
        edge: &Edge,
        section: &Wire,
        georges: &BRepFillCompatibleWires,
    ) -> Result<Vec<i32>, Failure> {
        let (v_first, v_last) = topexp::vertices(edge, false);
        let new_edges = georges.generated_shapes(edge);
        let nb_new_edges = new_edges.len();
        let mut sign = 1;
        let mut indices = Vec::new();
        for (kk, new_edge) in new_edges.iter().enumerate() {
            let index = position_in_section(section, new_edge, &mut sign)?;
            indices.push(index);
            if kk == 0 || kk + 1 == nb_new_edges {
                let ends = (v_first.clone(), v_last.clone());
                self.index_kept_vertices(&topods::edge(new_edge)?, ends, sign * index);
            }
        }
        Ok(indices)
    }

    /// Binds the first (last) vertex of the edge to the signed index when the split
    /// starts (ends) at it and it has no index yet.
    fn index_kept_vertices(
        &mut self,
        new_edge: &Edge,
        (v_first, v_last): (Vertex, Vertex),
        index: i32,
    ) {
        let (new_first, new_last) = topexp::vertices(new_edge, false);
        if new_first.is_same(&v_first) && !self.my_vertex_index.is_bound(&v_first) {
            self.my_vertex_index.bind(v_first.into_shape(), index);
        }
        if new_last.is_same(&v_last) && !self.my_vertex_index.is_bound(&v_last) {
            self.my_vertex_index.bind(v_last.into_shape(), -index);
        }
    }

    /// The no-check branch: each edge of each section is its own face index; the
    /// number of edges of a section is the greatest count; a section of one
    /// degenerated edge sets the degenerated flag of its end.
    pub(super) fn index_unchecked_wires(&mut self) -> Result<(), Failure> {
        let mut an_edge = Edge::default();
        for ii in 0..self.my_wires.len() {
            let wire = self.my_wires[ii].clone();
            let mut explorer = Explorer::new(&wire, ShapeEnum::Edge, ShapeEnum::Shape)?;
            let mut inde = 1;
            while explorer.more() {
                an_edge = topods::edge(explorer.current())?;
                self.index_unchecked_edge(&an_edge, inde);
                explorer.next()?;
                inde += 1;
            }
            inde -= 1;
            self.my_nb_edges_in_section = self.my_nb_edges_in_section.max(inde);
            if inde == 1 && BRepTool::degenerated(&an_edge)? {
                if ii == 0 {
                    self.my_degen1 = true;
                } else {
                    self.my_degen2 = true;
                }
            }
        }
        Ok(())
    }

    /// One edge of the no-check branch and its vertices.
    fn index_unchecked_edge(&mut self, edge: &Edge, inde: i32) {
        self.my_edge_new_indices
            .bind(edge.as_ref().clone(), vec![inde]);
        let (v1, v2) = topexp::vertices(edge, false);
        if !self.my_vertex_index.is_bound(&v1) {
            self.my_vertex_index.bind(v1.into_shape(), inde);
        }
        if !self.my_vertex_index.is_bound(&v2) {
            self.my_vertex_index.bind(v2.into_shape(), -inde);
        }
    }
}

/// The 1-based position of the edge in the wire explorer order of the section (one
/// past the last edge when absent); `sign` becomes the sense of the found edge.
fn position_in_section(section: &Wire, new_edge: &Shape, sign: &mut i32) -> Result<i32, Failure> {
    let mut index = 1;
    let mut explorer = WireExplorer::with_wire(section)?;
    while explorer.more() {
        let working_edge = explorer.current();
        if working_edge.is_same(new_edge) {
            *sign = if working_edge.orientation() == Orientation::Forward {
                1
            } else {
                -1
            };
            break;
        }
        explorer.next()?;
        index += 1;
    }
    Ok(index)
}
