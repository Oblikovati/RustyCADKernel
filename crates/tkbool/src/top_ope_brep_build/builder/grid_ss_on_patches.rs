//! The ON patches of a base face and the faces touching an edge (TopOpeBRepBuild_GridSS.cxx).
use super::TopOpeBRepBuildBuilder;
use crate::top_ope_brep_build::{TopOpeBRepBuildShapeSet, TopOpeBRepBuildWireEdgeSet};
use crate::top_ope_brep_ds::export::fds_data;
use crate::top_ope_brep_ds::{TopOpeBRepDSKind, TopOpeBRepDSShapeShapeInterference};
use rustycad_tkbrep::topods::{Shape, ShapeIterator};
use rustycad_tkernel::ncollection::{DataMap, IndexedMap, List};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

impl TopOpeBRepBuildBuilder {
    /// The edges of the ON parts and ON elements not avoided, and whether a wire is one.
    fn on_all_edges(
        &self,
        an_edges_on: &List<Shape>,
        avoid: &IndexedMap<Shape>,
    ) -> Result<(List<Shape>, bool), Failure> {
        let mut all = List::default();
        for e in an_edges_on.iter() {
            if !self.my_on_elem_map.contains(e)
                && !self.my_on_elem_map.contains(&e.reversed())
                && !avoid.contains(e)
            {
                all.append(e.clone());
            }
        }
        let mut has_wires = false;
        for i in 1..=self.my_on_elem_map.extent() {
            let e = self.my_on_elem_map.find_key(i)?;
            if e.shape_type()? == ShapeEnum::Wire {
                for w in ShapeIterator::new(e, true, true) {
                    if !avoid.contains(&w) {
                        all.append(w);
                        has_wires = true;
                    }
                }
            } else if !avoid.contains(e) {
                all.append(e.clone());
            }
        }
        Ok((all, has_wires))
    }

    /// `FillOnPatches(anEdgesON, aBaseFace, avoidMap)`: the faces of the base face built
    /// from its ON parts and ON elements lying on coincident faces of the other shape,
    /// recorded in the ON faces map; `b.fill_on_patches(&lon, &f, &avoid)?`.
    pub fn fill_on_patches(
        &mut self,
        an_edges_on: &List<Shape>,
        a_base_face: &Shape,
        avoid_map: &IndexedMap<Shape>,
    ) -> Result<(), Failure> {
        let ff = a_base_face.oriented(Orientation::Forward);
        let rank_bf = self.shape_rank(a_base_face);
        let rank_opp = match rank_bf {
            1 => 2,
            2 => 1,
            _ => return Ok(()),
        };
        let mut wes = TopOpeBRepBuildWireEdgeSet::new(&ff)?;
        let mut states = DataMap::default();
        let (all_edges, has_wires) = self.on_all_edges(an_edges_on, avoid_map)?;
        self.on_parts_on_faces(
            an_edges_on,
            a_base_face,
            avoid_map,
            rank_opp,
            &all_edges,
            &mut states,
            &mut wes,
        )?;
        self.on_elems_on_faces(
            a_base_face,
            avoid_map,
            rank_bf,
            rank_opp,
            has_wires,
            &all_edges,
            &mut states,
            &mut wes,
        )?;
        wes.init_shapes();
        wes.init_start_elements();
        if wes.more_shapes() || wes.more_start_elements() {
            let mut lof = List::default();
            self.gwes_make_faces(&ff, &mut wes, &mut lof)?;
            for f in lof.iter() {
                self.my_on_faces_map.add(f.clone(), a_base_face.clone());
            }
        }
        Ok(())
    }

    /// `FindFacesTouchingEdge(aFace, anEdge, aShRank, aFaces)`: the faces of the rank on
    /// the FACE/FACE transitions of the edge interferences of the face on the edge;
    /// `b.find_faces_touching_edge(&f, &e, 2, &mut l)?`.
    pub fn find_faces_touching_edge(
        &self,
        a_face: &Shape,
        an_edge: &Shape,
        a_sh_rank: i32,
        a_faces: &mut List<Shape>,
    ) -> Result<(), Failure> {
        let bds = self.hds()?.ds();
        let an_edge_ind = bds.shape_index(an_edge, true);
        if an_edge_ind == 0 {
            return Ok(());
        }
        for i in bds.shape_interferences(a_face, true).iter() {
            let ssi: Handle<TopOpeBRepDSShapeShapeInterference> = i.clone().cast();
            let Some(s) = ssi.get() else {
                continue;
            };
            let (gt, gi, st, si) = fds_data(i);
            if gt != TopOpeBRepDSKind::Edge || st != TopOpeBRepDSKind::Face || gi != an_edge_ind {
                continue;
            }
            let tfe = s.transition();
            if tfe.shape_before() != ShapeEnum::Face || tfe.shape_after() != ShapeEnum::Face {
                continue;
            }
            let fs = bds.shape(si, true).clone();
            if self.shape_rank(&fs) == a_sh_rank {
                a_faces.append(fs);
            }
        }
        Ok(())
    }
}
