use rustycad_tkernel::ncollection::{Array1, Array2, ArrayError};
use std::{
    cell::RefCell,
    panic::{AssertUnwindSafe, catch_unwind},
    rc::Rc,
};
pub type CopyTrace = Rc<RefCell<Vec<String>>>;
#[derive(Default)]
pub struct AssignOnlyItem {
    pub value: i32,
    pub trace: Option<CopyTrace>,
}
impl AssignOnlyItem {
    pub fn supplied(value: i32, trace: &CopyTrace) -> Self {
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
            !(mode == 1 && old == 2),
            "mode {mode}, target {old}: expected no early failure"
        );
        self.value = donor.value;
        assert!(
            !(mode == 2 && old == 2),
            "mode {mode}, target {old}: expected no late failure"
        );
    }
}
impl Drop for AssignOnlyItem {
    fn drop(&mut self) {
        if let Some(trace) = &self.trace {
            trace.borrow_mut().push(format!("X{}", self.value));
        }
    }
}
pub enum CopyArray<'a> {
    Linear(Array1<'a, AssignOnlyItem>),
    Matrix(Array2<'a, AssignOnlyItem>),
}
impl<'a> CopyArray<'a> {
    pub fn owned(matrix: bool, source: bool, trace: &CopyTrace) -> Self {
        let mut result = if matrix {
            Self::Matrix(Array2::with_bounds(-2, -1, 3, 4).unwrap())
        } else {
            Self::Linear(Array1::with_bounds(-2, 1).unwrap())
        };
        for (index, slot) in result.values_mut().iter_mut().enumerate() {
            *slot = AssignOnlyItem::supplied(index as i32 + if source { 11 } else { 1 }, trace);
        }
        if source {
            result.rebase_source();
        }
        result
    }
    pub fn borrowed(matrix: bool, buffer: &'a mut [AssignOnlyItem], source: bool) -> Self {
        let mut result = if matrix {
            Self::Matrix(Array2::from_buffer_bounds(buffer, -2, -1, 3, 4).unwrap())
        } else {
            Self::Linear(Array1::from_slice_bounds(buffer, -2, 1).unwrap())
        };
        if source {
            result.rebase_source();
        }
        result
    }
    fn rebase_source(&mut self) {
        match self {
            Self::Linear(array) => array.update_lower_bound(7),
            Self::Matrix(array) => {
                array.update_lower_row(7);
                array.update_lower_col(-4);
            }
        }
    }
    pub fn values(&self) -> &[AssignOnlyItem] {
        match self {
            Self::Linear(array) => array.as_slice(),
            Self::Matrix(array) => array.as_slice(),
        }
    }
    fn values_mut(&mut self) -> &mut [AssignOnlyItem] {
        match self {
            Self::Linear(array) => array.as_mut_slice(),
            Self::Matrix(array) => array.as_mut_slice(),
        }
    }
    pub fn copy_values(&mut self, source: &CopyArray<'_>, mode: i32) -> Result<bool, ArrayError> {
        let assign =
            |target: &mut AssignOnlyItem, donor: &AssignOnlyItem| target.assign(donor, mode);
        match (self, source) {
            (Self::Linear(target), CopyArray::Linear(donors)) => {
                let address = std::ptr::from_ref(target);
                target
                    .copy_values_with_assignment(donors, assign)
                    .map(|returned| std::ptr::from_ref(returned) == address)
            }
            (Self::Matrix(target), CopyArray::Matrix(donors)) => {
                let address = std::ptr::from_ref(target);
                target
                    .copy_values_with_assignment(donors, assign)
                    .map(|returned| std::ptr::from_ref(returned) == address)
            }
            _ => unreachable!("expected two arrays of the same dimensionality"),
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
    target: &mut CopyArray<'_>,
    source: &CopyArray<'_>,
    mode: i32,
    trace: &CopyTrace,
) -> String {
    let pointer = target.values().as_ptr();
    let result = catch_unwind(AssertUnwindSafe(|| target.copy_values(source, mode)));
    let (error, returned) = outcome(result);
    let same_storage = target.values().as_ptr() == pointer;
    format!(
        "{};{};{};{error};{};{};{}",
        target.bounds(),
        values_row(target.values()),
        values_row(source.values()),
        u8::from(returned),
        u8::from(same_storage),
        trace.borrow().join(",")
    )
}
fn outcome(result: std::thread::Result<Result<bool, ArrayError>>) -> (i32, bool) {
    match result {
        Ok(Ok(same)) => (0, same),
        Ok(Err(ArrayError::ShapeMismatch { .. })) => (3, false),
        Ok(Err(ArrayError::DimensionMismatch { .. })) => (2, false),
        Ok(Err(error)) => panic!("unexpected copy error {error}: expected dimensions or shape"),
        Err(_) => (1, false),
    }
}
pub fn values_row(values: &[AssignOnlyItem]) -> String {
    values
        .iter()
        .map(|item| item.value.to_string())
        .collect::<Vec<_>>()
        .join(",")
}
pub fn assert_native_row(label: &str, actual: &str) {
    let prefix = format!("{label}=");
    let expected =
        include_str!("../../../../tools/oracle/tkernel_array_copy_values_assignment.expected")
            .lines()
            .find_map(|line| line.strip_prefix(&prefix))
            .unwrap();
    assert_eq!(actual, expected, "native row {label}");
}
