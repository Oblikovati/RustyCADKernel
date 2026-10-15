//! The end of a build (TopOpeBRepBuild_End.cxx): the continuities of the edges of the
//! same domain faces restored, the tolerances of the vertices of the section raised,
//! and the edges of the result fused.
use super::TopOpeBRepBuildBuilder;
use crate::top_ope_brep_tool::{
    TopOpeBRepToolFuseEdges, fc2d_curve_on_surface, fc2d_has_curve_on_surface,
};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Compound, Edge, Face, Shape, Vertex};
use rustycad_tkbrep::toptools::{
    IndexedDataMapOfShapeListOfShape, IndexedMapOfShape, ShapeMapHasher,
};
use rustycad_tkernel::ncollection::{DataMap, List, Map};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::top_abs::{ShapeEnum, State};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::Point3;

impl TopOpeBRepBuildBuilder {
    /// The split face of `f` on the state containing the edge (`f` itself when none).
    fn split_with_edge(&self, f: &Face, sta: State, e: &Edge) -> Result<Face, Failure> {
        for sf in self.splits(f.as_ref(), sta).iter() {
            if sf.shape_type()? != ShapeEnum::Face {
                continue;
            }
            if Explorer::new(sf, ShapeEnum::Edge, ShapeEnum::Shape)?.any(|x| x.is_same(e.as_ref()))
            {
                return topods::face(sf);
            }
        }
        Ok(f.clone())
    }

    /// The continuity of an unsplit edge of two faces (with a same domain face, split)
    /// set between their split faces containing it.
    fn restore_edge_continuity(
        &self,
        e: &Edge,
        lf: &List<Shape>,
        sta: State,
    ) -> Result<(), Failure> {
        let mut it = lf.iter();
        let (Some(f1), Some(f2)) = (it.next(), it.next()) else {
            return Ok(());
        };
        let (f1, f2) = (topods::face(f1)?, topods::face(f2)?);
        let c = BRepTool::continuity_on_faces(e, &f1, &f2)?;
        if c == Continuity::C0 {
            return Ok(());
        }
        let hds = self.hds()?;
        if !hds.has_same_domain(f1.as_ref(), true) && !hds.has_same_domain(f2.as_ref(), true) {
            return Ok(());
        }
        let split =
            |f: &Face| self.is_split(f.as_ref(), sta) && !self.splits(f.as_ref(), sta).is_empty();
        let (s1, s2) = (split(&f1), split(&f2));
        if !s1 && !s2 {
            return Ok(());
        }
        let ff1 = if s1 {
            self.split_with_edge(&f1, sta, e)?
        } else {
            f1
        };
        let ff2 = if s2 {
            self.split_with_edge(&f2, sta, e)?
        } else {
            f2
        };
        BRepBuilder::default().continuity_on_faces(e, &ff1, &ff2, c)
    }

    /// The continuities of the edges of the shells of a shape with same domain faces.
    fn restore_continuities(&self, s: &Shape, sta: State) -> Result<(), Failure> {
        let hds = self.hds()?;
        for sh in Explorer::new(s, ShapeEnum::Shell, ShapeEnum::Shape)? {
            if !hds.ds().has_shape(&sh, true) {
                continue;
            }
            if !Explorer::new(&sh, ShapeEnum::Face, ShapeEnum::Shape)?
                .any(|f| hds.has_same_domain(&f, true))
            {
                continue;
            }
            let mut m = IndexedDataMapOfShapeListOfShape::default();
            topexp::map_shapes_and_ancestors(s, ShapeEnum::Edge, ShapeEnum::Face, &mut m)?;
            for ie in 1..=m.len() {
                let e = topods::edge(m.find_key(ie)?)?;
                if self.is_split(e.as_ref(), sta) {
                    continue;
                }
                let lf = m.find_from_index(ie)?.clone();
                if lf.len() >= 2 {
                    self.restore_edge_continuity(&e, &lf, sta)?;
                }
            }
        }
        Ok(())
    }

