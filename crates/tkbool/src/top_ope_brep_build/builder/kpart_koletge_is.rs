//! The detection of the koletge particular case (TopOpeBRepBuild_KPart.cxx).
use super::TopOpeBRepBuildBuilder;
use super::kpart::kpresu::*;
use super::kpart_koletge::fun_sortplcy;
use crate::top_ope_brep_ds::{TopOpeBRepDSConfig, TopOpeBRepDSShapeShapeInterference};
use crate::top_ope_brep_tool::{fun_tool_cylinder_face, fun_tool_in_s, fun_tool_plane_face};
use rustycad_tkbrep::brep_tools::outer_wire;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{IndexedMap, List};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{ShapeEnum, State};

impl TopOpeBRepBuildBuilder {
    /// `KPiskoletge()`: the shapes are single solids touching on one planar face (and at
    /// most one cylindrical face each) whose outer wire edges are section edges of
    /// each other and carry all its interferences; `b.kp_iskoletge()?`.
    pub fn kp_iskoletge(&self) -> Result<i32, Failure> {
        let (mut lshsd1, mut lshsd2, mut lfhsd1, mut lfhsd2) = Default::default();
        if !self.kp_iskoletgesh(&self.my_shape1, &mut lshsd1, &mut lfhsd1)? {
            return Ok(0);
        }
        let (lpl1, lcy1) = fun_sortplcy(&lfhsd1)?;
        if lpl1.len() != 1 || lcy1.len() > 1 {
            return Ok(0);
        }
        if !self.kp_iskoletgesh(&self.my_shape2, &mut lshsd2, &mut lfhsd2)? {
            return Ok(0);
        }
        let (lpl2, lcy2) = fun_sortplcy(&lfhsd2)?;
        if lpl2.len() != 1 || lshsd1.len() > 1 || lshsd2.len() > 1 {
            return Ok(0);
        }
        let (f1, f2) = (
            topods::face(lpl1.first().unwrap_or(&Shape::default()))?,
            topods::face(lpl2.first().unwrap_or(&Shape::default()))?,
        );
        let (outerw1, outerw2) = (outer_wire(&f1)?, outer_wire(&f2)?);
        let (mut mape1, mut mape2): (
            IndexedMap<Shape, ShapeMapHasher>,
            IndexedMap<Shape, ShapeMapHasher>,
        ) = Default::default();
        topexp::map_shapes_of_type(outerw1.as_ref(), ShapeEnum::Edge, &mut mape1)?;
        topexp::map_shapes_of_type(outerw2.as_ref(), ShapeEnum::Edge, &mut mape2)?;
        if !self.section_edges_on(outerw1.as_ref(), &mape2)?
            || !self.section_edges_on(outerw2.as_ref(), &mape1)?
        {
            return Ok(0);
        }
        if !self.all_i_on_section_edges(f1.as_ref(), &mape1, &mape2)?
            || !self.all_i_on_section_edges(f2.as_ref(), &mape2, &mape1)?
        {
            return Ok(0);
        }
        if !lcy1.is_empty() {
            if !(lcy1.len() == 1 && lcy2.len() == 1) {
                return Ok(0);
            }
            if !fun_tool_in_s(outerw1.as_ref(), f1.as_ref())?
                || !fun_tool_in_s(outerw2.as_ref(), f2.as_ref())?
            {
                return Ok(0);
            }
        }
        Ok(1)
    }

