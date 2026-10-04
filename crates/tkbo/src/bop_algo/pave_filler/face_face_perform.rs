//! Face/face task preparation and geometric result storage from PerformFF.
use super::face_face_task::BOPAlgoFaceFaceTask;
use super::{BOPAlgoPaveFiller, Glue};
use crate::bop_algo::{AlertIntersectionFailed, ParallelAlgo};
use crate::bop_ds::InterfFF;
use crate::int_tools::{Context, Curve};
use rustycad_tkbrep::{brep::BRepTool, topods::Face};
use rustycad_tkernel::{
    message::AlertHandle,
    ncollection::Map as IndexMap,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::{adaptor3d::Adaptor3dSurface, top_abs::ShapeEnum};
use rustycad_tkmath::geom_abs::SurfaceType;
use std::sync::Arc;

impl BOPAlgoPaveFiller {
    /// Executes ordinary Face/Face tasks and records geometric interferences.
    /// `filler.perform_ff()?` runs the source task phase before `MakeBlocks`.
    pub fn perform_ff(&mut self) -> Result<(), Failure> {
        let pairs = self.face_face_pairs()?;
        self.update_touched_face_info(&pairs)?;
        if pairs.is_empty() {
            return Ok(());
        }
        self.prepare_face_face_pairs(pairs)
    }

    fn reject_unported_seed_inputs(&self) -> Result<(), Failure> {
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        let edge_seeds = structure
            .interf_ee_records()
            .iter()
            .any(|row| row.has_index_new());
        let face_seeds = structure
            .interf_ef_records()
            .iter()
            .any(|row| row.has_index_new());
        if edge_seeds || face_seeds {
            return Err(Failure::with_type(
                "Standard_NotImplemented",
                "BOPAlgo_PaveFiller::PerformFF seam shifting and edge-face walking seeds are not ported",
                "",
            ));
        }
        Ok(())
    }

    fn update_touched_face_info(&mut self, pairs: &[[i32; 2]]) -> Result<(), Failure> {
        let mut faces = IndexMap::default();
        for pair in pairs {
            faces.add(pair[0]);
            faces.add(pair[1]);
        }
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        for index in 0..structure.nb_source_shapes() {
            let info = structure.shape_info(index)?;
            if info.shape_type() == ShapeEnum::Face && info.has_reference() {
                faces.add(index);
            }
        }
        let structure = self.data_structure.as_mut().ok_or_else(missing_structure)?;
        structure.update_face_info_on_map(&faces)?;
        structure.update_face_info_in_map(&faces)
    }

    fn face_face_pairs(&mut self) -> Result<Vec<[i32; 2]>, Failure> {
        let iterator = self.iterator.as_mut().ok_or_else(missing_iterator)?;
        iterator.initialize(ShapeEnum::Face, ShapeEnum::Face);
        let mut pairs = Vec::with_capacity(iterator.expected_length().max(0) as usize);
        while iterator.more() {
            let (first, second) = iterator.value()?;
            pairs.push([first, second]);
            iterator.next();
        }
        Ok(pairs)
    }

    fn prepare_face_face_pairs(&mut self, pairs: Vec<[i32; 2]>) -> Result<(), Failure> {
        if self.glue != Glue::Off {
            return self.append_empty_face_face_records(&pairs);
        }
        self.reject_unported_seed_inputs()?;
        let context = self.context.clone();
        let mut tasks = self.prepare_face_face_tasks(&pairs, &context)?;
        for task in &mut tasks {
            task.perform_parallel()?;
        }
        self.store_face_face_results(tasks)
    }

    fn prepare_face_face_tasks(
        &mut self,
        pairs: &[[i32; 2]],
        context: &Handle<Context>,
    ) -> Result<Vec<BOPAlgoFaceFaceTask<'static>>, Failure> {
        let mut tasks = Vec::with_capacity(pairs.len());
        for pair in pairs {
            if self.prepare_plane_pair(*pair)? {
                continue;
            }
            tasks.push(self.configured_face_face_task(*pair, context)?);
        }
        Ok(tasks)
    }

    fn prepare_plane_pair(&mut self, pair: [i32; 2]) -> Result<bool, Failure> {
        let faces = self.face_shapes(pair)?;
        let context = self.context.get().ok_or_else(missing_context)?;
        if !is_plane_pair(&faces, context)? {
            return Ok(false);
        }
        if !self.check_planes(pair[0], pair[1])? {
            self.append_empty_face_face_records(&[pair])?;
            return Ok(true);
        }
        Ok(false)
    }

    fn configured_face_face_task(
        &self,
        pair: [i32; 2],
        context: &Handle<Context>,
    ) -> Result<BOPAlgoFaceFaceTask<'static>, Failure> {
        let faces = self.face_shapes(pair)?;
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        let boxes = [
            structure.shape_info(pair[0])?.box_bounds().clone(),
            structure.shape_info(pair[1])?.box_bounds().clone(),
        ];
        let tolerance = face_pair_tolerance(&faces, context)?;
        let mut task = BOPAlgoFaceFaceTask::default();
        configure_task(self, &mut task, pair, &faces, &boxes, tolerance, context);
        Ok(task)
    }

    fn face_shapes(&self, pair: [i32; 2]) -> Result<[Face; 2], Failure> {
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        Ok([
            Face::unchecked(structure.shape(pair[0])?.clone()),
            Face::unchecked(structure.shape(pair[1])?.clone()),
        ])
    }

    fn append_empty_face_face_records(&mut self, pairs: &[[i32; 2]]) -> Result<(), Failure> {
        let structure = self.data_structure.as_mut().ok_or_else(missing_structure)?;
        for pair in pairs {
            structure.interf_ff().append(empty_face_face_record(*pair));
        }
        Ok(())
    }

    fn store_face_face_results(
        &mut self,
        tasks: Vec<BOPAlgoFaceFaceTask<'static>>,
    ) -> Result<(), Failure> {
        for mut task in tasks {
            if task.parallel_options().has_errors() || !task.intersector().is_done() {
                self.warn_failed_intersection();
                self.append_empty_face_face_records(&[task.indices()])?;
                continue;
            }
            self.store_face_face_result(&mut task)?;
        }
        Ok(())
    }

    fn store_face_face_result(
        &mut self,
        task: &mut BOPAlgoFaceFaceTask<'static>,
    ) -> Result<(), Failure> {
        task.intersector_mut().prepare_lines_3d(false)?;
        task.apply_trsf()?;
        let curves = task.intersector().lines()?.to_vec();
        let points = task.intersector().points().to_vec();
        let pair = task.indices();
        let record = build_face_face_record(task, &curves, &points)?;
        let has_results = record.curves().length() > 0 || record.points().length() > 0;
        let structure = self.data_structure.as_mut().ok_or_else(missing_structure)?;
        if has_results {
            structure.add_interference(pair[0], pair[1]);
        }
        structure.interf_ff().append(record);
        Ok(())
    }

    fn warn_failed_intersection(&self) {
        let alert = Arc::new(AlertIntersectionFailed) as AlertHandle;
        self.options.add_warning(&alert);
    }
}

