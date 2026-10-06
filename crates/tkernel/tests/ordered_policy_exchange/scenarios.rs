use super::{
    adapters::OrderedExchangeFixture,
    fixture::{SwapPolicy, SwapTrace, native_row},
};
use rustycad_tkernel::ncollection::{BaseAllocator, MapLinkNode};
use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    rc::Rc,
};
struct GraphSnapshot {
    first: *mut (),
    buckets: *const *mut MapLinkNode,
    arena: BaseAllocator,
}
impl GraphSnapshot {
    fn capture<M: OrderedExchangeFixture>(map: &M) -> Self {
        Self {
            first: map.basic_first_node().cast(),
            buckets: map
                .basic_bucket_slots()
                .map_or(std::ptr::null(), |slots| slots.as_ptr()),
            arena: map.allocator().clone(),
        }
    }
    fn comparison<M: OrderedExchangeFixture>(&self, map: &M) -> (u8, u8, u8) {
        let current = Self::capture(map);
        (
            u8::from(self.first == current.first),
            u8::from(self.buckets == current.buckets),
            u8::from(self.arena == current.arena),
        )
    }
}
struct SwapScenario<M> {
    first: M,
    second: M,
    before_first: GraphSnapshot,
    before_second: GraphSnapshot,
    trace: Rc<SwapTrace>,
}
impl<M: OrderedExchangeFixture> SwapScenario<M> {
    fn new(empty: bool, rejected: usize) -> Self {
        let trace = Rc::new(SwapTrace::default());
        let (first, second) = Self::populated_pair(&trace, empty);
        let (before_first, before_second) = (
            GraphSnapshot::capture(&first),
            GraphSnapshot::capture(&second),
        );
        trace.rejected.set(rejected);
        Self {
            first,
            second,
            before_first,
            before_second,
            trace,
        }
    }
    fn populated_pair(trace: &Rc<SwapTrace>, empty: bool) -> (M, M) {
        let mut first = M::construct(
            SwapPolicy::new(7, trace),
            101,
            BaseAllocator::incremental(1024),
        );
        let mut second = M::construct(
            SwapPolicy::new(19, trace),
            13,
            BaseAllocator::incremental(1024),
        );
        if !empty {
            first.insert(4, trace);
            first.insert(1, trace);
            second.insert(7, trace);
        }
        (first, second)
    }
    fn verify_release(self, empty: bool) {
        let trace = Rc::clone(&self.trace);
        let (first, second) = (
            self.first.allocator().clone(),
            self.second.allocator().clone(),
        );
        if !empty {
            assert!(first.reset(false).is_err());
            assert!(second.reset(false).is_err());
        }
        drop(self);
        assert_eq!(
            trace.drops.get(),
            if empty { 0 } else { 3 * M::PAYLOADS_PER_NODE }
        );
        assert!(first.reset(false).is_ok());
        assert!(second.reset(false).is_ok());
    }
    fn execute(&mut self, assign: bool) -> bool {
        catch_unwind(AssertUnwindSafe(|| {
            if assign {
                self.first.native_assign(&mut self.second);
            } else {
                self.first.native_exchange(&mut self.second);
            }
        }))
        .is_ok()
    }
    fn identity_columns(&self, success: bool) -> String {
        let (first, second) = if success {
            (&self.before_second, &self.before_first)
        } else {
            (&self.before_first, &self.before_second)
        };
        let (node1, table1, arena1) = first.comparison(&self.first);
        let (node2, table2, arena2) = second.comparison(&self.second);
        format!("{node1} {node2} {table1} {table2} {arena1} {arena2}")
    }
    fn state_columns(&self) -> String {
        format!(
            "{} {} {} {} {} {}",
            self.first.policy().state,
            self.second.policy().state,
            self.first.size(),
            self.second.size(),
            self.first.nb_buckets(),
            self.second.nb_buckets()
        )
    }
    fn match_native(&self, label: &str, success: bool) {
        let status = if success { "ok" } else { "throw" };
        native_row(
            label,
            format!(
                "{label} {status} {} {} 0 {} {}",
                self.state_columns(),
                self.identity_columns(success),
                self.trace.drops.get(),
                self.trace.events.borrow()
            ),
        );
    }
    fn match_keys(&self, success: bool, empty: bool) {
        if empty {
            assert!(self.first.keys().is_empty());
            assert!(self.second.keys().is_empty());
            return;
        }
        let (first, second) = if success {
            (vec![7], vec![4, 1])
        } else {
            (vec![4, 1], vec![7])
        };
        assert_eq!((self.first.keys(), self.second.keys()), (first, second));
    }
}
pub fn scenario<M: OrderedExchangeFixture>(
    prefix: &str,
    assign: bool,
    rejected: usize,
    empty: bool,
) {
    let mut scenario = SwapScenario::<M>::new(empty, rejected);
    let success = scenario.execute(assign);
    let operation = if assign { "assign" } else { "exchange" };
    let label = format!("{prefix}_{operation}_{rejected}_{}", u8::from(empty));
    scenario.match_native(&label, success);
    scenario.match_keys(success, empty);
    scenario.verify_release(empty);
}
pub fn self_assign<M: OrderedExchangeFixture>(prefix: &str) {
    let trace = Rc::new(SwapTrace::default());
    let mut map = M::construct(SwapPolicy::new(7, &trace), 101, BaseAllocator::common());
    map.insert(4, &trace);
    let node = map.basic_first_node();
    trace.rejected.set(1);
    map.native_self_assign();
    let label = format!("{prefix}_self");
    native_row(
        &label,
        format!(
            "{label} {} {} {} {} 0 {}",
            map.policy().state,
            map.size(),
            u8::from(node == map.basic_first_node()),
            trace.phase.get(),
            trace.drops.get()
        ),
    );
}

pub fn self_exchange<M: OrderedExchangeFixture>(prefix: &str, rejected: usize) {
    let trace = Rc::new(SwapTrace::default());
    let mut map = M::construct(SwapPolicy::new(7, &trace), 101, BaseAllocator::common());
    map.insert(4, &trace);
    let node = map.basic_first_node();
    trace.rejected.set(rejected);
    let success = catch_unwind(AssertUnwindSafe(|| map.native_self_exchange())).is_ok();
    let status = if success { "ok" } else { "throw" };
    let label = format!("{prefix}_self_exchange_{rejected}");
    native_row(
        &label,
        format!(
            "{label} {status} {} {} {} {} 0 {} {}",
            map.policy().state,
            map.size(),
            u8::from(node == map.basic_first_node()),
            trace.phase.get(),
            trace.drops.get(),
            trace.events.borrow()
        ),
    );
    assert_eq!(map.keys(), vec![4]);
    drop(map);
    assert_eq!(trace.drops.get(), M::PAYLOADS_PER_NODE);
}
