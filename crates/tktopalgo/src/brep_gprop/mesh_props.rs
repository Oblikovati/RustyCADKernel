// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepGProp/BRepGProp_MeshProps.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGProp_MeshProps`: the global properties of a surface mesh.
use super::mesh_triangle::GPropsSums;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeombase::gprop::{GProps, h_operator};
use rustycad_tkmath::gp::{self, Matrix3, Point3, Transform3, Xyz};
use rustycad_tkmath::poly::Triangulation;
use rustycad_tkmath::toploc::Location;
use std::ops::Deref;

/// `BRepGProp_MeshProps::BRepGProp_MeshObjType`: the mesh read as the boundary of a
/// volume (`Vinert`, with the location as apex) or as a surface (`Sinert`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeshObjType {
    /// `Vinert`: the volume between the mesh and the location.
    Vinert = 0,
    /// `Sinert`: the surface of the mesh.
    Sinert,
}

/// The Gauss points of the triangles in barycentric coordinates with their weights
/// (`GPtsWg`): three points of weight 1/6.
const GAUSS_POINTS_AND_WEIGHTS: [f64; 9] = [
    1. / 6.,
    1. / 6.,
    1. / 6.,
    2. / 3.,
    1. / 6.,
    1. / 6.,
    1. / 6.,
    2. / 3.,
    1. / 6.,
];

/// `BRepGProp_MeshProps`: the properties of a triangulation, as a surface or as a
/// volume (a `GProp_GProps`, docs/topalgo-design.md section 9); the inherited members
/// are reached through `Deref`.
///
/// ```
/// use rustycad_tktopalgo::brep_gprop::{BRepGPropMeshProps, MeshObjType};
/// let props = BRepGPropMeshProps::new(MeshObjType::Sinert);
/// assert_eq!(props.get_mesh_obj_type(), MeshObjType::Sinert);
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BRepGPropMeshProps {
    base: GProps,
    kind: MeshObjType,
}

impl Deref for BRepGPropMeshProps {
    type Target = GProps;
    fn deref(&self) -> &GProps {
        &self.base
    }
}

impl BRepGPropMeshProps {
    /// `BRepGProp_MeshProps(theType)`: an empty framework at the origin;
    /// `BRepGPropMeshProps::new(MeshObjType::Vinert)`.
    pub fn new(kind: MeshObjType) -> Self {
        Self {
            base: GProps::default(),
            kind,
        }
    }

    /// `SetLocation(theLocation)`: the point (and volume apex) of the computation;
    /// `props.set_location(p)`.
    pub fn set_location(&mut self, location: Point3) {
        self.base.loc = location;
    }

    /// `GetMeshObjType()`; `props.get_mesh_obj_type()`.
    pub fn get_mesh_obj_type(&self) -> MeshObjType {
        self.kind
    }

    /// `Perform(theMesh, theLoc, theOri)`: the mesh moved by `location`. A location that
    /// mirrors or scales is applied to a copy of the nodes; a rigid one moves the
    /// computation point into the mesh frame, then moves the centre and rotates the
    /// inertia back; nothing for a null or empty mesh; Standard_OutOfRange for a
    /// triangle node outside the mesh; `props.perform_located(&mesh, &location, ori)?`.
    pub fn perform_located(
        &mut self,
        mesh: &Handle<Triangulation>,
        location: &Location,
        orientation: Orientation,
    ) -> Result<(), Failure> {
        if is_empty(mesh) {
            return Ok(());
        }
        if location.is_identity() {
            return self.perform(mesh, orientation);
        }
        let transform = location.transformation();
        let scale = transform.scale_factor();
        let is_to_copy = scale * transform.matrix().determinant() < 0.
            || (scale.abs() - 1.).abs() > gp::resolution();
        if is_to_copy {
            let copy = Handle::new(transformed_copy(mesh, transform)?);
            return self.perform(&copy, orientation);
        }
        self.perform_rigid(mesh, transform, orientation)
    }

