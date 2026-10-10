//! `AIS_PlaneTrihedron` at OCCT 3d097a0: live plane and constructor-time selection children.
use super::{
    AisInteractiveObject, AisInteractiveObjectBase, AisKindOfInteractive, AisLine, AisPoint,
};
use crate::{prs_mgr::*, prs3d::*, select_mgr::*};
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::{Failure, Handle, SharedState, StandardTransient},
    tcollection::AsciiString,
    units_api::UnitsApi,
};
use rustycad_tkg3d::geom::{GeomCartesianPoint, GeomLine, GeomPlane};
use rustycad_tkmath::gp::Axis1;
use std::sync::Arc;
/// A selectable XY frame on a plane; `AisPlaneTrihedron::new(&plane, units)?`.
pub struct AisPlaneTrihedron {
    transient: StandardTransient,
    pub(super) base: AisInteractiveObjectBase,
    component: SharedState<Handle<GeomPlane>>,
    pub(super) children: SharedState<[Handle<dyn SelectMgrSelectableObject>; 3]>,
    pub(super) labels: SharedState<[AsciiString; 2]>,
    units: Arc<SharedState<UnitsApi>>,
}
rustycad_tkernel::impl_transient!(AisPlaneTrihedron,"AIS_PlaneTrihedron",dyn AisInteractiveObject,transient,
    interfaces:[dyn AisInteractiveObject,dyn SelectMgrSelectableObject,dyn PrsMgrPresentableObject]);
impl AisPlaneTrihedron {
    /// Retain a live plane and snapshot child owner objects; `AisPlaneTrihedron::new(&plane, units)?`.
    pub fn new(
        plane: &Handle<GeomPlane>,
        units: Arc<SharedState<UnitsApi>>,
    ) -> Result<Handle<Self>, Failure> {
        let object = Handle::new(Self {
            transient: StandardTransient::default(),
            base: AisInteractiveObjectBase::new(PrsMgrTypeOfPresentation3d::AllView),
            component: SharedState::new(plane.clone()),
            children: SharedState::new(std::array::from_fn(|_| Handle::default())),
            labels: SharedState::new([AsciiString::from("X"), AsciiString::from("Y")]),
            units,
        });
        object.initialize_plane_datum()?;
        *object.children.write() = [
            object.position()?.cast(),
            object.x_axis()?.cast(),
            object.y_axis()?.cast(),
        ];
        prs_mgr_presentable_bind(&object.cast());
        Ok(object)
    }
    fn initialize_plane_datum(&self) -> Result<(), Failure> {
        let datum = Handle::new(Prs3dDatumAspect::new()?);
        let length = self.units.write().any_to_ls(100., b"mm")?;
        datum.set_axis_length(length, length, length);
        for part in [Prs3dDatumParts::XAxis, Prs3dDatumParts::YAxis] {
            datum
                .line_aspect(part)
                .set_color(Color::named(NamedColor::ROYALBLUE1));
        }
        datum.set_draw_datum_axes(Prs3dDatumAxes::XYAxes);
        self.attributes().set_datum_aspect(datum);
        Ok(())
    }
    /// Return the retained plane; `trihedron.component()`.
    pub fn component(&self) -> Handle<GeomPlane> {
        self.component.read().clone()
    }
    /// Replace geometry without replacing selection children or scheduling updates; `trihedron.set_component(&plane)`.
    pub fn set_component(&self, plane: &Handle<GeomPlane>) {
        *self.component.write() = plane.clone();
    }
    /// Create a fresh blue line on the current X axis; `trihedron.x_axis()?`.
    pub fn x_axis(&self) -> Result<Handle<AisLine>, Failure> {
        self.plane_axis_line(self.component().pln().x_axis())
    }
    /// Create a fresh blue line on the current Y axis; `trihedron.y_axis()?`.
    pub fn y_axis(&self) -> Result<Handle<AisLine>, Failure> {
        self.plane_axis_line(self.component().pln().y_axis())
    }
    fn plane_axis_line(&self, axis: Axis1) -> Result<Handle<AisLine>, Failure> {
        let line = AisLine::new(&Handle::new(GeomLine::new(axis)), self.units.clone());
        line.set_color(Color::named(NamedColor::ROYALBLUE1))?;
        Ok(line)
    }
    /// Create a fresh datum at the live plane origin; `trihedron.position()?`.
    pub fn position(&self) -> Result<Handle<AisPoint>, Failure> {
        AisPoint::new(
            &Handle::new(GeomCartesianPoint::from_pnt(
                self.component().pln().location(),
            ))
            .cast(),
        )
    }
    /// Assign all three stored lengths and flag presentations for update; `trihedron.set_length(10.)`.
    pub fn set_length(&self, length: f64) {
        self.attributes()
            .datum_aspect()
            .set_axis_length(length, length, length);
        self.set_to_update_all();
    }
    /// Return the stored X length; `trihedron.get_length()`.
    pub fn get_length(&self) -> f64 {
        self.attributes()
            .datum_aspect()
            .axis_length(Prs3dDatumParts::XAxis)
    }
    /// Copy the X label without scheduling an update; `trihedron.set_x_label(&"U".into())`.
    pub fn set_x_label(&self, label: &AsciiString) {
        self.labels.write()[0] = label.clone();
    }
    /// Copy the Y label without scheduling an update; `trihedron.set_y_label(&"V".into())`.
    pub fn set_y_label(&self, label: &AsciiString) {
        self.labels.write()[1] = label.clone();
    }
}
impl AisInteractiveObject for AisPlaneTrihedron {
    fn interactive_base(&self) -> &AisInteractiveObjectBase {
        &self.base
    }
    fn kind_of_interactive(&self) -> AisKindOfInteractive {
        AisKindOfInteractive::Datum
    }
    fn signature(&self) -> i32 {
        4
    }
}
