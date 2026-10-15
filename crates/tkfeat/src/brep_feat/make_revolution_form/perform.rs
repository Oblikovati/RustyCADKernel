// Rust translation of BRepFeat_MakeRevolutionForm::Perform of
// ModelingAlgorithms/TKFeat/BRepFeat/BRepFeat_MakeRevolutionForm.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Perform`: the profile turned back by the second angle, turned by both angles
//! around the axis, the revolution cut by the planes at the heights of the rib on each
//! side of the plane of the wire, then glued on its sliding faces or combined with the
//! basis.
use super::BRepFeatMakeRevolutionForm;
use crate::brep_feat::common::maj_map_swept;
use crate::brep_feat::{PerfSelection, StatusError, face_until, tool};
use crate::errors::{no_such_object, null_object};
use crate::loc_ope::{CSIntersector, RevolutionForm};
use rustycad_tkbo::brep_algo_api::BRepAlgoAPICut;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Compound, Face, Shape};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{DataMap, List, Sequence};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomCurve, GeomLine};
use rustycad_tkg3d::top_abs::{self, ShapeEnum};
use rustycad_tkmath::gp::{Direction3, Point3, Transform3, Vector3};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeShape, BRepBuilderAPITransform};
use rustycad_tktopalgo::brep_lib::BRepLibMakeFace;

/// The first vertex of the profile (the origin without one).
fn first_point(pbase: &Face) -> Result<Point3, Failure> {
    for v in Explorer::new(pbase, ShapeEnum::Vertex, ShapeEnum::Shape)? {
        if !v.is_null() {
            return BRepTool::pnt(&topods::vertex(&v)?);
        }
    }
    Ok(Point3::default())
}

/// The edge of `moved` in the place of `e1` among the edges of `orig` (None when `e1`
/// is not one of them).
fn moved_edge(orig: &Face, moved: &Face, e1: &Shape) -> Result<Option<Shape>, Failure> {
    let mut ex2 = Explorer::new(moved, ShapeEnum::Edge, ShapeEnum::Shape)?;
    for e in Explorer::new(orig, ShapeEnum::Edge, ShapeEnum::Shape)? {
        if e.is_same(e1) {
            return Ok(Some(ex2.current().clone()));
        }
        ex2.next()?;
    }
    Ok(None)
}

/// Each list of `map` whose first edge is an edge of `orig` replaced by the edge in its
/// place in `moved` (Standard_NoSuchObject for an empty list, as natively).
fn remap(
    map: &mut DataMap<Shape, List<Shape>, ShapeMapHasher>,
    orig: &Face,
    moved: &Face,
) -> Result<(), Failure> {
    let keys: Vec<Shape> = map.items().map(|(k, _)| k.clone()).collect();
    for k in keys {
        let e1 =
            map.find(&k)?.first().cloned().ok_or_else(|| {
                no_such_object("BRepFeat_MakeRevolutionForm::Perform", "empty list")
            })?;
        if let Some(e2) = moved_edge(orig, moved, &e1)? {
            let l = map.change_find(&k)?;
            l.clear();
            l.append(e2);
        }
    }
    Ok(())
}

impl BRepFeatMakeRevolutionForm {
    /// The profile and the sketch face turned back by the second angle, the maps of the
    /// profile and of the sliding faces following the edges.
    fn turn_back(&mut self) -> Result<(), Failure> {
        let mut t = Transform3::default();
        t.set_rotation(self.axe, -self.angle2)?;
        let mut trsf = BRepBuilderAPITransform::new(t);
        trsf.perform(&self.rib.pbase.clone().into_shape(), false, false)?;
        let pbase = topods::face(&trsf.shape()?.clone())?;
        remap(&mut self.rib.lf_map, &self.rib.pbase, &pbase)?;
        remap(&mut self.slface, &self.rib.pbase, &pbase)?;
        self.rib.pbase = pbase;
        trsf.perform(&self.rib.skface.clone().into_shape(), false, false)?;
        self.rib.skface = topods::face(&trsf.shape()?.clone())?;
        Ok(())
    }

    /// The faces of the planes at the heights of the rib, extended over the basis.
    fn height_faces(&self, pt: Point3) -> Result<(Face, Face, Vector3), Failure> {
        let pln0 = self
            .pln
            .get()
            .ok_or_else(|| null_object("BRepFeat_MakeRevolutionForm::Perform", "null plane"))?
            .pln();
        let f = BRepLibMakeFace::from_plane(pln0)?.face()?;
        let n = Vector3::from(self.rib.normal(&f, pt)?);
        let vec1 = n * self.height1;
        let vec2 = n * (-self.height2);
        let mut f1 = BRepLibMakeFace::from_plane(pln0.translated(vec1))?.face()?;
        let mut f2 = BRepLibMakeFace::from_plane(pln0.translated(vec2))?.face()?;
        face_until(&self.rib.sbase, &mut f1)?;
        face_until(&self.rib.sbase, &mut f2)?;
        Ok((f1, f2, vec1))
    }

