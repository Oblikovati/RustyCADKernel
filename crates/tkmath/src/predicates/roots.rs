// SPDX-License-Identifier: GPL-2.0-only
//! Exact supplied-polynomial classification; no mapped OCCT solver is replaced (ADR 0004).
mod bernstein;
/// Exact parameter contacts between a planar Bézier curve and an infinite line.
pub mod bezier_line;
mod chain;
mod dyadic;
mod error;
mod isolate;
mod polynomial;
/// Exact signs of a second polynomial at each isolated source root.
pub mod signs;
pub use dyadic::DyadicBound;
pub use error::RootIsolationError;

/// Explicit approximation and work policy. Width is not a modelling tolerance.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RootIsolationOptions {
    /// Policy version; this implementation accepts version 1.
    pub version: u32,
    /// Largest allowed exact bracket width; finite and strictly positive.
    pub max_width: f64,
    /// Maximum number of subdivisions for the complete request.
    pub max_bisections: u16,
    /// Maximum admitted polynomial degree after exact basis conversion and reduction.
    pub max_degree: u16,
}
impl RootIsolationOptions {
    /// Select version-1 work limits with an explicit approximation width.
    /// Example: `RootIsolationOptions::new(0.001)` admits degree 32 and 4096 subdivisions.
    pub fn new(max_width: f64) -> Self {
        Self {
            version: 1,
            max_width,
            max_bisections: 4096,
            max_degree: 32,
        }
    }
}
/// A closed interval containing exactly one distinct root, with its exact multiplicity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RootBracket {
    lower: DyadicBound,
    upper: DyadicBound,
    multiplicity: usize,
}
impl RootBracket {
    /// Read certified exact bounds; example: `root.bounds().0.compare_f64(0.)?`.
    pub fn bounds(&self) -> (&DyadicBound, &DyadicBound) {
        (&self.lower, &self.upper)
    }
    /// Read the exact algebraic multiplicity, including tangencies; example: `root.multiplicity() == 2`.
    pub fn multiplicity(&self) -> usize {
        self.multiplicity
    }
    /// Obtain measured exact bracket width; example: `root.width().compare_f64(options.max_width)?`.
    pub fn width(&self) -> DyadicBound {
        self.upper.difference(&self.lower)
    }
}
/// A complete ordered list; failures never expose a partial list as successful isolation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RootIsolation {
    roots: Vec<RootBracket>,
    bisections: u16,
}
impl RootIsolation {
    /// Borrow every distinct root in increasing order; example: `isolation.roots().len()`.
    pub fn roots(&self) -> &[RootBracket] {
        &self.roots
    }
    /// Read the subdivision work used; example: `isolation.bisections() <= options.max_bisections`.
    pub fn bisections(&self) -> u16 {
        self.bisections
    }
}
/// Isolate all distinct real roots on a closed finite interval, using ascending coefficients.
///
/// Signs are exact for supplied binary64 values. Curve-to-polynomial construction
/// error is outside this contract. Cancellation is polled between integer operations.
/// ```
/// use rustycad_tkmath::predicates::roots::{isolate_polynomial_roots, RootIsolationOptions};
/// let options = RootIsolationOptions::new(0.01);
/// let roots = isolate_polynomial_roots(&[-2., 0., 1.], [-2., 2.], options, || false)?;
/// assert_eq!(roots.roots().len(), 2);
/// # Ok::<(), rustycad_tkmath::predicates::roots::RootIsolationError>(())
/// ```
pub fn isolate_polynomial_roots(
    coefficients: &[f64],
    interval: [f64; 2],
    options: RootIsolationOptions,
    cancelled: impl FnMut() -> bool,
) -> Result<RootIsolation, RootIsolationError> {
    isolate_basis(coefficients, interval, options, Basis::Power, cancelled)
}

/// Isolate roots of a scalar Bernstein polynomial with exact supplied control values.
///
/// The basis is `B(i,n,t) = choose(n,i) * t^i * (1-t)^(n-i)`, even when the query
/// interval is outside `[0,1]`. Bounds do not rescale the basis. Products used to
/// construct control values (such as rational weights) are outside this contract.
/// ```
/// use rustycad_tkmath::predicates::roots::{isolate_bernstein_roots, RootIsolationOptions};
/// let roots = isolate_bernstein_roots(&[1., -1., 1.], [0., 1.],
///     RootIsolationOptions::new(0.001), || false)?;
/// assert_eq!(roots.roots()[0].multiplicity(), 2);
/// # Ok::<(), rustycad_tkmath::predicates::roots::RootIsolationError>(())
/// ```
pub fn isolate_bernstein_roots(
    controls: &[f64],
    interval: [f64; 2],
    options: RootIsolationOptions,
    cancelled: impl FnMut() -> bool,
) -> Result<RootIsolation, RootIsolationError> {
    isolate_basis(controls, interval, options, Basis::Bernstein, cancelled)
}

enum Basis {
    Power,
    Bernstein,
}
impl Basis {
    fn convert(
        &self,
        coefficients: &[f64],
        limit: u16,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<polynomial::IntegerPolynomial, RootIsolationError> {
        match self {
            Self::Power => polynomial::IntegerPolynomial::from_binary64(coefficients, limit),
            Self::Bernstein => bernstein::from_controls(coefficients, limit, cancelled),
        }
    }
}
fn isolate_basis(
    coefficients: &[f64],
    interval: [f64; 2],
    options: RootIsolationOptions,
    basis: Basis,
    mut cancelled: impl FnMut() -> bool,
) -> Result<RootIsolation, RootIsolationError> {
    let (bounds, width) = checked_policy_interval(interval, options)?;
    let polynomial = basis.convert(coefficients, options.max_degree, &mut cancelled)?;
    let tower = chain::multiplicity_tower(polynomial, &mut cancelled)?;
    isolate::isolate(&tower, bounds, width, options, &mut cancelled)
}
fn checked_policy_interval(
    interval: [f64; 2],
    options: RootIsolationOptions,
) -> Result<([DyadicBound; 2], DyadicBound), RootIsolationError> {
    if options.version != 1 {
        return Err(RootIsolationError::OptionsVersion(options.version));
    }
    checked_interval(interval, options.max_width)
}
fn checked_interval(
    interval: [f64; 2],
    max_width: f64,
) -> Result<([DyadicBound; 2], DyadicBound), RootIsolationError> {
    if !interval.iter().all(|value| value.is_finite()) || interval[0] > interval[1] {
        return Err(RootIsolationError::Interval(interval));
    }
    if !max_width.is_finite() || max_width <= 0. {
        return Err(RootIsolationError::Width(max_width));
    }
    let convert =
        |value| DyadicBound::from_f64(value).map_err(|_| RootIsolationError::Interval(interval));
    let width =
        DyadicBound::from_f64(max_width).map_err(|_| RootIsolationError::Width(max_width))?;
    Ok(([convert(interval[0])?, convert(interval[1])?], width))
}