    /// The rigid branch of `Perform(theMesh, theLoc, theOri)`: the integration about the
    /// location moved by the inverse transformation, then the inertia tensor at the
    /// centre rotated by the transposed vectorial part and moved back to the location.
    fn perform_rigid(
        &mut self,
        mesh: &Handle<Triangulation>,
        transform: Transform3,
        orientation: Orientation,
    ) -> Result<(), Failure> {
        let inverse = transform.inverted()?;
        let loc_save = self.base.loc;
        self.base.loc.transform(inverse);
        self.perform(mesh, orientation)?;
        let g0 = self.base.g;
        self.base.g = Point3::from(self.base.g.xyz() + self.base.loc.xyz());
        let mut inertia0: Matrix3 = self.base.inertia;
        if g0.xyz().modulus() > gp::resolution() {
            inertia0 = self.base.inertia - h_operator(self.base.g, self.base.loc, self.base.dim);
        }
        let h_vec = inverse.matrix();
        inertia0 = (h_vec.transposed() * inertia0) * h_vec;
        self.base.g.transform(transform);
        let g0 = self.base.g;
        self.base.g = Point3::from(self.base.g.xyz() - loc_save.xyz());
        self.base.loc = loc_save;
        self.base.inertia = inertia0 + h_operator(g0, self.base.loc, self.base.dim);
        Ok(())
    }

    /// `Perform(theMesh, theOri)`: three Gauss points per triangle (the second and third
    /// nodes swapped for a REVERSED mesh), about the location; nothing for a null or
    /// empty mesh; the centre is the static moments themselves when the mass is below
    /// 1e-20; Standard_OutOfRange for a triangle or node index outside the mesh;
    /// `props.perform(&mesh, Orientation::Forward)?`.
    pub fn perform(
        &mut self,
        mesh: &Handle<Triangulation>,
        orientation: Orientation,
    ) -> Result<(), Failure> {
        let Some(triangulation) = mesh.get().filter(|_| !is_empty(mesh)) else {
            return Ok(());
        };
        let is_volume = self.kind == MeshObjType::Vinert;
        let mut sums: GPropsSums = [0.0; 10];
        for i in 1..=triangulation.nb_triangles() {
            let (n1, mut n2, mut n3) = triangulation.triangle(i)?.get();
            if orientation == Orientation::Reversed {
                std::mem::swap(&mut n2, &mut n3);
            }
            let nodes = [
                triangulation.node(n1)?,
                triangulation.node(n2)?,
                triangulation.node(n3)?,
            ];
            let [p1, p2, p3] = nodes;
            Self::calculate_props(
                (p1, p2, p3),
                self.base.loc,
                is_volume,
                &mut sums,
                3,
                &GAUSS_POINTS_AND_WEIGHTS,
            )?;
        }
        self.store(&sums);
        Ok(())
    }

    /// The mass, centre and inertia matrix of the ten sums.
    fn store(&mut self, sums: &GPropsSums) {
        self.base.dim = sums[0];
        let dim = self.base.dim;
        self.base.g = if dim.abs() >= 1.0e-20 {
            Point3::new(sums[1] / dim, sums[2] / dim, sums[3] / dim)
        } else {
            Point3::new(sums[1], sums[2], sums[3])
        };
        self.base.inertia = Matrix3::from_columns(
            Xyz::new(sums[4], -sums[7], -sums[8]),
            Xyz::new(-sums[7], sums[5], -sums[9]),
            Xyz::new(-sums[8], -sums[9], sums[6]),
        );
    }
}

/// A null mesh, or one without nodes or triangles.
fn is_empty(mesh: &Handle<Triangulation>) -> bool {
    mesh.get()
        .is_none_or(|m| m.nb_nodes() == 0 || m.nb_triangles() == 0)
}

/// `new Poly_Triangulation(NbNodes, NbTriangles, false)` with the nodes moved by
/// `transform` and the triangles copied.
fn transformed_copy(
    mesh: &Handle<Triangulation>,
    transform: Transform3,
) -> Result<Triangulation, Failure> {
    let source = mesh.get().ok_or_else(null_mesh)?;
    let copy = Triangulation::with_sizes(source.nb_nodes(), source.nb_triangles(), false, false)?;
    for i in 1..=source.nb_nodes() {
        copy.set_node(i, source.node(i)?.transformed(transform))?;
    }
    for i in 1..=source.nb_triangles() {
        copy.set_triangle(i, source.triangle(i)?)?;
    }
    Ok(copy)
}

fn null_mesh() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "BRepGProp_MeshProps::Perform: null triangulation; expected a mesh",
        "",
    )
}
