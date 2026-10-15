// BRepFeat sweep probes of tools/oracle/tkfeat_probe.cxx (the prism, drafted prism,
// revolution and pipe features); the Rust twin is crates/tkfeat/examples/ft_probe/
// sweeps.rs.
#include <BRepFeat_MakeDPrism.hxx>
#include <BRepFeat_MakePipe.hxx>
#include <BRepFeat_MakePrism.hxx>
#include <BRepFeat_MakeRevol.hxx>

// A feature after its Perform: done, its status error, then its shape when done.
template<class Feat> void ft_feature(Feat& f){
 ga_flag(f.IsDone());ga_int(f.CurrentStatusError());
 if(f.IsDone())ft_shape(f.Shape());}
// The planar face of the limits of a request: the square [-5, 15]^2 at the height z.
TopoDS_Shape ft_limit(double z){return ft_rect(-5.,-5.,15.,15.,z);}
// The profile of the prisms: the rectangle [2, 6] x [3, 5] on the top of the basis.
TopoDS_Face ft_profile(){return ft_rect(2.,3.,6.,5.,10.);}

// Runs the limit method of a prism-like feature: 0 the length a, 1 up to the face at a,
// 2 from the face at a to the face at b, 3 up to the end, 4 through all, 5 the length b
// towards the face at a, 6 from the end to the face at a, 7 up to the bottom face of the
// basis.
template<class Feat> void ft_limits(Feat& f,const TopoDS_Shape& b,int method,double a,double bb){
 if(method==0)f.Perform(a);
 else if(method==1)f.Perform(ft_limit(a));
 else if(method==2)f.Perform(ft_limit(a),ft_limit(bb));
 else if(method==3)f.PerformUntilEnd();
 else if(method==4)f.PerformThruAll();
 else if(method==5)f.PerformUntilHeight(ft_limit(a),bb);
 else if(method==6)f.PerformFromEnd(ft_limit(a));
 else f.Perform(ft_face_at(b,0.));}

// `ft_prism basis mode up method a b`: the prism of the profile on the top face of the
// basis (0 box, 1 cylinder) along +Z (up) or -Z, fused (1), cut (0) or alone (2).
void ft_prism_case(std::istream& in){
 GaReader r{in};int kind=r.integer(),mode=r.integer();bool up=r.flag();int method=r.integer();double a=r.real(),bb=r.real();
 GA_TRY(
  TopoDS_Shape b=ft_basis(kind);
  BRepFeat_MakePrism f(b,ft_profile(),ft_face_at(b,10.),gp_Dir(0.,0.,up?1.:-1.),mode,true);
  ft_limits(f,b,method,a,bb);ft_feature(f););
 ga_end();}

// `ft_dprism basis mode angle method a b`: the drafted prism of the profile with the
// draft `angle`, as `ft_prism` (method 0 the height a).
void ft_dprism_case(std::istream& in){
 GaReader r{in};int kind=r.integer(),mode=r.integer();double ang=r.real();int method=r.integer();double a=r.real(),bb=r.real();
 GA_TRY(
  TopoDS_Shape b=ft_basis(kind);
  BRepFeat_MakeDPrism f(b,ft_profile(),ft_face_at(b,10.),ang,mode,true);
  ft_limits(f,b,method,a,bb);ft_feature(f););
 ga_end();}

// The walls of the revolutions: the half planes y = 5 beyond x = 5 (0) and before it (1).
TopoDS_Shape ft_wall(int side){
 double x0=side?-5.:5.,x1=side?5.:15.;
 return ft_polygon_face({gp_Pnt(x0,5.,-5.),gp_Pnt(x1,5.,-5.),gp_Pnt(x1,5.,15.),gp_Pnt(x0,5.,15.)});}

// `ft_revol mode method a`: the rectangle x = 5, y in [2, 4], two high on (mode 1, 2) or
// under (mode 0) the top of the box, turned around the vertical axis through (5, 5): 0 by
// the angle a, 1 up to the wall 0, 2 through all, 3 by a towards the wall 0, 4 from the
// wall 1 to the wall 0.
void ft_revol_case(std::istream& in){
 GaReader r{in};int mode=r.integer(),method=r.integer();double a=r.real();
 GA_TRY(
  TopoDS_Shape b=ft_basis(0);double z0=mode?10.:8.,z1=mode?12.:10.;
  TopoDS_Face prof=ft_polygon_face({gp_Pnt(5.,2.,z0),gp_Pnt(5.,4.,z0),gp_Pnt(5.,4.,z1),gp_Pnt(5.,2.,z1)});
  BRepFeat_MakeRevol f(b,prof,ft_face_at(b,10.),gp_Ax1(gp_Pnt(5.,5.,0.),gp_Dir(0.,0.,1.)),mode,true);
  if(method==0)f.Perform(a);
  else if(method==1)f.Perform(ft_wall(0));
  else if(method==2)f.PerformThruAll();
  else if(method==3)f.PerformUntilAngle(ft_wall(0),a);
  else f.Perform(ft_wall(1),ft_wall(0));
  ft_feature(f););
 ga_end();}

// `ft_pipe mode z1 method a`: the square [2, 4]^2 on the top of the box swept along the
// segment from (3, 3, 10) to (3, 3, z1): 0 whole, 1 up to the face at a, 2 from the top
// face to the face at a.
void ft_pipe_case(std::istream& in){
 GaReader r{in};int mode=r.integer();double z1=r.real();int method=r.integer();double a=r.real();
 GA_TRY(
  TopoDS_Shape b=ft_basis(0);TopoDS_Face top=ft_face_at(b,10.);
  TopoDS_Wire spine=BRepBuilderAPI_MakeWire(BRepBuilderAPI_MakeEdge(gp_Pnt(3.,3.,10.),gp_Pnt(3.,3.,z1)).Edge()).Wire();
  BRepFeat_MakePipe f(b,ft_rect(2.,2.,4.,4.,10.),top,spine,mode,true);
  if(method==0)f.Perform();
  else if(method==1)f.Perform(ft_limit(a));
  else f.Perform(top,ft_limit(a));
  ft_feature(f););
 ga_end();}
