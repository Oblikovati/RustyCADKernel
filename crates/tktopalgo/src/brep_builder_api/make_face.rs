// Rust translation of BRepBuilderAPI_MakeFace.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepBuilderAPI_MakeFace`: a face on a surface, bounded by its parameter box or by
//! wires, built by the wrapped `BRepLib_MakeFace`.
use super::errors::BRepBuilderAPIFaceError;
use super::make_shape::{BRepBuilderAPIMakeShape, BRepBuilderAPIMakeShapeBase};
use super::wrap::{self, wrapped_constructors, wrapped_updates};
use crate::brep_lib::BRepLibMakeFace;
use rustycad_tkbrep::topods::{Face, Wire};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomSurface, SurfaceBounds};
use rustycad_tkmath::gp::{Cone3, Cylinder3, Plane3, Sphere3, Torus3};

/// `BRepBuilderAPI_MakeFace` (docs/topalgo-design.md section 10): each constructor, Init
/// and Add runs the `BRepLib_MakeFace` member of the same Rust name; `IsDone` and
/// `Error` are those of the BRepLib maker. The parameter boxes are
/// `(UMin, UMax, VMin, VMax)`.
///
/// ```
/// use rustycad_tkmath::gp::Plane3;
/// use rustycad_tktopalgo::brep_builder_api::{
///     BRepBuilderAPIFaceError, BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape,
/// };
/// let maker = BRepBuilderAPIMakeFace::from_plane_bounds(Plane3::default(), (0.0, 1.0, 0.0, 2.0))?;
/// assert!(maker.is_done());
/// assert_eq!(maker.error(), BRepBuilderAPIFaceError::FaceDone);
/// assert_eq!(BRepBuilderAPIMakeFace::new().error(), BRepBuilderAPIFaceError::NoFace);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone)]
pub struct BRepBuilderAPIMakeFace {
    base: BRepBuilderAPIMakeShapeBase,
    my_make_face: BRepLibMakeFace,
}

impl Default for BRepBuilderAPIMakeFace {
    fn default() -> Self {
        Self::new()
    }
}

impl BRepBuilderAPIMakeShape for BRepBuilderAPIMakeFace {
    fn make_shape_base(&self) -> &BRepBuilderAPIMakeShapeBase {
        &self.base
    }

    fn make_shape_base_mut(&mut self) -> &mut BRepBuilderAPIMakeShapeBase {
        &mut self.base
    }

    /// `IsDone()`: the flag of the BRepLib maker; `maker.is_done()`.
    fn is_done(&self) -> bool {
        self.my_make_face.is_done()
    }
}

impl BRepBuilderAPIMakeFace {
    /// `BRepBuilderAPI_MakeFace()`: no face, `BRepBuilderAPI_NoFace`;
    /// `BRepBuilderAPIMakeFace::new()`.
    pub fn new() -> Self {
        Self {
            base: BRepBuilderAPIMakeShapeBase::new(),
            my_make_face: BRepLibMakeFace::new(),
        }
    }

    /// Wraps a constructed BRepLib maker and takes its shape when it is done.
    fn wrap(my_make_face: BRepLibMakeFace) -> Result<Self, Failure> {
        let mut maker = Self {
            base: BRepBuilderAPIMakeShapeBase::new(),
            my_make_face,
        };
        maker.adopt()?;
        Ok(maker)
    }

    /// The step after each BRepLib call: done and the shape when the BRepLib maker is.
    fn adopt(&mut self) -> Result<(), Failure> {
        wrap::adopt(&mut self.base, &self.my_make_face)
    }