fn build_face_face_record(
    task: &BOPAlgoFaceFaceTask<'static>,
    curves: &[Curve],
    points: &[crate::int_tools::PntOn2Faces],
) -> Result<InterfFF, Failure> {
    let mut record = InterfFF::default();
    let pair = task.indices();
    record.set_indices(pair[0], pair[1]);
    record.set_tangent_faces(task.intersector().tangent_faces());
    record.init(curves.len() as i32, points.len() as i32);
    append_face_face_curves(&mut record, curves, task.tol_ff(), task)?;
    append_face_face_points(&mut record, points);
    Ok(record)
}

fn configure_task(
    filler: &BOPAlgoPaveFiller,
    task: &mut BOPAlgoFaceFaceTask<'static>,
    pair: [i32; 2],
    faces: &[Face; 2],
    boxes: &[rustycad_tkmath::bnd::BoundingBox3; 2],
    tolerance: f64,
    context: &Handle<Context>,
) {
    task.set_indices(pair[0], pair[1]);
    task.set_faces(&faces[0], &faces[1]);
    task.set_boxes(&boxes[0], &boxes[1]);
    task.set_tol_ff(tolerance);
    configure_task_options(filler, task, context);
}

fn configure_task_options(
    filler: &BOPAlgoPaveFiller,
    task: &mut BOPAlgoFaceFaceTask<'static>,
    context: &Handle<Context>,
) {
    task.set_parameters(
        filler.section_attribute.approximation(),
        filler.section_attribute.pcurve_on_s1(),
        filler.section_attribute.pcurve_on_s2(),
        1e-7,
    );
    task.set_fuzzy_value(filler.options.fuzzy_value());
    task.set_context(context);
    task.set_run_parallel(filler.options.run_parallel());
}

