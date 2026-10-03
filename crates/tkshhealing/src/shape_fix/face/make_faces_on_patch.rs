//! Root-wire and hole assignment used by `ShapeFix_ComposeShell::MakeFacesOnPatch`.
use super::{ShapeFixFace, ShapeFixFaceWireMap};
use crate::shape_analysis::{
    ShapeAnalysisEdge, ShapeAnalysisSurface, ShapeAnalysisTransferParametersProj,
};
use crate::shape_build::ShapeBuildReShape;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{self, Builder, Edge, Face, Shape, ShapeIterator, Vertex, Wire};
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};
use rustycad_tkmath::gp::Point2;
use rustycad_tkmath::toploc::Location;
use rustycad_tktopalgo::brep_top_adaptor::BRepTopAdaptorFClass2d;

/// Assigns patch wires to outside roots and their contained holes.
/// Example: `let faces = make_faces_on_patch(&surface, &mut loops, &source, &location, &context, true)?`.
pub(crate) fn make_faces_on_patch(
    surface: &Handle<dyn GeomSurface>,
    loops: &mut Sequence<Shape>,
    original_face: &Face,
    location: &Location,
    context: &Handle<ShapeBuildReShape>,
    invert_edge_status: bool,
) -> Result<Vec<Face>, Failure> {
    if loops.len() == 1 {
        return make_single_loop_face(surface, loops.value(1)?, location, invert_edge_status);
    }
    let support = support_face(surface, location)?;
    let roots = find_root_wires(loops, &support)?;
    let roots = recover_lost_roots(loops, roots)?;
    assemble_root_faces(surface, loops, &roots, original_face, location, context)
}

fn support_face(surface: &Handle<dyn GeomSurface>, location: &Location) -> Result<Face, Failure> {
    let mut face = Face::default();
    BRepBuilder::default().make_face_on_located_surface(
        &mut face,
        surface,
        location,
        precision::confusion(),
    )?;
    Ok(face)
}

fn make_single_loop_face(
    surface: &Handle<dyn GeomSurface>,
    shape: &Shape,
    location: &Location,
    invert_edge_status: bool,
) -> Result<Vec<Face>, Failure> {
    if shape.shape_type()? != ShapeEnum::Wire {
        return Ok(Vec::new());
    }
    let mut face = support_face(surface, location)?;
    Builder.add(&mut face, shape)?;
    orient_single_loop(&mut face, invert_edge_status)?;
    Ok(vec![face])
}

fn orient_single_loop(face: &mut Face, enabled: bool) -> Result<(), Failure> {
    if !enabled {
        return Ok(());
    }
    let fixer = ShapeFixFace::new(face.clone().into_shape());
    fixer.set_fix_add_natural_bound_mode(0);
    let mut map = ShapeFixFaceWireMap::default();
    fixer.fix_orientation_with_map(&mut map)?;
    *face = fixer.face()?;
    Ok(())
}

fn find_root_wires(loops: &Sequence<Shape>, support: &Face) -> Result<Vec<Shape>, Failure> {
    let mut roots = Vec::new();
    for index in 1..=loops.len() {
        let candidate = loops.value(index)?;
        if !is_manifold_wire(candidate)? {
            continue;
        }
        if wire_is_root(loops, index, support, candidate)? {
            roots.push(candidate.clone());
        }
    }
    Ok(roots)
}

fn wire_is_root(
    loops: &Sequence<Shape>,
    candidate_index: usize,
    support: &Face,
    candidate: &Shape,
) -> Result<bool, Failure> {
    let Some(sample) = wire_sample_point(&topods::wire(candidate)?, support)? else {
        return Ok(false);
    };
    Ok(other_wire_contains_sample(loops, candidate_index, support, candidate, sample)?.is_none())
}

fn other_wire_contains_sample(
    loops: &Sequence<Shape>,
    candidate_index: usize,
    support: &Face,
    candidate: &Shape,
    sample: Point2,
) -> Result<Option<()>, Failure> {
    for other_index in 1..=loops.len() {
        if candidate_index == other_index {
            continue;
        }
        let other = loops.value(other_index)?;
        if is_manifold_wire(other)? && contains_wire_sample(support, other, candidate, sample)? {
            return Ok(Some(()));
        }
    }
    Ok(None)
}

fn is_manifold_wire(shape: &Shape) -> Result<bool, Failure> {
    Ok(shape.shape_type()? == ShapeEnum::Wire
        && matches!(
            shape.orientation(),
            Orientation::Forward | Orientation::Reversed
        ))
}

