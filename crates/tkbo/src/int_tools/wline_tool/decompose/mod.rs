use super::WLineTool;
use crate::int_tools::Context;
use rustycad_tkbrep::topods::Face;
use rustycad_tkernel::{
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::{
    adaptor3d::Adaptor3dSurface, geom::SurfaceBounds, geom_adaptor::GeomAdaptorSurface,
};
use rustycad_tkgeomalgo::{
    geom_int::{GeomIntLineConstructor, adjust_periodic},
    int_patch::{IntPatchLine, IntPatchPointLine, IntPatchWLine},
    int_surf::{IntSurfLineOn2S, IntSurfPntOn2S},
};
use rustycad_tkmath::gp::{self, Point2, Vector2};
mod boundary;
mod correction;
mod find_point;
mod split;
use boundary::{BoundaryAxis, adjust_by_neighbour};
struct WalkingGroup {
    indices: Vec<i32>,
    boundary: bool,
    ends: Vec<IntSurfPntOn2S>,
}
struct Decomposition<'a> {
    line: &'a Handle<IntPatchWLine>,
    surfaces: [&'a Handle<GeomAdaptorSurface>; 2],
    faces: [&'a Face; 2],
    context: &'a Handle<Context>,
    tolerance: f64,
}
impl WLineTool {
    /// Splits walking lines at periodic boundaries using native endpoint corrections; `WLineTool::decomposition_of_wline(&line, &s1, &s2, &f1, &f2, &parts, true, 1e-7, &mut out, &context)?`.
    #[allow(clippy::too_many_arguments)]
    pub fn decomposition_of_wline(
        line: &Handle<IntPatchWLine>,
        first: &Handle<GeomAdaptorSurface>,
        last: &Handle<GeomAdaptorSurface>,
        face1: &Face,
        face2: &Face,
        constructor: &GeomIntLineConstructor,
        avoid_constructor: bool,
        tolerance: f64,
        output: &mut Vec<Handle<dyn IntPatchLine>>,
        context: &Handle<Context>,
    ) -> Result<bool, Failure> {
        if line.nb_pnts()? == 0 || (!avoid_constructor && constructor.nb_parts()? == 0) {
            return Ok(false);
        }
        let operation = Decomposition {
            line,
            surfaces: [first, last],
            faces: [face1, face2],
            context,
            tolerance,
        };
        let mut groups = operation.groups()?;
        if groups.len() <= 1 {
            return Ok(false);
        }
        operation.correct_ends(&mut groups)?;
        operation.split_groups(&groups, constructor, avoid_constructor, output)?;
        Ok(true)
    }
}
impl Decomposition<'_> {
    fn groups(&self) -> Result<Vec<WalkingGroup>, Failure> {
        let mut groups: Vec<WalkingGroup> = Vec::new();
        for index in 1..=self.line.nb_pnts()? {
            let boundary = boundary::point_on_boundary(self.line.point(index)?, &self.surfaces)?;
            if groups.last().is_none_or(|group| group.boundary != boundary) {
                groups.push(WalkingGroup {
                    indices: Vec::new(),
                    boundary,
                    ends: Vec::new(),
                });
            }
            groups
                .last_mut()
                .expect("a group was inserted for this point")
                .indices
                .push(index);
        }
        Ok(groups)
    }
    fn correct_ends(&self, groups: &mut [WalkingGroup]) -> Result<(), Failure> {
        for index in 0..groups.len() {
            if groups[index].boundary {
                continue;
            }
            groups[index].ends = self.correct_group_ends(groups, index)?;
        }
        Ok(())
    }
    fn correct_group_ends(
        &self,
        groups: &[WalkingGroup],
        index: usize,
    ) -> Result<Vec<IntSurfPntOn2S>, Failure> {
        let mut ends = Vec::new();
        for first in [true, false] {
            let Some(neighbour) = neighbour_group(groups, index, first) else {
                continue;
            };
            let endpoint = if first {
                *neighbour.indices.last().unwrap()
            } else {
                neighbour.indices[0]
            };
            let mut point = self.line.point(endpoint)?;
            if groups[index].indices.len() >= 2 {
                self.correct_point(&mut point, &groups[index], first)?;
            }
            ends.push(point);
        }
        Ok(ends)
    }
    fn neighbour(
        &self,
        group: &WalkingGroup,
        first: bool,
        surface: usize,
    ) -> Result<Point2, Failure> {
        let index = if first {
            group.indices[0]
        } else {
            *group.indices.last().unwrap()
        };
        Ok(self.line.point(index)?.value_on_surface(surface == 0))
    }
}
fn neighbour_group(groups: &[WalkingGroup], index: usize, first: bool) -> Option<&WalkingGroup> {
    let index = if first {
        index.checked_sub(1)?
    } else {
        index + 1
    };
    groups.get(index).filter(|group| group.boundary)
}
