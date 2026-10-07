//! `BOPAlgo_BuilderFace::PerformAreas`: draft faces from the loops, holes put into the
//! closest growth face containing them.
use super::BuilderFace;
use crate::bop_tools::{BoxSelector, BoxSet};
use crate::int_tools::Context;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::add_uv_bounds;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{Face, Shape, ShapeIterator};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{IndexedDataMap, IndexedMap, List};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::bnd::{BoundingBox2, BoundingBox3, box2_to_bvh};
use rustycad_tkmath::bvh::{BvhSetTraverse, BvhVec2d};
use rustycad_tkmath::toploc::Location;
use rustycad_tktopalgo::brep_bnd_lib::BRepBndLib;

/// The surface, location and tolerance every new face is made on.
struct FaceSupport {
    surface: Handle<dyn GeomSurface>,
    location: Location,
    tolerance: f64,
}

impl FaceSupport {
    fn make_face(&self) -> Result<Face, Failure> {
        let mut face = Face::default();
        BRepBuilder::default().make_face_on_located_surface(
            &mut face,
            &self.surface,
            &self.location,
            self.tolerance,
        )?;
        Ok(face)
    }
}

/// Holes and the growth face chosen for each (`aHoleFaceMap`).
type HoleFaces = IndexedDataMap<Shape, Shape, ShapeMapHasher>;
/// Growth faces and their holes (`aFaceHolesMap`).
type FaceHoles = IndexedDataMap<Shape, List<Shape>, ShapeMapHasher>;

impl BuilderFace {
    /// Builds the draft faces from the loops (`PerformAreas`).
    pub(super) fn perform_areas(&mut self) -> Result<(), Failure> {
        self.area.areas.clear();
        let (surface, location) = BRepTool::surface_with_location(&self.face)?;
        let support = FaceSupport {
            surface,
            location,
            tolerance: BRepTool::face_tolerance(&self.face)?,
        };
        let handle = self.area.context.clone();
        let context = super::required_context(&handle)?;
        if self.area.loops.is_empty() {
            if context.is_infinite_face(&self.face)? {
                let face = support.make_face()?;
                if BRepTool::natural_restriction(&self.face)? {
                    BRepBuilder::default().natural_restriction(&face, true)?;
                }
                self.area.areas.append(face.into_shape());
            }
            return Ok(());
        }
        let (mut growths, holes) = classify_loops(&self.area.loops, &support, context)?;
        if holes.is_empty() {
            for face in growths {
                self.area.areas.append(face.into_shape());
            }
            return Ok(());
        }
        let hole_faces = closest_growths(&growths, &holes, context)?;
        let mut face_holes = holes_by_face(&hole_faces)?;
        if holes.extent() != hole_faces.extent() {
            self.add_unused_holes(&support, &holes, &hole_faces, &mut face_holes, &mut growths)?;
        }
        for face in growths {
            if let Some(face_hole_list) = face_holes.seek(&face) {
                add_holes(&face, face_hole_list, support.tolerance, context)?;
            }
            // The face is a draft that does not contain any internal shapes.
            self.area.areas.append(face.into_shape());
        }
        Ok(())
    }

    /// Holes contained in no growth face go into a face on the whole surface when the face
    /// is unbounded.
    fn add_unused_holes(
        &self,
        support: &FaceSupport,
        holes: &IndexedMap<Shape, ShapeMapHasher>,
        hole_faces: &HoleFaces,
        face_holes: &mut FaceHoles,
        growths: &mut Vec<Face>,
    ) -> Result<(), Failure> {
        let mut bounds = BoundingBox3::default();
        BRepBndLib::add(&self.face, &mut bounds, true)?;
        if !(bounds.is_open_xmin()
            || bounds.is_open_xmax()
            || bounds.is_open_ymin()
            || bounds.is_open_ymax()
            || bounds.is_open_zmin()
            || bounds.is_open_zmax())
        {
            return Ok(());
        }
        let face = support.make_face()?;
        let mut unused = List::default();
        for hole in holes.iter() {
            if !hole_faces.contains(hole) {
                unused.append(hole.clone());
            }
        }
        face_holes.add(face.clone().into_shape(), unused);
        growths.push(face);
        Ok(())
    }
}

