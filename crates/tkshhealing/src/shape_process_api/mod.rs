// Rust translation of ModelingAlgorithms/TKShHealing/ShapeProcessAPI at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package ShapeProcessAPI: `ShapeProcessAPI_ApplySequence`, the API that applies a
//! ShapeProcess sequence of a resource file to a shape.
use crate::shape_process::{
    ShapeProcess, ShapeProcessContext, ShapeProcessOperLibrary, ShapeProcessShapeContext,
    ShapeProcessShapeMap,
};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;

/// `ShapeProcessAPI_ApplySequence`: a shape context on a resource file and the name of
/// the sequence to run.
///
/// ```
/// use rustycad_tkbrep::topods::Shape;
/// use rustycad_tkshhealing::shape_process_api::ShapeProcessAPIApplySequence;
/// let mut api = ShapeProcessAPIApplySequence::new("", "read").unwrap();
/// assert!(api.prepare_shape(&Shape::default(), false, rustycad_tkg3d::top_abs::ShapeEnum::Shape, &Default::default()).unwrap().is_null());
/// ```
pub struct ShapeProcessAPIApplySequence {
    context: Handle<ShapeProcessShapeContext>,
    sequence: String,
}

impl ShapeProcessAPIApplySequence {
    /// `ShapeProcessAPI_ApplySequence(rscName, seqName)`: a shape context on the resource
    /// file (detalisation FACE) and the operator library initialized.
    /// Example: `ShapeProcessAPIApplySequence::new("DEXCAF", "read.step")?`.
    pub fn new(resource: &str, sequence: &str) -> Result<Self, Failure> {
        let context = ShapeProcessShapeContext::new(resource, "")?;
        context.set_detalisation(ShapeEnum::Face);
        ShapeProcessOperLibrary::init()?;
        Ok(Self {
            context: Handle::new(context),
            sequence: sequence.to_owned(),
        })
    }

    /// `Context()`: the shape context (shared).
    pub fn context(&self) -> Handle<ShapeProcessShapeContext> {
        self.context.clone()
    }

    /// Writes the non-const `Context()` reference; `api.set_context(&context)`.
    pub fn set_context(&mut self, context: &Handle<ShapeProcessShapeContext>) {
        self.context = context.clone();
    }

    /// `PrepareShape(shape, fillmap, until, theProgress)`: the context initialized on the
    /// shape, the sequence run when the resource file defines `<seq>.exec.op`, and the
    /// result; a null shape is returned as is. `fillmap` and `until` are unused natively.
    /// Example: `api.prepare_shape(&shape, false, ShapeEnum::Shape, &range)?`.
    pub fn prepare_shape(
        &mut self,
        shape: &Shape,
        _fill_map: bool,
        _until: ShapeEnum,
        progress: &ProgressRange<'_>,
    ) -> Result<Shape, Failure> {
        if shape.is_null() {
            return Ok(shape.clone());
        }
        let ctx = self.context.get().ok_or_else(null_context)?;
        let resource = ctx.resource_manager().ok_or_else(null_context)?;
        ctx.init(shape);
        let key = format!("{}.exec.op", self.sequence);
        if resource.read().find(&key) {
            let base: Handle<ShapeProcessContext> = self.context.clone().cast();
            ShapeProcess::perform_sequence(&base, &self.sequence, progress)?;
        }
        Ok(ctx.result())
    }

    /// `ClearMap()`: does nothing ("myContext->Map().Clear()" is commented out natively).
    pub fn clear_map(&mut self) {}

    /// `Map()`: a copy of the map of the context.
    pub fn map(&self) -> Result<ShapeProcessShapeMap, Failure> {
        Ok(self.context.get().ok_or_else(null_context)?.map())
    }

    /// `PrintPreparationResult()`: the same report as
    /// `ShapeProcess_ShapeContext::PrintStatistics` (the native bodies are identical).
    pub fn print_preparation_result(&self) -> Result<(), Failure> {
        self.context
            .get()
            .ok_or_else(null_context)?
            .print_statistics()
    }
}

fn null_context() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "ShapeProcessAPI_ApplySequence: null shape context or resource manager",
        "",
    )
}
