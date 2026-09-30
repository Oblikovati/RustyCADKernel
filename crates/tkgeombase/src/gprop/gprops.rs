// Rust translation of ModelingData/TKGeomBase/GProp/GProp_GProps.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GProp_GProps`: the global properties of a compound geometric system.
use super::{PrincipalProps, h_operator};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{self, Axis1, Matrix3, Point3, Vector3, Xyz};
use rustycad_tkmath::math::{Jacobi, Matrix};

/// `GProp_GProps`: mass, centre of mass and inertia accumulated relative to a
/// system location; components are composed with [`GProps::add`].
///
/// `g` is the centre of mass *relative to* the location (the native protected
/// member), `inertia` the quadratic moments at the location. The protected members are
/// public fields so that the derived frameworks of other toolkits can write them
/// (docs/topalgo-design.md section 9).
///
/// ```
/// use rustycad_tkgeombase::gprop::{GProps, PGProps};
/// use rustycad_tkmath::gp::Point3;
/// let mut points = PGProps::new();
/// points.add_point(Point3::new(2.0, 0.0, 0.0));
/// let mut system = GProps::new();
/// system.add(&points, 3.0).unwrap();
/// assert_eq!(system.mass(), 3.0);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GProps {
    /// `g` (protected natively): the centre of mass relative to `loc`. The frameworks
    /// that derive from `GProp_GProps` in other toolkits (BRepGProp) write it, so it is
    /// public here.
    pub g: Point3,
    /// `loc` (protected natively): the point the inertia is accumulated at.
    pub loc: Point3,
    /// `dim` (protected natively): the mass.
    pub dim: f64,
    /// `inertia` (protected natively): the quadratic moments at `loc`.
    pub inertia: Matrix3,
}

impl GProps {
    /// `GProp_GProps()`: an empty system located at the origin; `GProps::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `GProp_GProps(SystemLocation)`: an empty system accumulating its inertia at
    /// `system_location`; `GProps::with_location(p)`.
    pub fn with_location(system_location: Point3) -> Self {
        Self {
            loc: system_location,
            ..Self::default()
        }
    }

    /// `Add(Item, Density = 1.0)`: compose the properties of `item` with the density
    /// `density`, moving its inertia to this location by Huygens' theorem;
    /// Standard_DomainError when `density <= gp::Resolution()`;
    /// `system.add(&component, 1.0)?`.
    pub fn add(&mut self, item: &GProps, density: f64) -> Result<(), Failure> {
        if density <= gp::resolution() {
            return Err(Failure::with_type(
                "Standard_DomainError",
                &format!("GProp_GProps::Add: Density = {density}; expected > gp::Resolution()"),
                "",
            ));
        }
        if self.loc.distance(&item.loc) <= gp::resolution() {
            self.add_centre(item.g.xyz() * (item.dim * density), item.dim * density);
            self.inertia += item.inertia * density;
            return Ok(());
        }
        let item_loc: Xyz = self.loc.xyz() - item.loc.xyz();
        let item_g: Xyz = item.loc.xyz() + item.g.xyz();
        self.add_centre(
            (item.g.xyz() - item_loc) * (item.dim * density),
            item.dim * density,
        );
        self.inertia += item.inertia_at(Point3::from(item_g), self.loc) * density;
        Ok(())
    }

    // The new relative centre of mass after adding the weighted centre `weighted`
    // of mass `mass`; the origin when the total mass is below 1e-20.
    fn add_centre(&mut self, weighted: Xyz, mass: f64) {
        let mut sum: Xyz = weighted;
        self.g.set_xyz(self.g.xyz() * self.dim);
        sum += self.g.xyz();
        self.dim += mass;
        if self.dim.abs() >= 1.0e-20 {
            sum /= self.dim;
            self.g.set_xyz(sum);
        } else {
            self.g.set_coordinates(0.0, 0.0, 0.0);
        }
    }

