use super::*;
impl Decomposition<'_> {
    pub(super) fn split_groups(
        &self,
        groups: &[WalkingGroup],
        constructor: &GeomIntLineConstructor,
        avoid: bool,
        output: &mut Vec<Handle<dyn IntPatchLine>>,
    ) -> Result<(), Failure> {
        let count = if avoid { 1 } else { constructor.nb_parts()? };
        for index in 1..=count {
            let (first, last) = if avoid {
                (1, self.line.nb_pnts()?)
            } else {
                let (a, b) = constructor.part(index)?;
                (a as i32, b as i32)
            };
            self.split_part(groups, first, last, output)?;
        }
        Ok(())
    }
    fn split_part(
        &self,
        groups: &[WalkingGroup],
        first: i32,
        last: i32,
        output: &mut Vec<Handle<dyn IntPatchLine>>,
    ) -> Result<(), Failure> {
        let mut points = Handle::new(IntSurfLineOn2S::new(None));
        for (index, group) in groups.iter().enumerate() {
            if group.boundary {
                continue;
            }
            if self.append_part_group(&points, groups, index, first, last)? {
                self.emit_line(&points, output);
                points = Handle::new(IntSurfLineOn2S::new(None));
            }
        }
        self.emit_line(&points, output);
        Ok(())
    }
    fn append_part_group(
        &self,
        points: &Handle<IntSurfLineOn2S>,
        groups: &[WalkingGroup],
        index: usize,
        first: i32,
        last: i32,
    ) -> Result<bool, Failure> {
        let group = &groups[index];
        let (low, high) = (group.indices[0], *group.indices.last().unwrap());
        let first_inside = first >= low && first <= high;
        let last_inside = last >= low && last <= high;
        let whole = !first_inside && !last_inside && first < low && last > high;
        if !whole && !first_inside && !last_inside {
            return Ok(false);
        }
        self.append_group(
            points,
            group,
            index,
            groups.len(),
            first,
            last,
            first_inside,
            last_inside,
        )?;
        Ok((whole || (first_inside && !last_inside)) && ends_line(groups, index))
    }
    #[allow(clippy::too_many_arguments)]
    fn append_group(
        &self,
        points: &Handle<IntSurfLineOn2S>,
        group: &WalkingGroup,
        index: usize,
        count: usize,
        first: i32,
        last: i32,
        first_inside: bool,
        last_inside: bool,
    ) -> Result<(), Failure> {
        let has_first = group.ends.len() == 2 || (!group.ends.is_empty() && index != 0);
        let has_last = group.ends.len() == 2 || (!group.ends.is_empty() && index + 1 != count);
        if !first_inside && has_first {
            points.add(&group.ends[0]);
        }
        for &parameter in &group.indices {
            if (first_inside && parameter < first) || (last_inside && parameter > last) {
                continue;
            }
            points.add(&self.line.point(parameter)?);
        }
        if !last_inside && has_last {
            points.add(group.ends.last().unwrap());
        }
        Ok(())
    }
    fn emit_line(
        &self,
        points: &Handle<IntSurfLineOn2S>,
        output: &mut Vec<Handle<dyn IntPatchLine>>,
    ) {
        if points.nb_points() <= 1 {
            return;
        }
        let line = Handle::new(IntPatchWLine::new_undecided(points, false));
        line.set_creating_way_info(self.line.get_creating_way());
        output.push(line.into_cast());
    }
}
fn ends_line(groups: &[WalkingGroup], index: usize) -> bool {
    if let Some(neighbour) = groups.get(index + 1) {
        return !(neighbour.boundary && neighbour.indices.is_empty());
    }
    true
}
