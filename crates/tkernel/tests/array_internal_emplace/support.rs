use self::InternalEmplaceItem as Item;
use rustycad_tkernel::ncollection::{Array1, Array2, ArrayError};
use std::{
    cell::RefCell,
    panic::{AssertUnwindSafe, catch_unwind},
    rc::Rc,
};
pub type InternalEmplaceTrace = Rc<RefCell<Vec<String>>>;
#[derive(Default)]
pub struct InternalEmplaceItem {
    pub value: i32,
    pub trace: Option<InternalEmplaceTrace>,
}
impl InternalEmplaceItem {
    pub fn supplied(value: i32, trace: &InternalEmplaceTrace) -> Self {
        Self {
            value,
            trace: Some(Rc::clone(trace)),
        }
    }
    fn record(&self, event: String, mode: i32, failure: i32) {
        self.trace.as_ref().unwrap().borrow_mut().push(event);
        assert_ne!(mode, failure, "operation mode {mode}: expected success");
    }
    pub fn construct_copy(&self, mode: i32) -> Self {
        self.record(format!("C{}", self.value), mode, 1);
        self.record(format!("K{}", self.value * 10), mode, 2);
        Self::supplied(self.value * 10, self.trace.as_ref().unwrap())
    }
    pub fn construct_move(&mut self, mode: i32) -> Self {
        self.record(format!("M{}", self.value), mode, 1);
        let constructed = self.value * 10;
        self.value = -self.value;
        self.record(format!("K{constructed}"), mode, 2);
        Self::supplied(constructed, self.trace.as_ref().unwrap())
    }
    pub fn assign_temporary(&mut self, temporary: &mut Self, mode: i32) {
        temporary.record(format!("A{}:{}", self.value, temporary.value), mode, 3);
        self.value = temporary.value;
        temporary.value = -temporary.value;
        temporary.record(format!("L{}", self.value), mode, 4);
    }
}
impl Drop for InternalEmplaceItem {
    fn drop(&mut self) {
        if let Some(trace) = &self.trace {
            trace.borrow_mut().push(format!("X{}", self.value));
        }
    }
}
#[derive(Clone, Copy)]
pub struct InternalEmplaceRequest {
    pub target: i32,
    pub source: i32,
    pub moved: bool,
    pub mode: i32,
    pub invalid: i32,
}
impl InternalEmplaceRequest {
    fn assignment(self) -> impl FnOnce(&mut Item, &mut Item) {
        let mode = self.mode;
        move |target, temporary| target.assign_temporary(temporary, mode)
    }

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
pub enum InternalEmplaceArray<'a> {
    Linear(Array1<'a, InternalEmplaceItem>),
    Matrix(Array2<'a, InternalEmplaceItem>),
}
impl<'a> InternalEmplaceArray<'a> {
    pub fn owned(matrix: bool, trace: &InternalEmplaceTrace) -> Self {
        let mut result = if matrix {
            Self::Matrix(Array2::with_bounds(-2, -1, 3, 5).unwrap())
        } else {
            Self::Linear(Array1::with_bounds(-2, 3).unwrap())
        };
        for (index, slot) in result.values_mut().iter_mut().enumerate() {
            *slot = InternalEmplaceItem::supplied(index as i32 + 1, trace);
        }
        result
    }
    pub fn borrowed(matrix: bool, buffer: &'a mut [InternalEmplaceItem]) -> Self {
        if matrix {
            return Self::Matrix(Array2::from_buffer_bounds(buffer, -2, -1, 3, 5).unwrap());
        }
        Self::Linear(Array1::from_slice_bounds(buffer, -2, 3).unwrap())
    }
    pub fn values(&self) -> &[InternalEmplaceItem] {
        match self {
            Self::Linear(array) => array.as_slice(),
            Self::Matrix(array) => array.as_slice(),
        }
    }
    fn values_mut(&mut self) -> &mut [InternalEmplaceItem] {
        match self {
            Self::Linear(array) => array.as_mut_slice(),
            Self::Matrix(array) => array.as_mut_slice(),
        }
    }
    pub fn assign(
        &mut self,
        request: InternalEmplaceRequest,
    ) -> Result<&mut InternalEmplaceItem, ArrayError> {
        if request.moved {
            return self.assign_moved(request);
        }
        self.assign_const(request)
    }
    fn assign_const(&mut self, request: InternalEmplaceRequest) -> Result<&mut Item, ArrayError> {
        let construct = |source: &InternalEmplaceItem| source.construct_copy(request.mode);
        let assign = request.assignment();
        match self {
            Self::Linear(array) => array.emplace_value_from_index_with_assignment(
                request.linear_index(false),
                request.linear_index(true),
                construct,
                assign,
            ),
            Self::Matrix(array) => array.emplace_value_from_index_with_assignment(
                request.coordinate(false),
                request.coordinate(true),
                construct,
                assign,
            ),
        }
    }
    fn assign_moved(&mut self, request: InternalEmplaceRequest) -> Result<&mut Item, ArrayError> {
        let construct = |source: &mut InternalEmplaceItem| source.construct_move(request.mode);
        let assign = request.assignment();
        match self {
            Self::Linear(array) => array.emplace_value_moved_from_index_with_assignment(
                request.linear_index(false),
                request.linear_index(true),
                construct,
                assign,
            ),
            Self::Matrix(array) => array.emplace_value_moved_from_index_with_assignment(
                request.coordinate(false),
                request.coordinate(true),
                construct,
                assign,
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
    array: &mut InternalEmplaceArray<'_>,
    request: InternalEmplaceRequest,
    trace: &InternalEmplaceTrace,
) -> String {
    let pointer = array.values().as_ptr();
    let target = array
        .values()
        .get(request.target as usize)
        .map(std::ptr::from_ref);
    let result = catch_unwind(AssertUnwindSafe(|| {
        array
            .assign(request)
            .map(|slot| Some(std::ptr::from_mut(slot).cast_const()) == target)
    }));
    let (error, same) = outcome_code(result, trace);
    let stable = pointer == array.values().as_ptr();
    exercise_row(array, trace, error, same, stable)
}
fn exercise_row(
    array: &InternalEmplaceArray<'_>,
    trace: &InternalEmplaceTrace,
    error: i32,
    same: bool,
    stable: bool,
) -> String {
    format!(
        "{};{};{error};{};{};{}",
        array.bounds(),
        values_row(array.values()),
        u8::from(stable),
        u8::from(same),
        trace.borrow().join(",")
    )
}

fn outcome_code(
    result: std::thread::Result<Result<bool, ArrayError>>,
    trace: &InternalEmplaceTrace,
) -> (i32, bool) {
    match result {
        Ok(Ok(same)) => {
            trace.borrow_mut().push("R".into());
            (0, same)
        }
        Ok(Err(ArrayError::Index { index, .. })) => (if index >= 50 { 2 } else { 3 }, false),
        Ok(Err(error)) => panic!("unexpected {error}: expected an index error"),
        Err(_) => (1, false),
    }
}
pub fn values_row(values: &[InternalEmplaceItem]) -> String {
    values
        .iter()
        .map(|item| item.value.to_string())
        .collect::<Vec<_>>()
        .join(",")
}
pub fn assert_native_row(label: &str, actual: &str) {
    let prefix = format!("{label}=");
    let expected = include_str!("../../../../tools/oracle/tkernel_array_internal_emplace.expected")
        .lines()
        .find_map(|line| line.strip_prefix(&prefix))
        .unwrap();
    assert_eq!(actual, expected, "native row {label}");
}
