//! Preserve the skipped successor after removal and libstdc++ ordering of equal roots.
use super::IntTools;
use crate::int_tools::Root;
use rustycad_tkg3d::top_abs::State;
impl IntTools {
    /// Removes close later roots with the native forward-only index; `IntTools::remove_identical_roots(&mut roots, 1e-7)`.
    pub fn remove_identical_roots(roots: &mut Vec<Root>, tolerance: f64) {
        let threshold = 0.5 * tolerance;
        let mut first = 0;
        while first < roots.len() {
            let mut next = first + 1;
            while next < roots.len() {
                if (roots[first].root() - roots[next].root()).abs() < threshold {
                    roots.remove(next);
                }
                // OCCT advances after Remove; the shifted successor is deliberately skipped.
                next += 1;
            }
            first += 1;
        }
    }
    /// Uses the native unstable ordering; the tolerance is unused; `IntTools::sort_roots(&mut roots, 0.)`.
    pub fn sort_roots(roots: &mut [Root], _tolerance: f64) {
        rustycad_tkernel::stl::sort_by(roots, |first, last| first.root() < last.root());
    }
    /// Classifies each root from its kind and endpoint values; `IntTools::find_root_states(&mut roots, 1e-7)`.
    pub fn find_root_states(roots: &mut [Root], tolerance: f64) {
        for root in roots {
            let (_, _, first, last) = root.interval();
            let states = match root.root_type() {
                0 => {
                    if first > 0. && last < 0. {
                        (State::Out, State::In)
                    } else {
                        (State::In, State::Out)
                    }
                }
                1 => (State::On, State::On),
                2 => smart_states(first, last, tolerance),
                _ => continue,
            };
            root.set_state_before(states.0);
            root.set_state_after(states.1);
        }
    }
}
fn smart_states(first: f64, last: f64, tolerance: f64) -> (State, State) {
    if last.abs() < tolerance {
        return (if first > 0. { State::Out } else { State::In }, State::On);
    }
    (State::On, if last > 0. { State::Out } else { State::In })
}
