// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_Filling.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomFill_Filling`: the poles and weights of a surface filled between boundary
//! pole rows, the base of `GeomFill_Coons`, `GeomFill_Curved` and `GeomFill_Stretch`.
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::errors;
use rustycad_tkmath::gp::Point3;

/// The result of a filling scheme (`GeomFill_Filling`): the pole grid and, for a
/// rational filling, the weight grid. `None` stands for the null array handle of a
/// scheme that was never initialized.
///
/// `let coons = GeomFillCoons::with_poles([&p1, &p2, &p3, &p4])?; coons.nb_u_poles()?`
#[derive(Clone, Default)]
pub struct GeomFillFilling {
    pub(super) is_rational: bool,
    pub(super) my_poles: Option<Array2<'static, Point3>>,
    pub(super) my_weights: Option<Array2<'static, f64>>,
}

impl GeomFillFilling {
    /// An empty, non-rational filling (`GeomFill_Filling()`); `GeomFillFilling::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `NbUPoles()`: the rows of the pole grid; Standard_NullObject before any
    /// `Init` (a null dereference natively); `filling.nb_u_poles()?`.
    pub fn nb_u_poles(&self) -> Result<i32, Failure> {
        Ok(self.poles_grid()?.col_length())
    }
    /// `NbVPoles()`: the columns of the pole grid; `filling.nb_v_poles()?`.
    pub fn nb_v_poles(&self) -> Result<i32, Failure> {
        Ok(self.poles_grid()?.row_length())
    }
    /// `Poles(Poles)`: the pole grid assigned to `poles` (bounds and size included);
    /// `filling.poles(&mut grid)?`.
    pub fn poles(&self, poles: &mut Array2<'_, Point3>) -> Result<(), Failure> {
        poles.assign(self.poles_grid()?);
        Ok(())
    }
    /// `isRational()`; `filling.is_rational()`.
    pub fn is_rational(&self) -> bool {
        self.is_rational
    }
    /// `Weights(Weights)`: the weight grid assigned to `weights`. A non-rational
    /// `Init` keeps the grid of an earlier rational one, as natively; without any,
    /// Standard_NullObject; `filling.weights(&mut grid)?`.
    pub fn weights(&self, weights: &mut Array2<'_, f64>) -> Result<(), Failure> {
        let grid = self.my_weights.as_ref().ok_or_else(|| {
            errors::null_object("GeomFill_Filling::Weights: no weights; expected a rational Init")
        })?;
        weights.assign(grid);
        Ok(())
    }
    fn poles_grid(&self) -> Result<&Array2<'static, Point3>, Failure> {
        self.my_poles
            .as_ref()
            .ok_or_else(|| errors::null_object("GeomFill_Filling: no poles; expected an Init"))
    }
    /// A fresh pole grid `(1..nu, 1..nv)` stored before it is filled, so that an
    /// exception leaves the partial grid, as natively.
    pub(super) fn new_poles(
        &mut self,
        nu: i32,
        nv: i32,
    ) -> Result<&mut Array2<'static, Point3>, Failure> {
        Ok(self.my_poles.insert(grid(nu, nv)?))
    }
    /// The weight grid counterpart of `new_poles`.
    pub(super) fn new_weights(
        &mut self,
        nu: i32,
        nv: i32,
    ) -> Result<&mut Array2<'static, f64>, Failure> {
        Ok(self.my_weights.insert(grid(nu, nv)?))
    }
}

/// `new NCollection_HArray2<T>(1, nu, 1, nv)`.
fn grid<T: Default + Clone>(nu: i32, nv: i32) -> Result<Array2<'static, T>, Failure> {
    Array2::with_bounds(1, nu, 1, nv).map_err(errors::array_bounds)
}

/// `Array(Index)` with the Standard_OutOfRange of the native accessor.
pub(super) fn at<T: Copy>(array: &Array1<'_, T>, index: i32) -> Result<T, Failure> {
    rustycad_tkg2d::geom2d::arrays::at(array, index)
}

/// `Grid->SetValue(Row, Col, Value)` with the Standard_OutOfRange of the native accessor.
pub(super) fn put2<T>(
    grid: &mut Array2<'_, T>,
    row: i32,
    col: i32,
    value: T,
) -> Result<(), Failure> {
    grid.set_value(row, col, value).map_err(|error| {
        errors::out_of_range(&format!(
            "NCollection_Array2::SetValue: ({row}, {col}); {error}"
        ))
    })
}

/// The Standard_DomainError of the scheme constructors (an empty native message).
pub(super) fn domain_error(member: &str, lengths: &[usize]) -> Failure {
    errors::domain(&format!(
        "{member}: boundary lengths {lengths:?}; expected opposite rows of equal length and one weight per pole"
    ))
}

