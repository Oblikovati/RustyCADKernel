//! Native integer layout, free-label placement and symmetric label filtering.
use super::{AisColorScale, color_scale::ColorScaleLayout};
use rustycad_tkernel::{
    standard::{Failure, exceptions::program_error},
    tcollection::ExtendedString,
};
use rustycad_tkservice::{
    aspect::AspectTypeOfColorScalePosition as Position,
    graphic3d::Graphic3dVerticalTextAlignment as Align,
};
pub(super) struct ColorScaleFrame {
    pub layout: ColorScaleLayout,
    pub bottom: i32,
    pub top: i32,
    pub height: i32,
    pub bar_x: i32,
    pub width: i32,
    pub label_width: i32,
    pub labels: Vec<ExtendedString>,
}
#[derive(Clone)]
pub(super) struct ColorScaleTextPlacement {
    pub text: ExtendedString,
    pub x: i32,
    pub y: i32,
    pub alignment: Align,
}
impl AisColorScale {
    pub(super) fn color_scale_frame(&self) -> Result<ColorScaleFrame, Failure> {
        let f = self.fields.read();
        let layout = f.layout.clone();
        let has_title = !f.inputs.title.is_empty();
        drop(f);
        let (bottom, top) = scale_bar_vertical_bounds(&layout, has_title);
        let labels = self.presentation_labels()?;
        let label_width = self.scale_frame_label_width(layout.label_pos, &labels)?;
        let (width, bar_x) = scale_bar_horizontal_bounds(&layout, label_width);
        Ok(ColorScaleFrame {
            layout,
            bottom,
            top,
            height: top - bottom,
            bar_x,
            width,
            label_width,
            labels,
        })
    }
    fn scale_frame_label_width(
        &self,
        position: Position,
        labels: &[ExtendedString],
    ) -> Result<i32, Failure> {
        if position == Position::None {
            return Ok(0);
        }
        self.maximum_label_width(labels)
    }
    fn presentation_labels(&self) -> Result<Vec<ExtendedString>, Failure> {
        if self.get_label_type() == rustycad_tkservice::aspect::AspectTypeOfColorScaleData::User {
            return Ok(self.fields.read().inputs.labels.clone());
        }
        let count = self.get_number_of_intervals() + i32::from(self.is_label_at_border());
        let mut labels = (1..=count)
            .map(|index| self.get_label(index))
            .collect::<Result<Vec<_>, _>>()?;
        if self.is_reversed() {
            labels.reverse();
        }
        Ok(labels)
    }
    fn maximum_label_width(&self, labels: &[ExtendedString]) -> Result<i32, Failure> {
        let mut width = 0;
        for label in labels.iter().filter(|label| !label.is_empty()) {
            width = width.max(self.text_width(label)?);
        }
        Ok(width)
    }
    pub(super) fn label_placements(
        &self,
        frame: &ColorScaleFrame,
    ) -> Result<Vec<ColorScaleTextPlacement>, Failure> {
        if frame.layout.label_pos == Position::None || frame.labels.is_empty() {
            return Ok(Vec::new());
        }
        let count = frame.labels.len() as i32;
        let border = self.is_label_at_border();
        let intervals = count - i32::from(border);
        let step = scale_label_step(frame.height, intervals)?;
        if step <= 0.0 {
            return Ok(Vec::new());
        }
        let filter = self.label_filter(count, intervals);
        if filter <= 0 {
            return Ok(Vec::new());
        }
        let y = frame.bottom + if border { 0 } else { (step / 2.0) as i32 };
        Ok(place_filtered_labels(frame, count, filter, y, step))
    }
    fn label_filter(&self, count: i32, intervals: i32) -> i32 {
        let f = self.fields.read();
        let layout = &f.layout;
        let title_height = if f.inputs.title.is_empty() {
            layout.spacing
        } else {
            layout.text_height + 2 * layout.spacing
        };
        let space = layout.height
            - title_height
            - (count.min(2) + (count - intervals - 1).abs()) * layout.text_height;
        if space <= 0 {
            return 0;
        }
        let value = f64::from(count) * f64::from(layout.text_height) / f64::from(space);
        value.trunc() as i32 + i32::from(value.fract() != 0.0)
    }
}
fn label_x(frame: &ColorScaleFrame) -> i32 {
    let base = frame.layout.x + frame.layout.spacing;
    match frame.layout.label_pos {
        Position::Center => base + (frame.width - frame.label_width) / 2,
        Position::Right => base + frame.width + frame.layout.spacing,
        _ => base,
    }
}
fn filtered_label_indexes(count: i32, filter: i32) -> Vec<i32> {
    let mut sweep = ScaleLabelSweep::new(count);
    while sweep.last - sweep.first >= filter || (sweep.last == 0 && sweep.first == 0) {
        sweep.step(count, filter);
    }
    for candidate in sweep.first..=sweep.last {
        if candidate % filter == 0
            && (candidate - sweep.last_first).abs() >= filter
            && (candidate - sweep.last_last).abs() >= filter
        {
            sweep.drawn.push(candidate);
            break;
        }
    }
    sweep.drawn
}

fn scale_bar_vertical_bounds(layout: &ColorScaleLayout, has_title: bool) -> (i32, i32) {
    let offset = layout.text_height / 2 + 2 * layout.spacing;
    let title_offset = if has_title {
        layout.text_height + layout.spacing
    } else {
        0
    };
    (
        layout.y + offset,
        layout.y + layout.height - title_offset - offset,
    )
}
fn scale_bar_horizontal_bounds(layout: &ColorScaleLayout, label_width: i32) -> (i32, i32) {
    let mut width = 5.max(20.min(layout.breadth - label_width - 3 * layout.spacing));
    if matches!(layout.label_pos, Position::Center | Position::None) {
        width += label_width;
    }
    let spacing = if label_width != 0 { layout.spacing } else { 0 };
    let x = layout.x
        + layout.spacing
        + if layout.label_pos == Position::Left {
            label_width + spacing
        } else {
            0
        };
    (width, x)
}

fn scale_label_step(height: i32, intervals: i32) -> Result<f64, Failure> {
    if intervals == 0 {
        return Err(program_error(
            "AIS_ColorScale label intervals=0; expected two border labels or non-border labels",
        ));
    }
    Ok(f64::from(height) / f64::from(intervals))
}
fn place_filtered_labels(
    frame: &ColorScaleFrame,
    count: i32,
    filter: i32,
    y: i32,
    step: f64,
) -> Vec<ColorScaleTextPlacement> {
    let x = label_x(frame);
    filtered_label_indexes(count, filter)
        .into_iter()
        .map(|index| ColorScaleTextPlacement {
            text: frame.labels[index as usize].clone(),
            x,
            y: y + (f64::from(index) * step) as i32,
            alignment: Align::Center,
        })
        .collect()
}

struct ScaleLabelSweep {
    first: i32,
    last: i32,
    last_first: i32,
    last_last: i32,
    drawn: Vec<i32>,
}
impl ScaleLabelSweep {
    fn new(count: i32) -> Self {
        Self {
            first: 0,
            last: count - 1,
            last_first: 0,
            last_last: count - 1,
            drawn: Vec::new(),
        }
    }

    fn step(&mut self, count: i32, filter: i32) {
        if self.first % filter == 0 {
            self.drawn.push(self.first);
            self.last_first = self.first;
        }
        if (count - 1 - self.last) % filter == 0 {
            self.drawn.push(self.last);
            self.last_last = self.last;
        }
        self.first += 1;
        self.last -= 1;
    }
}
