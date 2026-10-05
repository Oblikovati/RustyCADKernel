//! Checks Array2 prefix-view and fill behavior against the pinned native probe.
use rustycad_tkernel::ncollection::Array2;
use std::cell::Cell;
use std::rc::Rc;

struct CopyProbe {
    value: i32,
    assignments: Rc<Cell<usize>>,
}

impl Clone for CopyProbe {
    fn clone(&self) -> Self {
        self.assignments.set(self.assignments.get() + 1);
        Self {
            value: self.value,
            assignments: Rc::clone(&self.assignments),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.assignments.set(self.assignments.get() + 1);
        self.value = source.value;
        self.assignments = Rc::clone(&source.assignments);
    }
}

fn probe_buffer(assignments: &Rc<Cell<usize>>) -> [CopyProbe; 6] {
    std::array::from_fn(|index| CopyProbe {
        value: 100 + index as i32,
        assignments: Rc::clone(assignments),
    })
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
    let mut buffer = probe_buffer(&assignments);
    let rows = {
        let mut array = Array2::from_buffer(&mut buffer, 2, 2).unwrap();
        array.init(CopyProbe {
            value: 7,
            assignments: Rc::clone(&assignments),
        });
        let init_assignments = assignments.get();
        array.change_at(0, 0).unwrap().value = 20;
        format!(
            "init_assignments={init_assignments}\nvalues={}\n",
            rectangle_values(&array)
        )
    };
    let actual = format!("{rows}tail={},{}\n", buffer[4].value, buffer[5].value);
    assert_eq!(
        actual,
        include_str!("fixtures/ncollection_array2_buffer_prefix.expected")
    );
}
