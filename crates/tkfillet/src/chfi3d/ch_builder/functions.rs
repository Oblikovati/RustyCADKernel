// Rust translation of the chamfer functions set up by ChFi3d_ChBuilder::SimulSurf,
// PerformFirstSection and PerformSurf (ChFi3d_ChBuilder.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The chamfer functions of a contour: by its method (one distance, two distances, a
//! distance and an angle) and its mode (classic, constant throat, constant throat with
//! penetration).
use super::chamf_spine;
use crate::blend::{BlendFuncInv, BlendFunction};
use crate::blend_func::{
    ChAsym, ChAsymInv, ChamfInv, Chamfer, ConstThroat, ConstThroatInv, ConstThroatWithPenetration,
    ConstThroatWithPenetrationInv, GenChamfInvFunction, GenChamferFunction,
};
use crate::chfi_ds::{ChamfMethod, ChamfMode, ElSpine, SpineHandle, lock};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkmath::gp::Line3;

/// A chamfer function with its linear section.
pub(super) trait ChamferSection: BlendFunction + 'static {
    /// `Section(Param, U1, V1, U2, V2, Pdeb, Pfin, C)`.
    fn section_at(&self, uv: [f64; 4]) -> Result<(f64, f64, Line3), Failure>;
}

macro_rules! gen_chamfer_section {
    ($($class:ty),*) => {$(
        impl ChamferSection for $class {
            fn section_at(&self, uv: [f64; 4]) -> Result<(f64, f64, Line3), Failure> {
                self.section_line(uv)
            }
        }
    )*};
}
gen_chamfer_section!(Chamfer, ConstThroat, ConstThroatWithPenetration);

impl ChamferSection for ChAsym {
    fn section_at(&self, uv: [f64; 4]) -> Result<(f64, f64, Line3), Failure> {
        self.section_line(uv)
    }
}

/// The chamfer of a contour: its method, its mode and its distances (or distance and
/// angle).
#[derive(Clone, Copy)]
pub(super) struct ChamferKind {
    pub method: ChamfMethod,
    pub mode: ChamfMode,
    pub d1: f64,
    pub d2: f64,
    pub angle: f64,
}

/// The chamfer of the spine (Standard_ConstructionError with `message` when it is not a
/// chamfer spine).
pub(super) fn chamfer_kind(spine: &SpineHandle, message: &str) -> Result<ChamferKind, Failure> {
    let g = lock(spine);
    let c = chamf_spine(&g, message)?;
    let method = c.is_chamfer();
    let (d1, d2, angle) = match method {
        ChamfMethod::Sym => {
            let d = c.get_dist()?;
            (d, d, 0.)
        }
        ChamfMethod::TwoDist => {
            let (a, b) = c.dists()?;
            (a, b, 0.)
        }
        ChamfMethod::DistAngle => {
            let (d, a) = c.get_dist_angle()?;
            (d, 0., a)
        }
    };
    Ok(ChamferKind {
        method,
        mode: g.mode(),
        d1,
        d2,
        angle,
    })
}

/// The offset elementary spine of `hguide` (the native search over the lists of the
/// elementary spines and their offsets), None when there is none.
pub(super) fn offset_guide(
    spine: &SpineHandle,
    hguide: &Handle<ElSpine>,
) -> Option<Handle<ElSpine>> {
    let mut g = lock(spine);
    let ll = g.change_el_spines().clone();
    let off = g.change_offset_el_spines().clone();
    let mut found = None;
    for (es, o) in ll.iter().zip(off.iter()) {
        if es == hguide {
            found = Some(o.clone());
        }
    }
    found
}

/// What a walking needs from a chamfer function: the walking function and its inverse,
/// both on the same surfaces and guide.
pub(super) enum ChamferFunctions {
    Classic(Chamfer, ChamfInv),
    Throat(ConstThroat, ConstThroatInv),
    Penetration(ConstThroatWithPenetration, ConstThroatWithPenetrationInv),
    Asym(ChAsym, ChAsymInv),
}

