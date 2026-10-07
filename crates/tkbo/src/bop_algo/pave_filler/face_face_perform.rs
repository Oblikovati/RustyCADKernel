//! `BOPAlgo_PaveFiller::PerformFF` (`BOPAlgo_PaveFiller_6.cxx`): face/face intersection.
use super::face_face_task::BOPAlgoFaceFaceTask;
use super::{BOPAlgoPaveFiller, Glue, missing_structure};
use crate::bop_algo::ParallelAlgo;
use crate::bop_ds::{InterfFF, Pair};
use crate::int_tools::{Curve, PntOn2Faces, Tools};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{Edge, Face, ShapeIterator};
use rustycad_tkernel::ncollection::{DataMap, Map};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeomalgo::int_surf::IntSurfPntOn2S;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::geom_abs::SurfaceType;

#[path = "face_face_seam.rs"]
mod seam;

impl BOPAlgoPaveFiller {
    /// Refreshes the face records of all touched faces and intersects every face pair with
    /// interfering boxes, storing the section curves and points (`PerformFF`).
    /// `filler.perform_ff()?`.
    pub fn perform_ff(&mut self) -> Result<(), Failure> {
        let pairs = self.iterator_pairs(ShapeEnum::Face, ShapeEnum::Face)?;
        self.update_touched_face_info(&pairs)?;
        if pairs.is_empty() {
            return Ok(());
        }
        let edge_points = self.ee_new_vertices_by_edges()?;
        let mut tasks = Vec::new();
        for (first, second) in pairs {
            if let Some(task) = self.face_face_task(first, second, &edge_points)? {
                tasks.push(task);
            }
        }
        for task in &mut tasks {
            task.perform_parallel()?;
        }
        for mut task in tasks {
            self.store_face_face_task(&mut task)?;
        }
        Ok(())
    }

    fn update_touched_face_info(&mut self, pairs: &[(i32, i32)]) -> Result<(), Failure> {
        let mut faces: Map<i32> = Map::new(1);
        for (first, second) in pairs {
            faces.add(*first);
            faces.add(*second);
        }
        let structure = self.structure_mut()?;
        for index in 0..structure.nb_source_shapes() {
            let info = structure.shape_info(index)?;
            if info.shape_type() == ShapeEnum::Face && info.has_reference() {
                faces.add(index);
            }
        }
        structure.update_face_info_on_map(&faces)?;
        structure.update_face_info_in_map(&faces)
    }

    /// New vertices of EE interferences keyed by their edge pair (`aEEMap`).
    fn ee_new_vertices_by_edges(&self) -> Result<DataMap<Pair, Vec<i32>>, Failure> {
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        let mut map: DataMap<Pair, Vec<i32>> = DataMap::default();
        for record in structure.interf_ee_records().iter() {
            let Some(vertex) = record.get_index_new() else {
                continue;
            };
            let (first, second) = record.indices();
            let key = Pair::new(first, second);
            if let Some(list) = map.change_seek(&key) {
                list.push(vertex);
            } else {
                map.bind(key, vec![vertex]);
            }
        }
        Ok(map)
    }

