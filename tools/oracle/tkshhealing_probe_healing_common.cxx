// Printers shared by the ShapeAnalysis, ShapeConstruct and ShapeFix probe operations; the Rust
// twin is crates/tkshhealing/examples/sh_probe/healing_common.rs.
//
// sh_flags prints the ShapeExtend flags of a tool (Status of DONE1..DONE8, then FAIL1..FAIL8),
// which gives every bit of its status field; sh_curve3d and sh_curve2d print a curve (its
// type, and for a non-null one its range and its points at the finite ends and middle).
#include <ShapeExtend_Status.hxx>
template<class T> void sh_flags(const T& tool){
 for(int k=1;k<=8;++k)ga_flag(tool.Status(ShapeExtend_Status(k)));
 for(int k=10;k<=17;++k)ga_flag(tool.Status(ShapeExtend_Status(k)));}
void sh_curve3d(const occ::handle<Geom_Curve>& c,double f,double l){
 tp_type(c);if(c.IsNull())return;ga_real(f);ga_real(l);tp_curve_points(c,f,l);}
void sh_curve2d(const occ::handle<Geom2d_Curve>& c,double f,double l){
 tp_type(c);if(c.IsNull())return;ga_real(f);ga_real(l);tp_curve2d_points(c,f,l);}
void sh_vec2d(const gp_Vec2d& v){ga_real(v.X());ga_real(v.Y());}
