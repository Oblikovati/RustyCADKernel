use rustycad_tkernel::ncollection::{Array1, Array2, ArrayError};
use std::{
    cell::RefCell,
    panic::{AssertUnwindSafe, catch_unwind},
    rc::Rc,
};

pub type EmplaceTrace = Rc<RefCell<Vec<String>>>;

// No Clone: native move assignment does not require a copy constructor.
#[derive(Default)]
pub struct EmplaceItem {
    pub value: i32,
    pub trace: Option<EmplaceTrace>,
}

impl EmplaceItem {
    pub fn supplied(value: i32, trace: &EmplaceTrace) -> Self {
        Self {
            value,
            trace: Some(Rc::clone(trace)),
        }
    }
    pub fn construct(trace: &EmplaceTrace, mode: i32) -> Self {
        trace.borrow_mut().push("C9".into());
        assert_ne!(
            mode, 1,
            "constructor mode {mode}: expected a non-failing mode"
        );
        Self::supplied(9, trace)
    }
    pub fn assign_selected(&mut self, donor: &mut Self, mode: i32) {
        if mode != 4 {
            self.assign_move(donor, mode);
            return;
        }
        self.trace
            .as_ref()
            .unwrap()
            .borrow_mut()
            .push(format!("Q{}:{}", self.value, donor.value));
        self.value = donor.value;
    }
    fn assign_move(&mut self, donor: &mut Self, mode: i32) {
        let trace = self.trace.as_ref().unwrap();
        trace
            .borrow_mut()
            .push(format!("A{}:{}", self.value, donor.value));
        assert_ne!(
            mode, 2,
            "assignment mode {mode}: expected a non-failing mode"
        );
        self.value = donor.value;
        donor.value = -donor.value;
        assert_ne!(
            mode, 3,
            "assignment mode {mode}: expected no failure after mutation"
        );
    }
}

impl Drop for EmplaceItem {
    fn drop(&mut self) {
        if let Some(trace) = &self.trace {
            trace.borrow_mut().push(format!("X{}", self.value));
        }
    }
}

pub enum EmplaceArray<'a> {
    Linear(Array1<'a, EmplaceItem>),
    Matrix(Array2<'a, EmplaceItem>),
}

impl<'a> EmplaceArray<'a> {
    pub fn owned(matrix: bool, trace: &EmplaceTrace) -> Self {
        let mut result = if matrix {
            Self::Matrix(Array2::with_bounds(-2, -1, 3, 4).unwrap())
        } else {
            Self::Linear(Array1::with_bounds(-2, 1).unwrap())
        };
        for (index, slot) in result.values_mut().iter_mut().enumerate() {
            *slot = EmplaceItem::supplied(index as i32 + 1, trace);
        }
        result
    }
    pub fn borrowed(matrix: bool, buffer: &'a mut [EmplaceItem]) -> Self {
        if matrix {
            return Self::Matrix(Array2::from_buffer_bounds(buffer, -2, -1, 3, 4).unwrap());
        }
        Self::Linear(Array1::from_slice_bounds(buffer, -2, 1).unwrap())
    }
    pub fn values(&self) -> &[EmplaceItem] {
        match self {
            Self::Linear(array) => array.as_slice(),
            Self::Matrix(array) => array.as_slice(),
        }
    }
    pub fn values_mut(&mut self) -> &mut [EmplaceItem] {
        match self {
            Self::Linear(array) => array.as_mut_slice(),
            Self::Matrix(array) => array.as_mut_slice(),
        }
    }
    pub fn emplace(
        &mut self,
        trace: &EmplaceTrace,
        mode: i32,
        invalid: i32,
    ) -> Result<&mut EmplaceItem, ArrayError> {
        let construct = || EmplaceItem::construct(trace, mode);
        let assign =
            |slot: &mut EmplaceItem, donor: &mut EmplaceItem| slot.assign_selected(donor, mode);
        let index = if invalid == 0 { -1 } else { 2 };
        let row = if invalid == 1 { 0 } else { -2 };
        let col = if invalid == 2 { 5 } else { 4 };
        match self {
            Self::Linear(array) => array.emplace_value_with_assignment(index, construct, assign),
            Self::Matrix(array) => array.emplace_value_with_assignment(row, col, construct, assign),
        }
    }
}

pub fn exercise(
    array: &mut EmplaceArray<'_>,
    trace: &EmplaceTrace,
    mode: i32,
    invalid: i32,
) -> String {
    let address = array.values().get(1).map(std::ptr::from_ref);
    let result = catch_unwind(AssertUnwindSafe(|| {
        array
            .emplace(trace, mode, invalid)
            .map(|slot| Some(std::ptr::from_mut(slot).cast_const()) == address)
    }));
    let (error, same) = record_outcome(result, trace);
    format!(
        "{};{error};{};{}",
        values_row(array.values()),
        u8::from(same),
        trace.borrow().join(",")
    )
}

fn record_outcome(
    result: std::thread::Result<Result<bool, ArrayError>>,
    trace: &EmplaceTrace,
) -> (i32, bool) {
    match result {
        Ok(Ok(same)) => {
            trace.borrow_mut().push("R".into());
            (0, same)
        }
        Ok(Err(_)) => (2, false),
        Err(_) => (1, false),
    }
}

pub fn values_row(values: &[EmplaceItem]) -> String {
    values
        .iter()
        .map(|item| item.value.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

pub fn assert_native_row(label: &str, actual: &str) {
    let prefix = format!("{label}=");
    let expected =
        include_str!("../../../../tools/oracle/tkernel_array_emplace_assignment.expected")
            .lines()
            .find_map(|line| line.strip_prefix(&prefix))
            .unwrap();
    assert_eq!(actual, expected, "native row {label}");
}
