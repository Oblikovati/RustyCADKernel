// Rust translation of the geometry members of TopOpeBRepDS_DataStructure (surfaces, curves,
// points and the equivalences of curves and curve ends in
// ModelingAlgorithms/TKBool/TopOpeBRepDS/TopOpeBRepDS_DataStructure.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The geometries of the data structure: added with increasing indices, removed or
//! marked not kept, and merged into equivalence classes.
use super::{TopOpeBRepDSDataStructure, program_error, remove_first_same};
use crate::top_ope_brep_ds::{
    TopOpeBRepDSCurve, TopOpeBRepDSCurveData, TopOpeBRepDSPoint, TopOpeBRepDSPointData,
    TopOpeBRepDSSurface, TopOpeBRepDSSurfaceData,
};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::Failure;

impl TopOpeBRepDSDataStructure {
    /// `AddSurface(S)`: its index; `ds.add_surface(&s)`.
    pub fn add_surface(&mut self, s: &TopOpeBRepDSSurface) -> i32 {
        self.my_nb_surfaces += 1;
        self.my_surfaces
            .bind(self.my_nb_surfaces, TopOpeBRepDSSurfaceData::new(s));
        self.my_nb_surfaces
    }

    /// `RemoveSurface(I)`: unbound (the count is kept); `ds.remove_surface(1)`.
    pub fn remove_surface(&mut self, i: i32) {
        self.my_surfaces.unbind(&i);
    }

    /// `KeepSurface(I)`; `ds.keep_surface(1)`.
    pub fn keep_surface(&self, i: i32) -> bool {
        self.surface(i).keep()
    }

    /// `KeepSurface(S)`; `ds.keep_surface_of(&s)`.
    pub fn keep_surface_of(&self, s: &TopOpeBRepDSSurface) -> bool {
        s.keep()
    }

    /// `ChangeKeepSurface(I, FindKeep)` (the empty surface for an unknown index, as
    /// natively); `ds.change_keep_surface(1, true)`.
    pub fn change_keep_surface(&mut self, i: i32, find_keep: bool) {
        self.change_surface(i).change_keep(find_keep);
    }

    /// `ChangeKeepSurface(S, FindKeep)`; `ds.change_keep_surface_of(&mut s, true)`.
    pub fn change_keep_surface_of(&self, s: &mut TopOpeBRepDSSurface, find_keep: bool) {
        s.change_keep(find_keep);
    }

    /// `AddCurve(S)`: its index, also written as the DS index of `c` (natively through
    /// a const_cast of the argument); `ds.add_curve(&mut c)`.
    pub fn add_curve(&mut self, c: &mut TopOpeBRepDSCurve) -> i32 {
        self.my_nb_curves += 1;
        c.change_ds_index(self.my_nb_curves);
        self.my_curves
            .bind(self.my_nb_curves, TopOpeBRepDSCurveData::new(c));
        self.my_nb_curves
    }

    /// `RemoveCurve(I)`: its surface/curve interferences removed from its shapes, and
    /// the curve not kept; `ds.remove_curve(1)`.
    pub fn remove_curve(&mut self, i: i32) {
        let (s1, s2) = self.curve(i).get_shapes();
        let (i1, i2) = self.curve(i).get_sci();
        if !i1.is_null() {
            self.remove_shape_interference(&s1, &i1);
        }
        if !i2.is_null() {
            self.remove_shape_interference(&s2, &i2);
        }
        self.change_curve(i).change_keep(false);
    }

    /// `KeepCurve(I)`; `ds.keep_curve(1)`.
    pub fn keep_curve(&self, i: i32) -> bool {
        self.curve(i).keep()
    }

    /// `KeepCurve(C)`; `ds.keep_curve_of(&c)`.
    pub fn keep_curve_of(&self, c: &TopOpeBRepDSCurve) -> bool {
        c.keep()
    }

    /// `ChangeKeepCurve(I, FindKeep)`; `ds.change_keep_curve(1, false)`.
    pub fn change_keep_curve(&mut self, i: i32, find_keep: bool) {
        self.change_curve(i).change_keep(find_keep);
    }

    /// `ChangeKeepCurve(C, FindKeep)`; `ds.change_keep_curve_of(&mut c, false)`.
    pub fn change_keep_curve_of(&self, c: &mut TopOpeBRepDSCurve, find_keep: bool) {
        c.change_keep(find_keep);
    }

    /// `AddPoint(PDS)`: its index; `ds.add_point(&p)`.
    pub fn add_point(&mut self, pds: &TopOpeBRepDSPoint) -> i32 {
        self.my_nb_points += 1;
        self.my_points
            .bind(self.my_nb_points, TopOpeBRepDSPointData::new(pds));
        self.my_nb_points
    }

    /// `AddPointSS(PDS, S1, S2)`: `AddPoint` (the shapes are unused); `ds.add_point_ss(&p, &s1, &s2)`.
    pub fn add_point_ss(&mut self, pds: &TopOpeBRepDSPoint, _s1: &Shape, _s2: &Shape) -> i32 {
        self.add_point(pds)
    }

    /// `RemovePoint(I)`: not kept; Standard_ProgramError out of 1..=NbPoints;
    /// `ds.remove_point(1)?`.
    pub fn remove_point(&mut self, i: i32) -> Result<(), Failure> {
        self.change_point(i)?.change_keep(false);
        Ok(())
    }

