// Rust translation of ModelingAlgorithms/TKTopAlgo/MAT2d/MAT2d_Circuit.cxx (ConstructCircuit, UpDateLink, SortRefToEqui, PassByLast and the static SubSequence) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The construction of the circuit along the path of connexions: the portions of
//! the lines between successive connexions, and the links from the items of the
//! figure to their places in the circuit.
use super::CircuitState;
use crate::mat2d::connexion::Mat2dConnexion;
use crate::mat2d::{Mat2dBiInt, Mat2dFigure, Mat2dLine, Mat2dMiniPath, seq};
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::Vector2;

// `SubSequence(S1, IF, IL, S2)`: the items `if_..=il` of `s1`.
fn sub_sequence(s1: &Mat2dLine, if_: i32, il: i32) -> Result<Mat2dLine, Failure> {
    let mut s2: Mat2dLine = Sequence::default();
    for i in if_..=il {
        s2.append(seq::at(s1, i)?);
    }
    Ok(s2)
}

impl CircuitState {
    /// `ConstructCircuit(aFigure, IndRefLine, aPath)`: from the first item of the
    /// reference line, the portions of the lines between successive connexions of the
    /// path; each connexion is bound to the item after it.
    pub(super) fn construct_circuit(
        &mut self,
        fig_item: &Mat2dFigure,
        ind_ref_line: i32,
        road: &Mat2dMiniPath,
    ) -> Result<(), Failure> {
        let path = road.path();
        let mut prev_c: Handle<Mat2dConnexion> = seq::at(path, 1)?;
        let ref_line: Mat2dLine = seq::at(fig_item, ind_ref_line)?;
        self.geom_elements = sub_sequence(&ref_line, 1, prev_c.index_item_on_first())?;
        self.up_date_link(1, ind_ref_line, 1, prev_c.index_item_on_first());
        self.connexion_map
            .insert(self.geom_elements.length() + 1, prev_c.clone());
        let mut i_last_item: i32 = self.geom_elements.length();
        let mut cur_c: Handle<Mat2dConnexion> = Handle::default();
        for i in 2..=path.length() {
            cur_c = seq::at(path, i)?;
            i_last_item = self.add_portion(fig_item, &prev_c, &cur_c)?;
            prev_c = cur_c.clone();
        }
        let ind_last: i32 = ref_line.length();
        if ind_last == 1 {
            self.connexion_map.insert(1, cur_c);
            self.connexion_map.remove(&(i_last_item + 1));
        } else {
            self.append_portion(
                &ref_line,
                ind_ref_line,
                (prev_c.index_item_on_second(), ind_last),
            )?;
        }
        let keys: Vec<Mat2dBiInt> = self
            .link_ref_equi
            .iter()
            .filter(|(_, s)| s.len() > 1)
            .map(|(k, _)| *k)
            .collect();
        for key in keys {
            self.sort_ref_to_equi(key);
        }
        Ok(())
    }

    // The portion from the connexion `prev_c` to `cur_c` on their common line (through
    // its last item when the path passes by it); `cur_c` is bound after it. It
    // returns the new last item index.
    fn add_portion(
        &mut self,
        fig_item: &Mat2dFigure,
        prev_c: &Handle<Mat2dConnexion>,
        cur_c: &Handle<Mat2dConnexion>,
    ) -> Result<i32, Failure> {
        let mut i_last_item: i32;
        let line_index: i32 = cur_c.index_first_line();
        let line: Mat2dLine = seq::at(fig_item, line_index)?;
        if self.pass_by_last(prev_c, cur_c) {
            let ind_last: i32 = line.length();
            i_last_item =
                self.append_portion(&line, line_index, (prev_c.index_item_on_second(), ind_last))?;
            if line.length() > 1 {
                i_last_item =
                    self.append_portion(&line, line_index, (1, cur_c.index_item_on_first()))?;
            }
        } else {
            let range = (prev_c.index_item_on_second(), cur_c.index_item_on_first());
            i_last_item = self.append_portion(&line, line_index, range)?;
        }
        self.connexion_map.insert(i_last_item + 1, cur_c.clone());
        Ok(i_last_item)
    }

    // The items `range` of the line `line_index` are appended to the circuit and
    // linked; it returns the new number of items.
    fn append_portion(
        &mut self,
        line: &Mat2dLine,
        line_index: i32,
        (first, last): (i32, i32),
    ) -> Result<i32, Failure> {
        let portion: Mat2dLine = sub_sequence(line, first, last)?;
        let i_first: i32 = self.geom_elements.length() + 1;
        self.up_date_link(i_first, line_index, first, last);
        for item in portion.iter() {
            self.geom_elements.append(item.clone());
        }
        Ok(self.geom_elements.length())
    }

    /// `UpDateLink(IFirst, ILine, ICurveFirst, ICurveLast)`: the items
    /// `i_curve_first..=i_curve_last` of the line `i_line` are at the places
    /// `i_first..` of the circuit.
    pub(super) fn up_date_link(
        &mut self,
        i_first: i32,
        i_line: i32,
        i_curve_first: i32,
        i_curve_last: i32,
    ) {
        for (i, i_equi) in (i_curve_first..=i_curve_last).zip(i_first..) {
            self.link_ref_equi
                .entry(Mat2dBiInt::new(i_line, i))
                .or_default()
                .push(i_equi);
        }
    }

    // `SortRefToEqui(aBiInt)`: the places of an item start with the first one where
    // no connexion arrives (a rotation of the sequence).
    fn sort_ref_to_equi(&mut self, key: Mat2dBiInt) {
        let Some(places) = self.link_ref_equi.get(&key) else {
            return;
        };
        let split: Option<usize> = places
            .iter()
            .position(|place| !self.connexion_map.contains_key(place));
        let Some(offset) = split.filter(|&offset| offset > 0) else {
            return;
        };
        let mut rotated: Vec<i32> = places[offset..].to_vec();
        rotated.extend_from_slice(&places[..offset]);
        self.link_ref_equi.insert(key, rotated);
    }

    // `PassByLast(C1, C2)`: true when the portion of the line from the end of `c1` to
    // the start of `c2` passes by the last item of the line.
    fn pass_by_last(&self, c1: &Mat2dConnexion, c2: &Mat2dConnexion) -> bool {
        if c2.index_first_line() != c1.index_second_line() {
            return false;
        }
        if c2.index_item_on_first() < c1.index_item_on_second() {
            return true;
        }
        if c2.index_item_on_first() != c1.index_item_on_second() {
            return false;
        }
        if c1.index_first_line() == c2.index_second_line() {
            return true;
        }
        if c2.parameter_on_first() == c1.parameter_on_second() {
            let vect1 = Vector2::between(c1.point_on_second(), c1.point_on_first());
            let vect2 = Vector2::between(c2.point_on_first(), c2.point_on_second());
            return vect1.crossed(vect2) * self.direction > 0.;
        }
        c2.parameter_on_first() < c1.parameter_on_second()
    }
}
