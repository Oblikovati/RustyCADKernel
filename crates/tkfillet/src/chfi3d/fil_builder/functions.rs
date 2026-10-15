// Rust translation of the fillet functions set up by the restriction overloads of
// ChFi3d_FilBuilder::SimulSurf and ChFi3d_FilBuilder::PerformSurf (ChFi3d_FilBuilder.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The functions of the fillets on restrictions (obstacles): the walking function and
//! its inverses, constant radius or law of evolution, set up as natively.
use crate::blend::{
    BlendCurvPointFuncInv, BlendFuncInv, BlendRstRstFunction, BlendSurfCurvFuncInv,
    BlendSurfPointFuncInv, BlendSurfRstFunction,
};
use crate::blend_func::{ConstRadInv, EvolRadInv, SectionShape};
use crate::brep_blend::{
    BRepBlendCurvPointRadInv, BRepBlendRstRstConstRad, BRepBlendRstRstEvolRad,
    BRepBlendSurfCurvConstRadInv, BRepBlendSurfCurvEvolRadInv, BRepBlendSurfPointConstRadInv,
    BRepBlendSurfPointEvolRadInv, BRepBlendSurfRstConstRad, BRepBlendSurfRstEvolRad,
};
use crate::chfi_ds::{ElSpine, Shared, shared};
use crate::chfi3d::builder::RstRef;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dCurveOnSurface, Adaptor3dSurface};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeomalgo::law::LawFunction;
use rustycad_tkmath::gp::Circle3;

/// A fillet function between a surface and a restriction with its circular section.
pub(super) trait SurfRstFillet: BlendSurfRstFunction + 'static {
    /// `Section(Param, U, V, W, Pdeb, Pfin, C)`.
    fn section_at(
        &mut self,
        param: f64,
        uvw: (f64, f64, f64),
    ) -> Result<(f64, f64, Circle3), Failure>;
}

/// A fillet function between two restrictions with its circular section.
pub(super) trait RstRstFillet: BlendRstRstFunction + 'static {
    /// `Section(Param, U, V, Pdeb, Pfin, C)`.
    fn section_at(&mut self, param: f64, uv: (f64, f64)) -> Result<(f64, f64, Circle3), Failure>;
}

macro_rules! surf_rst_fillet {
    ($class:ty) => {
        impl SurfRstFillet for $class {
            fn section_at(
                &mut self,
                param: f64,
                uvw: (f64, f64, f64),
            ) -> Result<(f64, f64, Circle3), Failure> {
                self.section_circle(param, uvw)
            }
        }
    };
}
surf_rst_fillet!(BRepBlendSurfRstConstRad);
surf_rst_fillet!(BRepBlendSurfRstEvolRad);

macro_rules! rst_rst_fillet {
    ($class:ty) => {
        impl RstRstFillet for $class {
            fn section_at(
                &mut self,
                param: f64,
                uv: (f64, f64),
            ) -> Result<(f64, f64, Circle3), Failure> {
                self.section_circle(param, uv)
            }
        }
    };
}
rst_rst_fillet!(BRepBlendRstRstConstRad);
rst_rst_fillet!(BRepBlendRstRstEvolRad);

/// The functions of a walking between a surface and a restriction.
pub(super) struct SurfRstParts<F> {
    pub func: Shared<F>,
    pub finv: Box<dyn BlendFuncInv>,
    pub finvp: Box<dyn BlendSurfPointFuncInv>,
    pub finvc: Box<dyn BlendSurfCurvFuncInv>,
}

/// The functions of a walking between two restrictions.
pub(super) struct RstRstParts<F> {
    pub func: Shared<F>,
    pub finv: [Box<dyn BlendSurfCurvFuncInv>; 2],
    pub finvp: [Box<dyn BlendCurvPointFuncInv>; 2],
}

/// The side of a surface-restriction fillet on the restriction: the surface carrying the
/// restriction and the restriction with its reference.
pub(super) struct RstSideRefs<'a> {
    pub surf: &'a Handle<BRepAdaptorSurface>,
    pub rst: &'a RstRef<'a>,
}

/// The common inputs of the functions: the guide, the choice and the section shape.
pub(super) struct FunctionSettings<'a> {
    pub guide: &'a Handle<ElSpine>,
    pub choix: i32,
    pub shape: SectionShape,
}

fn general_surface(h: &Handle<BRepAdaptorSurface>) -> Handle<dyn Adaptor3dSurface> {
    h.clone().cast()
}

fn general_rst<T: ?Sized + rustycad_tkernel::standard::RuntimeTyped>(
    h: &Handle<T>,
) -> Handle<dyn Adaptor2dCurve2d> {
    h.clone().cast()
}

/// `Adaptor3d_CurveOnSurface HC; HC->Load(PC, HS)`.
fn curve_on_surface(
    rst: &RstRef<'_>,
    surf: &Handle<BRepAdaptorSurface>,
) -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
    Ok(Handle::new(Adaptor3dCurveOnSurface::with_curve(
        &general_rst(rst.pc),
        &general_surface(surf),
    )?)
    .cast())
}

