// Rust translation of BRepExtrema_DistShapeShape::Perform, BoxCalculation and
// DistanceInitiale (ModelingAlgorithms/TKTopAlgo/BRepExtrema/BRepExtrema_DistShapeShape.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepExtrema_DistShapeShape::Perform`: the inner-solid treatment, then the nine
//! map-to-map distance steps.
use super::tasks::MapPair;
use super::{BRepExtremaDistShapeShape, ShapeMaps};
use crate::brep_bnd_lib::BRepBndLib;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods;
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::message::{ProgressRange, ProgressScope};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::bnd::BoundingBox3;

/// `BoxCalculation(Map, SBox)`: the box of each shape of the map (with triangulations).
fn box_calculation(map: &IndexedMapOfShape) -> Result<Vec<BoundingBox3>, Failure> {
    let mut boxes: Vec<BoundingBox3> = Vec::with_capacity(map.extent());
    for i in 1..=map.extent() {
        let mut bounds = BoundingBox3::default();
        BRepBndLib::add(map.find_key(i)?, &mut bounds, true)?;
        boxes.push(bounds);
    }
    Ok(boxes)
}

impl ShapeMaps {
    /// The boxes of the three maps (`myBV1`, `myBE1`, `myBF1`). An empty map keeps its
    /// former boxes: natively the array is resized only for a non-empty map.
    fn rebuild_boxes(&mut self) -> Result<(), Failure> {
        if !self.vertices.is_empty() {
            self.vertex_boxes = box_calculation(&self.vertices)?;
        }
        if !self.edges.is_empty() {
            self.edge_boxes = box_calculation(&self.edges)?;
        }
        if !self.faces.is_empty() {
            self.face_boxes = box_calculation(&self.faces)?;
        }
        Ok(())
    }
}

/// True for a solid or a compound of solids (the shapes of the inner treatment).
fn is_solid(shape: &topods::Shape) -> Result<bool, Failure> {
    Ok(matches!(
        shape.shape_type()?,
        ShapeEnum::Solid | ShapeEnum::CompSolid
    ))
}

impl BRepExtremaDistShapeShape {
    /// `Perform(theRange)`: the minimum distance and its solutions; returns `IsDone()`.
    /// False without computing for a null shape and when the progress is stopped.
    ///
    /// A solid treatment first looks for a vertex of the other shape inside a solid
    /// (distance 0, an inner solution); otherwise the boxes are rebuilt for newly loaded
    /// shapes, the reference is the distance of the first vertices (1e30 without), and
    /// the vertex-vertex, vertex-edge, edge-vertex, vertex-face, face-vertex, edge-edge,
    /// edge-face, face-edge and (unless the reference is within the deflection of 0)
    /// face-face steps lower it. Then the solutions farther than the reference plus the
    /// deflection are removed, skipping the solution after each removed one, as
    /// natively. `dist.perform(&ProgressRange::default())?`.
    pub fn perform(&mut self, range: &ProgressRange<'_>) -> Result<bool, Failure> {
        self.my_is_done = false;
        self.my_inner_sol = false;
        self.my_solutions_shape1.clear();
        self.my_solutions_shape2.clear();
        if self.my_shape1.is_null() || self.my_shape2.is_null() {
            return Ok(false);
        }
        let is_solid1: bool = is_solid(&self.my_shape1)?;
        let is_solid2: bool = is_solid(&self.my_shape2)?;
        // One step per DistanceMapMap call, plus one per solid treatment.
        let root_steps: i32 = 9 + i32::from(is_solid1) + i32::from(is_solid2);
        let root_scope = ProgressScope::with_literal(
            range,
            "calculating distance",
            f64::from(root_steps),
            false,
        );
        if is_solid1 {
            let shape = self.my_shape1.clone();
            let vertices = self.maps2.vertices.clone();
            if !self.solid_treatment(&shape, &vertices, &root_scope.next(1.0))? {
                return Ok(false);
            }
        }
        if is_solid2 && !self.my_inner_sol {
            let shape = self.my_shape2.clone();
            let vertices = self.maps1.vertices.clone();
            if !self.solid_treatment(&shape, &vertices, &root_scope.next(1.0))? {
                return Ok(false);
            }
        }
        if !self.my_inner_sol {
            self.search_distance(&root_scope)?;
        }
        Ok(self.my_is_done)
    }

    /// The part of `Perform` without an inner solution: boxes, reference distance, the
    /// map-to-map steps and the final filter; false (not done) when stopped.
    fn search_distance(&mut self, root_scope: &ProgressScope<'_>) -> Result<(), Failure> {
        if !self.my_is_init_s1 {
            self.maps1.rebuild_boxes()?;
            self.my_is_init_s1 = true;
        }
        if !self.my_is_init_s2 {
            self.maps2.rebuild_boxes()?;
            self.my_is_init_s2 = true;
        }
        self.my_dist_ref = self.initial_distance()?;
        if !self.distance_vert_vert(&root_scope.next(1.0))? {
            return Ok(());
        }
        let (maps1, maps2) = (self.maps1.clone(), self.maps2.clone());
        let steps: [MapPair<'_>; 7] = [
            MapPair::vertices_edges(&maps1, &maps2),
            MapPair::edges_vertices(&maps1, &maps2),
            MapPair::vertices_faces(&maps1, &maps2),
            MapPair::faces_vertices(&maps1, &maps2),
            MapPair::edges_edges(&maps1, &maps2),
            MapPair::edges_faces(&maps1, &maps2),
            MapPair::faces_edges(&maps1, &maps2),
        ];
        for step in &steps {
            if !self.distance_map_map(step, &root_scope.next(1.0))? {
                return Ok(());
            }
        }
        if self.my_dist_ref.abs() > self.my_eps {
            let faces = MapPair::faces_faces(&maps1, &maps2);
            if !self.distance_map_map(&faces, &root_scope.next(1.0))? {
                return Ok(());
            }
        }
        self.remove_far_solutions();
        self.my_is_done = !self.my_solutions_shape1.is_empty();
        Ok(())
    }

    /// `DistanceInitiale` of the first vertices of both shapes, or 1e30 when a shape
    /// has no vertex.
    fn initial_distance(&self) -> Result<f64, Failure> {
        if self.maps1.vertices.is_empty() || self.maps2.vertices.is_empty() {
            return Ok(1.0e30);
        }
        let v1 = topods::vertex(self.maps1.vertices.find_key(1)?)?;
        let v2 = topods::vertex(self.maps2.vertices.find_key(1)?)?;
        Ok(BRepTool::pnt(&v1)?.distance(&BRepTool::pnt(&v2)?))
    }

    /// The final filter of `Perform` (Sergey KHROMOV, 2001): each solution farther than
    /// the reference plus the deflection is removed from both lists; the index still
    /// advances after a removal, so the solution that moves into its place is not
    /// checked (native quirk).
    fn remove_far_solutions(&mut self) {
        let mut i: usize = 0;
        while i < self.my_solutions_shape1.len() {
            if self.my_solutions_shape1[i].dist() > self.my_dist_ref + self.my_eps {
                self.my_solutions_shape1.remove(i);
                self.my_solutions_shape2.remove(i);
            }
            i += 1;
        }
    }
}
