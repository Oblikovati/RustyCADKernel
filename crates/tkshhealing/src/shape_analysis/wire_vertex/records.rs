use super::ShapeAnalysisWireVertex;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Xyz;

impl ShapeAnalysisWireVertex {
    /// Mark a shared topological vertex; `tool.set_same_vertex(1)?`.
    pub fn set_same_vertex(&mut self, num: i32) -> Result<(), Failure> {
        self.update_record(num, |record| record.status = 0)
    }
    /// Mark coordinate equality within vertex tolerances; `tool.set_same_coords(1)?`.
    pub fn set_same_coords(&mut self, num: i32) -> Result<(), Failure> {
        self.update_record(num, |record| record.status = 1)
    }
    /// Mark coordinate proximity within working precision; `tool.set_close(1)?`.
    pub fn set_close(&mut self, num: i32) -> Result<(), Failure> {
        self.update_record(num, |record| record.status = 2)
    }
    /// Record projection onto the following edge; `tool.set_end(1, point, 2.0)?`.
    pub fn set_end(&mut self, num: i32, position: Xyz, following: f64) -> Result<(), Failure> {
        self.update_record(num, |record| {
            record.status = 3;
            record.position = position;
            record.following = following;
        })
    }
    /// Record projection onto the preceding edge; `tool.set_start(1, point, 2.0)?`.
    pub fn set_start(&mut self, num: i32, position: Xyz, previous: f64) -> Result<(), Failure> {
        // Pinned OCCT writes myUFol here, despite the parameter and method names.
        self.update_record(num, |record| {
            record.status = 4;
            record.position = position;
            record.following = previous;
        })
    }
    /// Record an intersection; `tool.set_inters(1, point, 2.0, 3.0)?`.
    pub fn set_inters(
        &mut self,
        num: i32,
        position: Xyz,
        previous: f64,
        following: f64,
    ) -> Result<(), Failure> {
        self.update_record(num, |record| {
            record.status = 5;
            record.position = position;
            record.previous = previous;
            record.following = following;
        })
    }
    /// Record a disconnected junction; `tool.set_disjoined(1)?`.
    pub fn set_disjoined(&mut self, num: i32) -> Result<(), Failure> {
        self.update_record(num, |record| record.status = -1)
    }
    /// Read the native status number; `tool.status(1)?`.
    pub fn status(&self, num: i32) -> Result<i32, Failure> {
        Ok(self.read_record(num)?.status)
    }
    /// Read the recorded position; `tool.position(1)?`.
    pub fn position(&self, num: i32) -> Result<Xyz, Failure> {
        Ok(self.read_record(num)?.position)
    }
    /// Read the preceding parameter; `tool.u_previous(1)?`.
    pub fn u_previous(&self, num: i32) -> Result<f64, Failure> {
        Ok(self.read_record(num)?.previous)
    }
    /// Read the following parameter; `tool.u_following(1)?`.
    pub fn u_following(&self, num: i32) -> Result<f64, Failure> {
        Ok(self.read_record(num)?.following)
    }
    /// Return status, position and both parameters; `tool.data(1)?`.
    pub fn data(&self, num: i32) -> Result<(i32, Xyz, f64, f64), Failure> {
        let record = self.read_record(num)?;
        Ok((
            record.status,
            record.position,
            record.previous,
            record.following,
        ))
    }
    /// Find the next exact status after a rank; `tool.next_status(2, 0)?`.
    pub fn next_status(&self, status: i32, num: i32) -> Result<i32, Failure> {
        self.find_status(num, |candidate| candidate == status)
    }
    /// Find the next native criterion match; `tool.next_criter(1, 0)?`.
    pub fn next_criter(&self, criterion: i32, num: i32) -> Result<i32, Failure> {
        self.find_status(num, |status| criterion_matches(criterion, status))
    }
    fn find_status(&self, num: i32, matches: impl Fn(i32) -> bool) -> Result<i32, Failure> {
        let count = self.records.read().len() as i32;
        if count == 0 {
            return Ok(0);
        }
        for rank in num + 1..=count {
            if matches(self.status(rank)?) {
                return Ok(rank);
            }
        }
        Ok(0)
    }
}
fn criterion_matches(criterion: i32, status: i32) -> bool {
    match criterion {
        -1 => status < 0,
        0 => status == 0,
        // Native predicate excludes status 0 despite the header's >= 0 wording.
        1 => status > 0,
        2 => (0..=2).contains(&status),
        3 => status == 1 || status == 2,
        4 => status > 2,
        _ => false,
    }
}
