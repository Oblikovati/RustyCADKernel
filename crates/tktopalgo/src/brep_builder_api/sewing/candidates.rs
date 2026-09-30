// Rust translation of BRepBuilderAPI_Sewing::FindCandidates and AnalysisNearestEdges
// (BRepBuilderAPI_Sewing.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The choice of the sections to merge with a reference section: the sections within
//! the tolerance sorted by distance, then (manifold) the mutually best candidate, or
//! (non-manifold) the nearest candidates face by face.
use super::BRepBuilderAPISewing;
use super::failures::array1;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{self, Edge, Shape};
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::ncollection::{IndexedMap, Map, Sequence};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::real_small;

/// The distances of a sequence of sections to its first one (`EvaluateDistances` with
/// the reference 1): orientation, distance, length and smallest distance.
struct SectionDistances {
    forward: rustycad_tkernel::ncollection::Array1<'static, bool>,
    distance: rustycad_tkernel::ncollection::Array1<'static, f64>,
    length: rustycad_tkernel::ncollection::Array1<'static, f64>,
    min_dist: rustycad_tkernel::ncollection::Array1<'static, f64>,
}

/// The sections with the reference first (the others in order) and their indices in
/// the input sequence.
fn reference_first(
    seq_sections: &Sequence<Shape>,
    ind_reference: i32,
    edge1: &Edge,
) -> Result<(Sequence<Shape>, Sequence<i32>), Failure> {
    let (mut sections_new, mut candidates_new): (Sequence<Shape>, Sequence<i32>) =
        (Sequence::default(), Sequence::default());
    for i in 1..=seq_sections.length() {
        if i == ind_reference {
            sections_new.prepend(edge1.clone().into_shape());
            candidates_new.prepend(i);
        } else {
            sections_new.append(seq_sections.value(i as usize)?.clone());
            candidates_new.append(i);
        }
    }
    Ok((sections_new, candidates_new))
}

impl BRepBuilderAPISewing {
    /// The faces of the boundary of a section, as a set.
    fn bound_face_set(&self, section: &Shape) -> MapOfShape {
        let mut faces = MapOfShape::default();
        if let Some(list) = self.my_bound_faces.seek(&self.bound_of(section)) {
            for f in list.iter() {
                faces.add(f.clone());
            }
        }
        faces
    }

    /// `EvaluateDistances` of a sequence against its first section.
    fn distances_to_first(&self, sections: &Sequence<Shape>) -> Result<SectionDistances, Failure> {
        let n = sections.length();
        let mut d = SectionDistances {
            forward: array1(n, false)?,
            distance: array1(n, -1.0)?,
            length: array1(n, 0.0)?,
            min_dist: array1(n, 0.0)?,
        };
        self.evaluate_distances(
            sections,
            &mut d.forward,
            &mut d.distance,
            &mut d.length,
            &mut d.min_dist,
            1,
        )?;
        Ok(d)
    }

    /// The candidates of the reference (index 1 of the sequence) within the tolerance
    /// and longer than the minimal tolerance, sorted by distance (by smallest distance
    /// among equal ones), with their orientations.
    fn sorted_candidates(
        &self,
        d: &SectionDistances,
        n: i32,
        seq_candidates: &mut Sequence<i32>,
        seq_orientations: &mut Sequence<bool>,
    ) -> Result<(), Failure> {
        for i in 2..=n {
            let max_dist = d.distance[i];
            if !(max_dist >= 0.0
                && max_dist <= self.my_tolerance
                && d.length[i] > self.min_tolerance())
            {
                continue;
            }
            let ori = d.forward[i];
            let mut is_inserted = false;
            let mut j = 1;
            while j <= seq_candidates.len() && !is_inserted {
                let other = *seq_candidates.value(j)?;
                let delta = d.distance[i] - d.distance[other];
                if delta < precision::confusion()
                    && (delta.abs() > real_small() || d.min_dist[i] < d.min_dist[other])
                {
                    seq_candidates.insert_before(j, i)?;
                    seq_orientations.insert_before(j, ori)?;
                    is_inserted = true;
                }
                j += 1;
            }
            if !is_inserted {
                seq_candidates.append(i);
                seq_orientations.append(ori);
            }
        }
        Ok(())
    }

