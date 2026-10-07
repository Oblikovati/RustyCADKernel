// Rust translation of BRepFill_Generator::Perform
// (ModelingAlgorithms/TKBool/BRepFill/BRepFill_Generator.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Perform`: the faces between consecutive sections, then SameParameter, the closure
//! of a shell between two punctual sections and the history of the copied wires.
use super::BRepFillGenerator;
use super::face_closure::set_pcurves;
use super::kpart_detect::KPART_NONE;
use super::lateral::{FaceFrame, first_lateral, last_lateral};
use super::ruled_face::{FaceSections, RuledSurface, ruled_surface};
use super::section_pair::{SectionEnds, SectionPair};
use crate::brep_fill::ThruSectionErrorStatus;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{self, Edge, Face, Wire};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomSurface;

/// Whether the walk goes on after a face, or stops with an error status.
enum FaceFlow {
    Continue,
    Stop,
}

impl BRepFillGenerator {
    /// `Perform()`: computes the shell. A section edge without a 3d curve stops the
    /// computation with the Null3DCurve status and the faces built so far;
    /// `generator.perform()?`.
    pub fn perform(&mut self) -> Result<(), Failure> {
        self.my_status = ThruSectionErrorStatus::Done;
        BRepBuilder::default().make_shell(&mut self.my_shell);
        let nb: usize = self.my_wires.len();
        let mut modif_wires = IndexedMapOfShape::default();
        let mut ends = SectionEnds::default();
        let mut first_wire: bool = true;
        for i in 1..nb {
            let wire1: Wire = topods::wire(&self.my_wires[i - 1])?;
            let wire2: Wire = topods::wire(&self.my_wires[i])?;
            let mut pair = SectionPair::new(&wire1, &wire2, i == 1, i == nb - 1, &mut ends)?;
            let mut tantque: bool = pair.starts();
            while tantque {
                if let FaceFlow::Stop = self.ruled_face(&mut pair, first_wire, &mut modif_wires)? {
                    return Ok(());
                }
                tantque = pair.continues();
            }
            first_wire = false;
        }
        self.finish_shell(ends)?;
        self.update_wire_history(&modif_wires)
    }

    /// One face of the walk, added to the shell; `Stop` with the Null3DCurve status.
    fn ruled_face(
        &mut self,
        pair: &mut SectionPair,
        first_wire: bool,
        modif_wires: &mut IndexedMapOfShape,
    ) -> Result<FaceFlow, Failure> {
        let mut s: FaceSections = self.face_sections(pair)?;
        pair.advance()?;
        let (surface, itype) = match ruled_surface(&s)? {
            RuledSurface::Built(surface, itype) => (surface, itype),
            RuledSurface::Null3DCurve => {
                self.my_status = ThruSectionErrorStatus::Null3DCurve;
                return Ok(FaceFlow::Stop);
            }
        };
        let builder = BRepBuilder::default();
        let mut face = Face::default();
        builder.make_face_on_surface(&mut face, &surface, precision::confusion())?;
        let frame: FaceFrame = face_frame(&surface, itype)?;
        let edge3: Edge = first_lateral(pair, &mut s, &frame)?;
        let edge4: Edge = last_lateral(pair, &mut s, &frame)?;
        if !self.my_mutable_input {
            self.copy_input_edges(&mut s, pair, first_wire, itype, modif_wires)?;
        }
        set_pcurves(&s, &face, &frame, (&edge3, &edge4))?;
        self.close_face(&s, &mut face, itype, (&edge3, &edge4))?;
        let red: Edge = if s.degen1 {
            s.edge1.clone()
        } else {
            pair.current_edge1()
        };
        self.record_generated(&s, itype, &red, &face);
        Ok(FaceFlow::Continue)
    }
}

/// The bounds of the face surface and the range of its lateral edges.
fn face_frame(surface: &Handle<dyn GeomSurface>, itype: i32) -> Result<FaceFrame<'_>, Failure> {
    let Some(geometry) = surface.get() else {
        return Err(topods_null_surface());
    };
    let (u1, u2, v1, v2) = geometry.bounds();
    let range: (f64, f64) = if itype == KPART_NONE {
        (v1, v2)
    } else {
        (0., 1.)
    };
    Ok(FaceFrame {
        surface,
        itype,
        u_first: u1,
        u_last: u2,
        v_first: v1,
        v_last: v2,
        range,
    })
}

/// Standard_NullObject for a null ruled surface (a crash natively).
fn topods_null_surface() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "BRepFill_Generator::Perform: the ruled surface is null; expected a surface",
        "BRepFill_Generator",
    )
}
