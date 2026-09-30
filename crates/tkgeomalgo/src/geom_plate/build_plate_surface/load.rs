// Copyright (c) 1996-1999 Matra Datavision
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
// Rust translation of LoadCurve and LoadPoint of
// ModelingAlgorithms/TKGeomAlgo/GeomPlate/GeomPlate_BuildPlateSurface.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The loading of the constraint points into the plate.
use super::GeomPlateBuildPlateSurface;
use crate::geom_plate::live;
use crate::plate::{
    PlateD1, PlateD2, PlateFreeGtoCConstraint, PlateGtoCConstraint, PlatePinpointConstraint,
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::geom::{GeomSurfaceD1, GeomSurfaceD2};
use rustycad_tkmath::gp::{Point2, Point3, Vector3, Xyz};

/// The derivatives of the target and the initial surface at a constraint point.
enum TargetJet {
    G1(GeomSurfaceD1),
    G2(GeomSurfaceD2),
}

/// `Plate_D2(V5, V6, V7)` of a D2 output `(D2U, D2V, D2UV)`: the native code passes
/// D2V as `duv` and D2UV as `dvv`, for the target and the initial surface alike.
fn plate_d2(d2: &GeomSurfaceD2) -> PlateD2 {
    PlateD2::new(*d2.d2u.xyz(), *d2.d2v.xyz(), *d2.d2uv.xyz())
}

fn plate_d1(du: Vector3, dv: Vector3) -> PlateD1 {
    PlateD1::new(*du.xyz(), *dv.xyz())
}

impl GeomPlateBuildPlateSurface {
    /// Loads the G0 pinpoint constraint `p3d - S0(p2d)` at `p2d`.
    fn load_g0(&mut self, p2d: Point2, p3d: Point3) -> Result<(), Failure> {
        let pp: Point3 = self.surf_init_ref()?.d0(p2d.x(), p2d.y())?;
        let pdif = Xyz::new(-pp.x() + p3d.x(), -pp.y() + p3d.y(), -pp.z() + p3d.z());
        self.plate
            .load_pinpoint(&PlatePinpointConstraint::new(p2d.xy(), pdif, 0, 0));
        Ok(())
    }

    /// The G1 constraint of a curve point in the later loops of a free plate: the
    /// derivatives of the previous plate corrected along the target normal.
    fn load_g1_incremental(
        &mut self,
        p2d: Point2,
        target: &GeomSurfaceD1,
        init: &GeomSurfaceD1,
    ) -> Result<(), Failure> {
        let mut normal: Vector3 = target.d1u ^ target.d1v;
        let norm: f64 = normal.magnitude();
        if norm > 1.0e-12 {
            normal /= norm;
        }
        let der_plate_u = Vector3::from(self.prev_plate.evaluate_derivative(p2d.xy(), 1, 0)?);
        let der_plate_v = Vector3::from(self.prev_plate.evaluate_derivative(p2d.xy(), 0, 1)?);
        let mut du = Vector3::new(0.0, 0.0, 0.0);
        let mut dv = Vector3::new(0.0, 0.0, 0.0);
        du.set_scaled_sum(-(init.d1u + der_plate_u).dot(normal), normal, der_plate_u);
        dv.set_scaled_sum(-(init.d1v + der_plate_v).dot(normal), normal, der_plate_v);
        self.plate
            .load_pinpoint(&PlatePinpointConstraint::new(p2d.xy(), *du.xyz(), 1, 0));
        self.plate
            .load_pinpoint(&PlatePinpointConstraint::new(p2d.xy(), *dv.xyz(), 0, 1));
        Ok(())
    }

    /// Loads the G1 (or G2) matching constraint at `p2d`: a fixed one, a free one in
    /// the first loop of a non-linear plate, or pinpoints on the previous plate after
    /// (G1 curve points only, `nb_boucle` set).
    fn load_tangency(
        &mut self,
        p2d: Point2,
        target: TargetJet,
        nb_boucle: Option<i32>,
    ) -> Result<(), Failure> {
        let surf_init = self.surf_init.clone();
        let init = live(&surf_init, "the initial surface")?;
        match target {
            TargetJet::G1(d1) => {
                let init_d1 = init.d1(p2d.x(), p2d.y())?;
                let (d1final, d1init) =
                    (plate_d1(d1.d1u, d1.d1v), plate_d1(init_d1.d1u, init_d1.d1v));
                if !self.free {
                    return self.plate.load_gtoc(&PlateGtoCConstraint::g1(
                        p2d.xy(),
                        &d1init,
                        &d1final,
                    ));
                }
                if nb_boucle.is_none_or(|n| n == 1) {
                    let free = PlateFreeGtoCConstraint::g1(p2d.xy(), &d1init, &d1final, 1.0, 0)?;
                    return self.plate.load_free_gtoc(&free);
                }
                self.load_g1_incremental(p2d, &d1, &init_d1)
            }
            TargetJet::G2(d2) => {
                let init_d2 = init.d2(p2d.x(), p2d.y())?;
                let (d1final, d1init) =
                    (plate_d1(d2.d1u, d2.d1v), plate_d1(init_d2.d1u, init_d2.d1v));
                let (d2final, d2init) = (plate_d2(&d2), plate_d2(&init_d2));
                let gcc = PlateGtoCConstraint::g2(p2d.xy(), &d1init, &d1final, &d2init, &d2final)?;
                self.plate.load_gtoc(&gcc)
            }
        }
    }

    /// Loads the points of `myPlateCont` of each curve (`LoadCurve`): a G0 pinpoint,
    /// then the G1 or G2 matching up to `order_max`.
    pub(super) fn load_curve(&mut self, nb_boucle: i32, order_max: i32) -> Result<(), Failure> {
        for i in 1..=self.nt_lin_cont() {
            let cc_handle = self.lin_cont[(i - 1) as usize].clone();
            let cc = live(&cc_handle, "a curve constraint")?;
            if cc.order() == -1 {
                continue;
            }
            let tang: i32 = cc.order().min(order_max);
            if tang == -1 {
                continue;
            }
            let params: Vec<f64> = self
                .plate_cont
                .as_ref()
                .map_or_else(Vec::new, |p| p[(i - 1) as usize].clone());
            for u in params {
                let p3d: Point3 = cc.d0(u)?;
                let p2d: Point2 = self.uv_on_init(cc, u, || Ok(p3d))?;
                self.load_g0(p2d, p3d)?;
                if tang == 1 {
                    self.load_tangency(p2d, TargetJet::G1(cc.d1(u)?), Some(nb_boucle))?;
                }
                if tang == 2 {
                    self.load_tangency(p2d, TargetJet::G2(cc.d2(u)?), None)?;
                }
            }
        }
        Ok(())
    }

    /// Loads the point constraints (`LoadPoint`; the loop number is unused): a G0
    /// pinpoint, then the G1 (fixed or free) or G2 matching up to `order_max`.
    pub(super) fn load_point(&mut self, _nb_boucle: i32, order_max: i32) -> Result<(), Failure> {
        for i in 1..=self.nt_pnt_cont() {
            let pnt = self.pnt_cont[(i - 1) as usize].clone();
            let pnt = live(&pnt, "a point constraint")?;
            let p3d: Point3 = pnt.d0();
            let p2d: Point2 = pnt.pnt2d_on_surf();
            self.load_g0(p2d, p3d)?;
            let tang: i32 = pnt.order().min(order_max);
            if tang == 1 {
                let (p, v1, v2) = pnt.d1();
                let target = GeomSurfaceD1 {
                    point: p,
                    d1u: v1,
                    d1v: v2,
                };
                self.load_tangency(p2d, TargetJet::G1(target), None)?;
            }
            if tang == 2 {
                self.load_tangency(p2d, TargetJet::G2(pnt.d2()), None)?;
            }
        }
        Ok(())
    }
}
