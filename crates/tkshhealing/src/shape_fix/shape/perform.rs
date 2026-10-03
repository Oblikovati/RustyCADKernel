use super::*;
use crate::shape_fix::{ShapeFix, need_fix};
use rustycad_tkbrep::{
    topexp::Explorer,
    topods::{self, ShapeIterator},
};
use rustycad_tkernel::message::{ProgressRange, ProgressScope};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::toploc::Location;
impl ShapeFixShape {
    /// Dispatch repairs with native progress stages; `tool.perform_with_progress(&progress)?`.
    pub fn perform_with_progress(&self, progress: &ProgressRange<'_>) -> Result<bool, Failure> {
        let input = self.input_shape();
        if self.context().is_null() {
            return Err(Failure::new(
                "ShapeFix_Shape::Perform: null context; expected Init(shape) before Perform()",
            ));
        }
        // Existing Rust null-shape support is explicit; native ShapeType throws here.
        if input.is_null() {
            *self.result.write() = input;
            *self.status.write() = 0;
            return Ok(false);
        }
        self.perform_loaded_shape(&input, progress)
    }
    fn perform_loaded_shape(
        &self,
        input: &Shape,
        progress: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        let saved_area = self.prepare_face_mode(input)?;
        *self.status.write() = 0;
        if self.already_fixed(input)? {
            *self.result.write() = self.context().apply(input, ShapeEnum::Shape)?;
            return Ok(true);
        }
        let shape = self.prepare_vertices(input)?;
        let scope = ProgressScope::with_literal(progress, "Fixing stage", 2., false);
        let changed = self.dispatch_repair(&shape, &scope)?;
        if !scope.more() || !self.finish_repairs(&shape, &scope)? {
            return Ok(false);
        }
        self.fix_face_tool()
            .set_fix_small_area_wire_mode(saved_area);
        Ok(changed)
    }
    fn prepare_vertices(&self, input: &Shape) -> Result<Shape, Failure> {
        let mut shape = self.context().apply(input, ShapeEnum::Shape)?;
        if need_fix(self.fix_vertex_position_mode(), true) {
            ShapeFix::fix_vertex_position(&mut shape, self.precision(), &self.context())?;
        }
        Ok(shape)
    }
    fn input_shape(&self) -> Shape {
        self.fix_root_data().with(|fields| fields.shape.clone())
    }
    fn set_input(&self, shape: Shape) {
        self.fix_root_data().with_mut(|fields| fields.shape = shape);
    }
    fn prepare_face_mode(&self, shape: &Shape) -> Result<i32, Failure> {
        let face = self.fix_face_tool();
        let saved = face.fix_small_area_wire_mode();
        if saved == -1 && shape.shape_type()? == ShapeEnum::Face {
            face.set_fix_small_area_wire_mode(1);
        }
        Ok(saved)
    }
    fn already_fixed(&self, input: &Shape) -> Result<bool, Failure> {
        let mut unlocated = input.clone();
        unlocated.set_location(&Location::default(), true)?;
        if self.context().is_new_shape(input) || self.visited.read().contains(&unlocated) {
            return Ok(true);
        }
        self.visited.write().add(unlocated);
        Ok(false)
    }
    fn dispatch_repair(&self, shape: &Shape, scope: &ProgressScope<'_>) -> Result<bool, Failure> {
        match shape.shape_type()? {
            ShapeEnum::Compound | ShapeEnum::CompSolid => self.repair_children(shape, scope),
            ShapeEnum::Solid if need_fix(self.fix_solid_mode(), true) => {
                self.repair_solid(shape, &scope.next(1.))
            }
            ShapeEnum::Shell if need_fix(self.fix_free_shell_mode(), true) => {
                self.repair_shell(shape, &scope.next(1.))
            }
            ShapeEnum::Face if need_fix(self.fix_free_face_mode(), true) => self.repair_face(shape),
            ShapeEnum::Wire if need_fix(self.fix_free_wire_mode(), true) => {
                Err(Failure::with_type(
                    "Standard_NotImplemented",
                    "ShapeFix_Shape::Perform: free wire dispatch requires ShapeFix_Wire::Perform; expected ported aggregate wire repair",
                    "",
                ))
            }
            ShapeEnum::Edge => self.repair_edge(shape),
            _ => Ok(false),
        }
    }
    fn repair_children(&self, shape: &Shape, scope: &ProgressScope<'_>) -> Result<bool, Failure> {
        let original = self.input_shape();
        let saved_same = self.fix_same_parameter_mode();
        let saved_vertices = self.fix_vertex_tol_mode();
        self.set_fix_same_parameter_mode(0);
        self.set_fix_vertex_tol_mode(0);
        let range = scope.next(1.);
        let children = ProgressScope::with_literal(
            &range,
            "Fixing sub-shape",
            f64::from(shape.nb_children()),
            false,
        );
        let Some(changed) = self.repair_child_sequence(shape, &children)? else {
            return Ok(false);
        };
        self.set_fix_same_parameter_mode(saved_same);
        self.set_fix_vertex_tol_mode(saved_vertices);
        self.set_input(original);
        Ok(changed)
    }
    fn repair_child_sequence(
        &self,
        shape: &Shape,
        children: &ProgressScope<'_>,
    ) -> Result<Option<bool>, Failure> {
        let mut changed = false;
        for child in ShapeIterator::new(shape, true, true) {
            if !children.more() {
                break;
            }
            self.set_input(child);
            changed |= self.perform_with_progress(&children.next(1.))?;
        }
        Ok(children.more().then_some(changed))
    }
    fn repair_solid(&self, shape: &Shape, progress: &ProgressRange<'_>) -> Result<bool, Failure> {
        let solid = self.fix_solid_tool();
        solid.init(&topods::solid(shape)?);
        solid.set_context(&self.context());
        let changed = solid.perform_with_progress(progress)?;
        // Pinned OCCT sets DONE4 for solids too, despite its header saying DONE5.
        *self.status.write() |= ShapeExtend::encode_status(ShapeExtendStatus::Done4);
        Ok(changed)
    }
    fn repair_shell(&self, shape: &Shape, progress: &ProgressRange<'_>) -> Result<bool, Failure> {
        let shell = self.fix_shell_tool();
        shell.init(&topods::shell(shape)?);
        shell.set_context(&self.context());
        let changed = shell.perform_with_progress(progress)?;
        *self.status.write() |= ShapeExtend::encode_status(ShapeExtendStatus::Done4);
        Ok(changed)
    }
    fn repair_face(&self, shape: &Shape) -> Result<bool, Failure> {
        let face = self.fix_face_tool();
        let wire = face.fix_wire_tool();
        let saved = wire.topology_mode();
        wire.set_topology_mode(true);
        face.init(shape.clone());
        face.set_context(&self.context());
        let changed = face.perform()?;
        wire.set_topology_mode(saved);
        *self.status.write() |= ShapeExtend::encode_status(ShapeExtendStatus::Done3);
        Ok(changed)
    }
    fn repair_edge(&self, shape: &Shape) -> Result<bool, Failure> {
        let edge = self.fix_edge_tool();
        edge.set_context(&self.context());
        if edge.fix_vertex_tolerance(&topods::edge(shape)?)? {
            *self.status.write() |= ShapeExtend::encode_status(ShapeExtendStatus::Done1);
        }
        Ok(false)
    }
    fn finish_repairs(&self, shape: &Shape, scope: &ProgressScope<'_>) -> Result<bool, Failure> {
        *self.result.write() = self.context().apply(shape, ShapeEnum::Shape)?;
        if need_fix(self.fix_same_parameter_mode(), true) {
            self.same_parameter(&self.shape(), false, &scope.next(1.))?;
            if !scope.more() {
                return Ok(false);
            }
        }
        if need_fix(self.fix_vertex_tol_mode(), true) {
            self.fix_shared_vertex_tolerances()?;
        }
        let result = self.context().apply(&self.shape(), ShapeEnum::Shape)?;
        *self.result.write() = result;
        Ok(true)
    }
    /// Apply the native final parameter repair; `tool.same_parameter(&shape,false,&progress)?`.
    pub fn same_parameter(
        &self,
        shape: &Shape,
        enforce: bool,
        progress: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        ShapeFix::same_parameter(shape, enforce, 0., progress, &Handle::default())?;
        Ok(())
    }
    fn fix_shared_vertex_tolerances(&self) -> Result<(), Failure> {
        let shape = self.shape();
        let faces = Explorer::new(&shape, ShapeEnum::Face, ShapeEnum::Shape)?.collect::<Vec<_>>();
        if faces.len() < 2 {
            return Ok(());
        }
        // OCC25455: cutting edges can leave shared vertices with stale face tolerances.
        let fixer = self.fix_edge_tool();
        for face in faces {
            self.fix_face_vertex_tolerances(&fixer, &topods::face(&face)?)?;
        }
        Ok(())
    }
    fn fix_face_vertex_tolerances(
        &self,
        fixer: &super::super::ShapeFixEdge,
        face: &topods::Face,
    ) -> Result<(), Failure> {
        for edge in Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape)? {
            fixer.fix_vertex_tolerance_on_face(&topods::edge(&edge)?, face)?;
        }
        Ok(())
    }
}
