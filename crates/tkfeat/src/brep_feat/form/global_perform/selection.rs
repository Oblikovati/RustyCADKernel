// Rust translation of the local Boolean operation part (operation 2) of
// BRepFeat_Form::GlobalPerform of ModelingAlgorithms/TKFeat/BRepFeat/BRepFeat_Form.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The local Boolean operation: the tool cut by the solids of the limiting shapes that
//! are not in the basis, BRepFeat_Builder on the basis and the tool, and the parts of the
//! tool kept between the parameters of the limits along the barycentre curve.
use super::gluing::{cut_descendants, cut_solids};
use super::{Form, FormFeature, Run, fail};
use crate::brep_feat::form::descendants;
use crate::brep_feat::{BRepFeatBuilder, PerfSelection, StatusError, parametric_min_max, tool};
use rustycad_tkbo::brep_algo_api::BRepAlgoAPICut;
use rustycad_tkbool::brep_algo;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Compound, Shape};
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::elclib;

/// The parameter window of the kept parts along the barycentre curve: the intersection
/// parameters (`prmin`, `prmax`) and the projection parameters (`pbmin`, `pbmax`) of the
/// limits, and whether the curve meets them (`flag1`).
struct Window {
    prmin: f64,
    prmax: f64,
    pbmin: f64,
    pbmax: f64,
    flag1: bool,
}

/// The compound of the solids of the limiting shapes that are not in the basis.
fn limit_compound(form: &Form, run: &Run) -> Result<Shape, Failure> {
    let b = BRepBuilder::default();
    let mut comp = Compound::default();
    b.make_compound(&mut comp);
    let mut comp = comp.into_shape();
    let limits = &run.limits;
    let mut add = |s: &Shape, face, ori| -> Result<(), Failure> {
        let solid = tool(s, face, ori)?;
        if !solid.is_null() {
            b.add(&mut comp, &solid)?;
        }
        Ok(())
    };
    if !form.s_from.is_null() && !run.from_in_shape {
        add(&form.s_from, &limits.ffrom, limits.oriffrom)?;
    }
    if !form.s_until.is_null()
        && !run.until_in_shape
        && (form.s_from.is_null() || !form.s_from.is_same(&form.s_until))
    {
        add(&form.s_until, &limits.funtil, limits.orifuntil)?;
    }
    Ok(comp)
}

/// Update type of selection: a selection up to a limiting shape that is not in the basis
/// becomes no selection.
fn update_selection(form: &mut Form, run: &Run) {
    let sel = form.perf_selection;
    if (sel == PerfSelection::SelectionU && !run.until_in_shape)
        || (sel == PerfSelection::SelectionFU && !run.from_in_shape && !run.until_in_shape)
        || (sel == PerfSelection::SelectionShU && !run.until_in_shape)
    {
        form.perf_selection = PerfSelection::NoSelection;
    }
}

/// The tool cut by the solids of `comp`, the limiting faces mapped to what the cut made
/// of them; None (with the error set) when the cut is empty or invalid.
fn cut_tool(form: &mut Form, run: &Run, comp: &Shape) -> Result<Option<Shape>, Failure> {
    let tr_p = BRepAlgoAPICut::from_shapes(&form.g_shape, comp)?;
    // the result is necessarily a compound.
    if !Explorer::new(tr_p.shape(), ShapeEnum::Solid, ShapeEnum::Shape)?.more() {
        fail(form, StatusError::EmptyCutResult);
        return Ok(None);
    }
    // Only solids are preserved
    let the_g_shape = cut_solids(&tr_p)?;
    if !brep_algo::is_valid(&the_g_shape)? {
        fail(form, StatusError::InvShape);
        return Ok(None);
    }
    if !form.s_from.is_null() && !run.from_in_shape {
        for fac in Explorer::new(&form.s_from.clone(), ShapeEnum::Face, ShapeEnum::Shape)? {
            let desc = cut_descendants(&tr_p, &fac)?;
            form.map.bind(fac, desc);
        }
    }
    if !form.s_until.is_null() && !run.until_in_shape {
        for fac in Explorer::new(&form.s_until.clone(), ShapeEnum::Face, ShapeEnum::Shape)? {
            // Native quirk kept: the face itself is added when the whole map is empty,
            // which it never is here.
            let mut desc = List::default();
            if !tr_p.is_deleted(&fac)? {
                for m in tr_p.modified(&fac)? {
                    desc.append(m);
                }
            }
            form.map.bind(fac, desc);
        }
    }
    form.update_descendants_bop(&tr_p, &the_g_shape, true)?;
    Ok(Some(the_g_shape))
}

