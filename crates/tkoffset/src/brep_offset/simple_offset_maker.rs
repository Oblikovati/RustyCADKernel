mod walls;

// Translation of BRepOffset_MakeSimpleOffset at OCCT
// 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::{BRepOffsetSimpleOffset, build_simple_offset_wall, compute_simple_offset_max_angle};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::{BRepToolsModification, Modifier, ReShape};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Edge, Shape, Solid, Vertex};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkshhealing::shape_build::ShapeBuildReShape;

/// `BRepOffsetSimple_Status`: the state of a simple offset attempt.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(i32)]
pub enum BRepOffsetSimpleStatus {
    /// `BRepOffsetSimple_OK`.
    #[default]
    Ok = 0,
    /// `BRepOffsetSimple_NullInputShape`.
    NullInputShape = 1,
    /// `BRepOffsetSimple_ErrorOffsetComputation`.
    ErrorOffsetComputation = 2,
    /// `BRepOffsetSimple_ErrorWallFaceComputation`.
    ErrorWallFaceComputation = 3,
    /// `BRepOffsetSimple_ErrorInvalidNbShells`.
    ErrorInvalidNbShells = 4,
    /// `BRepOffsetSimple_ErrorNonClosedShell`.
    ErrorNonClosedShell = 5,
}

/// Stateful shell-producing `BRepOffset_MakeSimpleOffset` orchestration.
///
/// Shell results preserve the native modifier history. Solid mode rebuilds walls for the
/// currently ported boundary cases.
pub struct BRepOffsetMakeSimpleOffset {
    input_shape: Shape,
    offset_value: f64,
    tolerance: f64,
    build_solid: bool,
    max_angle: f64,
    error: BRepOffsetSimpleStatus,
    done: bool,
    modifier: Modifier,
    reshape: Handle<ShapeBuildReShape>,
    result_shape: Shape,
    wall_edges: DataMap<Vertex, Edge>,
}

impl Default for BRepOffsetMakeSimpleOffset {
    fn default() -> Self {
        Self {
            input_shape: Shape::default(),
            offset_value: 0.0,
            tolerance: rustycad_tkernel::precision::confusion(),
            build_solid: false,
            max_angle: 0.0,
            error: BRepOffsetSimpleStatus::Ok,
            done: false,
            modifier: Modifier::default(),
            reshape: Handle::new(ShapeBuildReShape::new()),
            result_shape: Shape::default(),
            wall_edges: DataMap::default(),
        }
    }
}

impl BRepOffsetMakeSimpleOffset {
    /// `BRepOffset_MakeSimpleOffset()`: no input shape or result.
    ///
    /// ```
    /// use rustycad_tkoffset::brep_offset::BRepOffsetMakeSimpleOffset;
    /// let maker = BRepOffsetMakeSimpleOffset::new();
    /// assert!(!maker.is_done());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// `BRepOffset_MakeSimpleOffset(shape, offset)`: initialize with an input and offset.
    ///
    /// ```
    /// use rustycad_tkoffset::brep_offset::BRepOffsetMakeSimpleOffset;
    /// use rustycad_tkbrep::topods::Shape;
    /// let maker = BRepOffsetMakeSimpleOffset::with_shape(&Shape::default(), 1.0);
    /// assert_eq!(maker.offset_value(), 1.0);
    /// ```
    pub fn with_shape(shape: &Shape, offset: f64) -> Self {
        let mut maker = Self::new();
        maker.input_shape = shape.clone();
        maker.offset_value = offset;
        maker
    }

    /// `Initialize(shape, offset)`: update inputs and clear run state.
    pub fn initialize(&mut self, shape: &Shape, offset: f64) {
        self.input_shape = shape.clone();
        self.offset_value = offset;
        self.clear();
    }

    /// `Perform()`: rebuild the offset shell and retain the modifier for history queries.
    pub fn perform(&mut self) -> Result<(), Failure> {
        self.clear();
        if self.input_shape.is_null() {
            self.error = BRepOffsetSimpleStatus::NullInputShape;
            return Ok(());
        }
        self.prepare_shell_modifier()?;
        if self.error != BRepOffsetSimpleStatus::Ok {
            return Ok(());
        }
        self.restore_degenerated_edges()?;
        if self.build_solid {
            self.build_solid_result()?;
            if self.error != BRepOffsetSimpleStatus::Ok {
                return Ok(());
            }
        }
        self.done = true;
        Ok(())
    }

    fn prepare_shell_modifier(&mut self) -> Result<(), Failure> {
        self.ensure_max_angle()?;
        self.modifier.init(&self.input_shape)?;
        self.run_shell_mapping()?;
        self.capture_result_shape()
    }

    fn ensure_max_angle(&mut self) -> Result<(), Failure> {
        if self.max_angle == 0.0 {
            self.max_angle = compute_simple_offset_max_angle(&self.input_shape)?;
        }
        Ok(())
    }

    fn run_shell_mapping(&mut self) -> Result<(), Failure> {
        let mapping =
            BRepOffsetSimpleOffset::new(&self.input_shape, self.offset_value, self.tolerance)?;
        let mapping: Handle<dyn BRepToolsModification> = Handle::new(mapping).cast();
        self.modifier.perform(&mapping, &ProgressRange::default())?;
        if !self.modifier.is_done() {
            self.error = BRepOffsetSimpleStatus::ErrorOffsetComputation;
        }
        Ok(())
    }

    fn capture_result_shape(&mut self) -> Result<(), Failure> {
        if self.error != BRepOffsetSimpleStatus::Ok {
            return Ok(());
        }
        self.result_shape = self.modifier.modified_shape(&self.input_shape)?;
        Ok(())
    }

