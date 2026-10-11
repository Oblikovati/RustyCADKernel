//! Native custom labels/colors, borrowed sequences and typed numerical label formatting.
use super::{AisColorScale, color_scale::ColorScaleFields};
use rustycad_tkernel::{
    quantity::Color,
    standard::{
        Failure,
        c_string::{FormatArg, sprintf_into},
        exceptions::{out_of_range, program_error},
    },
    tcollection::{AsciiString, ExtendedString},
};
use rustycad_tkservice::aspect::AspectTypeOfColorScaleData;
use std::{ops::Deref, sync::RwLockReadGuard};
macro_rules! scale_borrow {
    ($name:ident,$target:ty,$category:ident,$field:ident) => {
        /// Borrow retained scale storage; release before mutation, e.g. `drop(scale.labels())`.
        pub struct $name<'a>(RwLockReadGuard<'a, ColorScaleFields>);
        impl Deref for $name<'_> {
            type Target = $target;
            fn deref(&self) -> &Self::Target {
                &self.0.$category.$field
            }
        }
    };
}
scale_borrow!(AisColorScaleTitle, ExtendedString, inputs, title);
scale_borrow!(AisColorScaleFormat, AsciiString, inputs, format);
scale_borrow!(AisColorScaleLabels, Vec<ExtendedString>, inputs, labels);
scale_borrow!(AisColorScaleColors, Vec<Color>, inputs, colors);
impl AisColorScale {
    /// Borrow title storage; `scale.get_title().is_empty()`.
    pub fn get_title(&self) -> AisColorScaleTitle<'_> {
        AisColorScaleTitle(self.fields.read())
    }
    /// Copy the title; `scale.set_title(&ExtendedString::from_utf8("Pressure"))`.
    pub fn set_title(&self, title: &ExtendedString) {
        self.fields.write().inputs.title = title.clone();
    }
    /// Borrow the number format; `scale.get_format().as_bytes()`.
    pub fn get_format(&self) -> AisColorScaleFormat<'_> {
        AisColorScaleFormat(self.fields.read())
    }
    /// Native Format alias; `scale.format().as_bytes()`.
    pub fn format(&self) -> AisColorScaleFormat<'_> {
        self.get_format()
    }
    /// Copy a typed printf floating-point format; `scale.set_format(&AsciiString::from("%.2f"))`.
    pub fn set_format(&self, format: &AsciiString) {
        self.fields.write().inputs.format = format.clone();
    }
    /// Borrow custom labels; `scale.labels().len()`.
    pub fn labels(&self) -> AisColorScaleLabels<'_> {
        AisColorScaleLabels(self.fields.read())
    }
    /// Copy custom labels into output, replacing previous content; `scale.get_labels(&mut labels)`.
    pub fn get_labels(&self, labels: &mut Vec<ExtendedString>) {
        labels.clone_from(&self.fields.read().inputs.labels);
    }
    /// Copy custom labels without switching type; `scale.set_labels(&labels)`.
    pub fn set_labels(&self, labels: &[ExtendedString]) {
        self.fields.write().inputs.labels = labels.to_vec();
    }
    /// Borrow custom colors; `scale.colors().len()`.
    pub fn colors(&self) -> AisColorScaleColors<'_> {
        AisColorScaleColors(self.fields.read())
    }
    /// Native GetColors borrowed alias; `scale.get_colors().len()`.
    pub fn get_colors(&self) -> AisColorScaleColors<'_> {
        self.colors()
    }
    /// Copy custom colors into output, replacing previous content; `scale.copy_colors(&mut colors)`.
    pub fn copy_colors(&self, colors: &mut Vec<Color>) {
        colors.clone_from(&self.fields.read().inputs.colors);
    }
    /// Copy custom colors without switching type; `scale.set_colors(&colors)`.
    pub fn set_colors(&self, colors: &[Color]) {
        self.fields.write().inputs.colors = colors.to_vec();
    }
    /// Append for index<=0 or resize through a one-based index; `scale.set_label(&label,3)`.
    pub fn set_label(&self, label: &ExtendedString, index: i32) {
        let mut f = self.fields.write();
        let labels = &mut f.inputs.labels;
        let index = if index <= 0 {
            labels.len()
        } else {
            (index - 1) as usize
        };
        labels.resize_with(labels.len().max(index + 1), ExtendedString::default);
        labels[index] = label.clone();
    }
    /// Append or resize with native default colors; `scale.set_interval_color(red,3)`.
    pub fn set_interval_color(&self, color: Color, index: i32) {
        let mut f = self.fields.write();
        let colors = &mut f.inputs.colors;
        let index = if index <= 0 {
            colors.len()
        } else {
            (index - 1) as usize
        };
        colors.resize(colors.len().max(index + 1), Color::default());
        colors[index] = color;
    }
    /// Get one-based custom color or automatically sampled HLS; `scale.get_interval_color(1)?`.
    pub fn get_interval_color(&self, index: i32) -> Result<Color, Failure> {
        let f = self.fields.read();
        if f.inputs.color_type == AspectTypeOfColorScaleData::User {
            return Ok(usize::try_from(index - 1)
                .ok()
                .and_then(|i| f.inputs.colors.get(i))
                .copied()
                .unwrap_or_default());
        }
        let count = f.range.intervals;
        drop(f);
        self.scale_color_from_value(f64::from(index - 1), 0.0, f64::from(count - 1))
    }
    /// Get an automatic label or custom one-based value; native invalid custom indices raise.
    /// Example: `scale.get_label(1)?`.
    pub fn get_label(&self, index: i32) -> Result<ExtendedString, Failure> {
        let f = self.fields.read();
        if f.inputs.label_type == AspectTypeOfColorScaleData::User {
            return custom_scale_label(&f.inputs.labels, index);
        }
        let border = f.inputs.border;
        let format = f.inputs.format.clone();
        drop(f);
        let value = if border {
            self.interval_value(index - 1)
        } else {
            0.5 * (self.interval_value(index - 1) + self.interval_value(index))
        };
        format_scale_label(&format, value)
    }
}
fn format_scale_label(format: &AsciiString, value: f64) -> Result<ExtendedString, Failure> {
    let mut output = Vec::new();
    let count = sprintf_into(&mut output, format.as_bytes(), &[FormatArg::Real(value)]).map_err(
        |error| {
            program_error(&format!(
                "AIS_ColorScale format={:?}; expected one floating-point argument: {error}",
                format.as_bytes()
            ))
        },
    )?;
    Ok(ExtendedString::from_c_string(&output[..count], false))
}

fn custom_scale_label(labels: &[ExtendedString], index: i32) -> Result<ExtendedString, Failure> {
    usize::try_from(index - 1)
        .ok()
        .and_then(|i| labels.get(i))
        .cloned()
        .ok_or_else(|| {
            out_of_range(&format!(
                "AIS_ColorScale label index={index}; expected 1..={}",
                labels.len()
            ))
        })
}
