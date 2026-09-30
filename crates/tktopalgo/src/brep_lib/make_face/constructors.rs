// Rust translation of the constructors of BRepLib_MakeFace.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The constructors of `BRepLib_MakeFace`: each builds the Geom surface of its gp
//! argument, if any, and calls one Init, then adds the wire when it has one.
use super::BRepLibMakeFace;
use crate::brep_lib::errors::BRepLibFaceError;
use crate::brep_lib::find_surface::BRepLibFindSurface;
use crate::brep_lib::same_parameter::same_parameter_shape;
use crate::brep_lib::update_tolerances::update_tolerances;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{Edge, Face, ShapeIterator, Wire};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkg3d::geom::{
    GeomConicalSurface, GeomCylindricalSurface, GeomPlane, GeomSphericalSurface, GeomSurface,
    GeomToroidalSurface, SurfaceBounds,
};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{Cone3, Cylinder3, Plane3, Sphere3, Torus3};

/// The three constructors of one gp surface kind: the whole surface, a parameter box,
/// or a wire on the surface, each on `Geom_X(P)` at Precision::Confusion().
macro_rules! gp_surface_constructors {
    ($gp:ty, $native:literal, $geom:expr, $plain:ident, $bounds:ident, $wire:ident) => {
        #[doc = concat!("`BRepLib_MakeFace(const ", $native, "& P)`: the face over the natural bounds; `BRepLibMakeFace::", stringify!($plain), "(p)?`.")]
        pub fn $plain(surface: $gp) -> Result<Self, Failure> {
            let geom: Handle<dyn GeomSurface> = Handle::new($geom(surface)).cast();
            Self::initialized(|maker| maker.init_surface(&geom, true, precision::confusion()))
        }
        #[doc = concat!("`BRepLib_MakeFace(const ", $native, "& P, UMin, UMax, VMin, VMax)`; `BRepLibMakeFace::", stringify!($bounds), "(p, (0.0, 1.0, 0.0, 1.0))?`.")]
        pub fn $bounds(surface: $gp, bounds: SurfaceBounds) -> Result<Self, Failure> {
            let geom: Handle<dyn GeomSurface> = Handle::new($geom(surface)).cast();
            Self::initialized(|maker| maker.init_surface_bounds(&geom, bounds, precision::confusion()))
        }
        #[doc = concat!("`BRepLib_MakeFace(const ", $native, "& P, W, Inside)`: the face bounded by the wire (reversed when `inside` and it bounds an infinite area); `BRepLibMakeFace::", stringify!($wire), "(p, &w, true)?`.")]
        pub fn $wire(surface: $gp, wire: &Wire, inside: bool) -> Result<Self, Failure> {
            let geom: Handle<dyn GeomSurface> = Handle::new($geom(surface)).cast();
            Self::from_surface_wire(&geom, wire, inside)
        }
    };
}

impl BRepLibMakeFace {
    gp_surface_constructors!(
        Plane3,
        "gp_Pln",
        GeomPlane::from_pln,
        from_plane,
        from_plane_bounds,
        from_plane_wire
    );
    gp_surface_constructors!(
        Cylinder3,
        "gp_Cylinder",
        GeomCylindricalSurface::from_cylinder,
        from_cylinder,
        from_cylinder_bounds,
        from_cylinder_wire
    );
    gp_surface_constructors!(
        Cone3,
        "gp_Cone",
        GeomConicalSurface::from_cone,
        from_cone,
        from_cone_bounds,
        from_cone_wire
    );
    gp_surface_constructors!(
        Sphere3,
        "gp_Sphere",
        GeomSphericalSurface::from_sphere,
        from_sphere,
        from_sphere_bounds,
        from_sphere_wire
    );
    gp_surface_constructors!(
        Torus3,
        "gp_Torus",
        GeomToroidalSurface::from_torus,
        from_torus,
        from_torus_bounds,
        from_torus_wire
    );

    /// `BRepLib_MakeFace(F)`: [`Self::init_face`] (not marked done);
    /// `BRepLibMakeFace::from_face(&face)?`.
    pub fn from_face(face: &Face) -> Result<Self, Failure> {
        Self::initialized(|maker| maker.init_face(face))
    }

    /// `BRepLib_MakeFace(S, TolDegen)`: the face over the natural bounds of the surface;
    /// `BRepLibMakeFace::from_surface(&s, 1e-7)?`.
    pub fn from_surface(
        surface: &Handle<dyn GeomSurface>,
        tol_degen: f64,
    ) -> Result<Self, Failure> {
        Self::initialized(|maker| maker.init_surface(surface, true, tol_degen))
    }

