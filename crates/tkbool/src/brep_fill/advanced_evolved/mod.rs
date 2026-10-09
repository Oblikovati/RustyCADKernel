// Rust translation of ModelingAlgorithms/TKBool/BRepFill/BRepFill_AdvancedEvolved.hxx/.cxx
// (the class, Perform, PerformBoolean, IsLid, PerformSweep and UnifyShape) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepFill_AdvancedEvolved`: the evolved shape of a profile swept along a planar
//! spine with round transitions, closed by planar lids and rebuilt as a solid by the
//! Boolean operations.
mod degenerated;
mod lids;
mod orientation;
mod singularity;
mod solid;
mod tolerance;

use super::{BRepFillPipeShell, TransitionStyle};
use rustycad_tkbo::bop_algo::{BOPAlgoBuilder, BOPAlgoPaveFiller};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{Compound, Face, Shape, Wire};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomPlane;
use rustycad_tkshhealing::shape_upgrade::ShapeUpgradeUnifySameDomain;

/// `aPipeLinearTolerance`: the linear tolerance of the sweep and of the unification.
pub(super) const PIPE_LINEAR_TOLERANCE: f64 = 1.0e-4;
/// `aPipeAngularTolerance`: the angular tolerance of the sweep and of the unification.
pub(super) const PIPE_ANGULAR_TOLERANCE: f64 = 1.0e-2;

/// The status of an advanced evolved computation (the private native enumeration);
/// `AdvancedEvolvedStatus::Ok as u32 == u32::MAX`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum AdvancedEvolvedStatus {
    /// `BRepFill_AdvancedEvolved_Empty`: nothing computed.
    Empty = 0,
    /// `BRepFill_AdvancedEvolved_NotPlanarSpine`.
    NotPlanarSpine = 1,
    /// `BRepFill_AdvancedEvolved_SweepError`.
    SweepError = 2,
    /// `BRepFill_AdvancedEvolved_NoLids`.
    NoLids = 3,
    /// `BRepFill_AdvancedEvolved_NotSolid`.
    NotSolid = 4,
    /// `BRepFill_AdvancedEvolved_NotVolume`.
    NotVolume = 5,
    /// `BRepFill_AdvancedEvolved_OK`.
    Ok = u32::MAX,
}

/// The evolved solid of a profile along a planar spine (`BRepFill_AdvancedEvolved`);
/// `let mut ae = BRepFillAdvancedEvolved::new(); ae.perform(&spine, &profile, 1e-7, true)?;`.
pub struct BRepFillAdvancedEvolved {
    my_error_status: AdvancedEvolvedStatus,
    my_spine: Wire,
    my_profile: Wire,
    my_pipe_shell: Shape,
    // Lids can be split on several faces.
    my_top_bottom: Compound,
    my_result: Shape,
    my_fuzzy_value: f64,
    my_is_parallel: bool,
    my_debug_shapes_path: String,
}

impl Default for BRepFillAdvancedEvolved {
    /// `BRepFill_AdvancedEvolved()`: empty, no fuzzy value, parallel, debug path
    /// `C:\Temp`.
    fn default() -> Self {
        Self {
            my_error_status: AdvancedEvolvedStatus::Empty,
            my_spine: Wire::default(),
            my_profile: Wire::default(),
            my_pipe_shell: Shape::default(),
            my_top_bottom: Compound::default(),
            my_result: Shape::default(),
            my_fuzzy_value: 0.0,
            my_is_parallel: true,
            my_debug_shapes_path: "C:\\Temp".to_string(),
        }
    }
}

impl BRepFillAdvancedEvolved {
    /// `BRepFill_AdvancedEvolved()`; `BRepFillAdvancedEvolved::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `Perform(theSpine, theProfile, theTolerance, theSolidReq)` (natively a solid by
    /// default): the sweep, its lids and the solid; without `theSolidReq` the shell of
    /// the faces of the solid that are not lids; the tolerance is the fuzzy value when
    /// none is set; `ae.perform(&spine, &profile, 1e-7, true)?`.
    pub fn perform(
        &mut self,
        the_spine: &Wire,
        the_profile: &Wire,
        the_tolerance: f64,
        the_solid_req: bool,
    ) -> Result<(), Failure> {
        self.my_error_status = AdvancedEvolvedStatus::Empty;
        if self.my_fuzzy_value < precision::confusion() {
            self.my_fuzzy_value = the_tolerance;
        }
        self.get_spine_and_profile(the_spine, the_profile)?;
        self.my_pipe_shell = Shape::default();
        self.my_top_bottom = Compound::default();
        self.my_result = Shape::default();
        self.perform_sweep()?;
        self.get_lids()?;
        if self.my_error_status != AdvancedEvolvedStatus::NotSolid {
            return Ok(());
        }
        self.my_result = self.my_pipe_shell.clone();
        self.build_solid()?;
        if self.my_error_status != AdvancedEvolvedStatus::Ok || the_solid_req {
            return Ok(());
        }
        self.shell_without_lids()
    }

    /// `IsDone(theErrorCode)`: whether the status is OK, the status written to the code;
    /// `let mut code = 0; ae.is_done(Some(&mut code))`.
    pub fn is_done(&self, the_error_code: Option<&mut u32>) -> bool {
        if let Some(code) = the_error_code {
            *code = self.my_error_status as u32;
        }
        self.my_error_status == AdvancedEvolvedStatus::Ok
    }

    /// `Shape()`: the result; `ae.shape()`.
    pub fn shape(&self) -> &Shape {
        &self.my_result
    }

