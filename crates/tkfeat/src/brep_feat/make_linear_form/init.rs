// Rust translation of BRepFeat_MakeLinearForm::Init of
// ModelingAlgorithms/TKFeat/BRepFeat/BRepFeat_MakeLinearForm.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Init`: the wire centred between the directions when the second one is null, the
//! bounding face of the basis in the plane, the extreme faces of the wire, whether the
//! rib can slide on its extreme faces, and its profile (sliding: the wire itself as the
//! base; else the profile face closed on the bounding box).
use super::{BRepFeatMakeLinearForm, nulldir};
use crate::brep_feat::StatusError;
use crate::brep_feat::rib_slot::{Extremes, ProfileInput};
use rustycad_tkbo::brep_algo_api::BRepAlgoAPICommon;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Face, Shape, Shell, Wire};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{
    GeomCylindricalSurface, GeomGeometry, GeomPlane, GeomRectangularTrimmedSurface,
};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::{Point3, Transform3, Vector3};
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPITransform;
use rustycad_tktopalgo::brep_extrema::BRepExtremaExtCF;
use rustycad_tktopalgo::brep_lib::{BRepLibMakeEdge, BRepLibMakeFace};
use std::f64::consts::PI;

/// The largest tolerance of the vertices of `s`, at least `tol`.
fn max_vertex_tolerance(s: &Shape, mut tol: f64) -> Result<f64, Failure> {
    for v in Explorer::new(s, ShapeEnum::Vertex, ShapeEnum::Shape)? {
        let t = BRepTool::vertex_tolerance(&topods::vertex(&v)?)?;
        if t > tol {
            tol = t;
        }
    }
    Ok(tol)
}

/// Whether the segment from `p` along `d` touches the face once, within its tolerance.
fn touches(p: Point3, d: Vector3, face: &Face) -> Result<bool, Failure> {
    let p1 = Point3::new(p.x() + d.x(), p.y() + d.y(), p.z() + d.z());
    let ee = BRepLibMakeEdge::from_points(p, p1)?.edge()?;
    let ext = BRepExtremaExtCF::with_edge_face(&ee, face)?;
    let tol = BRepTool::face_tolerance(face)?;
    Ok(ext.nb_ext() == 1 && ext.square_distance(1)? <= tol * tol)
}

/// Whether the surface of the face is a plane or a cylinder (the rib may slide on it).
fn slides_on(face: &Face) -> Result<bool, Failure> {
    let mut s = BRepTool::surface(face)?;
    let trimmed: Handle<GeomRectangularTrimmedSurface> = s.cast();
    if let Some(t) = trimmed.get() {
        s = t.basis_surface();
    }
    Ok(!s.cast::<GeomPlane>().is_null() || !s.cast::<GeomCylindricalSurface>().is_null())
}

impl BRepFeatMakeLinearForm {
    /// The data of a new form reset.
    fn reset(&mut self, sbase: &Shape) {
        let r = &mut self.rib;
        r.base.set_done();
        r.base.my_generated.clear();
        r.lf_map.clear(false);
        r.base.my_shape.nullify();
        r.map.clear(false);
        r.f_shape.nullify();
        r.l_shape.nullify();
        r.sbase = sbase.clone();
        r.skface.nullify();
        r.pbase.nullify();
        r.g_shape.nullify();
        r.s_until.nullify();
        self.list_of_edges.clear();
        self.slface.clear(false);
    }

    /// The directions checked (opposite, else BadDirect), or the wire, the plane and the
    /// directions centred on the middle of the translation when the second is null.
    fn centre(&mut self, direc: Vector3, direc1: Vector3) -> Result<bool, Failure> {
        if !self.dir1.is_equal(nulldir(), self.tol, self.tol)? {
            let ang = self.dir1.angle(self.dir)?;
            if ang != PI {
                self.fail(StatusError::BadDirect);
                return Ok(false);
            }
            return Ok(true);
        }
        // Rib is centre in the middle of translation
        let dir_translation = (direc + direc1) * 0.5;
        let mut t = Transform3::default();
        t.set_translation(dir_translation);
        let mut trf = BRepBuilderAPITransform::new(t);
        trf.perform(&self.rib.wire.clone().into_shape(), false, false)?;
        self.rib.wire = topods::wire(
            &rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape::shape(&mut trf)?
                .clone(),
        )?;
        self.dir = direc - dir_translation;
        self.dir1 = direc1 - dir_translation;
        // the plane handle is shared with the caller, moved as natively
        if let Some(p) = self.pln.get() {
            p.transform(t)?;
        }
        Ok(true)
    }

