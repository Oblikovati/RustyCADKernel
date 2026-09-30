// Rust translation of GeomFill_GordonUtilities.pxx (Contact, ContactSet and the
// contact discovery: appendContact .. fillContactTable) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The profile/guide contacts of a Gordon network.
use super::gordon_params::{ParamTable, null_curve, parameter_resolution};
use super::network_surface_skin::point_of;
use crate::geom_api::ExtremaCurveCurve;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::stl;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCurve};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkgeombase::geom_lib::Tool;
use rustycad_tkmath::gp::{self, Point3};

/// `GordonUtilities::Contact`: one resolved profile/guide contact.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Contact {
    pub(crate) is_valid: bool,
    pub(crate) profile_param: f64,
    pub(crate) guide_param: f64,
    pub(crate) gap: f64,
}

impl Default for Contact {
    fn default() -> Self {
        Self {
            is_valid: false,
            profile_param: 0.0,
            guide_param: 0.0,
            gap: f64::MAX,
        }
    }
}

/// `GordonUtilities::ContactSet`: the admissible contacts of one pair.
#[derive(Clone, Debug, Default)]
pub(crate) struct ContactSet {
    pub(crate) candidates: Vec<Contact>,
}

// `GeomAdaptor_Curve(theCurve).EvalD0(theParameter)`.
pub(crate) fn adaptor_d0(
    curve: &Handle<GeomBSplineCurve>,
    parameter: f64,
) -> Result<Point3, Failure> {
    GeomAdaptorCurve::with_curve(&curve.cast())?.eval_d0(parameter)
}

// The first and last parameters of a working curve.
pub(crate) fn curve_bounds(curve: &Handle<GeomBSplineCurve>) -> Result<(f64, f64), Failure> {
    let c = curve.get().ok_or_else(null_curve)?;
    Ok((c.first_parameter(), c.last_parameter()))
}

/// `appendContact`: the pair of parameters when the curve points are within the
/// tolerance and no kept contact is within the parameter resolutions.
fn append_contact(
    (the_profile, the_guide): (&Handle<GeomBSplineCurve>, &Handle<GeomBSplineCurve>),
    (the_profile_param, the_guide_param): (f64, f64),
    the_tolerance: f64,
    the_contacts: &mut ContactSet,
) -> Result<(), Failure> {
    let a_profile_point: Point3 = adaptor_d0(the_profile, the_profile_param)?;
    let a_guide_point: Point3 = adaptor_d0(the_guide, the_guide_param)?;
    let a_gap: f64 = a_profile_point.distance(&a_guide_point);
    if a_gap > the_tolerance {
        return Ok(());
    }
    let (pf, pl) = curve_bounds(the_profile)?;
    let (gf, gl) = curve_bounds(the_guide)?;
    let (a_profile_tol, a_guide_tol) = (parameter_resolution(pf, pl), parameter_resolution(gf, gl));
    let known = the_contacts.candidates.iter().any(|a_contact| {
        (a_contact.profile_param - the_profile_param).abs() <= a_profile_tol
            && (a_contact.guide_param - the_guide_param).abs() <= a_guide_tol
    });
    if !known {
        the_contacts.candidates.push(Contact {
            is_valid: true,
            profile_param: the_profile_param,
            guide_param: the_guide_param,
            gap: a_gap,
        });
    }
    Ok(())
}

/// `appendExtremaContact`: two close extrema points located on both curves at their
/// midpoint (GeomLib_Tool::Parameter).
fn append_extrema_contact(
    curves: (&Handle<GeomBSplineCurve>, &Handle<GeomBSplineCurve>),
    (the_profile_point, the_guide_point): (Point3, Point3),
    the_tolerance: f64,
    the_contacts: &mut ContactSet,
) -> Result<(), Failure> {
    if the_profile_point.distance(&the_guide_point) > the_tolerance {
        return Ok(());
    }
    let a_shared_point = point_of((the_profile_point.xyz() + the_guide_point.xyz()) * 0.5);
    let Some(a_profile_param) = Tool::parameter(&curves.0.cast(), a_shared_point, the_tolerance)?
    else {
        return Ok(());
    };
    let Some(a_guide_param) = Tool::parameter(&curves.1.cast(), a_shared_point, the_tolerance)?
    else {
        return Ok(());
    };
    append_contact(
        curves,
        (a_profile_param, a_guide_param),
        the_tolerance,
        the_contacts,
    )
}

