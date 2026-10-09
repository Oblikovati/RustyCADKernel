// Rust translation of BRepAlgo_NormalProjection::Build
// (ModelingAlgorithms/TKBool/BRepAlgo/BRepAlgo_NormalProjection.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The projections of every edge on every face: the punctual ones as vertices, the
//! others as edges trimmed by the face (its section with the edge, else by the
//! classification of their middle).
use super::BRepAlgoNormalProjection;
use super::edge::ProjectedPiece;
use rustycad_tkbo::brep_algo_api::BRepAlgoAPISection;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve, BRepAdaptorSurface};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Compound, Face, Shape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::top_abs::{ShapeEnum, State};
use rustycad_tkgeombase::proj_lib::CompProjectedCurve;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
use rustycad_tktopalgo::brep_lib::BRepLibMakeVertex;
use rustycad_tktopalgo::brep_top_adaptor::BRepTopAdaptorFClass2d;

/// The sub-shapes of a kind, in explorer order.
fn all_of(shape: &Shape, kind: ShapeEnum) -> Result<Vec<Shape>, Failure> {
    let mut out = Vec::new();
    let mut ex = Explorer::new(shape, kind, ShapeEnum::Shape)?;
    while ex.more() {
        out.push(ex.current().clone());
        ex.next()?;
    }
    Ok(out)
}

/// The output of the projection of one edge: the result compound, the vertices of the
/// punctual projections and the descendants of the edge.
pub(super) struct BuildOutput {
    pub vertex_res: Compound,
    pub ya_vertex_res: bool,
    pub descen_list: List<Shape>,
}

impl BRepAlgoNormalProjection {
    /// `Build()`: every edge to project on every face of the shape (the projection
    /// within the maximal distance, approximated, trimmed by the face when limited);
    /// the projected vertices added last; `p.build()?`.
    pub fn build(&mut self) -> Result<(), Failure> {
        self.my_is_done = false;
        let edges = all_of(&self.my_to_proj, ShapeEnum::Edge)?;
        let faces = all_of(&self.my_shape, ShapeEnum::Face)?;
        let bb = BRepBuilder::default();
        let mut res = Compound::default();
        bb.make_compound(&mut res);
        self.my_res = res.into_shape();
        let mut out = BuildOutput {
            vertex_res: Compound::default(),
            ya_vertex_res: false,
            descen_list: List::default(),
        };
        bb.make_compound(&mut out.vertex_res);
        for edge in &edges {
            out.descen_list.clear();
            let hcur = Handle::new(BRepAdaptorCurve::with_edge(&topods::edge(edge)?)?);
            let elementary: bool = self.is_elementary(hcur.get().ok_or_else(null_adaptor)?)?;
            for face in &faces {
                self.project_on_face(edge, &topods::face(face)?, (&hcur, elementary), &mut out)?;
            }
            self.my_descendants
                .bind(edge.clone(), std::mem::take(&mut out.descen_list));
        }
        // An eventual wire creation is a specific method, BuildWire, that the user
        // calls; otherwise the relations of the ancestor and couple maps are lost.
        if out.ya_vertex_res {
            bb.add(&mut self.my_res, &out.vertex_res)?;
        }
        self.my_is_done = true;
        Ok(())
    }

    /// The projections of one edge on one face.
    fn project_on_face(
        &mut self,
        edge: &Shape,
        face: &Face,
        (hcur, elementary): (&Handle<BRepAdaptorCurve>, bool),
        out: &mut BuildOutput,
    ) -> Result<(), Failure> {
        let hsur: Handle<dyn Adaptor3dSurface> =
            Handle::new(BRepAdaptorSurface::with_face(face, true)?).cast();
        let s = hsur.get().ok_or_else(null_adaptor)?;
        // The computation of TolU and TolV.
        let tol_u: f64 = s.u_resolution(self.my_tol3d)? / 20.;
        let tol_v: f64 = s.v_resolution(self.my_tol3d)? / 20.;
        let curve: Handle<dyn Adaptor3dCurve> = hcur.clone().cast();
        let h_projector = Handle::new(CompProjectedCurve::with_max_dist(
            &hsur,
            &curve,
            tol_u,
            tol_v,
            self.my_max_dist,
        )?);
        let projector = h_projector.get().ok_or_else(null_adaptor)?;
        let mut degenerated = false;
        for k in 1..=projector.nb_curves() {
            let (single, p2d) = projector.is_single_pnt(k)?;
            if single {
                let surface = projector.get_surface();
                let p = surface
                    .get()
                    .ok_or_else(null_adaptor)?
                    .d0(p2d.x(), p2d.y())?;
                let prj: Shape = BRepLibMakeVertex::new(p)?.vertex()?.into_shape();
                out.descen_list.append(prj.clone());
                BRepBuilder::default().add(&mut out.vertex_res, &prj)?;
                out.ya_vertex_res = true;
                self.my_ancestor_map.bind(prj, edge.clone());
                continue;
            }
            let piece = ProjectedPiece::of(
                self,
                (&h_projector, k),
                (hcur, &hsur, face),
                (elementary, &mut degenerated),
            )?;
            let Some(piece) = piece else { continue };
            self.store_piece(edge, face, piece, out)?;
        }
        Ok(())
    }

    /// The projected edge kept: trimmed by the face (its section, or its middle in the
    /// face when the section fails or the edge is degenerated), or whole.
    fn store_piece(
        &mut self,
        edge: &Shape,
        face: &Face,
        piece: ProjectedPiece,
        out: &mut BuildOutput,
    ) -> Result<(), Failure> {
        if !self.my_face_bounds {
            return self.keep(edge, face, &piece.prj, out);
        }
        if !piece.degenerated {
            // The Boolean section gives the parts of the projected edge inside the face.
            let a_section = BRepAlgoAPISection::from_shapes(face.as_ref(), &piece.prj, None, true)?;
            if a_section.is_done() {
                for a_e in all_of(a_section.shape(), ShapeEnum::Edge)? {
                    BRepBuilder::default().add(&mut self.my_res, &a_e)?;
                    self.my_ancestor_map.bind(a_e.clone(), edge.clone());
                    self.my_corresp.bind(a_e, face.as_ref().clone());
                }
                return Ok(());
            }
        }
        // The section failed or the projection is degenerated: the middle classified.
        let classifier = BRepTopAdaptorFClass2d::new(face, precision::confusion())?;
        let pcur = piece.pcur2d.get().ok_or_else(null_adaptor)?;
        let pmil: f64 = (pcur.first_parameter() + pcur.last_parameter()) / 2.;
        let state = classifier.perform(pcur.value(pmil)?, true)?;
        if state == State::In || state == State::On {
            self.keep(edge, face, &piece.prj, out)?;
        }
        Ok(())
    }

    /// A projected edge added to the result, the descendants and the maps.
    fn keep(
        &mut self,
        edge: &Shape,
        face: &Face,
        prj: &Shape,
        out: &mut BuildOutput,
    ) -> Result<(), Failure> {
        BRepBuilder::default().add(&mut self.my_res, prj)?;
        out.descen_list.append(prj.clone());
        self.my_ancestor_map.bind(prj.clone(), edge.clone());
        self.my_corresp.bind(prj.clone(), face.as_ref().clone());
        Ok(())
    }
}

/// Standard_NullObject of a null adaptor (natively a crash).
pub(super) fn null_adaptor() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "BRepAlgo_NormalProjection: null adaptor or curve; expected one",
        "BRepAlgo_NormalProjection",
    )
}
