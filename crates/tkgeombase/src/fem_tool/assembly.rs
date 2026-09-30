// Rust translation of ModelingData/TKGeomBase/FEmTool/FEmTool_Assembly.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `FEmTool_Assembly`: the global quadratic system H X = B of the finite-element
//! approximation, with linear equality constraints G X = C solved by the Schur
//! complement G H^-1 G^t.
mod constraints;
mod solve;

use super::profile_matrix::ProfileMatrix;
use super::sparse_matrix::SparseMatrix;
use crate::native_failure::{array_failure, failure};
use rustycad_tkernel::ncollection::{Array1, Array2, HArray1, HArray2};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::math::{Matrix, Vector};

/// `occ::handle<NCollection_HArray2<occ::handle<NCollection_HArray1<int>>>>`: for each
/// (dimension, element), the global indexes of the element variables.
pub type AssemblyTable = Handle<HArray2<Handle<HArray1<i32>>>>;

/// One constraint row of G: its non-null segments, each a coefficient array over
/// a range of global variables (`NCollection_List<handle<HArray1<double>>>`).
pub(super) type ConstraintRow = Vec<Array1<'static, f64>>;

/// `FEmTool_Assembly`.
///
/// ```
/// use rustycad_tkgeombase::fem_tool::{Assembly, AssemblyTable};
/// use rustycad_tkernel::ncollection::{Array1, Array2, HArray1, HArray2};
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkmath::math::{Matrix, Vector};
/// let mut dependence = Array2::with_bounds(1, 1, 1, 1).unwrap();
/// dependence.init(1);
/// let table: AssemblyTable = Handle::new(HArray2::with_bounds(1, 1, 1, 1).unwrap());
/// table.change_array2()[(1, 1)] = Handle::new(HArray1::from_array(&Array1::from_shared(&[1, 2], 1)));
/// let mut assembly = Assembly::new(&dependence, &table)?;
/// let mut h = Matrix::filled(1, 2, 1, 2, 0.0).unwrap();
/// (h[(1, 1)], h[(2, 1)], h[(2, 2)]) = (2.0, 1.0, 2.0);
/// assembly.add_matrix(1, 1, 1, &h)?;
/// assembly.add_vector(1, 1, &Vector::filled(1, 2, 3.0)?)?;
/// assert!(assembly.solve()?);
/// let mut x = Vector::filled(1, 2, 0.0)?;
/// assembly.solution(&mut x)?;
/// assert!((x[1] - 1.0).abs() < 1e-12 && (x[2] - 1.0).abs() < 1e-12);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct Assembly {
    dep_table: Array2<'static, i32>,
    ref_table: AssemblyTable,
    is_solved: bool,
    h: Handle<ProfileMatrix>,
    b: Vector<'static>,
    ghgt: Handle<ProfileMatrix>,
    g: Vec<ConstraintRow>,
    c: Vec<f64>,
}

// A copy of `Dependence` with its bounds (NCollection_Array2 assignment).
fn owned_table(dependence: &Array2<'_, i32>) -> Result<Array2<'static, i32>, Failure> {
    let mut table: Array2<'static, i32> = Array2::with_bounds(
        dependence.lower_row(),
        dependence.upper_row(),
        dependence.lower_col(),
        dependence.upper_col(),
    )
    .map_err(|error| array_failure(error, "FEmTool_Assembly"))?;
    table
        .copy_values(dependence)
        .map_err(|error| array_failure(error, "FEmTool_Assembly"))?;
    Ok(table)
}

// Every global index of the table, dimension by dimension and element by element.
fn table_indexes(table: &AssemblyTable) -> Result<Vec<Vec<i32>>, Failure> {
    let Some(cells) = table.get() else {
        return Err(failure(
            "Standard_NullObject",
            "FEmTool_Assembly: null assembly table",
        ));
    };
    let cells = cells.array2();
    let mut result: Vec<Vec<i32>> = Vec::new();
    for dim in cells.lower_row()..=cells.upper_row() {
        for el in cells.lower_col()..=cells.upper_col() {
            result.push(element_indexes(&cells[(dim, el)])?);
        }
    }
    Ok(result)
}

// The global indexes of one (dimension, element) cell.
fn element_indexes(cell: &Handle<HArray1<i32>>) -> Result<Vec<i32>, Failure> {
    let Some(indexes) = cell.get() else {
        return Err(failure(
            "Standard_NullObject",
            "FEmTool_Assembly: null element index array",
        ));
    };
    Ok(indexes.array1().iter().copied().collect())
}

// MinIndex/MaxIndex: the extreme global index of the table (the first cell's first
// value when it holds none else); Standard_OutOfRange for an empty first cell.
fn index_range(cells: &[Vec<i32>]) -> Result<(i32, i32), Failure> {
    let first: i32 = *cells.first().and_then(|cell| cell.first()).ok_or_else(|| {
        Failure::out_of_range("FEmTool_Assembly: the first assembly table cell holds no index")
    })?;
    let values = cells.iter().flatten().copied();
    Ok((
        values.clone().fold(first, i32::min),
        values.fold(first, i32::max),
    ))
}

// FirstIndexes of H: for each global variable, the smallest variable of any element
// sharing it (1-based through I0 = 1 - B.Lower()).
fn first_indexes(
    cells: &[Vec<i32>],
    lower: i32,
    length: i32,
) -> Result<Array1<'static, i32>, Failure> {
    let mut first: Array1<'static, i32> =
        Array1::with_bounds(1, length).map_err(|error| array_failure(error, "FEmTool_Assembly"))?;
    first.init(length);
    let i0: i32 = 1 - lower;
    for cell in cells {
        let Some(minimum) = cell.iter().map(|value| value + i0).min() else {
            return Err(Failure::out_of_range(
                "FEmTool_Assembly: an assembly table cell holds no index",
            ));
        };
        for value in cell {
            let slot = first
                .change_value(value + i0)
                .map_err(|error| array_failure(error, "FEmTool_Assembly: FirstIndexes"))?;
            *slot = (*slot).min(minimum);
        }
    }
    Ok(first)
}

