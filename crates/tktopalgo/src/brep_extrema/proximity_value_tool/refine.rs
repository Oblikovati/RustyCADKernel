// Rust translation of the refinement members and file-static helpers of
// ModelingAlgorithms/TKTopAlgo/BRepExtrema/BRepExtrema_ProximityValueTool.cxx
// (getInfoForRefinement, calcEdgeRefinementStep, calcFaceRefinementStep,
// getShapesAdditionalVertices, getEdgeAdditionalVertices, getFaceAdditionalVertices,
// doRecurTrgSplit, getModelRange, getNodesOfTrg, getEdgesStatus) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The refinement of the coarser mesh of the proximity value tool.
use super::{BRepExtremaProximityValueTool, ProximityShape};
use crate::brep_extrema::{BRepExtremaProxPntStatus, BRepExtremaProximityDistTool};
use crate::brep_gprop::BRepGProp;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topods::{self, Edge, Face};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::integer::integer_last;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::gcpnts::{AbscissaPoint, QuasiUniformAbscissa};
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::bvh::{BvhArray3d, BvhVec3d, array, std_max};
use rustycad_tkmath::gp::{Point3, Xyz};
use rustycad_tkmath::poly::{Triangulation, mesh_purpose};
use rustycad_tkmath::toploc::Location;

/// A triangle of the refinement and the statuses of its three links.
type SplitTriangle = ([Point3; 3], [BRepExtremaProxPntStatus; 3]);

/// `calcEdgeRefinementStep(theEdge, theNbNodes)`: the edge length over the number of
/// polygon links (0 below two nodes).
fn calc_edge_refinement_step(edge: &Edge, nb_nodes: i32) -> Result<f64, Failure> {
    if nb_nodes < 2 {
        return Ok(0.0);
    }
    let curve = BRepAdaptorCurve::with_edge(edge)?;
    let len: f64 = AbscissaPoint::length(&curve as &dyn Adaptor3dCurve)?;
    Ok(len / f64::from(nb_nodes - 1))
}

/// `calcFaceRefinementStep(theFace, theNbTrg)`: twice the mean area of a triangle (0
/// without triangles).
fn calc_face_refinement_step(face: &Face, nb_trg: i32) -> Result<f64, Failure> {
    if nb_trg < 1 {
        return Ok(0.0);
    }
    let mut props = GProps::new();
    BRepGProp::surface_properties(face, &mut props, false, false)?;
    let area: f64 = props.mass();
    Ok(2.0 * (area / f64::from(nb_trg)))
}

/// `getInfoForRefinement(theShape, theShapeType, theNbNodes, theStep)` on the first
/// shape of `state`: the type is written first (even when the mesh is missing, as
/// natively), then the node count and the step; false without a mesh, for another type
/// or for a step below Confusion.
pub(super) fn info_for_refinement(state: &mut ProximityShape) -> Result<bool, Failure> {
    let shape = state.shape.clone();
    match shape.shape_type()? {
        ShapeEnum::Face => {
            state.shape_type = Some(ShapeEnum::Face);
            let face = topods::face(&shape)?;
            let (triangulation, _) = BRepTool::triangulation(&face, mesh_purpose::NONE)?;
            let Some(triangulation) = triangulation.get() else {
                return Ok(false);
            };
            state.nb_nodes = triangulation.nb_nodes();
            state.step = calc_face_refinement_step(&face, triangulation.nb_triangles())?;
        }
        ShapeEnum::Edge => {
            state.shape_type = Some(ShapeEnum::Edge);
            let edge = topods::edge(&shape)?;
            let (polygon, _) = BRepTool::polygon3d(&edge)?;
            let Some(polygon) = polygon.get() else {
                return Ok(false);
            };
            state.nb_nodes = polygon.nb_nodes();
            state.step = calc_edge_refinement_step(&edge, state.nb_nodes)?;
        }
        _ => return Ok(false),
    }
    Ok(state.step >= precision::confusion())
}

/// `getEdgeAdditionalVertices(theEdge, theStep, theAddVertices, theAddStatuses)`: the
/// inner points of a quasi-uniform sampling of the edge with the step (at least 3
/// points), all in the middle; false for an edge without a 3D curve or a step below
/// Confusion, or when the sampling fails.
fn edge_additional_vertices(
    edge: &Edge,
    step: f64,
    state: &mut ProximityShape,
) -> Result<bool, Failure> {
    let curve = BRepAdaptorCurve::with_edge(edge)?;
    if !curve.transformed_curve().is_3d_curve() || step < precision::confusion() {
        return Ok(false);
    }
    let len: f64 = AbscissaPoint::length(&curve as &dyn Adaptor3dCurve)?;
    let nb_sample_points: i32 = (len / step) as i32 + 1;
    let sampling = QuasiUniformAbscissa::with_nb_points(
        &curve as &dyn Adaptor3dCurve,
        nb_sample_points.max(3),
    )?;
    if !sampling.is_done() {
        return Ok(false);
    }
    let nb_nodes: i32 = sampling.nb_points()?;
    for vert_idx in 2..nb_nodes {
        // The extreme points are not added.
        let p: Point3 = curve.value(sampling.parameter(vert_idx)?)?;
        array::append(&mut state.add_vertices, BvhVec3d::new(p.x(), p.y(), p.z()));
        state.add_status.push(BRepExtremaProxPntStatus::Middle);
    }
    Ok(true)
}

