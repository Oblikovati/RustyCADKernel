// Rust translation of BRepFill_PipeShell::Add, SetLaw and DeleteProfile
// (ModelingAlgorithms/TKBool/BRepFill/BRepFill_PipeShell.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The profiles of a pipe shell, with their homothety law (given or computed from the
//! width between the spine and the guide).
use super::{BRepFillPipeShell, null};
use crate::brep_fill::section::BRepFillSection;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{self, Shape, Vertex};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkgeomalgo::geom_fill::{GeomFillLocationGuide, GeomFillLocationLaw as _};
use rustycad_tkgeomalgo::int_curve_surface::{HCurve, HInter};
use rustycad_tkgeomalgo::law::{LawFunction, LawInterpol};
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::gp::{Point2, Point3};
use rustycad_tktopalgo::brep_gprop::BRepGProp;
use rustycad_tktopalgo::brep_lib::BRepLibMakeFace;

impl BRepFillPipeShell {
    /// `Add(Profile, WithContact, WithCorrection)` (natively defaulted to false): the
    /// profile placed at its own position on the spine;
    /// `pipe.add(&profile, false, false)?`.
    pub fn add(
        &mut self,
        profile: &Shape,
        with_contact: bool,
        with_correction: bool,
    ) -> Result<(), Failure> {
        self.add_at(profile, &Vertex::default(), with_contact, with_correction)?;
        self.reset_loc()
    }

    /// `Add(Profile, Location, WithContact, WithCorrection)`: the profile placed at a
    /// vertex of the spine (replacing the same profile); with the automatic law it is
    /// the only profile and its homothety follows the width between spine and guide;
    /// `pipe.add_at(&profile, &vertex, false, false)?`.
    pub fn add_at(
        &mut self,
        profile: &Shape,
        location: &Vertex,
        with_contact: bool,
        with_correction: bool,
    ) -> Result<(), Failure> {
        self.delete_profile(profile)?;
        let mut s =
            BRepFillSection::with_profile(profile, location, with_contact, with_correction)?;
        if self.my_is_automatic_law {
            self.my_seq.clear();
            s.set(true);
            self.my_seq.push(s);
            self.my_section = Handle::default();
            self.reset_loc()?;
            return self.automatic_law(profile);
        }
        self.my_seq.push(s);
        self.my_section = Handle::default();
        self.reset_loc()
    }

    /// The homothety law of the automatic mode: the spine-guide width along the spine,
    /// divided by the width at the profile plane (between the intersections of the
    /// spine and the guide with it nearest to the profile barycentre).
    fn automatic_law(&mut self, profile: &Shape) -> Result<(), Failure> {
        let law = self.location()?.law(1)?;
        let loc: Handle<GeomFillLocationGuide> = law.cast();
        let loc = loc.get().ok_or_else(|| null("guide location law"))?;
        let (_, mut par_and_rad) = loc.compute_automatic_law()?;
        let mut global_props = GProps::new();
        BRepGProp::linear_properties(profile, &mut global_props, false, false)?;
        let bary_center: Point3 = global_props.centre_of_mass();
        let profile_face = BRepLibMakeFace::from_wire(&topods::wire(profile)?, true)?.face()?;
        let the_plane = BRepTool::surface(&profile_face)?;
        let ga_hplane: Adaptor3dSurfaceHandle =
            Handle::new(GeomAdaptorSurface::with_surface(&the_plane)?).cast();
        let curves: [HCurve; 2] = [loc.get_curve(), loc.guide()];
        let mut points_on_spines: [Point3; 2] = [Point3::default(); 2];
        for (curve, point) in curves.iter().zip(points_on_spines.iter_mut()) {
            *point = nearest_intersection(curve, &ga_hplane, bary_center, *point)?;
        }
        let initial_width: f64 = points_on_spines[0].distance(&points_on_spines[1]);
        scale_widths(&mut par_and_rad, initial_width)?;
        let nb: i32 = par_and_rad.upper();
        let (first, last) = (value_at(&par_and_rad, 1)?, value_at(&par_and_rad, nb)?);
        let is_periodic: bool = (first.y() - last.y()).abs() < precision::confusion();
        let interpol = LawInterpol::new();
        interpol.set(&par_and_rad, is_periodic)?;
        self.my_law = Handle::new(interpol).cast::<dyn LawFunction>();
        Ok(())
    }

    /// `SetLaw(Profile, L, WithContact, WithCorrection)`: the only profile, scaled by
    /// the law; `pipe.set_law(&profile, &law, false, false)?`.
    pub fn set_law(
        &mut self,
        profile: &Shape,
        l: &Handle<dyn LawFunction>,
        with_contact: bool,
        with_correction: bool,
    ) -> Result<(), Failure> {
        self.set_law_at(
            profile,
            l,
            &Vertex::default(),
            with_contact,
            with_correction,
        )?;
        self.reset_loc()
    }

    /// `SetLaw(Profile, L, Location, WithContact, WithCorrection)`: the only profile,
    /// placed at a vertex of the spine and scaled by the law;
    /// `pipe.set_law_at(&profile, &law, &vertex, false, false)?`.
    pub fn set_law_at(
        &mut self,
        profile: &Shape,
        l: &Handle<dyn LawFunction>,
        location: &Vertex,
        with_contact: bool,
        with_correction: bool,
    ) -> Result<(), Failure> {
        self.my_seq.clear();
        let mut s =
            BRepFillSection::with_profile(profile, location, with_contact, with_correction)?;
        s.set(true);
        self.my_seq.push(s);
        self.my_law = l.clone();
        self.my_section = Handle::default();
        self.reset_loc()
    }

    /// `DeleteProfile(Profile)`: the first section of this profile removed (the
    /// sections are then relocated); `pipe.delete_profile(&profile)?`.
    pub fn delete_profile(&mut self, profile: &Shape) -> Result<(), Failure> {
        if let Some(ii) = self
            .my_seq
            .iter()
            .position(|s| profile.is_same(s.original_shape()))
        {
            self.my_seq.remove(ii);
            self.my_section = Handle::default();
        }
        self.reset_loc()
    }
}

/// The intersection of the curve with the plane nearest to the barycentre (`point`
/// unchanged when there is none).
fn nearest_intersection(
    curve: &HCurve,
    plane: &Adaptor3dSurfaceHandle,
    bary_center: Point3,
    point: Point3,
) -> Result<Point3, Failure> {
    let mut intersector = HInter::default();
    intersector.perform(curve, plane)?;
    let mut min_dist: f64 = f64::MAX;
    let mut nearest: Point3 = point;
    for j in 1..=intersector.nb_points()? {
        let a_pint: Point3 = intersector.point(j)?.pnt();
        let a_dist: f64 = bary_center.distance(&a_pint);
        if a_dist < min_dist {
            min_dist = a_dist;
            nearest = a_pint;
        }
    }
    Ok(nearest)
}

/// The widths of the law divided by the initial width.
fn scale_widths(
    par_and_rad: &mut Array1<'static, Point2>,
    initial_width: f64,
) -> Result<(), Failure> {
    for i in 1..=par_and_rad.upper() {
        let p = par_and_rad
            .change_value(i)
            .map_err(crate::brep_fill::shape_table::table_error)?;
        *p = Point2::new(p.x(), p.y() / initial_width);
    }
    Ok(())
}

/// The point `i` of the law.
fn value_at(par_and_rad: &Array1<'static, Point2>, i: i32) -> Result<Point2, Failure> {
    par_and_rad
        .value(i)
        .copied()
        .map_err(crate::brep_fill::shape_table::table_error)
}
