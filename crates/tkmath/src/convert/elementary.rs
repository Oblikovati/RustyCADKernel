// Rust translation of FoundationClasses/TKMath/Convert/Convert_ElementarySurfaceToBSplineSurface.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Storage and accessors of the B-spline data computed for an elementary surface.
use crate::gp::{Axis3, Point3, Transform3};
use crate::plib::access::grid_cell;
use rustycad_tkernel::ncollection::{Array1, Array2, ArrayError};
use rustycad_tkernel::standard::Failure;

/// Degrees, periodicity, poles, weights, knots and multiplicities of the B-spline
/// equivalent to a cylinder, cone, sphere or torus
/// (`Convert_ElementarySurfaceToBSplineSurface` and its subclasses);
/// `ElementarySurfaceToBSplineSurface::from_sphere(&sphere)?.nb_u_poles() == 6`.
pub struct ElementarySurfaceToBSplineSurface {
    pub(super) poles: Array2<'static, Point3>,
    pub(super) weights: Array2<'static, f64>,
    pub(super) u_knots: Array1<'static, f64>,
    pub(super) v_knots: Array1<'static, f64>,
    pub(super) u_mults: Array1<'static, i32>,
    pub(super) v_mults: Array1<'static, i32>,
    pub(super) u_degree: i32,
    pub(super) v_degree: i32,
    pub(super) u_periodic: bool,
    pub(super) v_periodic: bool,
    pub(super) nb_u_poles: i32,
    pub(super) nb_v_poles: i32,
    pub(super) nb_u_knots: i32,
    pub(super) nb_v_knots: i32,
}

/// The maximal array sizes and the degrees the subclasses allocate with;
/// `SurfaceSizes { u_poles: 7, v_poles: 5, u_knots: 4, v_knots: 3, u_degree: 2, v_degree: 2 }`.
#[derive(Clone, Copy, Debug)]
pub(super) struct SurfaceSizes {
    pub(super) u_poles: i32,
    pub(super) v_poles: i32,
    pub(super) u_knots: i32,
    pub(super) v_knots: i32,
    pub(super) u_degree: i32,
    pub(super) v_degree: i32,
}

impl SurfaceSizes {
    /// Construction limits for an elementary-surface conversion; `SurfaceSizes::new(4, 3, 3, 2, 2, 1)`.
    pub(super) const fn new(
        u_poles: i32,
        v_poles: i32,
        u_knots: i32,
        v_knots: i32,
        u_degree: i32,
        v_degree: i32,
    ) -> Self {
        Self {
            u_poles,
            v_poles,
            u_knots,
            v_knots,
            u_degree,
            v_degree,
        }
    }
}

fn allocation(error: ArrayError) -> Failure {
    Failure::with_type(
        "Standard_OutOfMemory",
        &format!("Convert_ElementarySurfaceToBSplineSurface: {error}"),
        "",
    )
}

fn out_of_range(accessor: &str, index: i32, count: i32) -> Failure {
    Failure::out_of_range(&format!(
        "Convert_ElementarySurfaceToBSplineSurface::{accessor}: Index out of range: {index} not in 1..={count}"
    ))
}

fn checked(accessor: &str, index: i32, count: i32) -> Result<(), Failure> {
    if index < 1 || index > count {
        return Err(out_of_range(accessor, index, count));
    }
    Ok(())
}

impl ElementarySurfaceToBSplineSurface {
    /// Allocates maximum pole/knot storage for a custom elementary-surface conversion;
    /// `ElementarySurfaceToBSplineSurface::with_capacity(4, 3, 3, 2, 2, 1)?`.
    pub fn with_capacity(
        u_poles: i32,
        v_poles: i32,
        u_knots: i32,
        v_knots: i32,
        u_degree: i32,
        v_degree: i32,
    ) -> Result<Self, Failure> {
        Self::with_sizes(SurfaceSizes::new(
            u_poles, v_poles, u_knots, v_knots, u_degree, v_degree,
        ))
    }

    /// The protected base constructor with 1-based arrays of the maximal sizes;
    /// `ElementarySurfaceToBSplineSurface::with_sizes(sizes)?`.
    pub(super) fn with_sizes(sizes: SurfaceSizes) -> Result<Self, Failure> {
        Ok(Self {
            poles: Array2::with_bounds(1, sizes.u_poles, 1, sizes.v_poles).map_err(allocation)?,
            weights: Array2::with_bounds(1, sizes.u_poles, 1, sizes.v_poles).map_err(allocation)?,
            u_knots: Array1::with_bounds(1, sizes.u_knots).map_err(allocation)?,
            v_knots: Array1::with_bounds(1, sizes.v_knots).map_err(allocation)?,
            u_mults: Array1::with_bounds(1, sizes.u_knots).map_err(allocation)?,
            v_mults: Array1::with_bounds(1, sizes.v_knots).map_err(allocation)?,
            u_degree: sizes.u_degree,
            v_degree: sizes.v_degree,
            u_periodic: false,
            v_periodic: false,
            nb_u_poles: sizes.u_poles,
            nb_v_poles: sizes.v_poles,
            nb_u_knots: sizes.u_knots,
            nb_v_knots: sizes.v_knots,
        })
    }

