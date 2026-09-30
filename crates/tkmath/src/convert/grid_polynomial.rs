// Rust translation of FoundationClasses/TKMath/Convert/Convert_GridPolynomialToPoles.hxx/.cxx (constructors and accessors) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! A grid of polynomial surface patches converted to one B-spline surface.
use crate::gp::Point3;
use crate::plib::access::{array_value, grid_value};
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Failure;

/// Poles, knots and multiplicities of the B-spline surface equivalent to a grid
/// of polynomial patches (`Convert_GridPolynomialToPoles`); coefficients are the C
/// array `[NbVSurfaces][NbUSurfaces][MaxUDegree + 1][MaxVDegree + 1][3]`;
/// `GridPolynomialToPoles::from_single_patch(1, 1, &counts, &coeffs, &u, &v)?.is_done()`.
pub struct GridPolynomialToPoles {
    pub(super) u_flat_knots: Array1<'static, f64>,
    pub(super) v_flat_knots: Array1<'static, f64>,
    pub(super) u_knots: Array1<'static, f64>,
    pub(super) v_knots: Array1<'static, f64>,
    pub(super) u_mults: Array1<'static, i32>,
    pub(super) v_mults: Array1<'static, i32>,
    pub(super) poles: Array2<'static, Point3>,
    pub(super) u_degree: i32,
    pub(super) v_degree: i32,
    pub(super) done: bool,
}

/// The grid description of the general constructor: patch counts, continuities,
/// maximal degrees, coefficient counts `[1..NbU*NbV][1..2]`, coefficients,
/// polynomial domains and true breakpoints in U and V;
/// `PolynomialGrid { nb_u_surfaces: 1, nb_v_surfaces: 1, u_continuity: 0, ..grid }`.
#[derive(Clone, Copy)]
pub struct PolynomialGrid<'r, 'a> {
    /// Number of patches along U.
    pub nb_u_surfaces: i32,
    /// Number of patches along V.
    pub nb_v_surfaces: i32,
    /// Continuity across U breakpoints.
    pub u_continuity: i32,
    /// Continuity across V breakpoints.
    pub v_continuity: i32,
    /// Maximal U degree of the coefficient layout.
    pub max_u_degree: i32,
    /// Maximal V degree of the coefficient layout.
    pub max_v_degree: i32,
    /// Coefficient counts (degree + 1) in U and V of every patch.
    pub num_coeff_per_surface: &'r Array2<'a, i32>,
    /// Canonical coefficients.
    pub coefficients: &'r Array1<'a, f64>,
    /// Polynomial domain in U (two bounds).
    pub polynomial_u_intervals: &'r Array1<'a, f64>,
    /// Polynomial domain in V (two bounds).
    pub polynomial_v_intervals: &'r Array1<'a, f64>,
    /// U breakpoints of the composite surface.
    pub true_u_intervals: &'r Array1<'a, f64>,
    /// V breakpoints of the composite surface.
    pub true_v_intervals: &'r Array1<'a, f64>,
}

fn domain(message: &str) -> Failure {
    Failure::with_type("Standard_DomainError", &format!("Convert : {message}"), "")
}

impl GridPolynomialToPoles {
    fn unconverted() -> Self {
        Self {
            u_flat_knots: Array1::default(),
            v_flat_knots: Array1::default(),
            u_knots: Array1::default(),
            v_knots: Array1::default(),
            u_mults: Array1::default(),
            v_mults: Array1::default(),
            poles: Array2::default(),
            u_degree: 0,
            v_degree: 0,
            done: false,
        }
    }

