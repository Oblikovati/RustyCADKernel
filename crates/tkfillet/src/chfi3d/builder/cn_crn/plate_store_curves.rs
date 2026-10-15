// Rust translation of the storage of the boundaries of the plate in
// ChFi3d_Builder::PerformMoreThreeCorner, and of its partial result
// (ChFi3d_Builder_CnCrn.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The boundaries of the plate stored with their pcurves on it, in the order they were
//! given to the plate (the contours, the connections, the projections and the free
//! borders), their tolerances raised by the approximation error, and the regularities
//! of the G1 boundaries.
use super::plate_store::Plate;
use super::statics::plate_pcurve_at;
use super::{Corner, indices, slot};
use crate::chfi_ds::{Regul, lock};
use crate::chfi3d::builder::ChFi3dBuilder;
use crate::chfi3d::{fil_curve_in_ds, fil_point_in_ds, surf_data_at};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeomalgo::geom_plate::GeomPlateBuildPlateSurface;

/// The regularity of a curve between the plate and a fillet or a face.
fn regul(curve: i32, isurf: i32, (s2, s2_face): (i32, bool)) -> Regul {
    let mut r = Regul::new();
    r.set_curve(curve);
    r.set_s1(isurf, false);
    r.set_s2(s2, s2_face);
    r
}

impl ChFi3dBuilder {
    /// The tolerance of the point raised by `tol`.
    fn raise_point(&mut self, ip: i32, tol: f64) -> Result<(), Failure> {
        let p = self.ds.change_point(ip)?;
        p.set_tolerance(p.tolerance() + tol);
        Ok(())
    }

    /// The pcurve of the `n3d`-th boundary on the plate stored.
    fn plate_pcurve(
        &mut self,
        c: &Corner,
        psurf: &GeomPlateBuildPlateSurface,
        plate: &Plate,
    ) -> Result<i32, Failure> {
        let icurve = c.indcurve3d[c.n3d as usize];
        let pc = plate_pcurve_at(&psurf.curves2d()?, c.n3d)?;
        let interfc = fil_curve_in_ds(icurve, plate.isurf, &pc, plate.orien);
        self.ds
            .change_surface_interferences(plate.isurf)
            .append(interfc);
        Ok(icurve)
    }

    /// The `n3d`-th boundary: its tolerance raised and its pcurve on the plate stored.
    fn next_boundary(
        &mut self,
        c: &mut Corner,
        psurf: &GeomPlateBuildPlateSurface,
        plate: &Plate,
    ) -> Result<i32, Failure> {
        c.n3d += 1;
        let icurve = *slot(&mut c.indcurve3d, c.n3d)?;
        let tc = self.ds.change_curve(icurve);
        tc.set_tolerance(tc.tolerance() + plate.apperror);
        self.plate_pcurve(c, psurf, plate)
    }

    /// The curves of the plate stored, in the order of its boundaries.
    pub(super) fn store_plate_curves(
        &mut self,
        c: &mut Corner,
        psurf: &GeomPlateBuildPlateSurface,
        plate: &Plate,
    ) -> Result<(), Failure> {
        c.n3d = 0;
        self.store_stripe_curves(c, psurf, plate)?;
        self.store_connection_curves(c, psurf, plate)?;
        self.store_projection_curves(c, psurf, plate)?;
        if c.droit {
            let nb_libre = (0..c.nedge).filter(|&ic| c.is_libre_edge(ic)).count();
            for _ in 0..nb_libre {
                self.next_boundary(c, psurf, plate)?;
            }
        }
        Ok(())
    }

    /// The boundaries on the contours: the points of their curves, their tolerances,
    /// and the G1 regularity with the fillet.
    fn store_stripe_curves(
        &mut self,
        c: &mut Corner,
        psurf: &GeomPlateBuildPlateSurface,
        plate: &Plate,
    ) -> Result<(), Failure> {
        for ic in 0..c.nedge {
            if c.sharp[ic] {
                continue;
            }
            c.n3d += 1;
            let (_, icmoins) = indices(c.nedge, ic);
            let isfirst = c.sens[ic] == 1;
            let pc = lock(c.cd(ic)?).pcurve(isfirst).clone();
            let pc = pc.get().ok_or_else(|| {
                Failure::with_message("Standard_NullObject", "PerformMoreThreeCorner")
            })?;
            let (pardeb, parfin) = (pc.first_parameter(), pc.last_parameter());
            let icurve = *slot(&mut c.indcurve3d, c.n3d)?;
            let i1 = fil_point_in_ds(
                Orientation::Forward,
                icurve,
                c.indpoint[ic][0],
                pardeb,
                false,
            );
            let i2 = fil_point_in_ds(
                Orientation::Reversed,
                icurve,
                c.indpoint[ic][1],
                parfin,
                false,
            );
            self.ds.change_curve_interferences(icurve).append(i1);
            self.ds.change_curve_interferences(icurve).append(i2);
            self.ds
                .change_curve(icurve)
                .set_tolerance(c.errapp[ic] + plate.apperror);
            self.raise_point(c.indpoint[ic][0], plate.apperror)?;
            self.raise_point(c.indpoint[ic][1], plate.apperror)?;
            self.plate_pcurve(c, psurf, plate)?;
            let indice = lock(&surf_data_at(c.cd(ic)?, c.i[ic][icmoins])?).surf();
            self.my_regul
                .push(regul(icurve, plate.isurf, (indice, false)));
        }
        Ok(())
    }

