//! Martensite selection fixtures use real camera picking and observable native callbacks.
use super::detection_scene::DetectionScene;
use rustycad_tkernel::{
    ncollection::{Array1, List, Vec2},
    standard::{Failure, Handle},
};
use rustycad_tkmath::gp::Point2;
use rustycad_tkservice::graphic3d::*;
use rustycad_tkv3d::{ais::*, prs_mgr::*, prs3d::Prs3dDrawer, select_mgr::*};

pub struct SelectionActionsScene {
    pub detection: DetectionScene,
    pub status: Handle<AisGlobalStatus>,
    pub style: Handle<Prs3dDrawer>,
}
impl SelectionActionsScene {
    pub fn new(flags: i32, mask: i32, mode: i32, mutation: i32, failing: i32) -> Self {
        let detection = DetectionScene::new(flags, mask, mode, mutation, failing);
        detection.install_geometry().unwrap();
        detection.trace.write().selection_events = true;
        detection.object.set_auto_hilight(flags & 1024 == 0);
        detection
            .object
            .presentable_base()
            .fields
            .write()
            .display_status = if flags & 4096 == 0 {
            PrsMgrDisplayStatus::Displayed
        } else {
            PrsMgrDisplayStatus::Erased
        };
        let status = Handle::new(AisGlobalStatus::new());
        status.set_display_mode(3);
        status.add_selection_mode(0);
        status.set_sub_intensity(true);
        if flags & 2048 == 0 {
            detection
                .context
                .fields
                .write()
                .objects
                .bind(detection.object.cast(), status.clone());
        }
        let style = Handle::new(Prs3dDrawer::new());
        style.set_display_mode(mode);
        style.set_transparency(0.4);
        detection.context.set_selection_style(&style);
        if mutation == 3 {
            detection.object.set_hilight_attributes(&style);
        }
        Self {
            detection,
            status,
            style,
        }
    }
    pub fn owner(&self, index: i32) -> AisSelectedOwner {
        if index < 0 {
            return Handle::default();
        }
        self.detection.owners[index as usize].clone()
    }
    pub fn owners(&self, mask: i32) -> List<AisSelectedOwner> {
        let mut owners = List::default();
        for index in 0..3 {
            if mask & (1 << index) != 0 {
                owners.append(self.owner(index));
            }
        }
        owners
    }
    pub fn operation(&self, op: i32, a: i32, b: i32) -> Result<Vec<i32>, Failure> {
        if op < 18 {
            return self.selection_operation(op, a, b);
        }
        if op < 31 {
            return self.presentation_operation(op, a, b);
        }
        self.query_operation(op, a, b)
    }
    fn selection_operation(&self, op: i32, a: i32, b: i32) -> Result<Vec<i32>, Failure> {
        let scene = &self.detection;
        let ctx = &scene.context;
        match op {
            0 => {
                let owners: Vec<_> = self.owners(a).iter().cloned().collect();
                return Ok(vec![
                    ctx.select_owners(&Array1::from_shared(&owners, 1), selection_scheme(b))?
                        as i32,
                ]);
            }
            1 => return Ok(vec![ctx.add_select(&self.owner(a))? as i32]),
            2 => return Ok(vec![ctx.add_select_object(&scene.object.cast())? as i32]),
            3 => ctx.set_selected_owner(&self.owner(a), b != 0)?,
            4 => ctx.set_selected_object(&scene.object.cast(), b != 0)?,
            5 => ctx.add_or_remove_selected_owner(&self.owner(a), b != 0)?,
            6 => ctx.add_or_remove_selected_object(&scene.object.cast(), b != 0)?,
            7 => {
                return Ok(vec![i32::from(
                    ctx.set_selected_state(&self.owner(a), b != 0)?,
                )]);
            }
            8 => ctx.hilight_selected(b != 0)?,
            9 => ctx.unhilight_selected(b != 0)?,
            10 => ctx.clear_selected(b != 0)?,
            11 => ctx.init_selected(),
            12 => ctx.next_selected(),
            13 => scene.trace.write().mask = a,
            14 => scene.trace.write().failing = a,
            15 => scene.context.fields.write().last_picked = self.owner(a),
            16 => return Ok(vec![ctx.select_detected(selection_scheme(b))? as i32]),
            17 => scene.trace.write().consume_click = a != 0,
            _ => unreachable!(),
        }
        Ok(Vec::new())
    }
    fn presentation_operation(&self, op: i32, a: i32, b: i32) -> Result<Vec<i32>, Failure> {
        let scene = &self.detection;
        let ctx = &scene.context;
        let object = scene.object.cast();
        match op {
            18 => {
                return Ok(vec![ctx.select_point(
                    Vec2::new(40 + a, 30),
                    &scene.view,
                    selection_scheme(b),
                )? as i32]);
            }
            19 => {
                return Ok(vec![ctx.select_rectangle(
                    Vec2::new(20 + a, 10),
                    Vec2::new(60 + a, 50),
                    &scene.view,
                    selection_scheme(b),
                )? as i32]);
            }
            20 => {
                return Ok(vec![ctx.select_polygon(
                    &selection_polygon(a),
                    &scene.view,
                    selection_scheme(b),
                )? as i32]);
            }
            21 => ctx.highlight_owners(
                &self.owners(a),
                &if b == 0 {
                    Handle::default()
                } else {
                    self.style.clone()
                },
            )?,
            22 => ctx.unhighlight_owners(&self.owners(a), b != 0)?,
            23 => ctx.highlight_selected(&self.owner(a))?,
            24 => ctx.unhighlight_selected(b != 0)?,
            25 => ctx.highlight_global(&object, &self.style, a)?,
            26 => ctx.unhighlight_global(&object)?,
            27 => ctx.highlight_object_with_subintensity(&object, a)?,
            28 => ctx.highlight_owner_with_subintensity(&self.owner(a), b)?,
            29 => ctx.hilight_with_color(&object, &self.style, b != 0)?,
            30 => ctx.unhilight(&object, b != 0)?,
            _ => unreachable!(),
        }
        Ok(Vec::new())
    }
    fn query_operation(&self, op: i32, a: i32, b: i32) -> Result<Vec<i32>, Failure> {
        let scene = &self.detection;
        let ctx = &scene.context;
        match op {
            31 | 32 => {
                let mut style = self.style.clone();
                let found = if op == 31 {
                    ctx.owner_highlight_style(&self.owner(a), &mut style)
                } else {
                    ctx.object_highlight_style(&scene.object.cast(), &mut style)
                };
                Ok(vec![i32::from(found), style_mode(&style)])
            }
            33 => Ok(vec![i32::from(ctx.is_hilighted_owner(&self.owner(a)))]),
            34 => Ok(vec![style_mode(
                &ctx.get_sel_style(&scene.object.cast(), &self.owner(a))
                    .read(),
            )]),
            35 => Ok(vec![if a == 0 {
                ctx.select(b != 0)?
            } else {
                ctx.shift_select(b != 0)?
            } as i32]),
            36 => Ok(vec![if a == 0 {
                ctx.select_box(20, 10, 60, 50, &scene.view, b != 0)?
            } else {
                ctx.shift_select_box(20, 10, 60, 50, &scene.view, b != 0)?
            } as i32]),
            37 => Ok(vec![if a == 0 {
                ctx.select_polyline(&selection_polygon(0), &scene.view, b != 0)?
            } else {
                ctx.shift_select_polyline(&selection_polygon(0), &scene.view, b != 0)?
            } as i32]),
            38 => {
                scene.trace.write().append_on_highlight = self.owner(a);
                Ok(vec![])
            }
            39 => {
                let foreign = DetectionScene::new(0, 7, 2, 0, 0);
                ctx.select_point(Vec2::new(40, 30), &foreign.view, selection_scheme(b))?;
                Ok(vec![])
            }
            42 => {
                let child =
                    super::detection_scene::object::DetectionObject::new(scene.trace.clone());
                scene.object.add_child(&child.cast())?;
                ctx.set_context_to_object(&scene.object.cast())?;
                Ok(vec![i32::from(child.get_context() == *ctx)])
            }
            43 => {
                let foreign = DetectionScene::new(0, 7, 2, 0, 0);
                foreign
                    .context
                    .set_context_to_object(&scene.object.cast())?;
                Ok(vec![])
            }
            44 => {
                scene.object.clear_selections(false);
                Ok(vec![])
            }
            40 => Ok(vec![i32::from(
                ctx.is_detected_object(&scene.object.cast())?,
            )]),
            41 => {
                scene.operation(9, a)?;
                Ok(vec![])
            }
            _ => unreachable!(),
        }
    }
}
impl Drop for SelectionActionsScene {
    fn drop(&mut self) {
        self.detection.trace.write().append_on_highlight = Handle::default();
    }
}
pub fn selection_scheme(value: i32) -> AisSelectionScheme {
    match value {
        -1 => AisSelectionScheme::Unknown,
        0 => AisSelectionScheme::Replace,
        1 => AisSelectionScheme::Add,
        2 => AisSelectionScheme::Remove,
        3 => AisSelectionScheme::Xor,
        4 => AisSelectionScheme::Clear,
        5 => AisSelectionScheme::ReplaceExtra,
        _ => panic!("scheme={value}; expected -1..=5"),
    }
}
pub fn selection_polygon(offset: i32) -> Array1<'static, Point2> {
    let mut polygon = Array1::with_bounds(1, 4).unwrap();
    polygon.as_mut_slice().clone_from_slice(&[
        Point2::new((20 + offset) as f64, 10.),
        Point2::new((60 + offset) as f64, 10.),
        Point2::new((60 + offset) as f64, 50.),
        Point2::new((20 + offset) as f64, 50.),
    ]);
    polygon
}
pub fn style_mode(style: &Handle<Prs3dDrawer>) -> i32 {
    style.get().map_or(-99, |style| style.display_mode())
}
