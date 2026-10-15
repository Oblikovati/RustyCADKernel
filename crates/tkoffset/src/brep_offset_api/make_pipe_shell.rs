// Rust translation of ModelingAlgorithms/TKOffset/BRepOffsetAPI/BRepOffsetAPI_MakePipeShell.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepOffsetAPI_MakePipeShell`: the builder-API face of `BRepFill_PipeShell`, a shell
//! or solid swept by profiles along a spine.
use rustycad_tkbool::brep_fill::{BRepFillPipeShell, TransitionStyle, TypeOfContact};
use rustycad_tkbrep::topods::{Shape, Vertex, Wire};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkgeomalgo::geom_fill::GeomFillPipeError;
use rustycad_tkgeomalgo::law::LawFunction;
use rustycad_tkmath::gp::{Axis2, Direction3};
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeSweep;
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeShape, BRepBuilderAPIMakeShapeBase, BRepBuilderAPIPipeError,
    BRepBuilderAPITransitionMode,
};

/// The default angular bounds of `BRepFill_PipeShell::SetTransition`.
const TRANSITION_ANGLES: (f64, f64) = (1.0e-2, 6.0);

/// `BRepOffsetAPI_MakePipeShell`: sweeps the added profiles along the spine with the
/// chosen trihedron mode, tolerances and transition; `Build` makes the shell and
/// `MakeSolid` closes it.
///
/// ```
/// use rustycad_tkmath::gp::Point3;
/// use rustycad_tkoffset::brep_offset_api::BRepOffsetAPIMakePipeShell;
/// use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakePolygon, BRepBuilderAPIMakeShape, BRepBuilderAPIMakeWire};
/// use rustycad_tkernel::message::ProgressRange;
/// let spine_edge = BRepBuilderAPIMakeEdge::from_points(Point3::new(0.0, 0.0, 0.0), Point3::new(0.0, 0.0, 10.0))?.edge()?;
/// let spine = BRepBuilderAPIMakeWire::from_edge(&spine_edge)?.wire()?;
/// let square = BRepBuilderAPIMakePolygon::from_four_points(
///     Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0),
///     Point3::new(1.0, 1.0, 0.0), Point3::new(0.0, 1.0, 0.0), true)?;
/// let mut pipe = BRepOffsetAPIMakePipeShell::new(&spine)?;
/// pipe.add(square.wire()?.as_ref(), false, false)?;
/// pipe.build(&ProgressRange::default())?;
/// assert!(pipe.is_done());
/// assert!(pipe.make_solid()?);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct BRepOffsetAPIMakePipeShell {
    base: BRepBuilderAPIMakeShapeBase,
    my_pipe: BRepFillPipeShell,
}

impl BRepBuilderAPIMakeShape for BRepOffsetAPIMakePipeShell {
    fn make_shape_base(&self) -> &BRepBuilderAPIMakeShapeBase {
        &self.base
    }

    fn make_shape_base_mut(&mut self) -> &mut BRepBuilderAPIMakeShapeBase {
        &mut self.base
    }

    /// `Build()`: the sweep, done with its shape when it succeeds.
    fn build(&mut self, _range: &ProgressRange<'_>) -> Result<(), Failure> {
        if self.my_pipe.build()? {
            self.base.my_shape = self.my_pipe.shape().clone();
            self.base.set_done();
        } else {
            self.base.set_not_done();
        }
        Ok(())
    }

    /// `Generated(S)`: the shapes the sweep generated from `S`.
    fn generated(&mut self, shape: &Shape) -> Result<&List<Shape>, Failure> {
        self.base.my_generated = self.my_pipe.generated(shape);
        Ok(&self.base.my_generated)
    }
}

impl BRepPrimAPIMakeSweep for BRepOffsetAPIMakePipeShell {
    /// `FirstShape()`: the start section of the sweep.
    fn first_shape(&mut self) -> Result<Shape, Failure> {
        Ok(self.my_pipe.first_shape().clone())
    }

    /// `LastShape()`: the end section of the sweep.
    fn last_shape(&mut self) -> Result<Shape, Failure> {
        Ok(self.my_pipe.last_shape().clone())
    }
}

impl BRepOffsetAPIMakePipeShell {
    /// `BRepOffsetAPI_MakePipeShell(Spine)`: the corrected Frenet mode, the default
    /// tolerances (1e-4, 1e-4, 1e-2) and the transformed transition, not done;
    /// `BRepOffsetAPIMakePipeShell::new(&spine)?`.
    pub fn new(spine: &Wire) -> Result<Self, Failure> {
        let mut maker = Self {
            base: BRepBuilderAPIMakeShapeBase::new(),
            my_pipe: BRepFillPipeShell::new(spine)?,
        };
        maker.set_tolerance(1.0e-4, 1.0e-4, 1.0e-2);
        maker.set_transition_mode(BRepBuilderAPITransitionMode::Transformed);
        maker.base.set_not_done();
        Ok(maker)
    }

