// Rust translation of ModelingAlgorithms/TKHLR/HLRAlgo/HLRAlgo_PolyShellData.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! HLRAlgo_PolyShellData: the faces and the edge segments of one shell.
use super::bi_point::BiPoint;
use super::failures::{array_failure, null_handle};
use super::poly_data::PolyData;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::{Array1, List};
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkmath::bnd::BoundingBox3;
use std::sync::RwLockWriteGuard;

/// HLRAlgo_PolyShellData::ShellIndices: the packed box of the shell. Natively the
/// fields are uninitialized until `HLRAlgo_PolyAlgo::Update`; the Rust default is zero.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[allow(missing_docs)] // The native field names, documented on the type.
pub struct ShellIndices {
    pub min: i32,
    pub max: i32,
}

/// HLRAlgo_PolyShellData: a shell of the polyhedral algorithm: its faces (1-based,
/// null until set), the subset of them that hide, and the segments of its edges. A
/// transient shared by handles; each field is behind its own lock and the native
/// reference accessors return write guards (docs/hlr-design.md section 5).
///
/// ```
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkhlr::hlr_algo::{PolyData, PolyShellData};
/// let shell = PolyShellData::new(1).unwrap();
/// shell.poly_data().set_value(1, Handle::new(PolyData::new())).unwrap();
/// shell.update_hiding(1).unwrap();
/// assert!(shell.hiding());
/// assert_eq!(shell.edges().len(), 0);
/// ```
pub struct PolyShellData {
    base: StandardTransient,
    indices: SharedState<ShellIndices>,
    polyg: SharedState<Array1<'static, Handle<PolyData>>>,
    h_pol_hi: SharedState<Array1<'static, Handle<PolyData>>>,
    seg_list: SharedState<List<BiPoint>>,
}
impl_transient!(
    PolyShellData,
    "HLRAlgo_PolyShellData",
    StandardTransient,
    base
);

impl PolyShellData {
    /// HLRAlgo_PolyShellData(nbFace): `nb_face` null faces, no hiding face, no segment;
    /// Standard_RangeError for a negative count; `PolyShellData::new(4)?`.
    pub fn new(nb_face: i32) -> Result<Self, Failure> {
        let polyg = Array1::with_bounds(1, nb_face).map_err(array_failure(
            "HLRAlgo_PolyShellData::HLRAlgo_PolyShellData",
        ))?;
        Ok(Self {
            base: StandardTransient::default(),
            indices: SharedState::default(),
            polyg: SharedState::new(polyg),
            h_pol_hi: SharedState::new(Array1::default()),
            seg_list: SharedState::new(List::default()),
        })
    }

    /// UpdateGlobalMinMax: adds the projected ends of every segment, then the hiding
    /// triangles of every face, to `the_box`; a null face crashes natively and gives
    /// Standard_NullObject here; `shell.update_global_min_max(&mut bounds)?`.
    pub fn update_global_min_max(&self, the_box: &mut BoundingBox3) -> Result<(), Failure> {
        for segment in self.seg_list.read().iter() {
            let points = segment.points();
            for p in [points.pnt_p1, points.pnt_p2] {
                the_box.update_point(p.x(), p.y(), p.z());
            }
        }
        let faces: Vec<Handle<PolyData>> = self.polyg.read().iter().cloned().collect();
        for face in faces {
            let face = face.get().ok_or_else(|| {
                null_handle("HLRAlgo_PolyShellData::UpdateGlobalMinMax", "myPolyg(i)")
            })?;
            face.update_global_min_max(the_box)?;
        }
        Ok(())
    }

    /// UpdateHiding: sizes the hiding faces to `1..nb_hiding` null faces, or empties
    /// them when `nb_hiding <= 0`; `shell.update_hiding(2)?`.
    pub fn update_hiding(&self, nb_hiding: i32) -> Result<(), Failure> {
        let mut hiding = self.h_pol_hi.write();
        if nb_hiding <= 0 {
            *hiding = Array1::default();
            return Ok(());
        }
        let failure = array_failure("HLRAlgo_PolyShellData::UpdateHiding");
        hiding.resize_bounds(1, nb_hiding, false).map_err(failure)
    }

    /// Hiding: the shell has hiding faces; `shell.hiding()`.
    pub fn hiding(&self) -> bool {
        !self.h_pol_hi.read().is_empty()
    }

    /// PolyData(): the faces, writable through the guard; `shell.poly_data().upper()`.
    pub fn poly_data(&self) -> RwLockWriteGuard<'_, Array1<'static, Handle<PolyData>>> {
        self.polyg.write()
    }

    /// HidingPolyData(): the hiding faces, writable through the guard;
    /// `shell.hiding_poly_data().upper()`.
    pub fn hiding_poly_data(&self) -> RwLockWriteGuard<'_, Array1<'static, Handle<PolyData>>> {
        self.h_pol_hi.write()
    }

    /// Edges(): the segments of the shell's edges, writable through the guard;
    /// `shell.edges().append(segment)`.
    pub fn edges(&self) -> RwLockWriteGuard<'_, List<BiPoint>> {
        self.seg_list.write()
    }

    /// Indices(): the packed box of the shell, writable through the guard;
    /// `shell.indices().min = 0`.
    pub fn indices(&self) -> RwLockWriteGuard<'_, ShellIndices> {
        self.indices.write()
    }
}
