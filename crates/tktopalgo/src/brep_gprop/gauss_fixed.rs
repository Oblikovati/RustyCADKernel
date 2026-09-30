// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepGProp/BRepGProp_Gauss.cxx (the
// non-adaptive Compute overloads) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The non-adaptive `Compute` overloads of `BRepGProp_Gauss`: fixed Gauss orders on the
//! natural bounds of a face or along its boundary curves.
use super::domain::BRepGPropDomain;
use super::face::BRepGPropFace;
use super::gauss::{BRepGPropGauss, GaussProps, GaussType, Inertia, add_volume_part};
use super::gauss_table::{GaussRule, gauss_points_and_weights};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkmath::gp::Point3;
use rustycad_tkmath::math;

impl BRepGPropGauss {
    /// `Compute(theSurface, theLocation, theCoeff, theIsByPoint, Mass, G, I)`: the
    /// properties of the face on its natural bounds (the type of the integrator selects
    /// the surface or volume integrand), with `min(U|VIntegrationOrder, GaussPointsMax)`
    /// points; the mass is multiplied by the half ranges after the centre is computed,
    /// as natively; `gauss.compute_natural(&face, location, &[0.0; 3], true, &mut out)?`.
    pub fn compute_natural(
        &mut self,
        surface: &BRepGPropFace,
        location: Point3,
        coeff: &[f64],
        by_point: bool,
        out: &mut GaussProps,
    ) -> Result<(), Failure> {
        let (lower_u, upper_u, lower_v, upper_v) = surface.bounds()?;
        self.check_bounds((lower_u, upper_u, lower_v, upper_v));
        let u_order = surface.u_integration_order()?.min(math::gauss_points_max());
        let v_order = surface.v_integration_order()?.min(math::gauss_points_max());
        let (pu, wu) = gauss_points_and_weights(u_order)?;
        let (pv, wv) = gauss_points_and_weights(v_order)?;
        let um = 0.5 * self.add(upper_u, lower_u);
        let vm = 0.5 * self.add(upper_v, lower_v);
        let ur = 0.5 * self.add(upper_u, -lower_u);
        let mut vr = 0.5 * self.add(upper_v, -lower_v);
        let mut inertia = Inertia::default();
        for (point_v, weight_v) in pv.iter().zip(&wv) {
            let mut part = Inertia::default();
            let v = self.add(vm, self.mult(vr, *point_v));
            for (point_u, weight_u) in pu.iter().zip(&wu) {
                let u = self.add(um, self.mult(ur, *point_u));
                let (point, normal) = surface.normal(u, v)?;
                self.add_part(
                    point,
                    normal,
                    location,
                    *weight_u,
                    (coeff, by_point),
                    &mut part,
                );
            }
            self.mult_inertia(*weight_v, &mut part);
            self.add_inertia(&part, &mut inertia);
        }
        vr = self.mult(vr, ur);
        for field in [
            &mut inertia.ixx,
            &mut inertia.iyy,
            &mut inertia.izz,
            &mut inertia.ixy,
            &mut inertia.ixz,
            &mut inertia.iyz,
        ] {
            *field = self.mult(vr, *field);
        }
        self.convert(&inertia, (coeff, by_point), out);
        out.mass *= vr;
        Ok(())
    }

    /// `Compute(theSurface, theLocation, Mass, G, I)`: [`BRepGPropGauss::compute_natural`]
    /// for a Sinert integrator; Standard_ProgramError for a Vinert one;
    /// `gauss.compute_natural_surface(&face, location, &mut out)?`.
    pub fn compute_natural_surface(
        &mut self,
        surface: &BRepGPropFace,
        location: Point3,
        out: &mut GaussProps,
    ) -> Result<(), Failure> {
        self.assert_type(GaussType::Sinert)?;
        self.compute_natural(surface, location, &[], true, out)
    }

    /// `Compute(theSurface, theDomain, theLocation, Mass, G, I)`: the surface properties
    /// along the boundary curves of the domain (Sinert only, Standard_ProgramError
    /// otherwise); an edge without pcurve ends the computation and leaves `out` as it
    /// was; `gauss.compute_domain_surface(&mut face, &mut domain, location, &mut out)?`.
    pub fn compute_domain_surface(
        &mut self,
        surface: &mut BRepGPropFace,
        domain: &mut BRepGPropDomain,
        location: Point3,
        out: &mut GaussProps,
    ) -> Result<(), Failure> {
        self.assert_type(GaussType::Sinert)?;
        let (u1, u2, v1, v2) = surface.bounds()?;
        self.check_bounds((u1, u2, v1, v2));
        let u_order = surface.u_integration_order()?.min(math::gauss_points_max());
        let v_order = surface.v_integration_order()?.min(math::gauss_points_max());
        let order = u_order.max(v_order);
        let surface_rule = gauss_points_and_weights(order)?;
        let mut inertia = Inertia::default();
        while domain.more() {
            if !surface.load_edge(&domain.value()?)? {
                return Ok(());
            }
            let curve_order = surface
                .integration_order()?
                .min(math::gauss_points_max())
                .max(order);
            let curve_rule = gauss_points_and_weights(curve_order)?;
            let arc = self.surface_arc(surface, location, u1, (&curve_rule, &surface_rule))?;
            self.add_inertia(&arc, &mut inertia);
            domain.next()?;
        }
        self.convert_surface(&inertia, out);
        Ok(())
    }

