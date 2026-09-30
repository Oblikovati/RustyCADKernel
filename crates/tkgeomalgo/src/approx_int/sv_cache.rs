//! The two last results that both SvSurfaces templates keep (`MyPnt`... and
//! `MyPntbis`...), and the lookup that both `Compute` bodies open with.
use super::sv_surfaces::{ApproxIntSvComputed, ApproxIntSvSurfaces};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Point2, Point3, Vector2, Vector3};

/// One stored result: `MyParOnS1`, `MyParOnS2`, `MyPnt`, `MyTguv1`, `MyTguv2`, `MyTg`
/// and `MyIsTangent` (or their `bis` copies).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct SvResult {
    pub(crate) par_on_s1: Point2,
    pub(crate) par_on_s2: Point2,
    pub(crate) pnt: Point3,
    pub(crate) tguv1: Vector2,
    pub(crate) tguv2: Vector2,
    pub(crate) tg: Vector3,
    pub(crate) is_tangent: bool,
}

impl SvResult {
    // Whether the stored parameters are exactly `uv` (the native `==` tests).
    fn matches(&self, uv: [f64; 4]) -> bool {
        self.par_on_s1.x() == uv[0]
            && self.par_on_s1.y() == uv[1]
            && self.par_on_s2.x() == uv[2]
            && self.par_on_s2.y() == uv[3]
    }
}

/// The current and the previous result with their `MyHasBeenComputed` flags.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct SvCache {
    pub(crate) cur: SvResult,
    pub(crate) computed: bool,
    pub(crate) bis: SvResult,
    pub(crate) computed_bis: bool,
}

impl SvCache {
    /// The opening of `Compute`: the flag of the current result at `uv`; else the
    /// current result is saved as the previous one (when none is); then the previous
    /// result at `uv` is swapped with the current one. None when neither matches.
    pub(crate) fn lookup(&mut self, uv: [f64; 4]) -> Option<bool> {
        if self.computed {
            if self.cur.matches(uv) {
                return Some(self.cur.is_tangent);
            }
            if !self.computed_bis {
                self.bis = self.cur;
                self.computed_bis = self.computed;
            }
        }
        if self.computed_bis && self.bis.matches(uv) {
            std::mem::swap(&mut self.cur, &mut self.bis);
            return Some(self.cur.is_tangent);
        }
        None
    }

    /// `MyIsTangent = false; MyHasBeenComputed = MyHasBeenComputedbis = false;
    /// return false;`, the failure exit of `Compute`.
    pub(crate) fn fail(&mut self) -> bool {
        self.cur.is_tangent = false;
        self.computed = false;
        self.computed_bis = false;
        false
    }
}

/// The first step of the wrappers `Pnt`, `Tangency`, `TangencyOnSurf1` and
/// `TangencyOnSurf2`, identical in both templates: `Compute` on copies of the
/// parameters (its outputs are discarded; the wrappers then read the stored member).
pub(crate) fn compute_on_copies<S: ApproxIntSvSurfaces + ?Sized>(
    sv: &mut S,
    uv: [f64; 4],
) -> Result<bool, Failure> {
    let mut t: [f64; 4] = uv;
    let mut out = ApproxIntSvComputed::default();
    sv.compute(&mut t, &mut out)
}

/// The members `Pnt`, `Tangency`, `TangencyOnSurf1` and `TangencyOnSurf2` of an
/// SvSurfaces whose field `cache` is an [`SvCache`]: `Compute` on copies of the
/// parameters, then the stored point or tangent.
macro_rules! sv_stored_members {
    () => {
        fn pnt(&mut self, uv: [f64; 4]) -> Result<Point3, Failure> {
            super::sv_cache::compute_on_copies(self, uv)?;
            Ok(self.cache.cur.pnt)
        }
        fn tangency(&mut self, uv: [f64; 4]) -> Result<(bool, Vector3), Failure> {
            let flag: bool = super::sv_cache::compute_on_copies(self, uv)?;
            Ok((flag, self.cache.cur.tg))
        }
        fn tangency_on_surf1(&mut self, uv: [f64; 4]) -> Result<(bool, Vector2), Failure> {
            let flag: bool = super::sv_cache::compute_on_copies(self, uv)?;
            Ok((flag, self.cache.cur.tguv1))
        }
        fn tangency_on_surf2(&mut self, uv: [f64; 4]) -> Result<(bool, Vector2), Failure> {
            let flag: bool = super::sv_cache::compute_on_copies(self, uv)?;
            Ok((flag, self.cache.cur.tguv2))
        }
    };
}
pub(crate) use sv_stored_members;