    // The inertia of this item moved to `location`: back to the centre of mass
    // `item_g` (skipped when the relative centre is at the item location), then to
    // `location`.
    fn inertia_at(&self, item_g: Point3, location: Point3) -> Matrix3 {
        let mut item_inertia: Matrix3 = self.inertia;
        if self.g.xyz().modulus() > gp::resolution() {
            item_inertia -= h_operator(item_g, self.loc, self.dim);
        }
        item_inertia + h_operator(item_g, location, self.dim)
    }

    /// `Mass()`: the mass (length, area or volume times density); `system.mass()`.
    pub fn mass(&self) -> f64 {
        self.dim
    }

    /// `CentreOfMass()`: the absolute centre of mass; `system.centre_of_mass()`.
    pub fn centre_of_mass(&self) -> Point3 {
        Point3::from(self.loc.xyz() + self.g.xyz())
    }

    /// `MatrixOfInertia()`: the stored inertia minus the Huygens operator of the
    /// *relative* centre with respect to the origin; `system.matrix_of_inertia()`.
    pub fn matrix_of_inertia(&self) -> Matrix3 {
        self.inertia - h_operator(self.g, gp::origin(), self.dim)
    }

    /// `StaticMoments(Ix, Iy, Iz)`: the absolute centre times the mass;
    /// `let (ix, iy, iz) = system.static_moments();`.
    pub fn static_moments(&self) -> (f64, f64, f64) {
        let g: Xyz = self.loc.xyz() + self.g.xyz();
        (g.x() * self.dim, g.y() * self.dim, g.z() * self.dim)
    }

    /// `MomentOfInertia(A)`: the moment of inertia about the axis `axis`;
    /// `system.moment_of_inertia(axis)`.
    pub fn moment_of_inertia(&self, axis: Axis1) -> f64 {
        let direction: Xyz = axis.direction().xyz();
        if self.loc.distance(&axis.location()) <= gp::resolution() {
            return direction.dot(self.inertia * direction);
        }
        let axis_inertia: Matrix3 =
            self.matrix_of_inertia() + h_operator(self.centre_of_mass(), axis.location(), self.dim);
        direction.dot(axis_inertia * direction)
    }

    /// `RadiusOfGyration(A)`: `sqrt(MomentOfInertia(A) / Mass())`;
    /// `system.radius_of_gyration(axis)`.
    pub fn radius_of_gyration(&self, axis: Axis1) -> f64 {
        (self.moment_of_inertia(axis) / self.dim).sqrt()
    }

    /// `PrincipalProperties()`: the eigenvalues and eigenvectors of the matrix of
    /// inertia (math_Jacobi, StdFail_NotDone when it does not converge);
    /// `system.principal_properties()?`.
    pub fn principal_properties(&self) -> Result<PrincipalProps, Failure> {
        let axis_inertia: Matrix3 = self.matrix_of_inertia();
        let mut diag: Matrix<'static> = Matrix::with_bounds(1, 3, 1, 3).unwrap();
        for j in 1..=3 {
            for i in 1..=3 {
                *diag.change_value(i, j)? = axis_inertia.value(i, j)?;
            }
        }
        let jacobi = Jacobi::new(&diag)?;
        let moments: [f64; 3] = [jacobi.value(1)?, jacobi.value(2)?, jacobi.value(3)?];
        let vectors: &Matrix<'static> = jacobi.vectors()?;
        let mut axes: [Vector3; 3] = [Vector3::default(); 3];
        for (k, axis) in axes.iter_mut().enumerate() {
            let col: i32 = k as i32 + 1;
            *axis = Vector3::new(
                *vectors.value(1, col)?,
                *vectors.value(2, col)?,
                *vectors.value(3, col)?,
            );
        }
        let radii: [f64; 3] = moments.map(|moment| self.gyration_radius(moment));
        Ok(PrincipalProps::new(
            moments,
            radii,
            axes,
            self.centre_of_mass(),
        ))
    }

    // `sqrt(|I / dim|)`, 0 for a null mass (a single point).
    fn gyration_radius(&self, moment: f64) -> f64 {
        if self.dim == 0.0 {
            return 0.0;
        }
        (moment / self.dim).abs().sqrt()
    }
}