/// `petitchoix`: 1 (3 when the surface is reversed), plus one for an even choice.
fn petit_choix(or_surf: Orientation, choix: i32) -> i32 {
    let mut petitchoix = if or_surf == Orientation::Reversed {
        3
    } else {
        1
    };
    if choix % 2 == 0 {
        petitchoix += 1;
    }
    petitchoix
}

/// The constant-radius functions of a fillet between the surface `surf` (of orientation
/// `or_surf`) and the restriction `side`: `BRepBlend_SurfRstConstRad` and its inverses.
pub(super) fn surf_rst_const(
    surf: &Handle<BRepAdaptorSurface>,
    or_surf: Orientation,
    side: &RstSideRefs<'_>,
    rad: f64,
    settings: &FunctionSettings<'_>,
) -> Result<SurfRstParts<BRepBlendSurfRstConstRad>, Failure> {
    let (s, srst, guide) = (
        general_surface(surf),
        general_surface(side.surf),
        settings.guide.clone().cast(),
    );
    let mut func = BRepBlendSurfRstConstRad::new((&s, &srst, &general_rst(side.rst.pc)), &guide)?;
    func.set_refs(
        &general_surface(side.rst.sref),
        &general_rst(side.rst.pcref),
    );
    let hc = curve_on_surface(side.rst, side.surf)?;
    let mut finvc = BRepBlendSurfCurvConstRadInv::new(&s, &hc, &guide);
    let mut finvp = BRepBlendSurfPointConstRadInv::new(&s, &guide);
    let mut finv = ConstRadInv::new(&s, &general_surface(side.rst.sref), &guide);
    finv.set_curve(false, &general_rst(side.rst.pcref))?;
    let petitchoix = petit_choix(or_surf, settings.choix);
    finv.set_radius(rad, settings.choix);
    finvc.set_radius(rad, petitchoix);
    finvp.set_radius(rad, petitchoix);
    func.set_radius(rad, petitchoix);
    func.set_shape(settings.shape);
    Ok(SurfRstParts {
        func: shared(func),
        finv: Box::new(finv),
        finvp: Box::new(finvp),
        finvc: Box::new(finvc),
    })
}

/// The evolutive-radius functions of a fillet between a surface and a restriction:
/// `BRepBlend_SurfRstEvolRad` and its inverses on the law `law`.
pub(super) fn surf_rst_evol(
    surf: &Handle<BRepAdaptorSurface>,
    or_surf: Orientation,
    side: &RstSideRefs<'_>,
    law: &Handle<dyn LawFunction>,
    settings: &FunctionSettings<'_>,
) -> Result<SurfRstParts<BRepBlendSurfRstEvolRad>, Failure> {
    let (s, srst, guide) = (
        general_surface(surf),
        general_surface(side.surf),
        settings.guide.clone().cast(),
    );
    let mut func =
        BRepBlendSurfRstEvolRad::new((&s, &srst, &general_rst(side.rst.pc)), &guide, law)?;
    func.set_refs(
        &general_surface(side.rst.sref),
        &general_rst(side.rst.pcref),
    );
    let hc = curve_on_surface(side.rst, side.surf)?;
    let mut finvc = BRepBlendSurfCurvEvolRadInv::new(&s, &hc, &guide, law);
    let mut finvp = BRepBlendSurfPointEvolRadInv::new(&s, &guide, law);
    let mut finv = EvolRadInv::new(&s, &general_surface(side.rst.sref), &guide, law);
    finv.set_curve(false, &general_rst(side.rst.pcref))?;
    let petitchoix = petit_choix(or_surf, settings.choix);
    finv.set_choix(settings.choix);
    finvc.set_choix(petitchoix);
    finvp.set_choix(petitchoix);
    func.set_choix(petitchoix);
    func.set_shape(settings.shape);
    Ok(SurfRstParts {
        func: shared(func),
        finv: Box::new(finv),
        finvp: Box::new(finvp),
        finvc: Box::new(finvc),
    })
}

/// The two sides of a fillet between restrictions, with the orientations of their
/// reference faces.
pub(super) struct RstRstSides<'a> {
    pub side1: RstSideRefs<'a>,
    pub or1: Orientation,
    pub side2: RstSideRefs<'a>,
    pub or2: Orientation,
}

/// The choices of the inverse functions on the reference faces: 1 (3 when reversed) on
/// the first, 2 (3 when reversed) on the second.
fn rst_rst_choices(sides: &RstRstSides<'_>) -> (i32, i32) {
    let ch1 = if sides.or1 == Orientation::Reversed {
        3
    } else {
        1
    };
    let ch2 = if sides.or2 == Orientation::Reversed {
        3
    } else {
        2
    };
    (ch1, ch2)
}

