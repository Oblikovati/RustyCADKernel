// Rust translation of ModelingAlgorithms/TKShHealing/ShapeExtend/ShapeExtend_CompositeSurface.hxx/.cxx
// (constructors, Init, the patches and joints, the local/global mappings) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ShapeExtend_CompositeSurface`: a surface made of a grid of patches
//! (docs/shhealing-design.md section 2).
mod geometry;
mod joints;

use super::status::ShapeExtendParametrisation;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::{Array1, HArray1, HArray2};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg3d::geom::{GeomGeometry, GeomSurface};
use rustycad_tkmath::gp::{Point2, Transform2, TransformForm, Vector2};

/// `occ::handle<NCollection_HArray2<occ::handle<Geom_Surface>>>`: the grid of patches.
pub type SurfaceGrid = Handle<HArray2<Handle<dyn GeomSurface>>>;

/// `occ::handle<NCollection_HArray1<double>>`: joint values.
pub type JointValues = Handle<HArray1<f64>>;

/// The fields of a composite surface.
#[derive(Clone, Default)]
struct CompositeFields {
    /// `myPatches`.
    patches: SurfaceGrid,
    /// `myUJointValues`.
    u_joints: JointValues,
    /// `myVJointValues`.
    v_joints: JointValues,
    /// `myUClosed` (left uninitialized by the default constructor natively).
    u_closed: bool,
    /// `myVClosed`.
    v_closed: bool,
}

/// `ShapeExtend_CompositeSurface`: a grid of patches (`NbUPatches` rows by `NbVPatches`
/// columns) with a global parametrisation: the joint values `U(1) < ... < U(NbU + 1)` and
/// `V(1) < ... < V(NbV + 1)` map each patch's parameters linearly onto its cell.
///
/// It is a `Geom_Surface`; it holds its fields behind a lock (docs/geom-design.md section
/// 3) and never calls a patch while it holds the lock. Members that dereference the null
/// grid of a default-constructed surface natively return Standard_NullObject errors.
///
/// ```
/// use rustycad_tkshhealing::shape_extend::ShapeExtendCompositeSurface;
/// let surface = ShapeExtendCompositeSurface::new();
/// assert!(surface.patches().is_null());
/// ```
pub struct ShapeExtendCompositeSurface {
    parent: StandardTransient,
    fields: SharedState<CompositeFields>,
}
impl_transient!(
    ShapeExtendCompositeSurface,
    "ShapeExtend_CompositeSurface",
    dyn GeomSurface,
    parent,
    interfaces: [dyn GeomGeometry, dyn GeomSurface]
);

impl Default for ShapeExtendCompositeSurface {
    fn default() -> Self {
        Self {
            parent: StandardTransient::default(),
            fields: SharedState::new(CompositeFields::default()),
        }
    }
}

/// A Standard_NullObject failure for a null grid or joint array.
fn null_object(member: &str, what: &str) -> Failure {
    Failure::with_message(
        "Standard_NullObject",
        &format!(
            "ShapeExtend_CompositeSurface::{member}: null {what}; expected an initialized surface"
        ),
    )
}

impl ShapeExtendCompositeSurface {
    /// `ShapeExtend_CompositeSurface()`: no patch; `ShapeExtendCompositeSurface::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `ShapeExtend_CompositeSurface(GridSurf, param)`: [`Self::init`] on a new surface
    /// (its result is dropped, as natively); `ShapeExtendCompositeSurface::with_grid(&grid, p)?`.
    pub fn with_grid(
        grid: &SurfaceGrid,
        param: ShapeExtendParametrisation,
    ) -> Result<Self, Failure> {
        let surface = Self::new();
        surface.init(grid, param)?;
        Ok(surface)
    }

    /// `ShapeExtend_CompositeSurface(GridSurf, UJoints, VJoints)`: [`Self::init_with_joints`]
    /// on a new surface; `ShapeExtendCompositeSurface::with_joints(&grid, &u, &v)?`.
    pub fn with_joints(
        grid: &SurfaceGrid,
        u_joints: &Array1<'_, f64>,
        v_joints: &Array1<'_, f64>,
    ) -> Result<Self, Failure> {
        let surface = Self::new();
        surface.init_with_joints(grid, u_joints, v_joints)?;
        Ok(surface)
    }