    /// The surface integral along the current boundary curve: Gauss points `curve` on the
    /// curve, `rule` between `u1` and the curve in U (the V of the curve not clamped).
    fn surface_arc(
        &self,
        surface: &BRepGPropFace,
        location: Point3,
        u1: f64,
        rules: (&GaussRule, &GaussRule),
    ) -> Result<Inertia, Failure> {
        let ((cp, cw), (sp, sw)) = rules;
        let (l1, l2) = (surface.first_parameter()?, surface.last_parameter()?);
        let (lm, lr) = (0.5 * (l2 + l1), 0.5 * (l2 - l1));
        let mut arc = Inertia::default();
        for (point_l, weight_l) in cp.iter().zip(cw) {
            let (puv, vuv) = surface.d12d(lm + lr * point_l)?;
            let (v, u2) = (puv.y(), puv.x());
            let dul = vuv.y() * weight_l;
            let (um, ur) = (0.5 * (u2 + u1), 0.5 * (u2 - u1));
            let mut local = Inertia::default();
            for (point_u, weight_u) in sp.iter().zip(sw) {
                let u = self.add(um, self.mult(ur, *point_u));
                let (point, normal) = surface.normal(u, v)?;
                self.add_surface_part(point, normal, location, dul * weight_u, &mut local);
            }
            self.mult_inertia(ur, &mut local);
            self.add_inertia(&local, &mut arc);
        }
        self.mult_inertia(lr, &mut arc);
        Ok(arc)
    }

    /// `Compute(theSurface, theDomain, theLocation, theCoeff, theIsByPoint, Mass, G, I)`:
    /// the volume properties along the boundary curves (Vinert only,
    /// Standard_ProgramError otherwise), the curve points clamped to the U and V bounds
    /// (OCC104); an edge without pcurve ends the computation and leaves `out` as it was;
    /// `gauss.compute_domain(&mut face, &mut domain, location, &[0.0; 3], true, &mut out)?`.
    pub fn compute_domain(
        &mut self,
        surface: &mut BRepGPropFace,
        domain: &mut BRepGPropDomain,
        location: Point3,
        coeff: &[f64],
        by_point: bool,
        out: &mut GaussProps,
    ) -> Result<(), Failure> {
        self.assert_type(GaussType::Vinert)?;
        let bounds = surface.bounds()?;
        self.check_bounds(bounds);
        let mut inertia = Inertia::default();
        while domain.more() {
            if !surface.load_edge(&domain.value()?)? {
                return Ok(());
            }
            let v_order = surface.v_integration_order()?;
            let order = surface
                .integration_order()?
                .max(v_order)
                .min(math::gauss_points_max());
            let rule = gauss_points_and_weights(order)?;
            let arc = self.volume_arc(surface, location, bounds, &rule, (coeff, by_point))?;
            self.add_inertia(&arc, &mut inertia);
            domain.next()?;
        }
        self.convert_volume(&inertia, coeff, by_point, out);
        Ok(())
    }

    /// The volume integral along the current boundary curve, with one rule in both
    /// directions and plain arithmetic for the U points.
    fn volume_arc(
        &self,
        surface: &BRepGPropFace,
        location: Point3,
        bounds: (f64, f64, f64, f64),
        rule: &GaussRule,
        plane: (&[f64], bool),
    ) -> Result<Inertia, Failure> {
        let (u1, u2_bound, v1, v2) = bounds;
        let (points, weights) = rule;
        let (l1, l2) = (surface.first_parameter()?, surface.last_parameter()?);
        let (lm, lr) = (0.5 * (l2 + l1), 0.5 * (l2 - l1));
        let mut arc = Inertia::default();
        for (point_l, weight_l) in points.iter().zip(weights) {
            let (puv, vuv) = surface.d12d(lm + lr * point_l)?;
            let u2 = std_min(std_max(u1, puv.x()), u2_bound);
            let v = std_min(std_max(puv.y(), v1), v2);
            let dul = vuv.y() * weight_l;
            let (um, ur) = (0.5 * (u2 + u1), 0.5 * (u2 - u1));
            let mut local = Inertia::default();
            for (point_u, weight_u) in points.iter().zip(weights) {
                let (point, normal) = surface.normal(um + ur * point_u, v)?;
                add_volume_part(point, normal, location, dul * weight_u, plane, &mut local);
            }
            self.mult_inertia(ur, &mut local);
            self.add_inertia(&local, &mut arc);
        }
        self.mult_inertia(lr, &mut arc);
        Ok(arc)
    }

    /// One Gauss point of the integrand of the integrator's type.
    pub(super) fn add_part(
        &self,
        point: Point3,
        normal: rustycad_tkmath::gp::Vector3,
        location: Point3,
        weight: f64,
        plane: (&[f64], bool),
        out: &mut Inertia,
    ) {
        match self.kind {
            GaussType::Vinert => add_volume_part(point, normal, location, weight, plane, out),
            GaussType::Sinert => self.add_surface_part(point, normal, location, weight, out),
        }
    }

    /// The `convert` overload of the integrator's type.
    pub(super) fn convert(&self, inertia: &Inertia, plane: (&[f64], bool), out: &mut GaussProps) {
        match self.kind {
            GaussType::Vinert => self.convert_volume(inertia, plane.0, plane.1, out),
            GaussType::Sinert => self.convert_surface(inertia, out),
        }
    }
}
