// Rust translation of the editing members of ShapeExtend_WireData.cxx (SetLast,
// SetDegeneratedLast, Add, AddOriented, Remove, Set, Reverse) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Adding, removing, replacing and reversing the edges of a wire data.
use super::{ShapeExtendWireData, WireDataFields, position};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{self, Edge, Shape, ShapeIterator, Wire};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

/// FORWARD or REVERSED: the orientations of a manifold edge.
fn is_manifold(shape: &Shape) -> bool {
    matches!(
        shape.orientation(),
        Orientation::Forward | Orientation::Reversed
    )
}

impl WireDataFields {
    /// `myEdges->Append(edge)` for position 0, `InsertBefore(atnum, edge)` otherwise.
    fn insert_edge(&mut self, edge: Shape, atnum: i32) -> Result<(), Failure> {
        if atnum == 0 {
            self.edges.append(edge);
            return Ok(());
        }
        let index = position(atnum, "ShapeExtend_WireData::Add")?;
        self.edges.insert_before(index, edge)?;
        Ok(())
    }
}

impl ShapeExtendWireData {
    /// `SetLast(num)`: rotates the list so that the edge of rank `num` is the last one;
    /// nothing for 0; `data.set_last(2)?`.
    pub fn set_last(&self, num: i32) -> Result<(), Failure> {
        if num == 0 {
            return Ok(());
        }
        let mut fields = self.fields.write();
        let nb = fields.edges.length();
        let last = usize::try_from(nb).unwrap_or(0);
        for _ in (num + 1..=nb).rev() {
            let edge = fields.edges.value(last)?.clone();
            fields.edges.remove(last)?;
            fields.edges.insert_before(1, edge)?;
        }
        fields.seam_f = -1;
        Ok(())
    }

    /// `SetDegeneratedLast()`: makes the first degenerated edge the last one (`SetLast`);
    /// `data.set_degenerated_last()?`.
    pub fn set_degenerated_last(&self) -> Result<(), Failure> {
        for i in 1..=self.nb_edges() {
            if BRepTool::degenerated(&self.edge(i)?)? {
                return self.set_last(i);
            }
        }
        Ok(())
    }

    /// `Add(edge, atnum)`: appends the edge (inserts it before rank `atnum` when not 0);
    /// in the manifold mode an INTERNAL or EXTERNAL edge goes to the non-manifold edges
    /// instead. The orientation is tested before the null check, so a null edge (EXTERNAL
    /// natively) joins the non-manifold edges in the manifold mode; `data.add_edge(&e, 0)?`.
    pub fn add_edge(&self, edge: &Edge, atnum: i32) -> Result<(), Failure> {
        let mut fields = self.fields.write();
        if !is_manifold(edge) && fields.manifold_mode {
            fields.nonmanifold.append(edge.clone().into_shape());
            return Ok(());
        }
        if edge.is_null() {
            return Ok(());
        }
        fields.insert_edge(edge.clone().into_shape(), atnum)?;
        fields.seam_f = -1;
        Ok(())
    }

    /// `Add(wire, atnum)`: the edges of the wire in order, appended or inserted from rank
    /// `atnum`; its INTERNAL and EXTERNAL edges go to the non-manifold edges in the
    /// manifold mode, else after all the edges; nothing for a null wire;
    /// `data.add_wire(&w, 0)?`.
    pub fn add_wire(&self, wire: &Wire, atnum: i32) -> Result<(), Failure> {
        if wire.is_null() {
            return Ok(());
        }
        let mut non_manifold = Vec::new();
        let mut n = atnum;
        let mut it = ShapeIterator::new(wire, true, true);
        while it.more() {
            let edge = topods::edge(it.value()?)?.into_shape();
            it.next();
            let mut fields = self.fields.write();
            if !is_manifold(&edge) {
                if fields.manifold_mode {
                    fields.nonmanifold.append(edge);
                } else {
                    non_manifold.push(edge);
                }
                continue;
            }
            fields.insert_edge(edge, n)?;
            n += i32::from(n != 0);
        }
        let mut fields = self.fields.write();
        for edge in non_manifold {
            fields.edges.append(edge);
        }
        fields.seam_f = -1;
        Ok(())
    }

    /// `Add(wire data, atnum)`: the edges of `other` in order, appended or inserted from
    /// rank `atnum`, its INTERNAL and EXTERNAL edges after all the edges. Its non-manifold
    /// edges go to the non-manifold edges in the manifold mode; without it, the native
    /// loop inserts `other->Edge(i)` for i up to the number of non-manifold edges (the
    /// ordinary edges, not the non-manifold ones), which is kept. Both loops read the
    /// counts of `other` again at each step, as natively, so adding a list to itself does
    /// not end (natively neither); `data.add_wire_data(&other, 0)?`.
    pub fn add_wire_data(&self, other: &ShapeExtendWireData, atnum: i32) -> Result<(), Failure> {
        let mut non_manifold = Vec::new();
        let mut n = atnum;
        let mut i = 1;
        while i <= other.nb_edges() {
            let edge = other.edge(i)?;
            i += 1;
            if matches!(
                edge.orientation(),
                Orientation::Internal | Orientation::External
            ) {
                non_manifold.push(edge.into_shape());
                continue;
            }
            self.fields.write().insert_edge(edge.into_shape(), n)?;
            n += i32::from(n != 0);
        }
        for edge in non_manifold {
            self.fields.write().edges.append(edge);
        }
        self.add_foreign_nonmanifold(other, n)?;
        self.fields.write().seam_f = -1;
        Ok(())
    }