/// The window of the limits on the curve `c` for the selection of the feature; None
/// (with the error set) when the direction is undetermined.
fn limits_window(
    form: &Form,
    run: &mut Run,
    c: &Handle<dyn GeomCurve>,
) -> Result<Option<Window>, Failure> {
    let mut w = Window {
        prmin: f64::MAX - 2. * precision::confusion(),
        prmax: -f64::MAX + 2. * precision::confusion(),
        pbmin: f64::MAX,
        pbmax: -f64::MAX,
        flag1: false,
    };
    let curve = c.get().ok_or_else(|| {
        crate::errors::null_object("BRepFeat_Form::GlobalPerform", "null barycentre curve")
    })?;
    match form.perf_selection {
        PerfSelection::SelectionSh => {
            let r = parametric_min_max(&form.sbase, c, false)?;
            (w.prmin, w.prmax, w.pbmin, w.pbmax, w.flag1) =
                (r.prmin, r.prmax, r.prbmin, r.prbmax, r.flag);
        }
        PerfSelection::SelectionFU => {
            let r1 = parametric_min_max(&form.s_from, c, false)?;
            let r2 = parametric_min_max(&form.s_until, c, false)?;
            w.flag1 = r2.flag;
            // case of revolutions
            if curve.is_periodic() {
                let period = curve.period()?;
                w.prmax = r2.prmax;
                w.prmin = if w.flag1 {
                    elclib::in_period(r1.prmin, w.prmax - period, w.prmax)
                } else {
                    r1.prmin.min(r2.prmin)
                };
                w.pbmax = r2.prbmax;
                w.pbmin = elclib::in_period(r1.prbmin, w.pbmax - period, w.pbmax);
            } else {
                w.prmin = r1.prmin.min(r2.prmin);
                w.prmax = r1.prmax.max(r2.prmax);
                w.pbmin = r1.prbmin.min(r2.prbmin);
                w.pbmax = r1.prbmax.max(r2.prbmax);
            }
        }
        PerfSelection::SelectionShU => {
            if !form.just_feat && run.limits.sens == 0 {
                run.limits.sens = 1;
            }
            if run.limits.sens == 0 {
                return Ok(None);
            }
            let r1 = parametric_min_max(&form.s_until, c, false)?;
            let r2 = parametric_min_max(&form.sbase, c, false)?;
            w.flag1 = r2.flag;
            if run.limits.sens == 1 {
                (w.prmin, w.prmax, w.pbmin, w.pbmax) = (r2.prmin, r1.prmax, r2.prbmin, r1.prbmax);
            } else if run.limits.sens == -1 {
                (w.prmin, w.prmax, w.pbmin, w.pbmax) = (r1.prmin, r2.prmax, r1.prbmin, r2.prbmax);
            }
        }
        PerfSelection::SelectionU => {
            if run.limits.sens == 0 {
                return Ok(None);
            }
            // Find parts of the tool containing descendants of Shape Until
            let r1 = parametric_min_max(&form.s_until, c, false)?;
            w.flag1 = r1.flag;
            if run.limits.sens == 1 {
                (w.prmin, w.prmax, w.pbmin, w.pbmax) = (-f64::MAX, r1.prmax, -f64::MAX, r1.prbmax);
            } else if run.limits.sens == -1 {
                (w.prmin, w.prmax, w.pbmin, w.pbmax) = (r1.prmin, f64::MAX, r1.prbmin, f64::MAX);
            }
        }
        PerfSelection::NoSelection => {}
    }
    Ok(Some(w))
}

