//! `BOPAlgo_Tools::WiresToFaces`: the planar faces bounded by a set of wires.
use super::planes::{EdgeTangents, wire_plane};
use super::wires::new_compound;
use crate::bop_algo::BuilderFace;
use crate::bop_tools::{correct_shape_tolerances, correct_tolerances};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Face, Shape, ShapeIterator};
use rustycad_tkbrep::toptools::{IndexedMapOfShape, MapOfShape, ShapeMapHasher};
use rustycad_tkernel::ncollection::{DataMap, IndexedDataMap, List};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::Plane3;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeFace;
use rustycad_tktopalgo::brep_lib::build_pcurve_for_edges_on_plane;

/// The half size of the planar face the wires split.
const FACE_EXTENT: f64 = 1.0e8;

/// The planar wires with their plane and the largest tolerance of their edges.
struct PlanarWires {
    planes: IndexedDataMap<Shape, Plane3, ShapeMapHasher>,
    tolerances: DataMap<Shape, f64, ShapeMapHasher>,
}

/// Makes faces of the planar wires: the wires of one plane (parallel normals within
/// `angular_tolerance`, planes closer than the sum of the wire tolerances) split a large
/// planar face; a plane whose split fails is skipped. The tolerances of the faces are then
/// corrected. True when a face was built (`WiresToFaces`);
/// `wires_to_faces(&wires, &mut faces, 1e-8)?`.
pub fn wires_to_faces(
    wires: &Shape,
    faces: &mut Shape,
    angular_tolerance: f64,
) -> Result<bool, Failure> {
    let planar = planar_wires(wires)?;
    let mut result = new_compound();
    let mut used = MapOfShape::default();
    for position in 1..=planar.planes.extent() {
        let wire = planar.planes.find_key(position)?.clone();
        if used.contains(&wire) {
            continue;
        }
        let group = coplanar_wires(&planar, position, angular_tolerance, &mut used)?;
        let plane = *planar.planes.find_from_index(position)?;
        // A failure of one plane skips it, as the native catch does.
        if let Ok(split) = split_plane(&plane, &group) {
            let builder = BRepBuilder::default();
            for face in split.iter() {
                builder.add(&mut result, face)?;
            }
        }
    }
    let avoided = IndexedMapOfShape::default();
    correct_tolerances(&result, &avoided, 0.05, false)?;
    correct_shape_tolerances(&result, &avoided, false)?;
    let built = Iterator::next(&mut ShapeIterator::new(&result, true, true)).is_some();
    *faces = result;
    Ok(built)
}

/// The wires of `wires` that have a plane, with their edge tolerance.
fn planar_wires(wires: &Shape) -> Result<PlanarWires, Failure> {
    let mut planar = PlanarWires {
        planes: IndexedDataMap::with_hasher(ShapeMapHasher, 1),
        tolerances: DataMap::with_hasher(ShapeMapHasher, 1),
    };
    let mut tangents: EdgeTangents = IndexedDataMap::with_hasher(ShapeMapHasher, 1);
    let mut no_unique_plane = MapOfShape::default();
    let mut explorer = Explorer::new(wires, ShapeEnum::Wire, ShapeEnum::Shape)?;
    while explorer.more() {
        let wire = explorer.current().clone();
        explorer.next()?;
        if let Some(plane) = wire_plane(&wire, &mut tangents, &mut no_unique_plane)? {
            planar.planes.add(wire.clone(), plane);
            let tolerance = BRepTool::max_tolerance(&wire, ShapeEnum::Edge)?;
            planar.tolerances.bind(wire, tolerance);
        }
    }
    Ok(planar)
}

/// The wire at `position` with the later unused wires of the same plane.
fn coplanar_wires(
    planar: &PlanarWires,
    position: usize,
    angular_tolerance: f64,
    used: &mut MapOfShape,
) -> Result<Vec<Shape>, Failure> {
    let wire = planar.planes.find_key(position)?.clone();
    let plane = planar.planes.find_from_index(position)?;
    let tolerance = *planar.tolerances.find(&wire)?;
    used.add(wire.clone());
    let mut group = vec![wire];
    for later in position + 1..=planar.planes.extent() {
        let other = planar.planes.find_key(later)?;
        if used.contains(other) {
            continue;
        }
        let other_plane = planar.planes.find_from_index(later)?;
        if !plane
            .position()
            .direction()
            .is_parallel(other_plane.position().direction(), angular_tolerance)
        {
            continue;
        }
        let distance = plane.distance(other_plane.location());
        if distance > tolerance + *planar.tolerances.find(other)? {
            continue;
        }
        used.add(other.clone());
        group.push(other.clone());
    }
    Ok(group)
}

/// The faces into which the edges of the wires (both orientations) split the large face of
/// the plane.
fn split_plane(plane: &Plane3, wires: &[Shape]) -> Result<List<Shape>, Failure> {
    let mut edges: List<Shape> = List::default();
    for wire in wires {
        for edge in ShapeIterator::new(wire, true, true) {
            edges.append(edge.oriented(Orientation::Forward));
            edges.append(edge.oriented(Orientation::Reversed));
        }
    }
    let bounds = (-FACE_EXTENT, FACE_EXTENT, -FACE_EXTENT, FACE_EXTENT);
    let maker = BRepBuilderAPIMakeFace::from_plane_bounds(*plane, bounds)?;
    let face: Face =
        rustycad_tkbrep::topods::face(&maker.face()?.into_shape().oriented(Orientation::Forward))?;
    build_pcurve_for_edges_on_plane(edges.iter(), &face)?;
    let mut builder = BuilderFace::new();
    builder.set_shapes(&edges);
    builder.set_face(&face);
    builder.perform()?;
    if builder.options().has_errors() {
        return Err(Failure::with_type(
            "Standard_Failure",
            "BOPAlgo_Tools::WiresToFaces: the face builder failed on the plane; expected a successful split",
            "",
        ));
    }
    Ok(builder.areas().clone())
}
