// Rust translation of ChFi3d_Builder::Compute, ExtentAnalyse, PerformFilletOnVertex,
// PerformSingularCorner and the file static CompleteDS (ChFi3d_Builder.cxx) and of
// ChFi3d_Builder::UpdateTolesp (ChFi3d_Builder_C2.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The corners of the contours at the vertices: singular corners and the dispatch by the number of contours.
use super::ChFi3dBuilderOps;
use super::compute::spine_of;
use crate::chfi_ds::{State, lock};
use crate::chfi3d::{
    Arete, compute_arete, index_of_surf_data, index_point_in_ds, number_of_sharp_edges,
};
use rustycad_tkbool::top_ope_brep_ds::TopOpeBRepDSCurve;
use rustycad_tkernel::standard::Failure;

impl dyn ChFi3dBuilderOps + '_ {
    /// `PerformSingularCorner(Index)`: the ends of the contours at a vertex where their
    /// fillets degenerate to a point, closed by an edge on each fillet.
    pub(crate) fn perform_singular_corner(&mut self, index: i32) -> Result<(), Failure> {
        let (vtx, list) = {
            let b = self.base();
            (
                b.my_vdata_map.find_key(index)?,
                b.my_vdata_map.find_from_index(index)?.to_vec(),
            )
        };
        let mut ivtx = 0;
        for (i, stripe) in list.iter().enumerate() {
            let (num, sens) = index_of_surf_data(&vtx, stripe)?;
            let isfirst = sens == 1;
            let fd = crate::chfi3d::surf_data_at(stripe, num)?;
            let (cv1, cv2) = {
                let g = lock(&fd);
                (g.vertex(isfirst, 1).clone(), g.vertex(isfirst, 2).clone())
            };
            if !cv1.point().is_equal(&cv2.point(), 0.) {
                continue;
            }
            let b = self.base_mut();
            if i == 0 {
                ivtx = index_point_in_ds(&cv1, &mut b.ds)?;
            }
            let (von_s1, von_s2, surf) = {
                let g = lock(&fd);
                let at = |k: i32| -> Result<_, Failure> {
                    let fi = g.interference(k);
                    let pc = fi.pcurve_on_surf().get().ok_or_else(|| {
                        Failure::with_message("Standard_NullObject", "PerformSingularCorner")
                    })?;
                    pc.value(fi.parameter(isfirst))
                };
                (at(1)?, at(2)?, b.ds.surface(g.surf()).surface().clone())
            };
            let mut arete = Arete::default();
            compute_arete(
                (&cv1, von_s1),
                (&cv2, von_s2),
                &surf,
                &mut arete,
                (b.tolapp3d, b.tolapp2d),
                0,
            )?;
            let icurv = b.ds.add_curve(&mut TopOpeBRepDSCurve::new(
                &arete.c3d,
                arete.tolreached,
                false,
            ));
            let mut s = lock(stripe);
            s.set_curve(icurv, isfirst);
            s.set_parameters(isfirst, arete.pardeb, arete.parfin);
            *s.change_pcurve(isfirst) = arete.pcurv;
            s.set_index_point(ivtx, isfirst, 1);
            s.set_index_point(ivtx, isfirst, 2);
        }
        Ok(())
    }

    /// `PerformFilletOnVertex(Index)`: the corner at the `index`-th vertex, by the number
    /// of contours there and of sharp edges.
    pub(crate) fn perform_fillet_on_vertex(&mut self, index: i32) -> Result<(), Failure> {
        let (vtx, list) = {
            let b = self.base();
            (
                b.my_vdata_map.find_key(index)?,
                b.my_vdata_map.find_from_index(index)?.to_vec(),
            )
        };
        let (mut nondegenere, mut toujoursdegenere, mut isfirst) = (true, true, false);
        let mut sp = None;
        for stripe in &list {
            sp = Some(spine_of(stripe)?);
            let (num, sens) = index_of_surf_data(&vtx, stripe)?;
            isfirst = sens == 1;
            let fd = crate::chfi3d::surf_data_at(stripe, num)?;
            let g = lock(&fd);
            if g.vertex(isfirst, 1)
                .point()
                .is_equal(&g.vertex(isfirst, 2).point(), 0.)
            {
                nondegenere = false;
            } else {
                toujoursdegenere = false;
            }
        }
        let i = list.len() as i32;
        let nba = {
            let b = self.base();
            number_of_sharp_edges(&vtx, &b.my_ve_map, &b.my_ef_map)?
        };
        if !nondegenere {
            if toujoursdegenere {
                return self.perform_singular_corner(index);
            }
            return self.base_mut().perform_more_three_corner(index, i);
        }
        match i {
            1 => {
                let sp = sp.ok_or_else(|| Failure::new("PerformFilletOnVertex"))?;
                if lock(&sp).status(isfirst) == State::FreeBoundary {
                    return Ok(());
                }
                if nba > 3 {
                    self.base_mut().perform_intersection_at_end(index)
                } else if self.base().more_surfdata(index)? {
                    self.base_mut().perform_more_surfdata(index)
                } else {
                    self.base_mut().perform_one_corner(index, false)
                }
            }
            2 if nba <= 3 => self.perform_two_corner(index),
            3 if nba <= 3 => self.perform_three_corner(index),
            _ => self.base_mut().perform_more_three_corner(index, i),
        }
    }
}
