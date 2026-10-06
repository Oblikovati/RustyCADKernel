use rustycad_tkernel::ncollection::{Array1, Array2, ArrayError};
use std::{
    cell::RefCell,
    panic::{AssertUnwindSafe, catch_unwind},
    rc::Rc,
};
pub type InternalInitTrace = Rc<RefCell<Vec<String>>>;
#[derive(Default)]
pub struct InternalDonorItem {
    pub value: i32,
    pub trace: Option<InternalInitTrace>,
}
impl InternalDonorItem {
    pub fn supplied(value: i32, trace: &InternalInitTrace) -> Self {
        Self {
            value,
            trace: Some(Rc::clone(trace)),
        }
    }
    pub fn assign(&mut self, donor: &Self, mode: i32) {
        let old = self.value;
        self.trace
            .as_ref()
            .unwrap()
            .borrow_mut()
            .push(format!("Q{old}:{}", donor.value));
        assert!(
            !(mode == 1 && old == 2 || mode == 4 && old == 4),
            "mode {mode}, target {old}: expected no early failure"
        );
        self.value = donor.value;
        assert!(
            !(mode == 5 && old == 4),
            "mode {mode}, target {old}: expected no late failure"
        );
    }
    pub fn assign_self(&mut self, mode: i32) {
        self.trace
            .as_ref()
            .unwrap()
            .borrow_mut()
            .push(format!("S{}", self.value));
        assert_ne!(mode, 2, "self mode {mode}: expected no early failure");
        self.value += 100;
        assert_ne!(mode, 3, "self mode {mode}: expected no late failure");
    }
}
impl Drop for InternalDonorItem {
    fn drop(&mut self) {
        if let Some(trace) = &self.trace {
            trace.borrow_mut().push(format!("X{}", self.value));
        }
    }
}
pub enum InternalInitArray<'a> {
    Linear(Array1<'a, InternalDonorItem>),
    Matrix(Array2<'a, InternalDonorItem>),
}
impl<'a> InternalInitArray<'a> {
    pub fn owned(matrix: bool, trace: &InternalInitTrace) -> Self {
        let mut result = if matrix {
            Self::Matrix(Array2::with_bounds(-2, -1, 3, 5).unwrap())
        } else {
            Self::Linear(Array1::with_bounds(-2, 3).unwrap())
        };
        for (index, slot) in result.values_mut().iter_mut().enumerate() {
            *slot = InternalDonorItem::supplied(index as i32 + 1, trace);
        }
        result
    }
    pub fn borrowed(matrix: bool, buffer: &'a mut [InternalDonorItem]) -> Self {
        if matrix {
            return Self::Matrix(Array2::from_buffer_bounds(buffer, -2, -1, 3, 5).unwrap());
        }
        Self::Linear(Array1::from_slice_bounds(buffer, -2, 3).unwrap())
    }
    pub fn values(&self) -> &[InternalDonorItem] {
        match self {
            Self::Linear(array) => array.as_slice(),
            Self::Matrix(array) => array.as_slice(),
        }
    }
    fn values_mut(&mut self) -> &mut [InternalDonorItem] {
        match self {
            Self::Linear(array) => array.as_mut_slice(),
            Self::Matrix(array) => array.as_mut_slice(),
        }
    }
    pub fn initialize(&mut self, offset: i32, mode: i32, invalid: i32) -> Result<(), ArrayError> {
        let assign =
            |target: &mut InternalDonorItem, donor: &InternalDonorItem| target.assign(donor, mode);
        let self_assign = |slot: &mut InternalDonorItem| slot.assign_self(mode);
        let index = if invalid != 0 { 4 } else { -2 + offset };
        let row = if invalid == 1 { 0 } else { -2 + offset / 3 };
        let col = if invalid == 2 { 6 } else { 3 + offset % 3 };
        match self {
            Self::Linear(array) => {
                array.init_from_index_with_assignment(index, assign, self_assign)
            }
            Self::Matrix(array) => {
                array.init_from_index_with_assignment(row, col, assign, self_assign)
            }
        }
    }
    pub fn bounds(&self) -> String {
        match self {
            Self::Linear(array) => format!("{}:{}", array.lower(), array.upper()),
            Self::Matrix(array) => format!(
                "{}:{}:{}:{}:{}:{}",
                array.lower_row(),
                array.upper_row(),
                array.lower_col(),
                array.upper_col(),
                array.array1().lower(),
                array.array1().upper()
            ),
        }
    }
}
pub fn exercise(
    array: &mut InternalInitArray<'_>,
    offset: i32,
    mode: i32,
    invalid: i32,
    trace: &InternalInitTrace,
) -> String {
    let pointer = array.values().as_ptr();
    let result = catch_unwind(AssertUnwindSafe(|| array.initialize(offset, mode, invalid)));
    let error = init_outcome_code(result);
    format!(
        "{};{};{error};{};{}",
        array.bounds(),
        values_row(array.values()),
        u8::from(pointer == array.values().as_ptr()),
        trace.borrow().join(",")
    )
}
fn init_outcome_code(result: std::thread::Result<Result<(), ArrayError>>) -> i32 {
    match result {
        Ok(Ok(())) => 0,
        Ok(Err(_)) => 2,
        Err(_) => 1,
    }
}

pub fn values_row(values: &[InternalDonorItem]) -> String {
    values
        .iter()
        .map(|item| item.value.to_string())
        .collect::<Vec<_>>()
        .join(",")
}
pub fn assert_native_row(label: &str, actual: &str) {
    let prefix = format!("{label}=");
    let expected = include_str!("../../../../tools/oracle/tkernel_array_internal_init.expected")
        .lines()
        .find_map(|line| line.strip_prefix(&prefix))
        .unwrap();
    assert_eq!(actual, expected, "native row {label}");
}
