use rustycad_tkernel::ncollection::{Array1, Array2, ArrayError};
use std::{
    cell::RefCell,
    panic::{AssertUnwindSafe, catch_unwind},
    rc::Rc,
};
pub type ConstTrace = Rc<RefCell<Vec<String>>>;

// Native fixture deletes copy construction; this Rust payload similarly omits Clone.
#[derive(Default)]
pub struct CopyAssignmentOnly {
    pub value: i32,
    pub trace: Option<ConstTrace>,
}
impl CopyAssignmentOnly {
    pub fn supplied(value: i32, trace: &ConstTrace) -> Self {
        Self {
            value,
            trace: Some(Rc::clone(trace)),
        }
    }
    pub fn assign(&mut self, donor: &Self, mode: i32) {
        let previous = self.value;
        self.trace
            .as_ref()
            .unwrap()
            .borrow_mut()
            .push(format!("Q{previous}:{}", donor.value));
        assert!(
            !(mode == 1 && previous == 2),
            "mode {mode}, target {previous}: expected no early failure"
        );
        self.value = donor.value;
        assert!(
            !(mode == 2 && previous == 2),
            "mode {mode}, target {previous}: expected no late failure"
        );
    }
}
impl Drop for CopyAssignmentOnly {
    fn drop(&mut self) {
        if let Some(trace) = &self.trace {
            trace.borrow_mut().push(format!("X{}", self.value));
        }
    }
}
pub enum ConstArray<'a> {
    Linear(Array1<'a, CopyAssignmentOnly>),
    Matrix(Array2<'a, CopyAssignmentOnly>),
}
impl<'a> ConstArray<'a> {
    pub fn owned(matrix: bool, trace: &ConstTrace) -> Self {
        let mut result = if matrix {
            Self::Matrix(Array2::with_bounds(-2, -1, 3, 4).unwrap())
        } else {
            Self::Linear(Array1::with_bounds(-2, 1).unwrap())
        };
        for (index, slot) in result.values_mut().iter_mut().enumerate() {
            *slot = CopyAssignmentOnly::supplied(index as i32 + 1, trace);
        }
        result
    }
    pub fn borrowed(matrix: bool, buffer: &'a mut [CopyAssignmentOnly]) -> Self {
        if matrix {
            return Self::Matrix(Array2::from_buffer_bounds(buffer, -2, -1, 3, 4).unwrap());
        }
        Self::Linear(Array1::from_slice_bounds(buffer, -2, 1).unwrap())
    }
    pub fn values(&self) -> &[CopyAssignmentOnly] {
        match self {
            Self::Linear(array) => array.as_slice(),
            Self::Matrix(array) => array.as_slice(),
        }
    }
    fn values_mut(&mut self) -> &mut [CopyAssignmentOnly] {
        match self {
            Self::Linear(array) => array.as_mut_slice(),
            Self::Matrix(array) => array.as_mut_slice(),
        }
    }
    pub fn set(
        &mut self,
        donor: &CopyAssignmentOnly,
        mode: i32,
        invalid: i32,
    ) -> Result<(), ArrayError> {
        let assign =
            |slot: &mut CopyAssignmentOnly, source: &CopyAssignmentOnly| slot.assign(source, mode);
        let index = if invalid != 0 { 2 } else { -1 };
        let row = if invalid == 1 { 0 } else { -2 };
        let col = if invalid == 2 { 5 } else { 4 };
        match self {
            Self::Linear(array) => array.set_value_ref_with_assignment(index, donor, assign),
            Self::Matrix(array) => array.set_value_ref_with_assignment(row, col, donor, assign),
        }
    }
    pub fn init(&mut self, donor: &CopyAssignmentOnly, mode: i32) {
        let assign =
            |slot: &mut CopyAssignmentOnly, source: &CopyAssignmentOnly| slot.assign(source, mode);
        match self {
            Self::Linear(array) => array.init_ref_with_assignment(donor, assign),
            Self::Matrix(array) => array.init_ref_with_assignment(donor, assign),
        }
    }
}
pub fn exercise(
    array: &mut ConstArray<'_>,
    donor: &CopyAssignmentOnly,
    mode: i32,
    init: bool,
    invalid: i32,
) -> String {
    let address = array.values().as_ptr();
    let result = catch_unwind(AssertUnwindSafe(|| {
        if init {
            array.init(donor, mode);
            return Ok(());
        }
        array.set(donor, mode, invalid)
    }));
    let error = outcome_code(result);
    let same = address == array.values().as_ptr();
    result_row(array, donor, error, same)
}
fn result_row(
    array: &ConstArray<'_>,
    donor: &CopyAssignmentOnly,
    error: i32,
    same: bool,
) -> String {
    format!(
        "{};{};{error};{};{}",
        values_row(array.values()),
        donor.value,
        u8::from(same),
        trace_row(donor.trace.as_ref().unwrap())
    )
}
fn outcome_code(result: std::thread::Result<Result<(), ArrayError>>) -> i32 {
    match result {
        Ok(Ok(())) => 0,
        Ok(Err(_)) => 2,
        Err(_) => 1,
    }
}
pub fn values_row(values: &[CopyAssignmentOnly]) -> String {
    values
        .iter()
        .map(|item| item.value.to_string())
        .collect::<Vec<_>>()
        .join(",")
}
pub fn trace_row(trace: &ConstTrace) -> String {
    trace.borrow().join(",")
}
pub fn assert_native_row(label: &str, actual: &str) {
    let prefix = format!("{label}=");
    let expected = include_str!("../../../../tools/oracle/tkernel_array_const_assignment.expected")
        .lines()
        .find_map(|line| line.strip_prefix(&prefix))
        .unwrap();
    assert_eq!(actual, expected, "native row {label}");
}
