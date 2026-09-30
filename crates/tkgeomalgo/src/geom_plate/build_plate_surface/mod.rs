// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomPlate/GeomPlate_BuildPlateSurface.hxx/.cxx
// (the class, its constructors and the constraint members) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomPlate_BuildPlateSurface`: the plate surface that meets curve and point
//! constraints. The private members are in the submodules (docs/geomalgo-design.md
//! section 19).
mod contour;
mod courbe_jointive;
mod discretise;
mod errors;
mod intersect;
mod load;
mod perform;
mod projection;
mod results;
mod surf_init;
mod surf_init_normals;
mod verif;

use super::curve_constraint::GeomPlateCurveConstraint;
use super::live;
use super::point_constraint::GeomPlatePointConstraint;
use super::surface::GeomPlateSurface;
use crate::plate::PlatePlate;
use rustycad_tkernel::ncollection::HArray1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::errors as geom_errors;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkgeombase::extrema::ExtPS;

/// The construction of a plate surface that conforms to curve and point constraints
/// (`GeomPlate_BuildPlateSurface`): the curves are projected on an initial surface
/// (given, or an average plane, or an approximated first plate), discretised, and a
/// plate deforming the initial surface is solved; the number of points on each curve
/// grows until the constraints are met within their tolerances or the iterations end.
///
/// ```ignore
/// let mut builder = GeomPlateBuildPlateSurface::new(3, 10, 3, 1e-5, 1e-4, 0.01, 0.1, false)?;
/// builder.add_point(&Handle::new(GeomPlatePointConstraint::with_point(p, 0, 1e-4)?));
/// builder.perform(&ProgressRange::default())?;
/// let surface = builder.surface();
/// ```
pub struct GeomPlateBuildPlateSurface {
    pub(super) lin_cont: Vec<Handle<GeomPlateCurveConstraint>>,
    pub(super) par_cont: Option<Vec<Vec<f64>>>,
    pub(super) plate_cont: Option<Vec<Vec<f64>>>,
    pub(super) pnt_cont: Vec<Handle<GeomPlatePointConstraint>>,
    pub(super) surf_init: Handle<dyn GeomSurface>,
    pub(super) planar_surf_init: Handle<dyn GeomSurface>,
    pub(super) geom_plate_surface: Handle<GeomPlateSurface>,
    pub(super) plate: PlatePlate,
    pub(super) prev_plate: PlatePlate,
    pub(super) anisotropie: bool,
    pub(super) sense: Option<Vec<i32>>,
    pub(super) degree: i32,
    pub(super) init_order: Option<Vec<i32>>,
    pub(super) g0_error: f64,
    pub(super) g1_error: f64,
    pub(super) g2_error: f64,
    pub(super) nb_pts_on_cur: i32,
    pub(super) surf_init_is_give: bool,
    pub(super) nb_iter: i32,
    pub(super) proj: ExtPS<'static>,
    pub(super) tol2d: f64,
    pub(super) tol3d: f64,
    pub(super) tol_ang: f64,
    pub(super) tol_u: f64,
    pub(super) tol_v: f64,
    pub(super) nb_bounds: i32,
    pub(super) is_linear: bool,
    pub(super) free: bool,
}

/// The value `index` (absolute) of a native `NCollection_HArray1`, Standard_OutOfRange
/// outside it.
fn harray_value<T: Clone + Send + Sync + 'static>(
    array: &HArray1<T>,
    index: i32,
) -> Result<T, Failure> {
    array
        .array1()
        .value(index)
        .cloned()
        .map_err(geom_errors::array_bounds)
}

/// The Standard_ConstructionError of the constructors.
fn construction(message: &str) -> Failure {
    geom_errors::construction(message)
}

/// Standard_OutOfRange for the index `index` of a sequence of `length` elements.
pub(super) fn sequence_index(index: i32, length: usize, member: &str) -> Result<usize, Failure> {
    if index < 1 || index as usize > length {
        return Err(geom_errors::out_of_range(&format!(
            "GeomPlate_BuildPlateSurface::{member}: index {index} outside 1..={length}"
        )));
    }
    Ok(index as usize - 1)
}

impl GeomPlateBuildPlateSurface {
    /// The fields shared by the three constructors.
    #[allow(clippy::too_many_arguments)] // The native tolerance list.
    fn with_fields(
        degree: i32,
        nb_pts_on_cur: i32,
        nb_iter: i32,
        tol2d: f64,
        tol3d: f64,
        tol_ang: f64,
        anisotropie: bool,
    ) -> Self {
        Self {
            lin_cont: Vec::new(),
            par_cont: None,
            plate_cont: None,
            pnt_cont: Vec::new(),
            surf_init: Handle::default(),
            planar_surf_init: Handle::default(),
            geom_plate_surface: Handle::default(),
            plate: PlatePlate::new(),
            prev_plate: PlatePlate::new(),
            anisotropie,
            sense: None,
            degree,
            init_order: None,
            g0_error: 0.0,
            g1_error: 0.0,
            g2_error: 0.0,
            nb_pts_on_cur,
            surf_init_is_give: false,
            nb_iter,
            proj: ExtPS::new(),
            tol2d,
            tol3d,
            tol_ang,
            tol_u: 0.0,
            tol_v: 0.0,
            nb_bounds: 0,
            is_linear: true,
            free: false,
        }
    }