    /// `SetTemporaryDirectory(thePath)`: the path of the debug dumps (unused without
    /// the debug build); `ae.set_temporary_directory("/tmp/")`.
    pub fn set_temporary_directory(&mut self, the_path: &str) {
        self.my_debug_shapes_path = the_path.to_string();
    }

    /// `SetParallelMode(theVal)`: the parallel mode of the Boolean operations;
    /// `ae.set_parallel_mode(false)`.
    pub fn set_parallel_mode(&mut self, the_val: bool) {
        self.my_is_parallel = the_val;
    }

    /// `PerformBoolean(theArgsList, theResult)` (protected): the general fuse of the
    /// arguments with the fuzzy value; `None` when the filler or the builder fails;
    /// `ae.perform_boolean(&args)?`.
    pub fn perform_boolean(&self, the_args_list: &List<Shape>) -> Result<Option<Shape>, Failure> {
        let mut a_pf = BOPAlgoPaveFiller::new();
        a_pf.set_arguments(the_args_list);
        a_pf.options_mut().set_run_parallel(self.my_is_parallel);
        a_pf.options_mut().set_fuzzy_value(self.my_fuzzy_value);
        a_pf.perform();
        if a_pf.options().has_errors() {
            return Ok(None);
        }
        let mut a_builder = BOPAlgoBuilder::new();
        a_builder.set_arguments(the_args_list);
        a_builder
            .options_mut()
            .set_run_parallel(self.my_is_parallel);
        a_builder.perform_with_filler(&mut a_pf);
        if a_builder.options().has_errors() {
            return Ok(None);
        }
        Ok(Some(a_builder.shape().clone()))
    }

    /// `IsLid(theF, theMapOfLids)` (protected): whether the face lies on the plane
    /// (the same plane object) of one of the lids; `ae.is_lid(&face, &lids)?`.
    pub fn is_lid(
        &self,
        the_f: &Face,
        the_map_of_lids: &IndexedMapOfShape,
    ) -> Result<bool, Failure> {
        if the_map_of_lids.is_empty() {
            return Ok(false);
        }
        let a_pln_f: Handle<GeomPlane> = BRepTool::surface(the_f)?.cast();
        if a_pln_f.is_null() {
            return Ok(false);
        }
        for lid in the_map_of_lids.iter() {
            let a_plane: Handle<GeomPlane> =
                BRepTool::surface(&rustycad_tkbrep::topods::face(lid)?)?.cast();
            if a_plane.as_ptr() == a_pln_f.as_ptr() {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// `PerformSweep()` (protected): the pipe shell of the profile along the spine with
    /// round transitions; status NoLids when built, else SweepError; `ae.perform_sweep()?`.
    pub fn perform_sweep(&mut self) -> Result<(), Failure> {
        if self.my_error_status != AdvancedEvolvedStatus::Empty {
            return Ok(());
        }
        self.my_error_status = AdvancedEvolvedStatus::SweepError;
        let mut a_pipe = BRepFillPipeShell::new(&self.my_spine)?;
        a_pipe.set_tolerance(
            PIPE_LINEAR_TOLERANCE,
            PIPE_LINEAR_TOLERANCE,
            PIPE_ANGULAR_TOLERANCE,
        );
        a_pipe.set_transition(TransitionStyle::Round, 1.0e-2, 6.0);
        a_pipe.add(self.my_profile.as_ref(), false, false)?;
        if a_pipe.build()? {
            self.my_error_status = AdvancedEvolvedStatus::NoLids;
            self.my_pipe_shell = a_pipe.shape().clone();
        }
        Ok(())
    }

    /// `UnifyShape()` (protected): the result with the same-domain faces and edges
    /// merged (safe input, no internal edges, pipe tolerances); `ae.unify_shape()?`.
    pub fn unify_shape(&mut self) -> Result<(), Failure> {
        let a_unifier = ShapeUpgradeUnifySameDomain::empty();
        a_unifier.initialize(&self.my_result, true, true, false)?;
        a_unifier.set_safe_input_mode(true);
        a_unifier.allow_internal_edges(false);
        a_unifier.set_linear_tolerance(PIPE_LINEAR_TOLERANCE);
        a_unifier.set_angular_tolerance(PIPE_ANGULAR_TOLERANCE);
        a_unifier.build()?;
        self.my_result = a_unifier.shape();
        Ok(())
    }

    /// The shell of the faces of the result that are not lids (the result unchanged
    /// without such face).
    fn shell_without_lids(&mut self) -> Result<(), Failure> {
        let mut a_shell = rustycad_tkbrep::topods::Shell::default();
        let mut a_mf_lids = IndexedMapOfShape::default();
        rustycad_tkbrep::topexp::map_shapes_of_type(
            self.my_top_bottom.as_ref(),
            rustycad_tkg3d::top_abs::ShapeEnum::Face,
            &mut a_mf_lids,
        )?;
        let a_bb = rustycad_tkbrep::brep::BRepBuilder::default();
        let mut an_exp = rustycad_tkbrep::topexp::Explorer::new(
            &self.my_result,
            rustycad_tkg3d::top_abs::ShapeEnum::Face,
            rustycad_tkg3d::top_abs::ShapeEnum::Shape,
        )?;
        while an_exp.more() {
            if a_shell.is_null() {
                a_bb.make_shell(&mut a_shell);
            }
            let a_f = rustycad_tkbrep::topods::face(an_exp.current())?;
            if !self.is_lid(&a_f, &a_mf_lids)? {
                a_bb.add(&mut a_shell, &a_f)?;
            }
            an_exp.next()?;
        }
        if !a_shell.is_null() {
            self.my_result = a_shell.into_shape();
        }
        Ok(())
    }
}
