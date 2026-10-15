//! The paves of the point topology of an edge and the parameters on the reference edge (TopOpeBRepBuild_GridEE.cxx).
use super::TopOpeBRepBuildBuilder;
use super::grid_ee::{fun_aresamegeom, null_interference};
use crate::top_ope_brep_build::{
    TopOpeBRepBuildGTopo, TopOpeBRepBuildPave, TopOpeBRepBuildPaveSet,
};
use crate::top_ope_brep_ds::export::fun_ds_getoov;
use crate::top_ope_brep_ds::{TopOpeBRepDSConfig, TopOpeBRepDSKind, TopOpeBRepDSPointIterator};
use crate::top_ope_brep_tool::{TopOpeBRepToolTOOL, fc2d_curve_on_surface, fun_tool_bounds};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{self, Edge, Shape, Vertex};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::geom::GeomPlane;
use rustycad_tkg3d::top_abs::{self, Orientation, State};
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::Point2;
use rustycad_tkmath::{elclib, elslib};

impl TopOpeBRepBuildBuilder {
    /// The pave of a vertex at a parameter oriented `ori`, with its same domain vertex.
    fn make_pave(
        &self,
        vpv: &Shape,
        vig: &Shape,
        ori: Orientation,
        par: f64,
        has_vsd: Option<i32>,
        st1: TopOpeBRepDSKind,
    ) -> Result<Handle<TopOpeBRepBuildPave>, Failure> {
        let pv = Handle::new(TopOpeBRepBuildPave::new(&vpv.oriented(ori), par, false));
        let p = pv.get().ok_or_else(null_interference)?;
        if let Some(ivre) = has_vsd {
            p.set_has_same_domain(true);
            let hds = self.hds()?;
            let vsd = hds
                .ds()
                .shape_same_domain(&vpv.oriented(ori))
                .first()
                .cloned()
                .unwrap_or_default();
            let ivsd = hds.shape_index(&vsd, true);
            p.set_same_domain(if ivsd == ivre { vig } else { &vsd });
        }
        p.set_interference_type(st1);
        Ok(pv)
    }

    /// `GFillPointTopologyPVS(E, EPit, G1, PVS)`: the pave of the current point or
    /// vertex (its same domain reference) on the state ON of the first shape, at its
    /// parameter on the reference edge; on a closed reference edge, an INTERNAL vertex
    /// kept IN (EXTERNAL kept OUT) at its closing vertex gives the two bound paves;
    /// `b.g_fill_point_topology_pvs_at(&e, &it, &g, &mut pvs)?`.
    pub fn g_fill_point_topology_pvs_at(
        &self,
        e: &Shape,
        epit: &TopOpeBRepDSPointIterator,
        g1: &TopOpeBRepBuildGTopo,
        pvs: &mut TopOpeBRepBuildPaveSet,
    ) -> Result<(), Failure> {
        let epvs = pvs.edge().clone().into_shape();
        let st1 = epit
            .value()?
            .get()
            .ok_or_else(null_interference)?
            .support_type();
        let (tb, _) = g1.states_on()?;
        let conf = g1.config1();
        let hds = self.hds()?;
        let ig = epit.current()?;
        let ispoint = epit.is_point()?;
        let vig = if ispoint {
            self.new_vertex(ig)?.clone()
        } else {
            hds.shape(ig, true)
        };
        if vig.is_null() {
            return Ok(());
        }
        let mut has_vsd = None;
        let mut vpv = vig.clone();
        if !ispoint && hds.has_same_domain(&vig, true) {
            let ivre = hds.same_domain_reference(&vig);
            has_vsd = Some(ivre);
            vpv = hds.shape(ivre, true);
        }
        let mut par = epit.parameter()?;
        let mut ori = epit.orientation(tb)?;
        if conf == TopOpeBRepDSConfig::DiffOriented {
            ori = top_abs::complement(ori);
        }
        if !fun_aresamegeom(e, &epvs)? {
            ori = top_abs::complement(ori);
        }
        let lesmemes = e.is_equal(self.my_edge_reference.as_ref());
        if !lesmemes
            && let Some(p) = self.g_param_on_reference(&topods::vertex(&vpv)?, &topods::edge(e)?)?
        {
            par = p;
        }
        if !self.kp_bound(lesmemes, epit, tb, &vig)? {
            pvs.append(&self.make_pave(&vpv, &vig, ori, par, has_vsd, st1)?);
            return Ok(());
        }
        let (parf, parl) = fun_tool_bounds(&self.my_edge_reference)?;
        pvs.append(&self.make_pave(&vpv, &vig, Orientation::Forward, parf, has_vsd, st1)?);
        pvs.append(&self.make_pave(&vpv, &vig, Orientation::Reversed, parl, has_vsd, st1)?);
        pvs.remove_pv(false);
        Ok(())
    }

    /// Whether the vertex is the closing vertex (or its same domain one) of the closed
    /// reference edge, INTERNAL kept IN or EXTERNAL kept OUT.
    fn kp_bound(
        &self,
        lesmemes: bool,
        epit: &TopOpeBRepDSPointIterator,
        tb: State,
        vig: &Shape,
    ) -> Result<bool, Failure> {
        let vclo = TopOpeBRepToolTOOL::closed_e(&self.my_edge_reference)?;
        let ori_i = epit.orientation(State::In)?;
        let intext = (ori_i == Orientation::Internal && tb == State::In)
            || (ori_i == Orientation::External && tb == State::Out);
        let Some(vclo) = vclo.filter(|_| lesmemes && intext) else {
            return Ok(false);
        };
        if vclo.as_ref().is_same(vig) {
            return Ok(true);
        }
        Ok(fun_ds_getoov(vig, &self.hds()?.ds()).is_some_and(|vsd| vclo.as_ref().is_same(&vsd)))
    }

    /// `GParamOnReference(V, E, P)`: the parameter of the vertex on the p-curve of the
    /// reference edge on the plane of the reference face, for a conic or line p-curve
    /// (None otherwise); `b.g_param_on_reference(&v, &e)?`.
    pub fn g_param_on_reference(&self, v: &Vertex, _e: &Edge) -> Result<Option<f64>, Failure> {
        let su = BRepTool::surface(&self.my_face_reference)?;
        let suplan: Handle<GeomPlane> = su.cast();
        let Some(plane) = suplan.get() else {
            return Ok(None);
        };
        let (u, vv) = elslib::plane_parameters(plane.pln().position(), BRepTool::pnt(v)?);
        let p2 = Point2::new(u, vv);
        let (c2d, _, _, _) =
            fc2d_curve_on_surface(&self.my_edge_reference, &self.my_face_reference, false)?;
        if c2d.is_null() {
            return Err(Failure::with_type(
                "Standard_ProgramError",
                "TopOpeBRepBuild_Builder::GParamOnReference",
                "TopOpeBRepBuild_Builder",
            ));
        }
        let ac = Geom2dAdaptorCurve::with_curve(&c2d)?;
        Ok(match ac.get_type()? {
            CurveType::Line => Some(elclib::parameter(&ac.line()?, p2)?),
            CurveType::Circle => Some(elclib::parameter(&ac.circle()?, p2)?),
            CurveType::Ellipse => Some(elclib::parameter(&ac.ellipse()?, p2)?),
            CurveType::Hyperbola => Some(elclib::parameter(&ac.hyperbola()?, p2)?),
            CurveType::Parabola => Some(elclib::parameter(&ac.parabola()?, p2)?),
            _ => None,
        })
    }
}
