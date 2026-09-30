// Rust translation of IntAna_QuadQuadGeo.hxx/.lxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! IntAna_QuadQuadGeo: the geometric intersection of two natural quadrics
//! (plane, cylinder, sphere, cone, torus). The state, the constructors and the
//! tolerances are here; the results are built in `results`, each family of
//! Perform overloads in its own module.
mod axe_operator;
mod cones;
mod cylinders;
mod estim_dist;
mod plane_cone;
mod planes;
mod results;
mod spheres;
mod tori;

use super::ResultType;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Cone3, Cylinder3, Direction3, Plane3, Point3, Sphere3, Torus3};

/// IntAna_QuadQuadGeo: the geometric intersection of two quadrics: points,
/// lines, circles, an ellipse, a parabola, hyperbolas, empty, same, or no
/// geometric solution (IntAna_IntQuadQuad must compute it). A new intersection
/// is empty; each Perform keeps the fields it does not assign, as natively.
///
/// `let inter = QuadQuadGeo::planes(p1, p2, 1e-12, 1e-7)?; let line = inter.line(1)?;`
#[derive(Clone, Copy, Debug)]
pub struct QuadQuadGeo {
    done: bool,
    nbint: i32,
    typeres: ResultType,
    pt1: Point3,
    pt2: Point3,
    pt3: Point3,
    pt4: Point3,
    dir1: Direction3,
    dir2: Direction3,
    dir3: Direction3,
    dir4: Direction3,
    param1: f64,
    param2: f64,
    param3: f64,
    param4: f64,
    param1bis: f64,
    param2bis: f64,
    my_epsilon_distance: f64,
    my_epsilon_angle_cone: f64,
    my_epsilon_mini_circle_radius: f64,
    my_epsilon_cylinder_delta_radius: f64,
    my_epsilon_cylinder_delta_distance: f64,
    my_epsilon_axes_para: f64,
    my_common_gen: bool,
    my_pchar: Point3,
}

impl Default for QuadQuadGeo {
    /// IntAna_QuadQuadGeo(): not done, empty, null points and parameters, default
    /// directions, the native inner tolerances.
    fn default() -> Self {
        let mut inter = Self {
            done: false,
            nbint: 0,
            typeres: ResultType::Empty,
            pt1: Point3::new(0.0, 0.0, 0.0),
            pt2: Point3::new(0.0, 0.0, 0.0),
            pt3: Point3::new(0.0, 0.0, 0.0),
            pt4: Point3::new(0.0, 0.0, 0.0),
            dir1: Direction3::default(),
            dir2: Direction3::default(),
            dir3: Direction3::default(),
            dir4: Direction3::default(),
            param1: 0.0,
            param2: 0.0,
            param3: 0.0,
            param4: 0.0,
            param1bis: 0.0,
            param2bis: 0.0,
            my_epsilon_distance: 0.0,
            my_epsilon_angle_cone: 0.0,
            my_epsilon_mini_circle_radius: 0.0,
            my_epsilon_cylinder_delta_radius: 0.0,
            my_epsilon_cylinder_delta_distance: 0.0,
            my_epsilon_axes_para: 0.0,
            my_common_gen: false,
            my_pchar: Point3::new(0.0, 0.0, 0.0),
        };
        inter.init_tolerances();
        inter
    }
}

