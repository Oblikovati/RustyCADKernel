//! Float pixel accumulation with double extrema and native deferred normalization.
use super::*;
use rustycad_tkernel::{quantity::ColorRgba, standard::Failure};
use rustycad_tkservice::image::{ImageFormat, ImagePixMap};
pub(super) struct SelectionImageDepth {
    unnormalized: ImagePixMap,
    minimum: f64,
    maximum: f64,
    inverted: bool,
}
impl SelectionImageDepth {
    pub fn new(image: &ImagePixMap, inverted: bool) -> Self {
        let unnormalized = ImagePixMap::default();
        unnormalized.init_zero(ImageFormat::GrayF, image.width(), image.height(), 0, 0);
        Self {
            unnormalized,
            minimum: f64::MAX,
            maximum: -f64::MAX,
            inverted,
        }
    }
    pub fn fill<S: SelectMgrViewerSelectorSource + ?Sized>(
        &mut self,
        selector: &S,
        col: i32,
        row: i32,
        rank: i32,
    ) -> Result<(), Failure> {
        if self.unnormalized.is_empty() {
            return Ok(());
        }
        let value = self.picked_depth(selector, rank)?;
        if let Some(pixel) = self
            .unnormalized
            .change_data()
            .data
            .change_value_xy(col as usize, row as usize)
        {
            pixel.copy_from_slice(&value.to_ne_bytes());
        }
        Ok(())
    }
    fn picked_depth<S: SelectMgrViewerSelectorSource + ?Sized>(
        &mut self,
        selector: &S,
        rank: i32,
    ) -> Result<f32, Failure> {
        if rank < 1 || rank > selector.nb_picked() {
            return Ok(f32::MAX);
        }
        let depth = selector.picked_data(rank)?.depth;
        if depth < self.minimum {
            self.minimum = depth;
        }
        if depth > self.maximum {
            self.maximum = depth;
        }
        Ok(depth as f32)
    }
    fn normalization(&self) -> (f32, f32) {
        if self.minimum > self.maximum {
            return (0., 1.);
        }
        let from = self.minimum as f32;
        let delta = self.maximum as f32 - from;
        (from, if delta <= f32::EPSILON { 1. } else { delta })
    }
    pub fn flush(&self, image: &ImagePixMap) -> Result<(), Failure> {
        let (from, delta) = self.normalization();
        for row in 0..self.unnormalized.height() {
            for col in 0..self.unnormalized.width() {
                self.flush_pixel(image, col, row, from, delta)?;
            }
        }
        Ok(())
    }
    fn flush_pixel(
        &self,
        image: &ImagePixMap,
        col: usize,
        row: usize,
        from: f32,
        delta: f32,
    ) -> Result<(), Failure> {
        let stored = self.unnormalized.data();
        let pixel = stored
            .data
            .value_xy(col, row)
            .expect("normalized image pixel must exist");
        let depth = f32::from_ne_bytes(pixel.try_into().expect("GrayF pixel must have four bytes"));
        let value = self.normalized_pixel(depth, from, delta);
        let color = ColorRgba::new(value, value, value, 1.)?;
        drop(stored);
        image.set_pixel_color_rgba(col as i32, row as i32, &color, false);
        Ok(())
    }
    fn normalized_pixel(&self, depth: f32, from: f32, delta: f32) -> f32 {
        // Missing picks stay black even when the depth range is inverted.
        if depth <= -f32::MAX || depth >= f32::MAX {
            return 0.;
        }
        let normalized = (depth - from) / delta;
        if self.inverted {
            1. - normalized
        } else {
            normalized
        }
    }
}