    /// The point of the edge (by its p-curve, made when missing) on a face at the
    /// parameter, the new edge tolerance raised to the face tolerance.
    fn vertex_point_on_face(
        &self,
        e: &Edge,
        f: &Face,
        pv: f64,
        new_edges: &Map<Shape, ShapeMapHasher>,
    ) -> Result<Point3, Failure> {
        let bb = BRepBuilder::default();
        if !fc2d_has_curve_on_surface(e, f)? {
            let (c2d, _, _, tolpc) = fc2d_curve_on_surface(e, f, false)?;
            if c2d.is_null() {
                return Err(Failure::with_type(
                    "Standard_ProgramError",
                    "TopOpeBRepBuild_Builder::End 1",
                    "TopOpeBRepBuild_Builder",
                ));
            }
            bb.update_edge_pcurve_on_face(e, &c2d, f, BRepTool::edge_tolerance(e)?.max(tolpc))?;
        }
        let (c2d, _, _, _) = fc2d_curve_on_surface(e, f, false)?;
        let p2 = c2d
            .get()
            .ok_or_else(|| Failure::new("TopOpeBRepBuild_Builder::End: null p-curve"))?
            .value(pv)?;
        let pv3 = BRepAdaptorSurface::with_face(f, false)?.value(p2.x(), p2.y())?;
        if new_edges.contains(e.as_ref())
            && BRepTool::edge_tolerance(e)? < BRepTool::face_tolerance(f)?
        {
            bb.update_edge_tolerance(e, BRepTool::face_tolerance(f)?)?;
        }
        Ok(pv3)
    }

    /// The tolerance of a section vertex raised to the diagonal of the box of its points
    /// on its edges and their faces.
    fn raise_vertex_tolerance(
        &self,
        v: &Vertex,
        loe: &List<Shape>,
        idmoelof: &IndexedDataMapOfShapeListOfShape,
        maps: (&Map<Shape, ShapeMapHasher>, &Map<Shape, ShapeMapHasher>),
    ) -> Result<(), Failure> {
        let (new_edges, new_vertices) = maps;
        let bb = BRepBuilder::default();
        let mut tp = Vec::new();
        for e in loe.iter() {
            let e = topods::edge(e)?;
            let pv = BRepTool::parameter(v, &e)?;
            let (c3d, _, _) = BRepTool::curve(&e)?;
            if let Some(c) = c3d.get() {
                tp.push(c.value(pv)?);
            }
            for f in idmoelof.find_from_key(e.as_ref())?.iter() {
                tp.push(self.vertex_point_on_face(&e, &topods::face(f)?, pv, new_edges)?);
            }
            if new_vertices.contains(v.as_ref())
                && BRepTool::vertex_tolerance(v)? < BRepTool::edge_tolerance(&e)?
            {
                bb.update_vertex_tolerance(v, BRepTool::edge_tolerance(&e)?)?;
            }
        }
        let newtol = BRepTool::vertex_tolerance(v)?;
        let mut bx = BoundingBox3::default();
        bx.set_point(BRepTool::pnt(v)?);
        for p in &tp {
            bx.update_point(p.x(), p.y(), p.z());
        }
        let d = Point3::new(bx.x_min(), bx.y_min(), bx.z_min()).distance(&Point3::new(
            bx.x_max(),
            bx.y_max(),
            bx.z_max(),
        ));
        if d > newtol {
            bb.update_vertex_tolerance(v, d)?;
        }
        Ok(())
    }

    /// The tolerances of the vertices of the result on the section edges raised.
    fn raise_section_tolerances(&mut self) -> Result<(), Failure> {
        let (mut new_edges, mut new_vertices): (
            Map<Shape, ShapeMapHasher>,
            Map<Shape, ShapeMapHasher>,
        ) = Default::default();
        let (nc, np) = (self.hds()?.nb_curves(), self.hds()?.nb_points());
        for ic in 1..=nc {
            for e in self.new_edges(ic).iter() {
                new_edges.add(e.clone());
            }
        }
        for ip in 1..=np {
            new_vertices.add(self.new_vertex(ip)?.clone());
        }
        let bb = BRepBuilder::default();
        let mut r = Compound::default();
        bb.make_compound(&mut r);
        let mut r = r.into_shape();
        for s in self
            .merged(&self.my_shape1.clone(), self.my_state1)
            .clone()
            .iter()
        {
            bb.add(&mut r, s)?;
        }
        let loes = self.section()?.clone();
        let mut idmoelof = IndexedDataMapOfShapeListOfShape::default();
        topexp::map_shapes_and_ancestors(&r, ShapeEnum::Edge, ShapeEnum::Face, &mut idmoelof)?;
        let mut idmovloe = IndexedDataMapOfShapeListOfShape::default();
        topexp::map_shapes_and_unique_ancestors(
            &r,
            ShapeEnum::Vertex,
            ShapeEnum::Edge,
            &mut idmovloe,
            false,
        )?;
        let mut idmovloes = IndexedDataMapOfShapeListOfShape::default();
        for s in loes.iter() {
            topexp::map_shapes_and_ancestors(
                s,
                ShapeEnum::Vertex,
                ShapeEnum::Edge,
                &mut idmovloes,
            )?;
        }
        for iv in 1..=idmovloe.len() {
            let v = topods::vertex(idmovloe.find_key(iv)?)?;
            if !idmovloes.contains(v.as_ref()) {
                continue;
            }
            let loe = idmovloe.find_from_index(iv)?.clone();
            self.raise_vertex_tolerance(&v, &loe, &idmoelof, (&new_edges, &new_vertices))?;
        }
        Ok(())
    }

