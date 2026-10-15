//! The arguments of the virtual `SimulSurf` and `PerformSurf` members, grouped as the
//! native call sites pass them.
use crate::brep_blend::WalkingDomain;
use crate::chfi_ds::{ElSpine, SpineHandle};
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve2d, BRepAdaptorSurface};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::math::Vector;

/// The walking settings: `MaxStep`, `Fleche`, `TolGuide`, `Inside`, `Appro` and
/// `Forward`.
///
/// ```
/// use rustycad_tkfillet::chfi3d::WalkSettings;
/// let w = WalkSettings { max_step: 1.0, fleche: 1e-3, tol_guide: 1e-4, inside: false, appro: false, forward: true };
/// assert!(w.forward);
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WalkSettings {
    /// `MaxStep`: the largest step of the walking.
    pub max_step: f64,
    /// `Fleche`: the deflection of the sections.
    pub fleche: f64,
    /// `TolGuide`: the tolerance on the guide parameter.
    pub tol_guide: f64,
    /// `Inside`: the start is inside the faces.
    pub inside: bool,
    /// `Appro`: the start solution is approximate.
    pub appro: bool,
    /// `Forward`: the walking goes along the guide.
    pub forward: bool,
}

/// The guide parameters reached and the flags of the interferences at the ends
/// (`First`, `Last`, `Intf`, `Intl`).
///
/// ```
/// use rustycad_tkfillet::chfi3d::SurfOut;
/// let out = SurfOut { first: 0.0, last: 1.0, intf: 0, intl: 0 };
/// assert_eq!(out.last, 1.0);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SurfOut {
    /// `First`: the first guide parameter (in and out).
    pub first: f64,
    /// `Last`: the last guide parameter (in and out).
    pub last: f64,
    /// `Intf`: the first end meets an obstacle.
    pub intf: i32,
    /// `Intl`: the last end meets an obstacle.
    pub intl: i32,
}

/// The arguments of the two-surface `SimulSurf` and `PerformSurf`.
pub struct SurfCall<'a> {
    /// `Guide`: the elementary spine walked along.
    pub guide: &'a Handle<ElSpine>,
    /// `Spine`: the contour.
    pub spine: &'a SpineHandle,
    /// `Choix`: the side of the fillet.
    pub choix: i32,
    /// `S1`, `I1`: the first face and its domain.
    pub s1: &'a Handle<BRepAdaptorSurface>,
    /// The domain of the first face.
    pub i1: &'a WalkingDomain,
    /// `S2`, `I2`: the second face and its domain.
    pub s2: &'a Handle<BRepAdaptorSurface>,
    /// The domain of the second face.
    pub i2: &'a WalkingDomain,
    /// The walking settings (`MaxStep` and `Fleche` unused by `SimulSurf`).
    pub walk: WalkSettings,
    /// `RecOnS1`: the walking may stop on the restrictions of the first face.
    pub rec_on_s1: bool,
    /// `RecOnS2`: the walking may stop on the restrictions of the second face.
    pub rec_on_s2: bool,
    /// `Soldep`: the start solution.
    pub soldep: &'a Vector<'static>,
}

/// The restriction a side of a fillet runs along: `PC`, `Sref`, `PCref`.
pub struct RstRef<'a> {
    /// `PC`: the restriction on the surface.
    pub pc: &'a Handle<BRepAdaptorCurve2d>,
    /// `Sref`: the reference face of the restriction.
    pub sref: &'a Handle<BRepAdaptorSurface>,
    /// `PCref`: the restriction on the reference face.
    pub pcref: &'a Handle<BRepAdaptorCurve2d>,
}

/// One side of a fillet on a restriction: the surface, its domain, its orientation and
/// the restriction, if the side runs along one.
pub struct BlendSide<'a> {
    /// `S`: the surface.
    pub s: &'a Handle<BRepAdaptorSurface>,
    /// `I`: its domain.
    pub i: &'a WalkingDomain,
    /// `Or`: the orientation of the face.
    pub orientation: Orientation,
    /// The restriction of the side, if any.
    pub rst: Option<RstRef<'a>>,
}

/// The arguments of the `SimulSurf` and `PerformSurf` overloads on restrictions; the
/// flags are `RecP`, `RecS`, `RecRst` (one restriction) or `RecP1`, `RecRst1`, `RecP2`,
/// `RecRst2` (two restrictions).
pub struct RstCall<'a> {
    /// `Guide`: the elementary spine walked along.
    pub guide: &'a Handle<ElSpine>,
    /// `Spine`: the contour.
    pub spine: &'a SpineHandle,
    /// `Choix`: the side of the fillet.
    pub choix: i32,
    /// The first side.
    pub side1: BlendSide<'a>,
    /// The second side.
    pub side2: BlendSide<'a>,
    /// The walking settings.
    pub walk: WalkSettings,
    /// The reframing flags.
    pub rec: [bool; 4],
    /// `Soldep`: the start solution.
    pub soldep: &'a Vector<'static>,
}