    /// The last loop of `Add(wire data, atnum)`, over the non-manifold edges of `other`.
    fn add_foreign_nonmanifold(
        &self,
        other: &ShapeExtendWireData,
        mut n: i32,
    ) -> Result<(), Failure> {
        let mut i = 1;
        while i <= other.nb_non_manifold_edges() {
            if self.manifold_mode() {
                let edge = other.nonmanifold_edge(i)?.into_shape();
                self.fields.read().nonmanifold.append(edge);
            } else {
                // Natively `wire->Edge(i)`, not `NonmanifoldEdge(i)`.
                let edge = other.edge(i)?.into_shape();
                self.fields.write().insert_edge(edge, n)?;
                n += i32::from(n != 0);
            }
            i += 1;
        }
        Ok(())
    }

    /// `Add(shape, atnum)`: `Add` of the edge or of the wire; any other shape is ignored;
    /// Standard_NullObject for a null shape (a null dereference natively);
    /// `data.add_shape(&s, 0)?`.
    pub fn add_shape(&self, shape: &Shape, atnum: i32) -> Result<(), Failure> {
        match shape.shape_type()? {
            ShapeEnum::Edge => self.add_edge(&topods::edge(shape)?, atnum),
            ShapeEnum::Wire => self.add_wire(&topods::wire(shape)?, atnum),
            _ => Ok(()),
        }
    }

    /// `AddOriented(edge, mode)`: mode 0 appends, 1 appends reversed, 2 prepends, 3
    /// prepends reversed; nothing for a null edge or a negative mode (a mode above 3
    /// inserts at `mode / 2`, as natively); `data.add_oriented_edge(&e, 1)?`.
    pub fn add_oriented_edge(&self, edge: &Edge, mode: i32) -> Result<(), Failure> {
        if edge.is_null() || mode < 0 {
            return Ok(());
        }
        let mut oriented = edge.clone();
        if mode == 1 || mode == 3 {
            oriented.reverse();
        }
        self.add_edge(&oriented, mode / 2)
    }

    /// `AddOriented(wire, mode)`: as [`Self::add_oriented_edge`] with the wire reversed
    /// for modes 1 and 3; `data.add_oriented_wire(&w, 2)?`.
    pub fn add_oriented_wire(&self, wire: &Wire, mode: i32) -> Result<(), Failure> {
        if wire.is_null() || mode < 0 {
            return Ok(());
        }
        let mut oriented = wire.clone();
        if mode == 1 || mode == 3 {
            oriented.reverse();
        }
        self.add_wire(&oriented, mode / 2)
    }

    /// `AddOriented(shape, mode)`: of the edge or of the wire; Standard_NullObject for a
    /// null shape (a null dereference natively); `data.add_oriented_shape(&s, 0)?`.
    pub fn add_oriented_shape(&self, shape: &Shape, mode: i32) -> Result<(), Failure> {
        match shape.shape_type()? {
            ShapeEnum::Edge => self.add_oriented_edge(&topods::edge(shape)?, mode),
            ShapeEnum::Wire => self.add_oriented_wire(&topods::wire(shape)?, mode),
            _ => Ok(()),
        }
    }

    /// `Remove(num)`: removes the edge of rank `num` (the last one for 0);
    /// Standard_OutOfRange outside the list; `data.remove(0)?`.
    pub fn remove(&self, num: i32) -> Result<(), Failure> {
        let mut fields = self.fields.write();
        let rank = if num > 0 { num } else { fields.edges.length() };
        let index = position(rank, "ShapeExtend_WireData::Remove")?;
        fields.edges.remove(index)?;
        fields.seam_f = -1;
        Ok(())
    }

    /// `Set(edge, num)`: replaces the edge of rank `num` (the last one for 0 or below); in
    /// the manifold mode an INTERNAL or EXTERNAL edge replaces the non-manifold edge of
    /// rank `num` instead, or is appended to them when `num` is past their end;
    /// Standard_OutOfRange outside the list; `data.set(&e, 1)?`.
    pub fn set(&self, edge: &Edge, num: i32) -> Result<(), Failure> {
        let mut fields = self.fields.write();
        let shape = edge.clone().into_shape();
        if !is_manifold(edge) && fields.manifold_mode {
            let count = fields.nonmanifold.sequence(|s| s.length());
            if num <= count {
                let index = position(num, "ShapeExtend_WireData::Set")?;
                fields
                    .nonmanifold
                    .change_sequence(|s| s.set_value(index, shape))?;
            } else {
                fields.nonmanifold.append(shape);
            }
        } else {
            let rank = if num > 0 { num } else { fields.edges.length() };
            let index = position(rank, "ShapeExtend_WireData::Set")?;
            fields.edges.set_value(index, shape)?;
        }
        fields.seam_f = -1;
        Ok(())
    }

    /// `Reverse()`: reverses the order of the edges and each edge;
    /// `data.reverse()?`.
    pub fn reverse(&self) -> Result<(), Failure> {
        let mut fields = self.fields.write();
        let nb = usize::try_from(fields.edges.length()).unwrap_or(0);
        for i in 1..=nb / 2 {
            let first = fields.edges.value(i)?.reversed();
            let second = fields.edges.value(nb + 1 - i)?.reversed();
            fields.edges.set_value(i, second)?;
            fields.edges.set_value(nb + 1 - i, first)?;
        }
        if nb % 2 == 1 {
            let middle = nb.div_ceil(2);
            let reversed = fields.edges.value(middle)?.reversed();
            fields.edges.set_value(middle, reversed)?;
        }
        fields.seam_f = -1;
        Ok(())
    }
}
