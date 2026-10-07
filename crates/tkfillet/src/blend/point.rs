// Rust translation of Blend_Point.hxx/.lxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! A point of a blend line: the two extremities of a section with their parameters.
mod accessors;

use rustycad_tkmath::gp::{Point3, Vector2, Vector3};

/// The tangents of a section between two surfaces, or a surface and a restriction, or two
/// restrictions: 3d tangents at both extremities and the 2d tangents on both supports.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SectionTangents {
    /// `Tg1`: the 3d tangent at the first extremity.
    pub tg1: Vector3,
    /// `Tg2`: the 3d tangent at the second extremity.
    pub tg2: Vector3,
    /// `Tg12d`: the 2d tangent on the first support.
    pub tg12d: Vector2,
    /// `Tg22d`: the 2d tangent on the second support.
    pub tg22d: Vector2,
}

/// The tangents of a section between a surface and a curve: 3d tangents on both and the
/// 2d tangent on the surface.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CurveSectionTangents {
    /// `Tgs`: the 3d tangent on the surface.
    pub tgs: Vector3,
    /// `Tgc`: the 3d tangent on the curve.
    pub tgc: Vector3,
    /// `Tg2d`: the 2d tangent on the surface.
    pub tg2d: Vector2,
}

/// One point of a blend line (`Blend_Point`): the two extremities of a section and the
/// parameter on the guide line, with the parameters on the supports (two surfaces, a
/// surface and a curve, a surface and a restriction, two restrictions, or two curves) and,
/// unless it is a tangency point, the tangents.
///
/// Natively the default constructor leaves the parameters and support flags
/// uninitialized; the port sets them to 0 and false.
///
/// ```
/// use rustycad_tkfillet::blend::BlendPoint;
/// use rustycad_tkmath::gp::Point3;
/// let point = BlendPoint::on_surfaces(Point3::default(), Point3::default(), 0.5, [1., 2., 3., 4.]);
/// assert_eq!(point.parameters_on_s2()?, (3., 4.));
/// assert!(point.is_tangency_point());
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlendPoint {
    pt1: Point3,
    pt2: Point3,
    tg1: Vector3,
    tg2: Vector3,
    prm: f64,
    u1: f64,
    v1: f64,
    u2: f64,
    v2: f64,
    pc1: f64,
    pc2: f64,
    utg12d: f64,
    vtg12d: f64,
    utg22d: f64,
    vtg22d: f64,
    hass1: bool,
    hass2: bool,
    hasc1: bool,
    hasc2: bool,
    istgt: bool,
}

impl Default for BlendPoint {
    fn default() -> Self {
        Self {
            pt1: Point3::default(),
            pt2: Point3::default(),
            tg1: Vector3::default(),
            tg2: Vector3::default(),
            prm: 0.0,
            u1: 0.0,
            v1: 0.0,
            u2: 0.0,
            v2: 0.0,
            pc1: 0.0,
            pc2: 0.0,
            utg12d: 0.0,
            vtg12d: 0.0,
            utg22d: 0.0,
            vtg22d: 0.0,
            hass1: false,
            hass2: false,
            hasc1: false,
            hasc2: false,
            istgt: true,
        }
    }
}

impl BlendPoint {
    /// `Blend_Point()`: a tangency point without supports; `BlendPoint::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// A point on two surfaces with tangents; `BlendPoint::on_surfaces_with_tangents(p1, p2, w, uv, t)`.
    pub fn on_surfaces_with_tangents(
        pt1: Point3,
        pt2: Point3,
        param: f64,
        uv: [f64; 4],
        tangents: SectionTangents,
    ) -> Self {
        let mut point = Self::new();
        point.set_on_surfaces_with_tangents(pt1, pt2, param, uv, tangents);
        point
    }

    /// A point on two surfaces without tangents; `BlendPoint::on_surfaces(p1, p2, w, uv)`.
    pub fn on_surfaces(pt1: Point3, pt2: Point3, param: f64, uv: [f64; 4]) -> Self {
        let mut point = Self::new();
        point.set_on_surfaces(pt1, pt2, param, uv);
        point
    }