/// The functions of the chamfer `kind` on the surfaces along `hguide` (the offset guide
/// for a constant throat with penetration), set with the choice `choix`.
pub(super) fn chamfer_functions(
    kind: &ChamferKind,
    (s1, s2): (&Handle<dyn Adaptor3dSurface>, &Handle<dyn Adaptor3dSurface>),
    hguide: &Handle<ElSpine>,
    offset: Option<&Handle<ElSpine>>,
    choix: i32,
) -> Result<ChamferFunctions, Failure> {
    let guide: Handle<dyn Adaptor3dCurve> = hguide.clone().cast();
    let classic = kind.mode == ChamfMode::ClassicChamfer;
    Ok(match kind.method {
        ChamfMethod::Sym if classic => {
            let (mut f, mut i) = (Chamfer::new(s1, s2, &guide), ChamfInv::new(s1, s2, &guide));
            f.set_distances(kind.d1, kind.d1, choix);
            i.set_distances(kind.d1, kind.d1, choix);
            ChamferFunctions::Classic(f, i)
        }
        ChamfMethod::Sym => {
            let (mut f, mut i) = (
                ConstThroat::new(s1, s2, &guide),
                ConstThroatInv::new(s1, s2, &guide),
            );
            f.set_distances(kind.d1, kind.d1, choix);
            i.set_distances(kind.d1, kind.d1, choix);
            ChamferFunctions::Throat(f, i)
        }
        ChamfMethod::TwoDist if classic => {
            let (mut f, mut i) = (Chamfer::new(s1, s2, &guide), ChamfInv::new(s1, s2, &guide));
            f.set_distances(kind.d1, kind.d2, choix);
            i.set_distances(kind.d1, kind.d2, choix);
            ChamferFunctions::Classic(f, i)
        }
        ChamfMethod::TwoDist => {
            let og: Handle<dyn Adaptor3dCurve> = offset
                .ok_or_else(|| Failure::new("Construction of offset guide failed!"))?
                .clone()
                .cast();
            let mut f = ConstThroatWithPenetration::new(s1, s2, &og);
            let mut i = ConstThroatWithPenetrationInv::new(s1, s2, &og);
            let throat = kind.d1.max(kind.d2);
            f.set_distances(throat, throat, choix);
            i.set_distances(throat, throat, choix);
            ChamferFunctions::Penetration(f, i)
        }
        ChamfMethod::DistAngle => {
            let mut f = ChAsym::new(s1, s2, &guide)?;
            let mut i = ChAsymInv::new(s1, s2, &guide)?;
            f.set_dist_angle(kind.d1, kind.angle, choix);
            i.set_dist_angle(kind.d1, kind.angle, choix);
            ChamferFunctions::Asym(f, i)
        }
    })
}

/// Runs `run` with the concrete function and inverse of the chamfer.
pub(super) fn with_chamfer<R>(
    fns: ChamferFunctions,
    run: &mut dyn FnMut(&mut dyn ChamferRun) -> Result<R, Failure>,
) -> Result<R, Failure> {
    match fns {
        ChamferFunctions::Classic(f, i) => run(&mut Pair { f: Some(f), i }),
        ChamferFunctions::Throat(f, i) => run(&mut Pair { f: Some(f), i }),
        ChamferFunctions::Penetration(f, i) => run(&mut Pair { f: Some(f), i }),
        ChamferFunctions::Asym(f, i) => run(&mut Pair { f: Some(f), i }),
    }
}

/// A chamfer function with its inverse, the function taken once to be shared.
struct Pair<F, I> {
    f: Option<F>,
    i: I,
}

/// The function and inverse of a chamfer, as the walkings take them.
pub(super) trait ChamferRun {
    /// The walking function and the inverse.
    fn functions(&mut self) -> Result<(&mut dyn BlendFunction, &mut dyn BlendFuncInv), Failure>;
    /// The section at the parameters of a point of the line.
    fn section(&self, uv: [f64; 4]) -> Result<(f64, f64, Line3), Failure>;
    /// The walking function moved into a shared handle for the approximation.
    fn shared(&mut self) -> Result<crate::brep_blend::SharedAppFunction, Failure>;
}

impl<F: ChamferSection, I: BlendFuncInv> ChamferRun for Pair<F, I> {
    fn functions(&mut self) -> Result<(&mut dyn BlendFunction, &mut dyn BlendFuncInv), Failure> {
        let f = self
            .f
            .as_mut()
            .ok_or_else(|| Failure::new("chamfer function already shared"))?;
        Ok((f, &mut self.i))
    }
    fn section(&self, uv: [f64; 4]) -> Result<(f64, f64, Line3), Failure> {
        self.f
            .as_ref()
            .ok_or_else(|| Failure::new("chamfer function already shared"))?
            .section_at(uv)
    }
    fn shared(&mut self) -> Result<crate::brep_blend::SharedAppFunction, Failure> {
        let f = self
            .f
            .take()
            .ok_or_else(|| Failure::new("chamfer function already shared"))?;
        Ok(crate::chfi_ds::shared(f))
    }
}