    /// The face of the bounding box of the basis in the plane of the rib.
    fn bound_face(&mut self) -> Result<Face, Failure> {
        let (bnd, first_corner, last_corner) =
            self.rib.height_max(&self.rib.sbase, &Shape::default())?;
        self.bnd = bnd;
        let bnd_box = BRepPrimAPIMakeBox::from_points(first_corner, last_corner)?.solid()?;
        let pln = self.plane()?.pln();
        let b = 6. * self.bnd;
        let plane_face = BRepLibMakeFace::from_plane_bounds(pln, (-b, b, -b, b))?.face()?;
        let plane_s = BRepAlgoAPICommon::from_shapes(&bnd_box, &plane_face)?;
        let www = topods::wire(
            Explorer::new(plane_s.shape(), ShapeEnum::Wire, ShapeEnum::Shape)?.current(),
        )?;
        BRepLibMakeFace::from_plane_wire(pln, &www, true)?.face()
    }

    /// The plane of the rib (Standard_NullObject when null).
    pub(super) fn plane(&self) -> Result<&GeomPlane, Failure> {
        self.pln
            .get()
            .ok_or_else(|| crate::errors::null_object("BRepFeat_MakeLinearForm", "null plane"))
    }

    /// Whether the rib slides: its first face a plane or a cylinder, and the segments
    /// along the directions from both end points touching the extreme faces.
    fn sliding_allowed(&self, modify: bool, x: &Extremes) -> Result<bool, Failure> {
        let mut sliding = modify && slides_on(&x.first_face)?;
        // Control only start and end points -> no control at the middle - improve
        let r = &self.rib;
        if sliding {
            sliding = touches(r.first_pnt, self.dir, &x.first_face)?
                && touches(r.last_pnt, self.dir, &x.last_face)?;
        }
        if !self
            .dir1
            .is_equal(nulldir(), precision::confusion(), precision::confusion())?
            && sliding
        {
            sliding = touches(r.first_pnt, self.dir1, &x.first_face)?
                && touches(r.last_pnt, self.dir1, &x.last_face)?;
        }
        Ok(sliding)
    }

    /// `Init(Sbase, W, Plane, Direc, Direc1, Mode, Modify)`: the basis, the wire in the
    /// plane, the directions (opposite, or the second null to centre the rib) and the
    /// mode (0 slot, otherwise rib), sliding on request; the status error tells why it
    /// failed; `form.init(&sbase, &w, &pln, d, d1, 1, true)?`.
    #[allow(clippy::too_many_arguments)]
    pub fn init(
        &mut self,
        sbase: &Shape,
        w: &Wire,
        plane: &Handle<GeomPlane>,
        direc: Vector3,
        direc1: Vector3,
        mode: i32,
        modify: bool,
    ) -> Result<(), Failure> {
        self.reset(sbase);
        self.rib.wire = topods::wire(&w.oriented(Orientation::Forward))?;
        (self.dir, self.dir1, self.pln) = (direc, direc1, plane.clone());
        self.rib.fuse = mode != 0;
        // ---Determine Tolerance : max tolerance on parameters
        self.tol = max_vertex_tolerance(
            sbase,
            max_vertex_tolerance(&self.rib.wire.clone().into_shape(), precision::confusion())?,
        )?;
        if !self.centre(direc, direc1)? {
            return Ok(());
        }
        let bnd_face = self.bound_face()?;
        // ---Find support faces of the rib
        let mut x = Extremes::default();
        if !self.rib.extreme_faces(false, self.bnd, &self.pln, &mut x)? {
            self.fail(StatusError::NoExtFace);
            return Ok(());
        }
        let input = ProfileInput {
            revol_rib: false,
            my_tol: self.tol,
            pln: self.pln.clone(),
            bnd: self.bnd,
            bnd_face,
            check_pnt: self.rib.check_point(&x.first_edge, &self.pln)?,
        };
        self.rib.first_pnt = BRepTool::pnt(&x.first_vertex)?;
        self.rib.last_pnt = BRepTool::pnt(&x.last_vertex)?;
        let mut sli_list = List::default();
        sli_list.append(x.first_face.clone().into_shape());
        let sliding = self.sliding_allowed(modify, &x)?;
        self.profile(sliding, &input, &x, &mut sli_list)
    }