    /// A point on a surface `(u, v)` and a curve `w` with tangents;
    /// `BlendPoint::on_surface_curve_with_tangents(ps, pc, t, [u, v, w], tg)`.
    pub fn on_surface_curve_with_tangents(
        pts: Point3,
        ptc: Point3,
        param: f64,
        uvw: [f64; 3],
        tangents: CurveSectionTangents,
    ) -> Self {
        let mut point = Self::new();
        point.set_on_surface_curve_with_tangents(pts, ptc, param, uvw, tangents);
        point
    }

    /// A point on a surface and a curve without tangents;
    /// `BlendPoint::on_surface_curve(ps, pc, t, [u, v, w])`.
    pub fn on_surface_curve(pts: Point3, ptc: Point3, param: f64, uvw: [f64; 3]) -> Self {
        let mut point = Self::new();
        point.set_on_surface_curve(pts, ptc, param, uvw);
        point
    }

    /// A point on a surface and a curve on surface (parameter `pc`) with tangents;
    /// `BlendPoint::on_surface_rst_with_tangents(p1, p2, t, uv, pc, tg)`.
    pub fn on_surface_rst_with_tangents(
        pt1: Point3,
        pt2: Point3,
        param: f64,
        uv: [f64; 4],
        pc: f64,
        tangents: SectionTangents,
    ) -> Self {
        let mut point = Self::new();
        point.set_on_surface_rst_with_tangents(pt1, pt2, param, uv, pc, tangents);
        point
    }

    /// A point on a surface and a curve on surface without tangents;
    /// `BlendPoint::on_surface_rst(p1, p2, t, uv, pc)`.
    pub fn on_surface_rst(pt1: Point3, pt2: Point3, param: f64, uv: [f64; 4], pc: f64) -> Self {
        let mut point = Self::new();
        point.set_on_surface_rst(pt1, pt2, param, uv, pc);
        point
    }

    /// A point on two curves on surfaces (parameters `pc`) with tangents;
    /// `BlendPoint::on_rsts_with_tangents(p1, p2, t, uv, [pc1, pc2], tg)`.
    pub fn on_rsts_with_tangents(
        pt1: Point3,
        pt2: Point3,
        param: f64,
        uv: [f64; 4],
        pc: [f64; 2],
        tangents: SectionTangents,
    ) -> Self {
        let mut point = Self::new();
        point.set_on_rsts_with_tangents(pt1, pt2, param, uv, pc, tangents);
        point
    }

    /// A point on two curves on surfaces without tangents;
    /// `BlendPoint::on_rsts(p1, p2, t, uv, [pc1, pc2])`.
    pub fn on_rsts(pt1: Point3, pt2: Point3, param: f64, uv: [f64; 4], pc: [f64; 2]) -> Self {
        let mut point = Self::new();
        point.set_on_rsts(pt1, pt2, param, uv, pc);
        point
    }

    fn set_frame(&mut self, pt1: Point3, pt2: Point3, param: f64) {
        self.pt1 = pt1;
        self.pt2 = pt2;
        self.prm = param;
    }

    fn set_surface_parameters(&mut self, uv: [f64; 4]) {
        [self.u1, self.v1, self.u2, self.v2] = uv;
        self.hass1 = true;
        self.hass2 = true;
    }

    fn set_tangents(&mut self, tangents: SectionTangents) {
        self.istgt = false;
        self.tg1 = tangents.tg1;
        self.tg2 = tangents.tg2;
        (self.utg12d, self.vtg12d) = (tangents.tg12d.x(), tangents.tg12d.y());
        (self.utg22d, self.vtg22d) = (tangents.tg22d.x(), tangents.tg22d.y());
    }

    /// `SetValue` on two surfaces with tangents; `point.set_on_surfaces_with_tangents(p1, p2, w, uv, t)`.
    pub fn set_on_surfaces_with_tangents(
        &mut self,
        pt1: Point3,
        pt2: Point3,
        param: f64,
        uv: [f64; 4],
        tangents: SectionTangents,
    ) {
        self.set_on_surfaces(pt1, pt2, param, uv);
        self.set_tangents(tangents);
    }

    /// `SetValue` on two surfaces without tangents; `point.set_on_surfaces(p1, p2, w, uv)`.
    pub fn set_on_surfaces(&mut self, pt1: Point3, pt2: Point3, param: f64, uv: [f64; 4]) {
        self.set_frame(pt1, pt2, param);
        self.set_surface_parameters(uv);
        self.hasc1 = false;
        self.hasc2 = false;
        self.istgt = true;
    }

