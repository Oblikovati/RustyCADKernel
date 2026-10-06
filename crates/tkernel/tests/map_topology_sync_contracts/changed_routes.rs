use rustycad_tkernel::ncollection::{DoubleMap, HashPolicy};
use std::{
    cell::Cell,
    panic::{AssertUnwindSafe, catch_unwind},
};
#[derive(Default)]
struct CellRoute;
impl HashPolicy<Cell<usize>> for CellRoute {
    fn hash(&self, key: &Cell<usize>) -> usize {
        key.get()
    }
    fn equal(&self, first: &Cell<usize>, second: &Cell<usize>) -> bool {
        first.get() == second.get()
    }
}
type InteriorMap = DoubleMap<Cell<usize>, Cell<usize>, CellRoute, CellRoute>;
fn interior_map() -> InteriorMap {
    let mut map = InteriorMap::default();
    map.bind(Cell::new(1), Cell::new(11)).unwrap();
    map.bind(Cell::new(2), Cell::new(22)).unwrap();
    map
}
#[test]
fn changed_secondary_key_restores_first_chain_before_diagnosing_missing_partner() {
    let mut map = interior_map();
    let before = map.observer().snapshot();
    map.find1(&Cell::new(1)).unwrap().set(12);
    assert!(catch_unwind(AssertUnwindSafe(|| map.unbind1(&Cell::new(1)))).is_err());
    assert_eq!(map.observer().snapshot(), before);
    map.find1(&Cell::new(1)).unwrap().set(11);
    assert!(map.unbind1(&Cell::new(1)));
    assert_eq!(map.len(), 1);
}
#[test]
fn changed_primary_key_restores_second_chain_before_diagnosing_missing_partner() {
    let mut map = interior_map();
    let before = map.observer().snapshot();
    map.find2(&Cell::new(11)).unwrap().set(3);
    assert!(catch_unwind(AssertUnwindSafe(|| map.unbind2(&Cell::new(11)))).is_err());
    assert_eq!(map.observer().snapshot(), before);
    map.find2(&Cell::new(11)).unwrap().set(1);
    assert!(map.unbind2(&Cell::new(11)));
    assert_eq!(map.len(), 1);
}
