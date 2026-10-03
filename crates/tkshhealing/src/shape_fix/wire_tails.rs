//! Aggregate `ShapeFix_Wire::FixTails` edge removal and split ordering.
use super::ShapeFixWire;
use crate::shape_extend::ShapeExtendStatus;
use crate::shape_fix::FixRoot;
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topods::{self, Builder, Edge, Wire};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::Orientation;

impl ShapeFixWire {
    /// Removes edge tails found by the shared analyzer; `let changed = fixer.fix_tails()?`.
    pub fn fix_tails(&self) -> Result<bool, Failure> {
        if self.state.read().max_tail_width < 0.0 || !self.is_ready() {
            return Ok(false);
        }
        self.set_last_fix_status(ShapeExtendStatus::Ok);
        self.update_wire_if_context()?;
        let mut walk = TailWalk::new(self.nb_edges());
        while walk.has_pair() {
            if !self.inspect_tail_pair(&mut walk)? {
                continue;
            }
            self.apply_tail_pair(&mut walk)?;
        }
        self.set_notches_status_from_last_fix();
        Ok(self.last_fix_status(ShapeExtendStatus::Done))
    }

    pub(super) fn update_wire_if_context(&self) -> Result<(), Failure> {
        if !self.context().is_null() {
            self.update_wire()?;
        }
        Ok(())
    }

    fn inspect_tail_pair(&self, walk: &mut TailWalk) -> Result<bool, Failure> {
        let data = self.wire_data()?;
        let edges = [data.edge(walk.first)?, data.edge(walk.second)?];
        let Some(parts) = self.analyze_tail_pair(&edges, walk.check_angle)? else {
            walk.advance_pair();
            return Ok(false);
        };
        let plan = TailPlan::new(parts);
        if !plan.keeps_a_wire(data.nb_edges()) {
            walk.advance_pair();
            return Ok(false);
        }
        walk.parts = Some(plan);
        Ok(true)
    }

    fn analyze_tail_pair(
        &self,
        edges: &[Edge; 2],
        check_angle: bool,
    ) -> Result<Option<[Edge; 4]>, Failure> {
        let mut parts = std::array::from_fn(|_| Edge::default());
        let matched = self.working_analyzer().check_tail(
            &edges[0],
            &edges[1],
            if check_angle {
                self.state.read().max_tail_angle_sine
            } else {
                -1.0
            },
            self.state.read().max_tail_width,
            self.max_tolerance(),
            &mut parts,
        )?;
        Ok(matched.then_some(parts))
    }

    fn apply_tail_pair(&self, walk: &mut TailWalk) -> Result<(), Failure> {
        let plan = walk.parts.take().ok_or_else(missing_tail_plan)?;
        let data = self.wire_data()?;
        let originals = [data.edge(walk.first)?, data.edge(walk.second)?];
        self.replace_split_edges(&data, walk, &plan, &originals)?;
        if plan.remove_count == 2 {
            self.remove_two_tail_edges(walk, &plan)?;
        } else {
            self.remove_one_tail_edge(walk, &plan, &originals)?;
        }
        Ok(())
    }

    fn replace_split_edges(
        &self,
        data: &crate::shape_extend::ShapeExtendWireData,
        walk: &TailWalk,
        plan: &TailPlan,
        originals: &[Edge; 2],
    ) -> Result<(), Failure> {
        for (edge_index, original) in originals.iter().enumerate() {
            if !plan.splits[edge_index] {
                continue;
            }
            self.replace_split_edge_in_context(original, &plan.parts[edge_index])?;
            apply_oriented_parts(data, walk, plan, original.orientation(), edge_index)?;
        }
        Ok(())
    }

    fn replace_split_edge_in_context(&self, edge: &Edge, parts: &[Edge; 2]) -> Result<(), Failure> {
        let context = self.context();
        if context.is_null() {
            return Ok(());
        }
        let mut wire = Wire::default();
        Builder.make_wire(&mut wire);
        let mut wire_shape = wire.into_shape();
        for part in parts.iter().filter(|edge| !edge.is_null()) {
            Builder.add(&mut wire_shape, &part.clone().into_shape())?;
        }
        let forward = Edge::unchecked(edge.oriented(Orientation::Forward));
        let wire = topods::wire(&wire_shape)?;
        context.replace(&forward.into_shape(), &wire.into_shape())
    }

    fn remove_two_tail_edges(&self, walk: &mut TailWalk, plan: &TailPlan) -> Result<(), Failure> {
        let seam_index = walk.first
            + plan.splits[0] as i32
            + if walk.first < walk.second {
                0
            } else {
                plan.splits[1] as i32
            };
        self.fix_dummy_seam(seam_index)?;
        self.update_wire_if_context()?;
        self.or_last_fix_status(ShapeExtendStatus::Done);
        walk.advance_after_two_removals(plan);
        Ok(())
    }