    /// `Init(GridSurf, param)`: takes the grid, computes the joint values for `param` and
    /// returns `CheckConnectivity(Precision::Confusion())`; false for a null grid;
    /// `surface.init(&grid, ShapeExtendParametrisation::Natural)?`.
    pub fn init(
        &self,
        grid: &SurfaceGrid,
        param: ShapeExtendParametrisation,
    ) -> Result<bool, Failure> {
        if grid.is_null() {
            return Ok(false);
        }
        self.fields.write().patches = grid.clone();
        self.compute_joint_values(param)?;
        self.check_connectivity(precision::confusion())
    }

    /// `Init(GridSurf, UJoints, VJoints)`: takes the grid and the joint values; when
    /// either set is rejected, the natural joint values replace them and the result is
    /// false; false too when the patches are not connected; false for a null grid;
    /// `surface.init_with_joints(&grid, &u, &v)?`.
    pub fn init_with_joints(
        &self,
        grid: &SurfaceGrid,
        u_joints: &Array1<'_, f64>,
        v_joints: &Array1<'_, f64>,
    ) -> Result<bool, Failure> {
        if grid.is_null() {
            return Ok(false);
        }
        self.fields.write().patches = grid.clone();
        let mut ok = true;
        if !self.set_u_joint_values(u_joints)? || !self.set_v_joint_values(v_joints)? {
            ok = false;
            self.compute_joint_values(ShapeExtendParametrisation::Natural)?;
        }
        Ok(self.check_connectivity(precision::confusion())? && ok)
    }

    /// `Patches()`: the grid handle (null before `Init`); `surface.patches()`.
    pub fn patches(&self) -> SurfaceGrid {
        self.fields.read().patches.clone()
    }

    /// The grid, Standard_NullObject when there is none.
    fn grid(&self, member: &str) -> Result<SurfaceGrid, Failure> {
        let grid = self.patches();
        if grid.is_null() {
            return Err(null_object(member, "grid of patches"));
        }
        Ok(grid)
    }

    /// `NbUPatches()`: the number of rows of the grid; `surface.nb_u_patches()?`.
    pub fn nb_u_patches(&self) -> Result<i32, Failure> {
        let grid = self.grid("NbUPatches")?;
        Ok(grid.array2().col_length())
    }

    /// `NbVPatches()`: the number of columns of the grid; `surface.nb_v_patches()?`.
    pub fn nb_v_patches(&self) -> Result<i32, Failure> {
        let grid = self.grid("NbVPatches")?;
        Ok(grid.array2().row_length())
    }

    /// `Patch(i, j)`: the patch at row `i`, column `j`; Standard_OutOfRange outside the
    /// grid; `surface.patch(1, 1)?`.
    pub fn patch(&self, i: i32, j: i32) -> Result<Handle<dyn GeomSurface>, Failure> {
        let grid = self.grid("Patch")?;
        let array = grid.array2();
        let patch = array
            .value(i, j)
            .map_err(|e| e.into_failure("ShapeExtend_CompositeSurface::Patch"))?;
        Ok(patch.clone())
    }

    /// The patch at `(i, j)`, Standard_NullObject when it is a null surface (a null
    /// dereference natively).
    fn live_patch(&self, i: i32, j: i32) -> Result<Handle<dyn GeomSurface>, Failure> {
        let patch = self.patch(i, j)?;
        if patch.is_null() {
            return Err(null_object("Patch", &format!("patch ({i}, {j})")));
        }
        Ok(patch)
    }

    /// `Patch(U, V)`: the patch of the global parameters; `surface.patch_at(u, v)?`.
    pub fn patch_at(&self, u: f64, v: f64) -> Result<Handle<dyn GeomSurface>, Failure> {
        self.patch(self.locate_u_parameter(u)?, self.locate_v_parameter(v)?)
    }

    /// `Patch(pnt)`: the patch of the global point; `surface.patch_at_point(p)?`.
    pub fn patch_at_point(&self, pnt: Point2) -> Result<Handle<dyn GeomSurface>, Failure> {
        self.patch_at(pnt.x(), pnt.y())
    }

