//! The source-face-only branch of `BRepOffset_MakeOffset::MakeOffsetShape`.
use super::{BRepOffsetAnalyse, BRepOffsetError, BRepOffsetMakeOffset, BRepOffsetOffsetFace};
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    brep_tools::Quilt,
    topexp::Explorer,
    topods::{Face, Shape, Shell},
};
use rustycad_tkernel::{precision, standard::Failure};
use rustycad_tkg3d::top_abs::ShapeEnum;

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
        if face_count(&self.face_component)? > 1 {
            return self.build_multi_face_shell();
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
        let image = offset_face(&face, offset)?;
        self.record_offset_history(&face, &image)?;
        self.result_shape = shell_from_face(image.face())?;
        add_spherical_corrections(&mut self.result_shape, &image, offset)?;
        self.error = Some(BRepOffsetError::NoError);
        self.done = true;
        Ok(())
    }

    fn build_multi_face_shell(&mut self) -> Result<(), Failure> {
        let config = self.configuration()?;
        reject_unsupported_build_options(config, self.closing_faces.len())?;
        let faces = source_faces(&self.face_component)?;
        let analyzer = analyze_face(&self.face_component, config.offset, config.tolerance)?;
        let shared_support = faces_share_support_surface(&faces)?;
        let images = offset_face_images(&faces, config.offset, shared_support)?;
        self.record_face_histories(&faces, &images)?;
        let patches = if shared_support {
            let image_refs = images.iter().collect::<Vec<_>>();
            super::correct_singular_face::correct_singular_faces(&image_refs, config.offset)?
        } else {
            let joins = super::make_offset_build_adjacent::arc_patches_for_supported_faces(
                &faces,
                &images,
                &analyzer,
                config.offset,
            )?;
            self.record_join_histories(&joins);
            let spheres = super::make_offset_build_spheres::vertex_sphere_joins(
                &faces,
                &joins,
                &analyzer,
                config.offset,
            )?;
            self.record_vertex_sphere_histories(&spheres);
            joins
                .into_iter()
                .map(|join| join.face)
                .chain(spheres.into_iter().map(|join| join.face))
                .collect()
        };
        self.result_shape = offset_result_shell(&self.initial_shape, &images, &patches)?;
        self.error = Some(BRepOffsetError::NoError);
        self.done = true;
        Ok(())
    }

    fn record_face_histories(
        &mut self,
        sources: &[Face],
        images: &[BRepOffsetOffsetFace],
    ) -> Result<(), Failure> {
        for (source, image) in sources.iter().zip(images) {
            self.record_offset_history(source, image)?;
        }
        Ok(())
    }

    fn record_join_histories(
        &mut self,
        joins: &[super::make_offset_build_adjacent::SharedSupportJoin],
    ) {
        for join in joins {
            let source = join.source_edge.clone().into_shape();
            let image = join.face.clone().into_shape();
            self.record_join_generated_face(&source, &image);
        }
    }
}

fn offset_result_shell(
    source: &Shape,
    images: &[BRepOffsetOffsetFace],
    patches: &[Face],
) -> Result<Shape, Failure> {
    let shell = shell_from_faces(images, patches)?;
    if source.shape_type()? != ShapeEnum::Solid {
        return Ok(shell);
    }
    solid_from_shell(&shell)
}

fn solid_from_shell(shell: &Shape) -> Result<Shape, Failure> {
    let mut solid = rustycad_tkbrep::topods::Solid::default();
    let builder = BRepBuilder::default();
    builder.make_solid(&mut solid);
    builder.add(&mut solid, shell)?;
    Ok(solid.into_shape())
}

fn face_count(component: &Shape) -> Result<usize, Failure> {
    Ok(Explorer::new(component, ShapeEnum::Face, ShapeEnum::Shape)?.count())
}

fn source_faces(component: &Shape) -> Result<Vec<Face>, Failure> {
    Explorer::new(component, ShapeEnum::Face, ShapeEnum::Shape)
        .map(|faces| faces.map(Face::unchecked).collect())
}

fn faces_share_support_surface(faces: &[Face]) -> Result<bool, Failure> {
    let first = faces.first().ok_or_else(no_offset_face)?;
    let (surface, location) = BRepTool::surface_with_location(first)?;
    if !location.is_identity() {
        return Ok(false);
    }
    for face in &faces[1..] {
        let (candidate, candidate_location) = BRepTool::surface_with_location(face)?;
        if candidate.object_address() != surface.object_address()
            || !candidate_location.is_identity()
        {
            return Ok(false);
        }
    }
    Ok(true)
}

fn offset_face_images(
    faces: &[Face],
    offset: f64,
    share_created_edges: bool,
) -> Result<Vec<BRepOffsetOffsetFace>, Failure> {
    if share_created_edges {
        return offset_faces_with_shared_edges(faces, offset);
    }
    faces
        .iter()
        .map(|face| {
            BRepOffsetOffsetFace::from_face_with_created(
                face,
                offset,
                &mut super::offset_face::FaceCreatedImages::default(),
            )
        })
        .collect()
}

fn offset_faces_with_shared_edges(
    faces: &[Face],
    offset: f64,
) -> Result<Vec<BRepOffsetOffsetFace>, Failure> {
    let mut created = super::offset_face::FaceCreatedImages::default();
    faces
        .iter()
        .map(|face| BRepOffsetOffsetFace::from_face_with_created(face, offset, &mut created))
        .collect()
}

fn shell_from_faces(images: &[BRepOffsetOffsetFace], patches: &[Face]) -> Result<Shape, Failure> {
    let mut shell = Shell::default();
    let builder = BRepBuilder::default();
    builder.make_shell(&mut shell);
    add_offset_faces(&builder, &mut shell, images, patches)?;
    let shape = shell.into_shape();
    shape.set_closed(BRepTool::is_closed(&shape)?)?;
    Ok(shape)
}

fn add_offset_faces(
    builder: &BRepBuilder,
    shell: &mut Shell,
    images: &[BRepOffsetOffsetFace],
    patches: &[Face],
) -> Result<(), Failure> {
    for image in images {
        builder.add(shell, &image.face().clone().into_shape())?;
    }
    for patch in patches {
        builder.add(shell, &patch.clone().into_shape())?;
    }
    Ok(())
}

fn no_offset_face() -> Failure {
    Failure::with_message(
        "Standard_NoSuchObject",
        "standard offset requires at least one face",
    )
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

fn offset_face(face: &Face, offset: f64) -> Result<BRepOffsetOffsetFace, Failure> {
    let (_, location) = BRepTool::surface_with_location(face)?;
    if !location.is_identity() {
        return Err(not_implemented("single face has a nonidentity location"));
    }
    regular_offset_face(face, offset)
}

fn regular_offset_face(face: &Face, offset: f64) -> Result<BRepOffsetOffsetFace, Failure> {
    BRepOffsetOffsetFace::from_face(face, offset)
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

fn add_spherical_corrections(
    result: &mut Shape,
    image: &BRepOffsetOffsetFace,
    offset: f64,
) -> Result<(), Failure> {
    let patches = super::correct_singular_face::correct_singular_face(image, offset)?;
    let builder = BRepBuilder::default();
    result.set_free(true)?;
    for patch in patches {
        builder.add(result, &patch.into_shape())?;
    }
    Ok(())
}

fn not_implemented(reason: &str) -> Failure {
    let message = format!(
        "BRepOffset_MakeOffset::MakeOffsetShape {reason}; expected a single supported face"
    );
    Failure::with_message("Standard_NotImplemented", &message)
}
