//! `BRepAlgoAPI_Section`: the section of two shapes, surfaces or planes.
use super::boolean_operation::BRepAlgoAPIBooleanOperation;
use crate::bop_algo::{BOPAlgoPaveFiller, Operation, SectionAttribute};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
use rustycad_tkmath::geom_abs;
use rustycad_tkmath::gp::Plane3;
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape, BRepBuilderAPIMakeShell,
};
use std::ops::{Deref, DerefMut};

/// The section of two shapes: the section edges and vertices, with optional approximation
/// and p-curves on the faces of each shape.
///
/// ```
/// use rustycad_tkbo::brep_algo_api::BRepAlgoAPISection;
/// use rustycad_tkbo::bop_algo::Operation;
/// assert_eq!(BRepAlgoAPISection::new().operation(), Operation::Section);
/// ```
pub struct BRepAlgoAPISection {
    operation: BRepAlgoAPIBooleanOperation,
    approximation: bool,
    pcurve_on_1: bool,
    pcurve_on_2: bool,
}

impl Default for BRepAlgoAPISection {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for BRepAlgoAPISection {
    type Target = BRepAlgoAPIBooleanOperation;
    fn deref(&self) -> &BRepAlgoAPIBooleanOperation {
        &self.operation
    }
}

impl DerefMut for BRepAlgoAPISection {
    fn deref_mut(&mut self) -> &mut BRepAlgoAPIBooleanOperation {
        &mut self.operation
    }
}

impl BRepAlgoAPISection {
    /// An empty section (`BRepAlgoAPI_Section()`); `BRepAlgoAPISection::new()`.
    pub fn new() -> Self {
        Self::init(BRepAlgoAPIBooleanOperation::new())
    }

    /// A section on the results of a performed filler (`BRepAlgoAPI_Section(PF)`).
    /// `BRepAlgoAPISection::with_filler(filler)`.
    pub fn with_filler(filler: BOPAlgoPaveFiller) -> Self {
        Self::init(BRepAlgoAPIBooleanOperation::with_filler(filler))
    }

    /// The section of two shapes, built when `perform_now` (`BRepAlgoAPI_Section(S1, S2,
    /// PerformNow)`, or `(S1, S2, PF, PerformNow)` with a filler).
    /// `BRepAlgoAPISection::from_shapes(&a, &b, None, true)?`.
    pub fn from_shapes(
        first: &Shape,
        second: &Shape,
        filler: Option<BOPAlgoPaveFiller>,
        perform_now: bool,
    ) -> Result<Self, Failure> {
        let operation =
            BRepAlgoAPIBooleanOperation::with_shapes(first, second, filler, Operation::Section);
        let mut section = Self::init(operation);
        if perform_now {
            section.build()?;
        }
        Ok(section)
    }

    /// The section of a shape and a plane (`BRepAlgoAPI_Section(Sh, Pl, PerformNow)`).
    /// `BRepAlgoAPISection::from_shape_plane(&shape, plane, true)?`.
    pub fn from_shape_plane(
        shape: &Shape,
        plane: Plane3,
        perform_now: bool,
    ) -> Result<Self, Failure> {
        Self::from_shapes(
            shape,
            &make_shape(&plane_surface(plane))?,
            None,
            perform_now,
        )
    }

    /// The section of a shape and a surface (`BRepAlgoAPI_Section(Sh, Sf, PerformNow)`).
    /// `BRepAlgoAPISection::from_shape_surface(&shape, &surface, true)?`.
    pub fn from_shape_surface(
        shape: &Shape,
        surface: &Handle<dyn GeomSurface>,
        perform_now: bool,
    ) -> Result<Self, Failure> {
        Self::from_shapes(shape, &make_shape(surface)?, None, perform_now)
    }

    /// The section of a surface and a shape (`BRepAlgoAPI_Section(Sf, Sh, PerformNow)`).
    /// `BRepAlgoAPISection::from_surface_shape(&surface, &shape, true)?`.
    pub fn from_surface_shape(
        surface: &Handle<dyn GeomSurface>,
        shape: &Shape,
        perform_now: bool,
    ) -> Result<Self, Failure> {
        Self::from_shapes(&make_shape(surface)?, shape, None, perform_now)
    }

    /// The section of two surfaces (`BRepAlgoAPI_Section(Sf1, Sf2, PerformNow)`).
    /// `BRepAlgoAPISection::from_surfaces(&first, &second, true)?`.
    pub fn from_surfaces(
        first: &Handle<dyn GeomSurface>,
        second: &Handle<dyn GeomSurface>,
        perform_now: bool,
    ) -> Result<Self, Failure> {
        Self::from_shapes(&make_shape(first)?, &make_shape(second)?, None, perform_now)
    }

    /// The section operation without approximation nor p-curves (`Init(false)`).
    fn init(mut operation: BRepAlgoAPIBooleanOperation) -> Self {
        operation.set_operation(Operation::Section);
        Self {
            operation,
            approximation: false,
            pcurve_on_1: false,
            pcurve_on_2: false,
        }
    }

