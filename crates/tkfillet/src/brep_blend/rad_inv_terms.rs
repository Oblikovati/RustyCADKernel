//! The terms of the radius equation that the inverse functions of a fillet ending on a
//! point or on a curve share (`BRepBlend_SurfPoint*RadInv`, `BRepBlend_SurfCurv*RadInv`):
//! the vector `ref` from the point (or curve point) to the surface point offset by the
//! radius along the surface normal projected in the guide plane, and its derivatives.
use rustycad_tkmath::gp::Vector3;

/// `ref` of `Value`: `ns = (nplan.ns) nplan - ns` scaled by `1 / |nplan ^ ns|`, then
/// `ref = ray ns + corde` (the normal scaled before the radius, unlike [`NormalTerms`]).
pub(crate) fn value_ref(ray: f64, nplan: Vector3, ns: Vector3, corde: Vector3) -> Vector3 {
    let norm = nplan.crossed(ns).magnitude();
    let unsurnorm = 1. / norm;
    let mut projected = Vector3::default();
    projected.set_linear_form(nplan.dot(ns), nplan, -1., ns);
    projected *= unsurnorm;
    let mut reference = Vector3::default();
    reference.set_scaled_sum(ray, projected, corde);
    reference
}

/// The normal terms of `Derivatives` and `Values` at a surface normal `nsurf`.
pub(crate) struct NormalTerms {
    nplancrosnsurf: Vector3,
    pub(crate) unsurnorm: f64,
    raysurnorm: f64,
    raysurnorm2: f64,
    nplandotnsurf: f64,
    pub(crate) temp: Vector3,
}

impl NormalTerms {
    /// The terms for the radius `ray`, the plane normal `nplan` and the surface normal.
    pub(crate) fn new(ray: f64, nplan: Vector3, nsurf: Vector3) -> Self {
        let nplancrosnsurf = nplan.crossed(nsurf);
        let norm2 = nplancrosnsurf.square_magnitude();
        let norm = norm2.sqrt();
        let unsurnorm = 1. / norm;
        let unsurnorm2 = unsurnorm * unsurnorm;
        let nplandotnsurf = nplan.dot(nsurf);
        let mut temp = Vector3::default();
        temp.set_linear_form(nplandotnsurf, nplan, -1., nsurf);
        Self {
            nplancrosnsurf,
            unsurnorm,
            raysurnorm: ray * unsurnorm,
            raysurnorm2: ray * unsurnorm2,
            nplandotnsurf,
            temp,
        }
    }

    /// `ref = raysurnorm temp + corde`.
    pub(crate) fn reference(&self, corde: Vector3) -> Vector3 {
        let mut reference = Vector3::default();
        reference.set_scaled_sum(self.raysurnorm, self.temp, corde);
        reference
    }

    /// The derivative of `ref` along the guide, the plane normal turning by `dnplan`
    /// (`dwref`, the radius taken constant).
    pub(crate) fn guide_ref(&self, nplan: Vector3, dnplan: Vector3, nsurf: Vector3) -> Vector3 {
        let dwnorm = self.unsurnorm * self.nplancrosnsurf.dot(dnplan.crossed(nsurf));
        let mut dwtemp = Vector3::default();
        dwtemp.set_linear_form(self.nplandotnsurf, dnplan, dnplan.dot(nsurf), nplan);
        let mut dwref = Vector3::default();
        dwref.set_linear_form(
            self.raysurnorm,
            dwtemp,
            -self.raysurnorm2 * dwnorm,
            self.temp,
        );
        dwref
    }

    /// The derivative of `ref` along a surface parameter of normal derivative `dnsurf`
    /// and point derivative `dpts`.
    pub(crate) fn surface_ref(&self, nplan: Vector3, dnsurf: Vector3, dpts: Vector3) -> Vector3 {
        let dnorm = self.unsurnorm * self.nplancrosnsurf.dot(nplan.crossed(dnsurf));
        let mut dtemp = Vector3::default();
        dtemp.set_linear_form(nplan.dot(dnsurf), nplan, -1., dnsurf);
        let mut dref = Vector3::default();
        dref.set_linear_form_offset(
            self.raysurnorm,
            dtemp,
            -self.raysurnorm2 * dnorm,
            self.temp,
            dpts,
        );
        dref
    }

    /// The law term the evolutive inverses add to `dwref`: `dwref + dray unsurnorm temp`.
    pub(crate) fn with_law(&self, dwref: Vector3, dray: f64) -> Vector3 {
        let mut out = Vector3::default();
        out.set_linear_form(1., dwref, dray * self.unsurnorm, self.temp);
        out
    }
}

/// The guide plane at `w`: the plane normal `d1 / |d1|`, its derivative and the derivative
/// of the plane offset `theD` (`dtheD`), from the guide point and derivatives.
pub(crate) struct GuidePlane {
    pub(crate) nplan: Vector3,
    pub(crate) dnplan: Vector3,
    pub(crate) the_d: f64,
    pub(crate) dthe_d: f64,
}

impl GuidePlane {
    /// The plane of the derivative members: `nplan = (1 / |d1|) d1`,
    /// `dnplan = (-(nplan.d2) nplan + d2) / |d1|`.
    pub(crate) fn new(ptcur: rustycad_tkmath::gp::Point3, d1cur: Vector3, d2cur: Vector3) -> Self {
        let unsurnormd1cur = 1. / d1cur.magnitude();
        let nplan = unsurnormd1cur * d1cur;
        let the_d = -nplan.xyz().dot(ptcur.xyz());
        let mut dnplan = Vector3::default();
        dnplan.set_scaled_sum(-nplan.dot(d2cur), nplan, d2cur);
        dnplan *= unsurnormd1cur;
        let dthe_d = -nplan.xyz().dot(*d1cur.xyz()) - dnplan.xyz().dot(ptcur.xyz());
        Self {
            nplan,
            dnplan,
            the_d,
            dthe_d,
        }
    }

    /// `nplan.P + theD`.
    pub(crate) fn offset(&self, point: rustycad_tkmath::gp::Point3) -> f64 {
        self.nplan.xyz().dot(point.xyz()) + self.the_d
    }

    /// `dnplan.P + dtheD`.
    pub(crate) fn derivative_offset(&self, point: rustycad_tkmath::gp::Point3) -> f64 {
        self.dnplan.xyz().dot(point.xyz()) + self.dthe_d
    }
}

/// The ray of `Set(R, Choix)` of the constant inverse: `-|R|` but for the choices 3 and 4.
pub(crate) fn choice_ray(radius: f64, choix: i32) -> f64 {
    match choix {
        3 | 4 => radius.abs(),
        _ => -radius.abs(),
    }
}

/// The sign of `Set(Choix)` of the evolutive inverse: -1 but for the choices 3 and 4.
pub(crate) fn choice_sign(choix: i32) -> f64 {
    match choix {
        3 | 4 => 1.,
        _ => -1.,
    }
}