/// `appendBoundaryContacts`: the ends of each curve located on the other one.
fn append_boundary_contacts(
    curves: (&Handle<GeomBSplineCurve>, &Handle<GeomBSplineCurve>),
    the_tolerance: f64,
    the_contacts: &mut ContactSet,
) -> Result<(), Failure> {
    let (pf, pl) = curve_bounds(curves.0)?;
    let (gf, gl) = curve_bounds(curves.1)?;
    for a_profile_param in [pf, pl] {
        let point: Point3 = adaptor_d0(curves.0, a_profile_param)?;
        if let Some(a_guide_param) = Tool::parameter(&curves.1.cast(), point, the_tolerance)? {
            append_contact(
                curves,
                (a_profile_param, a_guide_param),
                the_tolerance,
                the_contacts,
            )?;
        }
    }
    for a_guide_param in [gf, gl] {
        let point: Point3 = adaptor_d0(curves.1, a_guide_param)?;
        if let Some(a_profile_param) = Tool::parameter(&curves.0.cast(), point, the_tolerance)? {
            append_contact(
                curves,
                (a_profile_param, a_guide_param),
                the_tolerance,
                the_contacts,
            )?;
        }
    }
    Ok(())
}

/// `appendLinearSegmentContact`: the closest points of two straight segments
/// (degree 1, two poles); false for other curves (the general search runs), true
/// otherwise even when no contact is added.
fn append_linear_segment_contact(
    curves: (&Handle<GeomBSplineCurve>, &Handle<GeomBSplineCurve>),
    the_tolerance: f64,
    the_contacts: &mut ContactSet,
) -> Result<bool, Failure> {
    let (profile, guide) = (
        curves.0.get().ok_or_else(null_curve)?,
        curves.1.get().ok_or_else(null_curve)?,
    );
    if profile.degree() != 1
        || guide.degree() != 1
        || profile.nb_poles() != 2
        || guide.nb_poles() != 2
    {
        return Ok(false);
    }
    let (pf, pl) = (profile.first_parameter(), profile.last_parameter());
    let (gf, gl) = (guide.first_parameter(), guide.last_parameter());
    if pl <= pf + precision::p_confusion() || gl <= gf + precision::p_confusion() {
        return Ok(true);
    }
    let (a_p0, a_p1) = (adaptor_d0(curves.0, pf)?, adaptor_d0(curves.0, pl)?);
    let (a_q0, a_q1) = (adaptor_d0(curves.1, gf)?, adaptor_d0(curves.1, gl)?);
    let a_u = a_p1.xyz() - a_p0.xyz();
    let a_v = a_q1.xyz() - a_q0.xyz();
    let a_w = a_p0.xyz() - a_q0.xyz();
    let (a_a, a_b, a_c) = (a_u.square_modulus(), a_u.dot(a_v), a_v.square_modulus());
    let (a_d, a_e) = (a_u.dot(a_w), a_v.dot(a_w));
    if a_a <= gp::resolution() || a_c <= gp::resolution() {
        return Ok(true);
    }
    let a_denom: f64 = a_a * a_c - a_b * a_b;
    if a_denom.abs() <= gp::resolution() {
        return Ok(false);
    }
    let (a_s, a_t) = closest_segment_parameters((a_a, a_b, a_c, a_d, a_e), a_denom);
    let params = (pf + a_s * (pl - pf), gf + a_t * (gl - gf));
    append_contact(curves, params, the_tolerance, the_contacts)?;
    Ok(true)
}

// The clamped closest-point parameters `(s, t)` of two segments from their Gram terms.
fn closest_segment_parameters(
    (a_a, a_b, a_c, a_d, a_e): (f64, f64, f64, f64, f64),
    a_denom: f64,
) -> (f64, f64) {
    let mut a_s: f64 = ((a_b * a_e - a_c * a_d) / a_denom).clamp(0.0, 1.0);
    let mut a_t: f64 = (a_b * a_s + a_e) / a_c;
    if a_t < 0.0 {
        a_t = 0.0;
        a_s = (-a_d / a_a).clamp(0.0, 1.0);
    } else if a_t > 1.0 {
        a_t = 1.0;
        a_s = ((a_b - a_d) / a_a).clamp(0.0, 1.0);
    }
    (a_s, a_t)
}