/// Builds an intersection with one `perform_*` member.
macro_rules! quad_quad_constructor {
    ($(#[$doc:meta])* $name:ident => $perform:ident($($arg:ident: $ty:ty),*)) => {
        $(#[$doc])*
        pub fn $name($($arg: $ty),*) -> Result<Self, Failure> {
            let mut inter: Self = Self::default();
            inter.$perform($($arg),*)?;
            Ok(inter)
        }
    };
}

impl QuadQuadGeo {
    /// IntAna_QuadQuadGeo(): the empty constructor. `QuadQuadGeo::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// InitTolerances (protected natively): 1e-14 for distances, the angular
    /// precision for cone angles and parallel axes, 0.01 Precision::Confusion() for
    /// the smallest circle, 1e-13 for cylinder radii, Precision::Confusion() for
    /// cylinder axes.
    fn init_tolerances(&mut self) {
        self.my_epsilon_distance = 1.0e-14;
        self.my_epsilon_angle_cone = precision::angular();
        self.my_epsilon_mini_circle_radius = 0.01 * precision::confusion();
        self.my_epsilon_cylinder_delta_radius = 1.0e-13;
        self.my_epsilon_cylinder_delta_distance = precision::confusion();
        self.my_epsilon_axes_para = precision::angular();
    }

    quad_quad_constructor!(
        /// IntAna_QuadQuadGeo(gp_Pln, gp_Pln, TolAng, Tol). `QuadQuadGeo::planes(p1, p2, 1e-12, 1e-7)?`.
        planes => perform_planes(first: Plane3, second: Plane3, tol_ang: f64, tol: f64));
    quad_quad_constructor!(
        /// IntAna_QuadQuadGeo(gp_Pln, gp_Cylinder, Tolang, Tol, H) (native default H = 0).
        /// `QuadQuadGeo::plane_cylinder(p, c, 1e-12, 1e-7, 0.0)?`.
        plane_cylinder => perform_plane_cylinder(plane: Plane3, cylinder: Cylinder3, tolang: f64, tol: f64, h: f64));
    quad_quad_constructor!(
        /// IntAna_QuadQuadGeo(gp_Pln, gp_Sphere). `QuadQuadGeo::plane_sphere(p, s)?`.
        plane_sphere => perform_plane_sphere(plane: Plane3, sphere: Sphere3));
    quad_quad_constructor!(
        /// IntAna_QuadQuadGeo(gp_Pln, gp_Cone, Tolang, Tol). `QuadQuadGeo::plane_cone(p, c, 1e-12, 1e-7)?`.
        plane_cone => perform_plane_cone(plane: Plane3, cone: Cone3, tolang: f64, tol: f64));
    quad_quad_constructor!(
        /// IntAna_QuadQuadGeo(gp_Cylinder, gp_Cylinder, Tol). `QuadQuadGeo::cylinders(c1, c2, 1e-7)?`.
        cylinders => perform_cylinders(first: Cylinder3, second: Cylinder3, tol: f64));
    quad_quad_constructor!(
        /// IntAna_QuadQuadGeo(gp_Cylinder, gp_Sphere, Tol). `QuadQuadGeo::cylinder_sphere(c, s, 1e-7)?`.
        cylinder_sphere => perform_cylinder_sphere(cylinder: Cylinder3, sphere: Sphere3, tol: f64));
    quad_quad_constructor!(
        /// IntAna_QuadQuadGeo(gp_Cylinder, gp_Cone, Tol). `QuadQuadGeo::cylinder_cone(c, k, 1e-7)?`.
        cylinder_cone => perform_cylinder_cone(cylinder: Cylinder3, cone: Cone3, tol: f64));
    quad_quad_constructor!(
        /// IntAna_QuadQuadGeo(gp_Sphere, gp_Sphere, Tol). `QuadQuadGeo::spheres(s1, s2, 1e-7)?`.
        spheres => perform_spheres(first: Sphere3, second: Sphere3, tol: f64));
    quad_quad_constructor!(
        /// IntAna_QuadQuadGeo(gp_Sphere, gp_Cone, Tol). `QuadQuadGeo::sphere_cone(s, c, 1e-7)?`.
        sphere_cone => perform_sphere_cone(sphere: Sphere3, cone: Cone3, tol: f64));
    quad_quad_constructor!(
        /// IntAna_QuadQuadGeo(gp_Cone, gp_Cone, Tol). `QuadQuadGeo::cones(c1, c2, 1e-7)?`.
        cones => perform_cones(first: Cone3, second: Cone3, tol: f64));
    quad_quad_constructor!(
        /// IntAna_QuadQuadGeo(gp_Pln, gp_Torus, Tol). `QuadQuadGeo::plane_torus(p, t, 1e-7)?`.
        plane_torus => perform_plane_torus(plane: Plane3, torus: Torus3, tol: f64));
    quad_quad_constructor!(
        /// IntAna_QuadQuadGeo(gp_Cylinder, gp_Torus, Tol). `QuadQuadGeo::cylinder_torus(c, t, 1e-7)?`.
        cylinder_torus => perform_cylinder_torus(cylinder: Cylinder3, torus: Torus3, tol: f64));
    quad_quad_constructor!(
        /// IntAna_QuadQuadGeo(gp_Cone, gp_Torus, Tol). `QuadQuadGeo::cone_torus(c, t, 1e-7)?`.
        cone_torus => perform_cone_torus(cone: Cone3, torus: Torus3, tol: f64));
    quad_quad_constructor!(
        /// IntAna_QuadQuadGeo(gp_Sphere, gp_Torus, Tol). `QuadQuadGeo::sphere_torus(s, t, 1e-7)?`.
        sphere_torus => perform_sphere_torus(sphere: Sphere3, torus: Torus3, tol: f64));
    quad_quad_constructor!(
        /// IntAna_QuadQuadGeo(gp_Torus, gp_Torus, Tol). `QuadQuadGeo::tori(t1, t2, 1e-7)?`.
        tori => perform_tori(first: Torus3, second: Torus3, tol: f64));

    /// IsDone(): true when the computation succeeded. `inter.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }

    /// TypeInter(): the kind of result; StdFail_NotDone when not done.
    /// `inter.type_inter()?`.
    pub fn type_inter(&self) -> Result<ResultType, Failure> {
        self.check_done("TypeInter")?;
        Ok(self.typeres)
    }

    /// NbSolutions(): the number of solutions; StdFail_NotDone when not done.
    /// `inter.nb_solutions()?`.
    pub fn nb_solutions(&self) -> Result<i32, Failure> {
        self.check_done("NbSolutions")?;
        Ok(self.nbint)
    }

    /// HasCommonGen(): the two cones of the last Perform share a generatrix.
    /// `inter.has_common_gen()`.
    pub fn has_common_gen(&self) -> bool {
        self.my_common_gen
    }

    /// PChar(): the characteristic point of the intersection of two cones with a
    /// common generatrix. `inter.p_char()`.
    pub fn p_char(&self) -> Point3 {
        self.my_pchar
    }

    fn check_done(&self, member: &str) -> Result<(), Failure> {
        if self.done {
            return Ok(());
        }
        Err(Failure::with_type(
            "StdFail_NotDone",
            &format!(
                "IntAna_QuadQuadGeo::{member}() - the intersection is not done; expected IsDone()"
            ),
            "",
        ))
    }
}

/// The Standard_DomainError of a result accessor.
fn domain_error(member: &str, detail: &str) -> Failure {
    Failure::with_type(
        "Standard_DomainError",
        &format!("IntAna_QuadQuadGeo::{member}() - {detail}"),
        "",
    )
}
