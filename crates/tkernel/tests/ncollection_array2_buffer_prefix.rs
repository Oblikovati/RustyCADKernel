//! Checks Array2 prefix-view and fill behavior against the pinned native probe.
use rustycad_tkernel::ncollection::{Array2, ArrayError};
use std::cell::Cell;
use std::rc::Rc;

struct CopyProbe {
    value: i32,
    assignments: Rc<Cell<usize>>,
    constructions: Rc<Cell<usize>>,
}

impl Clone for CopyProbe {
    fn clone(&self) -> Self {
        self.constructions.set(self.constructions.get() + 1);
        Self {
            value: self.value,
            assignments: Rc::clone(&self.assignments),
            constructions: Rc::clone(&self.constructions),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.assignments.set(self.assignments.get() + 1);
        self.value = source.value;
        self.assignments = Rc::clone(&source.assignments);
        self.constructions = Rc::clone(&source.constructions);
    }
}

fn probe_buffer(assignments: &Rc<Cell<usize>>, constructions: &Rc<Cell<usize>>) -> [CopyProbe; 6] {
    std::array::from_fn(|index| CopyProbe {
        value: 100 + index as i32,
        assignments: Rc::clone(assignments),
        constructions: Rc::clone(constructions),
    })
}

fn initialize_prefix(
    buffer: &mut [CopyProbe; 6],
    assignments: &Rc<Cell<usize>>,
    constructions: &Rc<Cell<usize>>,
) -> String {
    let mut array = Array2::from_buffer(buffer, 2, 2).unwrap();
    let mut fill = CopyProbe {
        value: 7,
        assignments: Rc::clone(assignments),
        constructions: Rc::clone(constructions),
    };
    let (assignment_count, construction_count) =
        borrow_and_fill(&mut array, &mut fill, assignments, constructions);
    format_prefix_rows(&array, assignment_count, construction_count)
}

fn borrow_and_fill(
    array: &mut Array2<'_, CopyProbe>,
    fill: &mut CopyProbe,
    assignments: &Rc<Cell<usize>>,
    constructions: &Rc<Cell<usize>>,
) -> (usize, usize) {
    array.init_ref(fill);
    let counts = (assignments.get(), constructions.get());
    fill.value = 99;
    array.change_at(0, 0).unwrap().value = 20;
    counts
}

fn format_prefix_rows(
    array: &Array2<'_, CopyProbe>,
    assignment_count: usize,
    construction_count: usize,
) -> String {
    format!(
        "init_assignments={assignment_count}\ncopy_constructions={construction_count}\nvalues={}\n",
        rectangle_values(array)
    )
}

fn rectangle_values(array: &Array2<'_, CopyProbe>) -> String {
    array
        .iter()
        .map(|probe| probe.value.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

#[test]
fn mutable_buffer_wraps_only_required_prefix_and_init_fills_that_rectangle() {
    let assignments = Rc::new(Cell::new(0));
    let constructions = Rc::new(Cell::new(0));
    let mut buffer = probe_buffer(&assignments, &constructions);
    let rows = initialize_prefix(&mut buffer, &assignments, &constructions);
    let empty_size = empty_prefix_size();
    let actual = format!(
        "{rows}tail={},{}\nempty_size={empty_size}\n",
        buffer[4].value, buffer[5].value
    );
    assert_eq!(
        actual,
        include_str!("fixtures/ncollection_array2_buffer_prefix.expected")
    );
}

fn empty_prefix_size() -> usize {
    let mut buffer: [i32; 0] = [];
    Array2::from_buffer(&mut buffer, 0, 5).unwrap().len()
}

#[test]
fn zero_rows_accept_an_empty_mutable_buffer() {
    assert_eq!(empty_prefix_size(), 0);
}

#[test]
fn empty_and_short_buffer_shapes_are_checked() {
    let mut buffer = [0; 3];
    assert_eq!(
        Array2::from_buffer(&mut buffer, 2, 2).err(),
        Some(ArrayError::DimensionMismatch {
            expected: 4,
            actual: 3,
        })
    );
}
