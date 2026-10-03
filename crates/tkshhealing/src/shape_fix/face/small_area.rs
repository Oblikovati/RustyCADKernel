//! Small-area wire removal from ShapeFix_Face.cxx at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::ShapeFixFace;
use crate::shape_analysis::ShapeAnalysisWire;
use crate::shape_fix::FixRoot;
use rustycad_tkbrep::{
    brep_tools::ReShape,
    topods::{self, Builder, Face, Shape, ShapeIterator, Wire},
};
use rustycad_tkernel::{message::Msg, standard::Failure, tcollection::AsciiString};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

impl ShapeFixFace {
    /// Removes negligible-area contours, optionally removing an entirely small face from context.
    /// Example: `fixer.fix_small_area_wire(true)?`.
    /// Native returns false when every wire is removed, even if context removes the face.
    pub fn fix_small_area_wire(&self, remove_small_face: bool) -> Result<bool, Failure> {
        let original = self.apply_small_area_context()?;
        let mut replacement = topods::face(&original.empty_copied()?)?;
        replacement.set_orientation(Orientation::Forward);
        let (removed, retained) = self.filter_small_area_wires(&original, &mut replacement)?;
        if removed == 0 {
            return Ok(false);
        }
        if retained == 0 {
            return self.remove_entire_small_face(&original, remove_small_face);
        }
        self.replace_small_area_face(original, replacement)
    }

    fn replace_small_area_face(
        &self,
        original: Face,
        mut replacement: Face,
    ) -> Result<bool, Failure> {
        replacement.set_orientation(original.orientation());
        if let Some(context) = self.context().get() {
            ReShape::replace(
                context,
                &original.into_shape(),
                &replacement.clone().into_shape(),
            )?;
        }
        *self.face_state.write() = replacement.into_shape();
        Ok(true)
    }

    fn apply_small_area_context(&self) -> Result<Face, Failure> {
        let mut face = self.face()?;
        if let Some(context) = self.context().get() {
            face = topods::face(&ReShape::apply(
                context,
                &face.into_shape(),
                ShapeEnum::Shape,
            )?)?;
            *self.face_state.write() = face.clone().into_shape();
        }
        Ok(face)
    }

    fn filter_small_area_wires(
        &self,
        face: &Face,
        replacement: &mut Face,
    ) -> Result<(i32, i32), Failure> {
        let (mut removed, mut retained) = (0, 0);
        for child in ShapeIterator::new(&face.clone().into_shape(), false, true) {
            if !small_area_child_eligible(&child)? {
                continue;
            }
            let wire = topods::wire(&child)?;
            if self.is_negligible_wire(&wire, face)? {
                removed += 1;
            } else {
                Builder.add(replacement, &wire)?;
                retained += 1;
            }
        }
        Ok((removed, retained))
    }

    fn is_negligible_wire(&self, wire: &Wire, face: &Face) -> Result<bool, Failure> {
        let analyzer = ShapeAnalysisWire::from_wire(wire, face, self.precision())?;
        if !analyzer.check_small_area(wire)? {
            return Ok(false);
        }
        self.send_warning(
            &wire.clone().into_shape(),
            &Msg::from_key(&AsciiString::from("FixAdvFace.FixSmallAreaWire.MSG0")),
        );
        Ok(true)
    }

    fn remove_entire_small_face(&self, face: &Face, enabled: bool) -> Result<bool, Failure> {
        if !enabled {
            return Ok(false);
        }
        if let Some(context) = self.context().get() {
            ReShape::remove(context, &face.clone().into_shape())?;
        }
        Ok(false)
    }
}

fn small_area_child_eligible(child: &Shape) -> Result<bool, Failure> {
    // Preserve the native conjunction: a manifold non-wire is cast and may fail.
    Ok(child.shape_type()? == ShapeEnum::Wire
        || matches!(
            child.orientation(),
            Orientation::Forward | Orientation::Reversed
        ))
}
