// Rust translation of Transformation, Section and ModifiedSection of
// ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_SectionPlacement.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The transformation that carries the section to its place on the path, and the
//! placed section curves. The private `SectionAxis` is called nowhere natively and
//! is not ported.
use super::location_law::LocationD0;
use super::section_placement::GeomFillSectionPlacement;
use super::sweep_failures::{live, sweep_failure};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkmath::gp::{Axis1, Axis3, Direction3, Point3, Transform3, Vector3};

impl GeomFillSectionPlacement {
    /// `Transformation(WithTranslation, WithCorrection)`: from the section frame to
    /// the law frame at `PathParam`, with the section point moved onto the path
    /// and, on correction, the section plane first turned normal to the path;
    /// `let trsf = sp.transformation(true, false)?;`.
    pub fn transformation(
        &self,
        with_translation: bool,
        with_correction: bool,
    ) -> Result<Transform3, Failure> {
        let (paxe, v) = self.path_frame()?;
        let mut psection = Point3::new(0.0, 0.0, 0.0);
        if with_translation || with_correction {
            psection = self.section_point()?;
        }
        let mut rot = Transform3::default();
        if with_correction && !self.my_is_point {
            rot = self.correction(paxe)?;
            psection.transform(rot);
        }
        let p = if with_translation {
            let ps = psection.xyz();
            Point3::new(-ps.x() + v.x(), -ps.y() + v.y(), -ps.z() + v.z())
        } else {
            Point3::new(0.0, 0.0, 0.0)
        };
        let saxe = Axis3::new(p, Direction3::Z, Direction3::X)?;
        let mut tf = Transform3::transformation_between(saxe, paxe);
        if with_correction {
            tf.multiply(rot);
        }
        Ok(tf)
    }

    /// The frame of the law at `PathParam` (its third column as the main direction,
    /// its first as X) and the law translation.
    fn path_frame(&self) -> Result<(Axis3, Vector3), Failure> {
        let mut placement = LocationD0::default();
        self.law("GeomFill_SectionPlacement::Transformation")?
            .d0(self.path_param, &mut placement)?;
        let (m, v) = (placement.m, placement.v);
        let d = Direction3::from_xyz(m.column(3)?)?;
        let dn = Direction3::from_xyz(m.column(1)?)?;
        Ok((Axis3::new(Point3::from(*v.xyz()), d, dn)?, v))
    }

    fn section_point(&self) -> Result<Point3, Failure> {
        if self.my_is_point {
            return Ok(self.my_point);
        }
        live(
            &self.my_section,
            "GeomFill_SectionPlacement::Transformation",
            "mySection",
        )?
        .value(self.sec_param)
    }

    /// The rotation about the section axis location that turns the profile normal
    /// onto the path direction (the identity when they are parallel); a
    /// non-planar profile raises Standard_Failure.
    fn correction(&self, paxe: Axis3) -> Result<Transform3, Failure> {
        let mut rot = Transform3::default();
        if !self.isplan {
            return Err(sweep_failure(
                "Standard_Failure",
                "Illegal usage: can't rotate non-planar profile",
            ));
        }
        let profile_normal = self.the_axe.direction();
        let spine_start_dir = paxe.direction();
        if !profile_normal.is_parallel(spine_start_dir, precision::angular()) {
            let dir_axe_of_rotation = profile_normal.crossed(spine_start_dir)?;
            let angle = profile_normal.angle_with_ref(spine_start_dir, dir_axe_of_rotation);
            let axe_of_rotation = Axis1::new(self.the_axe.location(), dir_axe_of_rotation);
            rot.set_rotation(axe_of_rotation, angle)?;
        }
        Ok(rot)
    }

    /// `Section(WithTranslation)`: a copy of the section curve moved by
    /// `Transformation(WithTranslation, false)`; `let c = sp.section(true)?;`.
    pub fn section(&self, with_translation: bool) -> Result<Handle<dyn GeomCurve>, Failure> {
        self.placed_copy(
            "GeomFill_SectionPlacement::Section",
            with_translation,
            false,
        )
    }

    /// `ModifiedSection(WithTranslation)`: the copy moved by
    /// `Transformation(WithTranslation, true)`; `let c = sp.modified_section(true)?;`.
    pub fn modified_section(
        &self,
        with_translation: bool,
    ) -> Result<Handle<dyn GeomCurve>, Failure> {
        self.placed_copy(
            "GeomFill_SectionPlacement::ModifiedSection",
            with_translation,
            true,
        )
    }

    fn placed_copy(
        &self,
        member: &str,
        with_translation: bool,
        with_correction: bool,
    ) -> Result<Handle<dyn GeomCurve>, Failure> {
        let the_section: Handle<dyn GeomCurve> =
            live(&self.my_section, member, "mySection")?.copy()?.cast();
        let trsf = self.transformation(with_translation, with_correction)?;
        live(&the_section, member, "TheSection")?.transform(trsf)?;
        Ok(the_section)
    }
}