    /// `SetMode(IsFrenet)`: the Frenet (or corrected Frenet) trihedron;
    /// `pipe.set_mode_frenet(false)?`.
    pub fn set_mode_frenet(&mut self, is_frenet: bool) -> Result<(), Failure> {
        self.my_pipe.set_frenet(is_frenet)
    }

    /// `SetDiscreteMode()`: the discrete trihedron; `pipe.set_discrete_mode()?`.
    pub fn set_discrete_mode(&mut self) -> Result<(), Failure> {
        self.my_pipe.set_discrete()
    }

    /// `SetMode(Axe)`: a fixed trihedron; `pipe.set_mode_axe(&axis)?`.
    pub fn set_mode_axe(&mut self, axe: &Axis2) -> Result<(), Failure> {
        self.my_pipe.set_axe(axe)
    }

    /// `SetMode(BiNormal)`: a constant binormal; `pipe.set_mode_bi_normal(dir)?`.
    pub fn set_mode_bi_normal(&mut self, bi_normal: Direction3) -> Result<(), Failure> {
        self.my_pipe.set_bi_normal(bi_normal)
    }

    /// `SetMode(SpineSupport)`: the Darboux trihedron of the support; whether the
    /// spine lies on it; `pipe.set_mode_support(&face)?`.
    pub fn set_mode_support(&mut self, spine_support: &Shape) -> Result<bool, Failure> {
        self.my_pipe.set_spine_support(spine_support)
    }

    /// `SetMode(AuxiliarySpine, CurvilinearEquivalence, KeepContact)`: a guide wire;
    /// `pipe.set_mode_auxiliary(&guide, true, TypeOfContact::NoContact)?`.
    pub fn set_mode_auxiliary(
        &mut self,
        auxiliary_spine: &Wire,
        curvilinear_equivalence: bool,
        keep_contact: TypeOfContact,
    ) -> Result<(), Failure> {
        self.my_pipe
            .set_auxiliary_spine(auxiliary_spine, curvilinear_equivalence, keep_contact)
    }

    /// `Add(Profile, WithContact, WithCorrection)`; `pipe.add(&profile, false, false)?`.
    pub fn add(
        &mut self,
        profile: &Shape,
        with_contact: bool,
        with_correction: bool,
    ) -> Result<(), Failure> {
        self.my_pipe.add(profile, with_contact, with_correction)
    }

    /// `Add(Profile, Location, WithContact, WithCorrection)`: a profile placed at a
    /// vertex of the spine; `pipe.add_at(&profile, &vertex, false, false)?`.
    pub fn add_at(
        &mut self,
        profile: &Shape,
        location: &Vertex,
        with_contact: bool,
        with_correction: bool,
    ) -> Result<(), Failure> {
        self.my_pipe
            .add_at(profile, location, with_contact, with_correction)
    }

    /// `SetLaw(Profile, L, WithContact, WithCorrection)`: the profile scaled by the law;
    /// `pipe.set_law(&profile, &law, false, false)?`.
    pub fn set_law(
        &mut self,
        profile: &Shape,
        law: &Handle<dyn LawFunction>,
        with_contact: bool,
        with_correction: bool,
    ) -> Result<(), Failure> {
        self.my_pipe
            .set_law(profile, law, with_contact, with_correction)
    }

    /// `SetLaw(Profile, L, Location, WithContact, WithCorrection)`;
    /// `pipe.set_law_at(&profile, &law, &vertex, false, false)?`.
    pub fn set_law_at(
        &mut self,
        profile: &Shape,
        law: &Handle<dyn LawFunction>,
        location: &Vertex,
        with_contact: bool,
        with_correction: bool,
    ) -> Result<(), Failure> {
        self.my_pipe
            .set_law_at(profile, law, location, with_contact, with_correction)
    }

    /// `Delete(Profile)`: removes a profile; `pipe.delete(&profile)?`.
    pub fn delete(&mut self, profile: &Shape) -> Result<(), Failure> {
        self.my_pipe.delete_profile(profile)
    }

    /// `IsReady()`: whether a profile was added; `pipe.is_ready()`.
    pub fn is_ready(&self) -> bool {
        self.my_pipe.is_ready()
    }

