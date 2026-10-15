// Rust translation of the static BuildDomains of
// ModelingAlgorithms/TKOffset/BRepOffsetAPI/BRepOffsetAPI_MakeOffset.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The domains of `BRepOffsetAPI_MakeOffset`: the faces bounded by the closed wires,
//! each holding the open wires inside it, and one offset algorithm per domain.
use rustycad_tkbool::brep_algo::BRepAlgoFaceRestrictor;
use rustycad_tkbool::brep_fill::BRepFillOffsetWire;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Face, Shape, Wire};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{self, ShapeEnum, State};
use rustycad_tkgeombase::extrema::{ExtAlgo, ExtFlag, ExtPS};
use rustycad_tkmath::geom_abs::JoinType;
use rustycad_tkmath::gp::Point2;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeFace;
use rustycad_tktopalgo::brep_top_adaptor::BRepTopAdaptorFClass2d;

/// The settings `BuildDomains` passes to each offset algorithm, and the side.
#[derive(Clone, Copy)]
pub(super) struct OffsetDomainOptions {
    pub(super) join: JoinType,
    pub(super) is_open_result: bool,
    pub(super) is_positive: bool,
}

/// `BuildDomains(myFace, WorkWires, myAlgos, ...)`: the face of the first wire when
/// none is given (StdFail_NotDone for a non-planar wire), the wires reversed when
/// their orientation and the side require it, and one `BRepFill_OffsetWire` per
/// domain appended to `algos` (one domain of all open wires without a closed wire);
/// returns whether the wires were reversed.
pub(super) fn build_domains(
    my_face: &mut Face,
    work_wires: &mut [Shape],
    algos: &mut Vec<BRepFillOffsetWire>,
    options: OffsetDomainOptions,
) -> Result<bool, Failure> {
    if my_face.is_null() {
        *my_face = face_of_first_wire(work_wires)?;
    }
    let is_was_reversed = reverse_wires_for_side(my_face, work_wires, options.is_positive)?;
    let (faces, mut open_wires) = restricted_faces(my_face, work_wires)?;
    if faces.is_empty() {
        let face = face_of_open_wires(my_face, &open_wires)?;
        algos.push(BRepFillOffsetWire::new(
            &face,
            options.join,
            options.is_open_result,
        )?);
        return Ok(is_was_reversed);
    }
    for face in &faces {
        classify_open_wires(face, &mut open_wires)?;
    }
    for face in &faces {
        algos.push(BRepFillOffsetWire::new(
            face,
            options.join,
            options.is_open_result,
        )?);
    }
    Ok(is_was_reversed)
}

/// The plane face of the first wire; StdFail_NotDone when it is not planar.
fn face_of_first_wire(work_wires: &[Shape]) -> Result<Face, Failure> {
    let first = work_wires.first().ok_or_else(no_wire)?;
    let face = BRepBuilderAPIMakeFace::from_wire(&topods::wire(first)?, true)?.face()?;
    if face.is_null() {
        return Err(Failure::with_message(
            "StdFail_NotDone",
            "BRepOffsetAPI_MakeOffset : the wire is not planar",
        ));
    }
    Ok(face)
}

/// Reverses every wire when the first one has the orientation of the first wire of
/// the face on the positive side, or the opposite one on the negative side.
fn reverse_wires_for_side(
    my_face: &Face,
    work_wires: &mut [Shape],
    is_positive: bool,
) -> Result<bool, Failure> {
    let wire1 = work_wires.first().ok_or_else(no_wire)?.orientation();
    let explorer = Explorer::new(my_face.as_ref(), ShapeEnum::Wire, ShapeEnum::Shape)?;
    if !explorer.more() {
        return Ok(false);
    }
    let wire2 = explorer.current().orientation();
    let same = wire1 == wire2 && is_positive;
    let opposite = wire1 == top_abs::complement(wire2) && !is_positive;
    if !(same || opposite) {
        return Ok(false);
    }
    for wire in work_wires.iter_mut() {
        *wire = wire.reversed();
    }
    Ok(true)
}

