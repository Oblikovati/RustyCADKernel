//! `Graphic3d_TextureSet`: the fixed-size array of texture maps bound together to a
//! primitive (one per texture unit).
use super::Graphic3dTextureMap;
use rustycad_tkernel::standard::exceptions::out_of_range;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};

/// A texture set over the native index range `0..size`.
///
/// Example: `let set = Graphic3dTextureSet::with_size(2); set.set_value(1, texture)?;`.
pub struct Graphic3dTextureSet {
    transient: StandardTransient,
    textures: SharedState<Vec<Handle<dyn Graphic3dTextureMap>>>,
}
rustycad_tkernel::impl_transient!(
    Graphic3dTextureSet,
    "Graphic3d_TextureSet",
    StandardTransient,
    transient
);

impl Default for Graphic3dTextureSet {
    /// An empty set.
    fn default() -> Self {
        Self {
            transient: StandardTransient::default(),
            textures: SharedState::new(Vec::new()),
        }
    }
}

impl Graphic3dTextureSet {
    /// A set of `count` null textures. A negative count makes the native
    /// `NCollection_Array1(0, count - 1)` allocate a wrapped `size_t` element count, which
    /// raises `Standard_OutOfMemory`; here it is that error with the count.
    ///
    /// Example: `Graphic3dTextureSet::with_size(2)?.size() == 2`.
    pub fn with_size(count: i32) -> Result<Self, Failure> {
        let length = usize::try_from(count).map_err(|_| {
            let message = format!("NCollection_Array1: allocation of a negative size; got {count} textures, expected >= 0");
            Failure::with_type("Standard_OutOfMemory", &message, "")
        })?;
        Ok(Self {
            transient: StandardTransient::default(),
            textures: SharedState::new(vec![Handle::default(); length]),
        })
    }
    /// A set of one texture; `Graphic3dTextureSet::with_texture(texture).size() == 1`.
    pub fn with_texture(texture: Handle<dyn Graphic3dTextureMap>) -> Self {
        Self {
            transient: StandardTransient::default(),
            textures: SharedState::new(vec![texture]),
        }
    }
    /// Whether the set is empty; `set.is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.textures.read().is_empty()
    }
    /// The number of textures; `set.size()`.
    pub fn size(&self) -> i32 {
        self.textures.read().len() as i32
    }
    /// The lower index (0); `set.lower()`.
    pub fn lower(&self) -> i32 {
        0
    }
    /// The upper index (`size - 1`); `set.upper()`.
    pub fn upper(&self) -> i32 {
        self.size() - 1
    }
    fn slot(&self, index: i32) -> Result<usize, Failure> {
        let size = self.size();
        usize::try_from(index)
            .ok()
            .filter(|slot| (*slot as i32) < size)
            .ok_or_else(|| {
                out_of_range(&format!(
                    "NCollection_Array1::Value: index out of range; got {index}, expected [0, {}]",
                    size - 1
                ))
            })
    }
    /// The first texture; `Standard_OutOfRange` for an empty set. `set.first()?`.
    pub fn first(&self) -> Result<Handle<dyn Graphic3dTextureMap>, Failure> {
        self.value(0)
    }
    /// Replaces the first texture; `set.set_first(texture)?`.
    pub fn set_first(&self, texture: Handle<dyn Graphic3dTextureMap>) -> Result<(), Failure> {
        self.set_value(0, texture)
    }
    /// The texture at `index`; `Standard_OutOfRange` outside `0..size`. `set.value(1)?`.
    pub fn value(&self, index: i32) -> Result<Handle<dyn Graphic3dTextureMap>, Failure> {
        let slot = self.slot(index)?;
        Ok(self.textures.read()[slot].clone())
    }
    /// Replaces the texture at `index`; `set.set_value(1, texture)?`.
    pub fn set_value(
        &self,
        index: i32,
        texture: Handle<dyn Graphic3dTextureMap>,
    ) -> Result<(), Failure> {
        let slot = self.slot(index)?;
        self.textures.write()[slot] = texture;
        Ok(())
    }
    /// The textures (`myTextures`); `set.textures()`.
    pub fn textures(&self) -> Vec<Handle<dyn Graphic3dTextureMap>> {
        self.textures.read().clone()
    }
    /// `Graphic3d_TextureSet::Iterator`: the textures in index order (none for a null set);
    /// `Graphic3dTextureSet::iter(&set)`.
    pub fn iter(
        set: &Handle<Graphic3dTextureSet>,
    ) -> std::vec::IntoIter<Handle<dyn Graphic3dTextureMap>> {
        set.get()
            .map(|set| set.textures.read().clone())
            .unwrap_or_default()
            .into_iter()
    }
}
