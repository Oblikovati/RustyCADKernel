//! Const buffer references expose genuine array fields without temporary intrusive ownership.
use super::{
    Graphic3dArrayOfPrimitives, Graphic3dBoundBuffer, Graphic3dBufferSource,
    Graphic3dIndexBufferSource,
};
use rustycad_tkernel::standard::Handle;
impl Graphic3dArrayOfPrimitives {
    /// Borrow the actual vertex-buffer field; `array.attributes_ref().ref_count()`.
    pub fn attributes_ref(&self) -> &Handle<dyn Graphic3dBufferSource> {
        // Driver callbacks observe the actual member count, without an owning getter temporary.
        &self.attribs
    }
    /// Borrow the actual index-buffer field, including null; `array.indices_ref().is_null()`.
    pub fn indices_ref(&self) -> &Handle<dyn Graphic3dIndexBufferSource> {
        // Driver callbacks observe the actual member count, without an owning getter temporary.
        &self.indices
    }
    /// Borrow the actual bound-buffer field; `array.bounds_ref().ref_count()`.
    pub fn bounds_ref(&self) -> &Handle<Graphic3dBoundBuffer> {
        // Driver callbacks observe the actual member count, without an owning getter temporary.
        &self.bounds
    }
}
