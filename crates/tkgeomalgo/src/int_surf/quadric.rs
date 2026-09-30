// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntSurf/IntSurf_Quadric.hxx/.lxx/.cxx
// (construction, accessors and parametric evaluation) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntSurf_Quadric`: a plane, cylinder, sphere, cone or torus seen as an implicit
//! function. The implicit members (distance, gradient, normal) are in `quadric_metric`.
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::elslib;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{
    Axis3, Cone3, Cylinder3, Line3, Plane3, Point3, Sphere3, Torus3, Vector3,
};

/// An elementary surface as an implicit function (`IntSurf_Quadric`).
///
/// `prm1..prm4` hold the native parameters: the plane equation coefficients, the radius
/// (cylinder, sphere), the reference radius, semi-angle and its cosine (cone), or the
/// major and minor radii (torus). `lin` is the axis of the surface.
/// `IntSurfQuadric::from_sphere(&sphere)?.distance(point)`.
#[derive(Clone, Copy, Debug)]
pub struct IntSurfQuadric {
    pub(super) ax3: Axis3,
    pub(super) lin: Line3,
    pub(super) typ: SurfaceType,
    pub(super) prm1: f64,
    pub(super) prm2: f64,
    pub(super) prm3: f64,
    pub(super) prm4: f64,
    pub(super) ax3direc: bool,
}

impl Default for IntSurfQuadric {
    fn default() -> Self {
        Self::new()
    }
}

/// `gp_Ax3::Direct`, which cannot fail on the orthonormal frame of a gp surface.
pub(super) fn frame_is_direct(ax3: &Axis3) -> bool {
    ax3.direct()
        .unwrap_or_else(|_| unreachable!("the X and Y directions of a gp_Ax3 are orthogonal"))
}

impl IntSurfQuadric {
    /// An undefined quadric (`GeomAbs_OtherSurface`) whose evaluations return zeros
    /// (`IntSurf_Quadric()`); `IntSurfQuadric::new()`.
    pub fn new() -> Self {
        Self {
            ax3: Axis3::default(),
            lin: Line3::default(),
            typ: SurfaceType::OtherSurface,
            prm1: 0.0,
            prm2: 0.0,
            prm3: 0.0,
            prm4: 0.0,
            ax3direc: false,
        }
    }

    /// The quadric of a plane (`IntSurf_Quadric(gp_Pln)`); the axis line stays the
    /// default Z axis, as natively. `IntSurfQuadric::from_plane(&plane)`.
    pub fn from_plane(p: &Plane3) -> Self {
        let mut quadric = Self::new();
        quadric.set_plane(p);
        quadric
    }

    /// The quadric of a cylinder (`IntSurf_Quadric(gp_Cylinder)`).
    /// `IntSurfQuadric::from_cylinder(&cylinder)`.
    pub fn from_cylinder(c: &Cylinder3) -> Self {
        let mut quadric = Self::new();
        quadric.set_cylinder(c);
        quadric
    }

    /// The quadric of a sphere (`IntSurf_Quadric(gp_Sphere)`); `IntSurfQuadric::from_sphere(&sphere)`.
    pub fn from_sphere(s: &Sphere3) -> Self {
        let mut quadric = Self::new();
        quadric.set_sphere(s);
        quadric
    }

    /// The quadric of a cone (`IntSurf_Quadric(gp_Cone)`); `IntSurfQuadric::from_cone(&cone)`.
    pub fn from_cone(c: &Cone3) -> Self {
        let mut quadric = Self::new();
        quadric.set_cone(c);
        quadric
    }

    /// The quadric of a torus (`IntSurf_Quadric(gp_Torus)`); `IntSurfQuadric::from_torus(&torus)`.
    pub fn from_torus(t: &Torus3) -> Self {
        let mut quadric = Self::new();
        quadric.set_torus(t);
        quadric
    }

