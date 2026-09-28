// Copyright (c) 1995-1999 Matra Datavision
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
// Rust translation of ModelingData/TKGeomBase/GProp/GProp_SelGProps.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GProp_SelGProps`: global properties of bounded elementary surfaces.
use super::GProps;
use super::elementary::{Frame, assemble_inertia, local_inertia};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Axis3, Cone3, Cylinder3, Point3, Sphere3, Torus3, Xyz};
use std::ops::Deref;

/// `GProp_SelGProps`: the area, centre and inertia of a patch of cylinder, cone,
/// sphere or torus; the inherited [`GProps`] members are reached through `Deref`.
///
/// ```
/// use rustycad_tkgeombase::gprop::SelGProps;
/// use rustycad_tkmath::gp::{Axis3, Cylinder3, Point3};
/// let cylinder = Cylinder3::new(Axis3::default(), 1.0).unwrap();
/// let props = SelGProps::from_cylinder(cylinder, 0.0, 1.0, 0.0, 2.0, Point3::default()).unwrap();
/// assert_eq!(props.mass(), 2.0);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SelGProps {
    base: GProps,
}

impl Deref for SelGProps {
    type Target = GProps;
    fn deref(&self) -> &GProps {
        &self.base
    }
}

/// The frame directions of a placement (X, Y and main directions).
pub(super) fn placement_frame(position: Axis3) -> Frame {
    [
        position.x_direction(),
        position.y_direction(),
        position.direction(),
    ]
}

/// `X0 + Ix Xa1 + Iy Xa2 + Iz Xa3` (and Y, Z): the centre from its local
/// coordinates, in the native operation order.
pub(super) fn local_point(origin: Point3, frame: Frame, [ix, iy, iz]: [f64; 3]) -> Point3 {
    let [a1, a2, a3] = frame.map(|d| d.xyz());
    let component = |o: f64, c: fn(&Xyz) -> f64| o + ix * c(&a1) + iy * c(&a2) + iz * c(&a3);
    Point3::new(
        component(origin.x(), Xyz::x),
        component(origin.y(), Xyz::y),
        component(origin.z(), Xyz::z),
    )
}

// The four angle functions of [angle1, angle2]: (sin a2, sin a1, cos a2, cos a1).
fn trig(angle1: f64, angle2: f64) -> (f64, f64, f64, f64) {
    (angle2.sin(), angle1.sin(), angle2.cos(), angle1.cos())
}

macro_rules! sel_constructor {
    ($name:ident, $shape:ty, $perform:ident, $doc:literal) => {
        #[doc = $doc]
        pub fn $name(
            surface: $shape,
            a1: f64,
            a2: f64,
            b1: f64,
            b2: f64,
            location: Point3,
        ) -> Result<Self, Failure> {
            let mut props: Self = Self::new();
            props.set_location(location);
            props.$perform(surface, a1, a2, b1, b2)?;
            Ok(props)
        }
    };
}

