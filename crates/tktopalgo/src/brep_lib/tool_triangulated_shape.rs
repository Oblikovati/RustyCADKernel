// Rust translation of BRepLib_ToolTriangulatedShape.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepLib_ToolTriangulatedShape`: the normals of the triangulation of a face.
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{self, Face};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkgeombase::geom_lib;
use rustycad_tkmath::gp::{Direction3, StandardDirection3, Xyz};
use rustycad_tkmath::poly::{self, Connect, Triangulation};
use rustycad_tkmath::toploc::Location;

/// `BRepLib_ToolTriangulatedShape`: a static tool.
///
/// ```
/// use rustycad_tkbrep::topods::Face;
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tktopalgo::brep_lib::BRepLibToolTriangulatedShape;
/// // A null triangulation is left as it is.
/// BRepLibToolTriangulatedShape::compute_normals(&Face::default(), &Handle::default())?;
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct BRepLibToolTriangulatedShape;

impl BRepLibToolTriangulatedShape {
    /// `ComputeNormals(theFace, theTris)`: with a new `Poly_Connect`;
    /// `BRepLibToolTriangulatedShape::compute_normals(&face, &tris)?`.
    pub fn compute_normals(face: &Face, tris: &Handle<Triangulation>) -> Result<(), Failure> {
        let mut connect = Connect::default();
        Self::compute_normals_with_connect(face, tris, &mut connect)
    }

    /// `ComputeNormals(theFace, theTris, thePolyConnect)`: the normals of a
    /// triangulation without them: the surface normals at the UV nodes (the face
    /// location ignored), or where the normal is undefined the mean of the unit normals
    /// of the triangles around the node (Z when that mean vanishes); without UV nodes or
    /// surface, `Poly::ComputeNormals`. `thePolyConnect` is loaded on the triangulation
    /// when first needed; `BRepLibToolTriangulatedShape::compute_normals_with_connect(&face, &tris, &mut connect)?`.
    pub fn compute_normals_with_connect(
        face: &Face,
        tris: &Handle<Triangulation>,
        connect: &mut Connect,
    ) -> Result<(), Failure> {
        let Some(mesh) = tris.get() else {
            return Ok(());
        };
        if mesh.has_normals() {
            return Ok(());
        }
        // take in face the surface location
        let zero_face = topods::face(&face.located(&Location::default(), false)?)?;
        let surface: Handle<dyn GeomSurface> = BRepTool::surface(&zero_face)?;
        if !mesh.has_uv_nodes() || surface.is_null() {
            // compute normals by averaging triangulation normals sharing the same vertex
            return poly::compute_normals(mesh);
        }
        mesh.add_normals()?;
        let mut norm = Direction3::from_standard(StandardDirection3::Z);
        for node in 1..=mesh.nb_nodes() {
            // try to retrieve normal from real surface first, when UV coordinates are available
            let code = geom_lib::norm_estim(
                &surface,
                mesh.uv_node(node)?,
                precision::confusion(),
                &mut norm,
            )?;
            if code > 1 {
                norm = flat_normal(tris, connect, node)?;
            }
            mesh.set_normal(node, norm)?;
        }
        Ok(())
    }
}

/// "compute flat normals": the mean of the unit normals of the triangles around `node`
/// (those at least Precision::Confusion() long), Z when it is shorter.
fn flat_normal(
    tris: &Handle<Triangulation>,
    connect: &mut Connect,
    node: i32,
) -> Result<Direction3, Failure> {
    if connect.triangulation().object_address() != tris.object_address() {
        connect.load(tris)?;
    }
    let mesh: &Triangulation = tris
        .get()
        .expect("non-null triangulation checked by the caller");
    let tol: f64 = precision::confusion();
    let mut eq_plan = Xyz::new(0.0, 0.0, 0.0);
    connect.initialize(node)?;
    while connect.more() {
        let (n0, n1, n2) = mesh.triangle(connect.value())?.get();
        let v1: Xyz = mesh.node(n1)?.xyz() - mesh.node(n0)?.xyz();
        let v2: Xyz = mesh.node(n2)?.xyz() - mesh.node(n1)?.xyz();
        let vv: Xyz = v1 ^ v2;
        let modulus: f64 = vv.modulus();
        if modulus >= tol {
            eq_plan += vv / modulus;
        }
        connect.next()?;
    }
    if eq_plan.modulus() > tol {
        return Direction3::from_xyz(eq_plan);
    }
    Ok(Direction3::from_standard(StandardDirection3::Z))
}
