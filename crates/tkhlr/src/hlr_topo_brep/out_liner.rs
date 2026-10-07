//! HLRTopoBRep_OutLiner.hxx/.lxx/.cxx, OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::{DSFiller, Data, MapOfShapeTool};
use crate::contap::ContapContour;
use crate::hlr_algo::Projector;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Compound, Shape, Shell};
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, SharedState, StandardTransient};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Direction3, Point3, Vector3};
use std::sync::RwLockWriteGuard;

/// HLRTopoBRep_OutLiner: a shape and its outlined copy, whose faces carry their
/// outlines and isolines as internal wires, with the data structure that records
/// them. A transient shared by handles: each field is behind its own lock and the
/// native reference accessors return write guards (docs/hlr-design.md section 5).
///
/// ```
/// use rustycad_tkhlr::hlr_topo_brep::OutLiner;
/// let outliner = OutLiner::new();
/// assert!(outliner.original_shape().is_null());
/// ```
pub struct OutLiner {
    base: StandardTransient,
    original_shape: SharedState<Shape>,
    out_lined_shape: SharedState<Shape>,
    ds: SharedState<Data>,
}
impl_transient!(OutLiner, "HLRTopoBRep_OutLiner", StandardTransient, base);

impl Default for OutLiner {
    fn default() -> Self {
        Self::new()
    }
}

impl OutLiner {
    /// HLRTopoBRep_OutLiner(): no shape; `OutLiner::new()`.
    pub fn new() -> Self {
        Self::with_shapes(&Shape::default(), &Shape::default())
    }
    /// HLRTopoBRep_OutLiner(OriSh); `OutLiner::with_original(&shape)`.
    pub fn with_original(ori_s: &Shape) -> Self {
        Self::with_shapes(ori_s, &Shape::default())
    }
    /// HLRTopoBRep_OutLiner(OriS, OutS); `OutLiner::with_shapes(&shape, &outlined)`.
    pub fn with_shapes(ori_s: &Shape, out_s: &Shape) -> Self {
        Self {
            base: StandardTransient::default(),
            original_shape: SharedState::new(ori_s.clone()),
            out_lined_shape: SharedState::new(out_s.clone()),
            ds: SharedState::new(Data::new()),
        }
    }
    /// OriginalShape(OriS); `outliner.set_original_shape(&shape)`.
    pub fn set_original_shape(&self, ori_s: &Shape) {
        *self.original_shape.write() = ori_s.clone();
    }
    /// OriginalShape(): a copy of the shape (the native reference); `outliner.original_shape()`.
    pub fn original_shape(&self) -> Shape {
        self.original_shape.read().clone()
    }
    /// OutLinedShape(OutS); `outliner.set_out_lined_shape(&shape)`.
    pub fn set_out_lined_shape(&self, out_s: &Shape) {
        *self.out_lined_shape.write() = out_s.clone();
    }
    /// OutLinedShape(): a copy of the outlined shape; `outliner.out_lined_shape()`.
    pub fn out_lined_shape(&self) -> Shape {
        self.out_lined_shape.read().clone()
    }
    /// DataStructure(): the data structure, for edition; `outliner.data_structure()`.
    pub fn data_structure(&self) -> RwLockWriteGuard<'_, Data> {
        self.ds.write()
    }

    /// Fill(P, MST, nbIso): when there is a shape and no outlined shape yet, the
    /// outlines seen through `p` (from the eye in perspective, along the view
    /// direction otherwise) and `nb_iso` isolines per face, then the outlined shape;
    /// `outliner.fill(&projector, &mut tools, 0)?`.
    pub fn fill(
        &self,
        p: &Projector,
        mst: &mut MapOfShapeTool,
        nb_iso: i32,
    ) -> Result<(), Failure> {
        let original = self.original_shape();
        if original.is_null() || !self.out_lined_shape.read().is_null() {
            return Ok(());
        }
        let mut vecz = Vector3::new(0., 0., 1.);
        let mut tr = p.transformation();
        tr.invert()?;
        vecz.transform(tr);
        let mut fo = ContapContour::new();
        if p.perspective() {
            let mut eye = Point3::default();
            eye.set_xyz(*vecz.xyz() * p.focus()?);
            fo.init_eye(eye);
        } else {
            let dir_z = Direction3::from_vector(vecz)?;
            fo.init_direction(Vector3::from(dir_z))?;
        }
        DSFiller::insert(&original, &mut fo, &mut self.ds.write(), mst, nb_iso)?;
        self.build_shape(mst)
    }

    // BuildShape: a compound of the shells of the original shape rebuilt face by face
    // (each face once), then the faces outside a shell, then the free edges.
    fn build_shape(&self, mst: &mut MapOfShapeTool) -> Result<(), Failure> {
        let original = self.original_shape();
        let b = BRepBuilder::default();
        let mut out = Compound::default();
        b.make_compound(&mut out);
        let mut out = Shape::from(out);
        let mut shape_map = MapOfShape::default();
        let mut exshell = Explorer::new(&original, ShapeEnum::Shell, ShapeEnum::Shape)?;
        while exshell.more() {
            let mut the_shell = Shell::default();
            b.make_shell(&mut the_shell);
            the_shell.set_closed(exshell.current().closed()?)?;
            let mut the_shell = Shape::from(the_shell);
            let mut exface = Explorer::new(exshell.current(), ShapeEnum::Face, ShapeEnum::Shape)?;
            while exface.more() {
                if shape_map.add(exface.current().clone()) {
                    self.process_face(&topods::face(exface.current())?, &mut the_shell, mst)?;
                }
                exface.next()?;
            }
            b.add(&mut out, &the_shell)?;
            exshell.next()?;
        }
        let mut exface = Explorer::new(&original, ShapeEnum::Face, ShapeEnum::Shell)?;
        while exface.more() {
            if shape_map.add(exface.current().clone()) {
                self.process_face(&topods::face(exface.current())?, &mut out, mst)?;
            }
            exface.next()?;
        }
        let mut exedge = Explorer::new(&original, ShapeEnum::Edge, ShapeEnum::Face)?;
        while exedge.more() {
            b.add(&mut out, exedge.current())?;
            exedge.next()?;
        }
        *self.out_lined_shape.write() = out;
        Ok(())
    }
}

mod process_face;
