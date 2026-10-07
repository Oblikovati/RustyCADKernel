//! The first part of `IntUnifyFaces` (ShapeUpgrade_UnifySameDomain.cxx): the faces of
//! the same domain grown from one face across its manifold edges, the faces with kept or
//! multi-connected edges dropped (or those edges made internal), and the boundary edges
//! oriented as in the faces.
use super::face_bounds::{add_ordinary_edges, edges_of};
use super::geometry::get_normal_to_surface;
use super::pcurve_transform::add_pcurves;
use super::same_domain::is_same_domain;
use super::{DataMapOfShapeMapOfShape, UnifyState};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Face, Shape};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, IndexedMapOfShape, MapOfShape};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::toploc::Location;
use rustycad_tktopalgo::brep_lib;

/// The maps of the whole shape that `IntUnifyFaces` reads.
#[derive(Clone, Copy)]
pub(super) struct GlobalMaps<'a> {
    pub edge_faces: &'a IndexedDataMapOfShapeListOfShape,
    pub face_shells: &'a DataMapOfShapeMapOfShape,
    pub free_bounds: &'a MapOfShape,
}

/// The faces to merge with one face and the boundary of their union.
pub(super) struct FaceGroup {
    pub faces: Vec<Shape>,
    pub edges: Vec<Shape>,
    pub removed: Vec<Shape>,
    pub internal: IndexedMapOfShape,
    pub ref_face: Face,
    pub ref_orientation: Orientation,
    pub base_surface: Handle<dyn GeomSurface>,
    pub temporary: MapOfShape,
    pub u_period: f64,
    pub v_period: f64,
}

impl FaceGroup {
    /// The group of one face: its ordinary edges and the reference face on the basis
    /// surface (identity location, the orientation of the face).
    fn new(face: &Face, base_surface: Handle<dyn GeomSurface>) -> Result<Self, Failure> {
        let (mut edges, mut removed, mut dummy) = (Vec::new(), Vec::new(), 0);
        add_ordinary_edges(&mut edges, face.as_ref(), &mut dummy, &mut removed)?;
        let mut ref_face = Face::default();
        BRepBuilder::default().make_face_on_located_surface(
            &mut ref_face,
            &base_surface,
            &Location::default(),
            0.,
        )?;
        ref_face.set_orientation(face.orientation());
        let surface = base_surface
            .get()
            .ok_or_else(super::geometry::null_surface)?;
        let u_period = if surface.is_u_periodic() {
            surface.u_period()?
        } else {
            0.
        };
        let v_period = if surface.is_v_periodic() {
            surface.v_period()?
        } else {
            0.
        };
        Ok(Self {
            faces: vec![face.clone().into_shape()],
            edges,
            removed,
            internal: IndexedMapOfShape::default(),
            ref_face,
            ref_orientation: face.orientation(),
            base_surface,
            temporary: MapOfShape::default(),
            u_period,
            v_period,
        })
    }

    /// The reference face taken forward (`F_RefFace`).
    pub fn forward_ref_face(&self) -> Face {
        let mut face = self.ref_face.clone();
        face.set_orientation(Orientation::Forward);
        face
    }
}

/// `isSameSets`: both null, or both with the same shells.
fn is_same_sets(first: Option<&MapOfShape>, second: Option<&MapOfShape>) -> bool {
    match (first, second) {
        (None, None) => true,
        (Some(a), Some(b)) => {
            a.extent() == b.extent()
                && a.iter().all(|s| b.contains(s))
                && b.iter().all(|s| a.contains(s))
        }
        _ => false,
    }
}

impl UnifyState {
    /// The adjacent faces of `face` that can be merged with it, collected with the
    /// boundary edges of the union (the 'find adjacent faces to union' loop).
    pub(super) fn collect_face_group(
        &mut self,
        face: &Face,
        base_surface: Handle<dyn GeomSurface>,
        maps: GlobalMaps<'_>,
        local_edge_faces: &IndexedDataMapOfShapeListOfShape,
        processed: &mut MapOfShape,
    ) -> Result<FaceGroup, Failure> {
        let mut group = FaceGroup::new(face, base_surface)?;
        let shells = maps.face_shells.seek(face.as_ref());
        let mut i = 1;
        while i <= group.edges.len() {
            let edge = group.edges[i - 1].clone();
            if let Some(next) = self.grow_across(
                face,
                &edge,
                (shells, maps),
                local_edge_faces,
                processed,
                &mut group,
            )? {
                i = next;
            }
            i += 1;
        }
        Ok(group)
    }

