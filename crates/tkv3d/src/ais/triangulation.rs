//! `AIS_Triangulation`: the shaded presentation of a `Poly_Triangulation`, with optional node
//! colors and normals, without selection.
use super::*;
use crate::prs_mgr::*;
use crate::prs3d::Prs3dShadingAspect;
use crate::select_mgr::*;
use rustycad_tkernel::ncollection::{HArray1, Vec3, Vec4};
use rustycad_tkernel::standard::{
    Failure, Handle, HandleReference, SharedState, StandardTransient,
};
use rustycad_tkmath::poly::Triangulation;
use rustycad_tkservice::graphic3d::*;

/// The private fields of `AIS_Triangulation`.
#[derive(Clone)]
struct TriangulationFields {
    triangulation: Handle<Triangulation>,
    color: Handle<HArray1<i32>>,
    flag_color: i32,
    nb_nodes: i32,
    nb_triangles: i32,
}

/// `AIS_Triangulation`: displays a triangulation in mode 0, colored per node when colors are
/// set. Example: `let object = AisTriangulation::new(&triangulation)?;`.
pub struct AisTriangulation {
    transient: StandardTransient,
    base: AisInteractiveObjectBase,
    fields: SharedState<TriangulationFields>,
}
rustycad_tkernel::impl_transient!(AisTriangulation, "AIS_Triangulation", dyn AisInteractiveObject, transient,
    interfaces: [dyn AisInteractiveObject, dyn SelectMgrSelectableObject, dyn PrsMgrPresentableObject]);

impl AisTriangulation {
    /// The object of a triangulation (its node and triangle counts are kept); a null
    /// triangulation is a `Standard_NullObject` failure (a null dereference natively).
    /// Example: `AisTriangulation::new(&triangulation)?`.
    pub fn new(triangulation: &Handle<Triangulation>) -> Result<Handle<Self>, Failure> {
        let value = triangulation.get().ok_or_else(|| {
            Failure::with_message(
                "Standard_NullObject",
                "AIS_Triangulation: null triangulation",
            )
        })?;
        let object = Handle::new(Self {
            transient: StandardTransient::default(),
            base: AisInteractiveObjectBase::new(PrsMgrTypeOfPresentation3d::AllView),
            fields: SharedState::new(TriangulationFields {
                triangulation: triangulation.clone(),
                color: Handle::default(),
                flag_color: 0,
                nb_nodes: value.nb_nodes(),
                nb_triangles: value.nb_triangles(),
            }),
        });
        prs_mgr_presentable_bind(&object.cast());
        Ok(object)
    }
    /// `SetColors`: one packed RGBX color per node; `object.set_colors(&colors)`.
    pub fn set_colors(&self, colors: &Handle<HArray1<i32>>) {
        let mut fields = self.fields.write();
        (fields.flag_color, fields.color) = (1, colors.clone());
    }
    /// `GetColors`; `object.get_colors()`.
    pub fn get_colors(&self) -> Handle<HArray1<i32>> {
        self.fields.read().color.clone()
    }
    /// `HasVertexColors`; `object.has_vertex_colors()`.
    pub fn has_vertex_colors(&self) -> bool {
        self.fields.read().flag_color == 1
    }
    /// `SetTriangulation`: the counts of the first triangulation are kept, as natively.
    /// Example: `object.set_triangulation(&other)`.
    pub fn set_triangulation(&self, triangulation: &Handle<Triangulation>) {
        self.fields.write().triangulation = triangulation.clone();
    }
    /// `GetTriangulation`; `object.get_triangulation()`.
    pub fn get_triangulation(&self) -> Handle<Triangulation> {
        self.fields.read().triangulation.clone()
    }
    /// `updatePresentation` (protected): mode 0 recomputed with node colors, else the shading
    /// aspect applied to its groups. Example: `object.update_presentation()`.
    pub fn update_presentation(&self) {
        if self.has_vertex_colors() {
            self.set_to_update(0);
            return;
        }
        let aspect = self.attributes().shading_aspect().aspect();
        for prs in self.presentations().iter().filter_map(Handle::get) {
            if prs.mode() != 0 {
                continue;
            }
            for group in prs.groups() {
                group.set_group_primitives_aspect(&aspect.clone().cast());
            }
        }
    }
    /// `attenuateColor` (private): the RGB bytes of a packed color scaled by a factor, with the
    /// opacity of the front material when transparent. Example: `object.attenuate_color(c, 1.)`.
    pub fn attenuate_color(&self, color: i32, composition: f64) -> Vec4<u8> {
        let rgbx = color.to_ne_bytes();
        let alpha = if self.is_transparent() {
            let material = self.attributes().shading_aspect().aspect().front_material();
            (255.0 - f64::from(material.transparency()) * 255.0) as u8
        } else {
            255
        };
        Vec4::new(
            (composition * f64::from(rgbx[0])) as u8,
            (composition * f64::from(rgbx[1])) as u8,
            (composition * f64::from(rgbx[2])) as u8,
            alpha,
        )
    }
}