    /// `SetValue` on a surface and a curve with tangents;
    /// `point.set_on_surface_curve_with_tangents(ps, pc, t, uvw, tg)`.
    pub fn set_on_surface_curve_with_tangents(
        &mut self,
        pts: Point3,
        ptc: Point3,
        param: f64,
        uvw: [f64; 3],
        tangents: CurveSectionTangents,
    ) {
        self.set_on_surface_curve(pts, ptc, param, uvw);
        self.istgt = false;
        self.tg1 = tangents.tgs;
        self.tg2 = tangents.tgc;
        (self.utg12d, self.vtg12d) = (tangents.tg2d.x(), tangents.tg2d.y());
    }

    /// `SetValue` on a surface and a curve without tangents;
    /// `point.set_on_surface_curve(ps, pc, t, uvw)`.
    pub fn set_on_surface_curve(&mut self, pts: Point3, ptc: Point3, param: f64, uvw: [f64; 3]) {
        self.set_frame(pts, ptc, param);
        (self.u1, self.v1, self.pc2) = (uvw[0], uvw[1], uvw[2]);
        self.hass1 = true;
        self.hass2 = false;
        self.hasc1 = false;
        self.hasc2 = true;
        self.istgt = true;
    }

    /// `SetValue` on a surface and a curve on surface with tangents;
    /// `point.set_on_surface_rst_with_tangents(p1, p2, t, uv, pc, tg)`.
    pub fn set_on_surface_rst_with_tangents(
        &mut self,
        pt1: Point3,
        pt2: Point3,
        param: f64,
        uv: [f64; 4],
        pc: f64,
        tangents: SectionTangents,
    ) {
        self.set_on_surface_rst(pt1, pt2, param, uv, pc);
        self.set_tangents(tangents);
    }

    /// `SetValue` on a surface and a curve on surface without tangents;
    /// `point.set_on_surface_rst(p1, p2, t, uv, pc)`.
    pub fn set_on_surface_rst(
        &mut self,
        pt1: Point3,
        pt2: Point3,
        param: f64,
        uv: [f64; 4],
        pc: f64,
    ) {
        self.set_frame(pt1, pt2, param);
        self.set_surface_parameters(uv);
        self.hasc1 = false;
        self.pc2 = pc;
        self.hasc2 = true;
        self.istgt = true;
    }

    /// `SetValue` on two curves on surfaces with tangents;
    /// `point.set_on_rsts_with_tangents(p1, p2, t, uv, pc, tg)`.
    pub fn set_on_rsts_with_tangents(
        &mut self,
        pt1: Point3,
        pt2: Point3,
        param: f64,
        uv: [f64; 4],
        pc: [f64; 2],
        tangents: SectionTangents,
    ) {
        self.set_on_rsts(pt1, pt2, param, uv, pc);
        self.set_tangents(tangents);
    }

    /// `SetValue` on two curves on surfaces without tangents;
    /// `point.set_on_rsts(p1, p2, t, uv, pc)`.
    pub fn set_on_rsts(
        &mut self,
        pt1: Point3,
        pt2: Point3,
        param: f64,
        uv: [f64; 4],
        pc: [f64; 2],
    ) {
        self.set_frame(pt1, pt2, param);
        self.set_surface_parameters(uv);
        [self.pc1, self.pc2] = pc;
        self.hasc1 = true;
        self.hasc2 = true;
        self.istgt = true;
    }

    /// `SetValue(Pt1, Pt2, Param, PC1, PC2)`: a point on two curves, without surface
    /// parameters (they keep their old values) nor tangents;
    /// `point.set_on_curves(p1, p2, t, 0.5, 0.25)`.
    pub fn set_on_curves(&mut self, pt1: Point3, pt2: Point3, param: f64, pc1: f64, pc2: f64) {
        self.set_frame(pt1, pt2, param);
        self.hass1 = false;
        self.hass2 = false;
        (self.pc1, self.pc2) = (pc1, pc2);
        self.hasc1 = true;
        self.hasc2 = true;
        self.istgt = true;
    }

    /// `SetParameter(Param)`: changes the parameter on the guide line;
    /// `point.set_parameter(1.0)`.
    pub fn set_parameter(&mut self, param: f64) {
        self.prm = param;
    }
}

#[cfg(test)]
#[path = "point_tests.rs"]
mod tests;
