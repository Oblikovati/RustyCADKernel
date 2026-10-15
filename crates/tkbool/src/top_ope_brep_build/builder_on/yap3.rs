//! The cases yap3, yap5 and yap7 of GFillONPartsWES2: the face is not of the same domain
//! as the connected face.
use super::context::OnContext;
use super::yapc::keep_by_sd_edge;
use super::{TopOpeBRepBuildBuilderON, fun_find_esd};
use crate::top_ope_brep_ds::TopOpeBRepDSConfig;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{self, ShapeEnum};

impl TopOpeBRepBuildBuilderON<'_> {
    /// The cases yap3 (D 3) and yap5 (D 2): the ON split oriented as the same domain
    /// edge of the face containing it.
    fn yap_sd(&mut self, c: &OnContext, d: i32) -> Result<(), Failure> {
        let pb = self.pb()?;
        let (ie, oe) = fun_find_esd(&pb.hds()?.ds(), &c.eon, &c.eg, &c.for_, d)?;
        if ie == 0 {
            return Ok(());
        }
        let esd = pb.hds()?.shape(ie, true);
        if let Some(o) = keep_by_sd_edge(c, &esd, oe)? {
            self.add_start(&c.esp_on.oriented(o))?;
        }
        Ok(())
    }

    /// The case yap3: a 3d same domain edge in the face.
    pub(super) fn yap3(&mut self, c: &OnContext) -> Result<(), Failure> {
        self.yap_sd(c, 3)
    }

    /// The case yap5: a 2d same domain edge in the face.
    pub(super) fn yap5(&mut self, c: &OnContext) -> Result<(), Failure> {
        self.yap_sd(c, 2)
    }

    /// The case yap7: a section edge on a face with same domain faces: the orientation
    /// of the transition in the geometry of the reference (dropped in the common of a
    /// cut when not an edge of the same domain faces).
    pub(super) fn yap7(&mut self, c: &OnContext) -> Result<(), Failure> {
        let pb = self.pb()?;
        let sd = pb.hds()?.ds().shape_same_domain_index(c.ifor).clone();
        let mut isbound = false;
        for s in sd.iter() {
            if Explorer::new(s, ShapeEnum::Edge, ShapeEnum::Shape)?
                .any(|e| e.is_same(c.eg.as_ref()))
            {
                isbound = true;
            }
        }
        let mut neworie = c.tfe.orientation(c.tb1, ShapeEnum::Face);
        if c.com_of_cut && !c.eg_bound_for && !isbound {
            return Ok(());
        }
        let samegeomref = if c.for_isref {
            true
        } else {
            let s = c.c_for == TopOpeBRepDSConfig::SameOriented;
            if c.ofor != c.oref_for { !s } else { s }
        };
        if !samegeomref {
            neworie = top_abs::complement(neworie);
        }
        self.add_start(&c.esp_on.oriented(neworie))
    }
}
