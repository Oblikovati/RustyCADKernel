//! Named texture I/O fake returns genuine Graphic3d handles without filesystem access.
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkservice::{graphic3d::*, image::ImagePixMap};
use rustycad_tkv3d::ais::*;
use std::sync::{
    Mutex,
    atomic::{AtomicUsize, Ordering},
};

pub struct TexturedTextureFactoryFixture {
    pub texture: Handle<dyn Graphic3dTexture2DSource>,
    pub calls: AtomicUsize,
    pub sources: Mutex<Vec<AisTexturedTextureSource>>,
    pub construction_failure: bool,
    pub done: bool,
}
impl TexturedTextureFactoryFixture {
    pub fn new(construction_failure: bool, done: bool) -> Self {
        Self {
            texture: Handle::new(Graphic3dTexture2D::from_pix_map(Handle::new(
                ImagePixMap::default(),
            )))
            .cast(),
            calls: AtomicUsize::new(0),
            sources: Mutex::new(Vec::new()),
            construction_failure,
            done,
        }
    }
}
impl AisTexturedTextureFactory for TexturedTextureFactoryFixture {
    fn create(
        &self,
        source: AisTexturedTextureSource,
    ) -> Result<Handle<dyn Graphic3dTexture2DSource>, Failure> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        self.sources.lock().unwrap().push(source);
        if self.construction_failure {
            return Err(Failure::new(
                "texture construction failed; expected an available injected source",
            ));
        }
        Ok(self.texture.clone())
    }
    fn is_done(&self, _: &Handle<dyn Graphic3dTexture2DSource>) -> bool {
        self.done
    }
}
