// Rust translation of FillShapesSameDomain and UnfillShapesSameDomain of
// ModelingAlgorithms/TKBool/TopOpeBRepDS/TopOpeBRepDS_DataStructure.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The same domain relation of two shapes: each in the same domain list of the other,
//! with a common reference shape and their orientations relative to it.
use super::{TopOpeBRepDSDataStructure, program_error};
use crate::top_ope_brep_ds::TopOpeBRepDSConfig;
use crate::top_ope_brep_tool::TopOpeBRepToolShapeTool;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::Failure;

impl TopOpeBRepDSDataStructure {
    /// The shape added with its rank, its orientation recorded when not yet defined (or
    /// defined as UNSHGEOMETRY); its index and whether it was (re)defined.
    fn add_with_orientation(&mut self, s: &Shape, rank: i32) -> Result<(i32, bool), Failure> {
        let i_s = self.add_shape_with_rank(s, rank);
        let sd = self.my_shapes.change_from_index(i_s as usize)?;
        let todef =
            !sd.my_orientation_def || sd.my_same_domain_ori == TopOpeBRepDSConfig::UnshGeometry;
        if todef {
            (sd.my_orientation, sd.my_orientation_def) = (s.orientation(), true);
        }
        Ok((i_s, todef))
    }

    /// The common reference of two shapes from their references (`r1`, `r2`).
    fn common_reference(
        (i_s1, r1): (i32, i32),
        (i_s2, r2): (i32, i32),
        ref_first: bool,
    ) -> Result<i32, Failure> {
        let r = match (r1 == i_s1, r2 == i_s2) {
            (true, true) => {
                if ref_first {
                    i_s1
                } else {
                    i_s2
                }
            }
            (true, false) => r2,
            (false, true) => r1,
            (false, false) => {
                if ref_first {
                    r1
                } else {
                    r2
                }
            }
        };
        if r == 0 {
            return Err(program_error("FacesFiller::Insert SD 2"));
        }
        Ok(r)
    }

    /// The new reference `r` of a shape, with its orientation relative to `sr` (same
    /// oriented when it is the reference and was not redefined).
    fn set_reference(
        &mut self,
        (s, i_s): (&Shape, i32),
        (r, sr): (i32, &Shape),
        todef: bool,
    ) -> Result<(), Failure> {
        let mut o = TopOpeBRepDSConfig::SameOriented;
        if (r != i_s || todef) && !TopOpeBRepToolShapeTool::shapes_same_oriented(s, sr)? {
            o = TopOpeBRepDSConfig::DiffOriented;
        }
        self.set_same_domain_ref_index(i_s, r);
        self.set_same_domain_ori_index(i_s, o);
        Ok(())
    }

    /// `FillShapesSameDomain(S1, S2, refFirst)` (refFirst natively defaulted to true):
    /// the shapes (ranks 1 and 2) in each other's same domain list; when the lists grew
    /// or an orientation was (re)defined, the common reference (the first or second
    /// when both are their own) given to both with their orientations relative to it,
    /// and the same domain indices 1 and 2; `ds.fill_shapes_same_domain(&f1, &f2, true)?`.
    pub fn fill_shapes_same_domain(
        &mut self,
        s1: &Shape,
        s2: &Shape,
        ref_first: bool,
    ) -> Result<(), Failure> {
        let (i_s1, todef1) = self.add_with_orientation(s1, 1)?;
        let (i_s2, todef2) = self.add_with_orientation(s2, 2)?;
        let (n1, n2) = (
            self.shape_same_domain(s1).len(),
            self.shape_same_domain(s2).len(),
        );
        self.add_shape_same_domain(s1, s2)?;
        self.add_shape_same_domain(s2, s1)?;
        let unchanged =
            self.shape_same_domain(s1).len() == n1 && self.shape_same_domain(s2).len() == n2;
        if unchanged && !todef1 && !todef2 {
            return Ok(());
        }
        let (r1, r2) = (self.same_domain_ref(s1), self.same_domain_ref(s2));
        let r = Self::common_reference((i_s1, r1), (i_s2, r2), ref_first)?;
        let mut sr = self.shape(r, true).clone();
        let o_sr = self.my_shapes.find_from_key(&sr)?.my_orientation;
        sr.set_orientation(o_sr);
        if r != r1 || todef1 {
            self.set_reference((s1, i_s1), (r, &sr), todef1)?;
        }
        if r != r2 || todef2 {
            self.set_reference((s2, i_s2), (r, &sr), todef2)?;
        }
        self.set_same_domain_ind(s1, 1);
        self.set_same_domain_ind(s2, 2);
        Ok(())
    }

    /// `FillShapesSameDomain(S1, S2, c1, c2, refFirst)`: the relation filled unless both
    /// shapes have a defined orientation and an UNSHGEOMETRY configuration; the shapes
    /// of UNSHGEOMETRY configuration then marked so;
    /// `ds.fill_shapes_same_domain_with_configs(&f1, &f2, c1, c2, true)?`.
    pub fn fill_shapes_same_domain_with_configs(
        &mut self,
        s1: &Shape,
        s2: &Shape,
        c1: TopOpeBRepDSConfig,
        c2: TopOpeBRepDSConfig,
        ref_first: bool,
    ) -> Result<(), Failure> {
        let i_s1 = self.add_shape_with_rank(s1, 1);
        let isdef1 = self
            .my_shapes
            .find_from_index(i_s1 as usize)?
            .my_orientation_def;
        let todef1 = !(c1 == TopOpeBRepDSConfig::UnshGeometry && isdef1);
        let i_s2 = self.add_shape_with_rank(s2, 2);
        let isdef2 = self
            .my_shapes
            .find_from_index(i_s2 as usize)?
            .my_orientation_def;
        let todef2 = !(c2 == TopOpeBRepDSConfig::UnshGeometry && isdef2);
        if todef1 || todef2 {
            self.fill_shapes_same_domain(s1, s2, ref_first)?;
        }
        if todef1 && c1 == TopOpeBRepDSConfig::UnshGeometry {
            self.set_same_domain_ori(s1, TopOpeBRepDSConfig::UnshGeometry);
        }
        if todef2 && c2 == TopOpeBRepDSConfig::UnshGeometry {
            self.set_same_domain_ori(s2, TopOpeBRepDSConfig::UnshGeometry);
        }
        Ok(())
    }

    /// `UnfillShapesSameDomain(S1, S2)`: each removed from the same domain list of the
    /// other; `ds.unfill_shapes_same_domain(&f1, &f2)?`.
    pub fn unfill_shapes_same_domain(&mut self, s1: &Shape, s2: &Shape) -> Result<(), Failure> {
        self.remove_shape_same_domain(s1, s2)?;
        self.remove_shape_same_domain(s2, s1)
    }
}
