// The two-loop line-pcurve case of ShapeFix_Face::FixLoopWire.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Extracts two closed loops after the narrow bowtie wire split.
use super::ShapeFixWire;
use crate::shape_analysis::ShapeAnalysisWire;
use crate::shape_extend::ShapeExtendWireData;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{
    self, Builder, Compound, Edge, Face, Shape, ShapeIterator, Vertex, Wire,
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;

/// Splits one line-pcurve bowtie face into the two triangular faces returned by OCCT.
pub(super) fn split_bowtie_face(face: &Face, tolerance: f64) -> Result<Option<Shape>, Failure> {
    let Some(wire) = single_face_wire(face)? else {
        return Ok(None);
    };
    let analyzer = Handle::new(ShapeAnalysisWire::from_wire(&wire, face, tolerance)?);
    let fixer = ShapeFixWire::new();
    fixer.init(&analyzer);
    if !fixer.fix_line_self_intersections()? {
        return Ok(None);
    }
    let loops = split_wire_loops(&analyzer.wire_data())?;
    let Some([first, second]) = two_loops(loops) else {
        return Ok(None);
    };
    make_face_compound(face, first, second).map(Some)
}

/// Returns a direct face wire only when the face has one boundary.
fn single_face_wire(face: &Face) -> Result<Option<Wire>, Failure> {
    let mut wires = Vec::new();
    for child in ShapeIterator::new(&face.clone().into_shape(), false, true) {
        if child.shape_type()? == ShapeEnum::Wire {
            wires.push(topods::wire(&child)?);
        }
    }
    Ok((wires.len() == 1).then(|| wires.remove(0)))
}

/// Splits the six-edge bowtie at its unique degree-four crossing vertex.
fn split_wire_loops(data: &Handle<ShapeExtendWireData>) -> Result<Vec<Vec<Edge>>, Failure> {
    let Some(data) = data.get() else {
        return Err(Failure::new(
            "ShapeFix_Face::FixLoopWire requires wire data",
        ));
    };
    let edges = wire_edges(data)?;
    if edges.len() != 6 {
        return Ok(Vec::new());
    }
    let (ends, center) = edge_vertex_indices(&edges)?;
    let Some(center) = center else {
        return Ok(Vec::new());
    };
    trace_center_cycles(&edges, &ends, center)
}

/// Reads the ordered edge list retained by the face wire fixer.
fn wire_edges(data: &ShapeExtendWireData) -> Result<Vec<Edge>, Failure> {
    (1..=data.nb_edges())
        .map(|index| data.edge(index))
        .collect()
}

/// Interns the edge endpoints and finds the sole degree-four vertex.
fn edge_vertex_indices(edges: &[Edge]) -> Result<(Vec<[usize; 2]>, Option<usize>), Failure> {
    let mut vertices: Vec<Vertex> = Vec::new();
    let mut ends = Vec::with_capacity(edges.len());
    for edge in edges {
        let (first, last) = rustycad_tkbrep::topexp::vertices(edge, false);
        ends.push([
            intern_vertex(&mut vertices, first),
            intern_vertex(&mut vertices, last),
        ]);
    }
    let degrees = vertex_degrees(&ends, vertices.len());
    let centers: Vec<usize> = degrees
        .iter()
        .enumerate()
        .filter_map(|(index, degree)| (*degree == 4).then_some(index))
        .collect();
    let center = centers.first().copied().filter(|_| centers.len() == 1);
    Ok((ends, center))
}

/// Returns an existing identical vertex index or adds the endpoint.
fn intern_vertex(vertices: &mut Vec<Vertex>, vertex: Vertex) -> usize {
    if let Some(index) = vertices.iter().position(|other| other.is_same(&vertex)) {
        return index;
    }
    vertices.push(vertex);
    vertices.len() - 1
}

/// Counts endpoint incidence by interned vertex index.
fn vertex_degrees(ends: &[[usize; 2]], count: usize) -> Vec<usize> {
    let mut degrees = vec![0; count];
    for [first, last] in ends {
        degrees[*first] += 1;
        degrees[*last] += 1;
    }
    degrees
}

/// Walks each simple cycle from the crossing to the next crossing.
fn trace_center_cycles(
    edges: &[Edge],
    ends: &[[usize; 2]],
    center: usize,
) -> Result<Vec<Vec<Edge>>, Failure> {
    let mut used = vec![false; edges.len()];
    let mut loops = Vec::new();
    for seed in 0..edges.len() {
        if used[seed] || !ends[seed].contains(&center) {
            continue;
        }
        loops.push(trace_one_cycle(edges, ends, center, seed, &mut used)?);
    }
    Ok(loops)
}

/// Consumes one edge cycle, returning empty when another vertex is not manifold.
fn trace_one_cycle(
    edges: &[Edge],
    ends: &[[usize; 2]],
    center: usize,
    seed: usize,
    used: &mut [bool],
) -> Result<Vec<Edge>, Failure> {
    let mut cycle = vec![edges[seed].clone()];
    used[seed] = true;
    let mut current = other_vertex(ends[seed], center);
    while current != center {
        let Some(next) =
            (0..edges.len()).find(|index| !used[*index] && ends[*index].contains(&current))
        else {
            return Ok(Vec::new());
        };
        cycle.push(edges[next].clone());
        used[next] = true;
        current = other_vertex(ends[next], current);
    }
    Ok(cycle)
}

/// Returns the opposite endpoint of an edge from the current vertex.
fn other_vertex(ends: [usize; 2], current: usize) -> usize {
    if ends[0] == current { ends[1] } else { ends[0] }
}

/// Keeps the result only for exactly two closed three-edge loops.
fn two_loops(mut loops: Vec<Vec<Edge>>) -> Option<[Vec<Edge>; 2]> {
    if loops.len() != 2 || loops.iter().any(|cycle| cycle.len() != 3) {
        return None;
    }
    let second = loops.pop()?;
    let first = loops.pop()?;
    Some([first, second])
}

/// Builds the split face compound while retaining the original support surface.
fn make_face_compound(face: &Face, first: Vec<Edge>, second: Vec<Edge>) -> Result<Shape, Failure> {
    let first_face = make_loop_face(face, &first)?;
    let second_face = make_loop_face(face, &second)?;
    let builder = Builder;
    let mut compound = Compound::default();
    builder.make_compound(&mut compound);
    let mut result = compound.into_shape();
    builder.add(&mut result, &first_face.into_shape())?;
    builder.add(&mut result, &second_face.into_shape())?;
    Ok(result)
}

/// Builds one triangular boundary wire and face on the source support surface.
fn make_loop_face(face: &Face, edges: &[Edge]) -> Result<Face, Failure> {
    for edge in edges {
        rustycad_tktopalgo::brep_lib::same_parameter_edge(edge, BRepTool::edge_tolerance(edge)?)?;
    }
    let wire = rustycad_tktopalgo::brep_lib::BRepLibMakeWire::from_three_edges(
        &edges[0], &edges[1], &edges[2],
    )?
    .wire()?;
    let (surface, _) = rustycad_tkbrep::brep::BRepTool::surface_with_location(face)?;
    rustycad_tktopalgo::brep_lib::BRepLibMakeFace::from_surface_wire(&surface, &wire, true)?.face()
}
