//! BRepMesh_IncrementalMesh twin of tkmesh_probe_incremental.cxx: one shape per
//! surface type of the algorithm factory, faces reported by counts and an FNV-1a hash.
use super::super::{flag, int};
use rustycad_tkbrep::{
    brep::BRepTool,
    topexp::Explorer,
    topods::{self, Face, Shape},
};
use rustycad_tkernel::{
    message::ProgressRange,
    ncollection::{Array1, Array2},
    standard::{Failure, Handle},
};
use rustycad_tkg3d::{
    geom::{GeomBezierCurve, GeomBezierSurface, GeomCurve, GeomSurface},
    top_abs::ShapeEnum,
};
use rustycad_tkmath::{
    gp::{Axis1, Direction3, Point3, Vector3},
    poly::mesh_purpose,
};
use rustycad_tkmesh::{
    brep_mesh::BRepMeshDiscretRoot, brep_mesh::BRepMeshIncrementalMesh, imesh_tools::MeshParameters,
};
use rustycad_tkprim::brep_prim_api::{
    BRepPrimAPIMakeBox, BRepPrimAPIMakeCone, BRepPrimAPIMakeCylinder, BRepPrimAPIMakePrism,
    BRepPrimAPIMakeRevol, BRepPrimAPIMakeSphere, BRepPrimAPIMakeTorus,
};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
use rustycad_tktopalgo::brep_lib::{BRepLibMakeEdge, BRepLibMakeFace};

/// The native FNV-1a over the little-endian bytes of each value.
struct IncrementalHash(u64);

impl IncrementalHash {
    fn add(&mut self, bits: u64) {
        for i in 0..8 {
            self.0 ^= (bits >> (8 * i)) & 0xFF;
            self.0 = self.0.wrapping_mul(1_099_511_628_211);
        }
    }
    fn real(&mut self, value: f64) {
        self.add(value.to_bits());
    }
    fn integer(&mut self, value: i32) {
        self.add(i64::from(value) as u64);
    }
}

fn bezier_edge() -> Result<Shape, Failure> {
    let mut poles: Array1<'_, Point3> = Array1::with_bounds(1, 4).map_err(array_failure)?;
    for (i, (x, z)) in [(2.0, 0.0), (4.0, 1.0), (3.0, 3.0), (5.0, 4.0)]
        .into_iter()
        .enumerate()
    {
        poles[i as i32 + 1] = Point3::new(x, 0.0, z);
    }
    let curve: Handle<dyn GeomCurve> = Handle::new(GeomBezierCurve::new(&poles)?).cast();
    Ok(BRepLibMakeEdge::from_curve(&curve)?.edge()?.into_shape())
}

fn bezier_face() -> Result<Shape, Failure> {
    let mut poles: Array2<'_, Point3> = Array2::with_bounds(1, 3, 1, 3).map_err(array_failure)?;
    for i in 1..=3 {
        for j in 1..=3 {
            let z = f64::from((i + j) % 3) * 0.75;
            *poles.change_value(i, j).map_err(array_failure)? =
                Point3::new(f64::from(i) * 2.0, f64::from(j) * 2.0, z);
        }
    }
    let surface: Handle<dyn GeomSurface> = Handle::new(GeomBezierSurface::new(&poles)?).cast();
    Ok(BRepLibMakeFace::from_surface(&surface, 1.0e-7)?
        .face()?
        .into_shape())
}

fn array_failure(error: rustycad_tkernel::ncollection::ArrayError) -> Failure {
    Failure::with_message("Standard_RangeError", &format!("{error}"))
}

fn shape(kind: i32) -> Result<Shape, Failure> {
    Ok(match kind {
        0 => BRepPrimAPIMakeBox::from_sizes(10.0, 10.0, 10.0)?
            .shape()?
            .clone(),
        1 => BRepPrimAPIMakeCylinder::new(5.0, 10.0).shape()?.clone(),
        2 => BRepPrimAPIMakeCone::new(10.0, 5.0, 10.0)?.shape()?.clone(),
        3 => BRepPrimAPIMakeSphere::new(10.0)?.shape()?.clone(),
        4 => BRepPrimAPIMakeTorus::new(10.0, 3.0)?.shape()?.clone(),
        5 => BRepPrimAPIMakePrism::new(&bezier_edge()?, Vector3::new(0.0, 6.0, 0.0), false, true)?
            .shape()?
            .clone(),
        6 => {
            let axis = Axis1::new(Point3::default(), Direction3::new(0.0, 0.0, 1.0)?);
            BRepPrimAPIMakeRevol::new(&bezier_edge()?, axis, 3.0, false)?
                .shape()?
                .clone()
        }
        _ => bezier_face()?,
    })
}