    /// One polynomial patch on `polynomial_u_intervals` x `polynomial_v_intervals`
    /// (the array and handle overloads); `num_coeff` must be `[1..2]` and the
    /// coefficients `[1..3 * (MaxU + 1) * (MaxV + 1)]`;
    /// `GridPolynomialToPoles::from_single_patch(1, 1, &counts, &coeffs, &u, &v)?`.
    pub fn from_single_patch(
        max_u_degree: i32,
        max_v_degree: i32,
        num_coeff: &Array1<'_, i32>,
        coefficients: &Array1<'_, f64>,
        polynomial_u_intervals: &Array1<'_, f64>,
        polynomial_v_intervals: &Array1<'_, f64>,
    ) -> Result<Self, Failure> {
        if num_coeff.lower() != 1 || num_coeff.upper() != 2 {
            return Err(domain(&format!(
                "Wrong Coefficients: counts {}..{}, expected 1..2",
                num_coeff.lower(),
                num_coeff.upper()
            )));
        }
        let size: i64 = 3 * (i64::from(max_u_degree) + 1) * (i64::from(max_v_degree) + 1);
        check_coefficients(coefficients, size)?;
        let mut result = Self::unconverted();
        result.u_degree = array_value(num_coeff, 1)? - 1;
        result.v_degree = array_value(num_coeff, 2)? - 1;
        result.check_degrees(max_u_degree, max_v_degree)?;
        let mut counts: Array2<'static, i32> =
            Array2::with_bounds(1, 1, 1, 2).map_err(|e| domain(&e.to_string()))?;
        counts[(1, 1)] = num_coeff[1];
        counts[(1, 2)] = num_coeff[2];
        let grid = PolynomialGrid {
            nb_u_surfaces: 1,
            nb_v_surfaces: 1,
            u_continuity: 0,
            v_continuity: 0,
            max_u_degree,
            max_v_degree,
            num_coeff_per_surface: &counts,
            coefficients,
            polynomial_u_intervals,
            polynomial_v_intervals,
            true_u_intervals: polynomial_u_intervals,
            true_v_intervals: polynomial_v_intervals,
        };
        result.perform(&grid)?;
        Ok(result)
    }

    /// A grid of patches joined with the given continuities (the array and handle
    /// overloads); the maximal degrees are raised to `2 * continuity + 1`;
    /// `GridPolynomialToPoles::from_grid(&grid)?`.
    pub fn from_grid(grid: &PolynomialGrid<'_, '_>) -> Result<Self, Failure> {
        let real_u_degree: i32 = grid.max_u_degree.max(2 * grid.u_continuity + 1);
        let real_v_degree: i32 = grid.max_v_degree.max(2 * grid.v_continuity + 1);
        let patches: i64 = i64::from(grid.nb_u_surfaces) * i64::from(grid.nb_v_surfaces);
        let counts: &Array2<'_, i32> = grid.num_coeff_per_surface;
        if counts.lower_row() != 1
            || i64::from(counts.upper_row()) != patches
            || counts.lower_col() != 1
            || counts.upper_col() != 2
        {
            return Err(domain(&format!(
                "Wrong NumCoeffPerSurface: rows {}..{}, columns {}..{}, expected 1..{patches} x 1..2",
                counts.lower_row(),
                counts.upper_row(),
                counts.lower_col(),
                counts.upper_col()
            )));
        }
        let size: i64 =
            3 * patches * (i64::from(real_u_degree) + 1) * (i64::from(real_v_degree) + 1);
        check_coefficients(grid.coefficients, size)?;
        let mut result = Self::unconverted();
        // Compute actual degrees from the supplied coefficient counts.
        for ii in 1..=patches as i32 {
            result.u_degree = raised_degree(result.u_degree, grid_value(counts, ii, 1)?);
            result.v_degree = raised_degree(result.v_degree, grid_value(counts, ii, 2)?);
        }
        result.check_degrees(real_u_degree, real_v_degree)?;
        let real_grid = PolynomialGrid {
            max_u_degree: real_u_degree,
            max_v_degree: real_v_degree,
            ..*grid
        };
        result.perform(&real_grid)?;
        Ok(result)
    }

