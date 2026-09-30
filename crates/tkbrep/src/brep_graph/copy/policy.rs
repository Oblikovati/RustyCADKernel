// Rust translation of the policies of ModelingData/TKBRep/BRepGraph/BRepGraph_Copy.hxx
// and the handle copies (`copySurface`, `copyCurve`, `copyPCurve`, `copyMeshHandle`) of
// BRepGraph_Copy.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The copy policies and the policy-driven handle copies.
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkmath::poly::{Polygon2D, Polygon3D, PolygonOnTriangulation, Triangulation};

/// `BRepGraph_Copy::GeomPolicy`: how geometry handles (`Geom_Curve`, `Geom_Surface`,
/// `Geom2d_Curve`) reach the copy.
///
/// ```
/// use rustycad_tkbrep::brep_graph::copy::GeomPolicy;
/// assert_eq!(GeomPolicy::default(), GeomPolicy::Copy);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum GeomPolicy {
    /// `Copy`: deep-clone geometry handles; the result is fully independent.
    #[default]
    Copy,
    /// `Share`: reuse the source geometry handles; only topology is duplicated.
    Share,
    /// `Drop`: pure topology: edges carry no curves, faces carry no surfaces.
    Drop,
}

/// `BRepGraph_Copy::MeshPolicy`: how mesh data (`Poly_Triangulation`, `Poly_Polygon3D`,
/// `Poly_PolygonOnTriangulation`) reaches the copy.
///
/// ```
/// use rustycad_tkbrep::brep_graph::copy::MeshPolicy;
/// assert_eq!(MeshPolicy::default(), MeshPolicy::Copy);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum MeshPolicy {
    /// `Copy`: deep-clone mesh data.
    #[default]
    Copy,
    /// `Share`: reuse the source mesh handles.
    Share,
    /// `Drop`: discard all mesh data on copied entities.
    Drop,
}

/// `BRepGraph_Copy::CachePolicy`: whether transient runtime cache services follow.
///
/// ```
/// use rustycad_tkbrep::brep_graph::copy::CachePolicy;
/// assert_eq!(CachePolicy::default(), CachePolicy::Drop);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CachePolicy {
    /// `Drop`: do not copy runtime cache services or entries.
    #[default]
    Drop,
    /// `CopyFresh`: copy fresh, remappable runtime cache entries.
    CopyFresh,
}

/// The deep copy of a geometry or mesh handle target (`Geom_Geometry::Copy()` down-cast
/// to the handle's class, or the mesh classes' `Copy()`); a raise propagates.
pub(crate) trait DeepCopy: RuntimeTyped {
    /// `Copy()`: a deep copy of the object.
    fn deep_copy(&self) -> Result<Handle<Self>, Failure>;
}

impl DeepCopy for dyn GeomSurface {
    fn deep_copy(&self) -> Result<Handle<Self>, Failure> {
        Ok(self.copy()?.cast())
    }
}

impl DeepCopy for dyn GeomCurve {
    fn deep_copy(&self) -> Result<Handle<Self>, Failure> {
        Ok(self.copy()?.cast())
    }
}

impl DeepCopy for dyn Geom2dCurve {
    fn deep_copy(&self) -> Result<Handle<Self>, Failure> {
        Ok(self.copy()?.cast())
    }
}

impl DeepCopy for Triangulation {
    fn deep_copy(&self) -> Result<Handle<Self>, Failure> {
        Ok(self.copy())
    }
}

impl DeepCopy for Polygon3D {
    fn deep_copy(&self) -> Result<Handle<Self>, Failure> {
        self.copy()
    }
}

impl DeepCopy for Polygon2D {
    fn deep_copy(&self) -> Result<Handle<Self>, Failure> {
        Ok(self.copy())
    }
}

impl DeepCopy for PolygonOnTriangulation {
    fn deep_copy(&self) -> Result<Handle<Self>, Failure> {
        self.copy()
    }
}

// The three-way handle policy both copy policies share.
#[derive(Clone, Copy)]
enum HandleMode {
    Copy,
    Share,
    Drop,
}

// `copySurface`, `copyCurve`, `copyPCurve` and `copyMeshHandle`: a null handle stays
// null; `Copy` deep-clones, `Share` returns the same handle and `Drop` a null one.
fn copy_handle<T: ?Sized + DeepCopy>(
    handle: &Handle<T>,
    mode: HandleMode,
) -> Result<Handle<T>, Failure> {
    let Some(stored) = handle.get() else {
        return Ok(handle.clone());
    };
    match mode {
        HandleMode::Copy => stored.deep_copy(),
        HandleMode::Share => Ok(handle.clone()),
        HandleMode::Drop => Ok(Handle::default()),
    }
}

/// `copySurface`/`copyCurve`/`copyPCurve(theHandle, thePolicy)`: a geometry handle under
/// the geometry policy.
pub(crate) fn copy_geometry<T: ?Sized + DeepCopy>(
    geometry: &Handle<T>,
    policy: GeomPolicy,
) -> Result<Handle<T>, Failure> {
    let mode = match policy {
        GeomPolicy::Copy => HandleMode::Copy,
        GeomPolicy::Share => HandleMode::Share,
        GeomPolicy::Drop => HandleMode::Drop,
    };
    copy_handle(geometry, mode)
}

/// `copyMeshHandle(theHandle, thePolicy)`: a mesh handle under the mesh policy.
pub(crate) fn copy_mesh<T: ?Sized + DeepCopy>(
    mesh: &Handle<T>,
    policy: MeshPolicy,
) -> Result<Handle<T>, Failure> {
    let mode = match policy {
        MeshPolicy::Copy => HandleMode::Copy,
        MeshPolicy::Share => HandleMode::Share,
        MeshPolicy::Drop => HandleMode::Drop,
    };
    copy_handle(mesh, mode)
}
