// ShapeExtend probe operations; the Rust twin is
// crates/tkshhealing/examples/sh_probe/shape_extend.rs.
#include <ShapeExtend.hxx>
// `flag status`: ShapeExtend::EncodeStatus(status), then DecodeStatus(flag, status).
void sh_status_case(std::istream& input){
 GaReader r{input};int flag=r.integer();ShapeExtend_Status status=ShapeExtend_Status(r.integer());
 ga_int(ShapeExtend::EncodeStatus(status));ga_flag(ShapeExtend::DecodeStatus(flag,status));
 ga_end();}