    /// `LocateUParameter(U)`: the row whose joint interval holds `u` (the first row below
    /// the second joint, the last row past the others); `surface.locate_u_parameter(u)?`.
    pub fn locate_u_parameter(&self, u: f64) -> Result<i32, Failure> {
        let nb = self.nb_u_patches()?;
        locate(&self.joints(true, "LocateUParameter")?, nb, u)
    }

    /// `LocateVParameter(V)`: the column of `v`, as [`Self::locate_u_parameter`];
    /// `surface.locate_v_parameter(v)?`.
    pub fn locate_v_parameter(&self, v: f64) -> Result<i32, Failure> {
        let nb = self.nb_v_patches()?;
        locate(&self.joints(false, "LocateVParameter")?, nb, v)
    }

    /// `LocateUVPoint(pnt, i, j)`: the row and column of a global point;
    /// `let (i, j) = surface.locate_uv_point(p)?;`.
    pub fn locate_uv_point(&self, pnt: Point2) -> Result<(i32, i32), Failure> {
        Ok((
            self.locate_u_parameter(pnt.x())?,
            self.locate_v_parameter(pnt.y())?,
        ))
    }
}

/// The body of `LocateUParameter`/`LocateVParameter`: the first `i - 1` with
/// `value < joints(i)` for `i` in `2..=nb`, else `nb`.
fn locate(joints: &JointValues, nb: i32, value: f64) -> Result<i32, Failure> {
    for i in 2..=nb {
        if value < joint(joints, i, "Locate")? {
            return Ok(i - 1);
        }
    }
    Ok(nb)
}

/// `joints->Value(i)`; Standard_OutOfRange outside the array.
fn joint(joints: &JointValues, i: i32, member: &str) -> Result<f64, Failure> {
    let array = joints.array1();
    array
        .value(i)
        .copied()
        .map_err(|e| e.into_failure(&format!("ShapeExtend_CompositeSurface::{member}")))
}

/// The cell of patch `(i, j)` along one direction: its joint interval `[a, b]` and the
/// patch's parameter interval `[p1, p2]`.
struct Cell {
    joint_first: f64,
    joint_last: f64,
    first: f64,
    last: f64,
}

impl ShapeExtendCompositeSurface {
    /// The joint values of one direction, Standard_NullObject when they are not computed.
    fn joints(&self, u_direction: bool, member: &str) -> Result<JointValues, Failure> {
        let fields = self.fields.read();
        let joints = if u_direction {
            &fields.u_joints
        } else {
            &fields.v_joints
        };
        if joints.is_null() {
            return Err(null_object(member, "joint values"));
        }
        Ok(joints.clone())
    }

    /// The U and V cells of patch `(i, j)`: `Bounds` of the patch and the joints around it.
    fn cells(&self, i: i32, j: i32, member: &str) -> Result<(Cell, Cell), Failure> {
        let (u1, u2, v1, v2) = self.live_patch(i, j)?.bounds();
        let (uj, vj) = (self.joints(true, member)?, self.joints(false, member)?);
        let u = Cell {
            joint_first: joint(&uj, i, member)?,
            joint_last: joint(&uj, i + 1, member)?,
            first: u1,
            last: u2,
        };
        let v = Cell {
            joint_first: joint(&vj, j, member)?,
            joint_last: joint(&vj, j + 1, member)?,
            first: v1,
            last: v2,
        };
        Ok((u, v))
    }

    /// `ULocalToGlobal(i, j, u)`: the global U of the local `u` of patch `(i, j)`;
    /// `surface.u_local_to_global(1, 1, 0.5)?`.
    pub fn u_local_to_global(&self, i: i32, j: i32, u: f64) -> Result<f64, Failure> {
        let (cell, _) = self.cells(i, j, "ULocalToGlobal")?;
        Ok(cell.local_to_global(u))
    }

    /// `VLocalToGlobal(i, j, v)`: the global V of the local `v`;
    /// `surface.v_local_to_global(1, 1, 0.5)?`.
    pub fn v_local_to_global(&self, i: i32, j: i32, v: f64) -> Result<f64, Failure> {
        let (_, cell) = self.cells(i, j, "VLocalToGlobal")?;
        Ok(cell.local_to_global(v))
    }

