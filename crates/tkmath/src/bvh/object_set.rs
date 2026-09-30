// Rust translation of FoundationClasses/TKMath/BVH/BVH_ObjectSet.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BVH_ObjectSet<T, N>`: an array of geometric objects to build a BVH over.
use super::bounding_box::BvhBox;
use super::object::BvhObject;
use super::set::BvhSet;
use super::vector::BvhVector;
use rustycad_tkernel::ncollection::{ArrayError, DynamicArray};
use rustycad_tkernel::standard::Failure;

/// A handle to a geometric object; `None` is a null handle.
///
/// The set owns its objects: upstream shares them through reference-counted
/// handles, which would need shared mutable state here.
pub type BvhObjectHandle<V> = Option<Box<dyn BvhObject<V> + Send>>;

/// `BVH_ObjectList`: the dynamic array of object handles.
pub type BvhObjectList<V> = DynamicArray<BvhObjectHandle<V>>;

/// Set of abstract geometric objects; each element is one object's box.
/// Example: `set.objects_mut().append(Some(Box::new(triangulation)));`.
pub struct BvhObjectSet<V: BvhVector> {
    objects: BvhObjectList<V>,
}
impl<V: BvhVector> Default for BvhObjectSet<V> {
    fn default() -> Self {
        Self {
            objects: DynamicArray::default(),
        }
    }
}
impl<V: BvhVector> std::fmt::Debug for BvhObjectSet<V> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("BvhObjectSet")
            .field("size", &self.objects.len())
            .finish()
    }
}

/// `NCollection_DynamicArray::Value` raising `Standard_OutOfRange`.
fn object_range_failure(error: ArrayError, index: i32) -> Failure {
    Failure::out_of_range(&format!("BVH_ObjectSet object index {index}: {error:?}"))
}

/// Dereferencing a null handle; upstream crashes, the port reports it.
fn null_object_failure(index: i32) -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        &format!("BVH_ObjectSet object {index} is a null handle, expected an object"),
        "",
    )
}

impl<V: BvhVector> BvhObjectSet<V> {
    /// Releases every object, then empties the array; `set.clear()`.
    pub fn clear(&mut self) {
        for object in self.objects.iter_mut() {
            *object = None;
        }
        self.objects.clear(false);
    }
    /// Returns the array of geometric objects; `set.objects().len()`.
    pub fn objects(&self) -> &BvhObjectList<V> {
        &self.objects
    }
    /// Returns the mutable array of geometric objects; `set.objects_mut()`.
    pub fn objects_mut(&mut self) -> &mut BvhObjectList<V> {
        &mut self.objects
    }
    /// Borrows the object at `index`, failing for a bad index or a null handle.
    /// Example: `set.object(0)?.object_box()?`.
    pub fn object(&self, index: i32) -> Result<&(dyn BvhObject<V> + Send + 'static), Failure> {
        let handle = self
            .objects
            .value(index)
            .map_err(|error| object_range_failure(error, index))?;
        handle.as_deref().ok_or_else(|| null_object_failure(index))
    }
    /// Mutably borrows the object at `index`; `set.object_mut(0)?.mark_dirty()`.
    pub fn object_mut(
        &mut self,
        index: i32,
    ) -> Result<&mut (dyn BvhObject<V> + Send + 'static), Failure> {
        let handle = self
            .objects
            .change_value(index)
            .map_err(|error| object_range_failure(error, index))?;
        handle
            .as_deref_mut()
            .ok_or_else(|| null_object_failure(index))
    }
}

impl<V: BvhVector> BvhSet<V> for BvhObjectSet<V> {
    /// Returns total number of objects.
    fn size(&self) -> i32 {
        self.objects.length()
    }
    /// Returns the box of the object, `myObjects.Value(index)->Box()`.
    fn element_box(&self, index: i32) -> Result<BvhBox<V>, Failure> {
        self.object(index)?.object_box()
    }
    /// Returns the object box center along `axis` (the general, not optimal, way).
    fn center(&self, index: i32, axis: i32) -> Result<V::Scalar, Failure> {
        Ok(self.element_box(index)?.center_axis(axis))
    }
    /// Swaps the two object handles; swapping a slot with itself changes nothing.
    fn swap(&mut self, first: i32, second: i32) -> Result<(), Failure> {
        for index in [first, second] {
            self.objects
                .value(index)
                .map_err(|error| object_range_failure(error, index))?;
        }
        if first == second {
            return Ok(());
        }
        let range = |error| object_range_failure(error, first);
        let taken = self.objects.change_value(first).map_err(range)?.take();
        let displaced = std::mem::replace(self.objects.change_value(second).map_err(range)?, taken);
        *self.objects.change_value(first).map_err(range)? = displaced;
        Ok(())
    }
}
