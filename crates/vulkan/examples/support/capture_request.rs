//! Parse and configure the shared camera contract of Martensite capture examples.
use rustycad_tkservice::{aspect::AspectNeutralWindow, graphic3d::Graphic3dCameraProjection};
use rustycad_tkv3d::v3d::V3dView;
pub struct ViewerCaptureRequest {
    pub width: i32,
    pub height: i32,
    pub perspective: i32,
    pub offset: f64,
    pub prefix: String,
    pub mode: String,
}
impl ViewerCaptureRequest {
    pub fn parse(arguments: &[String], fallback: &str) -> Result<Self, Box<dyn std::error::Error>> {
        if !(6..=7).contains(&arguments.len()) {
            return Err(format!(
                "received {} arguments; expected width height perspective offset prefix [mode]",
                arguments.len()
            )
            .into());
        }
        Ok(Self {
            width: capture_number(arguments, 1, "i32 width")?,
            height: capture_number(arguments, 2, "i32 height")?,
            perspective: capture_number(arguments, 3, "i32 perspective flag")?,
            offset: capture_number(arguments, 4, "f64 translation")?,
            prefix: arguments[5].clone(),
            mode: arguments
                .get(6)
                .cloned()
                .unwrap_or_else(|| fallback.to_owned()),
        })
    }
    pub fn configure(&self, window: &AspectNeutralWindow, view: &V3dView) {
        window.set_size(self.width, self.height);
        view.must_be_resized();
        let projection = if self.perspective != 0 {
            Graphic3dCameraProjection::Perspective
        } else {
            Graphic3dCameraProjection::Orthographic
        };
        view.camera().set_projection_type(projection);
        view.camera().set_scale(20.);
        view.camera().set_z_range(1., 100.).unwrap();
    }
}
fn capture_number<T: std::str::FromStr>(
    arguments: &[String],
    index: usize,
    shape: &str,
) -> Result<T, Box<dyn std::error::Error>> {
    arguments[index]
        .parse()
        .map_err(|_| format!("received {:?}; expected {shape}", arguments[index]).into())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn valid_capture_arguments_and_failures_are_explicit() {
        let mut arguments = vec!["capture", "64", "48", "1", "-3", "result"]
            .into_iter()
            .map(str::to_owned)
            .collect::<Vec<_>>();
        let request = ViewerCaptureRequest::parse(&arguments, "default").unwrap();
        assert_eq!(
            (
                request.width,
                request.height,
                request.perspective,
                request.offset
            ),
            (64, 48, 1, -3.)
        );
        assert_eq!(
            (request.prefix.as_str(), request.mode.as_str()),
            ("result", "default")
        );
        arguments.push("full".to_owned());
        assert_eq!(
            ViewerCaptureRequest::parse(&arguments, "default")
                .unwrap()
                .mode,
            "full"
        );
        arguments[1] = "bad".to_owned();
        assert!(ViewerCaptureRequest::parse(&arguments, "default").is_err());
        assert!(ViewerCaptureRequest::parse(&[], "default").is_err());
    }
}
