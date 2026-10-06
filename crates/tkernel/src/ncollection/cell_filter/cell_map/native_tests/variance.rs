use super::super::{CellIndex, CellMap};
use crate::ncollection::BaseAllocator;
fn shorten<'a>(_: &'a (), cells: CellMap<&'static str>) -> CellMap<&'a str> {
    cells
}
#[test]
fn private_cell_owner_is_covariant_in_borrowed_targets() {
    // CellFilter's geometry projection I::CellTarget already makes I invariant; the underlying
    // owning node graph should nevertheless preserve the ordinary payload covariance.
    let anchor = ();
    let mut cells = CellMap::new(BaseAllocator::incremental(1024));
    let index = CellIndex::from_slice(&[0]);
    cells.add(&index, &"target");
    let mut shortened = shorten(&anchor, cells);
    shortened
        .try_retain(&index, |target| {
            assert_eq!(*target, "target");
            Ok::<_, ()>(false)
        })
        .unwrap();
}