impl AisInteractiveObject for AisTriangulation {
    fn interactive_base(&self) -> &AisInteractiveObjectBase {
        &self.base
    }
}

impl PrsMgrPresentableObject for AisTriangulation {
    fn presentable_base(&self) -> &PrsMgrPresentableObjectBase {
        &self.base.presentable
    }
    /// `Compute`: in mode 0, one triangle array of the nodes (with normals and attenuated
    /// colors when present) in the shading aspect.
    fn compute(
        &self,
        _: &dyn HandleReference<PrsMgrPresentationManager>,
        prs: &dyn HandleReference<dyn Graphic3dStructureSource>,
        mode: i32,
    ) -> Result<(), Failure> {
        if mode != 0 {
            return Ok(());
        }
        let fields = self.fields.read().clone();
        let triangulation = fields.triangulation.get().ok_or_else(|| {
            Failure::with_message(
                "Standard_NullObject",
                "AIS_Triangulation: null triangulation",
            )
        })?;
        let has_normals = triangulation.has_normals();
        let has_colors = fields.flag_color == 1;
        let array = Handle::new(Graphic3dArrayOfTriangles::with_options(
            fields.nb_nodes,
            fields.nb_triangles * 3,
            has_normals,
            has_colors,
            false,
        )?);
        let group = prs.observe().current_group()?;
        let aspect = self.attributes().shading_aspect().aspect();
        let colors: Vec<i32> = match fields.color.get() {
            Some(colors) if has_colors => colors.array1().iter().copied().collect(),
            _ => Vec::new(),
        };
        let lower = fields.color.get().map_or(1, |c| c.array1().lower());
        for node in 1..=triangulation.nb_nodes() {
            let point = triangulation.node(node)?;
            if has_colors {
                let index = (node - lower) as usize;
                let color = *colors.get(index).ok_or_else(|| {
                    Failure::with_message("Standard_OutOfRange", "AIS_Triangulation: node color")
                })?;
                array.add_vertex_pnt_color4(&point, self.attenuate_color(color, 1.0))?;
            } else {
                array.add_vertex_pnt(&point)?;
            }
            if has_normals {
                let normal: Vec3<f32> = triangulation.normal_vec3(node)?;
                array.set_vertex_normal(
                    node,
                    f64::from(normal.x()),
                    f64::from(normal.y()),
                    f64::from(normal.z()),
                )?;
            }
        }
        for index in 1..=triangulation.nb_triangles() {
            let triangle = triangulation.triangle(index)?;
            for corner in 1..=3 {
                array.add_edge(triangle.value(corner)?)?;
            }
        }
        group.set_primitives_aspect(&aspect.cast());
        group.add_primitive_array(&array.cast(), true);
        Ok(())
    }
    /// `SetTransparency`: an own shading aspect (a copy of the linked one) with the
    /// transparency of the current facing model, then the presentation update.
    fn set_transparency(&self, value: f64) -> Result<(), Failure> {
        let drawer = self.attributes();
        if !drawer.has_own_shading_aspect() {
            drawer.set_shading_aspect(Handle::new(Prs3dShadingAspect::new()?));
            if drawer.has_link() {
                let linked = drawer.link().shading_aspect().aspect();
                let copy = linked.aspects_base().fields.read().clone();
                *drawer
                    .shading_aspect()
                    .aspect()
                    .aspects_base()
                    .fields
                    .write() = copy;
            }
        }
        let facing = self.current_facing_model();
        drawer.shading_aspect().set_transparency(value, facing)?;
        drawer.set_transparency(value as f32);
        self.update_presentation();
        Ok(())
    }
    /// `UnsetTransparency`: no own transparency; an own shading aspect of an object with a
    /// color or material is made opaque, then the presentation update.
    fn unset_transparency(&self) -> Result<(), Failure> {
        let drawer = self.attributes();
        drawer.set_transparency(0.0);
        if !drawer.has_own_shading_aspect() {
            return Ok(());
        }
        if self.has_color() || self.has_material() {
            drawer
                .shading_aspect()
                .set_transparency(0.0, self.current_facing_model())?;
        }
        self.update_presentation();
        Ok(())
    }
    crate::ais_presentable_overrides!();
}

impl SelectMgrSelectableObject for AisTriangulation {
    fn selectable_base(&self) -> &SelectMgrSelectableObjectBase {
        &self.base.selectable
    }
    /// `ComputeSelection`: nothing (the triangulation is not selectable).
    fn compute_selection(
        &self,
        _: &dyn HandleReference<SelectMgrSelection>,
        _: i32,
    ) -> Result<(), Failure> {
        Ok(())
    }
}
