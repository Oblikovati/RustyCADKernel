//! `UnifyEdges`, `UpdateMapOfShapes`, `CheckSharedVertices` and `SetFixWireModes` of
//! ShapeUpgrade_UnifySameDomain.cxx: the chains of edges merged, then the changed faces
//! and shells fixed in the context.
use super::UnifyState;
use super::face_bounds::edges_of;
use super::geometry::clear_rts;
use crate::shape_analysis::ShapeAnalysisEdge;
use crate::shape_fix::{FixRoot, ShapeFixFace, ShapeFixShell};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, IndexedMapOfShape, MapOfShape};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomPlane;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tktopalgo::brep_lib;

/// `UpdateMapOfShapes(theMapOfShapes, theContext)`: the images of the shapes added.
fn update_map_of_shapes(map: &mut MapOfShape, context: &dyn ReShape) -> Result<(), Failure> {
    let shapes: Vec<Shape> = map.iter().cloned().collect();
    for shape in shapes {
        let image = context.apply(&shape, ShapeEnum::Shape)?;
        if !image.is_same(&shape) {
            map.add(image);
        }
    }
    Ok(())
}

/// `CheckSharedVertices(theSeqEdges, theMapEdgesVertex, theMapKeepShape, theShareVertMap)`:
/// the vertices met twice as edge ends that have more than 2 edges or are kept.
fn check_shared_vertices(
    edges: &[Shape],
    vertex_edges: &IndexedDataMapOfShapeListOfShape,
    keep: &MapOfShape,
) -> Result<MapOfShape, Failure> {
    let analysis = ShapeAnalysisEdge::new();
    let mut seen = MapOfShape::default();
    let mut repeated = Vec::new();
    for shape in edges {
        let edge = topods::edge(shape)?;
        for vertex in [analysis.first_vertex(&edge), analysis.last_vertex(&edge)] {
            let vertex = vertex.into_shape();
            if !seen.add(vertex.clone()) {
                repeated.push(vertex);
            }
        }
    }
    let mut shared = MapOfShape::default();
    for vertex in repeated {
        if vertex_edges.find_from_key(&vertex)?.extent() > 2 || keep.contains(&vertex) {
            shared.add(vertex);
        }
    }
    Ok(shared)
}

/// `SetFixWireModes(theSff)`: no self-intersection, lacking, notch, topology, loop, gap
/// or small-edge fix.
fn set_fix_wire_modes(fixer: &ShapeFixFace) -> Result<(), Failure> {
    let tool = fixer.fix_wire_tool();
    let wire = tool.get().ok_or_else(|| {
        Failure::with_type(
            "Standard_NullObject",
            "ShapeUpgrade_UnifySameDomain: null FixWireTool; expected a ShapeFix_Wire",
            "",
        )
    })?;
    wire.set_fix_self_intersection_mode(0);
    wire.set_fix_nonadjacent_intersecting_edges_mode(0);
    wire.set_fix_lacking_mode(0);
    wire.set_fix_notched_edges_mode(0);
    wire.set_topology_mode(false);
    wire.set_remove_loop_mode(0);
    wire.set_fix_gaps_by_ranges(false);
    wire.set_fix_small_mode(0);
    Ok(())
}

impl UnifyState {
    /// `UnifyEdges()`.
    pub(super) fn unify_edges(&mut self) -> Result<(), Failure> {
        let result = self.apply(&self.shape)?;
        let mut edge_faces = IndexedDataMapOfShapeListOfShape::default();
        topexp::map_shapes_and_ancestors(
            &result,
            ShapeEnum::Edge,
            ShapeEnum::Face,
            &mut edge_faces,
        )?;
        let mut vertex_edges = IndexedDataMapOfShapeListOfShape::default();
        topexp::map_shapes_and_unique_ancestors(
            &result,
            ShapeEnum::Vertex,
            ShapeEnum::Edge,
            &mut vertex_edges,
            false,
        )?;
        let mut vf_map = IndexedDataMapOfShapeListOfShape::default();
        topexp::map_shapes_and_unique_ancestors(
            &result,
            ShapeEnum::Vertex,
            ShapeEnum::Face,
            &mut vf_map,
            false,
        )?;
        if self.safe_input_mode {
            let context = self.context.clone();
            update_map_of_shapes(
                &mut self.keep_shapes,
                context.get().ok_or_else(null_context)?,
            )?;
        }
        let mut edges = Vec::with_capacity(edge_faces.extent());
        for index in 1..=edge_faces.extent() {
            edges.push(edge_faces.find_key(index)?.clone());
        }
        let shared = check_shared_vertices(&edges, &vertex_edges, &self.keep_shapes)?;
        let merged = self.merge_seq(&edges, &vf_map, &shared)?;
        let changed = self.changed_faces(merged, &edge_faces)?;
        for index in 1..=changed.extent() {
            self.fix_changed_face(changed.find_key(index)?)?;
        }
        if changed.extent() > 0 {
            self.fix_changed_shells(&result)?;
        }
        self.shape = self.apply(&self.shape)?;
        Ok(())
    }

