use rustycad_tkernel::ncollection::{Array1, Array2, ArrayError};
use std::{
    cell::RefCell,
    panic::{AssertUnwindSafe, catch_unwind},
    rc::Rc,
};
pub type AssignmentTrace = Rc<RefCell<Vec<String>>>;

// No Clone: this probe must not gain a copy-construction requirement from assignment.
#[derive(Default)]
pub struct AssignmentItem {
    pub value: i32,
    pub trace: Option<AssignmentTrace>,
}
impl AssignmentItem {
    pub fn supplied(value: i32, trace: &AssignmentTrace) -> Self {
        Self {
            value,
            trace: Some(Rc::clone(trace)),
        }
    }
    pub fn assign(&mut self, donor: &mut Self, mode: i32) {
        let kind = if mode == 3 { "Q" } else { "A" };
        self.trace
            .as_ref()
            .unwrap()
            .borrow_mut()
            .push(format!("{kind}{}:{}", self.value, donor.value));
        assert_ne!(mode, 1, "assignment mode {mode}: expected no early failure");
        self.value = donor.value;
        if mode != 3 {
            donor.value = -donor.value;
        }
        assert_ne!(mode, 2, "assignment mode {mode}: expected no late failure");
    }
}
impl Drop for AssignmentItem {
    fn drop(&mut self) {
        if let Some(trace) = &self.trace {
            trace.borrow_mut().push(format!("X{}", self.value));
        }
    }
}

pub enum AssignmentArray<'a> {
    Linear(Array1<'a, AssignmentItem>),
    Matrix(Array2<'a, AssignmentItem>),
}
impl<'a> AssignmentArray<'a> {
    pub fn owned(matrix: bool, trace: &AssignmentTrace) -> Self {
        let mut result = if matrix {
            Self::Matrix(Array2::with_bounds(-2, -1, 3, 4).unwrap())
        } else {
            Self::Linear(Array1::with_bounds(-2, 1).unwrap())
        };
        for (index, slot) in result.values_mut().iter_mut().enumerate() {
            *slot = AssignmentItem::supplied(index as i32 + 1, trace);
        }
        result
    }
    pub fn borrowed(matrix: bool, buffer: &'a mut [AssignmentItem]) -> Self {
        if matrix {
            return Self::Matrix(Array2::from_buffer_bounds(buffer, -2, -1, 3, 4).unwrap());
        }
        Self::Linear(Array1::from_slice_bounds(buffer, -2, 1).unwrap())
    }
    pub fn values(&self) -> &[AssignmentItem] {
        match self {
            Self::Linear(array) => array.as_slice(),
            Self::Matrix(array) => array.as_slice(),
        }
    }
    fn values_mut(&mut self) -> &mut [AssignmentItem] {
        match self {
            Self::Linear(array) => array.as_mut_slice(),
            Self::Matrix(array) => array.as_mut_slice(),
        }
    }
    pub fn assign(
        &mut self,
        donor: &mut AssignmentItem,
        mode: i32,
        invalid: i32,
    ) -> Result<(), ArrayError> {
        let assign =
            |slot: &mut AssignmentItem, source: &mut AssignmentItem| slot.assign(source, mode);
        let (index, row, col) = fixture_coordinates(invalid);
        match self {
            Self::Linear(array) => array.set_value_with_assignment(index, donor, assign),
            Self::Matrix(array) => array.set_value_with_assignment(row, col, donor, assign),
        }
    }
}

fn fixture_coordinates(invalid: i32) -> (i32, i32, i32) {
    match invalid {
        1 => (-3, -3, 4),
        2 => (2, 0, 4),
        3 => (-1, -2, 2),
        4 => (-1, -2, 5),
        _ => (-1, -2, 4),
    }
}
fn outcome_code(result: std::thread::Result<Result<(), ArrayError>>) -> i32 {
    match result {
        Ok(Ok(())) => 0,
        Ok(Err(_)) => 2,
        Err(_) => 1,
    }
}

pub fn exercise(
    array: &mut AssignmentArray<'_>,
    donor: &mut AssignmentItem,
    mode: i32,
    invalid: i32,
) -> String {
    let address = std::ptr::from_ref(&array.values()[1]);
    let result = catch_unwind(AssertUnwindSafe(|| array.assign(donor, mode, invalid)));
    let error = outcome_code(result);
    let same = address == std::ptr::from_ref(&array.values()[1]);
    format!(
        "{};{};{error};{};{}",
        values_row(array.values()),
        donor.value,
        u8::from(same),
        trace_row(donor.trace.as_ref().unwrap())
    )
}
pub fn values_row(values: &[AssignmentItem]) -> String {
    values
        .iter()
        .map(|item| item.value.to_string())
        .collect::<Vec<_>>()
        .join(",")
}
pub fn trace_row(trace: &AssignmentTrace) -> String {
    trace.borrow().join(",")
}
pub fn assert_native_row(label: &str, actual: &str) {
    let prefix = format!("{label}=");
    let expected =
        include_str!("../../../../tools/oracle/tkernel_array_set_value_assignment.expected")
            .lines()
            .find_map(|line| line.strip_prefix(&prefix))
            .unwrap();
    assert_eq!(actual, expected, "native row {label}");
}