    /// The profile of the rib: the sliding base or the profile face, then the faces of
    /// the basis bound to themselves in the history.
    fn profile(
        &mut self,
        mut sliding: bool,
        input: &ProfileInput,
        x: &Extremes,
        sli_list: &mut List<Shape>,
    ) -> Result<(), Failure> {
        let mut concavite = 3; // a priori the profile is not concave
        if sliding {
            let mut prof = Face::default();
            if !self
                .rib
                .sliding_profile(&mut prof, input, &mut concavite, x)?
            {
                self.fail(StatusError::NoFaceProf);
                return Ok(());
            }
            // ---Propagation on faces of the initial shape to find the faces concerned
            let mut falseside = true;
            sliding = self.propagate(
                sli_list,
                &prof,
                self.rib.first_pnt,
                self.rib.last_pnt,
                &mut falseside,
            )?;
            if !falseside {
                self.fail(StatusError::FalseSide);
                return Ok(());
            }
        }
        let w = self.sliding_base(&mut sliding, x)?;
        if sliding {
            let b = BRepBuilder::default();
            let mut f = Face::default();
            b.make_face_on_surface(&mut f, &self.pln.clone().cast(), self.tol)?;
            w.set_closed(BRepTool::is_closed(&w)?)?;
            b.add(&mut f, &w)?;
            self.rib.skface = f.clone();
            self.rib.pbase = f;
            self.rib.s_until.nullify();
        } else if !self.no_sliding(input, &mut concavite, x, sli_list)? {
            return Ok(());
        }
        self.rib.sliding = sliding;
        for f in Explorer::new(&self.rib.sbase, ShapeEnum::Face, ShapeEnum::Shape)? {
            let mut thelist = List::default();
            thelist.append(f.clone());
            self.rib.map.bind(f, thelist);
        }
        Ok(())
    }

    /// The profile face of a rib not sliding, and the shell of the faces it crosses as
    /// the shape Until (false after a failure).
    fn no_sliding(
        &mut self,
        input: &ProfileInput,
        concavite: &mut i32,
        x: &Extremes,
        sli_list: &mut List<Shape>,
    ) -> Result<bool, Failure> {
        let mut prof = Face::default();
        if !self
            .rib
            .no_sliding_profile(&mut prof, input, concavite, x)?
        {
            self.fail(StatusError::NoFaceProf);
            return Ok(false);
        }
        let mut falseside = true;
        self.propagate(
            sli_list,
            &prof,
            self.rib.first_pnt,
            self.rib.last_pnt,
            &mut falseside,
        )?;
        if !falseside {
            self.fail(StatusError::FalseSide);
            return Ok(false);
        }
        self.slface.clear(false);
        let b = BRepBuilder::default();
        let mut shell = Shell::default();
        b.make_shell(&mut shell);
        let mut comp = shell.into_shape();
        for f in sli_list.iter() {
            b.add(&mut comp, f)?;
        }
        comp.set_closed(BRepTool::is_closed(&comp)?)?;
        self.rib.s_until = comp;
        self.rib.skface = prof.clone();
        self.rib.pbase = prof;
        Ok(true)
    }
}
