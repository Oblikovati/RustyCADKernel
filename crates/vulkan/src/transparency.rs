//! Native transparency parameters stay independent of Vulkan resource allocation.
use crate::{RenderError, error::rejected};
use rustycad_tkservice::graphic3d::{Graphic3dRenderTransparentMethod, Graphic3dRenderingParams};

/// Select the actual transparency algorithm; `RenderTransparency::Weighted { depth_factor: 0. }`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum RenderTransparency {
    /// Preserve insertion-order source-alpha blending.
    #[default]
    Unordered,
    /// Accumulate native weighted coverage and revealage.
    Weighted {
        /// Scale fragment depth in the native weight; for example, `depth_factor: 0.5`.
        depth_factor: f32,
    },
    /// Execute the native initial min/max pass and the requested dual-peeling iterations.
    DepthPeeling {
        /// Number of dual-peeling iterations; for example, `layers: 4`.
        layers: u32,
    },
}
impl RenderTransparency {
    pub(crate) fn from_parameters(params: &Graphic3dRenderingParams) -> Result<Self, RenderError> {
        let policy = match params.transparency_method {
            Graphic3dRenderTransparentMethod::BlendUnordered => Self::Unordered,
            Graphic3dRenderTransparentMethod::BlendOit => Self::Weighted {
                depth_factor: params.oit_depth_factor,
            },
            Graphic3dRenderTransparentMethod::DepthPeelingOit => Self::DepthPeeling {
                layers: positive_peeling_layers(params.nb_oit_depth_peeling_layers)?,
            },
        };
        policy.validate()?;
        Ok(policy)
    }
    pub(crate) fn validate(self) -> Result<(), RenderError> {
        match self {
            Self::Weighted { depth_factor } if !depth_factor.is_finite() => {
                Err(rejected(depth_factor, "a finite weighted OIT depth factor"))
            }
            Self::DepthPeeling { layers: 0 } => {
                Err(rejected(0, "a positive dual depth-peeling iteration count"))
            }
            _ => Ok(()),
        }
    }
}
fn positive_peeling_layers(layers: i32) -> Result<u32, RenderError> {
    u32::try_from(layers)
        .ok()
        .filter(|count| *count > 0)
        .ok_or_else(|| rejected(layers, "a positive dual depth-peeling iteration count"))
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct RenderRasterSettings {
    pub transparency: RenderTransparency,
    pub requested_samples: i32,
    pub alpha_to_coverage: bool,
    pub fixed_function: bool,
    pub disabled_capabilities: u8,
}
impl RenderRasterSettings {
    pub fn from_parameters(
        params: &Graphic3dRenderingParams,
        fixed_function: bool,
    ) -> Result<Self, RenderError> {
        Ok(Self {
            transparency: RenderTransparency::from_parameters(params)?,
            requested_samples: params.nb_msaa_samples,
            alpha_to_coverage: params.to_enable_alpha_to_coverage,
            fixed_function,
            disabled_capabilities: 0,
        })
    }
}
impl crate::RenderFrame {
    /// Choose a low-level frame algorithm; `frame.with_transparency(RenderTransparency::Weighted { depth_factor: 0. })?`.
    pub fn with_transparency(
        mut self,
        transparency: RenderTransparency,
    ) -> Result<Self, RenderError> {
        transparency.validate()?;
        self.raster.transparency = transparency;
        Ok(self)
    }
    /// Read the retained frame algorithm; `frame.transparency()`.
    pub fn transparency(&self) -> RenderTransparency {
        self.raster.transparency
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_modes_keep_depth_factor_pass_count_and_sample_request_independent() {
        let mut params = Graphic3dRenderingParams::new().unwrap();
        params.transparency_method = Graphic3dRenderTransparentMethod::BlendOit;
        params.oit_depth_factor = 0.75;
        params.nb_msaa_samples = 3;
        let raster = RenderRasterSettings::from_parameters(&params, true).unwrap();
        assert_eq!(
            raster.transparency,
            RenderTransparency::Weighted { depth_factor: 0.75 }
        );
        assert_eq!(raster.requested_samples, 3);
        assert!(raster.fixed_function && raster.alpha_to_coverage);
        params.transparency_method = Graphic3dRenderTransparentMethod::DepthPeelingOit;
        params.nb_oit_depth_peeling_layers = 7;
        assert_eq!(
            RenderTransparency::from_parameters(&params).unwrap(),
            RenderTransparency::DepthPeeling { layers: 7 }
        );
    }
    #[test]
    fn frame_policy_is_independent_and_rejects_invalid_replacement() {
        let original = crate::RenderFrame::new(2, 2, [0.; 4], vec![]).unwrap();
        let weighted = original
            .clone()
            .with_transparency(RenderTransparency::Weighted { depth_factor: 0.5 })
            .unwrap();
        assert_eq!(original.transparency(), RenderTransparency::Unordered);
        assert_eq!(
            weighted.transparency(),
            RenderTransparency::Weighted { depth_factor: 0.5 }
        );
        assert!(
            original
                .with_transparency(RenderTransparency::DepthPeeling { layers: 0 })
                .is_err()
        );
    }
    #[test]
    fn invalid_active_parameters_report_offending_value_and_expected_shape() {
        assert!(
            RenderTransparency::Weighted {
                depth_factor: f32::NAN
            }
            .validate()
            .unwrap_err()
            .to_string()
            .contains("finite")
        );
        let mut params = Graphic3dRenderingParams::new().unwrap();
        params.transparency_method = Graphic3dRenderTransparentMethod::DepthPeelingOit;
        params.nb_oit_depth_peeling_layers = -2;
        assert!(
            RenderTransparency::from_parameters(&params)
                .unwrap_err()
                .to_string()
                .contains("-2")
        );
        params.transparency_method = Graphic3dRenderTransparentMethod::BlendUnordered;
        assert_eq!(
            RenderTransparency::from_parameters(&params).unwrap(),
            RenderTransparency::Unordered
        );
    }
}
