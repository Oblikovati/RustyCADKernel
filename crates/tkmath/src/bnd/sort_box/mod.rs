// Rust translation of FoundationClasses/TKMath/Bnd/Bnd_BoundSortBox.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Bnd_BoundSortBox`: finds, among a fixed set of boxes, those touching a query box or plane.
use super::BoundingBox3;
use crate::gp::Plane3;
use rustycad_tkernel::ncollection::{Array1, List};
use rustycad_tkernel::standard::Failure;
use voxel_grid::{VoxelBox, VoxelGrid};

mod compare;
mod voxel_grid;

/// Voxel grid over the enclosing box plus a list of "large" boxes tested directly.
///
/// The native class shares the caller's `NCollection_HArray1` handle; here the array is
/// owned and exposed through `boxes()`, so writes made by `add` stay observable.
/// Example: `let mut sorter = BoundSortBox::default(); sorter.initialize(boxes);`.
#[derive(Clone, Default)]
pub struct BoundSortBox {
    enclosing: BoundingBox3,
    boxes: Option<Array1<'static, BoundingBox3>>,
    coefficients: [f64; 3],
    resolution: i32,
    last_result: List<i32>,
    large_boxes: Vec<i32>,
    voxel_grid: Option<VoxelGrid>,
}

/// `getBnd_VoxelGridResolution`: voxels per axis from the number of boxes.
fn grid_resolution(box_count: usize) -> i32 {
    match box_count {
        count if count > 40000 => 128,
        count if count > 10000 => 64,
        count if count > 1000 => 32,
        count if count > 100 => 16,
        _ => 8,
    }
}

/// `static_cast<int>(double)` as compiled for x86-64 (`cvttsd2si`): truncation toward zero,
/// NaN and out-of-range values (undefined behavior in ISO C++) giving `INT_MIN`.
fn native_int_cast(value: f64) -> i32 {
    if value.is_nan() || !(-2_147_483_648.0..2_147_483_648.0).contains(&value) {
        return i32::MIN;
    }
    value as i32
}

