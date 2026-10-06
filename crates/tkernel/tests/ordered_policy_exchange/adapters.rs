use super::fixture::{Payload, SwapPolicy, SwapTrace};
use rustycad_tkernel::ncollection::{
    BaseAllocator, BaseMap, BasicMapIteratorSource, OrderedDataMap, OrderedMap,
};
use std::rc::Rc;
pub type Set = OrderedMap<Payload, SwapPolicy>;
pub type Map = OrderedDataMap<Payload, Payload, SwapPolicy>;
pub trait OrderedExchangeFixture: BaseMap + BasicMapIteratorSource {
    fn construct(policy: SwapPolicy, buckets: usize, arena: BaseAllocator) -> Self;
    fn insert(&mut self, key: i32, trace: &Rc<SwapTrace>);
    fn policy(&self) -> &SwapPolicy;
    fn native_exchange(&mut self, other: &mut Self);
    fn native_assign(&mut self, other: &mut Self);
    fn native_self_assign(&mut self);
    fn keys(&self) -> Vec<i32>;
}
macro_rules! adapter {
    ($map:ty,$insert:expr,$keys:expr) => {
        impl OrderedExchangeFixture for $map {
            fn construct(policy: SwapPolicy, buckets: usize, arena: BaseAllocator) -> Self {
                Self::with_allocator(policy, buckets, arena)
            }
            fn insert(&mut self, key: i32, trace: &Rc<SwapTrace>) {
                ($insert)(self, key, trace);
            }
            fn policy(&self) -> &SwapPolicy {
                self.hasher()
            }
            fn native_exchange(&mut self, other: &mut Self) {
                self.exchange_with(other, SwapPolicy::native_swap);
            }
            fn native_assign(&mut self, other: &mut Self) {
                let address = std::ptr::from_mut(self);
                assert_eq!(
                    address,
                    std::ptr::from_mut(self.move_assign_with(other, SwapPolicy::native_swap))
                );
            }
            fn native_self_assign(&mut self) {
                let address = std::ptr::from_mut(self);
                assert_eq!(address, std::ptr::from_mut(self.assign_self()));
            }
            fn keys(&self) -> Vec<i32> {
                ($keys)(self)
            }
        }
    };
}
adapter!(
    Set,
    |map: &mut Set, key, trace| {
        map.add(Payload::new(key, trace));
    },
    |map: &Set| map.iter().map(|key| key.key).collect()
);
adapter!(
    Map,
    |map: &mut Map, key, trace| {
        map.bind(Payload::new(key, trace), Payload::new(key * 10, trace));
    },
    |map: &Map| map.items().map(|(key, _)| key.key).collect()
);
