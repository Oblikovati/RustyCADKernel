//! Martensite translation of StdPrs_Plane at OCCT 3d097a0.
use super::plane_arrows::draw_plane_arrows;
use super::plane_geometry::{draw_plane_edges, draw_plane_isos};
use crate::prs3d::Prs3dDrawer;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::{adaptor3d::Adaptor3dSurface, geom::GeomPlane};
use rustycad_tkmath::{geom_abs::SurfaceType, gp::Point3};
use rustycad_tkservice::graphic3d::{Graphic3dStructureMembers, Graphic3dStructureSource};

/// Rectangular plane edges, isolines and normals; `StdPrsPlane::add(&prs, &surface, &drawer)?`.
pub struct StdPrsPlane;
impl StdPrsPlane {
    /// Keep the current group even for nonplanes; `Self::add(&prs, &surface, &drawer)?`.
    pub fn add(
        prs: &Handle<dyn Graphic3dStructureSource>,
        surface: &dyn Adaptor3dSurface,
        drawer: &Handle<Prs3dDrawer>,
    ) -> Result<(), Failure> {
        let group = prs.current_group()?;
        if surface.get_type()? != SurfaceType::Plane {
            return Ok(());
        }
        let plane = GeomPlane::from_pln(surface.plane()?);
        let aspect = drawer.plane_aspect();
        draw_plane_edges(&group, &plane, &aspect)?;
        draw_plane_isos(&group, &plane, &aspect)?;
        draw_plane_arrows(prs, &group, &plane, &aspect)
    }
    /// Inclusive distance to the infinite plane, ignoring drawer size; `Self::matches(0., 0., 1., 1., &surface, &drawer)?`.
    pub fn matches(
        x: f64,
        y: f64,
        z: f64,
        distance: f64,
        surface: &dyn Adaptor3dSurface,
        _drawer: &Handle<Prs3dDrawer>,
    ) -> Result<bool, Failure> {
        if surface.get_type()? != SurfaceType::Plane {
            return Ok(false);
        }
        Ok(surface.plane()?.distance(Point3::new(x, y, z)).abs() <= distance)
    }
}