    /// The edges of the merged result fused (but the edges of the operands), the split
    /// and merged lists updated.
    fn fuse_result_edges(&mut self) -> Result<(), Failure> {
        let ls = self
            .change_merged(&self.my_shape1.clone(), self.my_state1)
            .clone();
        let mut new_ls = List::default();
        for sfe in ls.iter() {
            let mut fe = TopOpeBRepToolFuseEdges::new(sfe, false)?;
            let mut map_old: IndexedMapOfShape = IndexedMapOfShape::default();
            topexp::map_shapes_of_type(&self.my_shape1, ShapeEnum::Edge, &mut map_old)?;
            topexp::map_shapes_of_type(&self.my_shape2, ShapeEnum::Edge, &mut map_old)?;
            fe.avoid_edges(&map_old);
            let mle = fe.edges()?;
            let mut out = sfe.clone();
            if !mle.is_empty() {
                fe.perform()?;
                out = fe.shape()?.clone();
                let mre = fe.result_edges()?;
                let mlf = fe.faces()?;
                for sta in [State::In, State::Out, State::On] {
                    self.update_split_and_merged(&mle, &mre, &mlf, sta)?;
                }
            }
            new_ls.append(out);
        }
        *self.change_merged(&self.my_shape1.clone(), self.my_state1) = new_ls;
        Ok(())
    }

    /// `End()`: the continuities restored, the section vertex tolerances raised and the
    /// result edges fused; `b.end()?`.
    pub fn end(&mut self) -> Result<(), Failure> {
        for (s, sta) in [
            (self.my_shape1.clone(), self.my_state1),
            (self.my_shape2.clone(), self.my_state2),
        ] {
            self.restore_continuities(&s, sta)?;
        }
        self.raise_section_tolerances()?;
        self.fuse_result_edges()
    }

    /// The split edges of `e` without those fused (the merged edge of `e` then the
    /// result edge of the last fused group).
    fn update_split_edge(
        &mut self,
        e: &Shape,
        state: State,
        mle: &DataMap<i32, List<Shape>>,
        mre: &DataMap<i32, Shape>,
    ) -> Result<(), Failure> {
        let lst = self.change_split(e, state).clone();
        let mut kept = List::default();
        for edgecur in lst.iter() {
            let found = mle
                .items()
                .find(|(_, l)| l.iter().any(|ef| edgecur.is_same(ef)))
                .map(|(k, _)| *k);
            let Some(ilst) = found else {
                kept.append(edgecur.clone());
                continue;
            };
            let state_merged = if self.shape_rank(e) == 1 {
                self.my_state1
            } else {
                self.my_state2
            };
            let mut l = List::default();
            l.append(mre.find(&ilst)?.clone());
            *self.change_merged(e, state_merged) = l;
        }
        *self.change_split(e, state) = kept;
        Ok(())
    }

    /// `UpdateSplitAndMerged(mle, mre, mlf, state)`: the split edges fused removed (the
    /// merged edge set to the result edge) and the split faces replaced by their new
    /// faces; `b.update_split_and_merged(&mle, &mre, &mlf, State::In)?`.
    pub fn update_split_and_merged(
        &mut self,
        mle: &DataMap<i32, List<Shape>>,
        mre: &DataMap<i32, Shape>,
        mlf: &DataMap<Shape, Shape, ShapeMapHasher>,
        state: State,
    ) -> Result<(), Failure> {
        let keys: Vec<Shape> = self.msplit(state).items().map(|(k, _)| k.clone()).collect();
        for e in keys {
            match e.shape_type()? {
                ShapeEnum::Edge => self.update_split_edge(&e, state, mle, mre)?,
                ShapeEnum::Face => {
                    let lst = self.change_split(&e, state).clone();
                    let new: List<Shape> = lst
                        .iter()
                        .map(|f| mlf.find(f).cloned().unwrap_or_else(|_| f.clone()))
                        .collect();
                    *self.change_split(&e, state) = new;
                }
                _ => {}
            }
        }
        Ok(())
    }
}