    wrapped_constructors!(BRepBuilderAPIMakeFace, BRepLibMakeFace;
        from_face(face: &Face) = "BRepBuilderAPI_MakeFace(const TopoDS_Face& F)";
        from_plane(surface: Plane3) = "BRepBuilderAPI_MakeFace(const gp_Pln& P)";
        from_cylinder(surface: Cylinder3) = "BRepBuilderAPI_MakeFace(const gp_Cylinder& C)";
        from_cone(surface: Cone3) = "BRepBuilderAPI_MakeFace(const gp_Cone& C)";
        from_sphere(surface: Sphere3) = "BRepBuilderAPI_MakeFace(const gp_Sphere& S)";
        from_torus(surface: Torus3) = "BRepBuilderAPI_MakeFace(const gp_Torus& C)";
        from_surface(surface: &Handle<dyn GeomSurface>, tol_degen: f64)
            = "BRepBuilderAPI_MakeFace(const occ::handle<Geom_Surface>& S, TolDegen)";
        from_plane_bounds(surface: Plane3, bounds: SurfaceBounds)
            = "BRepBuilderAPI_MakeFace(P, UMin, UMax, VMin, VMax)";
        from_cylinder_bounds(surface: Cylinder3, bounds: SurfaceBounds)
            = "BRepBuilderAPI_MakeFace(C, UMin, UMax, VMin, VMax)";
        from_cone_bounds(surface: Cone3, bounds: SurfaceBounds)
            = "BRepBuilderAPI_MakeFace(C, UMin, UMax, VMin, VMax)";
        from_sphere_bounds(surface: Sphere3, bounds: SurfaceBounds)
            = "BRepBuilderAPI_MakeFace(S, UMin, UMax, VMin, VMax)";
        from_torus_bounds(surface: Torus3, bounds: SurfaceBounds)
            = "BRepBuilderAPI_MakeFace(C, UMin, UMax, VMin, VMax)";
        from_surface_bounds(surface: &Handle<dyn GeomSurface>, bounds: SurfaceBounds, tol_degen: f64)
            = "BRepBuilderAPI_MakeFace(S, UMin, UMax, VMin, VMax, TolDegen)";
        from_wire(wire: &Wire, only_plane: bool) = "BRepBuilderAPI_MakeFace(W, OnlyPlane)";
        from_plane_wire(surface: Plane3, wire: &Wire, inside: bool) = "BRepBuilderAPI_MakeFace(P, W, Inside)";
        from_cylinder_wire(surface: Cylinder3, wire: &Wire, inside: bool) = "BRepBuilderAPI_MakeFace(C, W, Inside)";
        from_cone_wire(surface: Cone3, wire: &Wire, inside: bool) = "BRepBuilderAPI_MakeFace(C, W, Inside)";
        from_sphere_wire(surface: Sphere3, wire: &Wire, inside: bool) = "BRepBuilderAPI_MakeFace(S, W, Inside)";
        from_torus_wire(surface: Torus3, wire: &Wire, inside: bool) = "BRepBuilderAPI_MakeFace(C, W, Inside)";
        from_surface_wire(surface: &Handle<dyn GeomSurface>, wire: &Wire, inside: bool)
            = "BRepBuilderAPI_MakeFace(S, W, Inside)";
        from_face_wire(face: &Face, wire: &Wire) = "BRepBuilderAPI_MakeFace(F, W)";
    );

    wrapped_updates!(BRepBuilderAPIMakeFace, my_make_face;
        init_face(face: &Face) = "Init(F)";
        init_surface(surface: &Handle<dyn GeomSurface>, bound: bool, tol_degen: f64) = "Init(S, Bound, TolDegen)";
        init_surface_bounds(surface: &Handle<dyn GeomSurface>, bounds: SurfaceBounds, tol_degen: f64)
            = "Init(S, UMin, UMax, VMin, VMax, TolDegen)";
        add(wire: &Wire) = "Add(W)";
    );

    /// `Error()`: the status of the BRepLib maker, through the native switch;
    /// `maker.error()`.
    pub fn error(&self) -> BRepBuilderAPIFaceError {
        BRepBuilderAPIFaceError::from(self.my_make_face.error())
    }

    /// `Face()` and `operator TopoDS_Face()`: the face of the BRepLib maker (not checked,
    /// as natively); `maker.face()?`.
    pub fn face(&self) -> Result<Face, Failure> {
        self.my_make_face.face()
    }
}
