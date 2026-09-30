// Rust translation of ModelingData/TKBRep/BRep/BRep_Builder.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The face members of `BRep_Builder`.
use super::{BRepBuilder, check_unlocked, check_unlocked_shape, relative_to};
use crate::brep::BRepTFace;
use crate::brep::access::face_tshape;
use crate::topods::{Face, TShape};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkmath::poly::Triangulation;
use rustycad_tkmath::toploc::Location;

impl BRepBuilder {
    /// `MakeFace(F)`: an undefined face (no surface); the inline native member makes no
    /// lock check; `builder.make_face(&mut face)`.
    pub fn make_face(&self, face: &mut Face) {
        self.make_shape(face, &Handle::new(BRepTFace::new()).into_cast());
    }

    /// `MakeFace(F, S, Tol)`: the TFace keeps its identity location, which is what the
    /// located overload stores for the identity; `builder.make_face_on_surface(&mut face, &surface, 1e-7)?`.
    pub fn make_face_on_surface(
        &self,
        face: &mut Face,
        surface: &Handle<dyn GeomSurface>,
        tolerance: f64,
    ) -> Result<(), Failure> {
        self.make_face_on_located_surface(face, surface, &Location::default(), tolerance)
    }

    /// `MakeFace(F, S, L, Tol)`;
    /// `builder.make_face_on_located_surface(&mut face, &surface, &location, 1e-7)?`.
    pub fn make_face_on_located_surface(
        &self,
        face: &mut Face,
        surface: &Handle<dyn GeomSurface>,
        location: &Location,
        tolerance: f64,
    ) -> Result<(), Failure> {
        let tface = BRepTFace::new();
        check_unlocked_shape(face, "BRep_Builder::MakeFace")?;
        tface.set_surface(surface);
        tface.set_tolerance(tolerance);
        tface.set_location(location);
        self.make_shape(face, &Handle::new(tface).into_cast());
        Ok(())
    }

    /// `MakeFace(F, T)`: a face with one active triangulation;
    /// `builder.make_face_with_triangulation(&mut face, &triangulation)?`.
    pub fn make_face_with_triangulation(
        &self,
        face: &mut Face,
        triangulation: &Handle<Triangulation>,
    ) -> Result<(), Failure> {
        let tface = BRepTFace::new();
        check_unlocked_shape(face, "BRep_Builder::MakeFace")?;
        tface.set_triangulation(triangulation, true);
        self.make_shape(face, &Handle::new(tface).into_cast());
        Ok(())
    }

    /// `MakeFace(F, Triangulations, Active)`; a null active triangulation activates the
    /// first one; `builder.make_face_with_triangulations(&mut face, &list, &Handle::default())?`.
    pub fn make_face_with_triangulations(
        &self,
        face: &mut Face,
        triangulations: &[Handle<Triangulation>],
        active: &Handle<Triangulation>,
    ) -> Result<(), Failure> {
        let tface = BRepTFace::new();
        check_unlocked_shape(face, "BRep_Builder::MakeFace")?;
        tface.set_triangulations(triangulations, active)?;
        self.make_shape(face, &Handle::new(tface).into_cast());
        Ok(())
    }

    /// `UpdateFace(F, S, L, Tol)`: the surface, the tolerance and the location relative
    /// to the face; `builder.update_face(&face, &surface, &location, 1e-7)?`.
    pub fn update_face(
        &self,
        face: &Face,
        surface: &Handle<dyn GeomSurface>,
        location: &Location,
        tolerance: f64,
    ) -> Result<(), Failure> {
        let tface = face_tshape(face, "BRep_Builder::UpdateFace")?;
        check_unlocked(tface, "BRep_Builder::UpdateFace")?;
        tface.set_surface(surface);
        tface.set_tolerance(tolerance);
        tface.set_location(&relative_to(location, face)?);
        tface.set_modified(true);
        Ok(())
    }

    /// `UpdateFace(F, T, theToReset)`: changes the triangulations (a null one removes
    /// them); `builder.update_face_triangulation(&face, &triangulation, true)?`.
    pub fn update_face_triangulation(
        &self,
        face: &Face,
        triangulation: &Handle<Triangulation>,
        to_reset: bool,
    ) -> Result<(), Failure> {
        let tface = face_tshape(face, "BRep_Builder::UpdateFace")?;
        check_unlocked(tface, "BRep_Builder::UpdateFace")?;
        tface.set_triangulation(triangulation, to_reset);
        tface.set_modified(true);
        Ok(())
    }

    /// `UpdateFace(F, Tol)`: sets the tolerance (it can decrease);
    /// `builder.update_face_tolerance(&face, 1e-7)?`.
    pub fn update_face_tolerance(&self, face: &Face, tolerance: f64) -> Result<(), Failure> {
        let tface = face_tshape(face, "BRep_Builder::UpdateFace")?;
        check_unlocked(tface, "BRep_Builder::UpdateFace")?;
        tface.set_tolerance(tolerance);
        tface.set_modified(true);
        Ok(())
    }

    /// `NaturalRestriction(F, N)`; `builder.natural_restriction(&face, true)?`.
    pub fn natural_restriction(&self, face: &Face, restriction: bool) -> Result<(), Failure> {
        let tface = face_tshape(face, "BRep_Builder::NaturalRestriction")?;
        check_unlocked(tface, "BRep_Builder::NaturalRestriction")?;
        tface.set_natural_restriction(restriction);
        tface.set_modified(true);
        Ok(())
    }
}