    /// The sharp edge `ic` limited at the end of its connection.
    fn limit_sharp_edge(&mut self, c: &Corner, ic: usize) -> Result<(), Failure> {
        let (icplus, _) = indices(c.nedge, ic);
        let e = &c.evive[ic];
        let sommet1 = BRepTool::pnt(&c.v1)?;
        let pf = BRepTool::pnt(&topexp::first_vertex(e, false))?;
        let pl = BRepTool::pnt(&topexp::last_vertex(e, false))?;
        let ori = if pf.distance(&sommet1) < pl.distance(&sommet1) {
            Orientation::Forward
        } else {
            Orientation::Reversed
        };
        let iarc1 = self.ds.add_shape(e.as_ref());
        let interf = fil_point_in_ds(ori, iarc1, c.indpoint[ic][1], c.p[ic][icplus], false);
        self.ds
            .change_shape_interferences(e.as_ref())
            .append(interf);
        Ok(())
    }

    /// The connections: the sharp edges limited at them, and the actual connections
    /// stored (G1 regularity with the face when their constraint is G1).
    fn store_connection_curves(
        &mut self,
        c: &mut Corner,
        psurf: &GeomPlateBuildPlateSurface,
        plate: &Plate,
    ) -> Result<(), Failure> {
        for ic in 0..c.nedge {
            if c.oksea[ic] {
                continue;
            }
            let (icplus, _) = indices(c.nedge, ic);
            if c.sharp[ic] && !c.deuxconges {
                self.limit_sharp_edge(c, ic)?;
            }
            if c.ponctuel[ic] || c.libre[ic] || c.moresurf[ic] {
                continue;
            }
            c.n3d += 1;
            let icurve = *slot(&mut c.indcurve3d, c.n3d)?;
            let tc = self.ds.change_curve(icurve);
            tc.set_tolerance(tc.tolerance() + plate.apperror);
            if !c.deuxconges {
                self.raise_point(c.indpoint[ic][1], plate.apperror)?;
                self.raise_point(c.indpoint[icplus][0], plate.apperror)?;
            }
            self.plate_pcurve(c, psurf, plate)?;
            if c.order[c.n3d as usize] == 1 {
                self.my_regul
                    .push(regul(icurve, plate.isurf, (c.numfa[ic][icplus], true)));
            }
        }
        Ok(())
    }

    /// The projections on several faces (G1 regularity with their face).
    fn store_projection_curves(
        &mut self,
        c: &mut Corner,
        psurf: &GeomPlateBuildPlateSurface,
        plate: &Plate,
    ) -> Result<(), Failure> {
        for ic in 0..c.nedge {
            if !c.moresurf[ic] {
                continue;
            }
            for nb in 1..=c.nbface.max(0) as usize {
                let eproj = c
                    .eproj
                    .get(nb - 1)
                    .cloned()
                    .ok_or_else(|| Failure::out_of_range("PerformMoreThreeCorner: Eproj"))?;
                if eproj.is_null() {
                    continue;
                }
                c.n3d += 1;
                let icurve = *slot(&mut c.indcurve3d, c.n3d)?;
                let tc = self.ds.change_curve(icurve);
                tc.set_tolerance(tc.tolerance() + plate.apperror);
                let indice = *slot(&mut c.indice, c.n3d)?;
                if indice != 0 {
                    self.raise_point(indice, plate.apperror)?;
                }
                self.plate_pcurve(c, psurf, plate)?;
                if c.order[c.n3d as usize] == 1 {
                    let fproj =
                        c.fproj.get(nb - 1).cloned().ok_or_else(|| {
                            Failure::out_of_range("PerformMoreThreeCorner: Fproj")
                        })?;
                    let iface = self.ds.add_shape(fproj.as_ref());
                    self.my_regul
                        .push(regul(icurve, plate.isurf, (iface, true)));
                }
            }
        }
        Ok(())
    }

    /// Without plate (there is only a partial result): the sharp edges limited at their
    /// connections, `done` false and `hasresult` true.
    pub(super) fn partial_result(&mut self, c: &mut Corner) -> Result<(), Failure> {
        self.done = false;
        self.hasresult = true;
        for ic in 0..c.nedge {
            if c.oksea[ic] {
                continue;
            }
            if c.sharp[ic] && !c.deuxconges {
                self.limit_sharp_edge(c, ic)?;
            }
        }
        Ok(())
    }
}
