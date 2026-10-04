//! Supplemental regressions for Poly_Connect, the coherent triangulation, node merging and
//! loop building.
use rustycad_tkernel::{ncollection::List, standard::Handle};
use rustycad_tkmath::{
    gp::{Direction2, Point3, Transform3, Xy, Xyz},
    poly::{
        CoherentLink, CoherentTriPtr, CoherentTriangle, CoherentTriangulation, Connect,
        MergeNodesTool, Triangle, Triangulation, TwoIntegers,
        make_loops::{
            HeapOfInteger, LeftWay2D, Link, LinkFlag, ListOfLink, MakeLoops, MakeLoops2DHelper,
            MakeLoopsHelper, result_code,
        },
    },
};
use std::collections::HashMap;
/// Four triangles around node 1 (a closed fan) plus a far triangle on nodes 6..8.
fn fan() -> Handle<Triangulation> {
    let points = [
        (0.0, 0.0),
        (1.0, 0.0),
        (0.0, 1.0),
        (-1.0, 0.0),
        (0.0, -1.0),
        (5.0, 5.0),
        (6.0, 5.0),
        (5.0, 6.0),
    ];
    let mesh = Triangulation::with_sizes(8, 5, false, false).unwrap();
    for (index, (x, y)) in points.into_iter().enumerate() {
        mesh.set_node(index as i32 + 1, Point3::new(x, y, 0.0))
            .unwrap();
    }
    for (index, (a, b, c)) in [(1, 2, 3), (1, 3, 4), (1, 4, 5), (1, 5, 2), (6, 7, 8)]
        .into_iter()
        .enumerate()
    {
        mesh.set_triangle(index as i32 + 1, Triangle::new(a, b, c))
            .unwrap();
    }
    Handle::new(mesh)
}
#[test]
fn connect_turns_around_a_node_and_reports_neighbours() {
    let mesh = fan();
    let mut connect = Connect::new(&mesh).unwrap();
    // Sides run from each node to the next: (1,2), (2,3), (3,1).
    assert_eq!(connect.triangles(1).unwrap(), (4, 0, 2));
    assert_eq!(connect.nodes(1).unwrap(), (5, 0, 4));
    connect.initialize(1).unwrap();
    let mut around = Vec::new();
    while connect.more() {
        around.push(connect.value());
        connect.next().unwrap();
    }
    // Starts from the last triangle that wrote the node, then walks the direct sense.
    assert_eq!(around, [4, 1, 2, 3]);
    assert_eq!(
        connect.triangle(9).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
    assert!(Connect::default().triangulation().is_null());
}
#[test]
fn coherent_structure_links_neighbours_and_exports() {
    let mut mesh = CoherentTriangulation::from_triangulation(&fan()).unwrap();
    assert_eq!((mesh.n_nodes(), mesh.n_triangles()), (8, 5));
    assert_eq!(mesh.triangle(0).unwrap().n_connections(), 2);
    assert_eq!(mesh.compute_links().unwrap(), 11);
    let (found, sides) = mesh.find_triangle(&CoherentLink::from_nodes(0, 1)).unwrap();
    assert!(found);
    assert_eq!(sides, [Some(0), Some(3)]);
    assert!(mesh.remove_triangle(4).unwrap());
    let mut free = List::default();
    assert!(mesh.get_free_nodes(&mut free));
    assert_eq!(free.iter().copied().collect::<Vec<_>>(), [5, 6, 7]);
    assert_eq!(mesh.get_triangulation().unwrap().nb_nodes(), 5);
    assert_eq!(
        mesh.clone_triangulation().unwrap().unwrap().n_triangles(),
        4
    );
}
#[test]
fn degenerate_sides_collapse_into_the_remaining_node() {
    let mesh = Triangulation::with_sizes(4, 2, false, false).unwrap();
    let points = [(0.0, 0.0), (1.0, 0.0), (1.0, 1e-9), (0.0, 1.0)];
    for (index, (x, y)) in points.into_iter().enumerate() {
        mesh.set_node(index as i32 + 1, Point3::new(x, y, 0.0))
            .unwrap();
    }
    mesh.set_triangle(1, Triangle::new(1, 2, 3)).unwrap();
    mesh.set_triangle(2, Triangle::new(1, 3, 4)).unwrap();
    let mut coherent = CoherentTriangulation::from_triangulation(&Handle::new(mesh)).unwrap();
    let mut removed = List::default();
    assert!(
        coherent
            .remove_degenerated(1e-6, Some(&mut removed))
            .unwrap()
    );
    assert_eq!(
        removed.iter().copied().collect::<Vec<_>>(),
        [TwoIntegers::new(1, 2)]
    );
    assert_eq!(coherent.n_triangles(), 1);
    assert!(coherent.triangle(0).unwrap().is_empty());
    assert_eq!(coherent.triangle(1).unwrap().node(1), 2);
}
#[test]
fn triangle_connections_and_rings_follow_native_rules() {
    let mut triangles = vec![
        CoherentTriangle::new(0, 1, 2),
        CoherentTriangle::new(2, 1, 3),
    ];
    assert!(CoherentTriangle::set_connection_any(&mut triangles, 0, 1).unwrap());
    assert_eq!(triangles[0].find_connection(1), 0);
    assert_eq!(triangles[1].get_connected_node(2), 0);
    assert!(CoherentTriangle::remove_connection_with(&mut triangles, 1, 0).unwrap());
    assert_eq!(triangles[0].n_connections(), 0);
    assert!(CoherentLink::from_triangle(&triangles[0], 3).is_err());
    let mut ring = CoherentTriPtr::new(7);
    ring.prepend(0, 8);
    ring.append(0, 9);
    assert_eq!(ring.iter().collect::<Vec<_>>(), [7, 9, 8]);
    assert_eq!(ring.iter_from(1).collect::<Vec<_>>(), [9, 8, 7]);
    ring.remove(0);
    assert_eq!(ring.get_triangle(), 9);
}
#[test]
fn ring_member_tokens_survive_edits_and_allocator_mismatch_is_rejected() {
    let arena = rustycad_tkernel::ncollection::BaseAllocator::incremental(4096);
    let wrong = rustycad_tkernel::ncollection::BaseAllocator::incremental(4096);
    let mut ring = CoherentTriPtr::new_in(1, arena.clone());
    let head = ring.head_member().unwrap();
    let second = ring.append_member(head, 2, arena.clone()).unwrap();
    let third = ring.prepend_member(head, 3, arena.clone()).unwrap();
    assert_eq!(ring.next_member(head).unwrap(), second);
    assert_eq!(ring.iter().collect::<Vec<_>>(), [1, 2, 3]);
    assert_eq!(ring.triangle(second).unwrap(), 2);
    assert!(ring.remove_member(third, &wrong).is_err());
    ring.remove_member(third, &arena).unwrap();
    assert!(ring.triangle(third).is_err());
    assert_eq!(
        ring.remove_member(third, &arena)
            .unwrap_err()
            .exception_type(),
        "Standard_NoSuchObject"
    );
    assert_eq!(ring.triangle(second).unwrap(), 2);
    assert_eq!(ring.previous_member(head).unwrap(), second);
    ring.set_member_triangle(second, 20).unwrap();
    assert_eq!(ring.iter_from_member(second).member_token(), Some(second));
    assert_eq!(ring.triangle(second).unwrap(), 20);
    ring.remove_member(second, &arena).unwrap();
    ring.remove_list_in(&arena).unwrap();
    assert!(ring.is_empty());
}
#[test]
fn coherent_ring_clone_has_independent_allocator_owned_members() {
    let mut ring = CoherentTriPtr::new(4);
    ring.append(0, 5);
    let mut clone = ring.clone();
    assert_ne!(ring.head_member(), clone.head_member());
    clone.set_triangle(9);
    assert_eq!(ring.get_triangle(), 4);
    assert_eq!(clone.get_triangle(), 9);
    let arena = rustycad_tkernel::ncollection::BaseAllocator::common();
    ring.remove_list_in(&arena).unwrap();
    assert_eq!(clone.iter().collect::<Vec<_>>(), [9, 5]);
    let empty = ring.clone();
    assert!(empty.is_empty());
}
#[test]
fn merge_tool_merges_by_distance_and_counts_quads_like_native() {
    let mut tool = MergeNodesTool::new(0.0, 0.01, -1);
    let a = [
        Xyz::new(0.0, 0.0, 0.0),
        Xyz::new(1.0, 0.0, 0.0),
        Xyz::new(0.0, 1.0, 0.0),
    ];
    let b = [
        Xyz::new(0.001, 0.0, 0.0),
        Xyz::new(1.0, 0.0, 0.0),
        Xyz::new(0.0, 1.0, 0.0),
    ];
    tool.add_triangle(&a).unwrap();
    tool.add_triangle(&b).unwrap();
    assert_eq!((tool.nb_nodes(), tool.nb_elements()), (3, 2));
    let result = tool.result().unwrap();
    assert_eq!(result.triangle(2).unwrap(), Triangle::new(1, 2, 3));
    let mut flipped = MergeNodesTool::new(0.0, 0.01, -1);
    flipped.add_triangle(&a).unwrap();
    flipped.add_triangle(&[b[0], b[2], b[1]]).unwrap();
    // Opposite normals keep the nodes apart unless merging opposite normals is enabled.
    assert_eq!(flipped.nb_nodes(), 6);
    let mut map = MergeNodesTool::new(0.0, 0.0, -1);
    *map.change_output() = Handle::default();
    map.add_quad(&[a[0], a[1], a[2], b[0]]).unwrap();
    // Without an output a quad counts as one element.
    assert_eq!(map.nb_elements(), 1);
    assert_eq!(
        map.add_element(&a[..2]).unwrap_err().exception_type(),
        "Standard_ProgramError"
    );
    let identity = Transform3::default();
    let merged = MergeNodesTool::merge_nodes(&fan(), identity, false, 0.5, 0.0, false).unwrap();
    assert!(merged.is_null());
}
/// A named fake helper over straight links between 2D points.
struct FakeSegmentHelper {
    points: Vec<Xy>,
    adjacent: HashMap<i32, ListOfLink>,
    none: ListOfLink,
}
impl FakeSegmentHelper {
    fn new(points: Vec<Xy>, links: &[Link]) -> Self {
        let mut adjacent: HashMap<i32, ListOfLink> = HashMap::new();
        for link in links {
            adjacent.entry(link.node1).or_default().append(*link);
            adjacent.entry(link.node2).or_default().append(*link);
        }
        Self {
            points,
            adjacent,
            none: ListOfLink::default(),
        }
    }
}
impl MakeLoopsHelper for FakeSegmentHelper {
    fn get_adjacent_links(&self, node: i32) -> &ListOfLink {
        self.adjacent.get(&node).unwrap_or(&self.none)
    }
}
impl MakeLoops2DHelper for FakeSegmentHelper {
    fn get_first_tangent(&self, link: &Link) -> Option<Direction2> {
        let d = self.points[link.node2 as usize - 1] - self.points[link.node1 as usize - 1];
        let d = if link.is_reversed() { -d } else { d };
        Direction2::new(d.x(), d.y()).ok()
    }
    fn get_last_tangent(&self, link: &Link) -> Option<Direction2> {
        self.get_first_tangent(link)
    }
}
#[test]
fn loops_close_squares_and_mark_hanging_links() {
    let points = vec![
        Xy::new(0.0, 0.0),
        Xy::new(1.0, 0.0),
        Xy::new(1.0, 1.0),
        Xy::new(0.0, 1.0),
        Xy::new(2.0, 2.0),
    ];
    let links = [
        Link::new(1, 2),
        Link::new(2, 3),
        Link::new(3, 4),
        Link::new(4, 1),
        Link::new(3, 5),
    ];
    let helper = FakeSegmentHelper::new(points, &links);
    let mut algo = MakeLoops::new(LeftWay2D::new(true, &helper));
    for link in &links {
        algo.add_link(link).unwrap();
    }
    assert_eq!(
        algo.perform().unwrap(),
        result_code::LOOPS_DONE | result_code::HANGING_LINKS
    );
    assert_eq!(algo.get_nb_loops(), 1);
    assert_eq!(algo.get_loop(1).unwrap().len(), 4);
    assert_eq!(algo.get_nb_hanging(), 1);
    assert_eq!(
        algo.set_link_orientation(&Link::new(5, 3), LinkFlag::None)
            .unwrap(),
        LinkFlag::Fwd
    );
    assert!(algo.find_link(&Link::new(9, 8)).is_null());
}
#[test]
fn heap_top_follows_the_packed_map_cursor_across_removals() {
    let mut heap = HeapOfInteger::new(1);
    assert_eq!(
        heap.top().unwrap_err().exception_type(),
        "Standard_NoSuchObject"
    );
    for value in [3, 1, 2] {
        heap.add(value);
    }
    let top = heap.top().unwrap();
    heap.remove(top);
    assert!(!heap.contains(top));
    assert_ne!(heap.top().unwrap(), top);
    heap.clear();
    assert!(heap.is_empty());
}
#[test]
fn perform_refuses_a_start_link_that_consumes_nothing() {
    // After ReplaceLink swaps nodes the helper still lists the old order; natively the
    // main loop then repeats one step forever.
    let points: Vec<Xy> = (0..5)
        .map(|k| {
            let angle = f64::from(k) * std::f64::consts::TAU / 5.0;
            Xy::new(angle.cos(), angle.sin())
        })
        .collect();
    let flags = [0, 3, 1, 3, 1];
    let links: Vec<Link> = (0..5)
        .map(|k| {
            let mut link = Link::new(k + 1, (k + 1) % 5 + 1);
            link.flags = flags[k as usize];
            link
        })
        .collect();
    let helper = FakeSegmentHelper::new(points, &links);
    let mut algo = MakeLoops::new(LeftWay2D::new(true, &helper));
    for link in &links {
        algo.add_link(link).unwrap();
    }
    let mut swapped = Link::new(1, 5);
    swapped.flags = 1;
    algo.replace_link(&links[4], &swapped).unwrap();
    assert_eq!(
        algo.perform().unwrap_err().exception_type(),
        "Standard_ProgramError"
    );
}
