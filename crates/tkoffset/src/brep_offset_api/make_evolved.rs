// Rust translation of ModelingAlgorithms/TKOffset/BRepOffsetAPI/BRepOffsetAPI_MakeEvolved.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepOffsetAPI_MakeEvolved`: an evolved shape, a profile swept along a planar spine
//! keeping its relation to the spine plane (`BRepFill_Evolved`), or the volume of that
//! sweep (`BRepFill_AdvancedEvolved`).
use rustycad_tkbool::brep_fill::{BRepFillAdvancedEvolved, BRepFillEvolved, axe};
use rustycad_tkbrep::topods::{self, Shape, ShapeIterator, Wire};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::geom_abs::JoinType;
use rustycad_tkmath::gp::{Axis3, Direction3, Point3};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeShape, BRepBuilderAPIMakeShapeBase};
use std::sync::OnceLock;

/// The options of `BRepOffsetAPI_MakeEvolved(theSpine, theProfile, ...)`, with the
/// native defaults (arc join, the profile in the reference frame, a shell, the profile
/// not required on the spine, 1e-7, the classic algorithm, sequential).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EvolvedOptions {
    /// `theJoinType`.
    pub join_type: JoinType,
    /// `theIsAxeProf`: the profile is given in the global frame (else in the frame
    /// `BRepFill::Axe` finds on the spine).
    pub is_axe_prof: bool,
    /// `theIsSolid`.
    pub is_solid: bool,
    /// `theIsProfOnSpine`: no result when the profile does not touch the spine.
    pub is_prof_on_spine: bool,
    /// `theTol`.
    pub tol: f64,
    /// `theIsVolume`: the `BRepFill_AdvancedEvolved` algorithm.
    pub is_volume: bool,
    /// `theRunInParallel`.
    pub run_in_parallel: bool,
}

impl Default for EvolvedOptions {
    fn default() -> Self {
        Self {
            join_type: JoinType::Arc,
            is_axe_prof: true,
            is_solid: false,
            is_prof_on_spine: false,
            tol: 0.0000001,
            is_volume: false,
            run_in_parallel: false,
        }
    }
}

/// `BRepOffsetAPI_MakeEvolved`: builds the evolved shape at construction.
///
/// ```
/// use rustycad_tkoffset::brep_offset_api::BRepOffsetAPIMakeEvolved;
/// use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
/// let maker = BRepOffsetAPIMakeEvolved::new();
/// assert!(!maker.is_done());
/// ```
pub struct BRepOffsetAPIMakeEvolved {
    base: BRepBuilderAPIMakeShapeBase,
    my_evolved: BRepFillEvolved,
    my_volume: BRepFillAdvancedEvolved,
    my_is_volume: bool,
}

impl Default for BRepOffsetAPIMakeEvolved {
    fn default() -> Self {
        Self::new()
    }
}

impl BRepBuilderAPIMakeShape for BRepOffsetAPIMakeEvolved {
    fn make_shape_base(&self) -> &BRepBuilderAPIMakeShapeBase {
        &self.base
    }

    fn make_shape_base_mut(&mut self) -> &mut BRepBuilderAPIMakeShapeBase {
        &mut self.base
    }

    /// `Build()`: the shape of whichever algorithm is done; always done.
    fn build(&mut self, _range: &ProgressRange<'_>) -> Result<(), Failure> {
        if self.my_evolved.is_done() {
            self.base.my_shape = self.my_evolved.shape().clone();
        } else if self.my_volume.is_done(None) {
            self.base.my_shape = self.my_volume.shape().clone();
        }
        self.base.set_done();
        Ok(())
    }
}

impl BRepOffsetAPIMakeEvolved {
    /// `BRepOffsetAPI_MakeEvolved()`: nothing built (the volume flag, uninitialized
    /// natively, is false); `BRepOffsetAPIMakeEvolved::new()`.
    pub fn new() -> Self {
        Self {
            base: BRepBuilderAPIMakeShapeBase::new(),
            my_evolved: BRepFillEvolved::new(),
            my_volume: BRepFillAdvancedEvolved::new(),
            my_is_volume: false,
        }
    }