/// `collectContacts`: the segment contact, or the contacts of the extrema, of the
/// minimal distance between the trimmed curves and of the curve ends, sorted by gap,
/// then profile and guide parameters (libstdc++ `std::sort`).
pub(crate) fn collect_contacts(
    the_profile: &Handle<GeomBSplineCurve>,
    the_guide: &Handle<GeomBSplineCurve>,
    the_tolerance: f64,
) -> Result<ContactSet, Failure> {
    let mut a_contacts = ContactSet::default();
    let curves = (the_profile, the_guide);
    if append_linear_segment_contact(curves, the_tolerance, &mut a_contacts)? {
        return Ok(a_contacts);
    }
    let mut an_extrema = ExtremaCurveCurve::with_curves(&the_profile.cast(), &the_guide.cast())?;
    for an_ext_idx in 1..=an_extrema.nb_extrema() {
        let points = an_extrema.points(an_ext_idx)?;
        append_extrema_contact(curves, points, the_tolerance, &mut a_contacts)?;
    }
    if let Some(points) = an_extrema.total_nearest_points()? {
        append_extrema_contact(curves, points, the_tolerance, &mut a_contacts)?;
    }
    append_boundary_contacts(curves, the_tolerance, &mut a_contacts)?;
    stl::sort_by(&mut a_contacts.candidates, |left, right| {
        if left.gap != right.gap {
            return left.gap < right.gap;
        }
        if left.profile_param != right.profile_param {
            return left.profile_param < right.profile_param;
        }
        left.guide_param < right.guide_param
    });
    Ok(a_contacts)
}

/// `chooseNearestContacts`: the lowest-gap contact of every pair as the first
/// ordering estimate; false when a pair has none.
fn choose_nearest_contacts(
    the_contacts: &[ContactSet],
    the_profile_params: &mut ParamTable,
    the_guide_params: &mut ParamTable,
) -> bool {
    let (nb_profiles, nb_guides) = (the_profile_params.rows, the_profile_params.cols);
    for a_profile_idx in 0..nb_profiles {
        for a_guide_idx in 0..nb_guides {
            let a_set: &ContactSet = &the_contacts[a_profile_idx * nb_guides + a_guide_idx];
            let Some(a_contact) = a_set.candidates.first() else {
                return false;
            };
            the_profile_params.set(a_profile_idx, a_guide_idx, a_contact.profile_param);
            the_guide_params.set(a_profile_idx, a_guide_idx, a_contact.guide_param);
        }
    }
    true
}

/// `maxNearestContactGap`: the largest best-contact gap of the table.
pub(crate) fn max_nearest_contact_gap(the_contacts: &[ContactSet]) -> f64 {
    the_contacts
        .iter()
        .filter_map(|a_set| a_set.candidates.first())
        .fold(0.0_f64, |a_max_gap, first| a_max_gap.max(first.gap))
}

/// `fillContactTable`: the contacts of every profile/guide pair (in pair order; the
/// parallel native loop computes the same independent sets), then the nearest ones as
/// the parameter tables; false when a pair has no contact.
pub(crate) fn fill_contact_table(
    the_profiles: &[Handle<GeomBSplineCurve>],
    the_guides: &[Handle<GeomBSplineCurve>],
    tables: (&mut ParamTable, &mut ParamTable),
    the_contact_sets: &mut Vec<ContactSet>,
    the_tolerance: f64,
) -> Result<bool, Failure> {
    the_contact_sets.clear();
    for a_profile in the_profiles {
        for a_guide in the_guides {
            let a_contact_set = collect_contacts(a_profile, a_guide, the_tolerance)?;
            let empty: bool = a_contact_set.candidates.is_empty();
            the_contact_sets.push(a_contact_set);
            if empty {
                return Ok(false);
            }
        }
    }
    Ok(choose_nearest_contacts(
        the_contact_sets,
        tables.0,
        tables.1,
    ))
}

/// `snappedParameter`: a parameter within the boundary resolution of an end moved to
/// that end.
pub(crate) fn snapped_parameter(
    the_curve: &Handle<GeomBSplineCurve>,
    the_parameter: f64,
    the_tolerance: f64,
) -> Result<f64, Failure> {
    let (a_first, a_last) = curve_bounds(the_curve)?;
    let a_tol: f64 =
        super::gordon_params::boundary_parameter_resolution(a_first, a_last, the_tolerance);
    if (the_parameter - a_first).abs() < a_tol {
        return Ok(a_first);
    }
    if (the_parameter - a_last).abs() < a_tol {
        return Ok(a_last);
    }
    Ok(the_parameter)
}
