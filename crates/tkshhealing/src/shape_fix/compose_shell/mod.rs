//! Shell composition state and public operations from `ShapeFix_ComposeShell`.
mod break_wires;
mod compute_code;
mod geometry;
mod grid;
mod init;
mod load_wires;

use crate::shape_analysis::{ShapeAnalysisTransferParametersProj, TransferParameters};
use crate::shape_extend::{ShapeExtend, ShapeExtendCompositeSurface};
use crate::shape_fix::{FixRoot, FixRootData, ShapeFixRoot};
use rustycad_tkbrep::topods::{Face, Shape};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Handle, SharedState};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::toploc::Location;

/// Mutable native fields of `ShapeFix_ComposeShell`.
#[derive(Clone)]
struct ComposeShellFields {
    grid: Handle<ShapeExtendCompositeSurface>,
    location: Location,
    face: Face,
    orientation: Orientation,
    result: Shape,
    status: i32,
    u_resolution: f64,
    v_resolution: f64,
    transfer_tool: Handle<dyn TransferParameters>,
    invert_edge_status: bool,
    closed_mode: bool,
    u_closed: bool,
    v_closed: bool,
    u_period: f64,
    v_period: f64,
}

impl Default for ComposeShellFields {
    fn default() -> Self {
        Self {
            grid: Handle::default(),
            location: Location::default(),
            face: Face::default(),
            orientation: Orientation::Forward,
            result: Shape::default(),
            status: 0,
            u_resolution: rustycad_tkernel::precision::infinite(),
            v_resolution: rustycad_tkernel::precision::infinite(),
            transfer_tool: Handle::new(ShapeAnalysisTransferParametersProj::new())
                .cast::<dyn TransferParameters>(),
            invert_edge_status: true,
            closed_mode: false,
            u_closed: false,
            v_closed: false,
            u_period: 0.0,
            v_period: 0.0,
        }
    }
}

/// Splits a face's wires on a composite surface and builds patch faces.
/// `let compose = ShapeFixComposeShell::new();`
pub struct ShapeFixComposeShell {
    parent: ShapeFixRoot,
    fields: SharedState<ComposeShellFields>,
}

impl_transient!(
    ShapeFixComposeShell,
    "ShapeFix_ComposeShell",
    ShapeFixRoot,
    parent,
    interfaces: [dyn FixRoot]
);

impl Default for ShapeFixComposeShell {
    fn default() -> Self {
        Self {
            parent: ShapeFixRoot::default(),
            fields: SharedState::new(ComposeShellFields::default()),
        }
    }
}

impl FixRoot for ShapeFixComposeShell {
    fn fix_root_data(&self) -> &FixRootData {
        self.parent.fix_root_data()
    }
}

impl ShapeFixComposeShell {
    /// Creates an empty shell composer; `ShapeFixComposeShell::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Initializes a composite grid, supporting face, and precision.
    /// `composer.init(&grid, &location, &face, 1e-7)?`.
    pub fn init(
        &self,
        grid: &Handle<ShapeExtendCompositeSurface>,
        location: &Location,
        face: &Face,
        precision: f64,
    ) -> Result<(), Failure> {
        init::initialize(self, grid, location, face, precision)
    }

    /// Reads the closed-surface modulo-period mode; `assert!(!composer.closed_mode())`.
    pub fn closed_mode(&self) -> bool {
        self.fields.read().closed_mode
    }

    /// Enables closed-surface modulo-period handling; `composer.set_closed_mode(true)`.
    pub fn set_closed_mode(&self, enabled: bool) {
        self.fields.write().closed_mode = enabled;
    }

    /// Returns the last built face or shell; `let result = composer.result()`.
    pub fn result(&self) -> Shape {
        self.fields.read().result.clone()
    }

    /// Decodes the last operation's status; `composer.status(ShapeExtendStatus::Done1)`.
    pub fn status(&self, status: crate::shape_extend::ShapeExtendStatus) -> bool {
        ShapeExtend::decode_status(self.fields.read().status, status)
    }

    /// Replaces the parameter transfer strategy, preserving dynamic dispatch.
    /// `composer.set_transfer_parameter_tool(&tool.cast())`.
    pub fn set_transfer_param_tool(&self, tool: &Handle<dyn TransferParameters>) {
        self.fields.write().transfer_tool = tool.clone();
    }

    /// Returns the current transfer strategy; `let tool = composer.transfer_parameter_tool()`.
    pub fn get_transfer_param_tool(&self) -> Handle<dyn TransferParameters> {
        self.fields.read().transfer_tool.clone()
    }

    /// Splits loaded wire segments at external joint-line vertices.
    /// `composer.break_wires(&mut segments)?`.
    pub fn break_wires(
        &self,
        segments: &mut rustycad_tkernel::ncollection::Sequence<
            crate::shape_fix::ShapeFixWireSegment,
        >,
    ) -> Result<(), Failure> {
        break_wires::break_wires(self, segments)
    }

    /// Loads face wires into oriented segments after applying the reshape context.
    /// `let segments = composer.load_wires()?` after `set_context` and `init`.
    pub fn load_wires(
        &self,
    ) -> Result<
        rustycad_tkernel::ncollection::Sequence<crate::shape_fix::ShapeFixWireSegment>,
        Failure,
    > {
        load_wires::load_wires(self)
    }

    /// Clears and incrementally fills a caller-owned sequence with oriented wire segments.
    /// `let mut segments = Sequence::default(); composer.load_wires_into(&mut segments)?`.
    pub fn load_wires_into(
        &self,
        segments: &mut rustycad_tkernel::ncollection::Sequence<
            crate::shape_fix::ShapeFixWireSegment,
        >,
    ) -> Result<(), Failure> {
        load_wires::load_wires_into(self, segments)
    }

    /// Classifies a wire interval relative to a patch line; `composer.compute_code(&wire, line, 1, 1, 0.25, 0.75, false)?`.
    pub fn compute_code(
        &self,
        wire: &crate::shape_extend::ShapeExtendWireData,
        line: rustycad_tkmath::gp::Line2,
        begin_edge: i32,
        end_edge: i32,
        begin_parameter: f64,
        end_parameter: f64,
        internal: bool,
    ) -> Result<i32, Failure> {
        compute_code::compute_code(
            self,
            wire,
            line,
            begin_edge,
            end_edge,
            begin_parameter,
            end_parameter,
            internal,
        )
    }
}