    /// Builds the task of one face pair, or records an empty interference for glued pairs and
    /// non-interfering planes.
    fn face_face_task(
        &mut self,
        first: i32,
        second: i32,
        edge_points: &DataMap<Pair, Vec<i32>>,
    ) -> Result<Option<BOPAlgoFaceFaceTask<'static>>, Failure> {
        if self.glue != Glue::Off {
            self.append_empty_face_face_record(first, second, Some(false))?;
            return Ok(None);
        }
        let faces = self.face_pair(first, second)?;
        let types = self.face_surface_types(&faces)?;
        if types == [SurfaceType::Plane, SurfaceType::Plane] && !self.check_planes(first, second)? {
            self.append_empty_face_face_record(first, second, None)?;
            return Ok(None);
        }
        let (shifted, shift) = if types != [SurfaceType::Plane, SurfaceType::Plane] {
            self.shift_faces_to_edge_intersections(&faces, edge_points)?
        } else {
            (faces.clone(), 0.)
        };
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        let boxes = [
            structure.shape_info(first)?.box_bounds().clone(),
            structure.shape_info(second)?.box_bounds().clone(),
        ];
        let mut task = BOPAlgoFaceFaceTask::default();
        task.set_run_parallel(self.options.run_parallel());
        task.set_indices(first, second);
        task.set_faces(&shifted[0], &shifted[1]);
        task.set_boxes(&boxes[0], &boxes[1]);
        // With closed edges the curve tolerance must not be less than the applied shift.
        task.set_tol_ff(shift.max(self.tolerance_ff(&faces, types)?));
        let seeds = self.ef_points(first, second)?;
        if !seeds.is_empty() {
            task.set_list(&seeds);
        }
        let attribute = self.section_attribute;
        task.set_parameters(
            attribute.approximation(),
            attribute.pcurve_on_s1(),
            attribute.pcurve_on_s2(),
            1.0e-7,
        );
        task.set_fuzzy_value(self.options.fuzzy_value());
        Ok(Some(task))
    }

    fn store_face_face_task(
        &mut self,
        task: &mut BOPAlgoFaceFaceTask<'static>,
    ) -> Result<(), Failure> {
        let [first, second] = task.indices();
        if !task.intersector().is_done() || task.parallel_options().has_errors() {
            self.append_empty_face_face_record(first, second, None)?;
            let faces = (
                task.face1().clone().into_shape(),
                task.face2().clone().into_shape(),
            );
            return self.add_intersection_failed_warning(&faces.0, &faces.1);
        }
        let tangent = task.intersector().tangent_faces();
        task.intersector_mut().prepare_lines_3d(false)?;
        task.apply_trsf()?;
        let curves: Vec<Curve> = task.intersector().lines()?.to_vec();
        let points: Vec<PntOn2Faces> = task.intersector().points().to_vec();
        let structure = self.structure_mut()?;
        if !curves.is_empty() || !points.is_empty() {
            structure.add_interference(first, second);
        }
        let mut record = InterfFF::default();
        record.set_indices(first, second);
        record.set_tangent_faces(tangent);
        record.init(curves.len() as i32, points.len() as i32);
        append_section_curves(&mut record, &curves, task)?;
        for point in &points {
            record.change_points().appended().set_pnt(*point.p1().pnt());
        }
        structure.interf_ff().append(record);
        Ok(())
    }

    fn append_empty_face_face_record(
        &mut self,
        first: i32,
        second: i32,
        tangent: Option<bool>,
    ) -> Result<(), Failure> {
        let mut record = InterfFF::default();
        record.set_indices(first, second);
        if let Some(tangent) = tangent {
            record.set_tangent_faces(tangent);
        }
        record.init(0, 0);
        self.structure_mut()?.interf_ff().append(record);
        Ok(())
    }

    fn face_pair(&self, first: i32, second: i32) -> Result<[Face; 2], Failure> {
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        Ok([
            Face::unchecked(structure.shape(first)?.clone()),
            Face::unchecked(structure.shape(second)?.clone()),
        ])
    }

    fn face_surface_types(&self, faces: &[Face; 2]) -> Result<[SurfaceType; 2], Failure> {
        let context = self.context_ref()?;
        Ok([
            context.surface_adaptor(&faces[0])?.read().get_type()?,
            context.surface_adaptor(&faces[1])?.read().get_type()?,
        ])
    }

    /// Native static `ToleranceFF`: the larger face tolerance, at least 5e-6 unless both
    /// surfaces are elementary.
    fn tolerance_ff(&self, faces: &[Face; 2], types: [SurfaceType; 2]) -> Result<f64, Failure> {
        let mut tolerance =
            BRepTool::face_tolerance(&faces[0])?.max(BRepTool::face_tolerance(&faces[1])?);
        let analytic = |kind: SurfaceType| {
            matches!(
                kind,
                SurfaceType::Plane
                    | SurfaceType::Cylinder
                    | SurfaceType::Cone
                    | SurfaceType::Sphere
                    | SurfaceType::Torus
            )
        };
        if !analytic(types[0]) || !analytic(types[1]) {
            tolerance = tolerance.max(5.0e-6);
        }
        Ok(tolerance)
    }

    /// Starting points of the walking from EF intersections between the faces (`GetEFPnts`).
    fn ef_points(&self, first: i32, second: i32) -> Result<Vec<IntSurfPntOn2S>, Failure> {
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        let mut shapes: Map<i32> = Map::new(1);
        for face in [first, second] {
            shapes.add(face);
            for sub_shape in structure.shape_info(face)?.sub_shapes() {
                shapes.add(*sub_shape);
            }
        }
        let mut points = Vec::new();
        for record in structure.interf_ef_records().iter() {
            if !record.has_index_new() {
                continue;
            }
            let (edge, opposite) = record.indices();
            if !shapes.contains(&edge) || !shapes.contains(&opposite) {
                continue;
            }
            let parameter = record.common_part().vertex_parameter1();
            let face = if opposite == first { second } else { first };
            if let Some(point) = self.ef_point(edge, opposite, face, parameter, face == first)? {
                points.push(point);
            }
        }
        Ok(points)
    }

    fn ef_point(
        &self,
        edge: i32,
        opposite: i32,
        face: i32,
        parameter: f64,
        on_first: bool,
    ) -> Result<Option<IntSurfPntOn2S>, Failure> {
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        let context = self.context_ref()?;
        let edge = Edge::unchecked(structure.shape(edge)?.clone());
        let opposite = Face::unchecked(structure.shape(opposite)?.clone());
        let face = Face::unchecked(structure.shape(face)?.clone());
        let (curve, _, _) = BRepTool::curve(&edge)?;
        let point = curve.eval_d0(parameter)?;
        let pcurve = BRepTool::curve_on_face(&edge, &face)?.pcurve;
        let opposite_projector = context.proj_ps(&opposite)?;
        let mut seed = IntSurfPntOn2S::default();
        if let Some(pcurve) = pcurve.get() {
            let uv = pcurve.value(parameter)?;
            let mut projection = opposite_projector.write();
            projection.perform(point)?;
            if !projection.is_done() {
                return Ok(None);
            }
            let (u, v) = projection.lower_distance_parameters()?;
            if on_first {
                seed.set_parameters(uv.x(), uv.y(), u, v);
            } else {
                seed.set_parameters(u, v, uv.x(), uv.y());
            }
            return Ok(Some(seed));
        }
        let face_projector = context.proj_ps(&face)?;
        let mut face_projection = face_projector.write();
        face_projection.perform(point)?;
        let mut projection = opposite_projector.write();
        projection.perform(point)?;
        if !face_projection.is_done() || !projection.is_done() {
            return Ok(None);
        }
        let (u1, v1) = face_projection.lower_distance_parameters()?;
        let (u2, v2) = projection.lower_distance_parameters()?;
        if on_first {
            seed.set_parameters(u1, v1, u2, v2);
        } else {
            seed.set_parameters(u2, v2, u1, v1);
        }
        Ok(Some(seed))
    }
}