fn contains_wire_sample(
    support: &Face,
    wire: &Shape,
    candidate: &Shape,
    point: Point2,
) -> Result<bool, Failure> {
    let Some(candidate_face) = face_with_wire(support, wire)? else {
        return Ok(false);
    };
    let classifier = BRepTopAdaptorFClass2d::new(&candidate_face, precision::p_confusion())?;
    let infinite = classifier.perform_infinite_point()?;
    let mut state = classifier.perform(point, false)?;
    if matches!(state, State::On | State::Unknown) {
        state = classify_edge_endpoints(&classifier, support, candidate)?;
    }
    Ok(state != infinite)
}

fn face_with_wire(support: &Face, wire: &Shape) -> Result<Option<Face>, Failure> {
    let mut face = support
        .clone()
        .empty_copied()
        .and_then(|shape| topods::face(&shape))?;
    let Some(manifold) = manifold_wire(wire)? else {
        return Ok(None);
    };
    Builder.add(&mut face, &manifold.into_shape())?;
    Ok(Some(face))
}

fn manifold_wire(shape: &Shape) -> Result<Option<Wire>, Failure> {
    let mut wire = Wire::default();
    Builder.make_wire(&mut wire);
    for child in ShapeIterator::new(&shape.clone(), false, false) {
        if matches!(
            child.orientation(),
            Orientation::Forward | Orientation::Reversed
        ) {
            Builder.add(&mut wire, &child)?;
        }
    }
    if wire.is_null() {
        return Ok(None);
    }
    Ok(Some(wire))
}

fn classify_edge_endpoints(
    classifier: &BRepTopAdaptorFClass2d,
    support: &Face,
    wire: &Shape,
) -> Result<State, Failure> {
    for edge in wire_edges(wire)? {
        let Some(point) = edge_sample_point(&edge, support, true)? else {
            continue;
        };
        let state = classifier.perform(point, false)?;
        if !matches!(state, State::On | State::Unknown) {
            return Ok(state);
        }
    }
    Ok(State::Unknown)
}

fn wire_edges(wire: &Shape) -> Result<Vec<Edge>, Failure> {
    let mut edges = Vec::new();
    for child in ShapeIterator::new(wire, false, false) {
        if child.shape_type()? != ShapeEnum::Edge || !is_manifold_orientation(child.orientation()) {
            continue;
        }
        edges.push(topods::edge(&child)?);
    }
    Ok(edges)
}

fn is_manifold_orientation(orientation: Orientation) -> bool {
    matches!(orientation, Orientation::Forward | Orientation::Reversed)
}

fn wire_sample_point(wire: &Wire, support: &Face) -> Result<Option<Point2>, Failure> {
    for edge in wire_edges(&wire.clone().into_shape())? {
        if let Some(point) = edge_sample_point(&edge, support, false)? {
            return Ok(Some(point));
        }
    }
    Ok(None)
}

fn edge_sample_point(edge: &Edge, support: &Face, end: bool) -> Result<Option<Point2>, Failure> {
    let (curve, first, last) = ShapeAnalysisEdge::new().pcurve_on_face(edge, support, true)?;
    let Some(curve) = curve.get() else {
        return Ok(None);
    };
    let parameter = if end { last } else { 0.5 * (first + last) };
    Ok(Some(curve.value(parameter)?))
}

fn recover_lost_roots(
    loops: &mut Sequence<Shape>,
    mut roots: Vec<Shape>,
) -> Result<Vec<Shape>, Failure> {
    if !roots.is_empty() {
        return Ok(roots);
    }
    roots.extend(loops.iter().cloned());
    loops.clear(None);
    Ok(roots)
}

fn assemble_root_faces(
    surface: &Handle<dyn GeomSurface>,
    loops: &mut Sequence<Shape>,
    roots: &[Shape],
    original_face: &Face,
    location: &Location,
    context: &Handle<ShapeBuildReShape>,
) -> Result<Vec<Face>, Failure> {
    let mut faces = Vec::new();
    remove_root_loops(loops, roots)?;
    for root in roots {
        let mut face = build_root_face(surface, root, location)?;
        let reverse = root_needs_reversal(&face)?;
        let holes = collect_root_holes(&face, loops, root, reverse, surface, location)?;
        add_holes(&mut face, &holes, original_face, context)?;
        faces.push(face);
    }
    Ok(faces)
}

fn remove_root_loops(loops: &mut Sequence<Shape>, roots: &[Shape]) -> Result<(), Failure> {
    let mut index = 1;
    while index <= loops.len() {
        let shape = loops.value(index)?;
        if roots.iter().any(|root| root.is_same(shape)) {
            loops.remove(index)?;
        } else {
            index += 1;
        }
    }
    Ok(())
}

fn build_root_face(
    surface: &Handle<dyn GeomSurface>,
    root: &Shape,
    location: &Location,
) -> Result<Face, Failure> {
    if root.shape_type()? != ShapeEnum::Wire {
        return Err(Failure::with_message(
            "Standard_TypeMismatch",
            &format!(
                "ShapeFix_ComposeShell::MakeFacesOnPatch: root {:?}; expected wire",
                root.shape_type()?
            ),
        ));
    }
    let mut face = support_face(surface, location)?;
    Builder.add(&mut face, root)?;
    Ok(face)
}

