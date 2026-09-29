// Intf probe operations (tkgeomalgo_probe.cxx); the Rust twin is
// crates/tkgeomalgo/examples/ga_probe/intf.rs.
#include <Intf.hxx>
// `p1 p2 p3 point`: the plane equation of the triangle, then whether it contains the point.
void ga_intf_triangle_case(std::istream& input){
 GaReader r{input};gp_Pnt p1=r.pnt(),p2=r.pnt(),p3=r.pnt(),point=r.pnt();
 gp_XYZ normal(-1.,-2.,-3.);double distance=-4.;
 Intf::PlaneEquation(p1,p2,p3,normal,distance);
 ga_xyz(normal);ga_real(distance);ga_flag(Intf::Contain(p1,p2,p3,point));ga_end();
}
