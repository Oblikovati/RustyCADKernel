// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepGProp/BRepGProp_VinertGK.cxx (the
// constructors and PrivatePerform) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The constructors of `BRepGProp_VinertGK` and its Gauss-Kronrod integration.
use super::domain::BRepGPropDomain;
use super::face::BRepGPropFace;
use super::tfunction::BRepGPropTFunction;
use super::vinert_gk::{BRepGPropVinertGK, VinertGKOptions};
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::epsilon;
use rustycad_tkgeombase::gprop::ValueType;
use rustycad_tkmath::geom_abs::IsoType;
use rustycad_tkmath::gp::{Matrix3, Plane3, Point3, Xyz};
use rustycad_tkmath::math::KronrodSingleIntegration;

/// The minimal T range of an outer integral (`aTTol`).
const T_TOL: f64 = 1.0e-9;

/// The ten values of the integration, in the native order (`k = 1..10`).
const VALUE_TYPES: [ValueType; 10] = [
    ValueType::Mass,
    ValueType::CenterMassX,
    ValueType::CenterMassY,
    ValueType::CenterMassZ,
    ValueType::InertiaXX,
    ValueType::InertiaYY,
    ValueType::InertiaZZ,
    ValueType::InertiaXY,
    ValueType::InertiaXZ,
    ValueType::InertiaYZ,
];

/// The integral values and their reached errors (`aValue`, `aTolReached`).
#[derive(Default)]
struct GkSums {
    value: [f64; 10],
    tol_reached: [f64; 10],
}

/// The surface of one PrivatePerform: the face tool, its domain, the region kind and
/// coefficients, the options.
struct GkRun<'a> {
    surface: &'a mut BRepGPropFace,
    domain: Option<&'a mut BRepGPropDomain>,
    by_point: bool,
    coeffs: &'a [f64],
    options: VinertGKOptions,
}

impl BRepGPropVinertGK {
    fn located(location: Point3) -> Self {
        let mut props = Self::new();
        props.set_location(location);
        props
    }

    /// `BRepGProp_VinertGK(theSurface, theLocation, ...)`;
    /// `BRepGPropVinertGK::with_face(&mut face, location, VinertGKOptions::default())?`.
    pub fn with_face(
        surface: &mut BRepGPropFace,
        location: Point3,
        options: VinertGKOptions,
    ) -> Result<Self, Failure> {
        let mut props = Self::located(location);
        props.perform(surface, options)?;
        Ok(props)
    }

    /// `BRepGProp_VinertGK(theSurface, thePoint, theLocation, ...)`;
    /// `BRepGPropVinertGK::with_face_point(&mut face, p, location, options)?`.
    pub fn with_face_point(
        surface: &mut BRepGPropFace,
        point: Point3,
        location: Point3,
        options: VinertGKOptions,
    ) -> Result<Self, Failure> {
        let mut props = Self::located(location);
        props.perform_point(surface, point, options)?;
        Ok(props)
    }

    /// `BRepGProp_VinertGK(theSurface, theDomain, theLocation, ...)`;
    /// `BRepGPropVinertGK::with_domain(&mut face, &mut domain, location, options)?`.
    pub fn with_domain(
        surface: &mut BRepGPropFace,
        domain: &mut BRepGPropDomain,
        location: Point3,
        options: VinertGKOptions,
    ) -> Result<Self, Failure> {
        let mut props = Self::located(location);
        props.perform_domain(surface, domain, options)?;
        Ok(props)
    }

    /// `BRepGProp_VinertGK(theSurface, theDomain, thePoint, theLocation, ...)`;
    /// `BRepGPropVinertGK::with_domain_point(&mut face, &mut domain, p, location, options)?`.
    pub fn with_domain_point(
        surface: &mut BRepGPropFace,
        domain: &mut BRepGPropDomain,
        point: Point3,
        location: Point3,
        options: VinertGKOptions,
    ) -> Result<Self, Failure> {
        let mut props = Self::located(location);
        props.perform_domain_point(surface, domain, point, options)?;
        Ok(props)
    }

