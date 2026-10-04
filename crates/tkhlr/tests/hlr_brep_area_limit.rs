//! Native replay of HLRBRep_AreaLimit state and handle semantics.
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::top_abs::{Orientation, State};
use rustycad_tkhlr::{hlr_algo::Intersection, hlr_brep::AreaLimit};

#[test]
fn native_area_limit_state_and_links() {
    let mut vertex =
        Intersection::with_values(Orientation::Reversed, 1, 2, 7, 0.25, 0.125, State::On);
    let states = [State::In, State::Out, State::On, State::Unknown];
    let limit = make_limit(&vertex, true, states);
    let mut output = snapshot("initial", &limit);
    vertex.set_index(99);
    let states = [State::Unknown, State::On, State::Out, State::In];
    let neighbor = make_limit(&vertex, false, states);
    finish_snapshot(&limit, &neighbor, &mut output);
    assert_eq!(
        output,
        include_str!("../../../tools/oracle/tkhlr_area_limit.expected")
    );
}

fn finish_snapshot(limit: &Handle<AreaLimit>, neighbor: &Handle<AreaLimit>, output: &mut String) {
    output.push_str(&snapshot("neighbor", neighbor));
    mutate_limit(limit, neighbor);
    output.push_str(&snapshot("changed", limit));
    output.push_str(&link_snapshot(limit, neighbor));
    limit.get().unwrap().clear();
    limit.get().unwrap().clear();
    output.push_str(&snapshot("cleared", limit));
}

fn mutate_limit(limit: &Handle<AreaLimit>, neighbor: &Handle<AreaLimit>) {
    let current = limit.get().unwrap();
    current.set_previous(neighbor);
    current.set_next(limit);
    current.set_state_before(State::Unknown);
    current.set_state_after(State::On);
    current.set_edge_before(State::Out);
    current.set_edge_after(State::In);
}

fn snapshot(label: &str, limit: &Handle<AreaLimit>) -> String {
    let limit = limit.get().unwrap();
    let (index, parameter) = (limit.vertex().index(), limit.vertex().parameter());
    let (boundary, interference) = (
        u8::from(limit.is_boundary()),
        u8::from(limit.is_interference()),
    );
    let (before, after) = (limit.state_before() as i32, limit.state_after() as i32);
    let (edge_before, edge_after) = (limit.edge_before() as i32, limit.edge_after() as i32);
    let (previous, next) = (
        u8::from(limit.previous().is_null()),
        u8::from(limit.next().is_null()),
    );
    format!(
        "{label} {index} {parameter} {boundary} {interference} {before} {after} {edge_before} {edge_after} {previous} {next}\n"
    )
}

fn link_snapshot(limit: &Handle<AreaLimit>, neighbor: &Handle<AreaLimit>) -> String {
    let previous_matches =
        limit.get().unwrap().previous().object_address() == neighbor.object_address();
    let next_matches = limit.get().unwrap().next().object_address() == limit.object_address();
    let untouched = neighbor.get().unwrap().next().is_null();
    format!(
        "links {} {} {}\n",
        u8::from(previous_matches),
        u8::from(next_matches),
        u8::from(untouched)
    )
}

fn make_limit(vertex: &Intersection, boundary: bool, states: [State; 4]) -> Handle<AreaLimit> {
    let [before, after, edge_before, edge_after] = states;
    Handle::new(AreaLimit::new(
        vertex,
        boundary,
        !boundary,
        before,
        after,
        edge_before,
        edge_after,
    ))
}
