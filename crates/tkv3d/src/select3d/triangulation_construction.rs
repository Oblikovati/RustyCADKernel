//! Constructors distinguish shared bounded free-edge handles from copied convenience slices.
use super::sensitive_triangulation::{Select3DSensitiveTriangulation, TriangulationState};
use super::triangulation_geometry::{free_edges, triangle_centers};
use super::{
    Select3DBndBox3d, Select3DSensitiveEntityBase, Select3DSensitiveSetBase,
    Select3DTypeOfSensitivity,
};
use crate::select_mgr::SelectMgrEntityOwnerSource;
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkmath::gp::{GeneralTransform3, Point3};
use rustycad_tkmath::poly::Triangulation;
use rustycad_tkmath::toploc::Location;

impl Select3DSensitiveTriangulation {
    /// The triangulation of an owner at a location: the triangles (interior) or the free edges,
    /// the mean of the triangle centers (or the cached box center without geometry); a
    /// singular location raises `Standard_ConstructionError`.
    /// Example: `Select3DSensitiveTriangulation::new(&owner, &trg, &location, true)?`.
    pub fn new(
        owner: &Handle<dyn SelectMgrEntityOwnerSource>,
        triangulation: &Handle<Triangulation>,
        init_loc: &Location,
        is_interior: bool,
    ) -> Result<Self, Failure> {
        let (state, cog) = automatic_triangulation_state(triangulation, is_interior)?;
        let entity = Self::assemble(owner, triangulation, init_loc, is_interior, cog, state)?;
        entity.compute_bounding_box();
        Ok(entity)
    }
    /// The triangulation with given free edges and center (`GetConnected`); the bounding box is
    /// left to compute. Example: `Select3DSensitiveTriangulation::with_free_edges(&owner, &trg, &loc, &edges, cog, false)?`.
    pub fn with_free_edges(
        owner: &Handle<dyn SelectMgrEntityOwnerSource>,
        triangulation: &Handle<Triangulation>,
        init_loc: &Location,
        free_edges: &[i32],
        cog: Point3,
        is_interior: bool,
    ) -> Result<Self, Failure> {
        let edges = shared_triangulation_edges(free_edges);
        Self::with_free_edge_handle(owner, triangulation, init_loc, &edges, cog, is_interior)
    }
    /// Retains native bounded edge storage so caller edits reach this entity and its copies.
    /// Example: `Select3DSensitiveTriangulation::with_free_edge_handle(&owner, &mesh, &loc, &edges, cog, false)?`.
    pub fn with_free_edge_handle(
        owner: &Handle<dyn SelectMgrEntityOwnerSource>,
        triangulation: &Handle<Triangulation>,
        init_loc: &Location,
        free_edges: &Handle<HArray1<i32>>,
        cog: Point3,
        is_interior: bool,
    ) -> Result<Self, Failure> {
        let state = triangulation_index_state(triangulation, free_edges.clone(), is_interior)?;
        Self::assemble(owner, triangulation, init_loc, is_interior, cog, state)
    }
    /// The entity of its parts.
    fn assemble(
        owner: &Handle<dyn SelectMgrEntityOwnerSource>,
        triangulation: &Handle<Triangulation>,
        init_loc: &Location,
        is_interior: bool,
        cog: Point3,
        state: TriangulationState,
    ) -> Result<Self, Failure> {
        Ok(Self {
            transient: StandardTransient::default(),
            entity: Select3DSensitiveEntityBase::new(owner),
            set: Select3DSensitiveSetBase::default(),
            triangul: triangulation.clone(),
            init_location: init_loc.clone(),
            cdg3d: cog,
            sens_type: sensitivity_kind(is_interior),
            inv_init_location: GeneralTransform3::from(init_loc.transformation().inverted()?),
            state: SharedState::new(state),
        })
    }
}
fn sensitivity_kind(interior: bool) -> Select3DTypeOfSensitivity {
    if interior {
        return Select3DTypeOfSensitivity::Interior;
    }
    Select3DTypeOfSensitivity::Boundary
}
pub(super) fn shared_triangulation_edges(edges: &[i32]) -> Handle<HArray1<i32>> {
    let array = Array1::from_shared(edges, 1);
    Handle::new(HArray1::from_array(&array))
}
fn automatic_triangulation_state(
    triangulation: &Handle<Triangulation>,
    interior: bool,
) -> Result<(TriangulationState, Point3), Failure> {
    let Some(mesh) = triangulation.get().filter(|mesh| mesh.has_geometry()) else {
        return cached_triangulation_state(triangulation);
    };
    let (edges, mut center) = if interior {
        (Handle::default(), triangle_centers(mesh)?)
    } else {
        let (edges, center) = free_edges(triangulation)?;
        (shared_triangulation_edges(&edges), center)
    };
    center /= f64::from(mesh.nb_triangles());
    let state = triangulation_index_state(triangulation, edges, interior)?;
    Ok((state, Point3::from(center)))
}
fn cached_triangulation_state(
    triangulation: &Handle<Triangulation>,
) -> Result<(TriangulationState, Point3), Failure> {
    let center = match triangulation.get().filter(|mesh| mesh.has_cached_min_max()) {
        Some(mesh) => {
            let bounds = mesh.cached_min_max();
            Point3::from((bounds.corner_min()?.xyz() + bounds.corner_max()?.xyz()) * 0.5)
        }
        None => Point3::default(),
    };
    let state = triangulation_index_state(triangulation, Handle::default(), true)?;
    Ok((state, center))
}
/// The BVH indexes `0..n-1` of `n` primitives.
fn triangulation_index_state(
    triangulation: &Handle<Triangulation>,
    free_edges: Handle<HArray1<i32>>,
    interior: bool,
) -> Result<TriangulationState, Failure> {
    let count = triangulation_primitive_count(triangulation, &free_edges, interior)?;
    let has_geometry = triangulation.get().is_some_and(Triangulation::has_geometry);
    Ok(TriangulationState {
        free_edges,
        primitives_nb: count,
        bvh_prim_indexes: has_geometry.then(|| (0..count).collect()),
        bnd_box: Select3DBndBox3d::default(),
    })
}
fn triangulation_primitive_count(
    triangulation: &Handle<Triangulation>,
    edges: &Handle<HArray1<i32>>,
    interior: bool,
) -> Result<i32, Failure> {
    let Some(mesh) = triangulation.get().filter(|mesh| mesh.has_geometry()) else {
        return Ok(0);
    };
    if interior {
        return Ok(mesh.nb_triangles());
    }
    let Some(edges) = edges.get() else {
        return Err(missing_edge_handle());
    };
    let length = edges.array1().len();
    if length % 2 != 0 {
        return Err(unpaired_edge_index(length));
    }
    Ok((length / 2) as i32)
}
fn missing_edge_handle() -> Failure {
    Failure::with_message(
        "Standard_NullObject",
        "null free-edge handle: expected bounded pairs for boundary geometry",
    )
}
fn unpaired_edge_index(length: usize) -> Failure {
    // OCCT 3d097a0 fills one extra BVH index for an unpaired last edge value.
    Failure::with_message(
        "Standard_OutOfRange",
        &format!(
            "BVH index {} from edge length {length}: expected [0, {}]",
            length / 2,
            length as i32 / 2 - 1
        ),
    )
}