    /// `BRepLib_MakeFace(S, UMin, UMax, VMin, VMax, TolDegen)`;
    /// `BRepLibMakeFace::from_surface_bounds(&s, (0.0, 1.0, 0.0, 1.0), 1e-7)?`.
    pub fn from_surface_bounds(
        surface: &Handle<dyn GeomSurface>,
        bounds: SurfaceBounds,
        tol_degen: f64,
    ) -> Result<Self, Failure> {
        Self::initialized(|maker| maker.init_surface_bounds(surface, bounds, tol_degen))
    }

    /// `BRepLib_MakeFace(S, W, Inside)`: the face on the surface bounded by the wire,
    /// reversed when `inside`, the wire is closed and the infinite point is inside;
    /// `BRepLibMakeFace::from_surface_wire(&s, &w, true)?`.
    pub fn from_surface_wire(
        surface: &Handle<dyn GeomSurface>,
        wire: &Wire,
        inside: bool,
    ) -> Result<Self, Failure> {
        let mut maker =
            Self::initialized(|maker| maker.init_surface(surface, false, precision::confusion()))?;
        maker.add(wire)?;
        if inside && BRepTool::is_closed(wire)? {
            maker.check_inside()?;
        }
        Ok(maker)
    }

    /// `BRepLib_MakeFace(F, W)`: [`Self::init_face`] then [`Self::add`];
    /// `BRepLibMakeFace::from_face_wire(&face, &w)?`.
    pub fn from_face_wire(face: &Face, wire: &Wire) -> Result<Self, Failure> {
        let mut maker = Self::from_face(face)?;
        maker.add(wire)?;
        Ok(maker)
    }

    /// `BRepLib_MakeFace(W, OnlyPlane)`: the face of the surface found through the wire
    /// (BRepLib_FindSurface, a plane with `only_plane`, `BRepLib_NotPlanar` without
    /// one) at the larger of 1.2 times the reached tolerance and the found tolerance;
    /// with `only_plane` the degenerated edges are left out of the wire. The tolerances
    /// are then updated, the edges made SameParameter, and a closed wire reversed when
    /// it bounds an infinite area; `BRepLibMakeFace::from_wire(&w, false)?`.
    pub fn from_wire(wire: &Wire, only_plane: bool) -> Result<Self, Failure> {
        let mut maker = Self::new();
        // Find a surface through the wire
        let found = BRepLibFindSurface::new(wire, -1., only_plane, true)?;
        if !found.found() {
            maker.my_error = BRepLibFaceError::NotPlanar;
            return Ok(maker);
        }
        // build the face and add the wire
        maker.my_error = BRepLibFaceError::FaceDone;
        let tol: f64 = std_max(1.2 * found.tolerance_reached(), found.tolerance());
        let mut face = Face::default();
        BRepBuilder::default().make_face_on_located_surface(
            &mut face,
            &found.surface(),
            &found.location(),
            tol,
        )?;
        maker.base.my_shape = face.into_shape();
        let bounding = if only_plane {
            without_degenerated(wire)?
        } else {
            wire.clone()
        };
        maker.add(&bounding)?;
        update_tolerances(&maker.base.my_shape, false)?;
        same_parameter_shape(&maker.base.my_shape, tol, true)?;
        if BRepTool::is_closed(&bounding)? {
            maker.check_inside()?;
        }
        Ok(maker)
    }
}

/// "get rid of degenerative edges in the input wire": a new wire of the other edges
/// (of the FORWARD wire) with the orientation and Closed flag of the input, or the input
/// itself when it has none.
fn without_degenerated(wire: &Wire) -> Result<Wire, Failure> {
    let builder = BRepBuilder::default();
    let mut result = Wire::default();
    builder.make_wire(&mut result);
    let forward = wire.oriented(Orientation::Forward);
    let mut has_degenerated: bool = false;
    let mut children = ShapeIterator::new(&forward, true, true);
    while let Ok(child) = children.value() {
        if BRepTool::degenerated(&Edge::unchecked(child.clone()))? {
            has_degenerated = true;
        } else {
            builder.add(&mut result, child)?;
        }
        children.next();
    }
    if !has_degenerated {
        return Ok(wire.clone());
    }
    // return to original orient
    result.set_orientation(wire.orientation());
    result.set_closed(wire.closed()?)?;
    Ok(result)
}
