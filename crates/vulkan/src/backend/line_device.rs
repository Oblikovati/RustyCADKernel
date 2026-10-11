//! Optional line support must not exclude devices capable of the original triangle path.
use crate::{LineStyle, RenderError, error::rejected};
use ash::{Instance, vk};
use std::ffi::CStr;
#[derive(Clone, Copy, Default)]
pub(super) struct LineCapabilities {
    pub bresenham: bool,
    pub wide: bool,
}
impl LineCapabilities {
    pub fn query(instance: &Instance, physical: vk::PhysicalDevice) -> Result<Self, RenderError> {
        let extensions = unsafe { instance.enumerate_device_extension_properties(physical) }
            .map_err(|error| {
                super::runtime::vk_error("enumerate line rasterization support", error)
            })?;
        let present = extensions.iter().any(|extension| unsafe { CStr::from_ptr(extension.extension_name.as_ptr()) } == ash::ext::line_rasterization::NAME);
        let wide =
            unsafe { instance.get_physical_device_features(physical) }.wide_lines == vk::TRUE;
        if !present {
            return Ok(Self {
                bresenham: false,
                wide,
            });
        }
        let mut lines = vk::PhysicalDeviceLineRasterizationFeaturesEXT::default();
        let mut features = vk::PhysicalDeviceFeatures2::default().push_next(&mut lines);
        unsafe { instance.get_physical_device_features2(physical, &mut features) };
        Ok(Self {
            bresenham: lines.bresenham_lines == vk::TRUE,
            wide,
        })
    }
    pub fn validate(&self, style: LineStyle, range: [f32; 2]) -> Result<(), RenderError> {
        if !self.bresenham
            || (!self.wide && style.raster_width() != 1.)
            || !(range[0]..=range[1]).contains(&style.raster_width())
        {
            return Err(rejected(
                (style.width, self.bresenham, self.wide, range),
                "Bresenham line support and width within the enabled device range",
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn absent_line_and_wide_capabilities_reject_only_unsupported_requests() {
        let narrow = LineCapabilities {
            bresenham: true,
            wide: false,
        };
        assert!(
            narrow
                .validate(LineStyle::new(1., 0xffff, 1).unwrap(), [1., 8.])
                .is_ok()
        );
        assert!(
            narrow
                .validate(LineStyle::new(2., 0xffff, 1).unwrap(), [1., 8.])
                .is_err()
        );
        assert!(
            LineCapabilities::default()
                .validate(LineStyle::new(1., 0xffff, 1).unwrap(), [1., 8.])
                .is_err()
        );
    }
    #[test]
    fn subpixel_native_lines_use_one_pixel_without_requiring_wide_lines() {
        let narrow = LineCapabilities {
            bresenham: true,
            wide: false,
        };
        let style = LineStyle::new(0.5, 0xffff, 1).unwrap();
        assert_eq!(style.parameters().0, 0.5);
        assert_eq!(style.raster_width(), 1.);
        assert!(narrow.validate(style, [1., 8.]).is_ok());
    }
    #[test]
    fn widths_must_fit_the_selected_device_range() {
        let wide = LineCapabilities {
            bresenham: true,
            wide: true,
        };
        assert!(
            wide.validate(LineStyle::new(3., 0xffff, 1).unwrap(), [1., 8.])
                .is_ok()
        );
        assert!(
            wide.validate(LineStyle::new(9., 0xffff, 1).unwrap(), [1., 8.])
                .is_err()
        );
    }
}

#[cfg(test)]
mod forced_blend_tests {
    use super::*;
    #[test]
    fn forced_blending_requires_the_same_enabled_line_capabilities() {
        let style = LineStyle::new(1., 0xffff, 1)
            .unwrap()
            .with_alpha_blending(true);
        let error = LineCapabilities::default()
            .validate(style, [1., 8.])
            .unwrap_err();
        assert!(error.to_string().contains("Bresenham line support"));
        let narrow = LineCapabilities {
            bresenham: true,
            wide: false,
        };
        assert!(narrow.validate(style, [1., 8.]).is_ok());
        let wide_style = LineStyle::new(3., 0xffff, 1)
            .unwrap()
            .with_alpha_blending(true);
        assert!(narrow.validate(wide_style, [1., 8.]).is_err());
        assert!(
            LineCapabilities {
                bresenham: true,
                wide: true
            }
            .validate(wide_style, [1., 2.])
            .is_err()
        );
    }
}
