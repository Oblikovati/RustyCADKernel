#![allow(dead_code)]
//! Actual AIS axes retain labels while geometry, appearance and selection change.
use crate::{line_scene::fixture, view_setup};
use martensite_vulkan::{Graphic3dRenderDriver, Graphic3dRenderView};
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::{Failure, Handle},
};
use rustycad_tkg3d::geom::{GeomAxis1Placement, GeomAxis2Placement};
use rustycad_tkmath::gp::{Axis1, Axis2, Direction3, Point3};
use rustycad_tkservice::aspect::{AspectNeutralWindow, AspectTypeOfLine};
use rustycad_tkv3d::{ais::*, prs_mgr::*, prs3d::Prs3dLineAspect, v3d::*};
pub const MODES: [&str; 18] = [
    "axis-x",
    "axis-y",
    "axis-z",
    "axis-line",
    "axis-axis1",
    "axis-ray",
    "axis-ray-default",
    "axis-no-labels",
    "axis-color",
    "axis-width",
    "axis-unset-color",
    "axis-unset-width",
    "axis-display-aspect",
    "axis-moved",
    "axis-erased",
    "axis-redisplayed",
    "axis-selected",
    "axis-detected",
];
pub struct AxisScene {
    pub view: Handle<V3dView>,
    pub window: Handle<AspectNeutralWindow>,
    pub context: Handle<AisInteractiveContext>,
    pub axis: Handle<AisAxis>,
}
impl AxisScene {
    pub fn new(driver: &Handle<Graphic3dRenderDriver>, mode: &str) -> Self {
        let viewer = V3dViewer::new(&driver.cast()).unwrap();
        let view = viewer.create_view().unwrap();
        let window = view_setup::configure_view(&view);
        let context = AisInteractiveContext::new(&viewer).unwrap();
        let axis = axis_object(mode).unwrap();
        axis.attributes().set_maximal_parameter_value(12.);
        context.display(&axis.cast(), false).unwrap();
        Self {
            view,
            window,
            context,
            axis,
        }
    }
    pub fn render_view(&self) -> Handle<Graphic3dRenderView> {
        self.view.get().unwrap().view().cast()
    }
    pub fn apply(&self, mode: &str) -> Result<(), Failure> {
        match mode {
            "axis-color" => self.axis.set_color(Color::named(NamedColor::RED)),
            "axis-width" => self.axis.set_width(3.),
            "axis-unset-color" => {
                self.axis.set_color(Color::named(NamedColor::RED))?;
                self.axis.unset_color()
            }
            "axis-unset-width" => {
                self.axis.set_width(3.)?;
                self.axis.unset_width()
            }
            "axis-display-aspect" => {
                self.axis
                    .set_display_aspect(&Handle::new(Prs3dLineAspect::new(
                        Color::named(NamedColor::GREEN),
                        AspectTypeOfLine::Dash,
                        2.,
                    )?))?;
                self.redisplay()
            }
            "axis-moved" => {
                self.axis
                    .set_axis2_placement(&frame(Point3::new(-2., 1., 0.)), AisTypeOfAxis::YAxis)?;
                self.redisplay()
            }
            "axis-no-labels" => {
                self.axis.attributes().datum_aspect().set_draw_labels(false);
                self.redisplay()
            }
            "axis-erased" => self.context.erase(&self.axis.cast(), false),
            "axis-redisplayed" => {
                self.context.erase(&self.axis.cast(), false)?;
                self.context.display(&self.axis.cast(), false)
            }
            "axis-selected" => self.context.set_selected_object(&self.axis.cast(), false),
            "axis-detected" => {
                let offset = self.axis.transformation().translation_part();
                let (x, y) = self.view.convert_world_to_pixels(
                    offset.x() - 1.,
                    offset.y() - 3.,
                    offset.z(),
                )?;
                self.context.move_to(x, y, &self.view, false)?;
                Ok(())
            }
            _ => Ok(()),
        }
    }
    fn redisplay(&self) -> Result<(), Failure> {
        self.context.redisplay(&self.axis.cast(), false, false)
    }
}
fn frame(origin: Point3) -> Handle<GeomAxis2Placement> {
    Handle::new(GeomAxis2Placement::new(
        Axis2::new(origin, Direction3::new(0., 0., 1.).unwrap()).unwrap(),
    ))
}
fn axis_object(mode: &str) -> Result<Handle<AisAxis>, Failure> {
    let units = fixture::units();
    let ray = Axis1::new(Point3::new(-4., -3., 0.), Direction3::new(1., 0., 0.)?);
    match mode {
        "axis-line" => AisAxis::new(&fixture::component(), units),
        "axis-axis1" => AisAxis::from_axis1(&Handle::new(GeomAxis1Placement::new(ray)), units),
        "axis-ray" | "axis-ray-default" => {
            AisAxis::from_ray(&ray, if mode == "axis-ray" { 6. } else { -1. }, units)
        }
        _ => {
            let kind = match mode {
                "axis-y" => AisTypeOfAxis::YAxis,
                "axis-z" => AisTypeOfAxis::ZAxis,
                _ => AisTypeOfAxis::XAxis,
            };
            let frame = frame(ray.location());
            let axis = AisAxis::from_axis2(&frame, kind, units)?;
            axis.attributes().datum_aspect().set_axis_length(6., 6., 6.);
            axis.set_axis2_placement(&frame, kind)?;
            Ok(axis)
        }
    }
}