impl BoundSortBox {
    /// `Initialize(theSetOfBoxes)`: adds every nonvoid box to the enclosing box (which is
    /// not reset, so repeated calls keep growing it), then sorts the boxes.
    /// Example: `sorter.initialize(boxes)?`.
    pub fn initialize(&mut self, boxes: Array1<'static, BoundingBox3>) -> Result<(), Failure> {
        for bounds in boxes.iter() {
            if !bounds.is_void() {
                self.enclosing.add_box(bounds);
            }
        }
        self.resolution = grid_resolution(boxes.len());
        self.boxes = Some(boxes);
        self.prepare_and_sort()
    }
    /// `Initialize(theEnclosingBox, theSetOfBoxes)`: uses the given enclosing box as is.
    /// Example: `sorter.initialize_with_enclosing(&global, boxes)?`.
    pub fn initialize_with_enclosing(
        &mut self,
        enclosing: &BoundingBox3,
        boxes: Array1<'static, BoundingBox3>,
    ) -> Result<(), Failure> {
        self.enclosing = enclosing.clone();
        self.resolution = grid_resolution(boxes.len());
        self.boxes = Some(boxes);
        self.prepare_and_sort()
    }
    /// With a void enclosing box the grid, coefficients and large-box list keep their
    /// previous state, as natively.
    fn prepare_and_sort(&mut self) -> Result<(), Failure> {
        if self.enclosing.is_void() {
            return Ok(());
        }
        self.calculate_coefficients();
        self.reset_voxel_grid();
        self.sort_boxes()
    }
    /// `Initialize(theEnclosingBox, theNbBoxes)`: allocates `count` void boxes indexed from 1
    /// to be filled by `add`; raises `Standard_NullValue` for `count <= 0`.
    /// Example: `sorter.initialize_with_count(&global, 3)?`.
    pub fn initialize_with_count(
        &mut self,
        enclosing: &BoundingBox3,
        count: i32,
    ) -> Result<(), Failure> {
        if count <= 0 {
            return Err(Failure::with_type(
                "Standard_NullValue",
                &format!("Unexpected: theNbBoxes <= 0 (got {count}, expected > 0)"),
                "",
            ));
        }
        let boxes =
            Array1::with_bounds(1, count).map_err(|error| range_failure("Initialize", error))?;
        self.boxes = Some(boxes);
        self.enclosing = enclosing.clone();
        self.resolution = grid_resolution(count as usize);
        if !self.enclosing.is_void() {
            self.calculate_coefficients();
            self.reset_voxel_grid();
        }
        Ok(())
    }
    /// `Add(theBox, theIndex)`: stores a box at a 1-based slot that must still be void
    /// (`Standard_MultiplyDefined` otherwise); a void box is ignored after that check.
    /// Example: `sorter.add(&bounds, 1)?`.
    pub fn add(&mut self, bounds: &BoundingBox3, index: i32) -> Result<(), Failure> {
        let boxes = self.boxes.as_mut().ok_or_else(uninitialized)?;
        let slot = boxes
            .change_value(index)
            .map_err(|error| range_failure("Add", error))?;
        if !slot.is_void() {
            return Err(Failure::with_type(
                "Standard_MultiplyDefined",
                &format!(" This box is already defined ! (index {index})"),
                "",
            ));
        }
        if bounds.is_void() {
            return Ok(());
        }
        *slot = bounds.clone();
        self.add_to_grid(bounds, index)
    }
    /// `Compare(gp_Pln)`: every box, in index order, that the plane does not miss; before any
    /// initialization the (native null) box array is treated as empty.
    /// Example: `sorter.compare_plane(plane)?`.
    pub fn compare_plane(&mut self, plane: Plane3) -> Result<&List<i32>, Failure> {
        self.last_result.clear();
        let Some(boxes) = self.boxes.as_ref() else {
            return Ok(&self.last_result);
        };
        for index in boxes.lower()..=boxes.upper() {
            let bounds = boxes
                .value(index)
                .map_err(|error| range_failure("Compare", error))?;
            if !bounds.is_out_plane(plane)? {
                self.last_result.append(index);
            }
        }
        Ok(&self.last_result)
    }
    /// The sorted boxes (the native shared array), or `None` before initialization;
    /// `sorter.boxes()`.
    pub fn boxes(&self) -> Option<&Array1<'static, BoundingBox3>> {
        self.boxes.as_ref()
    }
    fn calculate_coefficients(&mut self) {
        let limits = self.enclosing.limits();
        let spans = [
            limits.x_max - limits.x_min,
            limits.y_max - limits.y_min,
            limits.z_max - limits.z_min,
        ];
        self.coefficients = spans.map(|span| {
            if span == 0.0 {
                0.0
            } else {
                f64::from(self.resolution) / span
            }
        });
    }
    fn reset_voxel_grid(&mut self) {
        self.voxel_grid = Some(VoxelGrid::new(self.resolution));
        self.large_boxes.clear();
    }
    fn sort_boxes(&mut self) -> Result<(), Failure> {
        let Some(boxes) = self.boxes.take() else {
            return Ok(());
        };
        let sorted = (boxes.lower()..=boxes.upper()).try_for_each(|index| {
            let bounds = boxes
                .value(index)
                .map_err(|error| range_failure("Initialize", error))?;
            self.add_to_grid(bounds, index)
        });
        self.boxes = Some(boxes);
        sorted
    }
    /// `getBoundingVoxels`: voxel range of a nonvoid box, widened by one voxel each way and
    /// clamped to the grid; the enclosing corner raises `Standard_ConstructionError` if void.
    fn bounding_voxels(&self, bounds: &BoundingBox3) -> Result<VoxelBox, Failure> {
        let start = self.enclosing.corner_min()?;
        let [x_min, y_min, z_min, x_max, y_max, z_max] = bounds.get()?;
        let high = self.resolution - 1;
        let voxel = |value: f64, origin: f64, axis: usize| {
            native_int_cast((value - origin) * self.coefficients[axis])
        };
        Ok([
            voxel(x_min, start.x(), 0).wrapping_sub(1).clamp(0, high),
            voxel(y_min, start.y(), 1).wrapping_sub(1).clamp(0, high),
            voxel(z_min, start.z(), 2).wrapping_sub(1).clamp(0, high),
            voxel(x_max, start.x(), 0).wrapping_add(1).clamp(0, high),
            voxel(y_max, start.y(), 1).wrapping_add(1).clamp(0, high),
            voxel(z_max, start.z(), 2).wrapping_add(1).clamp(0, high),
        ])
    }
    /// `addBox`: boxes spanning more than a quarter of the grid on every axis are kept in the
    /// large-box list, the others in the voxel grid.
    fn add_to_grid(&mut self, bounds: &BoundingBox3, index: i32) -> Result<(), Failure> {
        if bounds.is_void() {
            return Ok(());
        }
        let voxels = self.bounding_voxels(bounds)?;
        let min_side = (0..3)
            .map(|axis| voxels[axis + 3] - voxels[axis])
            .min()
            .unwrap_or(0);
        if min_side * 4 > self.resolution {
            self.large_boxes.push(index);
            return Ok(());
        }
        let grid = self.voxel_grid.as_mut().ok_or_else(uninitialized)?;
        grid.add_box(index, voxels);
        Ok(())
    }
}

/// The native code dereferences a null handle here (no box array or no voxel grid yet).
fn uninitialized() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "Bnd_BoundSortBox: used before Initialize (expected an initialized box array and grid)",
        "",
    )
}

fn range_failure(operation: &str, error: impl std::fmt::Display) -> Failure {
    Failure::with_type(
        "Standard_OutOfRange",
        &format!("Bnd_BoundSortBox::{operation}: {error}"),
        "",
    )
}