    /// The Standard_ConstructionError checks of the iteration count and the degree.
    fn check_settings(&self, degree_message: &str) -> Result<(), Failure> {
        if self.nb_iter < 1 {
            return Err(construction(&format!(
                "GeomPlate :  Number of iteration must be >= 1; got {}",
                self.nb_iter
            )));
        }
        if self.degree < 2 {
            return Err(construction(&format!(
                "{degree_message}; got {}",
                self.degree
            )));
        }
        Ok(())
    }

    /// The constructor of the old interface: one constraint of order `tang(i)` with
    /// `n_points(i)` points on each curve of `tab_curve`, no initial surface
    /// (`GeomPlate_BuildPlateSurface(NPoints, TabCurve, Tang, Degree, NbIter, Tol2d,
    /// Tol3d, TolAng, TolCurv, Anisotropie)`; the native defaults are 3, 1e-5, 1e-4,
    /// 0.01, 0.1 and false; `TolCurv` is unused). Standard_ConstructionError for no
    /// iteration, no curve, no order, no point or a degree below 2.
    /// `GeomPlateBuildPlateSurface::with_curves(&np, &curves, &tang, 3, 3, 1e-5, 1e-4, 0.01, 0.1, false)?`.
    #[allow(clippy::too_many_arguments)] // The native ten-argument constructor.
    pub fn with_curves(
        n_points: &Handle<HArray1<i32>>,
        tab_curve: &Handle<HArray1<Handle<dyn Adaptor3dCurve>>>,
        tang: &Handle<HArray1<i32>>,
        degree: i32,
        nb_iter: i32,
        tol2d: f64,
        tol3d: f64,
        tol_ang: f64,
        _tol_curv: f64,
        anisotropie: bool,
    ) -> Result<Self, Failure> {
        let mut builder = Self::with_fields(degree, 0, nb_iter, tol2d, tol3d, tol_ang, anisotropie);
        let (n_points, tab_curve) = (live(n_points, "NPoints")?, live(tab_curve, "TabCurve")?);
        let tang = live(tang, "Tang")?;
        let nt_curve: i32 = tab_curve.array1().length();
        if nb_iter < 1 {
            return Err(construction(&format!(
                "GeomPlate :  Number of iteration must be >= 1; got {nb_iter}"
            )));
        }
        if nt_curve == 0 {
            return Err(construction("GeomPlate : the bounds Array is null"));
        }
        if tang.array1().length() == 0 {
            return Err(construction("GeomPlate : the constraints Array is null"));
        }
        let mut nbp: i32 = 0;
        for i in 1..=nt_curve {
            nbp += harray_value(n_points, i)?;
        }
        if nbp == 0 {
            return Err(construction(
                "GeomPlate : the resolution is impossible if the number of constraints points is 0",
            ));
        }
        builder.check_settings("GeomPlate ; the degree resolution must be upper of 2")?;
        for i in 1..=nt_curve {
            let (curve, order) = (harray_value(tab_curve, i)?, harray_value(tang, i)?);
            let npt: i32 = harray_value(n_points, i)?;
            let cont = GeomPlateCurveConstraint::new(&curve, order, npt, 0.0001, 0.01, 0.1)?;
            builder.lin_cont.push(Handle::new(cont));
        }
        Ok(builder)
    }

    /// A builder on the initial surface `surf` (`GeomPlate_BuildPlateSurface(Surf,
    /// Degree, NbPtsOnCur, NbIter, Tol2d, Tol3d, TolAng, TolCurv, Anisotropie)`; the
    /// native defaults are 3, 10, 3, 1e-5, 1e-4, 0.01, 0.1 and false; `TolCurv` is
    /// unused). Standard_ConstructionError for no iteration or a degree below 2.
    /// `GeomPlateBuildPlateSurface::with_surface(&s, 3, 10, 3, 1e-5, 1e-4, 0.01, 0.1, false)?`.
    #[allow(clippy::too_many_arguments)] // The native nine-argument constructor.
    pub fn with_surface(
        surf: &Handle<dyn GeomSurface>,
        degree: i32,
        nb_pts_on_cur: i32,
        nb_iter: i32,
        tol2d: f64,
        tol3d: f64,
        tol_ang: f64,
        _tol_curv: f64,
        anisotropie: bool,
    ) -> Result<Self, Failure> {
        let mut builder = Self::with_fields(
            degree,
            nb_pts_on_cur,
            nb_iter,
            tol2d,
            tol3d,
            tol_ang,
            anisotropie,
        );
        builder.surf_init = surf.clone();
        builder.check_settings("GeomPlate : the degree must be above 2")?;
        builder.surf_init_is_give = true;
        Ok(builder)
    }

