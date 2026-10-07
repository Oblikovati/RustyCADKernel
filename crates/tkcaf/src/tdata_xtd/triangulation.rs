// Rust port of ApplicationFramework/TKCAF/TDataXtd/TDataXtd_Triangulation.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TDataXtd_Triangulation`: a `Poly_Triangulation` kept on a label, changed only through
//! the attribute so that every change is backed up.
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, Guid, Handle, SharedState, StandardTransient, exceptions},
};
use rustycad_tklcaf::guid_from_text;
use rustycad_tklcaf::tdata_std::support::{cast_attribute, find_or_add};
use rustycad_tklcaf::tdf::{
    TdfAttribute, TdfAttributeBase, TdfAttributeState, TdfLabel, TdfRelocationTable,
};
use rustycad_tkmath::gp::{Direction3, Point2, Point3};
use rustycad_tkmath::poly::{Triangle, Triangulation};
use std::io;

/// `TDataXtd_Triangulation::GetID()`.
const TRIANGULATION_ID: Guid = guid_from_text("27AE2C44-60B0-41AE-AC18-BA3FDA538D03");

/// `TDataXtd_Triangulation`.
///
/// ```
/// use rustycad_tkcaf::tdata_xtd::TDataXtdTriangulation;
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkmath::poly::Triangulation;
/// use rustycad_tklcaf::tdf::TdfData;
/// let data = TdfData::new();
/// let mesh = Handle::new(Triangulation::with_sizes(3, 1, false, false).unwrap());
/// let attribute = TDataXtdTriangulation::set_triangulation_of(&data.root(), &mesh).unwrap();
/// assert_eq!((attribute.nb_nodes().unwrap(), attribute.nb_triangles().unwrap()), (3, 1));
/// ```
pub struct TDataXtdTriangulation {
    transient: StandardTransient,
    base: TdfAttributeBase,
    triangulation: SharedState<Handle<Triangulation>>,
}
impl_transient!(TDataXtdTriangulation, "TDataXtd_Triangulation", dyn TdfAttribute, transient, interfaces: [dyn TdfAttribute]);

impl Default for TDataXtdTriangulation {
    fn default() -> Self {
        Self::new()
    }
}

/// `theMesh->Copy()`, null for a null mesh (`Restore` and `Paste`).
fn copied(mesh: &Handle<Triangulation>) -> Handle<Triangulation> {
    mesh.get().map(Triangulation::copy).unwrap_or_default()
}

impl TDataXtdTriangulation {
    /// `GetID`.
    pub fn get_id() -> Guid {
        TRIANGULATION_ID
    }
    /// `Set(theLabel)`: finds or adds the attribute.
    pub fn set(label: &TdfLabel) -> Result<Handle<TDataXtdTriangulation>, Failure> {
        find_or_add(label, &Self::get_id(), false, Self::new)
    }
    /// `Set(theLabel, theTriangulation)`: finds or adds the attribute and sets the mesh.
    pub fn set_triangulation_of(
        label: &TdfLabel,
        mesh: &Handle<Triangulation>,
    ) -> Result<Handle<TDataXtdTriangulation>, Failure> {
        let attribute = Self::set(label)?;
        rustycad_tklcaf::live(&attribute, "TDataXtd_Triangulation::Set")?
            .set_triangulation(mesh)?;
        Ok(attribute)
    }
    /// `TDataXtd_Triangulation()`: without a mesh.
    pub fn new() -> Self {
        Self {
            transient: StandardTransient::default(),
            base: TdfAttributeBase::default(),
            triangulation: SharedState::new(Handle::default()),
        }
    }
    /// `Set(theTriangulation)`: always backed up.
    pub fn set_triangulation(&self, mesh: &Handle<Triangulation>) -> Result<(), Failure> {
        self.backup()?;
        *self.triangulation.write() = mesh.clone();
        Ok(())
    }
    /// `Get`.
    pub fn get(&self) -> Handle<Triangulation> {
        self.triangulation.read().clone()
    }
    /// The mesh for the delegating methods (`Standard_NullObject` without one; natively a
    /// null dereference).
    fn mesh(&self) -> Result<Handle<Triangulation>, Failure> {
        let mesh = self.get();
        if mesh.is_null() {
            return Err(exceptions::null_object(
                "TDataXtd_Triangulation: no Poly_Triangulation; expected Set first",
            ));
        }
        Ok(mesh)
    }
    /// Backs up, then changes the mesh.
    fn change<R>(
        &self,
        apply: impl FnOnce(&Triangulation) -> Result<R, Failure>,
    ) -> Result<R, Failure> {
        let mesh = self.mesh()?;
        self.backup()?;
        apply(rustycad_tklcaf::live(&mesh, "TDataXtd_Triangulation")?)
    }
    /// Reads the mesh.
    fn query<R>(
        &self,
        read: impl FnOnce(&Triangulation) -> Result<R, Failure>,
    ) -> Result<R, Failure> {
        read(rustycad_tklcaf::live(
            &self.mesh()?,
            "TDataXtd_Triangulation",
        )?)
    }
}

