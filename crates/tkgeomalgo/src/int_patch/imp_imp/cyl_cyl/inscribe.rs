// Rust translation of InscribePoint, InscribeInterval, ExcludeNearElements and MinMax
// of ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_ImpImpIntersection.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Periodic parameters brought into a target range, and the small sorts of the
//! cylinder-cylinder walking.
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::precision;
use rustycad_tkmath::bnd::Range1;
use rustycad_tkmath::elclib::in_period;

/// Whether `u` is in `[uf - tol, ul + tol]` (the test of InscribePoint).
fn is_inscribed(uf: f64, ul: f64, u: f64, tol_2d: f64) -> bool {
    (uf - u <= tol_2d) && (u - ul <= tol_2d)
}

/// `InscribePoint(theUfTarget, theUlTarget, theUGiven, theTol2D, thePeriod,
/// theFlForce)`: brings `u_given` into the target range by periods; with `fl_force`,
/// a point already inside is moved by one period when it stays inside. False for an
/// infinite parameter or when it cannot be inscribed.
pub(in super::super) fn inscribe_point(
    uf_target: f64,
    ul_target: f64,
    u_given: &mut f64,
    tol_2d: f64,
    period: f64,
    fl_force: bool,
) -> bool {
    if precision::is_infinite(*u_given) {
        return false;
    }
    if is_inscribed(uf_target, ul_target, *u_given, tol_2d) {
        // It has already been inscribed.
        if fl_force {
            let up: f64 = *u_given + period;
            if is_inscribed(uf_target, ul_target, up, tol_2d) {
                *u_given = up;
                return true;
            }
            let down: f64 = *u_given - period;
            if is_inscribed(uf_target, ul_target, down, tol_2d) {
                *u_given = down;
            }
        }
        return true;
    }
    let uf: f64 = uf_target - tol_2d;
    let ul: f64 = uf + period;
    *u_given = in_period(*u_given, uf, ul);
    is_inscribed(uf_target, ul_target, *u_given, tol_2d)
}

/// `InscribeInterval(theUfTarget, theUlTarget, theRange, theTol2D, thePeriod)`: shifts
/// the range so that one of its bounds is in the target range.
pub(in super::super) fn inscribe_interval(
    uf_target: f64,
    ul_target: f64,
    range: &mut Range1,
    tol_2d: f64,
    period: f64,
) -> bool {
    let mut upar: f64 = 0.0;
    if !range.get_min(&mut upar) {
        return false;
    }
    let delta: f64 = range.delta();
    let force_min: bool = (ul_target - upar).abs() < tol_2d;
    if inscribe_point(uf_target, ul_target, &mut upar, tol_2d, period, force_min) {
        range.set_void();
        range.add(upar);
        range.add(upar + delta);
        return true;
    }
    if !range.get_max(&mut upar) {
        return false;
    }
    let force_max: bool = (uf_target - upar).abs() < tol_2d;
    if !inscribe_point(uf_target, ul_target, &mut upar, tol_2d, period, force_max) {
        return false;
    }
    range.set_void();
    range.add(upar);
    range.add(upar - delta);
    true
}

/// `ExcludeNearElements(theArr, theNOfMembers, theUSurf1f, theUSurf1l, theTol)`: of two
/// sorted neighbours nearer than `tol`, the lower one becomes infinite (forgotten) and
/// the upper one their middle (or the lower one when it is 0 or a bound of the
/// surface); true when an element changed. The finite elements are in `[0, T]`.
pub(in super::super) fn exclude_near_elements(
    arr: &mut [f64],
    usurf1f: f64,
    usurf1l: f64,
    tol: f64,
) -> bool {
    let mut ret_val: bool = false;
    for i in 1..arr.len() {
        let (a, b) = (arr[i], arr[i - 1]);
        // Here a >= b.
        if precision::is_infinite(a) {
            break;
        }
        if (a - b) < tol {
            arr[i] = if b != 0.0 && b != usurf1f && b != usurf1l {
                (a + b) / 2.0
            } else {
                b
            };
            arr[i - 1] = precision::infinite();
            ret_val = true;
        }
    }
    ret_val
}

/// `MinMax(theParMIN, theParMAX)`: orders the two values.
pub(in super::super) fn min_max(par_min: &mut f64, par_max: &mut f64) {
    if *par_min > *par_max {
        std::mem::swap(par_min, par_max);
    }
}

/// `std::sort` of libstdc++ on at most 16 elements: the insertion sort of
/// `__final_insertion_sort`, with `less` as the only comparison (the order of equal or
/// unordered elements is the native one).
pub(in super::super) fn insertion_sort<T: Copy>(items: &mut [T], less: impl Fn(&T, &T) -> bool) {
    for i in 1..items.len() {
        let val: T = items[i];
        if less(&val, &items[0]) {
            items.copy_within(0..i, 1);
            items[0] = val;
            continue;
        }
        let mut j: usize = i;
        while less(&val, &items[j - 1]) {
            items[j] = items[j - 1];
            j -= 1;
        }
        items[j] = val;
    }
}

/// `Bnd_Range::Split(theVal, theList, thePeriod)` of every range of the list.
pub(in super::super) fn split_all(
    ranges: &List<Range1>,
    value: f64,
    period: f64,
) -> Result<List<Range1>, rustycad_tkernel::standard::Failure> {
    let mut out: List<Range1> = List::default();
    for r in ranges.iter() {
        r.split(value, &mut out, period)?;
    }
    Ok(out)
}
