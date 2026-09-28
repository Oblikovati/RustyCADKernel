//! Distance-field point queries (`bvh_nearest`) and voxel grids of geometries
//! of triangulations (`bvh_field`).
use super::builders::query_builder;
use super::sort_build::{fill, nodes_out};
use super::{flag, scalar_out, vector_in, vector_out};
use rustycad_tkernel::ncollection::{Vec3, Vec4};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bvh::{
    BvhBox, BvhDistanceField, BvhFieldVector, BvhGeometry, BvhObject, BvhObjectTransient,
    BvhPrimitiveSetAccess, BvhSet, BvhTriangulation, array, direction_to_nearest_point,
    distance_to_box,
};

/// Selects the N = 3 / N = 4 field vector type from dimension and precision.
macro_rules! dispatch_field {
    ($dimension:expr, $precision:expr, $run:ident, $($argument:expr),*) => {
        match ($dimension as i32, $precision as i32) {
            (3, 0) => $run::<Vec3<f64>>($($argument),*),
            (_, 0) => $run::<Vec4<f64>>($($argument),*),
            (3, _) => $run::<Vec3<f32>>($($argument),*),
            _ => $run::<Vec4<f32>>($($argument),*),
        }
    };
}

/// v: point, a, b, c, box minimum, box maximum (four values each).
fn nearest_run<V: BvhFieldVector>(v: &[f64]) {
    let [point, a, b, c, min, max]: [V; 6] = std::array::from_fn(|i| vector_in(&v[4 * i..]));
    vector_out(&direction_to_nearest_point(&point, &a, &b, &c));
    scalar_out(distance_to_box(&point, &min, &max));
    println!();
}
pub fn bvh_nearest_case(v: &[f64]) {
    dispatch_field!(v[0], v[1], nearest_run, &v[2..]);
}

/// `ProbeBoxObject`: a geometric object that is not a triangulation.
#[derive(Debug, Default)]
struct ProbeBoxObject<V: BvhFieldVector> {
    bounds: BvhBox<V>,
    transient: BvhObjectTransient,
}
impl<V: BvhFieldVector> BvhObject<V> for ProbeBoxObject<V> {
    fn object_transient(&self) -> &BvhObjectTransient {
        &self.transient
    }
    fn object_transient_mut(&mut self) -> &mut BvhObjectTransient {
        &mut self.transient
    }
    fn object_box(&self) -> Result<BvhBox<V>, Failure> {
        Ok(self.bounds)
    }
}

/// The request flags: maxsize sign parallel geomdirty meshdirty prebuild extra builder.
struct FieldFlags {
    maximum_size: i32,
    sign: bool,
    parallel: bool,
    geometry_dirty: bool,
    mesh_dirty: bool,
    prebuild: bool,
    extra: bool,
    builder: i32,
}

/// Meshes tagged by element W = object index, plus the optional box object.
fn geometry_in<V: BvhFieldVector>(
    flags: &FieldFlags,
    counts: &[usize],
    mut coordinates: &[f64],
) -> BvhGeometry<V> {
    let mut geometry = BvhGeometry::<V>::default();
    if flags.builder != 0 {
        let kind = if flags.builder == 1 { 0 } else { 2 };
        geometry.set_builder(Some(query_builder(kind, 1, 32, 0)));
    }
    for (object, count) in counts.iter().enumerate() {
        let mut mesh = BvhTriangulation::<V>::default();
        fill(&mut mesh, coordinates, *count);
        coordinates = &coordinates[3 * count * V::DIMENSION..];
        for record in mesh.elements.iter_mut() {
            record.w = object as i32;
        }
        if flags.mesh_dirty {
            mesh.mark_dirty();
        }
        geometry.objects_mut().append(Some(Box::new(mesh)));
    }
    if flags.extra {
        let bounds = BvhBox::from_point(vector_in(&[0.5; 4]));
        let object = ProbeBoxObject {
            bounds,
            transient: BvhObjectTransient::default(),
        };
        geometry.objects_mut().append(Some(Box::new(object)));
    }
    geometry
}

/// `geometry.BVH()` and each triangulation's `BVH()`.
fn prebuild<V: BvhFieldVector>(geometry: &mut BvhGeometry<V>) {
    geometry.bvh().unwrap();
    for index in 0..geometry.size() {
        if let Some(mesh) = geometry
            .object_set_mut()
            .object_mut(index)
            .unwrap()
            .triangulation_mut()
        {
            mesh.bvh().unwrap();
        }
    }
}

fn field_out<V: BvhFieldVector>(field: &BvhDistanceField<V>) {
    print!(
        "{} {} {} ",
        field.dimension_x(),
        field.dimension_y(),
        field.dimension_z()
    );
    vector_out(field.corner_min());
    vector_out(field.corner_max());
    vector_out(field.voxel_size());
    for z in 0..field.dimension_z() {
        for y in 0..field.dimension_y() {
            for x in 0..field.dimension_x() {
                scalar_out(field.voxel(x, y, z).unwrap());
            }
        }
    }
}

/// Dirty flags and tags of the (reordered) objects, then the top-level tree.
fn geometry_out<V: BvhFieldVector>(geometry: &mut BvhGeometry<V>) {
    print!("{} ", flag(geometry.is_dirty()));
    for index in 0..geometry.size() {
        let object = geometry.object_set_mut().object_mut(index).unwrap();
        let is_dirty = object.is_dirty();
        let tag = object
            .triangulation_mut()
            .and_then(|mesh| array::value(&mesh.elements, 0).ok().map(|record| record.w))
            .unwrap_or(-1);
        print!("{tag} {} ", flag(is_dirty));
    }
    nodes_out(geometry.bvh().unwrap());
    println!();
}

/// v: maxsize sign parallel geomdirty meshdirty prebuild extra builder objects counts..., vertices.
fn field_run<V: BvhFieldVector>(v: &[f64]) {
    let [
        maximum_size,
        sign,
        parallel,
        geometry_dirty,
        mesh_dirty,
        pre,
        extra,
        builder,
        objects,
    ] = std::array::from_fn(|i| v[i] as i32);
    let flags = FieldFlags {
        maximum_size,
        sign: sign != 0,
        parallel: parallel != 0,
        geometry_dirty: geometry_dirty != 0,
        mesh_dirty: mesh_dirty != 0,
        prebuild: pre != 0,
        extra: extra != 0,
        builder,
    };
    let counts: Vec<usize> = v[9..9 + objects as usize]
        .iter()
        .map(|count| *count as usize)
        .collect();
    let mut geometry = geometry_in::<V>(&flags, &counts, &v[9 + objects as usize..]);
    if flags.geometry_dirty {
        geometry.mark_dirty();
    }
    if flags.prebuild {
        prebuild(&mut geometry);
    }
    let mut field = BvhDistanceField::<V>::new(flags.maximum_size, flags.sign).unwrap();
    field.set_parallel(flags.parallel);
    print!("{} ", flag(field.build(&mut geometry).unwrap()));
    field_out(&field);
    geometry_out(&mut geometry);
}
pub fn bvh_field_case(v: &[f64]) {
    dispatch_field!(v[0], v[1], field_run, &v[2..]);
}
