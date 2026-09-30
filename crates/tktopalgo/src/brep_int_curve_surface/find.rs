//! `BRepIntCurveSurface_Inter::Find` and `FindPoint` (protected natively): the walk over
//! the faces and over the intersection points of the current face
//! (docs/topalgo-design.md section 12).
use super::inter::BRepIntCurveSurfaceInter;
use crate::brep_bnd_lib::BRepBndLib;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topods::Face;
use rustycad_tkernel::standard::exceptions::null_object;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkg3d::top_abs::State;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::Point2;

impl BRepIntCurveSurfaceInter {
    /// `Find()`: the next point of the current face, else the first point IN or ON a
    /// next face whose box meets the curve. Past the last face the face index becomes
    /// one more than the index it started from, so every `Next` at the end increments
    /// it again, as natively.
    pub(super) fn find(&mut self) -> Result<(), Failure> {
        if self.current_nb_points != 0
            && self.current_index <= self.current_nb_points
            && self.find_point()?
        {
            return Ok(());
        }
        self.current_nb_points = 0;
        self.current_index = 0;
        let length: i32 = self.faces.len() as i32;
        let mut i: i32 = self.ind_face + 1;
        while i <= length {
            if self.find_on_face(i)? {
                return Ok(());
            }
            i += 1;
        }
        if self.current_nb_points == 0 && i > length {
            self.ind_face = i;
        }
        Ok(())
    }

    /// One turn of the face loop of `Find` on face `i`: true when a point IN or ON the
    /// face is found (the face becomes the current one).
    fn find_on_face(&mut self, i: i32) -> Result<bool, Failure> {
        let face: Face = self.faces[i as usize - 1].clone();
        if self.is_out(i, &face)? {
            return Ok(false);
        }
        let surface: Handle<dyn Adaptor3dSurface> =
            Handle::new(BRepAdaptorSurface::with_face(&face, true)?).cast();
        self.intcs.perform(&self.curve_handle(), &surface)?;
        self.current_nb_points = self.intcs.nb_points()?;
        if self.current_nb_points == 0 {
            return Ok(false);
        }
        self.fast_class.initialize_surface(&surface)?;
        self.ind_face = i;
        if self.find_point()? {
            return Ok(true);
        }
        self.current_nb_points = 0;
        Ok(false)
    }

    /// Whether the box of face `i` misses the curve: the line itself for a line, the
    /// curve box for a finite curve, never for an infinite curve that is not a line.
    fn is_out(&mut self, i: i32, face: &Face) -> Result<bool, Failure> {
        let face_box: BoundingBox3 = self.face_box(i, face)?;
        let curve: &GeomAdaptorCurve = self.curve.get().ok_or_else(|| {
            null_object(
                "BRepIntCurveSurface_Inter::Find: no curve (a null handle); expected Init(curve) before Next",
            )
        })?;
        if curve.get_type()? == CurveType::Line {
            return Ok(face_box.is_out_line(curve.line()?));
        }
        Ok(!self.curve_box.is_void() && face_box.is_out_box(&self.curve_box))
    }

    /// The box of face `i`, built on first use (BRepBndLib::Add with the triangulation,
    /// enlarged by the tolerance) and kept until the next `Load`; a void box is rebuilt.
    fn face_box(&mut self, i: i32, face: &Face) -> Result<BoundingBox3, Failure> {
        let length: usize = self.faces.len();
        let tolerance: f64 = self.tolerance;
        let boxes = self
            .face_boxes
            .get_or_insert_with(|| vec![BoundingBox3::default(); length]);
        let face_box: &mut BoundingBox3 = &mut boxes[i as usize - 1];
        if face_box.is_void() {
            BRepBndLib::add(face, face_box, true)?;
            face_box.set_gap(tolerance);
        }
        Ok(face_box.clone())
    }

    /// `FindPoint()`: from the current index (1 when there is none), the first
    /// intersection point that the face classifier finds IN or ON the face becomes the
    /// current point; the state of the last classified point is kept either way.
    pub(super) fn find_point(&mut self) -> Result<bool, Failure> {
        let mut j: i32 = if self.current_index == 0 {
            1
        } else {
            self.current_index
        };
        while j <= self.current_nb_points {
            let point = self.intcs.point(j)?;
            let (u, v): (f64, f64) = (point.u(), point.v());
            self.current_state =
                self.fast_class
                    .classify(Point2::new(u, v), self.tolerance, true)?;
            if self.current_state == State::On || self.current_state == State::In {
                self.current_index = j;
                self.current_u = u;
                self.current_v = v;
                return Ok(true);
            }
            j += 1;
        }
        Ok(false)
    }
}
