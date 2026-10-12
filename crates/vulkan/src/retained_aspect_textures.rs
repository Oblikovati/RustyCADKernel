//! OCCT3d097a0 OpenGl_AspectsTextureSet checks image revisions during aspect synchronization.
use crate::{
    RenderError, retained_texture_images::RetainedTextureImages,
    surface_texture::SurfaceTextureImage,
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::graphic3d::*;
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};
type AspectImageBindings = BTreeMap<usize, AspectImageBinding>;
struct AspectImageBinding {
    resource_id: Vec<u8>,
    anonymous: Option<Arc<SurfaceTextureImage>>,
}
#[derive(Default)]
pub(crate) struct RetainedAspectTextures {
    bindings: Mutex<AspectImageBindings>,
}
impl RetainedAspectTextures {
    pub fn invalidate(&self, aspect: &Handle<dyn Graphic3dAspectsSource>) {
        self.bindings
            .lock()
            .expect("unpoisoned aspect texture readiness")
            .remove(&(aspect.object_address() as usize));
    }
    pub fn synchronize(&self) {
        self.bindings
            .lock()
            .expect("unpoisoned aspect texture readiness")
            .clear();
    }
    pub fn snapshot(
        &self,
        aspects: &Handle<dyn Graphic3dAspectsSource>,
        images: &RetainedTextureImages,
    ) -> Result<Option<Arc<SurfaceTextureImage>>, RenderError> {
        let mut bindings = self.bindings.lock().map_err(|e| {
            crate::error::rejected(e.to_string(), "unpoisoned aspect texture readiness")
        })?;
        let key = aspects.object_address() as usize;
        if let Some(binding) = bindings.get(&key) {
            return binding.current(images);
        }
        let texture = aspects.texture_map();
        let image = images.snapshot(&texture)?;
        let resource_id = texture.get_id().as_bytes().to_vec();
        let anonymous = resource_id.is_empty().then(|| image.clone()).flatten();
        bindings.insert(
            key,
            AspectImageBinding {
                resource_id,
                anonymous,
            },
        );
        Ok(image)
    }
}

impl AspectImageBinding {
    fn current(
        &self,
        images: &RetainedTextureImages,
    ) -> Result<Option<Arc<SurfaceTextureImage>>, RenderError> {
        if self.resource_id.is_empty() {
            return Ok(self.anonymous.clone());
        }
        images.bound_snapshot(&self.resource_id)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use rustycad_tkservice::image::{ImageFormat, ImagePixMap};
    fn readiness_image(red: u8) -> Handle<ImagePixMap> {
        let image = Handle::new(ImagePixMap::default());
        assert!(image.init_wrapper(ImageFormat::Rgba, Some(vec![red, 0, 0, 255]), 1, 1, 4));
        image.set_top_down(true);
        image
    }
    #[test]
    fn native_ready_aspect_ignores_content_revision_until_synchronization() {
        let images = RetainedTextureImages::default();
        let readiness = RetainedAspectTextures::default();
        let texture = Handle::new(Graphic3dTexture2D::from_pix_map(readiness_image(255)));
        let aspect = Handle::new(Graphic3dAspects::default());
        aspect.set_texture_map(texture.cast());
        let first = readiness
            .snapshot(&aspect.cast(), &images)
            .unwrap()
            .unwrap();
        texture.set_image(readiness_image(0));
        texture.update_revision();
        assert!(Arc::ptr_eq(
            &first,
            &readiness
                .snapshot(&aspect.cast(), &images)
                .unwrap()
                .unwrap()
        ));
        readiness.synchronize();
        let updated = readiness
            .snapshot(&aspect.cast(), &images)
            .unwrap()
            .unwrap();
        assert_eq!(first.bytes.as_slice(), [255, 0, 0, 255]);
        assert_eq!(updated.bytes.as_slice(), [0, 0, 0, 255]);
    }
    #[test]
    fn another_actual_binding_refreshes_the_shared_resource_and_explicit_aspect_rebinds() {
        let images = RetainedTextureImages::default();
        let readiness = RetainedAspectTextures::default();
        let texture = Handle::new(Graphic3dTexture2D::from_pix_map(readiness_image(255)));
        let aspect = Handle::new(Graphic3dAspects::default());
        aspect.set_texture_map(texture.cast());
        readiness.snapshot(&aspect.cast(), &images).unwrap();
        texture.set_image(readiness_image(64));
        texture.update_revision();
        let shared = images.snapshot(&texture.cast()).unwrap().unwrap();
        assert!(Arc::ptr_eq(
            &shared,
            &readiness
                .snapshot(&aspect.cast(), &images)
                .unwrap()
                .unwrap()
        ));
        readiness.invalidate(&aspect.cast());
        assert!(Arc::ptr_eq(
            &shared,
            &readiness
                .snapshot(&aspect.cast(), &images)
                .unwrap()
                .unwrap()
        ));
    }
}