    /// Whether `edge` is a manifold edge across which the group may grow.
    fn is_growth_edge(&self, edge: &Shape, maps: GlobalMaps<'_>) -> Result<bool, Failure> {
        if BRepTool::degenerated(&topods::edge(edge)?)? {
            return Ok(false);
        }
        let global = maps.edge_faces.find_from_key(edge)?;
        let blocked = global.extent() != 2
            || self.keep_shapes.contains(edge)
            || maps.free_bounds.contains(edge);
        Ok(self.allow_internal || !blocked)
    }

    /// The first unprocessed face across `edge` of the same domain (with compatible
    /// shells and normals) added to the group; the new 1-based loop index when the edge
    /// sequence was shortened.
    fn grow_across(
        &mut self,
        face: &Face,
        edge: &Shape,
        (shells, maps): (Option<&MapOfShape>, GlobalMaps<'_>),
        local_edge_faces: &IndexedDataMapOfShapeListOfShape,
        processed: &mut MapOfShape,
        group: &mut FaceGroup,
    ) -> Result<Option<usize>, Failure> {
        if !self.is_growth_edge(edge, maps)? {
            return Ok(None);
        }
        let list: Vec<Shape> = local_edge_faces
            .find_from_key(edge)?
            .iter()
            .cloned()
            .collect();
        if list.len() < 2 {
            return Ok(None);
        }
        let e = topods::edge(edge)?;
        let is_plane = !group.base_surface.cast::<GeomPlane>().is_null();
        if !self.safe_input_mode && is_plane {
            brep_lib::build_pcurve_for_edge_on_plane(&e, face)?;
        }
        let (first, last) = BRepTool::range(&e)?;
        let t_mid = (first + last) * 0.5;
        let normal = get_normal_to_surface(face, &e, t_mid)?;
        for shape in &list {
            let checked = topods::face(shape)?;
            if checked.is_same(face) || processed.contains(shape) {
                continue;
            }
            if !is_same_sets(shells, maps.face_shells.seek(shape)) {
                continue;
            }
            if let Some(n1) = normal
                && let Some(n2) = get_normal_to_surface(&checked, &e, t_mid)?
                && n1.angle(&n2) > self.ang_tol
            {
                continue;
            }
            if !is_same_domain(
                face,
                &checked,
                (self.lin_tol, self.ang_tol),
                &mut self.face_plane_map,
            )? {
                continue;
            }
            let mut dummy = 0;
            let changed =
                add_ordinary_edges(&mut group.edges, shape, &mut dummy, &mut group.removed)?;
            group.faces.push(shape.clone());
            processed.add(shape.clone());
            return Ok(changed.then_some(dummy));
        }
        Ok(None)
    }

    /// The `if (faces.Length() > 1)` block after the collection: the shared plane on the
    /// reference face, the p-curves of the other faces on it, and the kept or
    /// multi-connected edges between the faces.
    pub(super) fn prepare_face_group(
        &mut self,
        group: &mut FaceGroup,
        maps: GlobalMaps<'_>,
    ) -> Result<(), Failure> {
        if let Some(plane) = self.face_plane_map.seek(&group.faces[0]) {
            let plane: Handle<dyn GeomSurface> = plane.clone().cast();
            BRepBuilder::default().update_face(
                &group.ref_face,
                &plane,
                &Location::default(),
                precision::confusion(),
            )?;
        }
        add_pcurves(
            &group.faces,
            &group.forward_ref_face(),
            &mut group.temporary,
        )?;
        let mut map_ef = IndexedDataMapOfShapeListOfShape::default();
        for face in &group.faces {
            topexp::map_shapes_and_ancestors(face, ShapeEnum::Edge, ShapeEnum::Face, &mut map_ef)?;
        }
        let keep_edges = self.keep_edges_of(&map_ef, maps)?;
        if keep_edges.is_empty() {
            return Ok(());
        }
        if self.allow_internal {
            // "add multi-connected and keep edges as internal in new face".
            group
                .edges
                .extend(keep_edges.iter().map(|e| e.oriented(Orientation::Internal)));
            return Ok(());
        }
        drop_isolated_faces(group, &keep_edges, &map_ef)?;
        drop_faces_around_keep_edges(group, &keep_edges, &map_ef)
    }

    /// The edges shared by two of the faces that are kept, free bounds or connected to
    /// other faces.
    fn keep_edges_of(
        &self,
        map_ef: &IndexedDataMapOfShapeListOfShape,
        maps: GlobalMaps<'_>,
    ) -> Result<Vec<Shape>, Failure> {
        let mut keep = Vec::new();
        for i in 1..=map_ef.extent() {
            if map_ef.find_from_index(i)?.extent() != 2 {
                continue;
            }
            let edge = map_ef.find_key(i)?;
            let global = maps.edge_faces.find_from_key(edge)?;
            if global.extent() > 2
                || self.keep_shapes.contains(edge)
                || maps.free_bounds.contains(edge)
            {
                keep.push(edge.clone());
            }
        }
        Ok(keep)
    }
}

/// The faces of the keep edges.
fn faces_of_edges(
    edges: &[Shape],
    map_ef: &IndexedDataMapOfShapeListOfShape,
) -> Result<MapOfShape, Failure> {
    let mut faces = MapOfShape::default();
    for edge in edges {
        let list = map_ef.find_from_key(edge)?;
        faces.add(list.first().cloned().unwrap_or_default());
        faces.add(list.last().cloned().unwrap_or_default());
    }
    Ok(faces)
}

/// Whether the face has an edge shared with a face not around a keep edge.
fn connects_other_faces(
    face: &Shape,
    avoid: &MapOfShape,
    map_ef: &IndexedDataMapOfShapeListOfShape,
) -> Result<bool, Failure> {
    for edge in edges_of(face)? {
        let list = map_ef.find_from_key(&edge)?;
        if list.extent() > 1 && list.iter().any(|f| !avoid.contains(f)) {
            return Ok(true);
        }
    }
    Ok(false)
}

/// "Remove from the selection the faces which have no other connect edges and contain
/// multi-connected edges and/or keep edges."
fn drop_isolated_faces(
    group: &mut FaceGroup,
    keep: &[Shape],
    map_ef: &IndexedDataMapOfShapeListOfShape,
) -> Result<(), Failure> {
    let avoid = faces_of_edges(keep, map_ef)?;
    let mut i = 0;
    while i < group.faces.len() {
        let face = group.faces[i].clone();
        if avoid.contains(&face) && !connects_other_faces(&face, &avoid, map_ef)? {
            let mut dummy = 0;
            add_ordinary_edges(&mut group.edges, &face, &mut dummy, &mut group.removed)?;
            group.faces.remove(i);
            continue;
        }
        i += 1;
    }
    Ok(())
}

/// "check if the faces with keep edges contained in already updated the boundaries of
/// merged area": both faces of a keep edge removed from the group.
fn drop_faces_around_keep_edges(
    group: &mut FaceGroup,
    keep: &[Shape],
    map_ef: &IndexedDataMapOfShapeListOfShape,
) -> Result<(), Failure> {
    if group.faces.is_empty() {
        return Ok(());
    }
    let mut map_faces = MapOfShape::default();
    for face in &group.faces {
        map_faces.add(face.clone());
    }
    for edge in keep {
        let list = map_ef.find_from_key(edge)?;
        if list.extent() < 2 {
            continue;
        }
        let (first, last) = (
            list.first().cloned().unwrap_or_default(),
            list.last().cloned().unwrap_or_default(),
        );
        if !(map_faces.contains(&first) && map_faces.contains(&last)) {
            continue;
        }
        let mut i = 0;
        while i < group.faces.len() {
            let face = group.faces[i].clone();
            if face == first || face == last {
                let mut dummy = 0;
                add_ordinary_edges(&mut group.edges, &face, &mut dummy, &mut group.removed)?;
                group.faces.remove(i);
                continue;
            }
            i += 1;
        }
    }
    Ok(())
}

impl UnifyState {
    /// The edge orientations of the faces (internal for the kept edges between two
    /// faces when internal edges are allowed), the internal edges set apart, the edges
    /// reversed for a reversed reference face; the map of the edges of the faces to the
    /// faces is returned.
    pub(super) fn orient_group_edges(
        &self,
        group: &mut FaceGroup,
    ) -> Result<IndexedDataMapOfShapeListOfShape, Failure> {
        let mut map_ef = IndexedDataMapOfShapeListOfShape::default();
        for face in &group.faces {
            topexp::map_shapes_and_unique_ancestors(
                face,
                ShapeEnum::Edge,
                ShapeEnum::Face,
                &mut map_ef,
                false,
            )?;
        }
        for edge in group.edges.iter_mut() {
            let index = map_ef.find_index(edge);
            let faces = map_ef.find_from_index(index)?;
            if self.allow_internal && self.keep_shapes.contains(edge) && faces.extent() == 2 {
                edge.set_orientation(Orientation::Internal);
            }
            if edge.orientation() != Orientation::Internal {
                *edge = map_ef.find_key(index)?.clone();
            }
        }
        let mut i = 0;
        while i < group.edges.len() {
            if group.edges[i].orientation() == Orientation::Internal {
                group.internal.add(group.edges.remove(i));
                continue;
            }
            i += 1;
        }
        if group.ref_orientation == Orientation::Reversed {
            group.edges.iter_mut().for_each(Shape::reverse);
        }
        Ok(map_ef)
    }
}
