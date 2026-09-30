//! Exact CheckOrder control flow, including native both-mode fallback clearing.
use super::*;
use rustycad_tkbrep::topods::{self, Edge};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{Xy, Xyz};
#[derive(Default)]
struct OrderingEndpoints {
    first3: Xyz,
    last3: Xyz,
    first2: Xy,
    last2: Xy,
}
impl ShapeAnalysisWire {
    /// Public ordering check that also stores StatusOrder; `analyzer.check_order(true, false)?`.
    pub fn check_order(&self, closed: bool, mode3d: bool) -> Result<bool, Failure> {
        self.check_order_with(&mut ShapeAnalysisWireOrder::new(), closed, mode3d, false)?;
        let mut fields = self.fields.write();
        fields.status_order = fields.status;
        Ok(ShapeExtend::decode_status(
            fields.status_order,
            ShapeExtendStatus::Done,
        ))
    }
    /// Populate the supplied order tool and set LastCheckStatus; `analyzer.check_order_with(&mut order, true, false, false)?`.
    pub fn check_order_with(
        &self,
        order: &mut ShapeAnalysisWireOrder,
        closed: bool,
        mode3d: bool,
        both: bool,
    ) -> Result<bool, Failure> {
        if (!mode3d || both) && self.face().is_null() {
            self.set_last_status(ShapeExtendStatus::Fail2);
            return Ok(false);
        }
        self.set_last_status(ShapeExtendStatus::Ok);
        order.set_mode(mode3d, 0., both);
        let Some(all2d) = self.collect_order_endpoints(order, mode3d, both)? else {
            return Ok(false);
        };
        // OCCT changes mode without repopulating points, so failed mixed UV input
        // deliberately leaves an empty 3D ordering tool (3d097a0328e71b8).
        if both && !all2d {
            order.set_mode(true, 0., false);
        }
        order.perform(closed)?;
        self.store_order_status(order.status());
        Ok(self.last_check_status(ShapeExtendStatus::Done))
    }
    fn collect_order_endpoints(
        &self,
        order: &mut ShapeAnalysisWireOrder,
        mode3d: bool,
        both: bool,
    ) -> Result<Option<bool>, Failure> {
        let wire = self.wire_data();
        let mut all2d = true;
        for index in 1..=wire.nb_edges() {
            let edge = wire.edge(index)?;
            let mut endpoints = OrderingEndpoints::default();
            if (mode3d || both) && !Self::order_vertices(&edge, &mut endpoints)? {
                self.set_last_status(ShapeExtendStatus::Fail2);
                return Ok(None);
            }
            if (!mode3d || both) && !self.order_pcurve(&edge, &mut endpoints)? {
                if !mode3d && !both {
                    self.set_last_status(ShapeExtendStatus::Fail2);
                    return Ok(None);
                }
                all2d = false;
            }
            endpoints.append(order, mode3d, both);
        }
        Ok(Some(all2d))
    }
    fn order_vertices(edge: &Edge, endpoints: &mut OrderingEndpoints) -> Result<bool, Failure> {
        let analysis = ShapeAnalysisEdge::new();
        let first = analysis.first_vertex(edge);
        let last = analysis.last_vertex(edge);
        if first.is_null() || last.is_null() {
            return Ok(false);
        }
        endpoints.first3 = BRepTool::pnt(&first)?.xyz();
        endpoints.last3 = BRepTool::pnt(&last)?.xyz();
        Ok(true)
    }
    fn order_pcurve(
        &self,
        edge: &Edge,
        endpoints: &mut OrderingEndpoints,
    ) -> Result<bool, Failure> {
        let face = topods::face(&self.face().oriented(Orientation::Forward))?;
        let (curve, first, last) = ShapeAnalysisEdge::new().pcurve_on_face(edge, &face, true)?;
        let Some(curve) = curve.get() else {
            return Ok(false);
        };
        endpoints.first2 = curve.value(first)?.xy();
        endpoints.last2 = curve.value(last)?.xy();
        Ok(true)
    }
    fn store_order_status(&self, status: i32) {
        let status = match status {
            0 => ShapeExtendStatus::Ok,
            1 => ShapeExtendStatus::Done1,
            2 => ShapeExtendStatus::Done2,
            -1 => ShapeExtendStatus::Done3,
            -2 => ShapeExtendStatus::Done4,
            3 => ShapeExtendStatus::Done5,
            -10 => ShapeExtendStatus::Fail1,
            _ => return,
        };
        self.set_last_status(status);
    }
}
impl OrderingEndpoints {
    fn append(self, order: &mut ShapeAnalysisWireOrder, mode3d: bool, both: bool) {
        if both {
            order.add_both(self.first3, self.last3, self.first2, self.last2);
            return;
        }
        if mode3d {
            order.add_3d(self.first3, self.last3);
        } else {
            order.add_2d(self.first2, self.last2);
        }
    }
}
