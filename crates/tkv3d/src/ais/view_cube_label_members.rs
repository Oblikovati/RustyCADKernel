//! Label mutations use real native maps and preserve the six-name Y-up permutation.
use super::AisViewCubePresentationSource;
use crate::{
    prs_mgr::PrsMgrPresentableObjectMembers, prs3d::Prs3dDatumParts,
    v3d::V3dTypeOfOrientation as Orientation,
};
use rustycad_tkernel::{
    standard::{Failure, exceptions::program_error},
    tcollection::AsciiString,
};
/// Native labels and up convention; `cube.set_y_up(true,true)`.
pub trait AisViewCubeLabelMembers: AisViewCubePresentationSource {
    /// Copy a bound label, with an empty value for missing keys; `cube.box_side_label(orientation)`.
    fn box_side_label(&self, side: Orientation) -> AsciiString {
        self.view_cube_presentation_base()
            .settings
            .read()
            .side_labels
            .seek(&side)
            .cloned()
            .unwrap_or_default()
    }
    /// Bind a side and mark presentation stale, including identical labels; `cube.set_box_side_label(side,&name)?`.
    fn set_box_side_label(&self, side: Orientation, label: &AsciiString) -> Result<(), Failure> {
        if !super::ais_view_cube_is_box_side(side) {
            return Err(program_error(&format!(
                "AIS_ViewCube side={side:?}; expected one of six box-side orientations"
            )));
        }
        self.view_cube_presentation_base()
            .settings
            .write()
            .side_labels
            .bind(side, label.clone());
        self.set_to_update_all();
        Ok(())
    }
    /// Copy any datum label, with an empty value for missing keys; `cube.axis_label(Prs3dDatumParts::Origin)`.
    fn axis_label(&self, axis: Prs3dDatumParts) -> AsciiString {
        self.view_cube_presentation_base()
            .settings
            .read()
            .axes_labels
            .seek(&axis)
            .cloned()
            .unwrap_or_default()
    }
    /// Bind the three labels and always mark presentation stale; `cube.set_axes_labels(&x,&y,&z)`.
    fn set_axes_labels(&self, x: &AsciiString, y: &AsciiString, z: &AsciiString) {
        let mut settings = self.view_cube_presentation_base().settings.write();
        for (axis, label) in cube_label_axis_keys().into_iter().zip([x, y, z]) {
            settings.axes_labels.bind(axis, label.clone());
        }
        drop(settings);
        self.set_to_update_all();
    }
    /// Read the native viewer up convention; `cube.is_y_up()`.
    fn is_y_up(&self) -> bool {
        self.view_cube_presentation_base().settings.read().is_y_up
    }
    /// Change the convention and optionally remap existing names; unchanged convention is a no-op.
    /// Example: `cube.set_y_up(true,false)` retains all labels.
    fn set_y_up(&self, y_up: bool, update_labels: bool) {
        let mut settings = self.view_cube_presentation_base().settings.write();
        if settings.is_y_up == y_up {
            return;
        }
        settings.is_y_up = y_up;
        if update_labels {
            cube_remap_up_labels(&mut settings.side_labels, y_up);
        }
        drop(settings);
        self.set_to_update_all();
    }
}
impl<T: AisViewCubePresentationSource + ?Sized> AisViewCubeLabelMembers for T {}
pub(super) fn cube_label_axis_keys() -> [Prs3dDatumParts; 3] {
    [
        Prs3dDatumParts::XAxis,
        Prs3dDatumParts::YAxis,
        Prs3dDatumParts::ZAxis,
    ]
}
fn cube_remap_up_labels(labels: &mut super::AisViewCubeSideLabels, y_up: bool) {
    let (z_keys, y_keys) = cube_label_orientation_keys();
    let (source, target) = if y_up {
        (z_keys, y_keys)
    } else {
        (y_keys, z_keys)
    };
    let values = source.map(|key| labels.seek(&key).cloned().unwrap_or_default());
    for (key, value) in target.into_iter().zip(values) {
        labels.bind(key, value);
    }
}

fn cube_label_orientation_keys() -> ([Orientation; 6], [Orientation; 6]) {
    let z_keys = [
        Orientation::Yneg,
        Orientation::Ypos,
        Orientation::Zpos,
        Orientation::Zneg,
        Orientation::Xneg,
        Orientation::Xpos,
    ];
    let y_keys = [
        Orientation::Zpos,
        Orientation::Zneg,
        Orientation::Ypos,
        Orientation::Yneg,
        Orientation::Xpos,
        Orientation::Xneg,
    ];
    (z_keys, y_keys)
}