    /// Stores the surface kind, the frame and its handedness.
    fn set_frame(&mut self, typ: SurfaceType, ax3: Axis3) {
        self.typ = typ;
        self.ax3 = ax3;
        self.ax3direc = frame_is_direct(&ax3);
    }

    /// Becomes a plane (`SetValue(gp_Pln)`); the axis line keeps its previous value.
    /// `quadric.set_plane(&plane)`.
    pub fn set_plane(&mut self, p: &Plane3) {
        self.set_frame(SurfaceType::Plane, p.position());
        let [a, b, c, d] = p.coefficients().unwrap_or_else(|_| {
            unreachable!("the X and Y directions of a gp_Pln frame are orthogonal")
        });
        (self.prm1, self.prm2, self.prm3, self.prm4) = (a, b, c, d);
    }

    /// Becomes a cylinder (`SetValue(gp_Cylinder)`); `quadric.set_cylinder(&cylinder)`.
    pub fn set_cylinder(&mut self, c: &Cylinder3) {
        self.set_frame(SurfaceType::Cylinder, c.position());
        self.lin.set_position(self.ax3.axis());
        (self.prm1, self.prm2, self.prm3, self.prm4) = (c.radius(), 0.0, 0.0, 0.0);
    }

    /// Becomes a sphere (`SetValue(gp_Sphere)`); `quadric.set_sphere(&sphere)`.
    pub fn set_sphere(&mut self, s: &Sphere3) {
        self.set_frame(SurfaceType::Sphere, s.position());
        self.lin.set_position(self.ax3.axis());
        (self.prm1, self.prm2, self.prm3, self.prm4) = (s.radius(), 0.0, 0.0, 0.0);
    }

    /// Becomes a cone (`SetValue(gp_Cone)`); `quadric.set_cone(&cone)`.
    pub fn set_cone(&mut self, c: &Cone3) {
        self.set_frame(SurfaceType::Cone, c.position());
        self.lin.set_position(self.ax3.axis());
        self.prm1 = c.ref_radius();
        self.prm2 = c.semi_angle();
        self.prm3 = self.prm2.cos();
        self.prm4 = 0.0;
    }

    /// Becomes a torus (`SetValue(gp_Torus)`); `quadric.set_torus(&torus)`.
    pub fn set_torus(&mut self, t: &Torus3) {
        self.set_frame(SurfaceType::Torus, t.position());
        self.lin.set_position(self.ax3.axis());
        (self.prm1, self.prm2, self.prm3, self.prm4) =
            (t.major_radius(), t.minor_radius(), 0.0, 0.0);
    }

    /// The kind of surface (`TypeQuadric`); `quadric.type_quadric()`.
    pub const fn type_quadric(&self) -> SurfaceType {
        self.typ
    }

    /// The plane on the stored frame (`Plane`); `quadric.plane()`.
    pub fn plane(&self) -> Plane3 {
        Plane3::from_position(self.ax3)
    }

    /// The sphere of radius `prm1` (`Sphere`); Standard_ConstructionError when `prm1`
    /// is negative (for example on a plane quadric). `quadric.sphere()?`.
    pub fn sphere(&self) -> Result<Sphere3, Failure> {
        Sphere3::new(self.ax3, self.prm1)
    }

    /// The cylinder of radius `prm1` (`Cylinder`); Standard_ConstructionError when
    /// `prm1` is negative. `quadric.cylinder()?`.
    pub fn cylinder(&self) -> Result<Cylinder3, Failure> {
        Cylinder3::new(self.ax3, self.prm1)
    }

    /// The cone of semi-angle `prm2` and reference radius `prm1` (`Cone`);
    /// Standard_ConstructionError when they are invalid. `quadric.cone()?`.
    pub fn cone(&self) -> Result<Cone3, Failure> {
        Cone3::new(self.ax3, self.prm2, self.prm1)
    }

