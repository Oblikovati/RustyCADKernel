//! Consecutive bounds often share a matrix; unchanged constants must not dirty driver state.
use ash::vk;
#[derive(Default)]
pub(super) struct DrawState<'a> {
    pipeline: vk::Pipeline,
    transform: &'a [u8],
}
impl<'a> DrawState<'a> {
    pub fn advance(&mut self, pipeline: vk::Pipeline, transform: &'a [u8]) -> (bool, bool) {
        let bind = self.pipeline != pipeline;
        // A different layout can invalidate constants even when the bytes are identical.
        let push = bind || self.transform != transform;
        self.pipeline = pipeline;
        self.transform = transform;
        (bind, push)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use ash::vk::Handle;
    #[test]
    fn repeated_bounds_reuse_constants_but_live_transforms_upload() {
        let mut state = DrawState::default();
        let pipeline = vk::Pipeline::from_raw(1);
        assert_eq!(state.advance(pipeline, &[1; 64]), (true, true));
        assert_eq!(state.advance(pipeline, &[1; 64]), (false, false));
        assert_eq!(state.advance(pipeline, &[2; 64]), (false, true));
    }
    #[test]
    fn pipeline_change_repushes_even_identical_transform_bytes() {
        let mut state = DrawState::default();
        assert_eq!(
            state.advance(vk::Pipeline::from_raw(1), &[1; 64]),
            (true, true)
        );
        assert_eq!(
            state.advance(vk::Pipeline::from_raw(2), &[1; 64]),
            (true, true)
        );
        assert_eq!(
            state.advance(vk::Pipeline::from_raw(2), &[1; 80]),
            (false, true)
        );
    }
}