type RstRstAdaptors = (
    [Handle<dyn Adaptor3dSurface>; 2],
    [Handle<dyn Adaptor2dCurve2d>; 2],
    [Handle<dyn Adaptor3dCurve>; 2],
    Handle<dyn Adaptor3dCurve>,
);

/// The surfaces, the restrictions, the curves on surface `HC1`, `HC2` and the guide.
fn rst_rst_adaptors(
    sides: &RstRstSides<'_>,
    guide: &Handle<ElSpine>,
) -> Result<RstRstAdaptors, Failure> {
    Ok((
        [
            general_surface(sides.side1.surf),
            general_surface(sides.side2.surf),
        ],
        [
            general_rst(sides.side1.rst.pc),
            general_rst(sides.side2.rst.pc),
        ],
        [
            curve_on_surface(sides.side1.rst, sides.side1.surf)?,
            curve_on_surface(sides.side2.rst, sides.side2.surf)?,
        ],
        guide.clone().cast(),
    ))
}

/// The references `(Sref1, PCref1, Sref2, PCref2)` of the sides.
fn rst_rst_refs(
    sides: &RstRstSides<'_>,
) -> (
    [Handle<dyn Adaptor3dSurface>; 2],
    [Handle<dyn Adaptor2dCurve2d>; 2],
) {
    (
        [
            general_surface(sides.side1.rst.sref),
            general_surface(sides.side2.rst.sref),
        ],
        [
            general_rst(sides.side1.rst.pcref),
            general_rst(sides.side2.rst.pcref),
        ],
    )
}

/// The constant-radius functions of a fillet between two restrictions.
pub(super) fn rst_rst_const(
    sides: &RstRstSides<'_>,
    rad: f64,
    settings: &FunctionSettings<'_>,
) -> Result<RstRstParts<BRepBlendRstRstConstRad>, Failure> {
    let (s, pc, hc, guide) = rst_rst_adaptors(sides, settings.guide)?;
    let (sref, pcref) = rst_rst_refs(sides);
    let mut func = BRepBlendRstRstConstRad::new((&s[0], &pc[0], &s[1], &pc[1]), &guide)?;
    func.set_refs((&sref[0], &pcref[0], &sref[1], &pcref[1]));
    let mut finv1 = BRepBlendSurfCurvConstRadInv::new(&sref[0], &hc[1], &guide);
    let mut finvp1 = BRepBlendCurvPointRadInv::new(&guide, &hc[1]);
    let mut finv2 = BRepBlendSurfCurvConstRadInv::new(&sref[1], &hc[0], &guide);
    let mut finvp2 = BRepBlendCurvPointRadInv::new(&guide, &hc[0]);
    finv1.set_rst(&pcref[0]);
    finv2.set_rst(&pcref[1]);
    let (ch1, ch2) = rst_rst_choices(sides);
    finv1.set_radius(rad, ch1);
    finvp1.set_choix(settings.choix);
    finv2.set_radius(rad, ch2);
    finvp2.set_choix(settings.choix);
    func.set_radius(rad, settings.choix);
    func.set_shape(settings.shape);
    Ok(RstRstParts {
        func: shared(func),
        finv: [Box::new(finv1), Box::new(finv2)],
        finvp: [Box::new(finvp1), Box::new(finvp2)],
    })
}

/// The evolutive-radius functions of a fillet between two restrictions.
pub(super) fn rst_rst_evol(
    sides: &RstRstSides<'_>,
    law: &Handle<dyn LawFunction>,
    settings: &FunctionSettings<'_>,
) -> Result<RstRstParts<BRepBlendRstRstEvolRad>, Failure> {
    let (s, pc, hc, guide) = rst_rst_adaptors(sides, settings.guide)?;
    let (sref, pcref) = rst_rst_refs(sides);
    let mut func = BRepBlendRstRstEvolRad::new((&s[0], &pc[0], &s[1], &pc[1]), &guide, law)?;
    func.set_refs((&sref[0], &pcref[0], &sref[1], &pcref[1]));
    let mut finv1 = BRepBlendSurfCurvEvolRadInv::new(&sref[0], &hc[1], &guide, law);
    let mut finvp1 = BRepBlendCurvPointRadInv::new(&guide, &hc[1]);
    let mut finv2 = BRepBlendSurfCurvEvolRadInv::new(&sref[1], &hc[0], &guide, law);
    let mut finvp2 = BRepBlendCurvPointRadInv::new(&guide, &hc[0]);
    finv1.set_rst(&pcref[0]);
    finv2.set_rst(&pcref[1]);
    let (ch1, ch2) = rst_rst_choices(sides);
    finv1.set_choix(ch1);
    finvp1.set_choix(settings.choix);
    finv2.set_choix(ch2);
    finvp2.set_choix(settings.choix);
    func.set_choix(settings.choix);
    func.set_shape(settings.shape);
    Ok(RstRstParts {
        func: shared(func),
        finv: [Box::new(finv1), Box::new(finv2)],
        finvp: [Box::new(finvp1), Box::new(finvp2)],
    })
}