    /// A builder without initial surface (`GeomPlate_BuildPlateSurface(Degree,
    /// NbPtsOnCur, NbIter, Tol2d, Tol3d, TolAng, TolCurv, Anisotropie)`; the native
    /// defaults are 3, 10, 3, 1e-5, 1e-4, 0.01, 0.1 and false; `TolCurv` is unused).
    /// Standard_ConstructionError for no iteration or a degree below 2.
    /// `GeomPlateBuildPlateSurface::new(3, 10, 3, 1e-5, 1e-4, 0.01, 0.1, false)?`.
    #[allow(clippy::too_many_arguments)] // The native eight-argument constructor.
    pub fn new(
        degree: i32,
        nb_pts_on_cur: i32,
        nb_iter: i32,
        tol2d: f64,
        tol3d: f64,
        tol_ang: f64,
        _tol_curv: f64,
        anisotropie: bool,
    ) -> Result<Self, Failure> {
        let builder = Self::with_fields(
            degree,
            nb_pts_on_cur,
            nb_iter,
            tol2d,
            tol3d,
            tol_ang,
            anisotropie,
        );
        builder.check_settings("GeomPlate : the degree resolution must be upper of 2")?;
        Ok(builder)
    }

    /// Removes every curve and point constraint (`Init`); `builder.init()`.
    pub fn init(&mut self) {
        self.lin_cont.clear();
        self.pnt_cont.clear();
    }

    /// Loads the initial surface (`LoadInitSurface`); `builder.load_init_surface(&s)`.
    pub fn load_init_surface(&mut self, surf: &Handle<dyn GeomSurface>) {
        self.surf_init = surf.clone();
        self.surf_init_is_give = true;
    }

    /// Adds a curve constraint (`Add(GeomPlate_CurveConstraint)`);
    /// `builder.add_curve(&constraint)`.
    pub fn add_curve(&mut self, cont: &Handle<GeomPlateCurveConstraint>) {
        self.lin_cont.push(cont.clone());
    }

    /// The number of curves that bound the plate, all of them when 0
    /// (`SetNbBounds`); `builder.set_nb_bounds(4)`.
    pub fn set_nb_bounds(&mut self, nb_bounds: i32) {
        self.nb_bounds = nb_bounds;
    }

    /// Adds a point constraint (`Add(GeomPlate_PointConstraint)`);
    /// `builder.add_point(&constraint)`.
    pub fn add_point(&mut self, cont: &Handle<GeomPlatePointConstraint>) {
        self.pnt_cont.push(cont.clone());
    }

    /// The curve constraint `order` (1-based, in the order `Perform` leaves them)
    /// (`CurveConstraint`); Standard_OutOfRange outside the constraints.
    /// `builder.curve_constraint(1)?`.
    pub fn curve_constraint(
        &self,
        order: i32,
    ) -> Result<Handle<GeomPlateCurveConstraint>, Failure> {
        let index: usize = sequence_index(order, self.lin_cont.len(), "CurveConstraint")?;
        Ok(self.lin_cont[index].clone())
    }

    /// The point constraint `order` (1-based) (`PointConstraint`);
    /// Standard_OutOfRange outside the constraints. `builder.point_constraint(1)?`.
    pub fn point_constraint(
        &self,
        order: i32,
    ) -> Result<Handle<GeomPlatePointConstraint>, Failure> {
        let index: usize = sequence_index(order, self.pnt_cont.len(), "PointConstraint")?;
        Ok(self.pnt_cont[index].clone())
    }

    /// `myLinCont->Value(i)`, 1-based.
    pub(super) fn lin(&self, i: i32) -> Result<&GeomPlateCurveConstraint, Failure> {
        let index: usize = sequence_index(i, self.lin_cont.len(), "LinCont")?;
        live(&self.lin_cont[index], "a curve constraint")
    }

    /// `myPntCont->Value(i)`, 1-based.
    pub(super) fn pnt(&self, i: i32) -> Result<&GeomPlatePointConstraint, Failure> {
        let index: usize = sequence_index(i, self.pnt_cont.len(), "PntCont")?;
        live(&self.pnt_cont[index], "a point constraint")
    }

    /// The number of curve constraints (`myLinCont->Length()`).
    pub(super) fn nt_lin_cont(&self) -> i32 {
        self.lin_cont.len() as i32
    }

    /// The number of point constraints (`myPntCont->Length()`).
    pub(super) fn nt_pnt_cont(&self) -> i32 {
        self.pnt_cont.len() as i32
    }

    /// The initial surface, Standard_NullObject when there is none.
    pub(super) fn surf_init_ref(&self) -> Result<&dyn GeomSurface, Failure> {
        live(&self.surf_init, "the initial surface")
    }
}