/// `getModelRange(theLocation, theTr)`: the largest extent of the box of the mesh.
fn model_range(location: &Location, tr: &Triangulation) -> Result<f64, Failure> {
    let mut bounds = BoundingBox3::default();
    tr.min_max(&mut bounds, location.transformation(), false);
    let [xm, ym, zm, x_max, y_max, z_max] = bounds.get()?;
    let range: f64 = x_max - xm;
    let range: f64 = std_max(range, y_max - ym);
    Ok(std_max(range, z_max - zm))
}

/// `getNodesOfTrg` and `getEdgesStatus`: the transformed nodes of a triangle and the
/// statuses of its links (border when free).
fn triangle_of(
    tri_idx: i32,
    location: &Location,
    tr: &Handle<Triangulation>,
) -> Result<SplitTriangle, Failure> {
    let triangulation = tr
        .get()
        .ok_or_else(|| Failure::new("BRepExtrema_ProximityValueTool: a null triangulation"))?;
    let (n1, n2, n3) = triangulation.triangle(tri_idx)?.get();
    let transformation = location.transformation();
    let trg: [Point3; 3] = [
        triangulation.node(n1)?.transformed(transformation),
        triangulation.node(n2)?.transformed(transformation),
        triangulation.node(n3)?.transformed(transformation),
    ];
    let mut statuses = [BRepExtremaProxPntStatus::Middle; 3];
    for (j, status) in statuses.iter_mut().enumerate() {
        let k: i32 = (j as i32 + 1) % 3;
        if BRepExtremaProximityDistTool::is_edge_on_border(tri_idx, j as i32, k, tr)? {
            *status = BRepExtremaProxPntStatus::Border;
        }
    }
    Ok((trg, statuses))
}

/// The two halves of a triangle split at the middle of its link `bisected`: the new
/// links through the middle point are in the middle of the face.
fn halves((trg, statuses): SplitTriangle, bisected: usize, center: Point3) -> [SplitTriangle; 2] {
    let (mut trg1, mut trg2) = (trg, trg);
    let (mut statuses1, mut statuses2) = (statuses, statuses);
    let middle = BRepExtremaProxPntStatus::Middle;
    match bisected {
        0 => {
            (trg1[0], trg2[1]) = (center, center);
            (statuses1[2], statuses2[1]) = (middle, middle);
        }
        1 => {
            (trg1[1], trg2[2]) = (center, center);
            (statuses1[0], statuses2[2]) = (middle, middle);
        }
        _ => {
            (trg1[2], trg2[0]) = (center, center);
            (statuses1[1], statuses2[0]) = (middle, middle);
        }
    }
    [(trg1, statuses1), (trg2, statuses2)]
}

/// The index of the longest link of a triangle (the last of equal ones, as the native
/// nested comparisons).
fn longest_link(trg: &[Point3; 3]) -> usize {
    let d: [f64; 3] = [
        trg[0].distance(&trg[1]),
        trg[1].distance(&trg[2]),
        trg[2].distance(&trg[0]),
    ];
    if d[0] > d[1] {
        if d[0] > d[2] { 0 } else { 2 }
    } else if d[1] > d[2] {
        1
    } else {
        2
    }
}

impl BRepExtremaProximityValueTool {
    /// `getShapesAdditionalVertices()`: the steps compared (a sample count above the
    /// node count of an edge refines it too); a step more than twice the other one
    /// asks for the refinement of that shape to the other step. The refinement flags
    /// stay set for later computations (native quirk).
    pub(super) fn shapes_additional_vertices(&mut self) -> Result<bool, Failure> {
        let (mut step1, mut step2) = (self.shape1.step, self.shape2.step);
        match (self.shape1.shape_type, self.shape2.shape_type) {
            (Some(ShapeEnum::Edge), Some(ShapeEnum::Edge)) => {
                // A sample count above the node count asks for a finer step.
                if self.shape1.nb_samples > self.shape1.nb_nodes {
                    step1 = calc_edge_refinement_step(
                        &topods::edge(&self.shape1.shape)?,
                        self.shape1.nb_samples,
                    )?;
                    self.shape1.is_refinement_required = true;
                }
                if self.shape2.nb_samples > self.shape2.nb_nodes {
                    step2 = calc_edge_refinement_step(
                        &topods::edge(&self.shape2.shape)?,
                        self.shape2.nb_samples,
                    )?;
                    self.shape2.is_refinement_required = true;
                }
                self.refine_edges(step1, step2)
            }
            (Some(ShapeEnum::Face), Some(ShapeEnum::Face)) => self.refine_faces(step1, step2),
            _ => Ok(true),
        }
    }