    /// The compound of the half spaces beyond the planes at the heights (None after
    /// BadIntersect).
    fn height_tools(
        &self,
        f1: &Face,
        f2: &Face,
        pt: Point3,
        vec1: Vector3,
    ) -> Result<Option<Shape>, Failure> {
        let normale: Handle<dyn GeomCurve> =
            Handle::new(GeomLine::from_point(pt, Direction3::from_vector(vec1)?)).cast();
        let mut scur = Sequence::default();
        scur.append(normale);
        let mut asi1 = CSIntersector::with_shape(f1);
        let mut asi2 = CSIntersector::with_shape(f2);
        asi1.perform_curves(&scur)?;
        asi2.perform_curves(&scur)?;
        if !asi1.is_done() || !asi2.is_done() || asi1.nb_points(1)? != 1 || asi2.nb_points(1)? != 1
        {
            return Ok(None);
        }
        let (p1, p2) = (asi1.point(1, 1)?, asi2.point(1, 1)?);
        let b = BRepBuilder::default();
        let mut comp = Compound::default();
        b.make_compound(&mut comp);
        let mut comp = comp.into_shape();
        let s1 = tool(f1, p1.face(), p1.orientation())?;
        let s2 = tool(f2, p2.face(), top_abs::reverse(p2.orientation()))?;
        for s in [s1, s2] {
            if !s.is_null() {
                b.add(&mut comp, &s)?;
            }
        }
        Ok(Some(comp))
    }

    /// `Perform()`: the rib of revolution built, glued on its sliding faces or combined
    /// with the basis; `form.perform()?`.
    pub fn perform(&mut self) -> Result<(), Failure> {
        let r = &self.rib;
        if r.sbase.is_null() || r.skface.is_null() || r.pbase.is_null() {
            self.fail(StatusError::NotInitialized);
            return Ok(());
        }
        let pt = first_point(&self.rib.pbase)?;
        if self.angle2 != 0. {
            self.turn_back()?;
        }
        let the_form =
            RevolutionForm::with_axis(&self.rib.pbase, self.axe, self.angle1 + self.angle2)?;
        let vrai_form = the_form.shape()?.clone(); // uncut primitive
        let r = &mut self.rib;
        let ends = (the_form.first_shape(), the_form.last_shape());
        maj_map_swept(
            &r.pbase.clone().into_shape(),
            ends,
            |e| Ok(the_form.shapes(e)?.clone()),
            &mut r.map,
            &mut r.f_shape,
            &mut r.l_shape,
        )?;
        r.glued_f.clear(false);
        let (f1, f2, vec1) = self.height_faces(pt)?;
        let Some(comp) = self.height_tools(&f1, &f2, pt, vec1)? else {
            self.fail(StatusError::BadIntersect);
            return Ok(());
        };
        // cut of the rib by two parallel planes
        let tr_p = BRepAlgoAPICut::from_shapes(&vrai_form, &comp)?;
        self.cut_history(&tr_p, &vrai_form, (&f1, &f2))?;
        let sliding_map = sliding_map(&tr_p, &vrai_form)?;
        self.set_glued_faces(&the_form, &sliding_map)?;
        self.finish(tr_p.shape().clone())
    }

    /// The selection by the glued faces, the checks of the profile edges and of the
    /// glued faces with a shape Until, then the reconstruction.
    fn finish(&mut self, vrai_form: Shape) -> Result<(), Failure> {
        let r = &mut self.rib;
        r.perf_selection = if r.glued_f.is_empty() {
            PerfSelection::SelectionSh
        } else {
            PerfSelection::NoSelection
        };
        for e in Explorer::new(&r.pbase, ShapeEnum::Edge, ShapeEnum::Shape)? {
            if !r.map.is_bound(&e) {
                self.fail(StatusError::IncSlidFace);
                return Ok(());
            }
        }
        let r = &mut self.rib;
        r.g_shape = vrai_form;
        if !r.glued_f.is_empty() && !r.s_until.is_null() {
            self.fail(StatusError::InvShape);
            return Ok(());
        }
        self.rib.lf_perform() // topological reconstruction
    }
}

/// The faces of the revolution mapped to what the cut by the planes made of them
/// (themselves when unmodified, none when deleted).
fn sliding_map(
    tr_p: &BRepAlgoAPICut,
    vrai_form: &Shape,
) -> Result<DataMap<Shape, List<Shape>, ShapeMapHasher>, Failure> {
    let mut map: DataMap<Shape, List<Shape>, ShapeMapHasher> = DataMap::default();
    for fac in Explorer::new(vrai_form, ShapeEnum::Face, ShapeEnum::Shape)? {
        let mut thelist = List::default();
        if !tr_p.is_deleted(&fac)? {
            let desfaces = tr_p.modified(&fac)?;
            if desfaces.is_empty() {
                thelist.append(fac.clone());
            } else {
                for d in desfaces {
                    thelist.append(d);
                }
            }
        }
        map.bind(fac, thelist);
    }
    Ok(map)
}