    fn restore_degenerated_edges(&self) -> Result<(), Failure> {
        let mut explorer = Explorer::new(&self.input_shape, ShapeEnum::Edge, ShapeEnum::Shape)?;
        let builder = BRepBuilder::default();
        while explorer.more() {
            mark_degenerated_image(
                &self.modifier,
                &builder,
                Edge::unchecked(explorer.current().clone()),
            )?;
            explorer.next()?;
        }
        Ok(())
    }

    /// `Clear()`: clear done/error/cache/history, preserving inputs, output and build flag.
    fn clear(&mut self) {
        self.done = false;
        self.error = BRepOffsetSimpleStatus::Ok;
        self.max_angle = 0.0;
        self.reshape
            .get()
            .expect("reshape handle is initialized")
            .clear();
        self.wall_edges.clear(true);
    }

    fn build_solid_result(&mut self) -> Result<(), Failure> {
        let shell = match walls::make_solid(
            &self.input_shape,
            &self.result_shape,
            &self.modifier,
            &self.reshape,
            &mut self.wall_edges,
        )? {
            Ok(shell) => shell,
            Err(status) => {
                self.error = status;
                return Ok(());
            }
        };
        self.result_shape = make_solid(&shell)?;
        Ok(())
    }

    /// `GetErrorMessage()`: the native text for the current status.
    pub fn error_message(&self) -> &'static str {
        match self.error {
            BRepOffsetSimpleStatus::Ok => "",
            BRepOffsetSimpleStatus::NullInputShape => "Null input shape",
            BRepOffsetSimpleStatus::ErrorOffsetComputation => "Error during offset construction",
            BRepOffsetSimpleStatus::ErrorWallFaceComputation => "Error during building wall face",
            BRepOffsetSimpleStatus::ErrorInvalidNbShells => "Result contains two or more shells",
            BRepOffsetSimpleStatus::ErrorNonClosedShell => "Result shell is not closed",
        }
    }

    /// `GetSafeOffset(expectedTolerance)`: estimate a distance from max vertex tolerance.
    pub fn safe_offset(&mut self, expected_tolerance: f64) -> Result<f64, Failure> {
        if self.input_shape.is_null() {
            return Ok(0.0);
        }
        if self.max_angle == 0.0 {
            self.max_angle = compute_simple_offset_max_angle(&self.input_shape)?;
        }
        let max_tolerance = BRepTool::max_tolerance(&self.input_shape, ShapeEnum::Vertex)?;
        let estimate = (expected_tolerance - max_tolerance) / (2.0 * self.max_angle);
        Ok(if estimate < 0.0 { 0.0 } else { estimate })
    }

    /// `Generated(shape)`: return the offset image after reshape history.
    pub fn generated(&self, shape: &Shape) -> Result<Shape, Failure> {
        let generated = self.modifier.modified_shape(shape)?;
        if generated.is_null() {
            return Ok(generated);
        }
        self.reshape
            .get()
            .expect("reshape handle is initialized")
            .apply(&generated, ShapeEnum::Shape)
    }

    /// `Modified(shape)`: return a reshape replacement or a null shape.
    pub fn modified(&self, shape: &Shape) -> Result<Shape, Failure> {
        let (status, result) = self
            .reshape
            .get()
            .expect("reshape handle is initialized")
            .status(shape, false)?;
        Ok(if status == 0 {
            Shape::default()
        } else {
            result
        })
    }

    /// `GetError()`; `maker.error()`.
    pub fn error(&self) -> BRepOffsetSimpleStatus {
        self.error
    }

    /// `IsDone()`; `maker.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }

    /// `GetResultShape()`; `maker.result_shape()`.
    pub fn result_shape(&self) -> &Shape {
        &self.result_shape
    }

    /// `GetBuildSolidFlag()`; `maker.build_solid_flag()`.
    pub fn build_solid_flag(&self) -> bool {
        self.build_solid
    }

    /// `SetBuildSolidFlag(flag)`; `maker.set_build_solid_flag(true)`.
    pub fn set_build_solid_flag(&mut self, build_solid: bool) {
        self.build_solid = build_solid;
    }

    /// `GetOffsetValue()`; `maker.offset_value()`.
    pub fn offset_value(&self) -> f64 {
        self.offset_value
    }

    /// `SetOffsetValue(offset)`; `maker.set_offset_value(1.0)`.
    pub fn set_offset_value(&mut self, offset: f64) {
        self.offset_value = offset;
    }

    /// `GetTolerance()`; `maker.tolerance()`.
    pub fn tolerance(&self) -> f64 {
        self.tolerance
    }

    /// `SetTolerance(tolerance)`; `maker.set_tolerance(1e-7)`.
    pub fn set_tolerance(&mut self, tolerance: f64) {
        self.tolerance = tolerance;
    }
}

fn make_solid(shell: &rustycad_tkbrep::topods::Shell) -> Result<Shape, Failure> {
    let builder = BRepBuilder::default();
    let mut solid = Solid::default();
    builder.make_solid(&mut solid);
    builder.add(&mut solid, shell)?;
    Ok(solid.into_shape())
}

fn mark_degenerated_image(
    modifier: &Modifier,
    builder: &BRepBuilder,
    edge: Edge,
) -> Result<(), Failure> {
    if !BRepTool::degenerated(&edge)? {
        return Ok(());
    }
    let image = Edge::unchecked(modifier.modified_shape(&edge)?);
    builder.degenerated(&image, true)
}