/// Valid section curves with their boxes enlarged by the face pair tolerance plus the
/// maximal vertex tolerance of both faces.
fn append_section_curves(
    record: &mut InterfFF,
    curves: &[Curve],
    task: &BOPAlgoFaceFaceTask<'static>,
) -> Result<(), Failure> {
    let tolerance = task.tol_ff();
    let mut expansion = tolerance;
    if !curves.is_empty() {
        let first = BRepTool::max_tolerance(task.face1(), ShapeEnum::Vertex)?;
        let second = BRepTool::max_tolerance(task.face2(), ShapeEnum::Vertex)?;
        expansion += first.max(second);
    }
    for curve in curves {
        let mut bounds = BoundingBox3::default();
        if !Tools::check_curve(curve, &mut bounds)? {
            continue;
        }
        bounds.enlarge(expansion);
        let stored = record.change_curves().appended();
        stored.set_curve(curve);
        stored.set_box(&bounds);
        stored.set_tolerance(curve.tolerance().max(tolerance));
    }
    Ok(())
}

/// Iterates the edges of the wires of a face in native TopoDS_Iterator order.
fn face_edges(face: &Face) -> Result<Vec<Edge>, Failure> {
    let mut edges = Vec::new();
    let mut wires = ShapeIterator::new(face, true, true);
    while wires.more() {
        let mut items = ShapeIterator::new(wires.value()?, true, true);
        while items.more() {
            edges.push(Edge::unchecked(items.value()?.clone()));
            items.next();
        }
        wires.next();
    }
    Ok(edges)
}
