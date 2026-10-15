// Rust translation of BRepFeat_Form::GlobalPerform of
// ModelingAlgorithms/TKFeat/BRepFeat/BRepFeat_Form.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GlobalPerform`: the direction and the limiting faces of the feature from the
//! intersections of its curves with the shapes From and Until, then the gluing of the
//! tool on the basis (operation 1) or, when it is not possible, the local Boolean
//! operation keeping the parts of the tool between the limits (operation 2).
mod gluing;
mod selection;

use super::{Form, FormFeature};
use crate::brep_feat::StatusError;
use crate::loc_ope::CSIntersector;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Face, Shape};
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkg3d::top_abs::{self, Orientation, ShapeEnum};

/// The limits of the feature found from the intersections of its curves: the direction
/// (`sens`), and the limiting faces of the shapes From and Until with their orientations.
pub(super) struct Limits {
    pub(super) sens: i32,
    pub(super) ffrom: Face,
    pub(super) funtil: Face,
    pub(super) oriffrom: Orientation,
    pub(super) orifuntil: Orientation,
}

/// The state of one GlobalPerform.
pub(super) struct Run {
    pub(super) the_ope: i32,
    pub(super) change_ope: bool,
    pub(super) from_in_shape: bool,
    pub(super) until_in_shape: bool,
    pub(super) limits: Limits,
}

/// Whether every face of `s` is a face of `sbase`.
fn faces_in_shape(s: &Shape, sbase: &Shape) -> Result<bool, Failure> {
    for f in Explorer::new(s, ShapeEnum::Face, ShapeEnum::Shape)? {
        if !Explorer::new(sbase, ShapeEnum::Face, ShapeEnum::Shape)?.any(|x| x.is_same(&f)) {
            return Ok(false);
        }
    }
    Ok(true)
}

/// The face and orientation of a point of an intersector when it is a franc point,
/// reversed backward.
fn franc_point(
    asi: &CSIntersector,
    jj: i32,
    k: i32,
    sens: i32,
) -> Result<Option<(Orientation, Face)>, Failure> {
    let p = asi.point(jj, k)?;
    let mut oript = p.orientation();
    if oript != Orientation::Forward && oript != Orientation::Reversed {
        return Ok(None);
    }
    if sens == -1 {
        oript = top_abs::reverse(oript);
    }
    Ok(Some((oript, p.face().clone())))
}

impl Limits {
    fn new() -> Self {
        Self {
            sens: 0,
            ffrom: Face::default(),
            funtil: Face::default(),
            oriffrom: Orientation::Internal,
            orifuntil: Orientation::Internal,
        }
    }

    /// The limits from the curve `jj` meeting both shapes; IntervalOverlap when the
    /// curves disagree on the direction.
    fn limits_both(
        &mut self,
        asi1: &CSIntersector,
        asi2: &CSIntersector,
        jj: i32,
        periodic: bool,
    ) -> Result<Result<(), StatusError>, Failure> {
        if asi1.nb_points(jj)? <= 0 {
            return Ok(Ok(()));
        }
        let mf = asi1.point(jj, 1)?.parameter();
        let big_mf = asi1.point(jj, asi1.nb_points(jj)?)?.parameter();
        if asi2.nb_points(jj)? <= 0 {
            return Ok(Ok(()));
        }
        let mu = asi2.point(jj, 1)?.parameter();
        let big_mu = asi2.point(jj, asi2.nb_points(jj)?)?.parameter();
        if periodic {
            return Ok(Ok(()));
        }
        let (kf, ku);
        if mu <= big_mf && mf <= big_mu {
            // overlapping intervals
            self.sens = 1;
            (kf, ku) = (1, asi2.nb_points(jj)?);
        } else if mu > big_mf {
            if self.sens == -1 {
                return Ok(Err(StatusError::IntervalOverlap));
            }
            self.sens = 1;
            (kf, ku) = (1, asi2.nb_points(jj)?);
        } else {
            if self.sens == 1 {
                return Ok(Err(StatusError::IntervalOverlap));
            }
            self.sens = -1;
            (kf, ku) = (asi1.nb_points(jj)?, 1);
        }
        if self.oriffrom == Orientation::Internal
            && let Some((oript, face)) = franc_point(asi1, jj, kf, self.sens)?
        {
            self.oriffrom = top_abs::reverse(oript);
            self.ffrom = face;
        }
        if self.orifuntil == Orientation::Internal
            && let Some((oript, face)) = franc_point(asi2, jj, ku, self.sens)?
        {
            self.orifuntil = oript;
            self.funtil = face;
        }
        Ok(Ok(()))
    }

