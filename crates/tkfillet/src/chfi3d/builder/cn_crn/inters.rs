// Rust translation of the intersections of neighbouring contours in
// ChFi3d_Builder::PerformMoreThreeCorner (ChFi3d_Builder_CnCrn.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The neighbouring contours that meet on a common face (`oksea`): the parameters of
//! their meeting, and the surface data beyond it removed.
use super::statics::{SurfType, calcul_p2d_on_surf, remove_sd, surf_index};
use super::{Corner, indices};
use crate::chfi3d::builder::ChFi3dBuilder;
use crate::chfi3d::{SearchFd, angle_edge, index_of_surf_data, search_fd};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods;
use rustycad_tkernel::standard::Failure;
use std::f64::consts::PI;

impl ChFi3dBuilder {
    /// `oksea`: for each pair of neighbouring contours, whether they meet (not tried
    /// for tangent edges, cts60046); `inters` false when one pair does not.
    pub(super) fn intersections(&mut self, c: &mut Corner) -> Result<(), Failure> {
        let som = BRepTool::pnt(&c.v1)?;
        for ic in 0..c.nedge {
            let (icplus, _) = indices(c.nedge, ic);
            if c.sharp[ic] || c.sharp[icplus] {
                c.oksea[ic] = false;
            } else {
                c.oksea[ic] = self.meet(c, ic, icplus, som)?;
            }
            if !c.oksea[ic] {
                c.inters = false;
            }
        }
        Ok(())
    }

    /// The meeting of the contours `ic` and `icplus`: the surface data beyond it
    /// removed, the faces they crossed kept in `Fproj`, and the parameters.
    fn meet(
        &mut self,
        c: &mut Corner,
        ic: usize,
        icplus: usize,
        som: rustycad_tkmath::gp::Point3,
    ) -> Result<bool, Failure> {
        let angedg = angle_edge(&c.v1, &c.evive[ic], &c.evive[icplus])?.abs();
        if (angedg - PI).abs() <= 0.01 {
            return Ok(false);
        }
        let mut res = SearchFd::default();
        let (cd1, cd2) = (c.cd(ic)?.clone(), c.cd(icplus)?.clone());
        if !search_fd(
            &self.ds,
            (&cd1, &cd2),
            (c.sens[ic], c.sens[icplus]),
            (c.index[ic], c.index[icplus]),
            &mut res,
        )? {
            return Ok(false);
        }
        let (mut i1, mut i2) = res.i;
        let (pa1, pa2) = res.front.p;
        let (jf1, jfp) = res.front.jf;
        if i1 != c.index[ic] {
            i1 = self.cut_back(c, ic, i1, 3 - jf1, false)?;
        }
        if i2 != c.index[icplus] {
            i2 = self.cut_back(c, icplus, i2, 3 - jfp, true)?;
        }
        let p2 = calcul_p2d_on_surf(c.cd(ic)?, jf1, i1, pa1)?;
        let indice = surf_index(c, ic, i1, SurfType::Surface)?;
        let pic = self.surface_value(indice, p2.x(), p2.y())?;
        if pic.distance(&som) > c.distmin {
            c.distmini = true;
        }
        c.jf[ic] = jf1;
        (c.i[ic][icplus], c.i[icplus][ic]) = (i1, i2);
        (c.p[ic][icplus], c.p[icplus][ic]) = (pa1, pa2);
        Ok(true)
    }

    /// The point of the fillet surface `indice` of the structure at (u, v).
    pub(super) fn surface_value(
        &self,
        indice: i32,
        u: f64,
        v: f64,
    ) -> Result<rustycad_tkmath::gp::Point3, Failure> {
        let s = self.ds.surface(indice).surface().clone();
        s.get()
            .ok_or_else(|| Failure::with_message("Standard_NullObject", "PerformMoreThreeCorner"))?
            .value(u, v)
    }

    /// The surface data of the contour `ic` between its end and the meeting `found`
    /// removed, their faces of the side `side` appended to `Fproj` (from the end for the
    /// first contour, from the meeting for the second one); the new index of the end.
    fn cut_back(
        &mut self,
        c: &mut Corner,
        ic: usize,
        found: i32,
        side: i32,
        second: bool,
    ) -> Result<i32, Failure> {
        let index = c.index[ic];
        let (ideb, ifin) = if c.sens[ic] == 1 {
            (index, found - 1)
        } else {
            (found + 1, index)
        };
        let range: Vec<i32> = match (found < index, second) {
            (true, false) => (found..=index).rev().collect(),
            (false, false) => (index..=found).collect(),
            (true, true) => (found..=index).collect(),
            (false, true) => (index..=found).rev().collect(),
        };
        for nb in range {
            let kind = if side == 1 {
                SurfType::Face1
            } else {
                SurfType::Face2
            };
            let iface = surf_index(c, ic, nb, kind)?;
            c.fproj.push(topods::face(self.ds.shape(iface, true))?);
        }
        remove_sd(c.cd(ic)?, ideb, ifin)?;
        let (num, _) = index_of_surf_data(&c.v1, c.cd(ic)?)?;
        c.index[ic] = num;
        Ok(num)
    }
}
