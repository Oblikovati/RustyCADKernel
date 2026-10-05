//! Pinned comparison dispatch and early-exit behavior for NCollection vectors.
use rustycad_tkernel::ncollection::{Vec2, Vec3, Vec4};
use std::cell::RefCell;

#[derive(Clone, Copy)]
struct Probe<'a> {
    value: i32,
    events: &'a RefCell<Vec<String>>,
}

fn probe(value: i32, events: &RefCell<Vec<String>>) -> Probe<'_> {
    Probe { value, events }
}

impl PartialEq for Probe<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.events
            .borrow_mut()
            .push(format!("eq({},{})", self.value, other.value));
        self.value == other.value
    }

    #[allow(clippy::partialeq_ne_impl)] // This counter detects forbidden element-ne dispatch.
    fn ne(&self, other: &Self) -> bool {
        self.events
            .borrow_mut()
            .push(format!("ne({},{})", self.value, other.value));
        self.value != other.value
    }
}

fn comparison_row(name: &str, result: bool, events: &RefCell<Vec<String>>) -> String {
    format!(
        "{name}={} trace={}",
        result as u8,
        events.borrow().join(",")
    )
}

#[test]
fn vector_comparisons_match_pinned_native_dispatch_order() {
    let actual = [vec2_eq_row(), vec3_ne_row(), vec4_is_equal_row()].join("\n") + "\n";
    assert_eq!(
        actual,
        include_str!("fixtures/ncollection_vector_comparison_dispatch.expected")
    );
}

fn vec2_eq_row() -> String {
    let two_events = RefCell::new(Vec::new());
    let two_left = Vec2::new(probe(1, &two_events), probe(2, &two_events));
    let two_right = Vec2::new(probe(1, &two_events), probe(9, &two_events));
    comparison_row("vec2_eq", two_left == two_right, &two_events)
}

fn vec3_ne_row() -> String {
    let three_events = RefCell::new(Vec::new());
    let three_left = Vec3::new(
        probe(1, &three_events),
        probe(2, &three_events),
        probe(3, &three_events),
    );
    let three_right = Vec3::new(
        probe(9, &three_events),
        probe(2, &three_events),
        probe(3, &three_events),
    );
    comparison_row("vec3_ne", three_left != three_right, &three_events)
}

fn vec4_is_equal_row() -> String {
    let four_events = RefCell::new(Vec::new());
    let four_left = Vec4::from_array([1, 2, 3, 4].map(|value| probe(value, &four_events)));
    let four_right = Vec4::from_array([1, 2, 3, 9].map(|value| probe(value, &four_events)));
    comparison_row(
        "vec4_is_equal",
        four_left.is_equal(&four_right),
        &four_events,
    )
}