    /// `FindCandidates(seqSections, mapReference, seqCandidates, seqOrientations)`: the
    /// sections to merge with the last reference of `map_reference`; false when there
    /// is none, or when the best one shares a face with the reference that is not a
    /// closed surface across which they may merge.
    pub(crate) fn find_candidates(
        &self,
        seq_sections: &Sequence<Shape>,
        map_reference: &mut IndexedMap<i32>,
        seq_candidates: &mut Sequence<i32>,
        seq_orientations: &mut Sequence<bool>,
    ) -> Result<bool, Failure> {
        let nb_sections = seq_sections.length();
        if nb_sections <= 1 {
            return Ok(false);
        }
        let ind_reference = *map_reference.find_key(map_reference.extent())?;
        let edge1 = topods::edge(seq_sections.value(ind_reference as usize)?)?;
        let faces1 = self.bound_face_set(&edge1);
        let (sections_new, candidates_new) = reference_first(seq_sections, ind_reference, &edge1)?;
        let d = self.distances_to_first(&sections_new)?;
        self.sorted_candidates(&d, sections_new.length(), seq_candidates, seq_orientations)?;
        let mut nb_candidates = seq_candidates.length();
        if nb_candidates == 0 {
            // Section has no candidates to merge.
            return Ok(false);
        }
        // Replace candidate indices.
        for i in 1..=seq_candidates.len() {
            let replaced = *candidates_new.value(*seq_candidates.value(i)? as usize)?;
            seq_candidates.set_value(i, replaced)?;
        }
        if self.my_nonmanifold && nb_candidates > 1 {
            return self.nearest_candidates(seq_sections, seq_candidates, seq_orientations);
        }
        nb_candidates = self.best_candidate(
            seq_sections,
            map_reference,
            ind_reference,
            (seq_candidates, seq_orientations),
            nb_candidates,
        )?;
        if nb_candidates > 0
            && !self.shares_no_face(&edge1, &faces1, seq_sections, *seq_candidates.value(1)?)?
        {
            return Ok(false);
        }
        Ok(nb_candidates > 0)
    }

    /// The non-manifold choice: the first candidate kept by each analysis of the
    /// nearest edges (the first one with the distances), prepended to the candidates.
    fn nearest_candidates(
        &self,
        seq_sections: &Sequence<Shape>,
        seq_candidates: &mut Sequence<i32>,
        seq_orientations: &mut Sequence<bool>,
    ) -> Result<bool, Failure> {
        let (mut new_candidates, mut orientations_new): (Sequence<i32>, Sequence<bool>) =
            (Sequence::default(), Sequence::default());
        seq_candidates.prepend(1);
        seq_orientations.prepend(true);
        let mut k = 1;
        while k <= seq_sections.length() && seq_candidates.length() > 1 {
            self.analysis_nearest_edges(seq_sections, seq_candidates, seq_orientations, k == 1)?;
            if k == 1 && seq_candidates.is_empty() {
                return Ok(false);
            }
            if let (Some(&first), Some(&ori)) = (seq_candidates.first(), seq_orientations.first()) {
                new_candidates.append(first);
                orientations_new.append(ori);
            }
            k += 1;
        }
        seq_candidates.prepend_sequence(&mut new_candidates);
        seq_orientations.prepend_sequence(&mut orientations_new);
        Ok(true)
    }

