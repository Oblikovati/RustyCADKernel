// Copyright (c) 1992-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.
// Rust translation of IntAna_QuadQuadGeo::Perform(gp_Pln, gp_Cone, double, double)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The plane-cone intersection: the conic sections.
use super::QuadQuadGeo;
use crate::int_ana::{IntConicQuad, ResultType};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Cone3, Direction3, Line3, Plane3, Point3, Xyz};

/// Ellipses beyond this radius are refused (OCC513): too extreme to handle.
const ELLIPSE_LIMIT: f64 = 1.0E+9;
/// Hyperbolas beyond these radii are refused (OCC537).
const HYPERBOLA_LIMIT: f64 = 2.0E+6;

/// The angles of the plane-cone configuration: `cosa` and `|sin|` of the
/// semi-angle, `sint` and `cost` of the angle between the plane normal and the
/// axis (in `[0, PI/2]`), `costa = cos(t + angle)` (zero when the plane contains a
/// generatrix direction), and the plane frame `axex`, `axey`, `normp`.
struct ConeSection {
    apex: Point3,
    dist: f64,
    normp: Xyz,
    axex: Xyz,
    axey: Xyz,
    angl: f64,
    cosa: f64,
    sina: f64,
    sint: f64,
    cost: f64,
    costa: f64,
}

impl QuadQuadGeo {
    /// Perform(gp_Pln, gp_Cone, Tolang, Tol): with the apex in the plane (within
    /// `tol`) a point, one generatrix or two; otherwise a hyperbola (axis in the
    /// plane), a parabola (plane parallel to a generatrix), a circle (plane normal
    /// to the axis), a hyperbola or an ellipse. Ellipses above 1e9 and hyperbolas
    /// above 2e6 leave the intersection not done (for the biparametric method).
    /// `inter.perform_plane_cone(plane, cone, 1e-12, 1e-7)?`.
    pub fn perform_plane_cone(
        &mut self,
        plane: Plane3,
        cone: Cone3,
        tolang: f64,
        tol: f64,
    ) -> Result<(), Failure> {
        self.done = false;
        self.nbint = 0;
        let section: ConeSection = cone_section(plane, cone)?;
        if section.dist.abs() < tol {
            self.apex_in_plane(plane, cone, &section, tolang)?;
        } else if section.cost < tolang {
            self.axis_in_plane(&section)?;
        } else {
            self.conic_section(plane, cone, section, tolang)?;
        }
        self.done = !self.beyond_limits();
        Ok(())
    }

    /// The generatrices through the apex of a plane containing it.
    fn apex_in_plane(
        &mut self,
        plane: Plane3,
        cone: Cone3,
        s: &ConeSection,
        tolang: f64,
    ) -> Result<(), Failure> {
        if s.costa.abs() < tolang {
            // Plane parallel to the generatrix: the projection of the axis.
            self.typeres = ResultType::Line;
            self.nbint = 1;
            let [a, b, c, d] = plane.coefficients()?;
            let mut on_axis: Xyz = s.apex.xyz() + 10. * cone.axis().direction().xyz();
            let dist: f64 = a * on_axis.x() + b * on_axis.y() + c * on_axis.z() + d;
            on_axis -= dist * s.normp;
            self.pt1 = s.apex;
            self.dir1.set_xyz(on_axis - self.pt1.xyz())?;
        } else if s.cost < s.sina {
            // Plane "interior" to the cone.
            self.typeres = ResultType::Line;
            self.nbint = 2;
            self.pt1 = s.apex;
            self.pt2 = s.apex;
            let dh: f64 = (s.sina * s.sina - s.cost * s.cost).sqrt() / s.cosa;
            self.dir1.set_xyz(s.axex + dh * s.axey)?;
            self.dir2.set_xyz(s.axex - dh * s.axey)?;
        } else {
            // Plane "exterior" to the cone.
            self.typeres = ResultType::Point;
            self.nbint = 1;
            self.pt1 = s.apex;
        }
        Ok(())
    }

    /// The hyperbola of a plane that contains the direction of the axis.
    fn axis_in_plane(&mut self, s: &ConeSection) -> Result<(), Failure> {
        self.typeres = ResultType::Hyperbola;
        self.nbint = 2;
        self.pt1.set_xyz(s.apex.xyz() - s.dist * s.normp);
        self.pt2 = self.pt1;
        self.dir1 = Direction3::from_xyz(s.normp)?;
        self.dir2.set_xyz(s.axex)?;
        self.param1 = (s.dist / s.angl.tan()).abs();
        self.param2 = self.param1;
        self.param1bis = s.dist.abs();
        self.param2bis = self.param1bis;
        Ok(())
    }