fn root_needs_reversal(face: &Face) -> Result<bool, Failure> {
    let classifier = BRepTopAdaptorFClass2d::new(face, precision::p_confusion())?;
    Ok(classifier.perform_infinite_point()? == State::In)
}

fn collect_root_holes(
    face: &Face,
    loops: &mut Sequence<Shape>,
    root: &Shape,
    reverse: bool,
    surface: &Handle<dyn GeomSurface>,
    location: &Location,
) -> Result<Vec<Shape>, Failure> {
    let classifier = BRepTopAdaptorFClass2d::new(face, precision::p_confusion())?;
    collect_holes_for_root(loops, root, reverse, &classifier, surface, location)
}

fn collect_holes_for_root(
    loops: &mut Sequence<Shape>,
    root: &Shape,
    reverse: bool,
    classifier: &BRepTopAdaptorFClass2d,
    surface: &Handle<dyn GeomSurface>,
    location: &Location,
) -> Result<Vec<Shape>, Failure> {
    let mut holes = Vec::new();
    let mut index = 1;
    while index <= loops.len() {
        if take_hole(
            loops, index, root, classifier, surface, location, reverse, &mut holes,
        )? {
            continue;
        }
        index += 1;
    }
    Ok(holes)
}

#[allow(clippy::too_many_arguments)]
fn take_hole(
    loops: &mut Sequence<Shape>,
    index: usize,
    root: &Shape,
    classifier: &BRepTopAdaptorFClass2d,
    surface: &Handle<dyn GeomSurface>,
    location: &Location,
    reverse: bool,
    holes: &mut Vec<Shape>,
) -> Result<bool, Failure> {
    let shape = loops.value(index)?.clone();
    if shape.is_same(root) || !is_hole_of_root(&shape, classifier, surface, location, reverse)? {
        return Ok(false);
    }
    holes.push(shape);
    loops.remove(index)?;
    Ok(true)
}

fn is_hole_of_root(
    shape: &Shape,
    classifier: &BRepTopAdaptorFClass2d,
    surface: &Handle<dyn GeomSurface>,
    location: &Location,
    reverse: bool,
) -> Result<bool, Failure> {
    let Some(point) = shape_sample_point(shape, surface, location)? else {
        return Ok(false);
    };
    let state = classifier.perform(point, false)?;
    Ok((state == State::Out) == reverse)
}

fn shape_sample_point(
    shape: &Shape,
    surface: &Handle<dyn GeomSurface>,
    location: &Location,
) -> Result<Option<Point2>, Failure> {
    if shape.shape_type()? == ShapeEnum::Wire {
        return wire_sample_point(&topods::wire(shape)?, &support_face(surface, location)?);
    }
    if shape.shape_type()? != ShapeEnum::Vertex {
        return Ok(None);
    }
    let point = BRepTool::pnt(&topods::vertex(shape)?)?;
    let support = support_face(surface, location)?;
    let placed_surface = BRepTool::surface(&support)?;
    let analysis = ShapeAnalysisSurface::new(&placed_surface)?;
    Ok(Some(analysis.value_of_uv(point, precision::confusion())?))
}

fn add_holes(
    face: &mut Face,
    holes: &[Shape],
    original_face: &Face,
    context: &Handle<ShapeBuildReShape>,
) -> Result<(), Failure> {
    for hole in holes {
        add_hole(face, hole, original_face, context)?;
    }
    Ok(())
}

fn add_hole(
    face: &mut Face,
    hole: &Shape,
    original_face: &Face,
    context: &Handle<ShapeBuildReShape>,
) -> Result<(), Failure> {
    if hole.shape_type()? != ShapeEnum::Vertex {
        return Builder.add(face, hole);
    }
    let source = topods::vertex(hole)?;
    let vertex =
        ShapeAnalysisTransferParametersProj::copy_nm_vertex_face(&source, face, original_face)?;
    replace_vertex(context, hole, &vertex)?;
    Builder.add(face, &vertex.into_shape())
}

fn replace_vertex(
    context: &Handle<ShapeBuildReShape>,
    original: &Shape,
    replacement: &Vertex,
) -> Result<(), Failure> {
    let context = context.get().ok_or_else(null_context)?;
    rustycad_tkbrep::brep_tools::ReShape::replace(
        context,
        original,
        &replacement.clone().into_shape(),
    )
}

fn null_context() -> Failure {
    Failure::with_message(
        "Standard_NullObject",
        "ShapeFix_ComposeShell::MakeFacesOnPatch: null context for an internal vertex",
    )
}

#[cfg(test)]
#[path = "make_faces_on_patch/tests.rs"]
mod tests;
