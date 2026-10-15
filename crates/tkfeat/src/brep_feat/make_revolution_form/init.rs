// Rust translation of BRepFeat_MakeRevolutionForm::Init of
// ModelingAlgorithms/TKFeat/BRepFeat/BRepFeat_MakeRevolutionForm.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Init`: the angles the rib turns on each side of its plane (enough to reach the
//! heights at the smallest radius of the wire), the bounding face of the basis, the
//! extreme faces of the wire, whether it slides on its extreme faces, and its profile.
use super::BRepFeatMakeRevolutionForm;
use crate::brep_feat::StatusError;
use crate::brep_feat::rib_slot::{
    Extremes, ProfileInput, SlidingBase, max_vertex_tolerance, shell_of,
};
use crate::errors::null_object;
use crate::loc_ope::CSIntersector;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Face, Shape, Wire};
use rustycad_tkernel::ncollection::{List, Sequence};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomCurve, GeomLine, GeomPlane};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkgeomalgo::geom_api::{self, ProjectPointOnCurve};
use rustycad_tkgeomalgo::geom2d_api::ExtremaCurveCurve;
use rustycad_tkmath::gp::Axis1;
use std::f64::consts::PI;

/// The range of the axis line inside the basis (its first and last intersections, else
/// the whole line).
fn line_range(sbase: &Shape, line: &Handle<dyn GeomCurve>) -> Result<(f64, f64), Failure> {
    let mut asi = CSIntersector::with_shape(sbase);
    let mut scur = Sequence::default();
    scur.append(line.clone());
    asi.perform_curves(&scur)?;
    if asi.is_done() && asi.nb_points(1)? >= 2 {
        let n = asi.nb_points(1)?;
        return Ok((asi.point(1, 1)?.parameter(), asi.point(1, n)?.parameter()));
    }
    Ok((f64::MIN, f64::MAX))
}

/// The smallest positive distance of the edges of the wire to the axis (None when an end
/// does not project on it, NoProjPt natively).
fn min_radius(
    w: &Wire,
    line: &Handle<dyn GeomCurve>,
    plane: &GeomPlane,
    (line_first, line_last): (f64, f64),
) -> Result<Option<f64>, Failure> {
    let ln2d = geom_api::to_2d(line, &plane.pln())?;
    let mut rad = f64::MAX;
    for e in Explorer::new(w, ShapeEnum::Edge, ShapeEnum::Shape)? {
        let (c, f, l) = BRepTool::curve(&topods::edge(&e)?)?;
        let c2d = geom_api::to_2d(&c, &plane.pln())?;
        let extr = ExtremaCurveCurve::new(&ln2d, &c2d, line_first, line_last, f, l)?;
        let mut len = f64::MAX;
        if extr.nb_extrema() >= 1 {
            len = extr.lower_distance()?;
        }
        let cv = c.get().ok_or_else(|| {
            null_object("BRepFeat_MakeRevolutionForm::Init", "edge without curve")
        })?;
        let proj1 = ProjectPointOnCurve::with_curve(cv.value(f)?, line)?;
        let proj2 = ProjectPointOnCurve::with_curve(cv.value(l)?, line)?;
        if proj1.nb_points() < 1 || proj2.nb_points() < 1 {
            return Ok(None);
        }
        let par = proj1.distance(1)?.min(proj2.distance(1)?);
        if par < len {
            len = par;
        }
        if len < rad && len > 0. {
            rad = len;
        }
    }
    Ok(Some(rad))
}

/// The angle turned to reach the height `h` at the radius `rad`, a tenth of a half turn
/// more, at most a quarter turn.
fn angle(h: f64, rad: f64) -> f64 {
    let a = (h / rad).asin() + PI / 10.;
    if (a - PI / 2.) > precision::confusion() {
        PI / 2.
    } else {
        a
    }
}

impl BRepFeatMakeRevolutionForm {
    /// The angles of the rib from the smallest radius of the wire (false after NoProjPt).
    fn angles(
        &mut self,
        sbase: &Shape,
        w: &Wire,
        plane: &Handle<GeomPlane>,
        h1: f64,
        h2: f64,
    ) -> Result<bool, Failure> {
        let line: Handle<dyn GeomCurve> = Handle::new(GeomLine::new(self.axe)).cast();
        let range = line_range(sbase, &line)?;
        let pl = plane
            .get()
            .ok_or_else(|| null_object("BRepFeat_MakeRevolutionForm::Init", "null plane"))?;
        let Some(mut rad) = min_radius(w, &line, pl, range)? else {
            self.fail(StatusError::NoProjPt);
            return Ok(false);
        };
        let height = h1.min(h2);
        if rad <= height {
            rad = height + 0.01 * height;
        }
        self.angle1 = angle(h1, rad);
        self.angle2 = angle(h2, rad);
        Ok(true)
    }

