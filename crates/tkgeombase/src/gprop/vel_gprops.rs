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
// Rust translation of ModelingData/TKGeomBase/GProp/GProp_VelGProps.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GProp_VelGProps`: global properties of elementary solids.
use super::GProps;
use super::elementary::{Frame, assemble_inertia, local_inertia};
use super::sel_gprops::{local_point, placement_frame, torus_dm};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Cone3, Cylinder3, Point3, Sphere3, Torus3};
use std::ops::Deref;

/// `GProp_VelGProps`: the volume, centre and inertia of a sector of cylinder,
/// cone, sphere or torus (the solid between the surface patch and its axis);
/// the inherited [`GProps`] members are reached through `Deref`.
///
/// ```
/// use rustycad_tkgeombase::gprop::VelGProps;
/// use rustycad_tkmath::gp::{Axis3, Cylinder3, Point3};
/// let cylinder = Cylinder3::new(Axis3::default(), 1.0).unwrap();
/// let props = VelGProps::from_cylinder(cylinder, 0.0, 1.0, 0.0, 2.0, Point3::default()).unwrap();
/// assert_eq!(props.mass(), 1.0);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct VelGProps {
    base: GProps,
}

impl Deref for VelGProps {
    type Target = GProps;
    fn deref(&self) -> &GProps {
        &self.base
    }
}

macro_rules! vel_constructor {
    ($name:ident, $shape:ty, $perform:ident, $doc:literal) => {
        #[doc = $doc]
        pub fn $name(
            solid: $shape,
            a1: f64,
            a2: f64,
            b1: f64,
            b2: f64,
            location: Point3,
        ) -> Result<Self, Failure> {
            let mut props: Self = Self::new();
            props.set_location(location);
            props.$perform(solid, a1, a2, b1, b2)?;
            Ok(props)
        }
    };
}