impl SelGProps {
    /// `GProp_SelGProps()`: an empty framework at the origin; `SelGProps::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    sel_constructor!(
        from_cylinder,
        Cylinder3,
        perform_cylinder,
        "`GProp_SelGProps(S, Alpha1, Alpha2, Z1, Z2, SLocation)` on a cylinder; `SelGProps::from_cylinder(c, 0.0, 1.0, 0.0, 2.0, p)?`."
    );
    sel_constructor!(
        from_cone,
        Cone3,
        perform_cone,
        "`GProp_SelGProps(S, Alpha1, Alpha2, Z1, Z2, SLocation)` on a cone; `SelGProps::from_cone(c, 0.0, 1.0, 0.0, 2.0, p)?`."
    );
    sel_constructor!(
        from_sphere,
        Sphere3,
        perform_sphere,
        "`GProp_SelGProps(S, Teta1, Teta2, Alpha1, Alpha2, SLocation)` on a sphere; `SelGProps::from_sphere(s, 0.0, 1.0, -0.5, 0.5, p)?`."
    );
    sel_constructor!(
        from_torus,
        Torus3,
        perform_torus,
        "`GProp_SelGProps(S, Teta1, Teta2, Alpha1, Alpha2, SLocation)` on a torus; `SelGProps::from_torus(t, 0.0, 1.0, 0.0, 1.0, p)?`."
    );

    /// `SetLocation(SLocation)`: the point the inertia is accumulated at;
    /// `props.set_location(p)`.
    pub fn set_location(&mut self, location: Point3) {
        self.base.loc = location;
    }

    /// `Perform(S, Alpha1, Alpha2, Z1, Z2)` on a cylinder patch; StdFail_NotDone
    /// when math_Jacobi fails; `props.perform_cylinder(c, 0.0, 1.0, 0.0, 2.0)?`.
    pub fn perform_cylinder(
        &mut self,
        surface: Cylinder3,
        alpha1: f64,
        alpha2: f64,
        z1: f64,
        z2: f64,
    ) -> Result<(), Failure> {
        let frame: Frame = placement_frame(surface.position());
        let r: f64 = surface.radius();
        let base: &mut GProps = &mut self.base;
        base.dim = r * (z2 - z1) * (alpha2 - alpha1);
        let (sa2, sa1, ca2, ca1) = trig(alpha1, alpha2);
        let ix: f64 = r * (sa2 - sa1) / (alpha2 - alpha1);
        let iy: f64 = r * (ca1 - ca2) / (alpha2 - alpha1);
        base.g = local_point(surface.location(), frame, [ix, iy, (z2 + z1) / 2.0]);
        let icn2: f64 = r * r * (alpha2 - alpha1 + sa2 * ca2 - sa1 * ca1) / 2.0;
        let isn2: f64 = r * r * (alpha2 - alpha1 - sa2 * ca2 + sa1 * ca1) / 2.0;
        let iz2: f64 = (alpha2 - alpha1) * (z2 * z2 + z2 * z1 + z1 * z1) / 3.0;
        let icnsn: f64 = r * r * (sa2 * sa2 - sa1 * sa1) / 2.0;
        let icnz: f64 = (z2 + z1) * (sa2 - sa1) / 2.0;
        let isnz: f64 = (z2 + z1) * (ca1 - ca2) / 2.0;
        let dm = local_inertia(
            [isn2 + iz2, icn2 + iz2, alpha2 - alpha1],
            [-icnsn, -icnz, -isnz],
        );
        assemble_inertia(base, dm, frame, r * (z2 - z1))
    }

    /// `Perform(S, Alpha1, Alpha2, Z1, Z2)` on a cone patch (Z along the
    /// generatrix); `props.perform_cone(c, 0.0, 1.0, 0.0, 2.0)?`.
    ///
    /// The native ISn2 repeats the ICn2 formula (`+ Cn2 Sn2 - Cn1 Sn1`).
    pub fn perform_cone(
        &mut self,
        surface: Cone3,
        alpha1: f64,
        alpha2: f64,
        z1: f64,
        z2: f64,
    ) -> Result<(), Failure> {
        let frame: Frame = placement_frame(surface.position());
        let t: f64 = surface.semi_angle();
        let (cnt, snt): (f64, f64) = (t.cos(), t.sin());
        let r: f64 = surface.ref_radius();
        let (sn2, sn1, cn2, cn1) = trig(alpha1, alpha2);
        let auxi1: f64 = r + (z2 + z1) * snt / 2.0;
        let auxi2: f64 = (z2 * z2 + z1 * z2 + z1 * z1) / 3.0;
        let base: &mut GProps = &mut self.base;
        base.dim = (alpha2 - alpha1) * cnt * (z2 - z1) * auxi1;
        let mut ix: f64 = (r * r + r * (z2 + z1) * snt + snt * auxi2) / auxi1;
        let iy: f64 = ix * (cn1 - cn2) / (alpha2 - alpha1);
        ix = ix * (sn2 - sn1) / (alpha2 - alpha1);
        let iz: f64 = cnt * (r * (z2 + z1) / 2.0 + snt * auxi2) / auxi1;
        base.g = local_point(surface.location(), frame, [ix, iy, iz]);
        let r1: f64 = r + z1 * snt;
        let r2: f64 = r + z2 * snt;
        let zz: f64 = (z2 - z1) * cnt;
        let ir2: f64 = zz * snt * (r1 * r1 * r1 + r1 * r1 * r2 + r1 * r2 * r2 + r2 * r2 * r2) / 4.0;
        let icn2: f64 = ir2 * (alpha2 - alpha1 + cn2 * sn2 - cn1 * sn1) / 2.0;
        let isn2: f64 = ir2 * (alpha2 - alpha1 + cn2 * sn2 - cn1 * sn1) / 2.0;
        let iz2: f64 = zz
            * cnt
            * cnt
            * (z2 - z1)
            * (alpha2 - alpha1)
            * (r * auxi2 + snt * (z2 * z2 * z2 + z2 * z2 * z1 + z2 * z1 * z1 + z1 * z1 * z1))
            / 4.0;
        let icnsn: f64 = ir2 * (cn2 * cn2 - cn1 * cn1);
        let icnz: f64 = cnt * snt * zz * (r * (z1 + z2) / 2.0 + auxi2) * (sn2 - sn1);
        let isnz: f64 = cnt * snt * zz * (r * (z1 + z2) / 2.0 + auxi2) * (cn1 - cn2);
        let dm = local_inertia(
            [isn2 + iz2, icn2 + iz2, ir2 * (alpha2 - alpha1)],
            [-icnsn, -icnz, -isnz],
        );
        assemble_inertia(base, dm, frame, 1.0)
    }

    /// `Perform(S, Teta1, Teta2, Alpha1, Alpha2)` on a sphere patch (longitude
    /// Teta, latitude Alpha); `props.perform_sphere(s, 0.0, 1.0, -0.5, 0.5)?`.
    pub fn perform_sphere(
        &mut self,
        surface: Sphere3,
        teta1: f64,
        teta2: f64,
        alpha1: f64,
        alpha2: f64,
    ) -> Result<(), Failure> {
        let frame: Frame = placement_frame(surface.position());
        let r: f64 = surface.radius();
        let (snt2, snt1, cnt2, cnt1) = trig(teta1, teta2);
        let (snf2, snf1, cnf2, cnf1) = trig(alpha1, alpha2);
        let base: &mut GProps = &mut self.base;
        base.dim = r * r * (teta2 - teta1) * (snf2 - snf1);
        let lat: f64 = alpha2 - alpha1 + snf2 * cnf2 - snf1 * cnf1;
        let ix: f64 = r * (snt2 - snt1) / (teta2 - teta1) * lat / (snf2 - snf1) / 2.0;
        let iy: f64 = r * (cnt1 - cnt2) / (teta2 - teta1) * lat / (snf2 - snf1) / 2.0;
        let iz: f64 = r * (snf2 + snf1) / 2.0;
        base.g = local_point(surface.location(), frame, [ix, iy, iz]);
        let ir2: f64 =
            (cnf2 * snf2 * (cnf2 + 1.0) - cnf1 * snf1 * (cnf1 + 1.0) + alpha2 - alpha1) / 3.0;
        let icn2: f64 = (teta2 - teta1 + cnt2 * snt2 - cnt1 * snt1) * ir2 / 2.0;
        let isn2: f64 = (teta2 - teta1 - cnt2 * snt2 + cnt1 * snt1) * ir2 / 2.0;
        let icnsn: f64 = (snt2 * snt2 - snt1 * snt1) * ir2 / 2.0;
        let iz2: f64 = (teta2 - teta1) * (snf2 * snf2 * snf2 - snf1 * snf1 * snf1) / 3.0;
        let cube: f64 = cnf1 * cnf1 * cnf1 - cnf2 * cnf2 * cnf2;
        let icnz: f64 = (snt2 - snt1) * cube / 3.0;
        let isnz: f64 = (cnt1 - cnt2) * cube / 3.0;
        let dm = local_inertia(
            [isn2 + iz2, icn2 + iz2, ir2 * (teta2 - teta1)],
            [-icnsn, -icnz, -isnz],
        );
        assemble_inertia(base, dm, frame, r * r * r * r)
    }

    /// `Perform(S, Teta1, Teta2, Alpha1, Alpha2)` on a torus patch;
    /// `props.perform_torus(t, 0.0, 1.0, 0.0, 1.0)?`.
    pub fn perform_torus(
        &mut self,
        surface: Torus3,
        teta1: f64,
        teta2: f64,
        alpha1: f64,
        alpha2: f64,
    ) -> Result<(), Failure> {
        let frame: Frame = [
            surface.x_axis().direction(),
            surface.y_axis().direction(),
            surface.axis().direction(),
        ];
        let (rmax, rmin): (f64, f64) = (surface.major_radius(), surface.minor_radius());
        let (snt2, snt1, cnt2, cnt1) = trig(teta1, teta2);
        let (snf2, snf1, cnf2, cnf1) = trig(alpha1, alpha2);
        let base: &mut GProps = &mut self.base;
        base.dim = rmax * rmin * (teta2 - teta1) * (alpha2 - alpha1);
        let ring: f64 = rmin * (snf2 - snf1) / (alpha2 - alpha1) + rmax;
        let ix: f64 = (snt2 - snt1) / (teta2 - teta1) * ring;
        let iy: f64 = (cnt1 - cnt2) / (teta2 - teta1) * ring;
        let iz: f64 = rmin * (cnf1 - cnf2) / (alpha2 - alpha1);
        base.g = local_point(surface.location(), frame, [ix, iy, iz]);
        let ir2: f64 = rmax * rmax
            + 2.0 * rmax * rmin * (snf2 - snf1)
            + rmin * rmin / 2.0 * (snf2 * cnf2 - snf1 * cnf1);
        let dm = torus_inertia(ir2, [rmax, rmin], [teta1, teta2, alpha1, alpha2]);
        assemble_inertia(base, dm, frame, rmax * rmin)
    }
}

