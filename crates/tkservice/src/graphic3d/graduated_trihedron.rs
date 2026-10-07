//! `Graphic3d_GraduatedTrihedron`: the parameters of the obsolete graduated trihedron of a
//! view: the axis aspects, the fonts and the grid, with the cubic axes callback.
use super::Graphic3dCView;
use crate::font::FontFontAspect;
use rustycad_tkernel::quantity::{Color, NamedColor};
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::exceptions::out_of_range;
use rustycad_tkernel::tcollection::{AsciiString, ExtendedString};

/// `Graphic3d_GraduatedTrihedron::AxisAspect`: the drawing parameters of one axis.
/// Example: `Graphic3dGraduatedTrihedronAxisAspect::new("X", red, red)`.
#[derive(Clone, Debug, PartialEq)]
pub struct Graphic3dGraduatedTrihedronAxisAspect {
    /// `myName`.
    pub name: ExtendedString,
    /// `myToDrawName`.
    pub to_draw_name: bool,
    /// `myToDrawTickmarks`.
    pub to_draw_tickmarks: bool,
    /// `myToDrawValues`.
    pub to_draw_values: bool,
    /// `myNameColor`.
    pub name_color: Color,
    /// `myTickmarksNumber`: the splits along the axis.
    pub tickmarks_number: i32,
    /// `myTickmarksLength`.
    pub tickmarks_length: i32,
    /// `myColor`: the color of the axis and the values.
    pub color: Color,
    /// `myValuesOffset`.
    pub values_offset: i32,
    /// `myNameOffset`.
    pub name_offset: i32,
}

impl Default for Graphic3dGraduatedTrihedronAxisAspect {
    /// The default arguments: no name, black, offsets 10 and 30, 5 tickmarks of length 10.
    fn default() -> Self {
        let black = Color::named(NamedColor::BLACK);
        Self::new("", black, black)
    }
}

impl Graphic3dGraduatedTrihedronAxisAspect {
    /// An aspect with a name and colors, the other arguments at their defaults;
    /// `Graphic3dGraduatedTrihedronAxisAspect::new("X", red, red)`.
    pub fn new(name: &str, name_color: Color, color: Color) -> Self {
        Self {
            name: ExtendedString::new(name),
            to_draw_name: true,
            to_draw_tickmarks: true,
            to_draw_values: true,
            name_color,
            tickmarks_number: 5,
            tickmarks_length: 10,
            color,
            values_offset: 10,
            name_offset: 30,
        }
    }
}

/// `MinMaxValuesCallback`: the cubic axes callback of a view.
pub type Graphic3dGraduatedTrihedronCallback = fn(&dyn Graphic3dCView);

/// The graduated trihedron parameters; `Graphic3dGraduatedTrihedron::default().names_size() == 12`.
#[derive(Clone, Debug)]
pub struct Graphic3dGraduatedTrihedron {
    /// `myCubicAxesCallback`: null by default.
    pub cubic_axes_callback: Option<Graphic3dGraduatedTrihedronCallback>,
    /// `myNamesFont`.
    pub names_font: AsciiString,
    /// `myNamesStyle`.
    pub names_style: FontFontAspect,
    /// `myNamesSize`.
    pub names_size: i32,
    /// `myValuesFont`.
    pub values_font: AsciiString,
    /// `myValuesStyle`.
    pub values_style: FontFontAspect,
    /// `myValuesSize`.
    pub values_size: i32,
    /// `myArrowsLength`.
    pub arrows_length: f32,
    /// `myGridColor`.
    pub grid_color: Color,
    /// `myToDrawGrid`.
    pub to_draw_grid: bool,
    /// `myToDrawAxes`.
    pub to_draw_axes: bool,
    /// `myAxes`: the X, Y and Z aspects.
    pub axes: [Graphic3dGraduatedTrihedronAxisAspect; 3],
}

impl Default for Graphic3dGraduatedTrihedron {
    /// Arial bold names and regular values of size 12, arrows of 30, a white grid, the grid
    /// and the axes drawn, X red, Y green, Z blue.
    fn default() -> Self {
        let axis = |name: &str, color: NamedColor| {
            Graphic3dGraduatedTrihedronAxisAspect::new(
                name,
                Color::named(color),
                Color::named(color),
            )
        };
        Self {
            cubic_axes_callback: None,
            names_font: AsciiString::from("Arial"),
            names_style: FontFontAspect::Bold,
            names_size: 12,
            values_font: AsciiString::from("Arial"),
            values_style: FontFontAspect::Regular,
            values_size: 12,
            arrows_length: 30.0,
            grid_color: Color::named(NamedColor::WHITE),
            to_draw_grid: true,
            to_draw_axes: true,
            axes: [
                axis("X", NamedColor::RED),
                axis("Y", NamedColor::GREEN),
                axis("Z", NamedColor::BLUE1),
            ],
        }
    }
}

/// The checked axis index of `ChangeAxisAspect` and `AxisAspect`.
fn axis_index(index: i32, member: &str) -> Result<usize, Failure> {
    if !(0..=2).contains(&index) {
        return Err(out_of_range(&format!(
            "Graphic3d_GraduatedTrihedron::{member}: theIndex is out of bounds [0,2]: {index}"
        )));
    }
    Ok(index as usize)
}

impl Graphic3dGraduatedTrihedron {
    /// `ChangeAxisAspect`: the aspect of an axis 0..2; `trihedron.change_axis_aspect(1)?.tickmarks_number = 3`.
    pub fn change_axis_aspect(
        &mut self,
        index: i32,
    ) -> Result<&mut Graphic3dGraduatedTrihedronAxisAspect, Failure> {
        Ok(&mut self.axes[axis_index(index, "ChangeAxisAspect")?])
    }
    /// `AxisAspect`: the aspect of an axis 0..2; `trihedron.axis_aspect(0)?.name`.
    pub fn axis_aspect(
        &self,
        index: i32,
    ) -> Result<&Graphic3dGraduatedTrihedronAxisAspect, Failure> {
        Ok(&self.axes[axis_index(index, "AxisAspect")?])
    }
    /// `NamesSize`; `trihedron.names_size()`.
    pub fn names_size(&self) -> i32 {
        self.names_size
    }
    /// `CubicAxesCallback`: runs the callback on the view; false without one;
    /// `trihedron.cubic_axes_callback(view)`.
    pub fn run_cubic_axes_callback(&self, view: &dyn Graphic3dCView) -> bool {
        match self.cubic_axes_callback {
            Some(callback) => {
                callback(view);
                true
            }
            None => false,
        }
    }
}