/// The row lengths reported by a Standard_DomainError.
pub(super) fn lengths<T>(rows: [&Array1<'_, T>; 4]) -> [usize; 4] {
    rows.map(|row| row.len())
}

/// The blending ratios of one inner pole: `PU = (i - 1) / NU`, `PU1 = 1 - PU`, and the
/// same in V (halved in the curved scheme).
#[derive(Clone, Copy)]
pub(super) struct Ratios {
    pub(super) pu: f64,
    pub(super) pu1: f64,
    pub(super) pv: f64,
    pub(super) pv1: f64,
}

/// `(k - 1) / N` and its complement, halved when `halve`, as the native loops do.
fn ratio(k: i32, n: f64, halve: bool) -> (f64, f64) {
    let mut p: f64 = f64::from(k - 1) / n;
    let mut p1: f64 = 1.0 - p;
    if halve {
        p /= 2.;
        p1 /= 2.;
    }
    (p, p1)
}

/// The four boundary rows of the Stretch and Curved schemes: `b[0]` and `b[2]` are the
/// first and last columns, `b[3]` and `b[1]` the first and last rows.
pub(super) type Rows<'r, 'a, T> = [&'r Array1<'a, T>; 4];

/// The grid loop of `GeomFill_Stretch::Init` and `GeomFill_Curved::Init` (and their
/// weighted forms): the boundary rows copied to the grid border, then every inner
/// value from `inner(ratios, i, j)`, in the native order (column by column).
pub(super) fn fill_four_sided<T: Copy>(
    target: &mut Array2<'_, T>,
    rows: Rows<'_, '_, T>,
    halve: bool,
    inner: impl Fn(Ratios, i32, i32) -> Result<T, Failure>,
) -> Result<(), Failure> {
    let (npol_u, npol_v) = (target.col_length(), target.row_length());
    let (nu, nv) = (f64::from(npol_u - 1), f64::from(npol_v - 1));
    for i in 1..=npol_u {
        put2(target, i, 1, at(rows[0], i)?)?;
        put2(target, i, npol_v, at(rows[2], i)?)?;
    }
    for j in 2..=npol_v - 1 {
        let (pv, pv1) = ratio(j, nv, halve);
        put2(target, 1, j, at(rows[3], j)?)?;
        put2(target, npol_u, j, at(rows[1], j)?)?;
        for i in 2..=npol_u - 1 {
            let (pu, pu1) = ratio(i, nu, halve);
            let value: T = inner(Ratios { pu, pu1, pv, pv1 }, i, j)?;
            put2(target, i, j, value)?;
        }
    }
    Ok(())
}

/// The bilinear blend `PV1 R0(i) + PV R2(i) + PU R1(j) + PU1 R3(j)` of one coordinate.
pub(super) fn blend(r: Ratios, v: [f64; 4]) -> f64 {
    r.pv1 * v[0] + r.pv * v[1] + r.pu * v[2] + r.pu1 * v[3]
}

/// The corner correction `PU1 PV1 R0(1) + PU PV1 R1(1) + PU PV R2(NU) + PU1 PV R3(NV)`
/// of one coordinate.
pub(super) fn corners(r: Ratios, c: [f64; 4]) -> f64 {
    r.pu1 * r.pv1 * c[0] + r.pu * r.pv1 * c[1] + r.pu * r.pv * c[2] + r.pu1 * r.pv * c[3]
}

/// The values `R0(i), R2(i), R1(j), R3(j)` that `blend` weighs.
pub(super) fn blend_terms<T: Copy>(
    rows: Rows<'_, '_, T>,
    i: i32,
    j: i32,
) -> Result<[T; 4], Failure> {
    Ok([
        at(rows[0], i)?,
        at(rows[2], i)?,
        at(rows[1], j)?,
        at(rows[3], j)?,
    ])
}

/// The corner values `R0(1), R1(1), R2(NU), R3(NV)` that `corners` weighs.
pub(super) fn corner_terms<T: Copy>(rows: Rows<'_, '_, T>) -> Result<[T; 4], Failure> {
    let (nu, nv) = (rows[0].len() as i32, rows[1].len() as i32);
    Ok([
        at(rows[0], 1)?,
        at(rows[1], 1)?,
        at(rows[2], nu)?,
        at(rows[3], nv)?,
    ])
}

/// A point from a per-coordinate function (`P.SetCoord(k, ...)` for k = 1..3).
pub(super) fn point_from(coord: impl Fn(usize) -> f64) -> Point3 {
    Point3::new(coord(0), coord(1), coord(2))
}

/// The coordinates of a point as an array (`P.Coord(k)` for k = 1..3).
pub(super) fn coords(p: Point3) -> [f64; 3] {
    [p.x(), p.y(), p.z()]
}
