//! Protected-helper behavior for the Poly loop algorithm.

use rustycad_tkmath::{
    gp::Direction2,
    poly::make_loops::{
        LeftWay2D, LeftWay3D, Link, LinkFlag, LinkHasher, LinkTable, ListOfLink, LoopWay,
        MakeLoops, MakeLoops2DHelper, MakeLoops3DHelper, MakeLoopsHelper, get_link,
    },
};

#[derive(Default)]
struct EmptyHelper {
    adjacent: ListOfLink,
}

impl MakeLoopsHelper for EmptyHelper {
    fn get_adjacent_links(&self, _node: i32) -> &ListOfLink {
        &self.adjacent
    }
}

impl MakeLoops2DHelper for EmptyHelper {
    fn get_first_tangent(&self, _link: &Link) -> Option<Direction2> {
        None
    }

    fn get_last_tangent(&self, _link: &Link) -> Option<Direction2> {
        None
    }
}

impl MakeLoops3DHelper for EmptyHelper {
    fn get_first_tangent(&self, _link: &Link) -> Option<rustycad_tkmath::gp::Direction3> {
        None
    }

    fn get_last_tangent(&self, _link: &Link) -> Option<rustycad_tkmath::gp::Direction3> {
        None
    }

    fn get_normal(&self, _node: i32) -> Option<rustycad_tkmath::gp::Direction3> {
        None
    }
}

fn link_table() -> LinkTable {
    let mut links = LinkTable::with_hasher(LinkHasher, 10);
    links.add(Link::new(7, 9));
    links
}

#[test]
fn signed_get_link_matches_native_reversal() {
    let links = link_table();
    assert_link_fields(get_link(&links, 1).unwrap(), 1);
    assert_link_fields(get_link(&links, -1).unwrap(), 5);
}

fn assert_link_fields(link: Link, flags: i32) {
    assert_eq!((link.node1, link.node2, link.flags), (7, 9, flags));
    assert_eq!(
        link.is_reversed(),
        flags == (LinkFlag::Fwd as i32 | LinkFlag::Reversed as i32)
    );
}

#[test]
fn missing_get_link_indices_raise_native_range_error() {
    let links = link_table();
    for index in [0, 2, -2] {
        assert_range_error(get_link(&links, index).unwrap_err());
    }
}

fn assert_range_error(failure: rustycad_tkernel::standard::Failure) {
    assert_eq!(failure.exception_type(), "Standard_OutOfRange");
}

#[test]
fn builder_get_link_exposes_checked_protected_helper_behavior() {
    let helper = EmptyHelper::default();
    let mut builder = MakeLoops::new(LeftWay2D::new(true, &helper));
    builder.add_link(&Link::new(7, 9)).unwrap();
    assert_link_fields(builder.get_link(-1).unwrap(), 5);
    assert_range_error(builder.get_link(2).unwrap_err());
}

#[test]
fn empty_choose_left_way_candidates_raise_native_no_such_object() {
    let helper = EmptyHelper::default();
    let links = link_table();
    assert_no_candidate_error(LeftWay2D::new(true, &helper).choose_left_way(&links, 7, 1, &[]));
    assert_no_candidate_error(LeftWay3D::new(&helper).choose_left_way(&links, 7, 1, &[]));
}

fn assert_no_candidate_error(result: Result<i32, rustycad_tkernel::standard::Failure>) {
    let failure = result.unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_NoSuchObject");
    assert_eq!(failure.message(), "NCollection_List::First");
}