/// Splits the loops into growth faces and hole faces: a loop sharing an edge with a hole is
/// a growth, the others are classified by the 2D classifier.
fn classify_loops(
    loops: &List<Shape>,
    support: &FaceSupport,
    context: &Context,
) -> Result<(Vec<Face>, IndexedMap<Shape, ShapeMapHasher>), Failure> {
    let mut growths = Vec::new();
    let mut holes: IndexedMap<Shape, ShapeMapHasher> = IndexedMap::with_hasher(ShapeMapHasher, 1);
    let mut hole_edges: IndexedMap<Shape, ShapeMapHasher> =
        IndexedMap::with_hasher(ShapeMapHasher, 1);
    for wire in loops.iter() {
        let mut face = support.make_face()?.into_shape();
        BRepBuilder::default().add(&mut face, wire)?;
        let face = Face::unchecked(face);
        let mut is_growth = is_growth_wire(wire, &hole_edges);
        if !is_growth {
            // The fast check gave no result: run the classification.
            is_growth = !context.fclass2d(&face)?.read().is_hole();
        }
        if is_growth {
            growths.push(face);
        } else {
            holes.add(face.into_shape());
            topexp::map_shapes_of_type(wire, ShapeEnum::Edge, &mut hole_edges)?;
        }
    }
    Ok((growths, holes))
}

/// Whether the wire shares an edge with a hole face (`IsGrowthWire`).
fn is_growth_wire(wire: &Shape, hole_edges: &IndexedMap<Shape, ShapeMapHasher>) -> bool {
    hole_edges.extent() > 0
        && ShapeIterator::new(wire, true, true).any(|edge| hole_edges.contains(&edge))
}

/// For each hole, the innermost growth face containing it, found among the growths whose
/// UV box meets the hole box.
fn closest_growths(
    growths: &[Face],
    holes: &IndexedMap<Shape, ShapeMapHasher>,
    context: &Context,
) -> Result<HoleFaces, Failure> {
    let mut tree: BoxSet<BvhVec2d> = BoxSet::default();
    tree.set_size(holes.extent());
    for (position, hole) in holes.iter().enumerate() {
        let mut bounds = BoundingBox2::default();
        add_uv_bounds(&Face::unchecked(hole.clone()), &mut bounds)?;
        tree.add(position as i32 + 1, box2_to_bvh(&bounds)?);
    }
    tree.build()?;
    let mut hole_faces: HoleFaces = IndexedDataMap::with_hasher(ShapeMapHasher, 1);
    for face in growths {
        let mut bounds = BoundingBox2::default();
        add_uv_bounds(face, &mut bounds)?;
        let mut selector = BoxSelector::default();
        selector.set_box(box2_to_bvh(&bounds)?);
        selector.set_bvh_set(Some(&mut tree));
        selector.select()?;
        for index in selector.indices().iter() {
            let hole = holes.find_key(*index as usize)?;
            if !super::is_inside(hole, face, context)? {
                continue;
            }
            if let Some(was) = hole_faces.change_seek(hole) {
                if super::is_inside(face, was, context)? {
                    *was = face.clone().into_shape();
                }
                continue;
            }
            hole_faces.add(hole.clone(), face.clone().into_shape());
        }
    }
    Ok(hole_faces)
}

/// The back map from growth faces to their holes.
fn holes_by_face(hole_faces: &HoleFaces) -> Result<FaceHoles, Failure> {
    let mut face_holes: FaceHoles = IndexedDataMap::with_hasher(ShapeMapHasher, 1);
    for index in 1..=hole_faces.extent() {
        let hole = hole_faces.find_key(index)?.clone();
        let face = hole_faces.find_from_index(index)?;
        if let Some(list) = face_holes.change_seek(face) {
            list.append(hole);
            continue;
        }
        let mut list = List::default();
        list.append(hole);
        face_holes.add(face.clone(), list);
    }
    Ok(face_holes)
}

/// Adds the wire of each hole face to `face` and updates its cached classifier.
fn add_holes(
    face: &Face,
    holes: &List<Shape>,
    tolerance: f64,
    context: &Context,
) -> Result<(), Failure> {
    let mut shape = face.clone().into_shape();
    for hole in holes.iter() {
        // The hole face contains only one wire.
        if let Some(wire) = Iterator::next(&mut ShapeIterator::new(hole, true, true)) {
            BRepBuilder::default().add(&mut shape, &wire)?;
        }
    }
    context.fclass2d(face)?.write().init(face, tolerance)
}