/// The local inertia `Dm` shared by the torus surface and solid: the angular
/// terms of `IR2`; `angles = [Teta1, Teta2, Alpha1, Alpha2]` with, as the native
/// solid does, `Teta2` possibly replaced in the sines/cosines by the caller.
pub(super) fn torus_dm(
    ir2: f64,
    [rmax, rmin]: [f64; 2],
    [teta1, teta2]: [f64; 2],
    [snt2, snt1, cnt2, cnt1]: [f64; 4],
    [alpha1, alpha2]: [f64; 2],
) -> [[f64; 3]; 3] {
    let (snf2, snf1, cnf2, cnf1) = trig(alpha1, alpha2);
    let icn2: f64 = ir2 * (teta2 - teta1 + snt2 * cnt2 - snt1 * cnt1) / 2.0;
    let isn2: f64 = ir2 * (teta2 - teta1 - snt2 * cnt2 + snt1 * cnt1) / 2.0;
    let icnsn: f64 = ir2 * (snt2 * snt2 - snt1 * snt1) / 2.0;
    let iz2: f64 =
        (teta2 - teta1) * rmin * rmin * (alpha2 - alpha1 - snf2 * cnf2 + snf1 * cnf1) / 2.0;
    let arm: f64 = rmax + rmin * (cnf1 + cnf2) / 2.0;
    let icnz: f64 = rmin * (snt2 - snt1) * (cnf1 - cnf2) * arm;
    let isnz: f64 = rmin * (cnt2 - cnt1) * (cnf1 - cnf2) * arm;
    local_inertia(
        [isn2 + iz2, icn2 + iz2, ir2 * (teta2 - teta1)],
        [-icnsn, -icnz, -isnz],
    )
}

fn torus_inertia(ir2: f64, radii: [f64; 2], [t1, t2, a1, a2]: [f64; 4]) -> [[f64; 3]; 3] {
    let (snt2, snt1, cnt2, cnt1) = trig(t1, t2);
    torus_dm(ir2, radii, [t1, t2], [snt2, snt1, cnt2, cnt1], [a1, a2])
}