    /// The manifold choice: the first candidate is kept when it is a reference already,
    /// or when its own best candidate is this reference (or another reference, which is
    /// then prepended, without an orientation, as natively); otherwise it is dropped.
    /// Returns the number of candidates left.
    fn best_candidate(
        &self,
        seq_sections: &Sequence<Shape>,
        map_reference: &mut IndexedMap<i32>,
        ind_reference: i32,
        (seq_candidates, seq_orientations): (&mut Sequence<i32>, &mut Sequence<bool>),
        mut nb_candidates: i32,
    ) -> Result<i32, Failure> {
        while nb_candidates != 0 {
            let ind_candidate = *seq_candidates.value(1)?;
            // Candidate is approved if it is in the map.
            if map_reference.contains(&ind_candidate) {
                break;
            }
            map_reference.add(ind_candidate);
            let (mut candidates1, mut orientations1) = (Sequence::default(), Sequence::default());
            let found = self.find_candidates(
                seq_sections,
                map_reference,
                &mut candidates1,
                &mut orientations1,
            );
            map_reference.remove_last()?;
            if found? && !candidates1.is_empty() {
                let ind_candidate1 = *candidates1.value(1)?;
                if ind_candidate1 == ind_reference {
                    break;
                }
                if map_reference.contains(&ind_candidate1) {
                    seq_candidates.prepend(ind_candidate1);
                    nb_candidates += 1;
                    break;
                }
            }
            seq_candidates.remove(1)?;
            seq_orientations.remove(1)?;
            nb_candidates -= 1;
        }
        Ok(nb_candidates)
    }

    /// The final check of `FindCandidates` (gka): the faces of the best candidate are
    /// not faces of the reference, unless the two may merge across a closed surface.
    fn shares_no_face(
        &self,
        edge1: &Edge,
        faces1: &MapOfShape,
        seq_sections: &Sequence<Shape>,
        index: i32,
    ) -> Result<bool, Failure> {
        let edge2 = topods::edge(seq_sections.value(index as usize)?)?;
        let Some(faces) = self.my_bound_faces.seek(&self.bound_of(&edge2)) else {
            return Ok(true);
        };
        let mut is_ok = true;
        for f in faces.iter() {
            if !is_ok {
                break;
            }
            let face2 = topods::face(f)?;
            is_ok = !faces1.contains(&face2);
            if !is_ok {
                is_ok = self.is_merged_closed(edge1, &edge2, &face2)?;
            }
        }
        Ok(is_ok)
    }

    /// Whether a candidate section may merge with the work edge on the faces they
    /// share: on each shared face, the surface is closed along the candidate and the
    /// two may merge across it.
    fn merges_on_shared_faces(
        &self,
        candidate: &Shape,
        workedge: &Edge,
        faces: &rustycad_tkernel::ncollection::List<Shape>,
        map_faces: &MapOfShape,
    ) -> Result<bool, Failure> {
        let mut is_merged = true;
        for f in faces.iter() {
            if !is_merged {
                break;
            }
            if !map_faces.contains(f) {
                continue;
            }
            let face = topods::face(f)?;
            let (surf, loc) = BRepTool::surface_with_location(&face)?;
            let bound = self.bound_of(candidate);
            is_merged = self.is_closed_surface(&surf, &bound, &loc)?
                && self.is_merged_closed(&topods::edge(candidate)?, workedge, &face)?;
        }
        Ok(is_merged)
    }

    /// Whether a candidate stays: not the work edge, bounding faces, and merging with the
    /// work edge on the faces they share (otherwise it joins the sections that are no
    /// candidates).
    fn keeps_candidate(
        &self,
        sequence_sec: &Sequence<Shape>,
        index: i32,
        (work_index, work, map_faces): (i32, &Edge, &MapOfShape),
        seq_not_candidate: &mut Sequence<i32>,
    ) -> Result<bool, Failure> {
        if index == work_index {
            return Ok(false);
        }
        let candidate = sequence_sec.value(index as usize)?;
        let Some(listfaces) = self.my_bound_faces.seek(&self.bound_of(candidate)) else {
            return Ok(false);
        };
        let merged = self.merges_on_shared_faces(candidate, work, listfaces, map_faces)?;
        if !merged {
            seq_not_candidate.append(index);
        }
        Ok(merged)
    }