    /// `KeepPoint(I)`; `ds.keep_point(1)?`.
    pub fn keep_point(&self, i: i32) -> Result<bool, Failure> {
        Ok(self.point(i)?.keep())
    }

    /// `KeepPoint(P)`; `ds.keep_point_of(&p)`.
    pub fn keep_point_of(&self, p: &TopOpeBRepDSPoint) -> bool {
        p.keep()
    }

    /// `ChangeKeepPoint(I, FindKeep)`; `ds.change_keep_point(1, false)?`.
    pub fn change_keep_point(&mut self, i: i32, find_keep: bool) -> Result<(), Failure> {
        self.change_point(i)?.change_keep(find_keep);
        Ok(())
    }

    /// `ChangeKeepPoint(P, FindKeep)`; `ds.change_keep_point_of(&mut p, false)`.
    pub fn change_keep_point_of(&self, p: &mut TopOpeBRepDSPoint, find_keep: bool) {
        p.change_keep(find_keep);
    }

    /// `FindEquivalentCurvePoint(theCurve, theCurveStart, theRepresentativeStart)`: the
    /// representative curve end of a curve end (2 * curve + 0 for the start, + 1 for
    /// the end); `let (curve, start) = ds.find_equivalent_curve_point(2, true);`.
    pub fn find_equivalent_curve_point(
        &self,
        the_curve: i32,
        the_curve_start: bool,
    ) -> (i32, bool) {
        let mut a_point: i32 = 2 * the_curve + if the_curve_start { 0 } else { 1 };
        while let Some(&next) = self.my_equivalent_curve_points.seek(&a_point) {
            if next == a_point {
                break;
            }
            a_point = next;
        }
        (a_point / 2, a_point % 2 == 0)
    }

    /// `MergeEquivalentCurvePoints(theFirstCurve, theFirstCurveStart, theSecondCurve,
    /// theSecondCurveStart)`: both ends in one class, the smaller root representing it;
    /// `ds.merge_equivalent_curve_points(1, true, 2, false)`.
    pub fn merge_equivalent_curve_points(
        &mut self,
        the_first_curve: i32,
        the_first_curve_start: bool,
        the_second_curve: i32,
        the_second_curve_start: bool,
    ) {
        let (first_curve, first_start) =
            self.find_equivalent_curve_point(the_first_curve, the_first_curve_start);
        let (second_curve, second_start) =
            self.find_equivalent_curve_point(the_second_curve, the_second_curve_start);
        let first_root = 2 * first_curve + if first_start { 0 } else { 1 };
        let second_root = 2 * second_curve + if second_start { 0 } else { 1 };
        if first_root == second_root {
            return;
        }
        let (root, child) = (first_root.min(second_root), first_root.max(second_root));
        self.my_equivalent_curve_points.bind(root, root);
        self.my_equivalent_curve_points.bind(child, root);
    }

    /// `FindEquivalentCurve(I, theIsReversed)`: the representative of the curve and
    /// whether it is reversed relative to it; `let (root, rev) = ds.find_equivalent_curve(3);`.
    pub fn find_equivalent_curve(&self, i: i32) -> (i32, bool) {
        let (mut a_root, mut the_is_reversed) = (i, false);
        while self.curve(a_root).equivalent_curve() > 0
            && self.curve(a_root).equivalent_curve() != a_root
        {
            the_is_reversed ^= self.curve(a_root).is_equivalent_curve_reversed();
            a_root = self.curve(a_root).equivalent_curve();
        }
        (a_root, the_is_reversed)
    }

    /// `MergeEquivalentCurves(theFirst, theSecond, theIsReversed)`: both curves in one
    /// class, the smaller root representing it; `ds.merge_equivalent_curves(1, 2, false)`.
    pub fn merge_equivalent_curves(
        &mut self,
        the_first: i32,
        the_second: i32,
        the_is_reversed: bool,
    ) {
        let (first_root, first_reversed) = self.find_equivalent_curve(the_first);
        let (second_root, second_reversed) = self.find_equivalent_curve(the_second);
        if first_root == second_root {
            return;
        }
        let (reference, equivalent) = if first_root < second_root {
            (first_root, second_root)
        } else {
            (second_root, first_root)
        };
        let root_reversed = first_reversed ^ second_reversed ^ the_is_reversed;
        self.change_curve(reference)
            .set_equivalent_curve(reference, false);
        self.change_curve(equivalent)
            .set_equivalent_curve(reference, root_reversed);
    }

    /// `RemoveShapeInterference(S, I)`: the first occurrence of the interference
    /// object removed from the interferences of the shape (or from the shared empty list
    /// of an unknown shape); `ds.remove_shape_interference(&face, &i)`.
    pub fn remove_shape_interference(
        &mut self,
        s: &Shape,
        i: &rustycad_tkernel::standard::Handle<crate::top_ope_brep_ds::TopOpeBRepDSInterference>,
    ) {
        let l = self.change_shape_interferences(s);
        if Self::find_interference(l, i).is_some() {
            remove_first_same(l, i);
        }
    }

    /// `Point(I)` checked against the count (Standard_ProgramError natively).
    pub(super) fn check_point_index(&self, i: i32) -> Result<(), Failure> {
        if i < 1 || i > self.my_nb_points {
            return Err(program_error(&format!(
                "TopOpeBRepDS_DataStructure::Point: index {i}; expected 1..={}",
                self.my_nb_points
            )));
        }
        Ok(())
    }
}