/// The faces a `BRepAlgo_FaceRestrictor` builds on the face from the closed wires,
/// and the open wires; StdFail_NotDone when the restrictor fails.
fn restricted_faces(
    my_face: &Face,
    work_wires: &[Shape],
) -> Result<(Vec<Face>, Vec<Wire>), Failure> {
    let mut restrictor = BRepAlgoFaceRestrictor::new();
    restrictor.init(my_face, true, false);
    let mut open_wires = Vec::new();
    for shape in work_wires {
        let wire = topods::wire(shape)?;
        if is_closed_wire(&wire)? {
            restrictor.add(&wire);
        } else {
            open_wires.push(wire);
        }
    }
    restrictor.perform()?;
    if !restrictor.is_done() {
        return Err(Failure::with_message(
            "StdFail_NotDone",
            "BRepOffsetAPI_MakeOffset : Build Domains",
        ));
    }
    let mut faces = Vec::new();
    while restrictor.more() {
        faces.push(restrictor.current()?);
        restrictor.next()?;
    }
    Ok((faces, open_wires))
}

/// A wire flagged closed, or whose end vertices are the same.
fn is_closed_wire(wire: &Wire) -> Result<bool, Failure> {
    if wire.closed()? {
        return Ok(true);
    }
    let (first, last) = topexp::wire_vertices(wire);
    Ok(first.is_same(&last))
}

/// The single domain without closed wire: an empty copy of the face holding the
/// open wires.
fn face_of_open_wires(my_face: &Face, open_wires: &[Wire]) -> Result<Face, Failure> {
    let mut face = topods::face(&my_face.empty_copied()?)?;
    let builder = BRepBuilder::default();
    for wire in open_wires {
        builder.add(&mut face, wire.as_ref())?;
    }
    Ok(face)
}

/// Adds to the face (and removes from the list) each open wire whose first vertex
/// projects inside it.
fn classify_open_wires(face: &Face, open_wires: &mut Vec<Wire>) -> Result<(), Failure> {
    let surface = BRepAdaptorSurface::with_face(face, false)?;
    let tolerance = BRepTool::face_tolerance(face)?;
    let classifier = BRepTopAdaptorFClass2d::new(face, precision::confusion())?;
    let builder = BRepBuilder::default();
    let mut index = 0;
    while index < open_wires.len() {
        let projection = nearest_uv(&open_wires[index], &surface, tolerance)?;
        let inside = match projection {
            Some(uv) => classifier.perform(uv, true)? == State::In,
            None => false,
        };
        if inside {
            let wire = open_wires.remove(index);
            let mut domain = face.clone();
            builder.add(&mut domain, wire.as_ref())?;
        } else {
            index += 1;
        }
    }
    Ok(())
}

/// The UV point of the nearest projection of the first vertex of the wire on the
/// surface, when the extrema find one.
fn nearest_uv(
    wire: &Wire,
    surface: &BRepAdaptorSurface,
    tolerance: f64,
) -> Result<Option<Point2>, Failure> {
    let vertices = Explorer::new(wire.as_ref(), ShapeEnum::Vertex, ShapeEnum::Shape)?;
    let vertex = topods::vertex(vertices.current())?;
    let point = BRepTool::pnt(&vertex)?;
    let extrema = ExtPS::with_surface(
        point,
        surface,
        tolerance,
        tolerance,
        ExtFlag::Min,
        ExtAlgo::Grad,
    )?;
    let mut min_distance = precision::infinite();
    let mut nearest = None;
    for index in 1..=extrema.nb_ext()? {
        let distance = extrema.square_distance(index)?;
        if distance < min_distance {
            min_distance = distance;
            let (u, v) = extrema.point(index)?.parameter();
            nearest = Some(Point2::new(u, v));
        }
    }
    Ok(nearest)
}

/// Standard_NoSuchObject of `WorkWires.First()` on an empty list.
fn no_wire() -> Failure {
    Failure::with_message(
        "Standard_NoSuchObject",
        "BRepOffsetAPI_MakeOffset : no wire to offset; expected at least one wire",
    )
}