    /// `BRepGProp_VinertGK(theSurface, thePlane, theLocation, ...)`;
    /// `BRepGPropVinertGK::with_face_plane(&mut face, plane, location, options)?`.
    pub fn with_face_plane(
        surface: &mut BRepGPropFace,
        plane: Plane3,
        location: Point3,
        options: VinertGKOptions,
    ) -> Result<Self, Failure> {
        let mut props = Self::located(location);
        props.perform_plane(surface, plane, options)?;
        Ok(props)
    }

    /// `BRepGProp_VinertGK(theSurface, theDomain, thePlane, theLocation, ...)`;
    /// `BRepGPropVinertGK::with_domain_plane(&mut face, &mut domain, plane, location, options)?`.
    pub fn with_domain_plane(
        surface: &mut BRepGPropFace,
        domain: &mut BRepGPropDomain,
        plane: Plane3,
        location: Point3,
        options: VinertGKOptions,
    ) -> Result<Self, Failure> {
        let mut props = Self::located(location);
        props.perform_domain_plane(surface, domain, plane, options)?;
        Ok(props)
    }

    /// `PrivatePerform`: the Gauss-Kronrod integrals along each boundary (the U isoline
    /// at UMax without domain) of the requested values, then the mass, centre and
    /// inertia; -1 (kept as the error) without boundary or when an integral is not done.
    pub(super) fn private_perform(
        &mut self,
        surface: &mut BRepGPropFace,
        domain: Option<&mut BRepGPropDomain>,
        (by_point, coeffs): (bool, &[f64]),
        options: VinertGKOptions,
    ) -> Result<f64, Failure> {
        let mut run = GkRun {
            surface,
            domain,
            by_point,
            coeffs,
            options,
        };
        if run.count_curves()? == 0 {
            self.error_reached = -1.;
            return Ok(self.error_reached);
        }
        let Some(sums) = run.integrate(self.base.loc)? else {
            self.error_reached = -1.;
            return Ok(self.error_reached);
        };
        self.store(sums, (by_point, coeffs), options);
        Ok(self.absolut_error)
    }

    /// The mass, errors, centre (with the centre or inertia flag) and inertia (with the
    /// inertia flag) of the integrals; the centre and mass are 0 below
    /// `Epsilon(AbsolutError)`.
    fn store(
        &mut self,
        mut sums: GkSums,
        (by_point, coeffs): (bool, &[f64]),
        options: VinertGKOptions,
    ) {
        let v = &mut sums.value;
        self.base.dim = v[0];
        self.error_reached = sums.tol_reached[0];
        self.absolut_error = self.error_reached;
        let abs_dim = self.base.dim.abs();
        let vol_tol = epsilon(self.absolut_error);
        if abs_dim >= vol_tol {
            self.error_reached /= abs_dim;
        }
        if options.cg_flag || options.i_flag {
            let dim = self.base.dim;
            for axis in 0..3 {
                v[axis + 1] = match (abs_dim >= vol_tol, by_point) {
                    (true, true) => coeffs[axis] + v[axis + 1] / dim,
                    (true, false) => v[axis + 1] / dim,
                    (false, _) => 0.,
                };
            }
            if abs_dim < vol_tol {
                self.base.dim = 0.;
            }
            self.base.g = Point3::new(v[1], v[2], v[3]);
        }
        if options.i_flag {
            self.base.inertia = Matrix3::from_columns(
                Xyz::new(v[4], v[7], v[8]),
                Xyz::new(v[7], v[5], v[9]),
                Xyz::new(v[8], v[9], v[6]),
            );
        }
    }
}

