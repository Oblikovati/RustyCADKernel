//! AIS_Triangulation presentations, the twin of tools/oracle/tkv3d_probe_ais_triangulation.cxx.
use super::common::{Reader, attempt, end, flag, int, real};
use super::sensitive_display::display_driver::SensitiveDisplayDriver;
use rustycad_tkernel::ncollection::HArray1;
use rustycad_tkernel::quantity::{Color, NamedColor};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{Direction3, Point3};
use rustycad_tkmath::poly::{Triangle, Triangulation};
use rustycad_tkv3d::ais::*;
use rustycad_tkv3d::prs_mgr::*;
use rustycad_tkv3d::prs3d::Prs3dDrawer;
use rustycad_tkv3d::select_mgr::{SelectMgrSelectableObject, SelectMgrSelection};
use rustycad_tkv3d::v3d::V3dViewer;

fn mesh(normals: bool) -> Result<Handle<Triangulation>, Failure> {
    let t = Triangulation::with_sizes(4, 2, false, false)?;
    let nodes = [
        Point3::new(0., 0., 0.),
        Point3::new(2., 0., 0.),
        Point3::new(2., 1., 0.5),
        Point3::new(0., 1., 0.),
    ];
    for (i, node) in nodes.iter().enumerate() {
        t.set_node(i as i32 + 1, *node)?;
    }
    t.set_triangle(1, Triangle::new(1, 2, 3))?;
    t.set_triangle(2, Triangle::new(1, 3, 4))?;
    if normals {
        t.add_normals()?;
        for i in 1..=4 {
            t.set_normal(i, Direction3::new(0.1 * f64::from(i), 0.2, 1.)?)?;
        }
    }
    Ok(Handle::new(t))
}

/// `v3_ais_triangulation Normals Colors Link Count Op...`.
pub fn triangulation_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (normals, colors, link, count) = (r.flag(), r.flag(), r.integer(), r.integer());
    let (driver, trace) = SensitiveDisplayDriver::new_handle(0);
    let viewer = V3dViewer::new(&driver).unwrap();
    let manager = PrsMgrPresentationManager::new(&viewer.structure_manager());
    let o = AisTriangulation::new(&mesh(normals).unwrap()).unwrap();
    if link == 0 {
        o.attributes().setup_own_defaults().unwrap();
    }
    if link == 1 {
        let d = Handle::new(Prs3dDrawer::new());
        d.setup_own_defaults().unwrap();
        o.attributes().link_to(&d);
    }
    if colors {
        let c = HArray1::<i32>::with_bounds(1, 4).unwrap();
        for i in 1..=4u32 {
            let value = 0x0010_2030u32.wrapping_mul(i).wrapping_add(0x40) as i32;
            c.change_array1()[i as i32] = value;
        }
        o.set_colors(&Handle::new(c));
    }
    flag(o.has_vertex_colors());
    int(o.kind_of_interactive() as i32);
    int(o.signature());
    for _ in 0..count {
        match r.integer() {
            0 => attempt(|| {
                if manager.has_presentation(&o.clone().cast(), 0) {
                    manager.update(&o.clone().cast(), 0)
                } else {
                    manager
                        .presentation(&o.clone().cast(), 0, true, &Handle::default())
                        .map(|_| ())
                }
            }),
            1 => attempt(|| o.set_transparency(0.4)),
            2 => attempt(|| o.unset_transparency()),
            3 => attempt(|| o.set_color(Color::named(NamedColor::RED))),
            4 => attempt(|| {
                manager
                    .presentation(&o.clone().cast(), 1, true, &Handle::default())
                    .map(|_| ())
            }),
            _ => {
                let s = Handle::new(SelectMgrSelection::new(0));
                o.compute_selection(&s, 0).unwrap();
                int(s.entities().len() as i32);
            }
        }
        flag(o.is_transparent());
        real(o.transparency());
        flag(o.attributes().has_own_shading_aspect());
        int(o.presentations().len() as i32);
    }
    let events = trace.read();
    int(events.events.len() as i32);
    for value in &events.events {
        real(*value);
    }
    end();
}
