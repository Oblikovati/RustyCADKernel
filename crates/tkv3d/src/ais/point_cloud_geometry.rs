//! Native point-cloud copies preserve float-buffer quantization and array offsets.
use rustycad_tkernel::{
    ncollection::HArray1,
    quantity::Color,
    standard::{Failure, Handle},
};
use rustycad_tkmath::{
    bnd::BoundingBox3,
    gp::{Direction3, Point3},
};
use rustycad_tkservice::graphic3d::Graphic3dArrayOfPoints;
pub(super) fn point_cloud_bounds(
    points: &Handle<Graphic3dArrayOfPoints>,
) -> Result<BoundingBox3, Failure> {
    let mut bounds = BoundingBox3::default();
    let Some(points) = points.get() else {
        return Ok(bounds);
    };
    for index in 1..=points.vertex_number() {
        bounds.add_point(points.vertice(index)?);
    }
    Ok(bounds)
}
fn optional_length_matches<T: Send + Sync + 'static>(
    array: &Handle<HArray1<T>>,
    count: i32,
) -> bool {
    array
        .get()
        .is_none_or(|array| array.array1().length() == count)
}
pub(super) fn copy_point_cloud_arrays(
    coords: &Handle<HArray1<Point3>>,
    colors: &Handle<HArray1<Color>>,
    normals: &Handle<HArray1<Direction3>>,
) -> Result<Handle<Graphic3dArrayOfPoints>, Failure> {
    let Some(coords) = coords.get() else {
        return Ok(Handle::default());
    };
    let coords = coords.array1();
    let count = coords.length();
    if !optional_length_matches(colors, count) || !optional_length_matches(normals, count) {
        return Ok(Handle::default());
    }
    let points = Handle::new(Graphic3dArrayOfPoints::with_options(
        count,
        !colors.is_null(),
        !normals.is_null(),
    )?);
    for (offset, coord) in coords.iter().enumerate() {
        let index = points.add_vertex_pnt(coord)?;
        copy_cloud_attributes(&points, index, offset as i32, colors, normals)?;
    }
    Ok(points)
}
fn copy_cloud_attributes(
    points: &Graphic3dArrayOfPoints,
    index: i32,
    offset: i32,
    colors: &Handle<HArray1<Color>>,
    normals: &Handle<HArray1<Direction3>>,
) -> Result<(), Failure> {
    if let Some(colors) = colors.get() {
        let colors = colors.array1();
        points.set_vertex_color(
            index,
            *colors
                .value(colors.lower() + offset)
                .map_err(|e| e.into_failure("AIS_PointCloud::SetPoints colors"))?,
        )?;
    }
    if let Some(normals) = normals.get() {
        let normals = normals.array1();
        points.set_vertex_normal_dir(
            index,
            normals
                .value(normals.lower() + offset)
                .map_err(|e| e.into_failure("AIS_PointCloud::SetPoints normals"))?,
        )?;
    }
    Ok(())
}