    fn remove_one_tail_edge(
        &self,
        walk: &mut TailWalk,
        plan: &TailPlan,
        originals: &[Edge; 2],
    ) -> Result<(), Failure> {
        let removed = if plan.parts[0][0].is_null() { 1 } else { 0 };
        walk.advance_after_one_removal(removed, plan);
        self.remove_tail_edge(walk, plan, removed)?;
        self.remove_from_context(&originals[removed])?;
        self.update_wire_if_context()?;
        self.or_last_fix_status(ShapeExtendStatus::Done);
        Ok(())
    }

    fn remove_tail_edge(
        &self,
        walk: &TailWalk,
        plan: &TailPlan,
        removed_index: usize,
    ) -> Result<(), Failure> {
        let index = walk.edge_index(removed_index, plan);
        self.wire_data()?.remove(index)
    }

    fn remove_from_context(&self, edge: &Edge) -> Result<(), Failure> {
        let context = self.context();
        if context.is_null() {
            return Ok(());
        }
        let forward = Edge::unchecked(edge.oriented(Orientation::Forward));
        context.remove(&forward.into_shape())
    }

    pub(super) fn set_notches_status_from_last_fix(&self) {
        let status = self.state.read().last_fix_status;
        self.state.write().notches_status = status;
    }
}

struct TailWalk {
    count: i32,
    first: i32,
    second: i32,
    check_angle: bool,
    parts: Option<TailPlan>,
}

impl TailWalk {
    fn new(count: i32) -> Self {
        Self {
            count,
            first: count,
            second: 1,
            check_angle: true,
            parts: None,
        }
    }

    fn has_pair(&self) -> bool {
        self.count >= 2 && self.second <= self.count
    }

    fn advance_pair(&mut self) {
        self.first = self.second;
        self.second += 1;
        self.check_angle = true;
    }

    fn advance_after_two_removals(&mut self, plan: &TailPlan) {
        self.check_angle = true;
        if plan.splits == [true, true] {
            self.advance_pair();
        } else if plan.splits[0] == plan.splits[1] {
            self.count -= 2;
            self.move_back_or_wrap(self.count);
            self.check_angle = false;
        } else {
            self.count -= 1;
            if plan.splits[0] {
                self.first = self.first.min(self.count);
            } else {
                self.move_back_or_wrap(self.count);
            }
        }
    }

    fn move_back_or_wrap(&mut self, edge_count: i32) {
        if self.second >= 3 {
            self.first -= 1;
            self.second -= 1;
        } else {
            self.first = edge_count;
            self.second = 1;
        }
    }

    fn advance_after_one_removal(&mut self, removed: usize, plan: &TailPlan) {
        self.check_angle = false;
        self.count -= 1;
        if plan.splits[removed] {
            if removed == 0 {
                self.move_back_or_wrap(self.count);
            } else if self.second <= 1 {
                self.first = self.count;
            }
        }
    }

    fn edge_index(&self, removed: usize, plan: &TailPlan) -> i32 {
        if removed != 0 || !plan.splits[0] {
            self.index(removed)
        } else {
            self.index(removed) + 1
        }
    }

    fn index(&self, edge: usize) -> i32 {
        if edge == 0 { self.first } else { self.second }
    }
}

struct TailPlan {
    parts: [[Edge; 2]; 2],
    splits: [bool; 2],
    remove_count: usize,
}

impl TailPlan {
    fn new(parts: [Edge; 4]) -> Self {
        let parts = [
            [parts[0].clone(), parts[1].clone()],
            [parts[2].clone(), parts[3].clone()],
        ];
        let splits = [!parts[0][1].is_null(), !parts[1][1].is_null()];
        let remove_count = parts.iter().filter(|pair| !pair[0].is_null()).count();
        Self {
            parts,
            splits,
            remove_count,
        }
    }

    fn keeps_a_wire(&self, count: i32) -> bool {
        count + i32::from(self.splits[0]) + i32::from(self.splits[1]) > self.remove_count as i32
    }
}

fn apply_oriented_parts(
    data: &crate::shape_extend::ShapeExtendWireData,
    walk: &TailWalk,
    plan: &TailPlan,
    orientation: Orientation,
    edge_index: usize,
) -> Result<(), Failure> {
    let position = walk.index(edge_index);
    let start = usize::from(orientation == Orientation::Reversed);
    let add = if edge_index == 0 || walk.second < walk.first {
        0
    } else {
        i32::from(plan.splits[0])
    };
    let mut first = plan.parts[edge_index][start].clone();
    let mut second = plan.parts[edge_index][1 - start].clone();
    first.set_orientation(orientation);
    second.set_orientation(orientation);
    data.set(&first, position + add)?;
    data.add_edge(&second, position + 1 + add)
}

fn missing_tail_plan() -> Failure {
    Failure::new("ShapeFix_Wire::FixTails requires an analyzed edge pair")
}
