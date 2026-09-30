// Rust translation of ModelingData/TKGeomBase/FEmTool/FEmTool_LinearFlexion.hxx, FEmTool_LinearJerk.hxx, FEmTool_LinearTension.hxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `FEmTool_LinearFlexion`, `FEmTool_LinearJerk` and `FEmTool_LinearTension`: the
//! quadratic energies of the second, third and first derivative of an element.
use super::elementary_criterion::ElementaryCriterion;
use super::linear_criterion::{LinearCriterion, LinearKind, NegativeClamp};
use super::reference_matrix::ReferenceCache;
use rustycad_tkernel::impl_transient;
use std::sync::Mutex;

/// The flexion kind: J = integral of |C''|^2, scaled by `2 / coeff^3`.
pub struct Flexion;
/// The jerk kind: J = integral of |C'''|^2, scaled by `2 / coeff^5`.
pub struct Jerk;
/// The tension kind: J = integral of |C'|^2, scaled by `2 / coeff`.
pub struct Tension;

static FLEXION_CACHE: Mutex<ReferenceCache> = Mutex::new(ReferenceCache::new());
static JERK_CACHE: Mutex<ReferenceCache> = Mutex::new(ReferenceCache::new());
static TENSION_CACHE: Mutex<ReferenceCache> = Mutex::new(ReferenceCache::new());

impl LinearKind for Flexion {
    const NAME: &'static str = "FEmTool_LinearFlexion";
    const DER_ORDER: i32 = 2;
    const CLAMP: NegativeClamp = NegativeClamp::AfterSum;
    const ONE_BASED_TABLE: bool = false;
    const HESSIAN_FROM_LOWER: bool = false;
    fn scale(coeff: f64) -> f64 {
        2. / coeff.powf(3.)
    }
    fn cache() -> &'static Mutex<ReferenceCache> {
        &FLEXION_CACHE
    }
}

impl LinearKind for Jerk {
    const NAME: &'static str = "FEmTool_LinearJerk";
    const DER_ORDER: i32 = 3;
    const CLAMP: NegativeClamp = NegativeClamp::EachRow;
    const ONE_BASED_TABLE: bool = false;
    const HESSIAN_FROM_LOWER: bool = true;
    fn scale(coeff: f64) -> f64 {
        2. / coeff.powf(5.)
    }
    fn cache() -> &'static Mutex<ReferenceCache> {
        &JERK_CACHE
    }
}

impl LinearKind for Tension {
    const NAME: &'static str = "FEmTool_LinearTension";
    const DER_ORDER: i32 = 1;
    const CLAMP: NegativeClamp = NegativeClamp::Never;
    const ONE_BASED_TABLE: bool = true;
    const HESSIAN_FROM_LOWER: bool = true;
    fn scale(coeff: f64) -> f64 {
        2. / coeff
    }
    fn cache() -> &'static Mutex<ReferenceCache> {
        &TENSION_CACHE
    }
}

/// `FEmTool_LinearFlexion`: the bending energy of an element.
///
/// ```
/// use rustycad_tkgeombase::fem_tool::{ElementaryCriterion, LinearFlexion};
/// use rustycad_tkernel::ncollection::{Array2, HArray2};
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkmath::geom_abs::Shape;
/// let criterion = LinearFlexion::new(3, Shape::C0)?;
/// let mut coeff = Array2::with_bounds(0, 3, 1, 1).unwrap();
/// coeff.init(0.0);
/// criterion.set_coeff(&Handle::new(HArray2::from_owned(coeff)));
/// criterion.set(0.0, 1.0);
/// assert_eq!(criterion.value()?, 0.0);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub type LinearFlexion = LinearCriterion<Flexion>;
/// `FEmTool_LinearJerk`: the jerk energy of an element; see [`LinearFlexion`].
pub type LinearJerk = LinearCriterion<Jerk>;
/// `FEmTool_LinearTension`: the tension energy of an element; see [`LinearFlexion`].
pub type LinearTension = LinearCriterion<Tension>;

impl_transient!(
    LinearFlexion,
    "FEmTool_LinearFlexion",
    dyn ElementaryCriterion,
    parent,
    interfaces: [dyn ElementaryCriterion]
);
impl_transient!(
    LinearJerk,
    "FEmTool_LinearJerk",
    dyn ElementaryCriterion,
    parent,
    interfaces: [dyn ElementaryCriterion]
);
impl_transient!(
    LinearTension,
    "FEmTool_LinearTension",
    dyn ElementaryCriterion,
    parent,
    interfaces: [dyn ElementaryCriterion]
);
