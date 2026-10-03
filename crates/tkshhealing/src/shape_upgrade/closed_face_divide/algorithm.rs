use super::ShapeUpgradeClosedFaceDivide;
use crate::shape_extend::{ShapeExtend, ShapeExtendStatus};
use crate::shape_fix::{FixRoot, ShapeFixComposeShell};
use crate::shape_upgrade::FaceDivider;
use crate::shape_upgrade::{SplitSurface, SurfaceSplitValues};
use rustycad_tkbrep::{
    brep::BRepTool,
    brep_tools::ReShape,
    topexp::Explorer,
    topods::{self, Face},
};
use rustycad_tkernel::{
    ncollection::HSequence,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::toploc::Location;

pub(super) fn split_surface(
    tool: &ShapeUpgradeClosedFaceDivide,
    _area: f64,
) -> Result<bool, Failure> {
    let splitter_handle = tool.split_surface_tool().cast::<dyn SplitSurface>();
    let Some(splitter) = splitter_handle.get() else {
        return Ok(false);
    };
    let Some(face) = current_face(tool)? else {
        return Ok(false);
    };
    let source = tool.face_divide_data().with(|fields| fields.face.clone());
    let Some(bounds) = bounded_source_face(&source)? else {
        return Ok(false);
    };
    divide_bounded_face(tool, splitter, face, bounds)
}

fn divide_bounded_face(
    tool: &ShapeUpgradeClosedFaceDivide,
    splitter: &dyn SplitSurface,
    face: Face,
    bounds: (f64, f64, f64, f64),
) -> Result<bool, Failure> {
    let (surface, location) = BRepTool::surface_with_location(&face)?;
    let cuts = choose_cut_values(tool, &face, &surface, bounds)?;
    if !initialize_splitter(tool, splitter, &surface, bounds, cuts)? {
        return Ok(false);
    }
    compose_and_recurse(tool, splitter, &face, &location)
}

fn current_face(tool: &ShapeUpgradeClosedFaceDivide) -> Result<Option<Face>, Failure> {
    let result = tool.face_divide_data().with(|fields| fields.result.clone());
    if result.is_null() || result.shape_type()? != ShapeEnum::Face {
        add_status(tool, ShapeExtendStatus::Fail3);
        return Ok(None);
    }
    Ok(Some(topods::face(&result)?))
}

fn bounded_source_face(source: &Face) -> Result<Option<(f64, f64, f64, f64)>, Failure> {
    let bounds = crate::shape_analysis::ShapeAnalysis::get_face_uv_bounds(source)?;
    if [bounds.0, bounds.1, bounds.2, bounds.3]
        .into_iter()
        .any(rustycad_tkernel::precision::is_infinite)
    {
        return Ok(None);
    }
    Ok(Some(bounds))
}

fn choose_cut_values(
    tool: &ShapeUpgradeClosedFaceDivide,
    face: &Face,
    surface: &rustycad_tkernel::standard::Handle<dyn rustycad_tkg3d::geom::GeomSurface>,
    bounds: (f64, f64, f64, f64),
) -> Result<Option<(bool, SurfaceSplitValues)>, Failure> {
    if let Some(values) = super::seam::seam_cuts(face, tool.nb_split_points())? {
        return Ok(Some(values));
    }
    super::closure::closure_cuts(surface, bounds, tool.nb_split_points())
}

fn initialize_splitter(
    tool: &ShapeUpgradeClosedFaceDivide,
    splitter: &dyn SplitSurface,
    surface: &rustycad_tkernel::standard::Handle<dyn rustycad_tkg3d::geom::GeomSurface>,
    bounds: (f64, f64, f64, f64),
    cuts: Option<(bool, SurfaceSplitValues)>,
) -> Result<bool, Failure> {
    let Some((split_u, values)) = cuts else {
        return Ok(false);
    };
    splitter.init_surface_bounds(surface, bounds, 0.0)?;
    if split_u {
        splitter.set_u_split_values(&values)?;
    } else {
        splitter.set_v_split_values(&values)?;
    }
    let segment = tool.face_divide_data().with(|fields| fields.segment);
    splitter.perform(segment)?;
    Ok(splitter.status_flag(ShapeExtendStatus::Done))
}

fn compose_and_recurse(
    tool: &ShapeUpgradeClosedFaceDivide,
    splitter: &dyn SplitSurface,
    face: &Face,
    location: &Location,
) -> Result<bool, Failure> {
    let composed = compose_patches(tool, splitter, face, location)?;
    let context = tool.face_divide_base().context();
    add_status(tool, ShapeExtendStatus::Done2);
    recurse_result_faces(tool, &composed, &context)?;
    let applied = context.apply(&composed, ShapeEnum::Shape)?;
    tool.face_divide_data()
        .with_mut(|fields| fields.result = applied);
    Ok(true)
}

fn compose_patches(
    tool: &ShapeUpgradeClosedFaceDivide,
    splitter: &dyn SplitSurface,
    face: &Face,
    location: &Location,
) -> Result<rustycad_tkbrep::topods::Shape, Failure> {
    let composer = ShapeFixComposeShell::new();
    composer.init(
        &splitter.result_surfaces(),
        location,
        face,
        tool.precision(),
    )?;
    composer.set_max_tolerance(tool.max_tolerance());
    composer.set_context(&tool.face_divide_base().context());
    composer.perform()?;
    record_composer_status(tool, &composer);
    Ok(composer.result())
}

fn record_composer_status(tool: &ShapeUpgradeClosedFaceDivide, composer: &ShapeFixComposeShell) {
    if composer.status(ShapeExtendStatus::Fail) || !composer.status(ShapeExtendStatus::Done) {
        add_status(tool, ShapeExtendStatus::Fail2);
    }
}

fn recurse_result_faces(
    tool: &ShapeUpgradeClosedFaceDivide,
    result: &rustycad_tkbrep::topods::Shape,
    context: &Handle<crate::shape_build::ShapeBuildReShape>,
) -> Result<(), Failure> {
    let mut faces = Explorer::new(result, ShapeEnum::Face, ShapeEnum::Shape)?;
    while faces.more() {
        let source = faces.current().clone();
        recurse_result_face(tool, &source, context)?;
        faces.next()?;
    }
    Ok(())
}

fn recurse_result_face(
    tool: &ShapeUpgradeClosedFaceDivide,
    source: &rustycad_tkbrep::topods::Shape,
    context: &Handle<crate::shape_build::ShapeBuildReShape>,
) -> Result<(), Failure> {
    let applied = context.apply(source, ShapeEnum::Shape)?;
    let face = topods::face(&applied)?;
    tool.face_divide_data()
        .with_mut(|fields| fields.result = face.clone().into_shape());
    if split_surface(tool, 0.0)? {
        let replacement = tool.result();
        context.replace(&face.clone().into_shape(), &replacement)?;
    }
    Ok(())
}

fn add_status(tool: &ShapeUpgradeClosedFaceDivide, status: ShapeExtendStatus) {
    let encoded = ShapeExtend::encode_status(status);
    tool.face_divide_data()
        .with_mut(|fields| fields.status |= encoded);
}

pub(super) fn split_values(values: &[f64]) -> SurfaceSplitValues {
    let sequence = Handle::new(HSequence::<f64>::default());
    for value in values {
        sequence.append(*value);
    }
    sequence
}

pub(super) fn interior_values(interval: (f64, f64), count: i32) -> Vec<f64> {
    let step = (interval.1 - interval.0) / f64::from(count + 1);
    let mut values = Vec::new();
    let mut value = interval.0 + step;
    for _ in 0..count {
        values.push(value);
        value += step;
    }
    values
}