    /// `BRepOffsetAPI_MakeEvolved(theSpine, theProfile, ...)`: Standard_TypeMismatch
    /// for a spine that is neither a wire nor a face; the volume of the sweep, or the
    /// evolved shape (the profile placed by `BRepFill::Axe` unless `is_axe_prof`, and
    /// nothing when it must touch the spine and does not); not done when the volume
    /// algorithm fails or the profile misses the spine;
    /// `BRepOffsetAPIMakeEvolved::with_spine(&spine, &profile, EvolvedOptions::default())?`.
    pub fn with_spine(
        spine: &Shape,
        profile: &Wire,
        options: EvolvedOptions,
    ) -> Result<Self, Failure> {
        let spine_type = spine.shape_type()?;
        if spine_type != ShapeEnum::Wire && spine_type != ShapeEnum::Face {
            return Err(Failure::with_message(
                "Standard_TypeMismatch",
                "BRepOffsetAPI_MakeEvolved: face or wire is expected as a spine",
            ));
        }
        let mut maker = Self::new();
        maker.my_is_volume = options.is_volume;
        let built = if options.is_volume {
            maker.perform_volume(spine, profile, options)?
        } else {
            maker.perform_evolved(spine, profile, options)?
        };
        if built {
            maker.build(&ProgressRange::default())?;
        }
        Ok(maker)
    }

    /// The volume branch; whether the algorithm is done.
    fn perform_volume(
        &mut self,
        spine: &Shape,
        profile: &Wire,
        options: EvolvedOptions,
    ) -> Result<bool, Failure> {
        self.my_volume.set_parallel_mode(options.run_in_parallel);
        let a_spine = if spine.shape_type()? == ShapeEnum::Wire {
            topods::wire(spine)?
        } else {
            topods::wire(ShapeIterator::new(spine, true, true).value()?)?
        };
        self.my_volume
            .perform(&a_spine, profile, options.tol, options.is_solid)?;
        Ok(self.my_volume.is_done(None))
    }

    /// The evolved branch; false when the profile must touch the spine and does not.
    fn perform_evolved(
        &mut self,
        spine: &Shape,
        profile: &Wire,
        options: EvolvedOptions,
    ) -> Result<bool, Failure> {
        let mut axis = Axis3::new(
            Point3::new(0., 0., 0.),
            Direction3::new(0., 0., 1.)?,
            Direction3::new(1., 0., 0.)?,
        )?;
        if !options.is_axe_prof {
            let (frame, pos) = axe(spine, profile, options.tol.max(precision::confusion()))?;
            axis = frame;
            if options.is_prof_on_spine && !pos {
                return Ok(false);
            }
        }
        let (join, solid) = (options.join_type, options.is_solid);
        if spine.shape_type()? == ShapeEnum::Wire {
            self.my_evolved
                .perform_wire(&topods::wire(spine)?, profile, &axis, join, solid)?;
        } else {
            self.my_evolved
                .perform_face(&topods::face(spine)?, profile, &axis, join, solid)?;
        }
        Ok(true)
    }

    /// `Evolved()`: the classic algorithm; Standard_TypeMismatch in volume mode;
    /// `maker.evolved()?`.
    pub fn evolved(&self) -> Result<&BRepFillEvolved, Failure> {
        if self.my_is_volume {
            return Err(Failure::with_message(
                "Standard_TypeMismatch",
                "BRepOffsetAPI_MakeEvolved: myEvolved is accessed while in volume mode",
            ));
        }
        Ok(&self.my_evolved)
    }

    /// `GeneratedShapes(SpineShape, ProfShape)`: the shapes generated by a pair of a
    /// spine and a profile element (none unless the classic algorithm is done);
    /// `maker.generated_shapes(&spine_edge, &profile_edge)`.
    pub fn generated_shapes(&self, spine_shape: &Shape, prof_shape: &Shape) -> &List<Shape> {
        static EMPTY: OnceLock<List<Shape>> = OnceLock::new();
        if !self.my_evolved.is_done() {
            return EMPTY.get_or_init(List::default);
        }
        self.my_evolved.generated_shapes(spine_shape, prof_shape)
    }

    /// `Top()`: the top of the classic evolved shape; `maker.top()`.
    pub fn top(&self) -> &Shape {
        self.my_evolved.top()
    }

    /// `Bottom()`: the bottom of the classic evolved shape; `maker.bottom()`.
    pub fn bottom(&self) -> &Shape {
        self.my_evolved.bottom()
    }
}