    /// `GetStatus()`: the sweep status mapped to the builder-API error;
    /// `pipe.get_status()`.
    pub fn get_status(&self) -> BRepBuilderAPIPipeError {
        match self.my_pipe.get_status() {
            GeomFillPipeError::PipeOk => BRepBuilderAPIPipeError::PipeDone,
            GeomFillPipeError::PlaneNotIntersectGuide => {
                BRepBuilderAPIPipeError::PlaneNotIntersectGuide
            }
            GeomFillPipeError::ImpossibleContact => BRepBuilderAPIPipeError::ImpossibleContact,
            GeomFillPipeError::PipeNotOk => BRepBuilderAPIPipeError::PipeNotDone,
        }
    }

    /// `SetTolerance(Tol3d, BoundTol, TolAngular)` (natively defaulted to 1e-4, 1e-4,
    /// 1e-2); `pipe.set_tolerance(1e-4, 1e-4, 1e-2)`.
    pub fn set_tolerance(&mut self, tol3d: f64, bound_tol: f64, tol_angular: f64) {
        self.my_pipe.set_tolerance(tol3d, bound_tol, tol_angular);
    }

    /// `SetMaxDegree(NewMaxDegree)`; `pipe.set_max_degree(11)`.
    pub fn set_max_degree(&mut self, new_max_degree: i32) {
        self.my_pipe.set_max_degree(new_max_degree);
    }

    /// `SetMaxSegments(NewMaxSegments)`; `pipe.set_max_segments(30)`.
    pub fn set_max_segments(&mut self, new_max_segments: i32) {
        self.my_pipe.set_max_segments(new_max_segments);
    }

    /// `SetForceApproxC1(ForceApproxC1)`: try a C1 approximation of a C0 swept
    /// surface; `pipe.set_force_approx_c1(true)`.
    pub fn set_force_approx_c1(&mut self, force_approx_c1: bool) {
        self.my_pipe.set_force_approx_c1(force_approx_c1);
    }

    /// `SetTransitionMode(Mode)` (natively defaulted to Transformed), with the default
    /// angular bounds; `pipe.set_transition_mode(BRepBuilderAPITransitionMode::RightCorner)`.
    pub fn set_transition_mode(&mut self, mode: BRepBuilderAPITransitionMode) {
        let style = match mode {
            BRepBuilderAPITransitionMode::Transformed => TransitionStyle::Modified,
            BRepBuilderAPITransitionMode::RightCorner => TransitionStyle::Right,
            BRepBuilderAPITransitionMode::RoundCorner => TransitionStyle::Round,
        };
        let (angmin, angmax) = TRANSITION_ANGLES;
        self.my_pipe.set_transition(style, angmin, angmax);
    }

    /// `Simulate(NumberOfSection, Result)`: the sections at equal abscissas;
    /// `let sections = pipe.simulate(5)?;`.
    pub fn simulate(&mut self, number_of_section: i32) -> Result<List<Shape>, Failure> {
        self.my_pipe.simulate(number_of_section)
    }

    /// `MakeSolid()`: closes the built shell with its end faces; StdFail_NotDone before
    /// a successful build; `pipe.make_solid()?`.
    pub fn make_solid(&mut self) -> Result<bool, Failure> {
        if !self.is_done() {
            return Err(Failure::with_message(
                "StdFail_NotDone",
                "BRepOffsetAPI_MakePipeShell::MakeSolid",
            ));
        }
        let ok = self.my_pipe.make_solid()?;
        if ok {
            self.base.my_shape = self.my_pipe.shape().clone();
        }
        Ok(ok)
    }

    /// `ErrorOnSurface()`: the approximation error of the swept surfaces;
    /// `pipe.error_on_surface()`.
    pub fn error_on_surface(&self) -> f64 {
        self.my_pipe.error_on_surface()
    }

    /// `SetIsBuildHistory(theIsBuildHistory)`; `pipe.set_is_build_history(false)`.
    pub fn set_is_build_history(&mut self, is_build_history: bool) {
        self.my_pipe.set_is_build_history(is_build_history);
    }

    /// `IsBuildHistory()`; `pipe.is_build_history()`.
    pub fn is_build_history(&self) -> bool {
        self.my_pipe.is_build_history()
    }

    /// `Profiles(theProfiles)`: the added profiles; `let profiles = pipe.profiles();`.
    pub fn profiles(&self) -> List<Shape> {
        self.my_pipe.profiles()
    }

    /// `Spine()`; `pipe.spine()`.
    pub fn spine(&self) -> &Wire {
        self.my_pipe.spine()
    }
}
