use rustycad_tkernel::ncollection::{Array1, Array2};
use std::{cell::RefCell, rc::Rc};

pub fn linear_queries<T>(array: &Array1<'_, T>) -> String {
    format!(
        "{}:{}:{}:{}:{}:{}",
        array.len(),
        array.length(),
        u8::from(array.is_empty()),
        array.lower(),
        array.upper(),
        u8::from(array.is_deletable())
    )
}
fn matrix_query_prefix<T>(array: &Array2<'_, T>) -> String {
    format!(
        "{}:{}:{}:{}:{}:{}:{}:{}",
        array.len(),
        array.length(),
        u8::from(array.is_empty()),
        array.lower_row(),
        array.upper_row(),
        array.lower_col(),
        array.upper_col(),
        u8::from(array.is_deletable())
    )
}
pub fn matrix_queries<T>(array: &Array2<'_, T>) -> String {
    let prefix = matrix_query_prefix(array);
    format!(
        "{prefix}:{}:{}:{}:{}:{}:{}",
        array.nb_rows(),
        array.nb_columns(),
        array.row_size(),
        array.col_size(),
        array.row_length(),
        array.col_length()
    )
}
pub fn assert_surface_row(label: &str, actual: &str) {
    let prefix = format!("{label}=");
    let expected = include_str!("../../../../tools/oracle/tkernel_array_exact_surface.expected")
        .lines()
        .find_map(|line| line.strip_prefix(&prefix))
        .unwrap();
    assert_eq!(actual, expected, "native row {label}");
}
type SurfaceTrace = Rc<RefCell<Vec<i32>>>;
#[derive(Default)]
struct SurfaceCell {
    value: i32,
    trace: Option<SurfaceTrace>,
}
impl Drop for SurfaceCell {
    fn drop(&mut self) {
        if let Some(trace) = &self.trace {
            trace.borrow_mut().push(self.value);
        }
    }
}
enum SurfaceArray<'a> {
    Linear(Array1<'a, SurfaceCell>),
    Matrix(Array2<'a, SurfaceCell>),
}
impl<'a> SurfaceArray<'a> {
    fn owned(matrix: bool) -> Self {
        if matrix {
            return Self::Matrix(Array2::with_bounds(-2, -1, 3, 4).unwrap());
        }
        Self::Linear(Array1::with_bounds(-2, 1).unwrap())
    }
    fn borrowed(matrix: bool, buffer: &'a mut [SurfaceCell]) -> Self {
        if matrix {
            return Self::Matrix(Array2::from_buffer_bounds(buffer, -2, -1, 3, 4).unwrap());
        }
        Self::Linear(Array1::from_slice_bounds(buffer, -2, 1).unwrap())
    }
    fn values(&mut self) -> &mut [SurfaceCell] {
        match self {
            Self::Linear(array) => array.as_mut_slice(),
            Self::Matrix(array) => array.as_mut_slice(),
        }
    }
    fn pointer(&self) -> *const SurfaceCell {
        match self {
            Self::Linear(array) => array.data_ptr(),
            Self::Matrix(array) => array.data_ptr(),
        }
    }
    fn queries(&self) -> String {
        match self {
            Self::Linear(array) => linear_queries(array),
            Self::Matrix(array) => matrix_queries(array),
        }
    }
    fn take(&mut self) -> Self {
        match self {
            Self::Linear(array) => Self::Linear(std::mem::take(array)),
            Self::Matrix(array) => Self::Matrix(std::mem::take(array)),
        }
    }
}
fn trace_values(trace: &SurfaceTrace) -> String {
    trace
        .borrow()
        .iter()
        .map(i32::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
fn attach_trace(source: &mut SurfaceArray<'_>, trace: &SurfaceTrace) {
    for (offset, slot) in source.values().iter_mut().enumerate() {
        slot.value = offset as i32 + 1;
        slot.trace = Some(Rc::clone(trace));
    }
}
fn check_move(source: &mut SurfaceArray<'_>, trace: &SurfaceTrace, label: &str) {
    let pointer = source.pointer();
    let target = source.take();
    let actual = format!(
        "{};{};{};{};{}",
        source.queries(),
        target.queries(),
        u8::from(source.pointer().is_null()),
        u8::from(target.pointer() == pointer),
        trace_values(trace)
    );
    assert_surface_row(&format!("move_{label}"), &actual);
    drop(target);
    assert_surface_row(&format!("drop_{label}"), &trace_values(trace));
}
pub fn check_lifecycle(matrix: bool, owned: bool) {
    let trace = Rc::new(RefCell::new(Vec::new()));
    let mut buffer = std::array::from_fn::<_, 4, _>(|_| SurfaceCell::default());
    let mut source = if owned {
        SurfaceArray::owned(matrix)
    } else {
        SurfaceArray::borrowed(matrix, &mut buffer)
    };
    attach_trace(&mut source, &trace);
    let label = format!(
        "{}_{}",
        if matrix { "matrix" } else { "linear" },
        u8::from(owned)
    );
    check_move(&mut source, &trace, &label);
}
