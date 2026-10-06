use rustycad_tkernel::ncollection::{Array1, Array2, ArrayError};
use std::{
    cell::RefCell,
    panic::{AssertUnwindSafe, catch_unwind},
    rc::Rc,
};
pub type InternalSetTrace = Rc<RefCell<Vec<String>>>;
#[derive(Default)]
pub struct InternalSetItem {
    pub value: i32,
    pub trace: Option<InternalSetTrace>,
}
impl InternalSetItem {
    pub fn supplied(value: i32, trace: &InternalSetTrace) -> Self {
        Self {
            value,
            trace: Some(Rc::clone(trace)),
        }
    }
    fn record(&self, event: String, mode: i32) {
        self.trace.as_ref().unwrap().borrow_mut().push(event);
        assert_ne!(mode, 1, "assignment mode {mode}: expected no early failure");
    }
    pub fn assign_copy(&mut self, donor: &Self, mode: i32) {
        self.record(format!("C{}:{}", self.value, donor.value), mode);
        self.value = donor.value;
        assert_ne!(mode, 2, "assignment mode {mode}: expected no late failure");
    }
    pub fn assign_move(&mut self, donor: &mut Self, mode: i32) {
        self.record(format!("M{}:{}", self.value, donor.value), mode);
        self.value = donor.value;
        donor.value = -donor.value;
        assert_ne!(mode, 2, "assignment mode {mode}: expected no late failure");
    }
    pub fn assign_self(&mut self, moved: bool, mode: i32) {
        self.record(
            format!("{}S{}", if moved { "M" } else { "C" }, self.value),
            mode,
        );
        self.value = if moved {
            -self.value * 10
        } else {
            self.value + 100
        };
        assert_ne!(mode, 2, "self mode {mode}: expected no late failure");
    }
}
impl Drop for InternalSetItem {
    fn drop(&mut self) {
        if let Some(trace) = &self.trace {
            trace.borrow_mut().push(format!("X{}", self.value));
        }
    }
}
#[derive(Clone, Copy)]
pub struct InternalSetRequest {
    pub target: i32,
    pub source: i32,
    pub moved: bool,
    pub mode: i32,
    pub invalid: i32,
}
impl InternalSetRequest {
    fn coordinate(self, source: bool) -> (i32, i32) {
        let offset = if source { self.source } else { self.target };
        let (row, col) = (-2 + offset / 3, 3 + offset % 3);
        match (source, self.invalid) {
            (false, 1 | 3) => (40, col),
            (true, 2 | 3) => (50, col),
            (false, 4 | 6) => (row, 41),
            (true, 5 | 6) => (row, 51),
            _ => (row, col),
        }
    }
    fn linear_index(self, source: bool) -> i32 {
        let row = self.coordinate(source).0;
        if row >= 40 {
            return row;
        }
        -2 + if source { self.source } else { self.target }
    }
}
pub enum InternalSetArray<'a> {
    Linear(Array1<'a, InternalSetItem>),
    Matrix(Array2<'a, InternalSetItem>),
}
impl<'a> InternalSetArray<'a> {
    pub fn owned(matrix: bool, trace: &InternalSetTrace) -> Self {
        let mut result = if matrix {
            Self::Matrix(Array2::with_bounds(-2, -1, 3, 5).unwrap())
        } else {
            Self::Linear(Array1::with_bounds(-2, 3).unwrap())
        };
        for (index, slot) in result.values_mut().iter_mut().enumerate() {
            *slot = InternalSetItem::supplied(index as i32 + 1, trace);
        }
        result
    }
    pub fn borrowed(matrix: bool, buffer: &'a mut [InternalSetItem]) -> Self {
        if matrix {
            return Self::Matrix(Array2::from_buffer_bounds(buffer, -2, -1, 3, 5).unwrap());
        }
        Self::Linear(Array1::from_slice_bounds(buffer, -2, 3).unwrap())
    }
    pub fn values(&self) -> &[InternalSetItem] {
        match self {
            Self::Linear(array) => array.as_slice(),
            Self::Matrix(array) => array.as_slice(),
        }
    }
    fn values_mut(&mut self) -> &mut [InternalSetItem] {
        match self {
            Self::Linear(array) => array.as_mut_slice(),
            Self::Matrix(array) => array.as_mut_slice(),
        }
    }
    pub fn assign(&mut self, request: InternalSetRequest) -> Result<(), ArrayError> {
        if request.moved {
            return self.assign_moved(request);
        }
        self.assign_const(request)
    }
    fn assign_const(&mut self, request: InternalSetRequest) -> Result<(), ArrayError> {
        let assign = |target: &mut InternalSetItem, donor: &InternalSetItem| {
            target.assign_copy(donor, request.mode)
        };
        let assign_self = |slot: &mut InternalSetItem| slot.assign_self(false, request.mode);
        match self {
            Self::Linear(array) => array.set_value_from_index_with_assignment(
                request.linear_index(false),
                request.linear_index(true),
                assign,
                assign_self,
            ),
            Self::Matrix(array) => array.set_value_from_index_with_assignment(
                request.coordinate(false),
                request.coordinate(true),
                assign,
                assign_self,
            ),
        }
    }
    fn assign_moved(&mut self, request: InternalSetRequest) -> Result<(), ArrayError> {
        let assign = |target: &mut InternalSetItem, donor: &mut InternalSetItem| {
            target.assign_move(donor, request.mode)
        };
        let assign_self = |slot: &mut InternalSetItem| slot.assign_self(true, request.mode);
        match self {
            Self::Linear(array) => array.set_value_moved_from_index_with_assignment(
                request.linear_index(false),
                request.linear_index(true),
                assign,
                assign_self,
            ),
            Self::Matrix(array) => array.set_value_moved_from_index_with_assignment(
                request.coordinate(false),
                request.coordinate(true),
                assign,
                assign_self,
            ),
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
    array: &mut InternalSetArray<'_>,
    request: InternalSetRequest,
    trace: &InternalSetTrace,
) -> String {
    let pointer = array.values().as_ptr();
    let result = catch_unwind(AssertUnwindSafe(|| array.assign(request)));
    let error = outcome_code(result);
    format!(
        "{};{};{error};{};{}",
        array.bounds(),
        values_row(array.values()),
        u8::from(pointer == array.values().as_ptr()),
        trace.borrow().join(",")
    )
}
fn outcome_code(result: std::thread::Result<Result<(), ArrayError>>) -> i32 {
    match result {
        Ok(Ok(())) => 0,
        Ok(Err(ArrayError::Index { index, .. })) => {
            if index >= 50 {
                2
            } else {
                3
            }
        }
        Ok(Err(error)) => panic!("unexpected {error}: expected an index error"),
        Err(_) => 1,
    }
}
pub fn values_row(values: &[InternalSetItem]) -> String {
    values
        .iter()
        .map(|item| item.value.to_string())
        .collect::<Vec<_>>()
        .join(",")
}
pub fn assert_native_row(label: &str, actual: &str) {
    let prefix = format!("{label}=");
    let expected =
        include_str!("../../../../tools/oracle/tkernel_array_internal_set_value.expected")
            .lines()
            .find_map(|line| line.strip_prefix(&prefix))
            .unwrap();
    assert_eq!(actual, expected, "native row {label}");
}