    /// The parabola, circle, hyperbola or ellipse around the axis point on the
    /// plane; the frame turns to -x, -y when that point is behind the apex.
    fn conic_section(
        &mut self,
        plane: Plane3,
        cone: Cone3,
        mut s: ConeSection,
        tolang: f64,
    ) -> Result<(), Failure> {
        let inter =
            IntConicQuad::line_plane(Line3::from_axis(cone.axis()), plane, tolang, 0.0, 0.0)?;
        let center: Point3 = inter.point(1)?;
        let distance: f64 = s.apex.distance(&center);
        // The parameter of the apex on the axis is negative (see the cone definition).
        if inter.param_on_conic(1)? + cone.ref_radius() / s.angl.tan() < 0. {
            s.axex.reverse();
            s.axey.reverse();
        }
        if s.costa.abs() < tolang {
            self.typeres = ResultType::Parabola;
            self.nbint = 1;
            let deltacenter: f64 = distance / 2. / s.cosa;
            s.axex.normalize()?;
            self.pt1.set_xyz(center.xyz() - deltacenter * s.axex);
            self.set_section_frame(&s)?;
            self.param1 = deltacenter * s.sina * s.sina;
        } else if s.sint < tolang {
            self.typeres = ResultType::Circle;
            self.nbint = 1;
            self.pt1 = center;
            self.dir1 = cone.position().direction();
            self.dir2 = cone.position().x_direction();
            self.param1 = s.apex.distance(&center) * s.angl.tan().abs();
        } else if s.cost < s.sina {
            self.section_hyperbola(&mut s, center, distance)?;
        } else {
            self.section_ellipse(&mut s, center, distance)?;
        }
        Ok(())
    }

    /// The two branches of the hyperbola of a steep plane.
    fn section_hyperbola(
        &mut self,
        s: &mut ConeSection,
        center: Point3,
        distance: f64,
    ) -> Result<(), Failure> {
        let (sina, cost, sint, cosa) = (s.sina, s.cost, s.sint, s.cosa);
        self.typeres = ResultType::Hyperbola;
        self.nbint = 2;
        s.axex.normalize()?;
        let deltacenter: f64 = sint * sina * sina * distance / (sina * sina - cost * cost);
        self.pt1.set_xyz(center.xyz() - deltacenter * s.axex);
        self.pt2 = self.pt1;
        self.set_section_frame(s)?;
        self.param1 = cost * sina * cosa * distance / (sina * sina - cost * cost);
        self.param2 = self.param1;
        self.param1bis = cost * sina * distance / (sina * sina - cost * cost).sqrt();
        self.param2bis = self.param1bis;
        Ok(())
    }

    /// The ellipse of a shallow plane (`cost > sina`).
    fn section_ellipse(
        &mut self,
        s: &mut ConeSection,
        center: Point3,
        distance: f64,
    ) -> Result<(), Failure> {
        let (sina, cost, sint, cosa) = (s.sina, s.cost, s.sint, s.cosa);
        self.typeres = ResultType::Ellipse;
        self.nbint = 1;
        let radius: f64 = cost * sina * cosa * distance / (cost * cost - sina * sina);
        let deltacenter: f64 = sint * sina * sina * distance / (cost * cost - sina * sina);
        s.axex.normalize()?;
        self.pt1.set_xyz(center.xyz() + deltacenter * s.axex);
        self.set_section_frame(s)?;
        self.param1 = radius;
        self.param1bis = cost * sina * distance / (cost * cost - sina * sina).sqrt();
        Ok(())
    }

    /// `dir1 = normp`, `dir2 = axex`.
    fn set_section_frame(&mut self, s: &ConeSection) -> Result<(), Failure> {
        self.dir1 = Direction3::from_xyz(s.normp)?;
        self.dir2.set_xyz(s.axex)
    }

    /// Ellipses and hyperbolas too extreme to handle (Value, ProjLib, ...).
    fn beyond_limits(&self) -> bool {
        let ellipse: bool = self.typeres == ResultType::Ellipse
            && self.nbint >= 1
            && (self.param1.abs() > ELLIPSE_LIMIT || self.param1bis.abs() > ELLIPSE_LIMIT);
        let hyperbola2: bool = self.typeres == ResultType::Hyperbola
            && self.nbint >= 2
            && (self.param2.abs() > HYPERBOLA_LIMIT || self.param2bis.abs() > HYPERBOLA_LIMIT);
        let hyperbola1: bool = self.typeres == ResultType::Hyperbola
            && self.nbint >= 1
            && (self.param1.abs() > HYPERBOLA_LIMIT || self.param1bis.abs() > HYPERBOLA_LIMIT);
        ellipse || hyperbola2 || hyperbola1
    }
}

/// The plane-cone angles: `normp` is the plane normal (reversed for an indirect
/// plane), `axey = normp ^ axis`, `axex = axey ^ normp`.
fn cone_section(plane: Plane3, cone: Cone3) -> Result<ConeSection, Failure> {
    let [a, b, c, d] = plane.coefficients()?;
    let apex: Point3 = cone.apex();
    // The signed distance from the apex to the plane.
    let dist: f64 = a * apex.x() + b * apex.y() + c * apex.z() + d;
    let mut normp: Xyz = plane.axis().direction().xyz();
    if !plane.direct()? {
        normp.reverse();
    }
    let axis: Xyz = cone.axis().direction().xyz();
    let axey: Xyz = normp.crossed(axis);
    let axex: Xyz = axey.crossed(normp);
    let angl: f64 = cone.semi_angle();
    let (cosa, sina) = (angl.cos(), angl.sin().abs());
    let sint: f64 = axey.modulus();
    let cost: f64 = axis.dot(normp).abs();
    Ok(ConeSection {
        apex,
        dist,
        normp,
        axex,
        axey,
        angl,
        cosa,
        sina,
        sint,
        cost,
        // sin((PI/2 - t) - angl) = cos(t + angl), t brought back into [0, PI/2].
        costa: cost * cosa - sint * sina,
    })
}
