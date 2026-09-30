// Rust translation of GeomFill_GordonUtilities.pxx (reorderNetwork) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The reordering of a Gordon network into a monotone layout.
use super::gordon_contacts::ContactSet;
use super::gordon_normalize::{NetworkTables, is_monotone_layout, open_closed_boundary_candidates};
use super::gordon_order::{
    ContactTargets, OrderedCurve, choose_ordered_contact, ordered_curves, oriented_curve,
};
use super::gordon_params::ParamTable;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomBSplineCurve;

/// The network being reordered: the source curves, their tables and contacts.
pub(crate) struct SourceNetwork<'n> {
    pub(crate) profiles: &'n mut Vec<Handle<GeomBSplineCurve>>,
    pub(crate) guides: &'n mut Vec<Handle<GeomBSplineCurve>>,
    pub(crate) profile_params: &'n mut ParamTable,
    pub(crate) guide_params: &'n mut ParamTable,
    pub(crate) contact_sets: &'n [ContactSet],
}

// `i / (n - 1)`, or 0 for a single curve (the native targets).
fn target(i: usize, n: usize) -> f64 {
    if n == 1 {
        0.0
    } else {
        i as f64 / (n - 1) as f64
    }
}

// The orientation of each ordered curve: forward when its parameter at the first
// ordered opposite curve is at most the one at the last.
fn orientations(
    order: &[OrderedCurve],
    opposite: (&OrderedCurve, &OrderedCurve),
    param_at: impl Fn(usize, usize) -> f64,
) -> Vec<bool> {
    order
        .iter()
        .map(|curve| {
            param_at(curve.index, opposite.0.index) <= param_at(curve.index, opposite.1.index)
        })
        .collect()
}

// The targets of the pair (pi, gi) of the ordered network, from the selections made
// before it in `result`.
fn pair_targets(
    (pi, gi): (usize, usize),
    (nb_p, nb_g): (usize, usize),
    (forward_p, forward_g): (bool, bool),
    result: (&ParamTable, &ParamTable),
) -> ContactTargets {
    let (is_profile_seam, is_guide_seam) = (gi + 1 == nb_g, pi + 1 == nb_p);
    ContactTargets {
        profile_forward: forward_p,
        guide_forward: forward_g,
        profile_target: target(gi, nb_g),
        guide_target: target(pi, nb_p),
        previous_profile_param: if gi == 0 {
            0.0
        } else {
            result.0.at(pi, gi - 1)
        },
        previous_guide_param: if pi == 0 {
            0.0
        } else {
            result.1.at(pi - 1, gi)
        },
        has_previous_profile: gi != 0,
        has_previous_guide: pi != 0,
        previous_profile_target: if gi < 2 {
            0.0
        } else {
            (gi - 1) as f64 / (nb_g - 1) as f64
        },
        previous_guide_target: if pi < 2 {
            0.0
        } else {
            (pi - 1) as f64 / (nb_p - 1) as f64
        },
        first_profile_param: if is_profile_seam {
            result.0.at(pi, 0)
        } else {
            0.0
        },
        first_guide_param: if is_guide_seam {
            result.1.at(0, gi)
        } else {
            0.0
        },
        is_profile_seam,
        is_guide_seam,
    }
}

/// `reorderNetwork`: the profiles ordered by their mean guide parameter and the guides
/// by their mean profile parameter (a closed family opened, see `ordered_curves`), each
/// curve oriented along the opposite family, one contact chosen per pair in row order,
/// the seam contacts opened; the network is replaced only when the layout is monotone.
pub(crate) fn reorder_network(src: SourceNetwork<'_>, the_tolerance: f64) -> Result<bool, Failure> {
    let (nb_p, nb_g) = (src.profiles.len(), src.guides.len());
    if nb_p < 2 || nb_g < 2 {
        return Ok(false);
    }
    let a_profile_order: Vec<OrderedCurve> = ordered_curves(&src.guide_params.row_means());
    let a_guide_order: Vec<OrderedCurve> = ordered_curves(&src.profile_params.column_means());
    let g_ends = (&a_guide_order[0], &a_guide_order[nb_g - 1]);
    let p_ends = (&a_profile_order[0], &a_profile_order[nb_p - 1]);
    let profile_forward =
        orientations(&a_profile_order, g_ends, |p, g| src.profile_params.at(p, g));
    let guide_forward = orientations(&a_guide_order, p_ends, |g, p| src.guide_params.at(p, g));
    let mut a_profiles: Vec<Handle<GeomBSplineCurve>> = Vec::with_capacity(nb_p);
    for (curve, forward) in a_profile_order.iter().zip(&profile_forward) {
        a_profiles.push(oriented_curve(&src.profiles[curve.index], *forward)?);
    }
    let mut a_guides: Vec<Handle<GeomBSplineCurve>> = Vec::with_capacity(nb_g);
    for (curve, forward) in a_guide_order.iter().zip(&guide_forward) {
        a_guides.push(oriented_curve(&src.guides[curve.index], *forward)?);
    }
    let mut result = (ParamTable::new(nb_p, nb_g), ParamTable::new(nb_p, nb_g));
    for (pi, src_p) in a_profile_order.iter().enumerate() {
        for (gi, src_g) in a_guide_order.iter().enumerate() {
            let forward = (profile_forward[pi], guide_forward[gi]);
            let t = pair_targets((pi, gi), (nb_p, nb_g), forward, (&result.0, &result.1));
            let set: &ContactSet = &src.contact_sets[src_p.index * nb_g + src_g.index];
            let curves = (&src.profiles[src_p.index], &src.guides[src_g.index]);
            let Some(a_contact) = choose_ordered_contact(set, curves, &t, the_tolerance)? else {
                return Ok(false);
            };
            result.0.set(pi, gi, a_contact.profile_param);
            result.1.set(pi, gi, a_contact.guide_param);
        }
    }
    let mut net = NetworkTables {
        profiles: &a_profiles,
        guides: &a_guides,
        profile_params: &mut result.0,
        guide_params: &mut result.1,
    };
    open_closed_boundary_candidates(&mut net, the_tolerance)?;
    if !is_monotone_layout(&result.0, &result.1) {
        return Ok(false);
    }
    *src.profiles = a_profiles;
    *src.guides = a_guides;
    *src.profile_params = result.0;
    *src.guide_params = result.1;
    Ok(true)
}