impl GkRun<'_> {
    /// The number of boundary curves: 1 without domain, the edges of the domain else
    /// (explored from its start).
    fn count_curves(&mut self) -> Result<i32, Failure> {
        let Some(domain) = self.domain.as_deref_mut() else {
            return Ok(1);
        };
        let mut count = 0;
        domain.init()?;
        while domain.more() {
            count += 1;
            domain.next()?;
        }
        Ok(count)
    }

    /// The flags of the ten values: the mass, the centre with either flag, the inertia
    /// with the inertia flag.
    fn flags(&self) -> [bool; 10] {
        let centre = self.options.cg_flag || self.options.i_flag;
        std::array::from_fn(|k| k == 0 || (k < 4 && centre) || (k >= 4 && self.options.i_flag))
    }

    /// The loop over the boundaries; `None` when an integral is not done.
    fn integrate(&mut self, location: Point3) -> Result<Option<GkSums>, Failure> {
        let mut sums = GkSums::default();
        let (u_min, _, _, _) = self.surface.bounds()?;
        let mut is_more = match self.domain.as_deref_mut() {
            None => true,
            Some(domain) => {
                domain.init()?;
                domain.more()
            }
        };
        while is_more {
            self.load_boundary()?;
            if !self.boundary_integrals(location, u_min, &mut sums)? {
                return Ok(None);
            }
            is_more = match self.domain.as_deref_mut() {
                None => false,
                Some(domain) => {
                    domain.next()?;
                    domain.more()
                }
            };
        }
        Ok(Some(sums))
    }

    /// The next boundary: the U isoline at UMax, or the pcurve of the domain's edge (an
    /// edge without pcurve keeps the previous curve: the result of Load is ignored, as
    /// natively).
    fn load_boundary(&mut self) -> Result<(), Failure> {
        match self.domain.as_deref() {
            None => self.surface.load_iso(false, IsoType::IsoU),
            Some(domain) => {
                let edge = domain.value()?;
                self.surface.load_edge(&edge).map(|_| ())
            }
        }
    }

    /// The integrals of the requested values along the current boundary, over its T
    /// knot spans; false when one is not done.
    fn boundary_integrals(
        &mut self,
        location: Point3,
        u_min: f64,
        sums: &mut GkSums,
    ) -> Result<bool, Failure> {
        let (t_min, t_max) = (
            self.surface.first_parameter()?,
            self.surface.last_parameter()?,
        );
        let crv_tol = 0.1 * self.options.tolerance;
        let mut t_func = BRepGPropTFunction::new(
            self.surface,
            location,
            self.by_point,
            self.coeffs,
            u_min,
            crv_tol,
        );
        let knots: Vec<f64> = self
            .surface
            .get_t_knots(t_min, t_max)?
            .iter()
            .copied()
            .collect();
        let nb_intervals = knots.len() as i32 - 1;
        let nb_pnts =
            std::cmp::min(15, self.surface.integration_order()? / nb_intervals + 1).max(5);
        let mut integral = KronrodSingleIntegration::default();
        for (k, value_type) in VALUE_TYPES.iter().enumerate() {
            if !self.flags()[k] {
                continue;
            }
            t_func.set_value_type(*value_type);
            let Some((value, tol)) = spans_integral(
                &mut t_func,
                &mut integral,
                &knots,
                (nb_pnts, crv_tol, self.options.tolerance),
            )?
            else {
                return Ok(false);
            };
            sums.value[k] += value;
            sums.tol_reached[k] += tol;
        }
        Ok(true)
    }
}

/// The integral of one value over the T spans longer than 1e-9 and its reached error;
/// `None` when a span integral is not done.
fn spans_integral(
    t_func: &mut BRepGPropTFunction,
    integral: &mut KronrodSingleIntegration,
    knots: &[f64],
    (nb_pnts, crv_tol, tolerance): (i32, f64, f64),
) -> Result<Option<(f64, f64)>, Failure> {
    let tol_span = 0.9 * tolerance;
    let (mut value, mut tol_reached) = (0., 0.);
    for span in knots.windows(2) {
        let (t1, t2) = (span[0], span[1]);
        if t2 - t1 < T_TOL {
            continue;
        }
        t_func.set_nb_kronrod_points(nb_pnts);
        t_func.init();
        t_func.set_tolerance(crv_tol / (t2 - t1));
        integral.perform_with_tolerance(t_func, t1, t2, nb_pnts, tol_span, 1000);
        if let Some(failure) = t_func.take_failure() {
            return Err(failure);
        }
        if !integral.is_done() {
            return Ok(None);
        }
        value += integral.value()?;
        let err1 = t_func.absolut_error() * (t2 - t1);
        tol_reached += integral.absolute_error()? + err1;
    }
    Ok(Some((value, tol_reached)))
}
