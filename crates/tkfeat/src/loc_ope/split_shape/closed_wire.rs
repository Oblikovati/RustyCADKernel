// Rust translation of LocOpe_SplitShape::AddClosedWire of
// ModelingAlgorithms/TKFeat/LocOpe/LocOpe_SplitShape.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `AddClosedWire`: a closed wire inside a face makes a new face bounded by it, and the
//! old face gets it as a hole.
use super::statics::{is_inside, is_inside_wire};
use super::{SplitShape, to_list, to_vec};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Face, Wire};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{self, Orientation, ShapeEnum, State};
use rustycad_tktopalgo::brep_top_adaptor::BRepTopAdaptorFClass2d;

impl SplitShape {
    /// The face of `f` bounded by `w`, the wire reversed when it bounds a hole, and the
    /// orientation the wire got.
    fn face_on_wire(w: &Wire, f: &Face) -> Result<(Face, Orientation), Failure> {
        let b = BRepBuilder::default();
        let mut or_wire = w.orientation();
        let mut new_face = topods::face(&f.empty_copied()?)?;
        new_face.set_orientation(Orientation::Forward);
        b.add(&mut new_face, w)?;
        let classif = BRepTopAdaptorFClass2d::new(&new_face, precision::p_confusion())?;
        if classif.perform_infinite_point()? == State::In {
            // le wire donne defini un trou
            new_face = topods::face(&f.empty_copied()?)?;
            new_face.set_orientation(Orientation::Forward);
            or_wire = top_abs::reverse(or_wire);
            b.add(&mut new_face, &w.oriented(or_wire))?;
        }
        Ok((new_face, or_wire))
    }

    /// `AddClosedWire(W, F)`: the descendant of `f` holding `w` replaced by itself with
    /// `w` as a hole and the face inside `w` (the wires of the descendant inside `w` go
    /// to the new face); false when no descendant holds `w`.
    pub(super) fn add_closed_wire(&mut self, w: &Wire, f: &Face) -> Result<bool, Failure> {
        // On cherche la face descendante de F qui continent le wire
        let mut lf = to_vec(self.list_of(f)?);
        let mut found = None;
        for (k, s) in lf.iter().enumerate() {
            if is_inside(&topods::face(s)?, w)? {
                found = Some(k);
                break;
            }
        }
        let Some(k) = found else {
            return Ok(false);
        };
        let b = BRepBuilder::default();
        let (mut new_face, or_wire) = Self::face_on_wire(w, f)?;
        let mut face_ref = topods::face(&lf.remove(k))?;
        face_ref.set_orientation(Orientation::Forward);
        *self.list_mut(f)? = to_list(lf);
        let mut new_ref = topods::face(&face_ref.empty_copied()?)?;
        new_ref.set_orientation(Orientation::Forward);
        // On suppose que les edges du wire ont des courbes 2d.
        // Comme on ne change pas de surface de base, pas besoin d`UpdateEdge.
        for x in Explorer::new(
            &face_ref.oriented(Orientation::Forward),
            ShapeEnum::Wire,
            ShapeEnum::Shape,
        )? {
            let wir = topods::wire(&x)?;
            if is_inside_wire(f, &wir, w)? {
                b.add(&mut new_face, &wir)?;
            } else {
                b.add(&mut new_ref, &wir)?;
            }
        }
        b.add(&mut new_ref, &w.oriented(top_abs::reverse(or_wire)))?;
        let lf = self.list_mut(f)?;
        lf.append(new_ref.into_shape());
        lf.append(new_face.into_shape());
        Ok(true)
    }
}
