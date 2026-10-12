//! Native sampler readiness follows parameter identity/revision; UV transforms remain independently live.
use crate::{
    RenderError,
    error::rejected,
    surface_texture::{SurfaceSamplerPolicy, SurfaceTextureImage},
};
use rustycad_tkernel::standard::{Handle, WeakHandle};
use rustycad_tkservice::graphic3d::Graphic3dTextureParams;
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex, Weak},
};
struct TextureSamplerSnapshot {
    parameters: WeakHandle<Graphic3dTextureParams>,
    revision: u32,
    image: Weak<SurfaceTextureImage>,
    policy: SurfaceSamplerPolicy,
}
#[derive(Default)]
pub(crate) struct RetainedTextureSamplers {
    snapshots: Mutex<BTreeMap<Vec<u8>, TextureSamplerSnapshot>>,
}
impl RetainedTextureSamplers {
    pub fn prune(&self) -> Result<(), RenderError> {
        self.snapshots
            .lock()
            .map_err(|error| rejected(error.to_string(), "unpoisoned texture sampler cache"))?
            .retain(|_, snapshot| {
                !snapshot.parameters.upgrade().is_null() && snapshot.image.strong_count() > 0
            });
        Ok(())
    }
    pub fn bound_parameters(
        &self,
        image: &Arc<SurfaceTextureImage>,
        fallback: &Handle<Graphic3dTextureParams>,
    ) -> Result<Handle<Graphic3dTextureParams>, RenderError> {
        let snapshots = self
            .snapshots
            .lock()
            .map_err(|e| rejected(e.to_string(), "unpoisoned texture sampler cache"))?;
        Ok(snapshots
            .get(&image.id)
            .map(|snapshot| snapshot.parameters.upgrade())
            .filter(|parameters| !parameters.is_null())
            .unwrap_or_else(|| fallback.clone()))
    }
    pub fn policy(
        &self,
        image: &Arc<SurfaceTextureImage>,
        parameters: &Handle<Graphic3dTextureParams>,
    ) -> Result<SurfaceSamplerPolicy, RenderError> {
        let mut snapshots = self
            .snapshots
            .lock()
            .map_err(|error| rejected(error.to_string(), "unpoisoned texture sampler cache"))?;
        if let Some(snapshot) = snapshots
            .get(&image.id)
            .filter(|old| old.matches(image, parameters))
        {
            return Ok(snapshot.policy.clone());
        }
        let snapshot = TextureSamplerSnapshot::new(image, parameters);
        let policy = snapshot.policy.clone();
        if !image.id.is_empty() {
            snapshots.insert(image.id.clone(), snapshot);
        }
        Ok(policy)
    }
}
impl TextureSamplerSnapshot {
    fn new(image: &Arc<SurfaceTextureImage>, parameters: &Handle<Graphic3dTextureParams>) -> Self {
        Self {
            parameters: parameters.downgrade(),
            revision: parameters.sampler_revision(),
            image: Arc::downgrade(image),
            policy: SurfaceSamplerPolicy::read(parameters),
        }
    }
    fn matches(
        &self,
        image: &Arc<SurfaceTextureImage>,
        parameters: &Handle<Graphic3dTextureParams>,
    ) -> bool {
        !image.id.is_empty()
            && self.parameters == parameters.downgrade()
            && self.revision == parameters.sampler_revision()
            && self
                .image
                .upgrade()
                .is_some_and(|old| Arc::ptr_eq(&old, image))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::surface_texture::SurfaceImageEncoding;
    use rustycad_tkservice::graphic3d::Graphic3dTypeOfTextureFilter;
    fn sampler_image() -> Arc<SurfaceTextureImage> {
        Arc::new(SurfaceTextureImage {
            lifetime: Arc::default(),
            id: vec![1],
            revision: 0,
            extent: [4, 4],
            encoding: SurfaceImageEncoding::Linear8,
            alpha_only: false,
            bytes: Arc::new(vec![0; 64]),
            top_down: true,
            mipmaps: true,
            supplied_levels: vec![],
        })
    }
    #[test]
    fn level_range_changes_wait_for_native_sampler_revision_and_new_resource_identity() {
        let image = sampler_image();
        let parameters = Handle::new(Graphic3dTextureParams::default());
        let cache = RetainedTextureSamplers::default();
        assert_eq!(cache.policy(&image, &parameters).unwrap().first_level, 0);
        parameters.set_levels_range(1, 1000);
        assert_eq!(cache.policy(&image, &parameters).unwrap().first_level, 0);
        parameters.set_filter(Graphic3dTypeOfTextureFilter::Bilinear);
        assert_eq!(cache.policy(&image, &parameters).unwrap().first_level, 1);
        parameters.set_levels_range(2, 1000);
        assert_eq!(
            cache
                .policy(&sampler_image(), &parameters)
                .unwrap()
                .first_level,
            2
        );
    }
    #[test]
    fn switching_parameter_handles_reinitializes_and_does_not_retain_owners() {
        let image = sampler_image();
        let first = Handle::new(Graphic3dTextureParams::default());
        let second = Handle::new(Graphic3dTextureParams::default());
        let cache = RetainedTextureSamplers::default();
        cache.policy(&image, &first).unwrap();
        first.set_levels_range(1, 1000);
        cache.policy(&image, &second).unwrap();
        assert_eq!(cache.policy(&image, &first).unwrap().first_level, 1);
        drop(first);
        drop(second);
        cache.prune().unwrap();
        assert!(cache.snapshots.lock().unwrap().is_empty());
    }
}