    /// The limits from the curve `jj` meeting the shape Until only (for base case prism
    /// on mySUntil -> ambivalent direction -> preferable direction = 1).
    fn limits_until(&mut self, asi2: &CSIntersector, jj: i32) -> Result<(), Failure> {
        let n = asi2.nb_points(jj)?;
        if n <= 0 {
            return Ok(());
        }
        if self.sens != 1 {
            let (first, last) = (
                asi2.point(jj, 1)?.parameter(),
                asi2.point(jj, n)?.parameter(),
            );
            self.sens = if first * last <= 0. {
                1
            } else if first < 0. {
                -1
            } else {
                1
            };
        }
        let ku = if self.sens == -1 { 1 } else { n };
        if self.orifuntil == Orientation::Internal
            && self.sens != 0
            && let Some((oript, face)) = franc_point(asi2, jj, ku, self.sens)?
        {
            self.orifuntil = oript;
            self.funtil = face;
        }
        Ok(())
    }

    /// Find sens, FFrom, FUntil: from the intersections of the curves with the shapes
    /// From (`asi1`) and Until (`asi2`).
    fn find(
        scur: &Sequence<Handle<dyn GeomCurve>>,
        asi1: &CSIntersector,
        asi2: &CSIntersector,
    ) -> Result<Result<Self, StatusError>, Failure> {
        let mut limits = Self::new();
        for jj in 1..=scur.len() as i32 {
            if asi1.is_done() && asi2.is_done() {
                let periodic = scur[jj as usize].get().is_some_and(|c| c.is_periodic());
                if let Err(e) = limits.limits_both(asi1, asi2, jj, periodic)? {
                    return Ok(Err(e));
                }
            } else if asi2.is_done() {
                limits.limits_until(asi2, jj)?;
            } else {
                limits.sens = 1;
                break;
            }
        }
        Ok(Ok(limits))
    }
}

/// The intersector of `s` with the curves (not performed for a null shape).
fn intersector(
    s: &Shape,
    scur: &Sequence<Handle<dyn GeomCurve>>,
) -> Result<CSIntersector, Failure> {
    let mut asi = CSIntersector::new();
    if !s.is_null() {
        asi.init(s);
        asi.perform_curves(scur)?;
    }
    Ok(asi)
}

/// Stops the feature with the error `e`.
pub(super) fn fail(form: &mut Form, e: StatusError) {
    form.status_error = e;
    form.base.set_not_done();
}

/// `GlobalPerform()`: performs the feature; its status error tells why it failed;
/// `global_perform(&mut feature)?`.
pub(crate) fn global_perform<F: FormFeature + ?Sized>(feature: &mut F) -> Result<(), Failure> {
    let form = feature.form_mut();
    if !form.all_valid() {
        fail(form, StatusError::NotInitialized);
        return Ok(());
    }
    //--- Initialisation
    let mut the_ope = 2;
    if form.just_feat && !form.fuse {
        fail(form, StatusError::InvOption);
        return Ok(());
    } else if !form.just_feat && !form.glued_f.is_empty() {
        the_ope = 1;
    }
    let from_in_shape = !form.s_from.is_null() && faces_in_shape(&form.s_from, &form.sbase)?;
    let until_in_shape = !form.s_until.is_null() && faces_in_shape(&form.s_until, &form.sbase)?;
    let mut scur = Sequence::default();
    feature.curves(&mut scur)?;
    let form = feature.form_mut();
    //--- 1) by intersection
    // Intersection Tool Shape From, Intersection Tool Shape Until
    let asi1 = intersector(&form.s_from, &scur)?;
    let asi2 = intersector(&form.s_until, &scur)?;
    let limits = match Limits::find(&scur, &asi1, &asi2)? {
        Ok(limits) => limits,
        Err(e) => {
            fail(form, e);
            return Ok(());
        }
    };
    let mut run = Run {
        the_ope,
        change_ope: false,
        from_in_shape,
        until_in_shape,
        limits,
    };
    let mut the_glue = crate::loc_ope::Gluer::new();
    //--- case of gluing
    if run.the_ope == 1 {
        the_glue = gluing::prepare_glue(form, &mut run)?;
    }
    //--- if the gluing is always applicable
    if run.the_ope == 1 {
        gluing::perform_glue(form, &mut run, &mut the_glue)?;
    }
    //--- case without gluing + Tool with proper dimensions
    if run.the_ope == 2 && run.change_ope && form.just_gluer {
        form.just_gluer = false;
        run.the_ope = 0;
    }
    //--- case without gluing
    if run.the_ope == 2 && !selection::perform_without_glue(feature, &mut run)? {
        return Ok(());
    }
    feature.form_mut().status_error = StatusError::Ok;
    Ok(())
}