    /// `AnalysisNearestEdges(sequenceSec, seqIndCandidate, seqOrientations,
    /// evalDist)`: the work edge (the first candidate) and the candidates that share
    /// one of its faces without merging across it are removed; with `eval_dist`, a
    /// candidate is removed too when it is out of the tolerance of the work edge or
    /// nearer to one of those removed sections.
    pub(crate) fn analysis_nearest_edges(
        &self,
        sequence_sec: &Sequence<Shape>,
        seq_ind_candidate: &mut Sequence<i32>,
        seq_orientations: &mut Sequence<bool>,
        eval_dist: bool,
    ) -> Result<(), Failure> {
        let work_index = *seq_ind_candidate.value(1)?;
        let workedge = sequence_sec.value(work_index as usize)?.clone();
        let Some(workfaces) = self
            .my_bound_faces
            .seek(&self.bound_of(&workedge))
            .filter(|l| !l.is_empty())
        else {
            return Ok(());
        };
        let mut map_faces = MapOfShape::default();
        for f in workfaces.iter() {
            map_faces.add(f.clone());
        }
        let work = topods::edge(&workedge)?;
        let mut seq_not_candidate: Sequence<i32> = Sequence::default();
        seq_not_candidate.append(work_index);
        let mut i = 1;
        while i <= seq_ind_candidate.len() {
            let index = *seq_ind_candidate.value(i)?;
            let work = (work_index, &work, &map_faces);
            if self.keeps_candidate(sequence_sec, index, work, &mut seq_not_candidate)? {
                i += 1;
                continue;
            }
            seq_ind_candidate.remove(i)?;
            seq_orientations.remove(i)?;
        }
        if seq_ind_candidate.is_empty() || seq_not_candidate.length() == 1 || !eval_dist {
            return Ok(());
        }
        let map_index =
            self.farther_candidates(sequence_sec, seq_ind_candidate, &seq_not_candidate)?;
        for i2 in (1..=seq_ind_candidate.len()).rev() {
            if map_index.contains(&(i2 as i32)) {
                seq_ind_candidate.remove(i2)?;
                seq_orientations.remove(i2)?;
            }
        }
        Ok(())
    }

    /// The positions of the candidates to drop: out of the tolerance of the work edge,
    /// or nearer to one of the other removed sections than to the work edge.
    fn farther_candidates(
        &self,
        sequence_sec: &Sequence<Shape>,
        seq_ind_candidate: &Sequence<i32>,
        seq_not_candidate: &Sequence<i32>,
    ) -> Result<Map<i32>, Failure> {
        let nb_candidates = seq_ind_candidate.length();
        // `TotTabDist(1, n)`: natively an uninitialized NCollection_Array2 row; the
        // entries never written belong to positions already in the map, so their value
        // does not change the result.
        let mut work_dist: Vec<f64> = vec![0.0; nb_candidates as usize];
        let mut map_index: Map<i32> = Map::default();
        for k in 1..=seq_not_candidate.len() {
            let index1 = *seq_not_candidate.value(k)?;
            let mut tmp_seq: Sequence<Shape> = Sequence::default();
            tmp_seq.append(sequence_sec.value(index1 as usize)?.clone());
            for kk in 1..=seq_ind_candidate.len() {
                tmp_seq.append(
                    sequence_sec
                        .value(*seq_ind_candidate.value(kk)? as usize)?
                        .clone(),
                );
            }
            let d = self.distances_to_first(&tmp_seq)?;
            for n in 1..tmp_seq.length() {
                if k == 1 {
                    let dist = d.distance[n + 1];
                    if dist == -1.0 || dist > self.my_tolerance {
                        map_index.add(n);
                        continue;
                    }
                    work_dist[(n - 1) as usize] = dist;
                    continue;
                }
                let dist = d.distance[n];
                if dist == -1.0 || dist > self.my_tolerance {
                    continue;
                }
                if d.distance[n + 1] < work_dist[(n - 1) as usize] {
                    map_index.add(n);
                }
            }
        }
        Ok(map_index)
    }
}
