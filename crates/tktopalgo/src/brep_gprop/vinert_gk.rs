// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepGProp/BRepGProp_VinertGK.hxx/.cxx
// (the class and its Perform members) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGProp_VinertGK`: the volume properties of the region between a face and a
//! point or a plane by adaptive Gauss-Kronrod integration. The constructors and the
//! integration itself are in `vinert_gk_perform.rs`.
use super::domain::BRepGPropDomain;
use super::face::BRepGPropFace;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::gp::{Plane3, Point3};
use std::ops::Deref;

/// The options of every Perform: the relative tolerance (`theTolerance`, 0.001 by
/// default natively), and whether to compute the centre (`theCGFlag`) and the inertia
/// (`theIFlag`, which implies the centre).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VinertGKOptions {
    /// `theTolerance`.
    pub tolerance: f64,
    /// `theCGFlag`.
    pub cg_flag: bool,
    /// `theIFlag`.
    pub i_flag: bool,
}

impl Default for VinertGKOptions {
    /// The native defaults: tolerance 0.001, no centre, no inertia.
    fn default() -> Self {
        Self {
            tolerance: 0.001,
            cg_flag: false,
            i_flag: false,
        }
    }
}

/// `BRepGProp_VinertGK`: the volume (and on request the centre and inertia) of the
/// region delimited by a face and the location, a point or a plane (a `GProp_GProps`,
/// docs/topalgo-design.md section 9); the inherited members are reached through
/// `Deref`. The centre and inertia keep their previous values when not requested.
///
/// ```
/// use rustycad_tktopalgo::brep_gprop::BRepGPropVinertGK;
/// let props = BRepGPropVinertGK::new();
/// assert_eq!(props.get_error_reached(), 0.0);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BRepGPropVinertGK {
    pub(super) base: GProps,
    pub(super) error_reached: f64,
    pub(super) absolut_error: f64,
}

impl Deref for BRepGPropVinertGK {
    type Target = GProps;
    fn deref(&self) -> &GProps {
        &self.base
    }
}

impl BRepGPropVinertGK {
    /// `BRepGProp_VinertGK()`: an empty framework, errors 0; `BRepGPropVinertGK::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `SetLocation(theLocation)`: the vertex of the region; `props.set_location(p)`.
    pub fn set_location(&mut self, location: Point3) {
        self.base.loc = location;
    }

    /// `GetErrorReached()`: the relative error of the volume (-1 after a failure);
    /// `props.get_error_reached()`.
    pub fn get_error_reached(&self) -> f64 {
        self.error_reached
    }

    /// The point shift of the by-point Performs: `thePoint - loc`.
    fn shift(&self, point: Point3) -> [f64; 4] {
        let xyz = point.xyz() - self.base.loc.xyz();
        [xyz.x(), xyz.y(), xyz.z(), 0.0]
    }

    /// The plane coefficients of the by-plane Performs, moved to the location.
    fn plane_coefficients(&self, plane: Plane3) -> Result<[f64; 4], Failure> {
        let [a, b, c, d] = plane.coefficients()?;
        let loc = self.base.loc;
        Ok([a, b, c, d - a * loc.x() - b * loc.y() - c * loc.z()])
    }

    /// `Perform(theSurface, theTolerance, theCGFlag, theIFlag)`: the naturally bounded
    /// face and the location (the U isoline at UMax as the only boundary); the absolute
    /// error; `props.perform(&mut face, VinertGKOptions::default())?`.
    pub fn perform(
        &mut self,
        surface: &mut BRepGPropFace,
        options: VinertGKOptions,
    ) -> Result<f64, Failure> {
        self.private_perform(surface, None, (true, &[0.0; 3]), options)
    }

    /// `Perform(theSurface, thePoint, ...)`: the inertia about `point`;
    /// `props.perform_point(&mut face, p, VinertGKOptions::default())?`.
    pub fn perform_point(
        &mut self,
        surface: &mut BRepGPropFace,
        point: Point3,
        options: VinertGKOptions,
    ) -> Result<f64, Failure> {
        let shift = self.shift(point);
        self.private_perform(surface, None, (true, &shift[..3]), options)
    }

    /// `Perform(theSurface, theDomain, ...)`: the face bounded by the domain;
    /// `props.perform_domain(&mut face, &mut domain, VinertGKOptions::default())?`.
    pub fn perform_domain(
        &mut self,
        surface: &mut BRepGPropFace,
        domain: &mut BRepGPropDomain,
        options: VinertGKOptions,
    ) -> Result<f64, Failure> {
        self.private_perform(surface, Some(domain), (true, &[0.0; 3]), options)
    }

    /// `Perform(theSurface, theDomain, thePoint, ...)`;
    /// `props.perform_domain_point(&mut face, &mut domain, p, VinertGKOptions::default())?`.
    pub fn perform_domain_point(
        &mut self,
        surface: &mut BRepGPropFace,
        domain: &mut BRepGPropDomain,
        point: Point3,
        options: VinertGKOptions,
    ) -> Result<f64, Failure> {
        let shift = self.shift(point);
        self.private_perform(surface, Some(domain), (true, &shift[..3]), options)
    }

    /// `Perform(theSurface, thePlane, ...)`: the region between the natural face and
    /// `plane`; `props.perform_plane(&mut face, plane, VinertGKOptions::default())?`.
    pub fn perform_plane(
        &mut self,
        surface: &mut BRepGPropFace,
        plane: Plane3,
        options: VinertGKOptions,
    ) -> Result<f64, Failure> {
        let coeffs = self.plane_coefficients(plane)?;
        self.private_perform(surface, None, (false, &coeffs), options)
    }

    /// `Perform(theSurface, theDomain, thePlane, ...)`;
    /// `props.perform_domain_plane(&mut face, &mut domain, plane, VinertGKOptions::default())?`.
    pub fn perform_domain_plane(
        &mut self,
        surface: &mut BRepGPropFace,
        domain: &mut BRepGPropDomain,
        plane: Plane3,
        options: VinertGKOptions,
    ) -> Result<f64, Failure> {
        let coeffs = self.plane_coefficients(plane)?;
        self.private_perform(surface, Some(domain), (false, &coeffs), options)
    }
}