    /// The edge-edge branch after the sample counts.
    fn refine_edges(&mut self, mut step1: f64, mut step2: f64) -> Result<bool, Failure> {
        if step1 / step2 > 2.0 {
            self.shape1.is_refinement_required = true;
            step1 = step2;
        } else if step2 / step1 > 2.0 {
            self.shape2.is_refinement_required = true;
            step2 = step1;
        }
        for (state, step) in [(&mut self.shape1, step1), (&mut self.shape2, step2)] {
            if state.is_refinement_required
                && !edge_additional_vertices(&topods::edge(&state.shape)?, step, state)?
            {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// The face-face branch: the stored steps compared; only the first shape that needs
    /// the refinement is refined.
    fn refine_faces(&mut self, mut step1: f64, mut step2: f64) -> Result<bool, Failure> {
        if step1 / step2 > 2.0 {
            self.shape1.is_refinement_required = true;
            step1 = self.shape2.step;
        } else if step2 / step1 > 2.0 {
            self.shape2.is_refinement_required = true;
            step2 = self.shape1.step;
        }
        if self.shape1.is_refinement_required {
            return self.face_additional_vertices(true, step1);
        }
        if self.shape2.is_refinement_required {
            return self.face_additional_vertices(false, step2);
        }
        Ok(true)
    }

    /// `getFaceAdditionalVertices(theFace, theStep, theAddVertices, theAddStatuses)` on
    /// the first or the second shape: each triangle split recursively (the cell filter
    /// reset to the model range) until the halves are smaller than the step; false
    /// without a triangulation.
    fn face_additional_vertices(&mut self, first: bool, step: f64) -> Result<bool, Failure> {
        let tol: f64 = precision::confusion();
        let face = topods::face(if first {
            &self.shape1.shape
        } else {
            &self.shape2.shape
        })?;
        let (tr, location) = BRepTool::triangulation(&face, mesh_purpose::NONE)?;
        let Some(triangulation) = tr.get() else {
            return Ok(false);
        };
        let cell_size: f64 = std_max(
            tol,
            model_range(&location, triangulation)? / f64::from(integer_last()),
        );
        self.my_cells.reset(cell_size, None)?;
        let mut added = (BvhArray3d::default(), Vec::new());
        {
            let state = if first {
                &mut self.shape1
            } else {
                &mut self.shape2
            };
            std::mem::swap(&mut added.0, &mut state.add_vertices);
            std::mem::swap(&mut added.1, &mut state.add_status);
        }
        let mut result: Result<(), Failure> = Ok(());
        for tri_idx in 1..=triangulation.nb_triangles() {
            result = triangle_of(tri_idx, &location, &tr)
                .and_then(|trg| self.do_recur_trg_split(trg, (tol, step), &mut added));
            if result.is_err() {
                break;
            }
        }
        let state = if first {
            &mut self.shape1
        } else {
            &mut self.shape2
        };
        state.add_vertices = added.0;
        state.add_status = added.1;
        result.map(|()| true)
    }

    /// `doRecurTrgSplit(theTrg, theEdgesStatus, theTol, theStep, theAddVertices,
    /// theAddStatuses)`: a triangle whose area exceeds the step (by Precision's square
    /// confusion) is split at the middle of its longest link; the middle point is added
    /// (with the status of that link) unless a point within the tolerance was added.
    fn do_recur_trg_split(
        &mut self,
        (trg, statuses): SplitTriangle,
        (tol, step): (f64, f64),
        added: &mut (BvhArray3d, Vec<BRepExtremaProxPntStatus>),
    ) -> Result<(), Failure> {
        let side1: Xyz = trg[1].xyz() - trg[0].xyz();
        let side2: Xyz = trg[2].xyz() - trg[0].xyz();
        let trg_area: f64 = 0.5 * side1.cross_magnitude(side2);
        if trg_area - step < precision::square_confusion() {
            return Ok(());
        }
        let bisected: usize = longest_link(&trg);
        let mut center: Point3 = trg[bisected];
        center.barycenter(0.5, trg[(bisected + 1) % 3], 0.5);
        let mut bounds = BoundingBox3::default();
        bounds.add_point(center);
        bounds.enlarge(tol);
        let (corner_min, corner_max) = (bounds.corner_min()?.xyz(), bounds.corner_max()?.xyz());
        self.my_inspector.set_current(center.xyz());
        self.my_cells
            .inspect_range(&corner_min, &corner_max, &mut self.my_inspector);
        if self.my_inspector.is_need_add() {
            array::append(
                &mut added.0,
                BvhVec3d::new(center.x(), center.y(), center.z()),
            );
            added.1.push(statuses[bisected]);
            self.my_inspector.add(center.xyz());
            self.my_cells
                .add_range(&array::size(&added.0), &corner_min, &corner_max);
        }
        for half in halves((trg, statuses), bisected, center) {
            self.do_recur_trg_split(half, (tol, step), added)?;
        }
        Ok(())
    }
}