/// The `Poly_Triangulation` methods, covered so that changes are backed up.
impl TDataXtdTriangulation {
    /// `Deflection()`.
    pub fn deflection(&self) -> Result<f64, Failure> {
        self.query(|mesh| Ok(mesh.deflection()))
    }
    /// `Deflection(theDeflection)`.
    pub fn set_deflection(&self, deflection: f64) -> Result<(), Failure> {
        self.change(|mesh| {
            mesh.set_deflection(deflection);
            Ok(())
        })
    }
    /// `RemoveUVNodes`.
    pub fn remove_uv_nodes(&self) -> Result<(), Failure> {
        self.change(|mesh| {
            mesh.remove_uv_nodes();
            Ok(())
        })
    }
    /// `NbNodes`.
    pub fn nb_nodes(&self) -> Result<i32, Failure> {
        self.query(|mesh| Ok(mesh.nb_nodes()))
    }
    /// `NbTriangles`.
    pub fn nb_triangles(&self) -> Result<i32, Failure> {
        self.query(|mesh| Ok(mesh.nb_triangles()))
    }
    /// `HasUVNodes`.
    pub fn has_uv_nodes(&self) -> Result<bool, Failure> {
        self.query(|mesh| Ok(mesh.has_uv_nodes()))
    }
    /// `Node(theIndex)`.
    pub fn node(&self, index: i32) -> Result<Point3, Failure> {
        self.query(|mesh| mesh.node(index))
    }
    /// `SetNode(theIndex, theNode)`.
    pub fn set_node(&self, index: i32, node: Point3) -> Result<(), Failure> {
        self.change(|mesh| mesh.set_node(index, node))
    }
    /// `UVNode(theIndex)`.
    pub fn uv_node(&self, index: i32) -> Result<Point2, Failure> {
        self.query(|mesh| mesh.uv_node(index))
    }
    /// `SetUVNode(theIndex, theUVNode)`.
    pub fn set_uv_node(&self, index: i32, node: Point2) -> Result<(), Failure> {
        self.change(|mesh| mesh.set_uv_node(index, node))
    }
    /// `Triangle(theIndex)`.
    pub fn triangle(&self, index: i32) -> Result<Triangle, Failure> {
        self.query(|mesh| mesh.triangle(index))
    }
    /// `SetTriangle(theIndex, theTriangle)`.
    pub fn set_triangle(&self, index: i32, triangle: Triangle) -> Result<(), Failure> {
        self.change(|mesh| mesh.set_triangle(index, triangle))
    }
    /// `SetNormal(theIndex, theNormal)`.
    pub fn set_normal(&self, index: i32, normal: Direction3) -> Result<(), Failure> {
        self.change(|mesh| mesh.set_normal(index, normal))
    }
    /// `HasNormals`.
    pub fn has_normals(&self) -> Result<bool, Failure> {
        self.query(|mesh| Ok(mesh.has_normals()))
    }
    /// `Normal(theIndex)`.
    pub fn normal(&self, index: i32) -> Result<Direction3, Failure> {
        self.query(|mesh| mesh.normal(index))
    }
}

impl TdfAttribute for TDataXtdTriangulation {
    fn attribute_base(&self) -> &TdfAttributeBase {
        &self.base
    }
    fn id(&self) -> Guid {
        TRIANGULATION_ID
    }
    /// `Restore`: a copy of the backed-up mesh.
    fn restore(&self, attribute: &Handle<dyn TdfAttribute>) -> Result<(), Failure> {
        let other =
            cast_attribute::<TDataXtdTriangulation>(attribute, "TDataXtd_Triangulation::Restore")?;
        let mesh = copied(&rustycad_tklcaf::live(&other, "TDataXtd_Triangulation::Restore")?.get());
        *self.triangulation.write() = mesh;
        Ok(())
    }
    fn new_empty(&self) -> Handle<dyn TdfAttribute> {
        Handle::new(Self::new()).cast()
    }
    /// `Paste`: a copy of the mesh (no backup, as natively).
    fn paste(
        &self,
        into: &Handle<dyn TdfAttribute>,
        _table: &Handle<TdfRelocationTable>,
    ) -> Result<(), Failure> {
        let target =
            cast_attribute::<TDataXtdTriangulation>(into, "TDataXtd_Triangulation::Paste")?;
        let target = rustycad_tklcaf::live(&target, "TDataXtd_Triangulation::Paste")?;
        *target.triangulation.write() = copied(&self.get());
        Ok(())
    }
    /// `Dump`: `Triangulation` only (the native dump is a TODO).
    fn dump(&self, stream: &mut dyn io::Write) -> io::Result<()> {
        write!(stream, "Triangulation")
    }
}
