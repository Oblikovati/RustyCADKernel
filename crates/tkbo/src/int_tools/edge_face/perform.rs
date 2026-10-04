use super::EdgeFace;
use super::classify::{append_projectable_ranges, classify_common_parts};
use super::postprocess::postprocess_touch_parts;
use super::quick_coincidence::is_coincident;
use crate::int_tools::{BeanFaceIntersector, CommonPrt};
use rustycad_tkbrep::{brep::BRepTool, brep_adaptor::BRepAdaptorCurve};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkmath::geom_abs::CurveType;

impl EdgeFace {
    /// Performs source checks, BeanFace range generation and common-part classification.
    pub fn perform(&mut self) -> Result<(), Failure> {
        self.error_status = 0;
        self.check_data()?;
        if self.error_status != 0 {
            return Ok(());
        }
        self.ensure_context();
        self.done = false;
        self.initialize_adaptors()?;
        if self.quick_coincidence_check && self.quick_coincidence()? {
            return Ok(());
        }
        self.perform_bean_face()
    }

    fn check_data(&mut self) -> Result<(), Failure> {
        if BRepTool::degenerated(&self.edge)? {
            self.error_status = 2;
        }
        if !BRepTool::is_geometric_edge(&self.edge)? {
            self.error_status = 3;
        }
        Ok(())
    }

    fn ensure_context(&mut self) {
        if self.context.is_null() {
            self.context = Handle::new(crate::int_tools::Context::default());
        }
    }

    fn initialize_adaptors(&mut self) -> Result<(), Failure> {
        self.curve = Handle::new(BRepAdaptorCurve::with_edge(&self.edge)?);
        let cached = self
            .context
            .get()
            .expect("edge-face context")
            .surface_adaptor(&self.face)?;
        self.surface = cached.read().shallow_copy()?.cast();
        let curve = self.curve.get().expect("initialized edge-face curve");
        let (edge_tol, face_tol) = (
            BRepTool::edge_tolerance(&self.edge)?,
            BRepTool::face_tolerance(&self.face)?,
        );
        self.criteria = edge_face_criteria(curve.get_type()?, edge_tol, face_tol, self.fuzzy_value);
        Ok(())
    }

    fn quick_coincidence(&mut self) -> Result<bool, Failure> {
        let curve = self.curve.get().expect("initialized edge-face curve");
        let surface = self.surface.get().expect("initialized edge-face surface");
        if !is_coincident(
            curve,
            surface,
            self.context.get().expect("edge-face context"),
            &self.face,
            self.range,
            self.criteria,
        )? {
            return Ok(false);
        }
        let mut part = CommonPrt::default();
        part.set_edge1(&self.edge);
        part.set_type(rustycad_tkg3d::top_abs::ShapeEnum::Edge);
        part.set_range1(self.range);
        self.common_parts.append(part);
        self.done = true;
        Ok(true)
    }

    fn perform_bean_face(&mut self) -> Result<(), Failure> {
        let curve = self.curve.get().expect("initialized edge-face curve");
        let surface = self.surface.get().expect("initialized edge-face surface");
        let tolerances = self.intersection_tolerances()?;
        let mut intersector =
            BeanFaceIntersector::from_adaptors(curve, surface, tolerances.0, tolerances.1)?;
        intersector.set_bean_parameters(self.range.first(), self.range.last());
        intersector.set_context(&self.context);
        intersector.perform()?;
        self.update_minimum_distance(intersector.minimal_square_distance());
        if !intersector.is_done() {
            return Ok(());
        }
        append_projectable_ranges(self, intersector.result())?;
        classify_common_parts(self)?;
        postprocess_touch_parts(self)
    }

    fn intersection_tolerances(&self) -> Result<(f64, f64), Failure> {
        Ok((
            BRepTool::edge_tolerance(&self.edge)? + self.fuzzy_value * 0.5,
            BRepTool::face_tolerance(&self.face)? + self.fuzzy_value * 0.5,
        ))
    }

    fn update_minimum_distance(&mut self, squared: f64) {
        if squared < rustycad_tkernel::standard::real::real_last() {
            self.minimum_distance = squared.sqrt();
        }
    }
}

fn edge_face_criteria(curve: CurveType, edge: f64, face: f64, fuzz: f64) -> f64 {
    let (edge_tol, face_tol) = (edge + fuzz * 0.5, face + fuzz * 0.5);
    if matches!(curve, CurveType::BSplineCurve | CurveType::BezierCurve) {
        let first_ratio = edge_tol / face_tol;
        let second_ratio = face_tol / edge_tol;
        if first_ratio > 100.0 || second_ratio > 100.0 {
            return if edge_tol > face_tol {
                edge_tol
            } else {
                face_tol
            };
        }
        return 1.5 * edge_tol + face_tol;
    }
    edge_tol + face_tol
}
