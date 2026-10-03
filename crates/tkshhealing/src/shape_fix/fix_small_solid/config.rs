use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tktopalgo::brep_gprop::BRepGProp;
use std::cmp::Ordering;

#[derive(Clone)]
pub(super) struct SmallSolidConfig {
    pub(super) mode: i32,
    pub(super) volume_threshold: f64,
    pub(super) width_factor_threshold: f64,
}

impl Default for SmallSolidConfig {
    fn default() -> Self {
        Self {
            mode: 0,
            volume_threshold: precision::infinite(),
            width_factor_threshold: precision::infinite(),
        }
    }
}

impl SmallSolidConfig {
    pub(super) fn has_threshold(&self) -> bool {
        self.uses_volume() && self.volume_threshold < precision::infinite()
            || self.uses_width_factor() && self.width_factor_threshold < precision::infinite()
    }

    pub(super) fn is_small(&self, solid: &Shape) -> Result<bool, Failure> {
        let volume = volume(solid)?;
        if self.uses_volume() && volume > self.volume_threshold {
            return Ok(false);
        }
        self.within_width_factor(solid, volume)
    }

    fn within_width_factor(&self, solid: &Shape, volume: f64) -> Result<bool, Failure> {
        if !self.uses_width_factor() || self.width_factor_threshold >= precision::infinite() {
            return Ok(true);
        }
        let area = surface_area(solid)?;
        let limit = self.width_factor_threshold * area * 0.5;
        Ok(volume.partial_cmp(&limit) != Some(Ordering::Greater))
    }

    fn uses_width_factor(&self) -> bool {
        self.mode == 0 || self.mode == 1
    }

    fn uses_volume(&self) -> bool {
        self.mode == 0 || self.mode == 2
    }
}

pub(super) fn surface_area(shape: &Shape) -> Result<f64, Failure> {
    let mut properties = GProps::new();
    BRepGProp::surface_properties(shape, &mut properties, false, false)?;
    Ok(properties.mass())
}

fn volume(shape: &Shape) -> Result<f64, Failure> {
    let mut properties = GProps::new();
    BRepGProp::volume_properties(shape, &mut properties, false, false, false)?;
    Ok(properties.mass())
}