fn face_report(face: &Face) -> Result<(), Failure> {
    let triangulation = BRepTool::triangulation(face, mesh_purpose::NONE)?.0;
    flag(!triangulation.is_null());
    if triangulation.is_null() {
        return Ok(());
    }
    int(triangulation.nb_nodes());
    int(triangulation.nb_triangles());
    let mut hash = IncrementalHash(1_469_598_103_934_665_603);
    for i in 1..=triangulation.nb_nodes() {
        let point = triangulation.node(i)?;
        hash.real(point.x());
        hash.real(point.y());
        hash.real(point.z());
        if triangulation.has_uv_nodes() {
            let uv = triangulation.uv_node(i)?;
            hash.real(uv.x());
            hash.real(uv.y());
        }
    }
    for i in 1..=triangulation.nb_triangles() {
        let (a, b, c) = triangulation.triangle(i)?.get();
        hash.integer(a);
        hash.integer(b);
        hash.integer(c);
    }
    print!("{} ", hash.0);
    Ok(())
}

fn run(kind: i32, deflection: i32, mode: i32) -> Result<(), Failure> {
    let shape = shape(kind)?;
    let parameters = MeshParameters {
        deflection: if deflection == 0 { 0.5 } else { 0.1 },
        internal_vertices_mode: mode == 1,
        enable_control_surface_deflection_all_surfaces: mode == 2,
        relative: mode == 3,
        ..MeshParameters::default()
    };
    let mesher =
        BRepMeshIncrementalMesh::with_parameters(&shape, &parameters, &ProgressRange::default())?;
    print!("OK ");
    flag(mesher.is_done());
    int(mesher.get_status_flags());
    let mut faces = 0;
    let mut explorer = Explorer::new(&shape, ShapeEnum::Face, ShapeEnum::Shape)?;
    while explorer.more() {
        face_report(&topods::face(explorer.current())?)?;
        explorer.next()?;
        faces += 1;
    }
    int(faces);
    Ok(())
}

/// `mesh_incremental_shape <kind> <deflection> <mode>`.
pub fn incremental_shape_case(values: &[i32]) {
    if let Err(failure) = run(values[0], values[1], values[2]) {
        print!("{} ", failure.exception_type());
    }
    println!();
}

/// `mesh_triangulator <kind>`: the polygons of tkmesh_probe_incremental.cxx.
pub fn triangulator_case(kind: i32) {
    use rustycad_tkmath::gp::Xyz;
    use rustycad_tkmesh::brep_mesh::BRepMeshTriangulator;
    let xy = [
        (0.0, 0.0),
        (4.0, 0.0),
        (4.0, 3.0),
        (0.0, 3.0),
        (2.0, 5.0),
        (1.0, 1.0),
        (3.0, 1.0),
        (3.0, 2.0),
        (1.0, 2.0),
        (5.0, 1.5),
    ];
    let nodes: Vec<Xyz> = xy.iter().map(|(x, y)| Xyz::new(*x, *y, 0.25 * x)).collect();
    let outlines: [Vec<Vec<i32>>; 6] = [
        vec![vec![0, 1, 2]],
        vec![vec![0, 1, 2, 3]],
        vec![vec![0, 6, 2, 3]],
        vec![vec![0, 1, 9, 2, 4, 3]],
        vec![vec![0, 1, 2, 4, 3], vec![5, 8, 7, 6]],
        vec![vec![0, 5, 1, 9, 2, 7, 4, 8, 3]],
    ];
    let wires = &outlines[kind as usize];
    let normal = Direction3::new(-0.25, 0.0, 1.0).unwrap();
    let mut triangulator = BRepMeshTriangulator::new(&nodes, wires, normal).unwrap();
    let mut triangles = Vec::new();
    print!("OK ");
    flag(triangulator.perform(&mut triangles).unwrap());
    int(triangles.len() as i32);
    for triangle in triangles {
        let (a, b, c) = triangle.get();
        int(a);
        int(b);
        int(c);
    }
    println!();
}