    /// Replaces the first shape (`Init1(S1)`); `section.init1(&shape)`.
    pub fn init1(&mut self, shape: &Shape) {
        let mut arguments = List::default();
        arguments.append(shape.clone());
        self.operation.set_arguments(&arguments);
    }

    /// Replaces the first shape by a plane (`Init1(Pl)`); `section.init1_plane(plane)?`.
    pub fn init1_plane(&mut self, plane: Plane3) -> Result<(), Failure> {
        self.init1(&make_shape(&plane_surface(plane))?);
        Ok(())
    }

    /// Replaces the first shape by a surface (`Init1(Sf)`); `section.init1_surface(&surface)?`.
    pub fn init1_surface(&mut self, surface: &Handle<dyn GeomSurface>) -> Result<(), Failure> {
        self.init1(&make_shape(surface)?);
        Ok(())
    }

    /// Replaces the second shape (`Init2(S2)`); `section.init2(&shape)`.
    pub fn init2(&mut self, shape: &Shape) {
        let mut tools = List::default();
        tools.append(shape.clone());
        self.operation.set_tools(&tools);
    }

    /// Replaces the second shape by a plane (`Init2(Pl)`); `section.init2_plane(plane)?`.
    pub fn init2_plane(&mut self, plane: Plane3) -> Result<(), Failure> {
        self.init2(&make_shape(&plane_surface(plane))?);
        Ok(())
    }

    /// Replaces the second shape by a surface (`Init2(Sf)`); `section.init2_surface(&surface)?`.
    pub fn init2_surface(&mut self, surface: &Handle<dyn GeomSurface>) -> Result<(), Failure> {
        self.init2(&make_shape(surface)?);
        Ok(())
    }

    /// Approximates the section curves (`Approximation`); `section.approximation(true)`.
    pub fn approximation(&mut self, approximate: bool) {
        self.approximation = approximate;
    }

    /// Computes the p-curves of the section edges on the first shape (`ComputePCurveOn1`).
    /// `section.compute_pcurve_on1(true)`.
    pub fn compute_pcurve_on1(&mut self, compute: bool) {
        self.pcurve_on_1 = compute;
    }

    /// Computes the p-curves of the section edges on the second shape (`ComputePCurveOn2`).
    /// `section.compute_pcurve_on2(true)`.
    pub fn compute_pcurve_on2(&mut self, compute: bool) {
        self.pcurve_on_2 = compute;
    }

    /// Computes the section with the section attributes (`Build`, with `SetAttributes`).
    /// `section.build()?`.
    pub fn build(&mut self) -> Result<(), Failure> {
        let attribute =
            SectionAttribute::with_flags(self.approximation, self.pcurve_on_1, self.pcurve_on_2);
        self.operation.section_attribute = Some(attribute);
        self.operation.build()
    }

    /// The face of the first shape the section edge comes from (`HasAncestorFaceOn1`).
    /// `section.has_ancestor_face_on1(&edge)`.
    pub fn has_ancestor_face_on1(&self, edge: &Shape) -> Result<Option<Shape>, Failure> {
        Ok(self.ancestor_faces(edge)?.map(|(first, _)| first))
    }

    /// The face of the second shape the section edge comes from (`HasAncestorFaceOn2`).
    /// `section.has_ancestor_face_on2(&edge)`.
    pub fn has_ancestor_face_on2(&self, edge: &Shape) -> Result<Option<Shape>, Failure> {
        Ok(self.ancestor_faces(edge)?.map(|(_, second)| second))
    }

    /// The faces of the face/face interference the section edge comes from
    /// (`HasAncestorFaces`).
    fn ancestor_faces(&self, edge: &Shape) -> Result<Option<(Shape, Shape)>, Failure> {
        if edge.is_null() || edge.shape_type()? != rustycad_tkg3d::top_abs::ShapeEnum::Edge {
            return Ok(None);
        }
        let Some(structure) = self.ds_filler().and_then(BOPAlgoPaveFiller::data_structure) else {
            return Ok(None);
        };
        for record in structure.interf_ff_records().iter() {
            let (first, second) = record.indices();
            for curve in record.curves().iter() {
                for block in curve.pave_blocks() {
                    let index = block.lock().expect("pave-block lock").edge();
                    if index >= 0 && edge.is_same(structure.shape(index)?) {
                        return Ok(Some((
                            structure.shape(first)?.clone(),
                            structure.shape(second)?.clone(),
                        )));
                    }
                }
            }
        }
        Ok(None)
    }
}

fn plane_surface(plane: Plane3) -> Handle<dyn GeomSurface> {
    Handle::new(GeomPlane::from_pln(plane)).cast()
}

/// A face on a C2 (or smoother) surface, else a shell (`MakeShape`).
fn make_shape(surface: &Handle<dyn GeomSurface>) -> Result<Shape, Failure> {
    let continuity = surface.get().map(|stored| stored.continuity());
    if continuity.is_some_and(|value| value as i32 >= geom_abs::Shape::C2 as i32) {
        let mut face = BRepBuilderAPIMakeFace::from_surface(surface, precision::confusion())?;
        return Ok(face.shape()?.clone());
    }
    let mut shell = BRepBuilderAPIMakeShell::with_surface(surface, false)?;
    Ok(shell.shape()?.clone())
}