    /// The data of a new form reset (the profile map, the form and the shape Until kept,
    /// as natively).
    fn reset(
        &mut self,
        sbase: &Shape,
        w: &Wire,
        plane: &Handle<GeomPlane>,
        (h1, h2): (f64, f64),
        mode: i32,
    ) -> Result<(), Failure> {
        let r = &mut self.rib;
        r.skface.nullify();
        r.pbase.nullify();
        r.fuse = mode != 0;
        // ---Determination Tolerance : tolerance max on parameters
        self.tol = max_vertex_tolerance(sbase, max_vertex_tolerance(w, precision::confusion())?)?;
        r.wire = topods::wire(&w.oriented(Orientation::Forward))?;
        self.pln = plane.clone();
        (self.height1, self.height2) = (h1, h2);
        r.sbase = sbase.clone();
        self.slface.clear(false);
        r.base.my_shape.nullify();
        r.map.clear(false);
        r.f_shape.nullify();
        r.l_shape.nullify();
        Ok(())
    }

    /// `Init(Sbase, W, Plane, Axis, H1, H2, Mode, Modify)`: the basis, the wire in the
    /// plane, the axis, the heights of the rib on each side of the plane and the mode (0
    /// groove, otherwise rib), sliding on request; the status error tells why it failed;
    /// `form.init(&sbase, &w, &pln, axis, 1., 1., 1, true)?`.
    #[allow(clippy::too_many_arguments)]
    pub fn init(
        &mut self,
        sbase: &Shape,
        w: &Wire,
        plane: &Handle<GeomPlane>,
        axis: Axis1,
        h1: f64,
        h2: f64,
        mode: i32,
        modify: bool,
    ) -> Result<(), Failure> {
        self.rib.base.set_done();
        self.axe = axis;
        if !self.angles(sbase, w, plane, h1, h2)? {
            return Ok(());
        }
        self.reset(sbase, w, plane, (h1, h2), mode)?;
        let (bnd, bnd_face) = self.rib.bound_face(&self.pln)?;
        self.bnd = bnd;
        // ---Find base faces of the rib
        let mut x = Extremes::default();
        if !self.rib.extreme_faces(true, self.bnd, &self.pln, &mut x)? {
            self.fail(StatusError::NoExtFace);
            return Ok(());
        }
        let input = ProfileInput {
            revol_rib: true,
            my_tol: self.tol,
            pln: self.pln.clone(),
            bnd,
            bnd_face,
            check_pnt: self.rib.check_point(&x.first_edge, &self.pln)?,
        };
        self.rib.first_pnt = BRepTool::pnt(&x.first_vertex)?;
        self.rib.last_pnt = BRepTool::pnt(&x.last_vertex)?;
        let mut sli_list = List::default();
        sli_list.append(x.first_face.clone().into_shape());
        let Some(sliding) = self.sliding_allowed(modify, &x)? else {
            return Ok(());
        };
        self.profile(sliding, input, &x, &mut sli_list)
    }

    /// The profile of the rib: the sliding base or the profile face, then the faces of
    /// the basis bound to themselves in the history.
    fn profile(
        &mut self,
        mut sliding: bool,
        mut input: ProfileInput,
        x: &Extremes,
        sli_list: &mut List<Shape>,
    ) -> Result<(), Failure> {
        let mut concavite = 3; // a priori the profile is not concave
        if sliding {
            let mut prof = Face::default();
            if !self
                .rib
                .sliding_profile(&mut prof, &input, &mut concavite, x)?
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
        let mut sb = SlidingBase {
            rib: &mut self.rib,
            tol: self.tol,
            list_of_edges: &mut self.list_of_edges,
            slface: &mut self.slface,
            revol: true,
        };
        let w = sb.sliding_base(&mut sliding, x)?;
        if sliding {
            self.rib.sliding_face(&self.pln, &w, self.tol)?;
        } else if !self.no_sliding(&mut input, &mut concavite, x, sli_list)? {
            return Ok(());
        }
        self.sliding = sliding;
        self.rib.bind_basis_faces()
    }

    /// The profile face of a rib not sliding, closed on the part of the bounding face cut
    /// by the axis holding the material, and the shell of the faces it crosses as the
    /// shape Until (false after a failure).
    fn no_sliding(
        &mut self,
        input: &mut ProfileInput,
        concavite: &mut i32,
        x: &Extremes,
        sli_list: &mut List<Shape>,
    ) -> Result<bool, Failure> {
        let Some(new_bnd_face) = self.new_bound_face(input)? else {
            self.fail(StatusError::InvShape);
            return Ok(false);
        };
        input.bnd_face = new_bnd_face;
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
        self.rib.s_until = shell_of(sli_list)?;
        self.rib.skface = prof.clone();
        self.rib.pbase = prof;
        Ok(true)
    }
}
