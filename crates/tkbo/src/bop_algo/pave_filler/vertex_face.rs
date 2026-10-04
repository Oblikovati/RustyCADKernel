//! Vertex/face solvers run before their results mutate source vertices.
use super::BOPAlgoPaveFiller;
use crate::bop_algo::{AlertIntersectionOfPairOfShapesFailed, Glue};
use crate::bop_ds::{DataStructure, InterfVF};
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{Compound, Face, Vertex},
};
use rustycad_tkernel::{message::AlertHandle, ncollection::Map, standard::Failure};
use rustycad_tkg3d::top_abs::ShapeEnum;
use std::sync::Arc;

#[path = "vertex_face/ee_vertices.rs"]
mod ee_vertices;

struct VertexFaceCandidate {
    representative: i32,
    face: i32,
    originals: Map<i32>,
}

#[derive(Clone, Copy)]
struct VertexFaceHit {
    u: f64,
    v: f64,
    tolerance: f64,
}

impl BOPAlgoPaveFiller {
    /// Intersects prepared vertex/face candidates and EE vertices; `filler.perform_vf()?`.
    pub fn perform_vf(&mut self) -> Result<(), Failure> {
        let pairs = self.selected_vertex_face_pairs()?;
        if self.glue == Glue::Full {
            return self.initialize_glued_faces(&pairs);
        }
        self.vf_structure()?
            .interf_vf()
            .set_increment(if pairs.is_empty() { 10 } else { pairs.len() });
        let candidates = self.vertex_face_candidates(&pairs)?;
        let hits = self.compute_vertex_faces(&candidates)?;
        for (candidate, hit) in candidates.iter().zip(hits) {
            if let Some(hit) = hit {
                self.accept_vertex_face(candidate, hit)?;
            }
        }
        self.treat_vertices_ee()
    }

    fn compute_vertex_faces(
        &mut self,
        candidates: &[VertexFaceCandidate],
    ) -> Result<Vec<Option<VertexFaceHit>>, Failure> {
        candidates
            .iter()
            .map(|candidate| {
                self.compute_vertex_face(candidate.representative, candidate.face, true)
            })
            .collect()
    }

    fn vf_structure(&mut self) -> Result<&mut DataStructure, Failure> {
        self.data_structure.as_mut().ok_or_else(|| Failure::new(
            "BOPAlgo_PaveFiller::PerformVF received absent DS; expected initialized data structure"))
    }

    fn selected_vertex_face_pairs(&mut self) -> Result<Vec<(i32, i32)>, Failure> {
        let iterator = self.iterator.as_mut().ok_or_else(|| Failure::new(
            "BOPAlgo_PaveFiller::PerformVF received absent iterator; expected initialized iterator"))?;
        iterator.initialize(ShapeEnum::Vertex, ShapeEnum::Face);
        let mut pairs = Vec::new();
        while iterator.more() {
            pairs.push(iterator.value()?);
            iterator.next();
        }
        Ok(pairs)
    }

    fn initialize_glued_faces(&mut self, pairs: &[(i32, i32)]) -> Result<(), Failure> {
        let ds = self.vf_structure()?;
        for &(vertex, face) in pairs {
            if !ds.shape_info(face)?.has_sub_shape(vertex) {
                ds.change_face_info(face)?;
            }
        }
        Ok(())
    }

    fn vertex_face_candidates(
        &mut self,
        pairs: &[(i32, i32)],
    ) -> Result<Vec<VertexFaceCandidate>, Failure> {
        let ds = self.vf_structure()?;
        let mut candidates = Vec::new();
        for &(vertex, face) in pairs {
            if ds.shape_info(face)?.has_sub_shape(vertex)
                || ds.has_interference_between(vertex, face)
            {
                continue;
            }
            ds.change_face_info(face)?;
            if !ds.has_interference_shape_subshapes(vertex, face)? {
                group_vertex_face(&mut candidates, ds.same_domain_index(vertex), face, vertex);
            }
        }
        Ok(candidates)
    }

    fn compute_vertex_face(
        &mut self,
        vertex: i32,
        face: i32,
        warn: bool,
    ) -> Result<Option<VertexFaceHit>, Failure> {
        let ds = self.vf_structure()?;
        let vertex = Vertex::unchecked(ds.shape(vertex)?.clone());
        let face = Face::unchecked(ds.shape(face)?.clone());
        let mut hit = VertexFaceHit {
            u: 0.,
            v: 0.,
            tolerance: 0.,
        };
        let status = self
            .context
            .get()
            .expect("initialized VF context")
            .compute_vf(
                &vertex,
                &face,
                &mut hit.u,
                &mut hit.v,
                &mut hit.tolerance,
                self.options.fuzzy_value(),
            );
        match status {
            Ok(0) => Ok(Some(hit)),
            Ok(_) => Ok(None),
            Err(_) if warn => {
                self.warn_vertex_face(&vertex, &face)?;
                Ok(None)
            }
            Err(failure) => Err(failure),
        }
    }

    fn warn_vertex_face(&self, vertex: &Vertex, face: &Face) -> Result<(), Failure> {
        let builder = BRepBuilder::default();
        let mut compound = Compound::default();
        builder.make_compound(&mut compound);
        let mut shape = compound.into_shape();
        builder.add(&mut shape, &vertex.clone().into_shape())?;
        builder.add(&mut shape, &face.clone().into_shape())?;
        let alert: AlertHandle = Arc::new(AlertIntersectionOfPairOfShapesFailed::new(&shape));
        self.options.add_warning(&alert);
        Ok(())
    }

    fn accept_vertex_face(
        &mut self,
        candidate: &VertexFaceCandidate,
        hit: VertexFaceHit,
    ) -> Result<(), Failure> {
        let mut representative = candidate.representative;
        for original in candidate.originals.iter() {
            let ds = self.vf_structure()?;
            append_vf_record(ds, *original, candidate.face, hit);
            representative = self.update_vertex(*original, hit.tolerance)?;
            let ds = self.vf_structure()?;
            if ds.is_new_shape(representative) {
                let last = ds.interf_vf().len() - 1;
                ds.interf_vf()
                    .change_value(last)
                    .expect("newly appended VF record")
                    .set_index_new(representative);
            }
        }
        self.vf_structure()?
            .change_face_info(candidate.face)?
            .change_vertices_in()
            .add(representative);
        Ok(())
    }
}

fn group_vertex_face(
    candidates: &mut Vec<VertexFaceCandidate>,
    representative: i32,
    face: i32,
    original: i32,
) {
    if let Some(candidate) = candidates
        .iter_mut()
        .find(|candidate| candidate.representative == representative && candidate.face == face)
    {
        candidate.originals.add(original);
        return;
    }
    let mut originals = Map::default();
    originals.add(original);
    candidates.push(VertexFaceCandidate {
        representative,
        face,
        originals,
    });
}

fn append_vf_record(ds: &mut DataStructure, vertex: i32, face: i32, hit: VertexFaceHit) {
    let mut record = InterfVF::default();
    record.set_indices(vertex, face);
    record.set_uv(hit.u, hit.v);
    ds.interf_vf().append(record);
    ds.add_interference(vertex, face);
}
