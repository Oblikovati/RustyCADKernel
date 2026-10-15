// Rust translation of the static PerformPlan, IsSameOriented and MakeSolid of
// ModelingAlgorithms/TKOffset/BRepOffsetAPI/BRepOffsetAPI_ThruSections.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The solid of a loft: the open ends capped by plane (or other) faces, oriented so
//! that the infinite point is outside.
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Face, Shape, ShapeIterator, Shell, Solid, Wire};
use rustycad_tkbrep::toptools::IndexedDataMapOfShapeListOfShape;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkg3d::top_abs::{ShapeEnum, State};
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIFindPlane, BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape,
};
use rustycad_tktopalgo::brep_class3d::BRepClass3dSolidClassifier;

/// `PerformPlan(W, presPln, theFace)`: true for a punctual wire (no face); else the
/// plane face of the wire when `BRepBuilderAPI_FindPlane` finds its plane, or any face
/// `BRepBuilderAPI_MakeFace` makes of it; false when none.
pub(super) fn perform_plan(wire: &Wire, pres_pln: f64, face: &mut Face) -> Result<bool, Failure> {
    if all_edges_degenerated(wire)? {
        return Ok(true);
    }
    if wire.is_null() {
        return Ok(false);
    }
    let searcher = BRepBuilderAPIFindPlane::from_shape(wire.as_ref(), pres_pln)?;
    if searcher.found() {
        let plane: Handle<dyn GeomSurface> = searcher.plane().cast();
        if plane.is_null() {
            return Err(null_plane());
        }
        *face = BRepBuilderAPIMakeFace::from_surface_wire(&plane, wire, true)?.face()?;
        return Ok(true);
    }
    let maker = BRepBuilderAPIMakeFace::from_wire(wire, false)?;
    if maker.is_done() {
        *face = maker.face()?;
        return Ok(true);
    }
    Ok(false)
}

/// Whether every child edge of the wire is degenerated (true without child).
fn all_edges_degenerated(wire: &Wire) -> Result<bool, Failure> {
    let mut children = ShapeIterator::new(wire.as_ref(), true, true);
    let mut is_degen = true;
    while children.more() {
        if !BRepTool::degenerated(&topods::edge(children.value()?)?)? {
            is_degen = false;
        }
        children.next();
    }
    Ok(is_degen)
}

/// `IsSameOriented(aFace, aShell)`: whether the first edge of the face is used by the
/// first shell face adjacent to it in the opposite orientation.
pub(super) fn is_same_oriented(face: &Shape, shell: &Shape) -> Result<bool, Failure> {
    let explorer = Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape)?;
    let an_edge = explorer.current().clone();
    let or1 = an_edge.orientation();
    let mut ef_map = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(shell, ShapeEnum::Edge, ShapeEnum::Face, &mut ef_map)?;
    let adjacent_face = ef_map
        .find_from_key(&an_edge)?
        .first()
        .cloned()
        .ok_or_else(no_adjacent_face)?;
    let mut the_edge = Shape::default();
    let mut edges = Explorer::new(&adjacent_face, ShapeEnum::Edge, ShapeEnum::Shape)?;
    while edges.more() {
        the_edge = edges.current().clone();
        if the_edge.is_same(&an_edge) {
            break;
        }
        edges.next()?;
    }
    Ok(or1 != the_edge.orientation())
}

/// `MakeSolid(shell, wire1, wire2, presPln, face1, face2)`: StdFail_NotDone for a null
/// shell; an open shell gets the cap faces of both wires (oriented with the shell)
/// when both can be made; the solid is oriented outward and flagged closed.
pub(super) fn make_solid(
    shell: &mut Shell,
    (wire1, wire2): (&Wire, &Wire),
    pres_pln: f64,
    (face1, face2): (&mut Face, &mut Face),
) -> Result<Solid, Failure> {
    if shell.is_null() {
        return Err(Failure::with_message(
            "StdFail_NotDone",
            "Thrusections is not build",
        ));
    }
    if !shell.closed()? {
        close_extremities(shell, (wire1, wire2), pres_pln, (face1, face2))?;
    }
    let solid = oriented_solid(shell)?;
    solid.set_closed(true)?;
    Ok(solid)
}

/// Caps both ends of an open shell when both cap faces can be made.
fn close_extremities(
    shell: &mut Shell,
    (wire1, wire2): (&Wire, &Wire),
    pres_pln: f64,
    (face1, face2): (&mut Face, &mut Face),
) -> Result<(), Failure> {
    if !perform_plan(wire1, pres_pln, face1)? || !perform_plan(wire2, pres_pln, face2)? {
        return Ok(());
    }
    orient_cap(face1, shell)?;
    orient_cap(face2, shell)?;
    let builder = BRepBuilder::default();
    for face in [&*face1, &*face2] {
        if !face.is_null() {
            builder.add(shell, face.as_ref())?;
        }
    }
    shell.set_closed(true)
}

/// Reverses a cap face that is not oriented as the shell.
fn orient_cap(face: &mut Face, shell: &Shell) -> Result<(), Failure> {
    if !face.is_null() && !is_same_oriented(face.as_ref(), shell.as_ref())? {
        face.reverse();
    }
    Ok(())
}

/// The solid of the shell, made of the reversed shell when the infinite point is
/// classified inside.
pub(super) fn oriented_solid(shell: &Shell) -> Result<Solid, Failure> {
    let builder = BRepBuilder::default();
    let mut solid = Solid::default();
    builder.make_solid(&mut solid);
    builder.add(&mut solid, shell.as_ref())?;
    let mut classifier = BRepClass3dSolidClassifier::with_shape(solid.as_ref())?;
    classifier.perform_infinite_point(precision::confusion())?;
    if classifier.state() == State::In {
        builder.make_solid(&mut solid);
        builder.add(&mut solid, &shell.reversed())?;
    }
    Ok(solid)
}

/// Standard_NullObject of a found plane without geometry.
fn null_plane() -> Failure {
    Failure::with_message(
        "Standard_NullObject",
        "BRepBuilderAPI_FindPlane found a null plane; expected a plane",
    )
}

/// Standard_NoSuchObject of a cap edge without adjacent shell face.
fn no_adjacent_face() -> Failure {
    Failure::with_message(
        "Standard_NoSuchObject",
        "IsSameOriented: an edge without adjacent face; expected a shell face",
    )
}