impl VelGProps {
    /// `GProp_VelGProps()`: an empty framework at the origin; `VelGProps::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    vel_constructor!(
        from_cylinder,
        Cylinder3,
        perform_cylinder,
        "`GProp_VelGProps(S, Alpha1, Alpha2, Z1, Z2, VLocation)` on a cylinder; `VelGProps::from_cylinder(c, 0.0, 1.0, 0.0, 2.0, p)?`."
    );
    vel_constructor!(
        from_cone,
        Cone3,
        perform_cone,
        "`GProp_VelGProps(S, Alpha1, Alpha2, Z1, Z2, VLocation)` on a cone; `VelGProps::from_cone(c, 0.0, 1.0, 0.0, 2.0, p)?`."
    );
    vel_constructor!(
        from_sphere,
        Sphere3,
        perform_sphere,
        "`GProp_VelGProps(S, Teta1, Teta2, Alpha1, Alpha2, VLocation)` on a sphere; `VelGProps::from_sphere(s, 0.0, 1.0, -0.5, 0.5, p)?`."
    );
    vel_constructor!(
        from_torus,
        Torus3,
        perform_torus,
        "`GProp_VelGProps(S, Teta1, Teta2, Alpha1, Alpha2, VLocation)` on a torus; `VelGProps::from_torus(t, 0.0, 1.0, 0.0, 1.0, p)?`."
    );

    /// `SetLocation(VLocation)`: the point the inertia is accumulated at;
    /// `props.set_location(p)`.
    pub fn set_location(&mut self, location: Point3) {
        self.base.loc = location;
    }

    /// `Perform(S, Alpha1, Alpha2, Z1, Z2)` on a cylinder sector; StdFail_NotDone
    /// when math_Jacobi fails; `props.perform_cylinder(c, 0.0, 1.0, 0.0, 2.0)?`.
    pub fn perform_cylinder(
        &mut self,
        solid: Cylinder3,
        alpha1: f64,
        alpha2: f64,
        z1: f64,
        z2: f64,
    ) -> Result<(), Failure> {
        let frame: Frame = placement_frame(solid.position());
        let origin: Point3 = solid.location();
        let rayon: f64 = solid.radius();
        let base: &mut GProps = &mut self.base;
        base.dim = rayon * rayon * (z2 - z1) / 2.0;
        let (sa2, sa1, ca2, ca1) = (alpha2.sin(), alpha1.sin(), alpha2.cos(), alpha1.cos());
        let (dsin, dcos): (f64, f64) = (sa2 - sa1, ca1 - ca2);
        let coef: f64 = rayon / (alpha2 - alpha1);
        let [a1, a2, a3] = frame.map(|d| d.xyz());
        let half: f64 = z2 + z1;
        base.g.set_coordinates(
            origin.x() + (coef * (a1.x() * dsin + a2.x() * dcos)) + (a3.x() * half / 2.0),
            origin.y() + (coef * (a1.y() * dsin + a2.y() * dcos)) + (a3.y() * half / 2.0),
            origin.z() + (coef * (a1.z() * dsin + a2.z() * dcos)) + (a3.z() * half / 2.0),
        );
        let icn2: f64 = base.dim / 2.0 * (alpha2 - alpha1 + sa2 * ca2 - sa1 * ca1);
        let isn2: f64 = base.dim / 2.0 * (alpha2 - alpha1 - sa2 * ca2 + sa1 * ca1);
        let iz2: f64 = base.dim * (alpha2 - alpha1) * (z2 * z2 + z1 * z2 + z1 * z1);
        let icnsn: f64 = base.dim * (ca2 * ca2 - ca1 * ca1) / 2.0;
        let icnz: f64 = base.dim * (z2 + z1) / 2.0 * dsin;
        let isnz: f64 = base.dim * (z2 + z1) / 2.0 * dcos;
        base.dim *= alpha2 - alpha1;
        let rr: f64 = rayon * rayon;
        let dm = local_inertia(
            [rr * isn2 + iz2, rr * icn2 + iz2, rr * base.dim],
            [-rr * icnsn, -rayon * icnz, -rayon * isnz],
        );
        assemble_inertia(base, dm, frame, 1.0)
    }

    /// `Perform(S, Alpha1, Alpha2, Z1, Z2)` on a cone sector (Z along the
    /// generatrix); the native ISn2 repeats the ICn2 formula;
    /// `props.perform_cone(c, 0.0, 1.0, 0.0, 2.0)?`.
    pub fn perform_cone(
        &mut self,
        solid: Cone3,
        alpha1: f64,
        alpha2: f64,
        z1: f64,
        z2: f64,
    ) -> Result<(), Failure> {
        let frame: Frame = placement_frame(solid.position());
        let t: f64 = solid.semi_angle();
        let (cnt, snt): (f64, f64) = (t.cos(), t.sin());
        let r: f64 = solid.ref_radius();
        let (sn2, sn1, cn2, cn1) = (alpha2.sin(), alpha1.sin(), alpha2.cos(), alpha1.cos());
        let zz: f64 = (z2 - z1) * (z2 - z1) * cnt * snt;
        let auxi1: f64 = 2.0 * r + (z2 + z1) * snt;
        let base: &mut GProps = &mut self.base;
        base.dim = zz * (alpha2 - alpha1) * auxi1 / 2.0;
        let r1: f64 = r + z1 * snt;
        let r2: f64 = r + z2 * snt;
        let coef0: f64 = r1 * r1 + r1 * r2 + r2 * r2;
        let iz: f64 =
            cnt * (r * (z2 + z1) + 2.0 * snt * (z1 * z1 + z1 * z2 + z2 * z2) / 3.0) / auxi1;
        let ix: f64 = coef0 * (sn2 - sn1) / (alpha2 - alpha1) / auxi1;
        let iy: f64 = coef0 * (cn1 - cn2) / (alpha2 - alpha1) / auxi1;
        base.g = local_point(solid.location(), frame, [ix, iy, iz]);
        let ir2: f64 = zz * (r2 * r2 * r2 + r2 * r2 * r1 + r1 * r1 * r2 + r1 * r1 * r1) / 4.0;
        let icn2: f64 = ir2 * (alpha2 - alpha1 + cn2 * sn2 - cn1 * sn1) / 2.0;
        let isn2: f64 = ir2 * (alpha2 - alpha1 + cn2 * sn2 - cn1 * sn1) / 2.0;
        let iz2: f64 = zz
            * cnt
            * cnt
            * (alpha2 - alpha1)
            * (z1 * z1 * (r / 3.0 + z1 * snt / 4.0)
                + z2 * z2 * (r / 3.0 + z2 * snt / 4.0)
                + z1 * z2 * (r / 3.0 + z1 * snt / 4.0 + z2 * snt / 4.0));
        let icnsn: f64 = ir2 * (cn2 * cn2 - cn1 * cn1);
        let icnz: f64 = (z1 + z2) * zz * coef0 * (sn2 - sn1) / 3.0;
        let isnz: f64 = (z1 + z2) * zz * coef0 * (cn1 - cn2) / 3.0;
        let dm = local_inertia(
            [isn2 + iz2, icn2 + iz2, ir2 * (alpha2 - alpha1)],
            [-icnsn, -icnz, -isnz],
        );
        assemble_inertia(base, dm, frame, 1.0)
    }

    /// `Perform(S, Teta1, Teta2, Alpha1, Alpha2)` on a sphere sector;
    /// `props.perform_sphere(s, 0.0, 1.0, -0.5, 0.5)?`.
    pub fn perform_sphere(
        &mut self,
        solid: Sphere3,
        teta1: f64,
        teta2: f64,
        alpha1: f64,
        alpha2: f64,
    ) -> Result<(), Failure> {
        let frame: Frame = placement_frame(solid.position());
        let r: f64 = solid.radius();
        let (cnt1, snt1, cnt2, snt2) = (teta1.cos(), teta1.sin(), teta2.cos(), teta2.sin());
        let (cnf1, snf1, cnf2, snf2) = (alpha1.cos(), alpha1.sin(), alpha2.cos(), alpha2.sin());
        let base: &mut GProps = &mut self.base;
        base.dim = (teta2 - teta1) * r * r * r * (snf2 - snf1) / 3.0;
        let lat: f64 = alpha2 - alpha1 + snf2 * cnf2 - snf1 * cnf1;
        let ix: f64 = r * (snt2 - snt1) / (teta2 - teta1) * lat / (snf2 - snf1) / 2.0;
        let iy: f64 = r * (cnt1 - cnt2) / (teta2 - teta1) * lat / (snf2 - snf1) / 2.0;
        let iz: f64 = r * (snf2 + snf1) / 2.0;
        base.g = local_point(solid.location(), frame, [ix, iy, iz]);
        let ir2: f64 =
            (cnf2 * snf2 * (cnf2 + 1.0) - cnf1 * snf1 * (cnf1 + 1.0) + alpha2 - alpha1) / 9.0;
        let icn2: f64 = (teta2 - teta1 + cnt2 * snt2 - cnt1 * snt1) * ir2 / 2.0;
        let isn2: f64 = (teta2 - teta1 - cnt2 * snt2 + cnt1 * snt1) * ir2 / 2.0;
        let icnsn: f64 = (snt2 * snt2 - snt1 * snt1) * ir2 / 2.0;
        let iz2: f64 = (teta2 - teta1) * (snf2 * snf2 * snf2 - snf1 * snf1 * snf1) / 9.0;
        let cube: f64 = cnf1 * cnf1 * cnf1 - cnf2 * cnf2 * cnf2;
        let icnz: f64 = (snt2 - snt1) * cube / 9.0;
        let isnz: f64 = (cnt1 - cnt2) * cube / 9.0;
        let dm = local_inertia(
            [isn2 + iz2, icn2 + iz2, ir2 * (teta2 - teta1)],
            [-icnsn, -icnz, -isnz],
        );
        assemble_inertia(base, dm, frame, r * r * r * r * r)
    }

    /// `Perform(S, Teta1, Teta2, Alpha1, Alpha2)` on a torus sector.
    ///
    /// The native code takes `Cnt2`/`Snt2` from `Alpha2` instead of `Teta2`, and
    /// its IR2 adds `Rmin² / 2` to the surface formula; both are kept;
    /// `props.perform_torus(t, 0.0, 1.0, 0.0, 1.0)?`.
    pub fn perform_torus(
        &mut self,
        solid: Torus3,
        teta1: f64,
        teta2: f64,
        alpha1: f64,
        alpha2: f64,
    ) -> Result<(), Failure> {
        let frame: Frame = placement_frame(solid.position());
        let (rmax, rmin): (f64, f64) = (solid.major_radius(), solid.minor_radius());
        let (cnt1, snt1) = (teta1.cos(), teta1.sin());
        let (cnt2, snt2) = (alpha2.cos(), alpha2.sin());
        let (snf2, snf1, cnf1) = (alpha2.sin(), alpha1.sin(), alpha1.cos());
        let cnf2: f64 = alpha2.cos();
        let base: &mut GProps = &mut self.base;
        base.dim = rmax * rmin * rmin * (teta2 - teta1) * (alpha2 - alpha1) / 2.0;
        let ring: f64 = rmin * (snf2 - snf1) / (alpha2 - alpha1) + rmax;
        let ix: f64 = (snt2 - snt1) / (teta2 - teta1) * ring;
        let iy: f64 = (cnt1 - cnt2) / (teta2 - teta1) * ring;
        let iz: f64 = rmin * (cnf1 - cnf2) / (alpha2 - alpha1);
        base.g = local_point(solid.location(), frame, [ix, iy, iz]);
        let ir2: f64 = rmax * rmax
            + rmin * rmin / 2.0
            + 2.0 * rmax * rmin * (snf2 - snf1)
            + rmin * rmin / 2.0 * (snf2 * cnf2 - snf1 * cnf1);
        let dm = torus_dm(
            ir2,
            [rmax, rmin],
            [teta1, teta2],
            [snt2, snt1, cnt2, cnt1],
            [alpha1, alpha2],
        );
        assemble_inertia(base, dm, frame, rmax * rmin * rmin / 2.0)
    }
}
