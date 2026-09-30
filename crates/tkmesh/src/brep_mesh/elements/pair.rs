use rustycad_tkernel::standard::Failure;
/// Up to two adjacent element indices with native negative sentinels; `PairOfIndex::default()`.
#[derive(Clone, Copy, Debug)]
pub struct PairOfIndex {
    indices: [i32; 2],
}
impl Default for PairOfIndex {
    fn default() -> Self {
        Self { indices: [-1; 2] }
    }
}
impl PairOfIndex {
    /// Restore both negative sentinels; `pair.clear()`.
    pub fn clear(&mut self) {
        self.indices = [-1; 2];
    }
    /// Append using the first negative slot; `pair.append(5)?`.
    pub fn append(&mut self, index: i32) -> Result<(), Failure> {
        if self.indices[0] < 0 {
            self.indices[0] = index;
            return Ok(());
        }
        self.require_second_empty(index, "Append")?;
        self.indices[1] = index;
        Ok(())
    }
    /// Shift the first slot to the second; `pair.prepend(5)?`.
    pub fn prepend(&mut self, index: i32) -> Result<(), Failure> {
        self.require_second_empty(index, "Prepend")?;
        self.indices[1] = self.indices[0];
        self.indices[0] = index;
        Ok(())
    }
    /// Test the first slot only, preserving native hole behavior; `pair.is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.indices[0] < 0
    }
    /// Count initialized slots according to native sentinels; `pair.extent()`.
    pub fn extent(&self) -> i32 {
        if self.indices[0] < 0 {
            0
        } else if self.indices[1] < 0 {
            1
        } else {
            2
        }
    }
    /// Read the first slot including its sentinel; `pair.first_index()`.
    pub fn first_index(&self) -> i32 {
        self.indices[0]
    }
    /// Read the last initialized slot or first sentinel; `pair.last_index()`.
    pub fn last_index(&self) -> i32 {
        if self.indices[1] < 0 {
            self.indices[0]
        } else {
            self.indices[1]
        }
    }
    /// Read a one-based slot without checking initialization; `pair.index(2)?`.
    pub fn index(&self, position: i32) -> Result<i32, Failure> {
        Ok(self.indices[pair_position(position, "Index")?])
    }
    /// Assign a slot, permitting native holes; `pair.set_index(2,7)?`.
    pub fn set_index(&mut self, position: i32, index: i32) -> Result<(), Failure> {
        self.indices[pair_position(position, "SetIndex")?] = index;
        Ok(())
    }
    /// Remove a slot, shifting the second for position one; `pair.remove_index(1)?`.
    pub fn remove_index(&mut self, position: i32) -> Result<(), Failure> {
        if pair_position(position, "RemoveIndex")? == 0 {
            self.indices[0] = self.indices[1];
        }
        self.indices[1] = -1;
        Ok(())
    }
    fn require_second_empty(&self, index: i32, operation: &str) -> Result<(), Failure> {
        if self.indices[1] < 0 {
            return Ok(());
        }
        Err(Failure::out_of_range(&format!(
            "BRepMesh_PairOfIndex::{operation}: index {index} with occupied pair {:?}; expected at most two indices",
            self.indices
        )))
    }
}
fn pair_position(position: i32, operation: &str) -> Result<usize, Failure> {
    if position == 1 || position == 2 {
        return Ok((position - 1) as usize);
    }
    Err(Failure::out_of_range(&format!(
        "BRepMesh_PairOfIndex::{operation}: position {position}; expected 1 or 2"
    )))
}
