// Rust translation of ModelingData/TKBRep/BRep/BRep_TFace.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRep_TFace`: the face TShape with a surface, a tolerance and triangulations.
use super::dump::{dump_base, dump_class_name, dump_flag, dump_handle, dump_location};
use super::errors::program_error;
use crate::topods::{self, TShape, TShapeBase, dump_tshape};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::dump::{self, DumpStream};
use rustycad_tkernel::standard::real::real_epsilon;
use rustycad_tkernel::standard::{Failure, Handle, SharedState};
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkmath::poly::Triangulation;
use rustycad_tkmath::poly::mesh_purpose::{self, MeshPurpose};
use rustycad_tkmath::toploc::Location;
use std::io;

/// The private fields of `BRep_TFace`.
#[derive(Clone)]
struct TFaceFields {
    triangulations: Vec<Handle<Triangulation>>,
    active: Handle<Triangulation>,
    surface: Handle<dyn GeomSurface>,
    location: Location,
    tolerance: f64,
    natural_restriction: bool,
}

/// `BRep_TFace`: a face with its located surface, its tolerance (`RealEpsilon()` when
/// new), its natural restriction flag and its triangulations, one of them active.
///
/// ```
/// use rustycad_tkbrep::brep::BRepTFace;
/// let face = BRepTFace::new();
/// assert!(face.surface().is_null() && !face.natural_restriction());
/// assert_eq!(face.nb_triangulations(), 0);
/// ```
pub struct BRepTFace {
    parent: topods::TFace,
    fields: SharedState<TFaceFields>,
}
impl_transient!(BRepTFace, "BRep_TFace", topods::TFace, parent, interfaces: [dyn TShape]);

/// Sets or clears the `Poly_MeshPurpose_Active` bit of a non-null triangulation.
fn mark_active(triangulation: &Handle<Triangulation>, active: bool) {
    let Some(triangulation) = triangulation.get() else {
        return;
    };
    let purpose = triangulation.mesh_purpose();
    triangulation.set_mesh_purpose(if active {
        purpose | mesh_purpose::ACTIVE
    } else {
        purpose & !mesh_purpose::ACTIVE
    });
}

impl BRepTFace {
    /// `BRep_TFace()`: no surface, the tolerance `RealEpsilon()`; `BRepTFace::new()`.
    pub fn new() -> Self {
        Self {
            parent: topods::TFace::new(),
            fields: SharedState::new(TFaceFields {
                triangulations: Vec::new(),
                active: Handle::default(),
                surface: Handle::default(),
                location: Location::default(),
                tolerance: real_epsilon(),
                natural_restriction: false,
            }),
        }
    }
    /// `Surface()`; `face.surface()`.
    pub fn surface(&self) -> Handle<dyn GeomSurface> {
        self.fields.read().surface.clone()
    }
    /// `Surface(S)`; `face.set_surface(&surface)`.
    pub fn set_surface(&self, surface: &Handle<dyn GeomSurface>) {
        self.fields.write().surface = surface.clone();
    }
    /// `Location()`; `face.location()`.
    pub fn location(&self) -> Location {
        self.fields.read().location.clone()
    }
    /// `Location(L)`; `face.set_location(&location)`.
    pub fn set_location(&self, location: &Location) {
        self.fields.write().location = location.clone();
    }
    /// `Tolerance()`; `face.tolerance()`.
    pub fn tolerance(&self) -> f64 {
        self.fields.read().tolerance
    }
    /// `Tolerance(T)`; `face.set_tolerance(1e-7)`.
    pub fn set_tolerance(&self, tolerance: f64) {
        self.fields.write().tolerance = tolerance;
    }
    /// `NaturalRestriction()`; `face.natural_restriction()`.
    pub fn natural_restriction(&self) -> bool {
        self.fields.read().natural_restriction
    }
    /// `NaturalRestriction(R)`; `face.set_natural_restriction(true)`.
    pub fn set_natural_restriction(&self, restriction: bool) {
        self.fields.write().natural_restriction = restriction;
    }
    /// `Triangulations()`: a copy of the triangulation list; `face.triangulations()`.
    pub fn triangulations(&self) -> Vec<Handle<Triangulation>> {
        self.fields.read().triangulations.clone()
    }
    /// `NbTriangulations()`; `face.nb_triangulations()`.
    pub fn nb_triangulations(&self) -> i32 {
        self.fields.read().triangulations.len() as i32
    }
    /// `ActiveTriangulation()`; `face.active_triangulation()`.
    pub fn active_triangulation(&self) -> Handle<Triangulation> {
        self.fields.read().active.clone()
    }

    /// `Triangulation(thePurpose)`: the active triangulation for `NONE`, else the first
    /// one whose purpose shares a bit with `purpose`, else the first one when `purpose`
    /// has `AnyFallback`, else a null handle; `face.triangulation(mesh_purpose::NONE)`.
    pub fn triangulation(&self, purpose: MeshPurpose) -> Handle<Triangulation> {
        let fields = self.fields.read().clone();
        if purpose == mesh_purpose::NONE {
            return fields.active;
        }
        let matching = fields.triangulations.iter().find(|triangulation| {
            triangulation
                .get()
                .is_some_and(|t| t.mesh_purpose() & purpose != 0)
        });
        if let Some(found) = matching {
            return found.clone();
        }
        if purpose & mesh_purpose::ANY_FALLBACK != 0 && !fields.triangulations.is_empty() {
            return fields.triangulations[0].clone();
        }
        Handle::default()
    }