    /// Sets the active counts before finalization; `surface.set_actual_counts(2, 2, 2, 1)`.
    pub fn set_actual_counts(&mut self, u_poles: i32, v_poles: i32, u_knots: i32, v_knots: i32) {
        (self.nb_u_poles, self.nb_v_poles) = (u_poles, v_poles);
        (self.nb_u_knots, self.nb_v_knots) = (u_knots, v_knots);
    }

    /// Sets the output degrees; `surface.set_degrees(2, 1)`.
    pub fn set_degrees(&mut self, u_degree: i32, v_degree: i32) {
        (self.u_degree, self.v_degree) = (u_degree, v_degree);
    }

    /// Sets output periodicity; `surface.set_periodicity(true, false)`.
    pub fn set_periodicity(&mut self, u_periodic: bool, v_periodic: bool) {
        (self.u_periodic, self.v_periodic) = (u_periodic, v_periodic);
    }

    /// Mutable pole storage before finalization; `surface.poles_mut()[(1, 1)] = point`.
    pub fn poles_mut(&mut self) -> &mut Array2<'static, Point3> {
        &mut self.poles
    }

    /// Mutable weight storage before finalization; `surface.weights_mut()[(1, 1)] = 1.0`.
    pub fn weights_mut(&mut self) -> &mut Array2<'static, f64> {
        &mut self.weights
    }

    /// Mutable U-knot storage before finalization; `surface.u_knots_mut()[1] = 0.0`.
    pub fn u_knots_mut(&mut self) -> &mut Array1<'static, f64> {
        &mut self.u_knots
    }

    /// Mutable V-knot storage before finalization; `surface.v_knots_mut()[1] = 0.0`.
    pub fn v_knots_mut(&mut self) -> &mut Array1<'static, f64> {
        &mut self.v_knots
    }

    /// Mutable U multiplicities before finalization; `surface.u_multiplicities_mut()[1] = 3`.
    pub fn u_multiplicities_mut(&mut self) -> &mut Array1<'static, i32> {
        &mut self.u_mults
    }

    /// Mutable V multiplicities before finalization; `surface.v_multiplicities_mut()[1] = 2`.
    pub fn v_multiplicities_mut(&mut self) -> &mut Array1<'static, i32> {
        &mut self.v_mults
    }

    /// Shrinks arrays to active counts in native order, preserving elements;
    /// `surface.finalize()?`.
    pub fn finalize(&mut self) -> Result<(), Failure> {
        resize_grid(&mut self.poles, self.nb_u_poles, self.nb_v_poles)?;
        resize_grid(&mut self.weights, self.nb_u_poles, self.nb_v_poles)?;
        self.trim_knots()
    }

    fn trim_knots(&mut self) -> Result<(), Failure> {
        resize_knots(&mut self.u_knots, self.nb_u_knots)?;
        resize_knots(&mut self.u_mults, self.nb_u_knots)?;
        resize_knots(&mut self.v_knots, self.nb_v_knots)?;
        resize_knots(&mut self.v_mults, self.nb_v_knots)
    }

    /// Pole and knot counts of `nb_u_spans` x `nb_v_spans` quadratic spans;
    /// `surface.set_counts(2, 1)`.
    pub(super) fn set_counts(&mut self, nb_u_spans: i32, nb_v_spans: i32) {
        self.nb_u_poles = nb_u_spans.wrapping_mul(2).wrapping_add(1);
        self.nb_v_poles = nb_v_spans.wrapping_mul(2).wrapping_add(1);
        self.nb_u_knots = nb_u_spans.wrapping_add(1);
        self.nb_v_knots = nb_v_spans.wrapping_add(1);
    }

    /// Weights every pole row `i` by `w1(i) * w2(j)` (`i`, `j` even take the
    /// half-angle cosines) and moves the poles from the XOY frame into `position`;
    /// `surface.weigh_and_place(position, (0.5, 1.0))?`.
    pub(super) fn weigh_and_place(
        &mut self,
        position: Axis3,
        cosines: (f64, f64),
    ) -> Result<(), Failure> {
        let trsf: Transform3 = Transform3::transformation_between(position, Axis3::default());
        for i in 1..=self.nb_u_poles {
            let w1: f64 = if i % 2 == 0 { cosines.0 } else { 1. };
            for j in 1..=self.nb_v_poles {
                let w2: f64 = if j % 2 == 0 { cosines.1 } else { 1. };
                *grid_cell(&mut self.weights, i, j)? = w1 * w2;
                let pole: &mut Point3 = grid_cell(&mut self.poles, i, j)?;
                *pole = pole.transformed(trsf);
            }
        }
        Ok(())
    }

    /// Degree in U; `surface.u_degree()`.
    pub fn u_degree(&self) -> i32 {
        self.u_degree
    }

    /// Degree in V; `surface.v_degree()`.
    pub fn v_degree(&self) -> i32 {
        self.v_degree
    }

    /// Number of poles in U; `surface.nb_u_poles()`.
    pub fn nb_u_poles(&self) -> i32 {
        self.nb_u_poles
    }

    /// Number of poles in V; `surface.nb_v_poles()`.
    pub fn nb_v_poles(&self) -> i32 {
        self.nb_v_poles
    }

    /// Number of knots in U; `surface.nb_u_knots()`.
    pub fn nb_u_knots(&self) -> i32 {
        self.nb_u_knots
    }

    /// Number of knots in V; `surface.nb_v_knots()`.
    pub fn nb_v_knots(&self) -> i32 {
        self.nb_v_knots
    }

    /// Periodicity in U; `surface.is_u_periodic()`.
    pub fn is_u_periodic(&self) -> bool {
        self.u_periodic
    }

    /// Periodicity in V; `surface.is_v_periodic()`.
    pub fn is_v_periodic(&self) -> bool {
        self.v_periodic
    }

    /// Pole of one-based indices (deprecated `Pole`); `surface.pole(1, 1)?`.
    pub fn pole(&self, u_index: i32, v_index: i32) -> Result<Point3, Failure> {
        checked("Pole", u_index, self.nb_u_poles)?;
        checked("Pole", v_index, self.nb_v_poles)?;
        Ok(self.poles[(u_index, v_index)])
    }

    /// Weight of one-based indices (deprecated `Weight`); `surface.weight(1, 1)?`.
    pub fn weight(&self, u_index: i32, v_index: i32) -> Result<f64, Failure> {
        checked("Weight", u_index, self.nb_u_poles)?;
        checked("Weight", v_index, self.nb_v_poles)?;
        Ok(self.weights[(u_index, v_index)])
    }

    /// U knot of one-based index (deprecated `UKnot`); `surface.u_knot(1)?`.
    pub fn u_knot(&self, u_index: i32) -> Result<f64, Failure> {
        checked("UKnot", u_index, self.nb_u_knots)?;
        Ok(self.u_knots[u_index])
    }

    /// V knot of one-based index (deprecated `VKnot`); `surface.v_knot(1)?`.
    pub fn v_knot(&self, v_index: i32) -> Result<f64, Failure> {
        checked("VKnot", v_index, self.nb_v_knots)?;
        Ok(self.v_knots[v_index])
    }

    /// U multiplicity of one-based index (deprecated `UMultiplicity`); `surface.u_multiplicity(1)?`.
    pub fn u_multiplicity(&self, u_index: i32) -> Result<i32, Failure> {
        checked("UMultiplicity", u_index, self.nb_u_knots)?;
        Ok(self.u_mults[u_index])
    }

    /// V multiplicity of one-based index (deprecated `VMultiplicity`); `surface.v_multiplicity(1)?`.
    pub fn v_multiplicity(&self, v_index: i32) -> Result<i32, Failure> {
        checked("VMultiplicity", v_index, self.nb_v_knots)?;
        Ok(self.v_mults[v_index])
    }

    /// All poles, rows along U; `surface.poles()[(1, 1)]`.
    pub fn poles(&self) -> &Array2<'static, Point3> {
        &self.poles
    }

    /// All weights; `surface.weights()[(1, 1)]`.
    pub fn weights(&self) -> &Array2<'static, f64> {
        &self.weights
    }

    /// U knots; `surface.u_knots()[1]`.
    pub fn u_knots(&self) -> &Array1<'static, f64> {
        &self.u_knots
    }

    /// V knots; `surface.v_knots()[1]`.
    pub fn v_knots(&self) -> &Array1<'static, f64> {
        &self.v_knots
    }

    /// U multiplicities; `surface.u_multiplicities()[1]`.
    pub fn u_multiplicities(&self) -> &Array1<'static, i32> {
        &self.u_mults
    }

    /// V multiplicities; `surface.v_multiplicities()[1]`.
    pub fn v_multiplicities(&self) -> &Array1<'static, i32> {
        &self.v_mults
    }
}

fn resize_knots<T: Default + Clone>(
    array: &mut Array1<'static, T>,
    count: i32,
) -> Result<(), Failure> {
    if array.len() as i64 == i64::from(count) {
        return Ok(());
    }
    array.resize_bounds(1, count, true).map_err(allocation)
}

fn resize_grid<T: Default + Clone>(
    array: &mut Array2<'static, T>,
    rows: i32,
    columns: i32,
) -> Result<(), Failure> {
    if array.nb_rows() as i64 == i64::from(rows) && array.nb_columns() as i64 == i64::from(columns)
    {
        return Ok(());
    }
    // The native ResizeWithTrim keeps each value at its original (row, column).
    array
        .resize_trim_bounds(1, rows, 1, columns, true)
        .map_err(allocation)
}
