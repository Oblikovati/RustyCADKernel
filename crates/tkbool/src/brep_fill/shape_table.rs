//! The tables of shapes (`NCollection_Array2<TopoDS_Shape>`) shared by BRepFill_Sweep
//! and BRepFill_TrimShellCorner, with checked cell access.
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::ncollection::{Array2, ArrayError, HArray2};
use rustycad_tkernel::standard::{Failure, Handle};

/// A table of shapes (`NCollection_Array2<TopoDS_Shape>`).
pub type ShapeTable = Array2<'static, Shape>;

/// Standard_OutOfRange for a table access, with the offending access.
pub(crate) fn table_error(error: ArrayError) -> Failure {
    Failure::with_type(
        "Standard_OutOfRange",
        &format!("BRepFill: {error}; expected an index inside the shape table"),
        "NCollection_Array2",
    )
}

/// The item at `(row, col)` of a table; Standard_OutOfRange outside its bounds.
pub(crate) fn cell<'a, T>(
    table: &'a Array2<'static, T>,
    row: i32,
    col: i32,
) -> Result<&'a T, Failure> {
    table.value(row, col).map_err(table_error)
}

/// The item at `(row, col)` of a table, for writing; Standard_OutOfRange outside its
/// bounds.
pub(crate) fn cell_mut<'a, T>(
    table: &'a mut Array2<'static, T>,
    row: i32,
    col: i32,
) -> Result<&'a mut T, Failure> {
    table.change_value(row, col).map_err(table_error)
}

/// A table with the inclusive bounds, every item at its default value.
pub(crate) fn new_table<T: Default>(
    rows: (i32, i32),
    cols: (i32, i32),
) -> Result<Array2<'static, T>, Failure> {
    Array2::with_bounds(rows.0, rows.1, cols.0, cols.1).map_err(table_error)
}

/// A copy of a table (`NCollection_Array2::operator=`).
pub(crate) fn clone_table<T: Clone>(source: &Array2<'static, T>) -> Array2<'static, T> {
    let mut copy = Array2::default();
    copy.assign(source);
    copy
}

/// The table wrapped in a new shared handle.
pub(crate) fn to_handle(table: &ShapeTable) -> Handle<HArray2<Shape>> {
    Handle::new(HArray2::from_owned(clone_table(table)))
}

/// A copy of the table held by a handle (`new HArray2(bounds); ChangeArray2() = Array2()`);
/// Standard_NullObject for a null handle (a crash natively).
pub(crate) fn copy_table(
    source: &Handle<HArray2<Shape>>,
    owner: &str,
) -> Result<ShapeTable, Failure> {
    let Some(shared) = source.get() else {
        return Err(Failure::with_type(
            "Standard_NullObject",
            &format!("{owner}: null shape table; expected an NCollection_HArray2<TopoDS_Shape>"),
            owner,
        ));
    };
    Ok(clone_table(&shared.array2()))
}
