//! The source-face-only branch of `BRepOffset_MakeOffset::MakeOffsetShape`.
use super::{BRepOffsetAnalyse, BRepOffsetError, BRepOffsetMakeOffset};
use rustycad_tkbrep::{
    brep::BRepTool,
    brep_tools::Quilt,
    topexp::Explorer,
    topods::{Face, Shape, Shell},
};
use rustycad_tkernel::{precision, standard::Failure};
use rustycad_tkg3d::geom::GeomPlane;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Transform3;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;

impl BRepOffsetMakeOffset {
    /// Builds the parallel shell for one face with no shared edges or caps.
    /// `maker.make_offset_shape()?`.
    pub fn make_offset_shape(&mut self) -> Result<(), Failure> {
        self.done = false;
        self.result_shape = Shape::default();
        self.face_component =
            super::make_offset_check::face_component(&self.initial_shape)?.into_shape();
        if !self.check_input_data()? {
            return Ok(());
        }
        self.build_single_face_shell()
    }

    fn build_single_face_shell(&mut self) -> Result<(), Failure> {
        let config = self.configuration()?;
        reject_unsupported_build_options(config, self.closing_faces.len())?;
        let face = single_face(&self.face_component)?;
        let analyzer = analyze_face(&self.face_component, config.offset, config.tolerance)?;
        reject_tangent_edges(&analyzer, &face)?;
        let offset = face_offset(&self.face_offsets, &face, config.offset);
        let image = offset_plane_face(&face, offset)?;
        self.result_shape = shell_from_face(&image)?;
        self.error = Some(BRepOffsetError::NoError);
        self.done = true;
        Ok(())
    }
}

fn reject_unsupported_build_options(
    config: super::BRepOffsetMakeOffsetConfiguration,
    closing_face_count: usize,
) -> Result<(), Failure> {
    if config.join_type != rustycad_tkmath::geom_abs::JoinType::Arc
        || config.intersection
        || config.self_intersection
        || config.thickening
        || config.remove_internal_edges
        || closing_face_count != 0
    {
        return Err(not_implemented(
            "standard offset options exceed the single-face path",
        ));
    }
    Ok(())
}

fn single_face(component: &Shape) -> Result<Face, Failure> {
    let mut faces = Explorer::new(component, ShapeEnum::Face, ShapeEnum::Shape)?;
    if !faces.more() {
        return Err(not_implemented("offset has no face"));
    }
    let face = faces.current().clone();
    faces.next()?;
    if faces.more() {
        return Err(not_implemented(
            "standard offset requires exactly one source face",
        ));
    }
    Ok(Face::unchecked(face))
}

fn analyze_face(shape: &Shape, offset: f64, tolerance: f64) -> Result<BRepOffsetAnalyse, Failure> {
    let angle_tolerance = 4.0
        * (tolerance / ((offset * 0.5).abs() + precision::confusion()))
            .min(1.0)
            .asin();
    let mut analyzer = BRepOffsetAnalyse::new();
    analyzer.set_offset_value(offset);
    analyzer.perform(shape, angle_tolerance)?;
    Ok(analyzer)
}

fn reject_tangent_edges(analyzer: &BRepOffsetAnalyse, face: &Face) -> Result<(), Failure> {
    let mut tangent_edges = Vec::new();
    analyzer.edges_on_face(
        face,
        rustycad_tkfillet::chfi_ds::TypeOfConcavity::Tangential,
        &mut tangent_edges,
    )?;
    if tangent_edges.is_empty() {
        return Ok(());
    }
    Err(not_implemented(
        "single-face offset unexpectedly has tangent edges",
    ))
}

fn face_offset(offsets: &[(Face, f64)], face: &Face, global: f64) -> f64 {
    offsets
        .iter()
        .find(|(candidate, _)| candidate.is_same(&face.clone().into_shape()))
        .map_or(global, |(_, offset)| *offset)
}

fn offset_plane_face(face: &Face, offset: f64) -> Result<Face, Failure> {
    let (surface, location) = BRepTool::surface_with_location(face)?;
    if !location.is_identity() {
        return Err(not_implemented(
            "single-plane face has a nonidentity location",
        ));
    }
    let plane = surface.cast::<GeomPlane>();
    let Some(plane) = plane.get() else {
        return Err(not_implemented("single-face path requires a plane surface"));
    };
    translate_plane_face(face, plane, offset)
}

fn translate_plane_face(face: &Face, plane: &GeomPlane, offset: f64) -> Result<Face, Failure> {
    let normal = plane.pln().position().direction();
    let translation = plane_translation(normal, oriented_offset(face, offset));
    let mut moved = rustycad_tktopalgo::brep_builder_api::BRepBuilderAPITransform::with_shape(
        &face.clone().into_shape(),
        translation,
        true,
        false,
    )?;
    Ok(Face::unchecked(moved.shape()?.clone()))
}

fn oriented_offset(face: &Face, offset: f64) -> f64 {
    if face.orientation() == rustycad_tkg3d::top_abs::Orientation::Reversed {
        -offset
    } else {
        offset
    }
}

fn plane_translation(normal: rustycad_tkmath::gp::Direction3, offset: f64) -> Transform3 {
    Transform3::translation([
        normal.x() * offset,
        normal.y() * offset,
        normal.z() * offset,
    ])
}

fn shell_from_face(face: &Face) -> Result<Shape, Failure> {
    let mut quilt = Quilt::new();
    quilt.add(&face.clone().into_shape())?;
    let shells = quilt.shells()?;
    let mut explorer = Explorer::new(&shells, ShapeEnum::Shell, ShapeEnum::Shape)?;
    if !explorer.more() {
        return Err(not_implemented("quilt made no shell"));
    }
    let shell = explorer.current().clone();
    explorer.next()?;
    if explorer.more() {
        return Err(not_implemented(
            "single-face offset produced multiple shells",
        ));
    }
    Ok(Shell::unchecked(shell).into_shape())
}

fn not_implemented(reason: &str) -> Failure {
    let message = format!(
        "BRepOffset_MakeOffset::MakeOffsetShape {reason}; expected a single supported face"
    );
    Failure::with_message("Standard_NotImplemented", &message)
}
