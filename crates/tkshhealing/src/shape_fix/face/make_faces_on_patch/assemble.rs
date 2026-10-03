//! Root faces, hole assignment, and context updates for one patch.
use super::super::make_faces_on_patch::support_face;
use super::roots::is_manifold_wire;
use crate::shape_analysis::{ShapeAnalysisSurface, ShapeAnalysisTransferParametersProj};
use crate::shape_build::ShapeBuildReShape;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{self, Builder, Edge, Face, Shape, ShapeIterator, Vertex, Wire};
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkg3d::top_abs::{ShapeEnum, State};
use rustycad_tkmath::gp::Point2;
use rustycad_tkmath::toploc::Location;
use rustycad_tktopalgo::brep_top_adaptor::BRepTopAdaptorFClass2d;

pub(super) fn assemble_root_faces(
    surface: &Handle<dyn GeomSurface>,
    loops: &mut Sequence<Shape>,
    roots: Vec<Shape>,
    original_face: &Face,
    location: &Location,
    context: &Handle<ShapeBuildReShape>,
) -> Result<Vec<Face>, Failure> {
    let assembly = RootFaceAssembly {
        surface,
        original_face,
        location,
        context,
    };
    append_root_faces(&assembly, loops, roots)
}

struct RootFaceAssembly<'a> {
    surface: &'a Handle<dyn GeomSurface>,
    original_face: &'a Face,
    location: &'a Location,
    context: &'a Handle<ShapeBuildReShape>,
}

fn append_root_faces(
    assembly: &RootFaceAssembly<'_>,
    loops: &mut Sequence<Shape>,
    mut roots: Vec<Shape>,
) -> Result<Vec<Face>, Failure> {
    let mut faces = Vec::new();
    remove_root_loops(loops, &roots)?;
    let mut root_index = 0;
    while root_index < roots.len() {
        faces.push(assemble_one_root(assembly, loops, &roots[root_index])?);
        root_index += 1;
        if root_index == roots.len() && !loops.is_empty() {
            append_leftover_roots(loops, &mut roots)?;
        }
    }
    Ok(faces)
}

fn assemble_one_root(
    assembly: &RootFaceAssembly<'_>,
    loops: &mut Sequence<Shape>,
    root: &Shape,
) -> Result<Face, Failure> {
    let mut face = build_root_face(assembly.surface, root, assembly.location)?;
    let reverse = root_needs_reversal(&face)?;
    let holes = collect_root_holes(
        &face,
        loops,
        root,
        reverse,
        assembly.surface,
        assembly.location,
    )?;
    add_holes(&mut face, &holes, assembly.original_face, assembly.context)?;
    Ok(face)
}

pub(super) fn append_leftover_roots(
    loops: &mut Sequence<Shape>,
    roots: &mut Vec<Shape>,
) -> Result<(), Failure> {
    for shape in loops.iter() {
        if shape.shape_type()? == ShapeEnum::Wire && is_manifold_wire(shape)? {
            roots.push(shape.clone());
        }
    }
    loops.clear(None);
    Ok(())
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
        return first_wire_sample_point(&topods::wire(shape)?, &support_face(surface, location)?);
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

fn first_wire_sample_point(wire: &Wire, support: &Face) -> Result<Option<Point2>, Failure> {
    let iterator = ShapeIterator::new(&wire.clone().into_shape(), true, true);
    if !iterator.more() {
        return Ok(None);
    }
    edge_sample_point(&topods::edge(iterator.value()?)?, support)
}

fn edge_sample_point(edge: &Edge, support: &Face) -> Result<Option<Point2>, Failure> {
    let found = BRepTool::curve_on_face(edge, support)?;
    let Some(curve) = found.pcurve.get() else {
        return Ok(None);
    };
    Ok(Some(curve.value(0.5 * (found.first + found.last))?))
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