    /// `sectionedgesON(HDS, outerw1, mape2)` (static): each edge of the wire has same
    /// domain edges, all in the map.
    fn section_edges_on(
        &self,
        outerw1: &Shape,
        mape2: &IndexedMap<Shape, ShapeMapHasher>,
    ) -> Result<bool, Failure> {
        let hds = self.hds()?;
        for e1 in Explorer::new(outerw1, ShapeEnum::Edge, ShapeEnum::Shape)? {
            let sd = hds.same_domain(&e1);
            if sd.is_empty() || !sd.iter().all(|e2| mape2.contains(e2)) {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// `allIonsectionedges(HDS, f1, mape1, mape2)` (static): the geometry of each
    /// interference of the face is in its map (bound) or the other one.
    fn all_i_on_section_edges(
        &self,
        f1: &Shape,
        mape1: &IndexedMap<Shape, ShapeMapHasher>,
        mape2: &IndexedMap<Shape, ShapeMapHasher>,
    ) -> Result<bool, Failure> {
        let hds = self.hds()?;
        let li = hds.ds().shape_interferences(f1, true).clone();
        for i in li.iter() {
            let ssi: Handle<TopOpeBRepDSShapeShapeInterference> = i.clone().cast();
            let s = ssi.get().ok_or_else(|| {
                Failure::with_type(
                    "Standard_NullObject",
                    "allIonsectionedges: not a shape/shape interference",
                    "TopOpeBRepBuild_Builder",
                )
            })?;
            let e1 = hds.shape(s.geometry(), true);
            let isbound = if s.gbound() {
                mape1.contains(&e1)
            } else {
                mape2.contains(&e1)
            };
            if !isbound {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// `KPiskoletgesh(Sarg, lShsd, lfhsd)`: a tangent shape (iskolesh) whose same domain
    /// faces are cylindrical, or planar with an outer wire of split section edges;
    /// `b.kp_iskoletgesh(&s, &mut lsd, &mut lfsd)?`.
    pub fn kp_iskoletgesh(
        &self,
        sarg: &Shape,
        lshsd: &mut List<Shape>,
        lfhsd: &mut List<Shape>,
    ) -> Result<bool, Failure> {
        if !self.funkp_kpiskolesh(sarg, lshsd)? {
            return Ok(false);
        }
        self.kp_lhsd_list(sarg, ShapeEnum::Face, lfhsd)?;
        let hds = self.hds()?;
        for fac in lfhsd.iter() {
            if fun_tool_cylinder_face(fac)? {
                continue;
            }
            if !fun_tool_plane_face(fac)? {
                return Ok(false);
            }
            let outerw = outer_wire(&topods::face(fac)?)?;
            if outerw.is_null() {
                return Ok(false);
            }
            for e in Explorer::new(outerw.as_ref(), ShapeEnum::Edge, ShapeEnum::Shape)? {
                let isse = hds.ds().is_section_edge(&topods::edge(&e)?, true);
                if self.splits(&e, State::On).is_empty() || !isse {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }

    /// `KPiskoletgeanalyse(config2, Stsol1, Stsol2, ires)`: the result of the tangent
    /// solids by their states and the orientation of the faces;
    /// `b.kp_iskoletgeanalyse(TopOpeBRepDSConfig::SameOriented, State::In, State::Out)`.
    pub fn kp_iskoletgeanalyse(
        &self,
        config2: TopOpeBRepDSConfig,
        stsol1: State,
        stsol2: State,
    ) -> i32 {
        use State::{In, Out};
        match (config2, stsol1, stsol2) {
            (TopOpeBRepDSConfig::DiffOriented, In, In) => RESNULL,
            (TopOpeBRepDSConfig::DiffOriented, Out, In) => RESSHAPE1,
            (TopOpeBRepDSConfig::DiffOriented, In, Out) => RESSHAPE2,
            (TopOpeBRepDSConfig::DiffOriented, Out, Out) => RESNEWSOL,
            (TopOpeBRepDSConfig::SameOriented, In, In) => RESSHAPE1,
            (TopOpeBRepDSConfig::SameOriented, Out, In) => RESNULL,
            (TopOpeBRepDSConfig::SameOriented, In, Out) => RESNEWSOL,
            (TopOpeBRepDSConfig::SameOriented, Out, Out) => RESSHAPE2,
            _ => RESUNDEF,
        }
    }
}
