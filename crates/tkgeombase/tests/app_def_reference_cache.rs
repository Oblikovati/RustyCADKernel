//! AppDef_Variational over the process-wide reference cache of the FEmTool linear
//! criteria (docs/tkgeombase-progress.md, FEmTool). The exception of a WorkDegree
//! above 14 depends on the calls made before it in the process, so this binary holds
//! one test: in `app_def_behavior` the other tests filled the cache in parallel and
//! the assertion flipped between runs.
#[path = "app_def_behavior/support.rs"]
mod support;

use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::app_def::{Variational, VariationalOptions};
use rustycad_tkgeombase::app_par_curves::Constraint::PassPoint;
use support::{arc_line, constraints, kind};

fn smoothing(options: VariationalOptions) -> Result<Variational, Failure> {
    let ends = constraints(&[(1, PassPoint), (8, PassPoint)]);
    Variational::new(&arc_line(8), 1, 8, &ends, options)
}

#[test]
fn a_degree_above_the_reference_integrals_depends_on_the_cached_order() {
    let o = VariationalOptions::default();
    let too_high = VariationalOptions {
        max_degree: 50,
        with_cutting: false,
        ..o
    };
    // MaxDegree is clamped to 30 by the constructor only. On an empty cache the
    // element criteria refuse the degree (their reference integrals stop at 14).
    assert_eq!(kind(smoothing(too_high)), "Standard_ConstructionError");
    // A refused call leaves the cache empty, so it is refused again.
    assert_eq!(kind(smoothing(too_high)), "Standard_ConstructionError");
    // A default smoothing caches the integrals of its constraint order; the same
    // order then skips the check and reads past the 120 integrals.
    assert_eq!(kind(smoothing(o)), "none");
    assert_eq!(kind(smoothing(too_high)), "Standard_OutOfRange");
}
