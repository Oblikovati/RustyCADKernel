//! Native std::swap policy prefixes precede canonical ordered graph exchange.
#[path = "ordered_policy_exchange/adapters.rs"]
mod adapters;
#[path = "ordered_policy_exchange/fixture.rs"]
mod fixture;
#[path = "ordered_policy_exchange/scenarios.rs"]
mod scenarios;
use adapters::{Map, Set};
use scenarios::{scenario, self_assign};
macro_rules! suites {
    ($module:ident,$map:ty,$prefix:literal) => {
        mod $module {
            use super::*;
            #[test]
            fn exchange_all_native_throw_prefixes() {
                for rejected in 0..4 {
                    scenario::<$map>($prefix, false, rejected, false);
                }
            }
            #[test]
            fn move_assignment_all_native_throw_prefixes() {
                for rejected in 0..4 {
                    scenario::<$map>($prefix, true, rejected, false);
                }
            }
            #[test]
            fn exchange_empty_graphs_keeps_native_bucket_selections() {
                scenario::<$map>($prefix, false, 0, true);
            }
            #[test]
            fn move_assignment_empty_graphs_keeps_native_bucket_selections() {
                scenario::<$map>($prefix, true, 0, true);
            }
            #[test]
            fn self_move_assignment_never_invokes_the_policy() {
                self_assign::<$map>($prefix);
            }
        }
    };
}
suites!(set, Set, "set");
suites!(data, Map, "data");