/// The refinement of the window at the face of a part of the tool that is a
/// descendant of a limiting shape: at the end of the window for the shape Until
/// (`at_until`), at its start for the shape From.
fn refine_at_face(
    w: &mut Window,
    c: &Handle<dyn GeomCurve>,
    face: &Shape,
    part: &Shape,
    forward: bool,
) -> Result<(), Failure> {
    let delta = precision::confusion();
    let r1 = parametric_min_max(face, c, false)?;
    let r2 = parametric_min_max(part, c, false)?;
    let (flag2, flag3) = (r2.flag, r1.flag);
    if forward {
        let mut test_ok = !flag2;
        if flag2 {
            test_ok = !w.flag1;
            if w.flag1 && r2.prmax > w.prmin + delta {
                test_ok = !flag3;
                if flag3 && r1.prmax == r2.prmax {
                    test_ok = true;
                }
            }
        }
        if r1.prbmin < w.pbmax && test_ok {
            if flag2 {
                w.flag1 = flag2;
                w.prmax = r2.prmax;
            }
            w.pbmax = r1.prbmin;
        }
    } else {
        let mut test_ok = !flag2;
        if flag2 {
            test_ok = !w.flag1;
            if w.flag1 && r2.prmin < w.prmax - delta {
                test_ok = !flag3;
                if flag3 && r1.prmin == r2.prmin {
                    test_ok = true;
                }
            }
        }
        if r1.prbmax > w.pbmin && test_ok {
            if flag2 {
                w.flag1 = flag2;
                w.prmin = r2.prmin;
            }
            w.pbmin = r1.prbmax;
        }
    }
    Ok(())
}

/// Finer choice of ParametricMinMax in case when the tool intersects Shapes From and
/// Until: the first face of each part that descends from the limiting shape `s`
/// narrows the window (at its end for Until along the direction, at its start for From).
fn refine_at_limit(
    w: &mut Window,
    c: &Handle<dyn GeomCurve>,
    s: &Shape,
    builder: &BRepFeatBuilder,
    lshape: &List<Shape>,
    sens: i32,
    until: bool,
) -> Result<(), Failure> {
    if s.is_null() {
        return Ok(());
    }
    let mut map_f = MapOfShape::default();
    descendants(s, builder, &mut map_f)?;
    if map_f.is_empty() {
        return Ok(());
    }
    for part in lshape.iter() {
        let Some(face) =
            Explorer::new(part, ShapeEnum::Face, ShapeEnum::Shape)?.find(|f| map_f.contains(f))
        else {
            continue;
        };
        // modif of the test for cts21181 : (prbmax2 and prnmin2) -> (prbmin1 and prbmax1)
        // correction take into account flag2 for pro15323 and flag3 for pro16060
        match (sens, until) {
            (1, true) | (-1, false) => refine_at_face(w, c, &face, part, true)?,
            (-1, true) | (1, false) => refine_at_face(w, c, &face, part, false)?,
            _ => {}
        }
    }
    Ok(())
}

/// The window bounds applying to a part: those of the projections unless both the part
/// and the limits meet the curve.
fn part_window(w: &Window, part: (f64, f64, f64, f64), flag2: bool) -> (f64, f64, f64, f64) {
    let (prmin1, prmax1, prbmin1, prbmax1) = part;
    if !flag2 || !w.flag1 {
        (w.pbmin, w.pbmax, prbmin1, prbmax1)
    } else {
        (w.prmin, w.prmax, prmin1, prmax1)
    }
}

/// Parse PartsOfTool to preserve or not depending on ParametricMinMax: the parts
/// overlapping the window are kept; NoParts when none is.
fn keep_parts(
    form: &mut Form,
    w: &Window,
    c: &Handle<dyn GeomCurve>,
    builder: &mut BRepFeatBuilder,
    lshape: &List<Shape>,
) -> Result<bool, Failure> {
    let delta = precision::confusion();
    let curve = c.get().ok_or_else(|| {
        crate::errors::null_object("BRepFeat_Form::GlobalPerform", "null barycentre curve")
    })?;
    let mut keep = false;
    for part in lshape.iter() {
        let (bounds, flag2) = if curve.is_periodic() {
            let period = curve.period()?;
            let r = parametric_min_max(part, c, true)?;
            let prmin1 = if r.flag {
                elclib::in_period(r.prmin, r.prmax - period, r.prmax)
            } else {
                r.prmin
            };
            let prbmin1 = elclib::in_period(r.prbmin, r.prbmax - period, r.prbmax);
            ((prmin1, r.prmax, prbmin1, r.prbmax), r.flag)
        } else {
            let r = parametric_min_max(part, c, false)?;
            ((r.prmin, r.prmax, r.prbmin, r.prbmax), r.flag)
        };
        let (pmin, pmax, min, max) = part_window(w, bounds, flag2);
        if min <= pmax - delta && max >= pmin + delta {
            keep = true;
            builder.keep_part(part)?;
        }
    }
    // Case when no part of the tool is preserved
    if !keep {
        fail(form, StatusError::NoParts);
    }
    Ok(keep)
}

