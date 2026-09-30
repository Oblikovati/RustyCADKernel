// Rust translation of GeomAPI_PointsToBSplineSurface.hxx/.cxx (the constructors, the
// Interpolate overloads, the forwarding Init and the accessors) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomAPI_PointsToBSplineSurface`: a B-spline surface approximating or interpolating
//! a grid of points (or of heights over a regular XY grid).
use rustycad_tkernel::ncollection::Array2;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomBSplineSurface;
use rustycad_tkgeombase::approx::ParametrizationType;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::Point3;

/// The regular XY grid of the height overloads: `(X0, dX, Y0, dY)`.
pub type HeightGrid = (f64, f64, f64, f64);

/// The approximation settings of the Init overloads: `(DegMin, DegMax, Continuity,
/// Tol3D)` (the native defaults are 3, 8, C2 and 1e-3).
pub type SurfaceFitSettings = (i32, i32, Shape, f64);

/// `GeomAPI_PointsToBSplineSurface`: the B-spline surface of two AppDef passes, first
/// along the columns of the grid (the V direction) then along the poles of those
/// curves (the U direction), by least squares (AppDef_BSplineCompute), interpolation
/// (a negative tolerance) or smoothing (AppDef_Variational). The exceptions of AppDef
/// and Geom are returned as `Err`.
///
/// ```
/// use rustycad_tkgeomalgo::geom_api::PointsToBSplineSurface;
/// use rustycad_tkernel::ncollection::Array2;
/// use rustycad_tkmath::geom_abs::Shape;
/// use rustycad_tkmath::gp::Point3;
/// let mut grid: Array2<'static, Point3> = Array2::with_bounds(1, 4, 1, 4).unwrap();
/// for i in 1..=4 {
///     for j in 1..=4 {
///         grid[(i, j)] = Point3::new(f64::from(i), f64::from(j), f64::from(i * j) / 10.0);
///     }
/// }
/// let fit = PointsToBSplineSurface::with_points(&grid, (3, 8, Shape::C2, 1e-3)).unwrap();
/// assert!(fit.is_done());
/// ```
#[derive(Default)]
pub struct PointsToBSplineSurface {
    pub(super) my_is_done: bool,
    pub(super) my_surface: Handle<GeomBSplineSurface>,
}

impl PointsToBSplineSurface {
    /// `GeomAPI_PointsToBSplineSurface()`: not done. `PointsToBSplineSurface::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `GeomAPI_PointsToBSplineSurface(Points, DegMin, DegMax, Continuity, Tol3D)`.
    /// `PointsToBSplineSurface::with_points(&grid, (3, 8, Shape::C2, 1e-3))?`.
    pub fn with_points(
        points: &Array2<'_, Point3>,
        settings: SurfaceFitSettings,
    ) -> Result<Self, Failure> {
        let mut fit: Self = Self::new();
        fit.init(points, settings)?;
        Ok(fit)
    }
    /// `GeomAPI_PointsToBSplineSurface(Points, ParType, DegMin, DegMax, Continuity,
    /// Tol3D)`. `PointsToBSplineSurface::with_parametrization(&grid,
    /// ParametrizationType::Centripetal, (3, 8, Shape::C2, 1e-3))?`.
    pub fn with_parametrization(
        points: &Array2<'_, Point3>,
        par_type: ParametrizationType,
        settings: SurfaceFitSettings,
    ) -> Result<Self, Failure> {
        let mut fit: Self = Self::new();
        fit.init_parametrization(points, par_type, settings, false)?;
        Ok(fit)
    }
    /// `GeomAPI_PointsToBSplineSurface(Points, Weight1, Weight2, Weight3, DegMax,
    /// Continuity, Tol3D)`: the variational smoothing.
    /// `PointsToBSplineSurface::with_weights(&grid, (1.0, 0.0, 0.0), 8, Shape::C2, 1e-3)?`.
    pub fn with_weights(
        points: &Array2<'_, Point3>,
        weights: (f64, f64, f64),
        deg_max: i32,
        continuity: Shape,
        tol3d: f64,
    ) -> Result<Self, Failure> {
        let mut fit: Self = Self::new();
        fit.init_weights(points, weights, deg_max, continuity, tol3d)?;
        Ok(fit)
    }
    /// `GeomAPI_PointsToBSplineSurface(ZPoints, X0, dX, Y0, dY, DegMin, DegMax,
    /// Continuity, Tol3D)`: heights over a regular grid.
    /// `PointsToBSplineSurface::with_heights(&z, (0.0, 1.0, 0.0, 1.0), (3, 8, Shape::C2, 1e-3))?`.
    pub fn with_heights(
        z_points: &Array2<'_, f64>,
        grid: HeightGrid,
        settings: SurfaceFitSettings,
    ) -> Result<Self, Failure> {
        let mut fit: Self = Self::new();
        fit.init_heights(z_points, grid, settings)?;
        Ok(fit)
    }

    /// `Init(Points, DegMin, DegMax, Continuity, Tol3D)`: chord-length
    /// parametrization, not periodic. `fit.init(&grid, (3, 8, Shape::C2, 1e-3))?`.
    pub fn init(
        &mut self,
        points: &Array2<'_, Point3>,
        settings: SurfaceFitSettings,
    ) -> Result<(), Failure> {
        self.init_parametrization(points, ParametrizationType::ChordLength, settings, false)
    }
    /// `Interpolate(Points, thePeriodic)`: the chord-length interpolation (degree 3,
    /// C2, a negative tolerance). `fit.interpolate(&grid, false)?`.
    pub fn interpolate(
        &mut self,
        points: &Array2<'_, Point3>,
        the_periodic: bool,
    ) -> Result<(), Failure> {
        self.interpolate_parametrization(points, ParametrizationType::ChordLength, the_periodic)
    }
    /// `Interpolate(Points, ParType, thePeriodic)`: degree 3, C2, tolerance -1.
    /// `fit.interpolate_parametrization(&grid, ParametrizationType::Centripetal, false)?`.
    pub fn interpolate_parametrization(
        &mut self,
        points: &Array2<'_, Point3>,
        par_type: ParametrizationType,
        the_periodic: bool,
    ) -> Result<(), Failure> {
        self.init_parametrization(points, par_type, (3, 3, Shape::C2, -1.0), the_periodic)
    }
    /// `Interpolate(ZPoints, X0, dX, Y0, dY)`: degree 3, C2, tolerance -1.
    /// `fit.interpolate_heights(&z, (0.0, 1.0, 0.0, 1.0))?`.
    pub fn interpolate_heights(
        &mut self,
        z_points: &Array2<'_, f64>,
        grid: HeightGrid,
    ) -> Result<(), Failure> {
        self.init_heights(z_points, grid, (3, 3, Shape::C2, -1.0))
    }

    /// `Surface()`: StdFail_NotDone when the approximation failed. `fit.surface()?`.
    pub fn surface(&self) -> Result<Handle<GeomBSplineSurface>, Failure> {
        if !self.my_is_done {
            return Err(crate::geom2d_api::not_done(
                "GeomAPI_PointsToBSplineSurface::Surface",
            ));
        }
        Ok(self.my_surface.clone())
    }
    /// `operator occ::handle<Geom_BSplineSurface>()`: Surface(), which raises
    /// StdFail_NotDone when the approximation failed. `fit.as_surface()?`.
    pub fn as_surface(&self) -> Result<Handle<GeomBSplineSurface>, Failure> {
        self.surface()
    }
    /// `IsDone()`. `fit.is_done()`.
    pub fn is_done(&self) -> bool {
        self.my_is_done
    }
}