    /// The torus of radii `prm1`, `prm2` (`Torus`); Standard_ConstructionError when
    /// they are invalid. `quadric.torus()?`.
    pub fn torus(&self) -> Result<Torus3, Failure> {
        Torus3::new(self.ax3, self.prm1, self.prm2)
    }

    /// The point of parameters `(u, v)` (`Value`); the origin for an undefined quadric.
    /// `quadric.value(0.5, 0.25)`.
    pub fn value(&self, u: f64, v: f64) -> Point3 {
        let (ax3, prm1, prm2) = (self.ax3, self.prm1, self.prm2);
        match self.typ {
            SurfaceType::Plane => elslib::plane_value(u, v, ax3),
            SurfaceType::Cylinder => elslib::cylinder_value(u, v, ax3, prm1),
            SurfaceType::Sphere => elslib::sphere_value(u, v, ax3, prm1),
            SurfaceType::Cone => elslib::cone_value(u, v, ax3, prm1, prm2),
            SurfaceType::Torus => elslib::torus_value(u, v, ax3, prm1, prm2),
            _ => Point3::new(0.0, 0.0, 0.0),
        }
    }

    /// The point and first derivatives (`D1`); `None` for an undefined quadric, whose
    /// native output arguments are left unchanged. `quadric.d1(0.5, 0.25)`.
    pub fn d1(&self, u: f64, v: f64) -> Option<(Point3, Vector3, Vector3)> {
        let (ax3, prm1, prm2) = (self.ax3, self.prm1, self.prm2);
        match self.typ {
            SurfaceType::Plane => Some(elslib::plane_d1(u, v, ax3)),
            SurfaceType::Cylinder => Some(elslib::cylinder_d1(u, v, ax3, prm1)),
            SurfaceType::Sphere => Some(elslib::sphere_d1(u, v, ax3, prm1)),
            SurfaceType::Cone => Some(elslib::cone_d1(u, v, ax3, prm1, prm2)),
            SurfaceType::Torus => Some(elslib::torus_d1(u, v, ax3, prm1, prm2)),
            _ => None,
        }
    }

    /// The derivative of orders `(nu, nv)` (`DN`); a null vector for an undefined
    /// quadric. `quadric.dn(0.5, 0.25, 1, 1)`.
    pub fn dn(&self, u: f64, v: f64, nu: i32, nv: i32) -> Vector3 {
        let (ax3, prm1, prm2) = (self.ax3, self.prm1, self.prm2);
        match self.typ {
            SurfaceType::Plane => elslib::plane_dn(u, v, ax3, nu, nv),
            SurfaceType::Cylinder => elslib::cylinder_dn(u, v, ax3, prm1, nu, nv),
            SurfaceType::Sphere => elslib::sphere_dn(u, v, ax3, prm1, nu, nv),
            SurfaceType::Cone => elslib::cone_dn(u, v, ax3, prm1, prm2, nu, nv),
            SurfaceType::Torus => elslib::torus_dn(u, v, ax3, prm1, prm2, nu, nv),
            _ => Vector3::new(0.0, 0.0, 0.0),
        }
    }

    /// The parameters of `p` (`Parameters`); `None` for an undefined quadric, whose
    /// native output arguments are left unchanged. `quadric.parameters(point)`.
    pub fn parameters(&self, p: Point3) -> Option<(f64, f64)> {
        let (ax3, prm1, prm2) = (self.ax3, self.prm1, self.prm2);
        match self.typ {
            SurfaceType::Plane => Some(elslib::plane_parameters(ax3, p)),
            SurfaceType::Cylinder => Some(elslib::cylinder_parameters(ax3, prm1, p)),
            SurfaceType::Sphere => Some(elslib::sphere_parameters(ax3, prm1, p)),
            SurfaceType::Cone => Some(elslib::cone_parameters(ax3, prm1, prm2, p)),
            SurfaceType::Torus => Some(elslib::torus_parameters(ax3, prm1, prm2, p)),
            _ => None,
        }
    }
}