/// case JustFeature -> all PartsOfTool overlapping the window, in a compound, make the
/// shape.
fn just_feature(
    form: &mut Form,
    w: &Window,
    c: &Handle<dyn GeomCurve>,
    lshape: &List<Shape>,
) -> Result<(), Failure> {
    let delta = precision::confusion();
    let b = BRepBuilder::default();
    let mut compo = Compound::default();
    b.make_compound(&mut compo);
    let mut compo = compo.into_shape();
    for part in lshape.iter() {
        let r = parametric_min_max(part, c, false)?;
        let (pmin, pmax, min, max) = part_window(w, (r.prmin, r.prmax, r.prbmin, r.prbmax), r.flag);
        if min < pmax - delta && max > pmin + delta && !part.is_null() {
            b.add(&mut compo, part)?;
        }
    }
    form.base.my_shape = compo;
    Ok(())
}

/// Selection of pieces of tool to be preserved; false (with the error set) when the
/// feature fails.
fn select_parts<F: FormFeature + ?Sized>(
    feature: &mut F,
    run: &mut Run,
    builder: &mut BRepFeatBuilder,
    lshape: &List<Shape>,
) -> Result<bool, Failure> {
    let c = feature.baryc_curve()?;
    let form = feature.form_mut();
    if c.is_null() {
        fail(form, StatusError::EmptyBaryCurve);
        return Ok(false);
    }
    let Some(mut w) = limits_window(form, run, &c)? else {
        fail(form, StatusError::IncDirection);
        return Ok(false);
    };
    let sens = run.limits.sens;
    if form.perf_selection != PerfSelection::NoSelection {
        refine_at_limit(
            &mut w,
            &c,
            &form.s_until.clone(),
            builder,
            lshape,
            sens,
            true,
        )?;
        refine_at_limit(
            &mut w,
            &c,
            &form.s_from.clone(),
            builder,
            lshape,
            sens,
            false,
        )?;
    }
    if !form.just_feat {
        return keep_parts(form, &w, &c, builder, lshape);
    }
    just_feature(form, &w, &c, lshape)?;
    Ok(true)
}

/// The case without gluing: false when the feature failed (its error is set).
pub(super) fn perform_without_glue<F: FormFeature + ?Sized>(
    feature: &mut F,
    run: &mut Run,
) -> Result<bool, Failure> {
    let form = feature.form_mut();
    let mut the_g_shape = form.g_shape.clone();
    let comp = limit_compound(form, run)?;
    update_selection(form, run);
    let has_solid = Explorer::new(&comp, ShapeEnum::Solid, ShapeEnum::Shape)?.more();
    if has_solid && !comp.is_null() && !form.g_shape.is_null() {
        match cut_tool(form, run, &comp)? {
            Some(g) => the_g_shape = g,
            None => return Ok(false),
        }
    }
    //--- generation of "just feature" for assembly = Parts of tool
    let b_flag = form.perf_selection != PerfSelection::NoSelection;
    let mut the_builder = BRepFeatBuilder::new();
    the_builder.init_with_tool(&form.sbase, &the_g_shape);
    the_builder.set_operation_with_flag(i32::from(form.fuse), b_flag);
    the_builder.perform();
    let mut lshape = List::default();
    the_builder.parts_of_tool(&mut lshape)?;
    if !lshape.is_empty()
        && form.perf_selection != PerfSelection::NoSelection
        && !select_parts(feature, run, &mut the_builder, &lshape)?
    {
        return Ok(false);
    }
    let form = feature.form_mut();
    //--- Generation of result myShape
    if !form.just_feat {
        // removal of edges of section that have no common vertices
        // with PartsOfTool preserved
        if b_flag {
            the_builder.perform_result()?;
        }
        form.base.my_shape = the_builder.shape().clone();
    }
    form.base.set_done();
    Ok(true)
}