    /// `Triangulation(theTriangulation, theToReset)`: resets the list to the input (or
    /// clears it for a null input), activates an input already in the list, or
    /// replaces the active triangulation in the list; the Active purpose bits follow;
    /// `face.set_triangulation(&triangulation, true)`.
    pub fn set_triangulation(&self, triangulation: &Handle<Triangulation>, to_reset: bool) {
        if to_reset || triangulation.is_null() {
            self.reset_triangulation(triangulation);
            return;
        }
        let fields = self.fields.read().clone();
        if fields.triangulations.contains(triangulation) {
            mark_active(&fields.active, false);
            self.fields.write().active = triangulation.clone();
            mark_active(triangulation, true);
            return;
        }
        let Some(position) = fields
            .triangulations
            .iter()
            .position(|t| *t == fields.active)
        else {
            return;
        };
        mark_active(&fields.active, false);
        let mut fields = self.fields.write();
        fields.triangulations[position] = triangulation.clone();
        fields.active = triangulation.clone();
        drop(fields);
        mark_active(triangulation, true);
    }

    /// The reset branch of `Triangulation(T, theToReset)`.
    fn reset_triangulation(&self, triangulation: &Handle<Triangulation>) {
        let active = self.fields.read().active.clone();
        mark_active(&active, false);
        let mut fields = self.fields.write();
        fields.active = Handle::default();
        fields.triangulations.clear();
        if triangulation.is_some() {
            fields.triangulations.push(triangulation.clone());
            fields.active = triangulation.clone();
        }
        drop(fields);
        mark_active(triangulation, true);
    }

    /// `Triangulations(theTriangulations, theActiveTriangulation)`: stores the list and
    /// its active member (the first one for a null input). A null member or an active
    /// triangulation outside the list raises Standard_ProgramError (a release-build
    /// `Standard_ASSERT_RAISE`), after the Active bits of the members before it were
    /// cleared; `face.set_triangulations(&list, &Handle::default())?`.
    pub fn set_triangulations(
        &self,
        triangulations: &[Handle<Triangulation>],
        active: &Handle<Triangulation>,
    ) -> Result<(), Failure> {
        if triangulations.is_empty() {
            let mut fields = self.fields.write();
            fields.active = Handle::default();
            fields.triangulations.clear();
            return Ok(());
        }
        let mut active_in_list = false;
        for (index, triangulation) in triangulations.iter().enumerate() {
            if triangulation.is_null() {
                return Err(program_error(
                    "Try to set list with NULL triangulation to the face",
                    &format!("null triangulation at list index {index}"),
                ));
            }
            active_in_list |= triangulation == active;
            mark_active(triangulation, false);
        }
        if active.is_some() && !active_in_list {
            return Err(program_error(
                "Active triangulation isn't part of triangulations list",
                "the active triangulation is not a list member",
            ));
        }
        self.store_triangulations(triangulations, active);
        Ok(())
    }

    /// Stores a checked list and marks its active member.
    fn store_triangulations(
        &self,
        triangulations: &[Handle<Triangulation>],
        active: &Handle<Triangulation>,
    ) {
        let active = if active.is_null() {
            triangulations[0].clone()
        } else {
            active.clone()
        };
        let mut fields = self.fields.write();
        fields.triangulations = triangulations.to_vec();
        fields.active = active.clone();
        drop(fields);
        mark_active(&active, true);
    }
}

impl Default for BRepTFace {
    fn default() -> Self {
        Self::new()
    }
}

impl TShape for BRepTFace {
    fn tshape_base(&self) -> &TShapeBase {
        TShape::tshape_base(&self.parent)
    }
    /// A new face with the same surface, location and tolerance, without triangulation.
    fn empty_copy(&self) -> Result<Handle<dyn TShape>, Failure> {
        let copy = Self::new();
        let fields = self.fields.read().clone();
        copy.set_surface(&fields.surface);
        copy.set_location(&fields.location);
        copy.set_tolerance(fields.tolerance);
        Ok(Handle::new(copy).into_cast())
    }
    /// The class name, the `TopoDS_TShape` dump, the active triangulation, the surface,
    /// the location, the tolerance, the flag and each triangulation.
    fn dump_json(&self, output: &mut DumpStream, depth: i32) -> io::Result<()> {
        let address = std::ptr::from_ref(self).cast::<()>() as usize;
        dump_class_name(output, "BRep_TFace")?;
        dump_base(output, depth, |nested, _| {
            dump_tshape(self.tshape_base(), address, nested)
        })?;
        let fields = self.fields.read().clone();
        dump_handle(
            output,
            depth,
            "ActiveTriangulation",
            &fields.active,
            |t, s, _| t.dump_json(s),
        )?;
        dump_handle(output, depth, "Surface", &fields.surface, |c, s, d| {
            c.dump_json(s, d)
        })?;
        dump_location(output, depth, "Location", &fields.location)?;
        dump::dump_real(output, "Tolerance", fields.tolerance)?;
        dump_flag(output, "NaturalRestriction", fields.natural_restriction)?;
        for triangulation in &fields.triangulations {
            dump_handle(output, depth, "Triangulation", triangulation, |t, s, _| {
                t.dump_json(s)
            })?;
        }
        Ok(())
    }
}
