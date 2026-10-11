//! Native rubber-band constructor, appearance, point edits and protected cache oracle.
use super::{
    array::array_state,
    common::{Reader, attempt, end, flag, int, real},
    sensitive_display::display_driver,
};
use rustycad_tkernel::{
    ncollection::Vec2,
    quantity::{Color, NamedColor},
    standard::{Failure, Handle},
};
use rustycad_tkservice::{aspect::*, graphic3d::*};
use rustycad_tkv3d::{ais::*, prs_mgr::*, v3d::V3dViewer};
fn snapshot(band: &AisRubberBand) {
    int(band.signature());
    int(band.kind_of_interactive() as i32);
    int(band.z_layer());
    flag(band.has_color());
    flag(band.is_filling());
    flag(band.is_polygon_closed());
    real(band.line_width());
    int(band.line_type() as i32);
    real(band.fill_transparency());
    for color in [band.line_color(), band.fill_color()] {
        for value in [color.red(), color.green(), color.blue()] {
            real(value);
        }
    }
    let points = band.points().read();
    int(points.length());
    for point in points.iter() {
        int(point.x);
        int(point.y);
    }
}
fn caches(band: &AisRubberBand) {
    let fields = &band.rubber_band_base().fields;
    for array in [
        fields.borders.read().cast::<Graphic3dArrayOfPrimitives>(),
        fields.triangles.read().cast(),
    ] {
        flag(array.is_null());
        if !array.is_null() {
            array_state(&array);
        }
    }
}
fn mutate(band: &AisRubberBand, operation: i32, value: f64) -> Result<(), Failure> {
    let red = Color::named(NamedColor::RED);
    match operation {
        0 => band.set_rectangle(10, 20, 30, 40),
        1 => band.set_rectangle(30, 40, 10, 20),
        2 => band.add_point(Vec2::new(20, 10)),
        3 => return band.remove_last_point(),
        4 => band.clear_points(),
        5 => band.set_polygon_closed(false),
        6 => band.set_polygon_closed(true),
        7 => band.set_filling(false),
        8 => band.set_filling(true),
        9 => return band.set_filling_color(red, value),
        10 => return band.set_fill_transparency(value),
        11 => return band.set_fill_color(red),
        12 => band.set_line_color(red),
        13 => return band.set_line_width(value),
        14 => band.set_line_type(AspectTypeOfLine::Dot),
        15 => {
            flag(band.fill_triangles()?);
            caches(band);
        }
        18 => {
            band.clear_points();
            for (x, y) in [(8, 8), (8, 36), (20, 20), (36, 36), (36, 8)] {
                band.add_point(Vec2::new(x, y));
            }
        }
        19 => {
            let array = band.rubber_band_base().fields.triangles.read().clone();
            if !array.is_null() {
                for index in 1..=array.vertex_number() {
                    array.set_vertex_normal(index, 0., 1., 0.)?;
                }
            }
        }
        _ => unreachable!(),
    }
    Ok(())
}
pub fn rubber_band_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let ctor = r.integer();
    let width = r.real();
    let closed = r.flag();
    let transparency = r.real();
    let count = r.integer();
    let value = r.real();
    let (driver, _) = display_driver::SensitiveDisplayDriver::new_handle(0);
    let viewer = V3dViewer::new(&driver).unwrap();
    let manager = PrsMgrPresentationManager::new(&viewer.structure_manager());
    let prs = Graphic3dStructure::new(&viewer.structure_manager(), &Handle::default()).unwrap();
    attempt(|| {
        let white = Color::named(NamedColor::WHITE);
        let band = match ctor {
            0 => AisRubberBand::new()?,
            1 => AisRubberBand::with_line(white, AspectTypeOfLine::Dash, width, closed)?,
            _ => AisRubberBand::with_filling(
                white,
                AspectTypeOfLine::Dash,
                Color::named(NamedColor::BLUE),
                transparency,
                width,
                closed,
            )?,
        };
        snapshot(&band);
        for _ in 0..count {
            let op = r.integer();
            attempt(|| {
                if op == 16 || op == 17 {
                    let borders = band.rubber_band_base().fields.borders.read().clone();
                    let triangles = band.rubber_band_base().fields.triangles.read().clone();
                    band.compute(&manager, &prs.cast(), if op == 16 { 0 } else { 1 })?;
                    flag(borders == *band.rubber_band_base().fields.borders.read());
                    flag(triangles == *band.rubber_band_base().fields.triangles.read());
                    caches(&band);
                    return Ok(());
                }
                mutate(&band, op, value)
            });
            snapshot(&band);
        }
        int(prs.groups().len() as i32);
        Ok(())
    });
    end();
}