impl Assembly {
    /// `FEmTool_Assembly(Dependence, Table)`: the system over the global variables
    /// `MinIndex..=MaxIndex` of `table`, nullified; `Assembly::new(&dependence, &table)?`.
    pub fn new(dependence: &Array2<'_, i32>, table: &AssemblyTable) -> Result<Self, Failure> {
        let cells: Vec<Vec<i32>> = table_indexes(table)?;
        let (min_index, max_index) = index_range(&cells)?;
        let b: Vector<'static> = Vector::with_bounds(min_index, max_index)?;
        let first: Array1<'static, i32> = first_indexes(&cells, b.lower(), b.length())?;
        let mut assembly: Self = Self {
            dep_table: owned_table(dependence)?,
            ref_table: table.clone(),
            is_solved: false,
            h: Handle::new(ProfileMatrix::new(&first)?),
            b,
            ghgt: Handle::default(),
            g: Vec::new(),
            c: Vec::new(),
        };
        assembly.nullify_matrix();
        assembly.nullify_vector();
        Ok(assembly)
    }

    /// `NullifyMatrix()`; `assembly.nullify_matrix()`.
    pub fn nullify_matrix(&mut self) {
        self.h.init(0.);
        self.is_solved = false;
    }

    /// `NullifyVector()`; `assembly.nullify_vector()`.
    pub fn nullify_vector(&mut self) {
        self.b.init(0.);
    }

    /// `AddMatrix(Element, Dimension1, Dimension2, Mat)`: adds the lower triangle of
    /// `mat` (rows and columns from its lower bounds) into H; Standard_DomainError
    /// when `Dependence(Dimension1, Dimension2)` is 0; `assembly.add_matrix(1, 1, 1, &m)?`.
    pub fn add_matrix(
        &mut self,
        element: i32,
        dimension1: i32,
        dimension2: i32,
        mat: &Matrix<'_>,
    ) -> Result<(), Failure> {
        let dependent: i32 = *self
            .dep_table
            .value(dimension1, dimension2)
            .map_err(|error| array_failure(error, "FEmTool_Assembly::AddMatrix"))?;
        if dependent == 0 {
            return Err(failure(
                "Standard_DomainError",
                "FEmTool_Assembly::AddMatrix",
            ));
        }
        let t1: Array1<'static, i32> = self.cell(dimension1, element)?;
        let t2: Array1<'static, i32> = self.cell(dimension2, element)?;
        let nvarl: i32 = t1.lower();
        let nvaru: i32 = t1.upper().min(nvarl + mat.row_number() - 1);
        let (i0, row0, col0) = (
            1 - self.b.lower(),
            mat.lower_row() - nvarl,
            mat.lower_col() - nvarl,
        );
        for i in nvarl..=nvaru {
            let big_i: i32 = *t1
                .value(i)
                .map_err(|error| array_failure(error, "FEmTool_Assembly"))?
                + i0;
            for j in nvarl..=i {
                let big_j: i32 = *t2
                    .value(j)
                    .map_err(|error| array_failure(error, "FEmTool_Assembly"))?
                    + i0;
                let value: f64 = *mat.value(row0 + i, col0 + j)?;
                *self.h.change_value(big_i, big_j)? += value;
            }
        }
        self.is_solved = false;
        Ok(())
    }

    /// `AddVector(Element, Dimension, Vec)`: adds `vec` (from its lower bound) into B
    /// at the element's global indexes; `assembly.add_vector(1, 1, &v)?`.
    pub fn add_vector(
        &mut self,
        element: i32,
        dimension: i32,
        vec: &Vector<'_>,
    ) -> Result<(), Failure> {
        let t: Array1<'static, i32> = self.cell(dimension, element)?;
        let nvarl: i32 = t.lower();
        let nvaru: i32 = t.upper().min(nvarl + vec.length() - 1);
        let i0: i32 = vec.lower() - nvarl;
        for i in nvarl..=nvaru {
            let index: i32 = *t
                .value(i)
                .map_err(|error| array_failure(error, "FEmTool_Assembly"))?;
            *self.b.change_value(index)? += *vec.value(i0 + i)?;
        }
        Ok(())
    }

    /// `NbGlobVar()`: the number of global variables; `assembly.nb_glob_var()`.
    pub fn nb_glob_var(&self) -> i32 {
        self.b.length()
    }

    /// `AssemblyTable()`: the table given at construction; `assembly.assembly_table()`.
    pub fn assembly_table(&self) -> &AssemblyTable {
        &self.ref_table
    }

    /// `GetAssemblyTable(AssTable)` (deprecated natively): a copy of the table
    /// handle; `let table = assembly.get_assembly_table();`.
    #[deprecated(note = "Use AssemblyTable() returning const reference instead")]
    pub fn get_assembly_table(&self) -> AssemblyTable {
        self.ref_table.clone()
    }

    // Table(dimension, element): Standard_OutOfRange outside the table.
    fn cell(&self, dimension: i32, element: i32) -> Result<Array1<'static, i32>, Failure> {
        let cells = self.ref_table.array2();
        let cell: &Handle<HArray1<i32>> = cells
            .value(dimension, element)
            .map_err(|error| array_failure(error, "FEmTool_Assembly: assembly table"))?;
        let Some(indexes) = cell.get() else {
            return Err(failure(
                "Standard_NullObject",
                "FEmTool_Assembly: null element index array",
            ));
        };
        Ok(indexes.array1().clone())
    }
}
