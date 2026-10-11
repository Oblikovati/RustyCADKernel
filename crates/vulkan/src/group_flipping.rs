//! OpenGl_Flipper at OCCT 3d097a0 saves ModelWorld before testing axis orientation.
use rustycad_tkernel::ncollection::Mat4;
use rustycad_tkmath::gp::Axis2;
use rustycad_tkservice::graphic3d::{Graphic3dFlipper, Graphic3dGroupMembers};

#[derive(Clone, Copy)]
struct FlipModelSnapshot {
    model: Mat4<f32>,
    clip: [f32; 16],
    changed: bool,
}

pub(crate) struct GroupFlipStack {
    current: FlipModelSnapshot,
    saved: Vec<FlipModelSnapshot>,
    projection: Mat4<f64>,
    view: Mat4<f32>,
    precise_view: Option<Mat4<f64>>,
}
impl GroupFlipStack {
    pub fn scene(
        context: &crate::text_submission::TextSceneContext<'_>,
        clip: [f32; 16],
    ) -> Result<Self, crate::RenderError> {
        let (model, view) = crate::persistent_transform::lighting_matrices(context)?;
        let ordinary = context.structure.trsf_pers.is_null()
            && context.group.transform_persistence().is_null();
        Ok(Self::new(
            Mat4::from_data(model),
            Mat4::from_data(view),
            context.camera.projection_matrix(),
            ordinary.then(|| context.camera.orientation_matrix()),
            clip,
        ))
    }
    pub fn new(
        model: Mat4<f32>,
        view: Mat4<f32>,
        projection: Mat4<f64>,
        precise_view: Option<Mat4<f64>>,
        clip: [f32; 16],
    ) -> Self {
        Self {
            current: FlipModelSnapshot {
                model,
                clip,
                changed: false,
            },
            saved: Vec::new(),
            projection,
            view,
            precise_view,
        }
    }
    pub fn apply(&mut self, plane: Option<Axis2>) {
        let Some(plane) = plane else {
            if let Some(saved) = self.saved.pop() {
                self.current = saved;
            }
            return;
        };
        self.saved.push(self.current);
        let rotation = Graphic3dFlipper::new(plane).compute(&(self.view * self.current.model));
        if rotation.is_identity() {
            return;
        }
        self.current.model *= rotation;
        self.current.clip = self.flipped_clip();
        self.current.changed = true;
    }
    fn flipped_clip(&self) -> [f32; 16] {
        if let Some(view) = self.precise_view {
            return (self.projection * view * Mat4::<f64>::from_other(&self.current.model))
                .data()
                .map(|value| value as f32);
        }
        (Mat4::<f32>::from_other(&self.projection) * self.view * self.current.model).data()
    }
    pub fn clip(&self) -> [f32; 16] {
        self.current.clip
    }
    pub fn override_model(&self) -> Option<Mat4<f32>> {
        self.current.changed.then_some(self.current.model)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustycad_tkmath::gp::{Direction3, Point3};
    fn stack(signs: [f32; 3]) -> GroupFlipStack {
        let mut view = Mat4::<f32>::identity().data();
        for (index, sign) in [0, 5, 10].into_iter().zip(signs) {
            view[index] = sign;
        }
        GroupFlipStack::new(
            Mat4::identity(),
            Mat4::from_data(view),
            Mat4::identity(),
            None,
            Mat4::<f32>::identity().data(),
        )
    }
    #[test]
    fn eight_axis_signs_choose_native_half_turns() {
        for (signs, expected) in [
            ([1., 1., 1.], [1., 1., 1.]),
            ([-1., 1., 1.], [-1., -1., 1.]),
            ([1., -1., 1.], [-1., -1., 1.]),
            ([-1., -1., 1.], [-1., -1., 1.]),
            ([1., 1., -1.], [-1., 1., -1.]),
            ([-1., 1., -1.], [-1., 1., -1.]),
            ([1., -1., -1.], [1., -1., -1.]),
            ([-1., -1., -1.], [1., -1., -1.]),
        ] {
            let mut flips = stack(signs);
            flips.apply(Some(Axis2::default()));
            let matrix = flips.current.model.data();
            assert_eq!([matrix[0], matrix[5], matrix[10]], expected);
        }
    }
    #[test]
    fn nested_identity_push_restores_each_saved_model_and_exact_clip() {
        let mut flips = stack([-1., -1., 1.]);
        let initial = flips.clip();
        flips.apply(Some(Axis2::default()));
        let rotated = flips.current.model;
        flips.apply(Some(Axis2::default()));
        assert_eq!(flips.current.model, rotated);
        flips.apply(None);
        assert_eq!(flips.current.model, rotated);
        flips.apply(None);
        assert_eq!(flips.clip(), initial);
        assert!(flips.override_model().is_none());
        flips.apply(None);
        assert_eq!(flips.clip(), initial);
    }
    #[test]
    fn offset_reference_origin_is_fixed_by_the_rotation() {
        let mut flips = stack([-1., -1., 1.]);
        let plane = Axis2::with_x_direction(
            Point3::new(2., 3., 0.),
            Direction3::new(0., 0., 1.).unwrap(),
            Direction3::new(1., 0., 0.).unwrap(),
        )
        .unwrap();
        flips.apply(Some(plane));
        let matrix = flips.current.model.data();
        assert_eq!([matrix[12], matrix[13], matrix[14]], [4., 6., 0.]);
        assert_eq!(flips.override_model(), Some(flips.current.model));
    }
    #[test]
    fn no_rotation_preserves_the_original_clip_precision() {
        let mut flips = stack([1.; 3]);
        flips.current.clip[0] = 0.12345679;
        flips.precise_view = Some(Mat4::identity());
        flips.apply(Some(Axis2::default()));
        assert_eq!(flips.clip()[0], 0.12345679);
        assert!(flips.override_model().is_none());
        flips.apply(None);
        assert_eq!(flips.clip()[0], 0.12345679);
    }
}