    /// `LocalToGlobal(i, j, uv)`: both; `surface.local_to_global(1, 1, uv)?`.
    pub fn local_to_global(&self, i: i32, j: i32, uv: Point2) -> Result<Point2, Failure> {
        let (u, v) = self.cells(i, j, "LocalToGlobal")?;
        Ok(Point2::new(
            u.local_to_global(uv.x()),
            v.local_to_global(uv.y()),
        ))
    }

    /// `UGlobalToLocal(i, j, U)`: the local u on patch `(i, j)` of the global `u`;
    /// `surface.u_global_to_local(1, 1, 0.5)?`.
    pub fn u_global_to_local(&self, i: i32, j: i32, u: f64) -> Result<f64, Failure> {
        let (cell, _) = self.cells(i, j, "UGlobalToLocal")?;
        Ok(cell.global_to_local(u))
    }

    /// `VGlobalToLocal(i, j, V)`: the local v of the global `v`;
    /// `surface.v_global_to_local(1, 1, 0.5)?`.
    pub fn v_global_to_local(&self, i: i32, j: i32, v: f64) -> Result<f64, Failure> {
        let (_, cell) = self.cells(i, j, "VGlobalToLocal")?;
        Ok(cell.global_to_local(v))
    }

    /// `GlobalToLocal(i, j, UV)`: both; `surface.global_to_local(1, 1, uv)?`.
    pub fn global_to_local(&self, i: i32, j: i32, uv: Point2) -> Result<Point2, Failure> {
        let (u, v) = self.cells(i, j, "GlobalToLocal")?;
        Ok(Point2::new(
            u.global_to_local(uv.x()),
            v.global_to_local(uv.y()),
        ))
    }

    /// `GlobalToLocalTransformation(i, j, uFact, Trsf)`: the global-to-local mapping of
    /// patch `(i, j)` as a U affinity factor and a 2D transformation (a translation then a
    /// scale by the V factor); true unless both are the identity;
    /// `let (changed, u_fact, trsf) = surface.global_to_local_transformation(1, 1)?;`.
    pub fn global_to_local_transformation(
        &self,
        i: i32,
        j: i32,
    ) -> Result<(bool, f64, Transform2), Failure> {
        let (u, v) = self.cells(i, j, "GlobalToLocalTransformation")?;
        let (scale_u, scale_v) = (u.inverse_scale(), v.inverse_scale());
        let shift = Vector2::new(
            u.first / scale_u - u.joint_first,
            v.first / scale_v - v.joint_first,
        );
        let u_fact = scale_u / scale_v;
        let (mut shift_trsf, mut scale_trsf) = (Transform2::default(), Transform2::default());
        if shift.x() != 0.0 || shift.y() != 0.0 {
            shift_trsf.set_translation(shift);
        }
        if scale_v != 1.0 {
            scale_trsf.set_scale(Point2::new(0.0, 0.0), scale_v);
        }
        let trsf = scale_trsf * shift_trsf;
        Ok((
            u_fact != 1.0 || trsf.form() != TransformForm::Identity,
            u_fact,
            trsf,
        ))
    }
}

impl Cell {
    /// `u * scale + (joint - u1 * scale)` with `scale = (b - a) / (u2 - u1)`: the order of
    /// the native formula, stable for an infinite `u1`.
    fn local_to_global(&self, local: f64) -> f64 {
        let scale = (self.joint_last - self.joint_first) / (self.last - self.first);
        local * scale + (self.joint_first - self.first * scale)
    }

    /// `U * scale + (u1 - joint * scale)` with `scale = (u2 - u1) / (b - a)`.
    fn global_to_local(&self, global: f64) -> f64 {
        let scale = self.inverse_scale();
        global * scale + (self.first - self.joint_first * scale)
    }

    /// `(u2 - u1) / (b - a)`.
    fn inverse_scale(&self) -> f64 {
        (self.last - self.first) / (self.joint_last - self.joint_first)
    }
}