fn is_plane_pair(faces: &[Face; 2], context: &Context) -> Result<bool, Failure> {
    let first = context.surface_adaptor(&faces[0])?;
    let second = context.surface_adaptor(&faces[1])?;
    Ok(first.read().get_type()? == SurfaceType::Plane
        && second.read().get_type()? == SurfaceType::Plane)
}

fn empty_face_face_record(pair: [i32; 2]) -> InterfFF {
    let mut record = InterfFF::default();
    record.set_indices(pair[0], pair[1]);
    record.init(0, 0);
    record
}

fn append_face_face_curves(
    record: &mut InterfFF,
    curves: &[Curve],
    tolerance: f64,
    task: &BOPAlgoFaceFaceTask<'static>,
) -> Result<(), Failure> {
    let expansion = tolerance + maximum_face_vertex_tolerance(task)?;
    for curve in curves {
        let mut bounds = rustycad_tkmath::bnd::BoundingBox3::default();
        if !crate::int_tools::Tools::check_curve(curve, &mut bounds)? {
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

fn maximum_face_vertex_tolerance(task: &BOPAlgoFaceFaceTask<'static>) -> Result<f64, Failure> {
    Ok(BRepTool::max_tolerance(task.face1(), ShapeEnum::Vertex)?
        .max(BRepTool::max_tolerance(task.face2(), ShapeEnum::Vertex)?))
}

fn append_face_face_points(record: &mut InterfFF, points: &[crate::int_tools::PntOn2Faces]) {
    for pair in points {
        record.change_points().appended().set_pnt(*pair.p1().pnt());
    }
}

fn face_pair_tolerance(faces: &[Face; 2], context: &Handle<Context>) -> Result<f64, Failure> {
    let types = face_surface_types(faces, context)?;
    let maximum = BRepTool::face_tolerance(&faces[0])?.max(BRepTool::face_tolerance(&faces[1])?);
    Ok(raised_face_tolerance(maximum, types))
}

fn face_surface_types(
    faces: &[Face; 2],
    context: &Handle<Context>,
) -> Result<[SurfaceType; 2], Failure> {
    let context = context.get().ok_or_else(missing_context)?;
    let first = context.surface_adaptor(&faces[0])?;
    let second = context.surface_adaptor(&faces[1])?;
    Ok([first.read().get_type()?, second.read().get_type()?])
}

fn raised_face_tolerance(maximum: f64, types: [SurfaceType; 2]) -> f64 {
    if types.iter().all(|kind| is_analytic_surface(*kind)) {
        return maximum;
    }
    maximum.max(5e-6)
}

fn is_analytic_surface(kind: SurfaceType) -> bool {
    matches!(
        kind,
        SurfaceType::Plane
            | SurfaceType::Cylinder
            | SurfaceType::Cone
            | SurfaceType::Sphere
            | SurfaceType::Torus
    )
}

fn missing_structure() -> Failure {
    Failure::new("BOPAlgo_PaveFiller::PerformFF requires initialized data structure")
}

fn missing_iterator() -> Failure {
    Failure::new("BOPAlgo_PaveFiller::PerformFF requires initialized iterator")
}

fn missing_context() -> Failure {
    Failure::new("BOPAlgo_PaveFiller::PerformFF requires initialized geometric context")
}