    fn check_degrees(&self, max_u_degree: i32, max_v_degree: i32) -> Result<(), Failure> {
        if self.u_degree > max_u_degree {
            return Err(domain(&format!(
                "Incoherence between NumCoeffPerSurface and MaxUDegree: {} > {max_u_degree}",
                self.u_degree
            )));
        }
        if self.v_degree > max_v_degree {
            return Err(domain(&format!(
                "Incoherence between NumCoeffPerSurface and MaxVDegree: {} > {max_v_degree}",
                self.v_degree
            )));
        }
        Ok(())
    }

    fn check_done(&self) -> Result<(), Failure> {
        if self.done {
            return Ok(());
        }
        Err(Failure::with_type(
            "StdFail_NotDone",
            "GridPolynomialToPoles: the conversion is not done",
            "",
        ))
    }

    /// Number of poles in U; `conversion.nb_u_poles()?`.
    pub fn nb_u_poles(&self) -> Result<i32, Failure> {
        self.check_done()?;
        Ok(self.poles.nb_rows() as i32)
    }

    /// Number of poles in V; `conversion.nb_v_poles()?`.
    pub fn nb_v_poles(&self) -> Result<i32, Failure> {
        self.check_done()?;
        Ok(self.poles.nb_columns() as i32)
    }

    /// Poles, rows along U; `conversion.poles()?[(1, 1)]`.
    pub fn poles(&self) -> Result<&Array2<'static, Point3>, Failure> {
        self.check_done()?;
        Ok(&self.poles)
    }

    /// Degree in U; `conversion.u_degree()?`.
    pub fn u_degree(&self) -> Result<i32, Failure> {
        self.check_done()?;
        Ok(self.u_degree)
    }

    /// Degree in V; `conversion.v_degree()?`.
    pub fn v_degree(&self) -> Result<i32, Failure> {
        self.check_done()?;
        Ok(self.v_degree)
    }

    /// Number of knots in U; `conversion.nb_u_knots()?`.
    pub fn nb_u_knots(&self) -> Result<i32, Failure> {
        self.check_done()?;
        Ok(self.u_knots.len() as i32)
    }

    /// Number of knots in V; `conversion.nb_v_knots()?`.
    pub fn nb_v_knots(&self) -> Result<i32, Failure> {
        self.check_done()?;
        Ok(self.v_knots.len() as i32)
    }

    /// U knots (with the bounds of the true U breakpoints); `conversion.u_knots()?`.
    pub fn u_knots(&self) -> Result<&Array1<'static, f64>, Failure> {
        self.check_done()?;
        Ok(&self.u_knots)
    }

    /// V knots (with the bounds of the true V breakpoints); `conversion.v_knots()?`.
    pub fn v_knots(&self) -> Result<&Array1<'static, f64>, Failure> {
        self.check_done()?;
        Ok(&self.v_knots)
    }

    /// U multiplicities; `conversion.u_multiplicities()?`.
    pub fn u_multiplicities(&self) -> Result<&Array1<'static, i32>, Failure> {
        self.check_done()?;
        Ok(&self.u_mults)
    }

    /// V multiplicities; `conversion.v_multiplicities()?`.
    pub fn v_multiplicities(&self) -> Result<&Array1<'static, i32>, Failure> {
        self.check_done()?;
        Ok(&self.v_mults)
    }

    /// True when the interpolation succeeded; `conversion.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
}

// The coefficient array must span exactly 1..size.
fn check_coefficients(coefficients: &Array1<'_, f64>, size: i64) -> Result<(), Failure> {
    if coefficients.lower() != 1 || i64::from(coefficients.upper()) != size {
        return Err(domain(&format!(
            "Wrong Coefficients: {}..{}, expected 1..{size}",
            coefficients.lower(),
            coefficients.upper()
        )));
    }
    Ok(())
}

// `if (count > degree + 1) degree = count - 1`.
fn raised_degree(degree: i32, count: i32) -> i32 {
    if count > degree + 1 {
        count - 1
    } else {
        degree
    }
}