    /// The faces of the edges recorded in the context.
    fn changed_faces(
        &self,
        merged: bool,
        edge_faces: &IndexedDataMapOfShapeListOfShape,
    ) -> Result<IndexedMapOfShape, Failure> {
        let mut changed = IndexedMapOfShape::default();
        if !merged {
            return Ok(changed);
        }
        let context = self.context()?;
        for index in 1..=edge_faces.extent() {
            if context.is_recorded(edge_faces.find_key(index)?) {
                for face in edge_faces.find_from_index(index)?.iter() {
                    changed.add(face.clone());
                }
            }
        }
        Ok(changed)
    }

    /// "fix changed faces and replace them in the local context".
    fn fix_changed_face(&self, face: &Shape) -> Result<(), Failure> {
        let face = self.apply(face)?;
        if face.is_null() {
            return Ok(());
        }
        let face = topods::face(&face)?;
        if !self.safe_input_mode {
            // "for a planar face create and store pcurve of edge on face to speed up all
            // operations; but this is allowed only when non-safe mode in force".
            let (surface, _) = BRepTool::surface_with_location(&face)?;
            if !clear_rts(&surface).cast::<GeomPlane>().is_null() {
                let edges = edges_of(face.as_ref())?;
                brep_lib::build_pcurve_for_edges_on_plane(edges.iter(), &face)?;
            }
        }
        let fixer = ShapeFixFace::new(face.clone().into_shape());
        if self.safe_input_mode {
            fixer.set_context(&self.context);
        }
        let tolerance = precision::confusion();
        fixer.set_precision(tolerance);
        fixer.set_min_tolerance(tolerance);
        fixer.set_max_tolerance(1.0_f64.max(tolerance * 1000.));
        fixer.set_fix_orientation_mode(0);
        fixer.set_fix_add_natural_bound_mode(0);
        fixer.set_fix_intersecting_wires_mode(0);
        fixer.set_fix_loop_wires_mode(0);
        fixer.set_fix_split_face_mode(0);
        fixer.set_fix_periodic_degenerated_mode(0);
        set_fix_wire_modes(&fixer)?;
        fixer.perform()?;
        self.context()?
            .replace(face.as_ref(), fixer.face()?.as_ref())
    }

    /// "fix changed shell and replace it in the local context".
    fn fix_changed_shells(&self, result: &Shape) -> Result<(), Failure> {
        let mut result1 = self.apply(result)?;
        let mut changed = false;
        for shape in Explorer::new(&result1, ShapeEnum::Shell, ShapeEnum::Shape)? {
            let shell = topods::shell(&shape)?;
            let fixer = Handle::new(ShapeFixShell::new());
            let fix = fixer.get().ok_or_else(null_context)?;
            fix.fix_face_orientation(&shell, true, false)?;
            let new_shell = fix.shell();
            if !new_shell.is_same(&shell) {
                self.context()?.replace(&shape, new_shell.as_ref())?;
                changed = true;
            }
        }
        if changed {
            result1 = self.apply(&result1)?;
        }
        self.context()?.replace(&self.shape, &result1)
    }
}

fn null_context() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "ShapeUpgrade_UnifySameDomain: null context; expected a ShapeBuild_ReShape",
        "",
    )
}
