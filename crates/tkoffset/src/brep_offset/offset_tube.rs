//! Free-boundary pipe mode of `BRepOffset_Offset::Init(Edge, Offset)`.
use super::offset_rails::trimmed_curve;
use super::offset_rails_geometry::{construction_error, unsupported_edge};
use rustycad_tkbrep::{
    topexp::{Explorer, vertices},
    topods::{Edge, Face, Shape, Wire},
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::{
    geom::GeomSurface,
    top_abs::{Orientation, ShapeEnum},
};
use rustycad_tkgeomalgo::geom_fill::GeomFillPipe;
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeFace;

pub(super) fn initialize_edge_tube(edge: &Edge, offset: f64) -> Result<Face, Failure> {
    if !edge.location().is_identity() {
        return Err(unsupported_edge("Edge"));
    }
    let curve = trimmed_curve(edge)?;
    let surface = edge_tube_surface(&curve, offset.abs())?;
    let face = edge_tube_face(&surface, offset)?;
    validate_tube_history_edges(&face)?;
    Ok(face)
}

pub(super) fn generated_tube_edge(face: &Face, initial: &Shape, source: &Shape) -> Shape {
    let (_, last) = vertices(&Edge::unchecked(initial.clone()), false);
    let index = if last.is_same(source) { 2 } else { 3 };
    let Ok(wires) = Explorer::new(
        &face.clone().into_shape(),
        ShapeEnum::Wire,
        ShapeEnum::Shape,
    ) else {
        return Shape::default();
    };
    if !wires.more() {
        return Shape::default();
    }
    let wire = Wire::unchecked(wires.current().clone());
    tube_edge_at(&wire, face, index)
}

fn tube_edge_at(wire: &Wire, face: &Face, index: usize) -> Shape {
    let Ok(mut edges) = Explorer::new(
        &wire.oriented(Orientation::Forward),
        ShapeEnum::Edge,
        ShapeEnum::Shape,
    ) else {
        return Shape::default();
    };
    let Some(edge) = explorer_edge(&mut edges, index) else {
        return Shape::default();
    };
    orient_generated_tube_edge(Edge::unchecked(edge), wire, face).into_shape()
}

fn explorer_edge(explorer: &mut Explorer, index: usize) -> Option<Shape> {
    for current in 0..=index {
        if !explorer.more() {
            return None;
        }
        let edge = explorer.current().clone();
        if current == index {
            return Some(edge);
        }
        explorer.next().ok()?;
    }
    None
}

fn orient_generated_tube_edge(mut edge: Edge, wire: &Wire, face: &Face) -> Edge {
    if wire.orientation() == Orientation::Reversed {
        edge.reverse();
    }
    if face.orientation() == Orientation::Reversed {
        edge.reverse();
    }
    edge
}

fn edge_tube_surface(
    curve: &Handle<dyn rustycad_tkg3d::geom::GeomCurve>,
    radius: f64,
) -> Result<Handle<dyn GeomSurface>, Failure> {
    let mut pipe = GeomFillPipe::with_radius(curve, radius)?;
    pipe.perform(1.0e-4, false, Continuity::C1, 11, 30)?;
    if !pipe.is_done() {
        return Err(construction_error("GeomFill_Pipe : Cannot make a surface"));
    }
    Ok(pipe.surface())
}

fn edge_tube_face(surface: &Handle<dyn GeomSurface>, offset: f64) -> Result<Face, Failure> {
    let maker =
        BRepBuilderAPIMakeFace::from_surface(surface, rustycad_tkernel::precision::confusion())?;
    let mut face = maker.face()?;
    if offset < 0.0 {
        face.reverse();
    }
    Ok(face)
}

fn validate_tube_history_edges(face: &Face) -> Result<(), Failure> {
    let wires = Explorer::new(
        &face.clone().into_shape(),
        ShapeEnum::Wire,
        ShapeEnum::Shape,
    )?;
    let Some(wire) = first_wire(&wires) else {
        return Err(tube_history_topology_failure(0));
    };
    let edge_count = Explorer::new(
        &wire.oriented(Orientation::Forward),
        ShapeEnum::Edge,
        ShapeEnum::Shape,
    )?
    .count();
    if edge_count < 4 {
        return Err(tube_history_topology_failure(edge_count));
    }
    Ok(())
}

fn first_wire(wires: &Explorer) -> Option<Wire> {
    if wires.more() {
        Some(Wire::unchecked(wires.current().clone()))
    } else {
        None
    }
}

fn tube_history_topology_failure(edge_count: usize) -> Failure {
    let message =
        format!("BRepOffset_Offset::Generated needs four first-wire edges; found {edge_count}");
    Failure::with_type("Standard_NotImplemented", &message, "BRepOffset_Offset")
}

#[cfg(test)]
mod tests {
    use super::{Face, validate_tube_history_edges};

    #[test]
    fn rejects_face_without_native_generated_edge_slots() {
        let failure = validate_tube_history_edges(&Face::default()).unwrap_err();
        assert_eq!(failure.exception_type(), "Standard_NotImplemented");
        assert!(failure.message().contains("found 0"));
    }
}
