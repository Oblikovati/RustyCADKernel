// TKGeomBase oracle: the GeomBndLib and BndLib packages (tkgeombase_probe.cxx). Curves and
// surfaces are read with the LProp builders (tkgeombase_probe_lprop_build.cxx), extended
// here with Bezier and rectangular trimmed surfaces. A box prints IsVoid, IsWhole, its six
// (four) open flags, its gap and the structured Get(). The Rust twin is
// crates/tkgeombase/examples/gb_probe/bndlib.rs.
#include <BndLib.hxx>
#include <BndLib_Add2dCurve.hxx>
#include <BndLib_Add3dCurve.hxx>
#include <BndLib_AddSurface.hxx>
#include <Adaptor2d_OffsetCurve.hxx>
#include <Bnd_Box.hxx>
#include <Bnd_Box2d.hxx>
#include <Geom_BezierSurface.hxx>
#include <Geom_RectangularTrimmedSurface.hxx>
#include <Geom2dAdaptor_Curve.hxx>
#include <GeomAdaptor_Curve.hxx>
#include <GeomAdaptor_Surface.hxx>
#include <GeomAdaptor_TransformedSurface.hxx>
#include <GeomBndLib_Circle.hxx>
#include <GeomBndLib_Circle2d.hxx>
#include <GeomBndLib_Curve.hxx>
#include <GeomBndLib_Curve2d.hxx>
#include <GeomBndLib_Ellipse.hxx>
#include <GeomBndLib_Ellipse2d.hxx>
#include <GeomBndLib_Hyperbola.hxx>
#include <GeomBndLib_Hyperbola2d.hxx>
#include <GeomBndLib_Line.hxx>
#include <GeomBndLib_Line2d.hxx>
#include <GeomBndLib_Parabola.hxx>
#include <GeomBndLib_Parabola2d.hxx>
#include <GeomBndLib_Surface.hxx>
#include <GeomBndLib_OptimizationHelpers.pxx>
#include <GeomBndLib_SamplingHelpers.pxx>
#include <GeomBndLib_SplineHelpers.pxx>
#include <gp_Cone.hxx>
#include <gp_Cylinder.hxx>
#include <gp_Sphere.hxx>
#include <gp_Torus.hxx>
void gb_bnd_box(const Bnd_Box& b){
 gb_flag(b.IsVoid());gb_flag(b.IsWhole());
 gb_flag(b.IsOpenXmin());gb_flag(b.IsOpenXmax());gb_flag(b.IsOpenYmin());
 gb_flag(b.IsOpenYmax());gb_flag(b.IsOpenZmin());gb_flag(b.IsOpenZmax());
 gb_real(b.GetGap());const auto [x,X,y,Y,z,Z]=b.Get();
 gb_real(x);gb_real(X);gb_real(y);gb_real(Y);gb_real(z);gb_real(Z);
}
void gb_bnd_box2d(const Bnd_Box2d& b){
 gb_flag(b.IsVoid());gb_flag(b.IsWhole());
 gb_flag(b.IsOpenXmin());gb_flag(b.IsOpenXmax());gb_flag(b.IsOpenYmin());gb_flag(b.IsOpenYmax());
 gb_real(b.GetGap());const auto [x,X,y,Y]=b.Get();gb_real(x);gb_real(X);gb_real(y);gb_real(Y);
}
// Surface specs: 0 an LProp surface, 1 a rational Bezier surface (nu, nv, pole+weight),
// 2 a rectangular trimmed surface (u1, u2, v1, v2, then a surface spec).
occ::handle<Geom_Surface> gb_bnd_surface(GbReader& r){
 const int code=r.integer();
 if(code==0)return gb_lp_surface(r);
 if(code==1){
  const int nu=r.integer(),nv=r.integer();NCollection_Array2<gp_Pnt> p(1,nu,1,nv);NCollection_Array2<double> w(1,nu,1,nv);
  for(int i=1;i<=nu;++i)for(int j=1;j<=nv;++j){p(i,j)=r.pnt();w(i,j)=r.real();}
  return new Geom_BezierSurface(p,w);
 }
 double u1=r.real(),u2=r.real(),v1=r.real(),v2=r.real();
 occ::handle<Geom_Surface> s=gb_bnd_surface(r);return new Geom_RectangularTrimmedSurface(s,u1,u2,v1,v2);
}
// The four box members of an evaluator: 0 Box(tol), 1 Box(range, tol), 2 BoxOptimal(tol),
// 3 BoxOptimal(range, tol).
template<class E,class P> void gb_bnd_curve_op(const E& e,int op,double u1,double u2,double tol,P print){
 if(op==0)print(e.Box(tol));else if(op==1)print(e.Box(u1,u2,tol));
 else if(op==2)print(e.BoxOptimal(tol));else print(e.BoxOptimal(u1,u2,tol));
}
// bnd_curve3d: mode = 4*source + op; source 0 the handle, 1 GeomAdaptor_Curve(C),
// 2 GeomAdaptor_Curve(C, U1, U2), 3 BndLib_Add3dCurve on a box holding [-1, 1]^3.
void gb_bnd_curve3d_case(std::istream& in){
 GbReader r{in};const int mode=r.integer();const double u1=r.real(),u2=r.real(),tol=r.real();
 occ::handle<Geom_Curve> c;try{c=gb_lp_curve3d(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<'\n';return;}
 const int op=mode%4,src=mode/4;
 GB_TRY(
  if(src==0){GeomBndLib_Curve e(c);gb_int(e.GetType());gb_bnd_curve_op(e,op,u1,u2,tol,gb_bnd_box);}
  else if(src==1){GeomAdaptor_Curve a(c);GeomBndLib_Curve e(a);gb_int(e.GetType());gb_bnd_curve_op(e,op,u1,u2,tol,gb_bnd_box);}
  else if(src==2){GeomAdaptor_Curve a(c,u1,u2);GeomBndLib_Curve e(a);gb_int(e.GetType());gb_bnd_curve_op(e,op,u1,u2,tol,gb_bnd_box);}
  else{GeomAdaptor_Curve a(c);Bnd_Box b;b.Update(-1.,-1.,-1.,1.,1.,1.);
   if(op==0)BndLib_Add3dCurve::Add(a,tol,b);else if(op==1)BndLib_Add3dCurve::Add(a,u1,u2,tol,b);
   else if(op==2)BndLib_Add3dCurve::AddOptimal(a,tol,b);else BndLib_Add3dCurve::AddOptimal(a,u1,u2,tol,b);
   gb_bnd_box(b);})
 gb_end();
}
// bnd_curve2d: mode = 4*source + op; sources 0-2 as bnd_curve3d, 3 BndLib_Add2dCurve on the
// handle (op 0 Add(tol), 1 Add(range), 2 AddOptimal(range)), 4 an Adaptor2d_OffsetCurve of
// offset 0.5 (op 0 BndLib Add(tol), 1 BndLib Add(range), 2 Curve2d Box(tol), 3 Curve2d
// BoxOptimal(range)), 5 BndLib_Add2dCurve on a Geom2dAdaptor_Curve (op 0 Add(tol), 1
// Add(range)); every BndLib box starts as [-1, 1]^2.
void gb_bnd_curve2d_case(std::istream& in){
 GbReader r{in};const int mode=r.integer();const double u1=r.real(),u2=r.real(),tol=r.real();
 occ::handle<Geom2d_Curve> c;try{c=gb_lp_curve2d(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<'\n';return;}
 const int op=mode%4,src=mode/4;
 GB_TRY(
  Bnd_Box2d b;b.Update(-1.,-1.,1.,1.);
  if(src==0){GeomBndLib_Curve2d e(c);gb_int(e.GetType());gb_bnd_curve_op(e,op,u1,u2,tol,gb_bnd_box2d);}
  else if(src==1){Geom2dAdaptor_Curve a(c);GeomBndLib_Curve2d e(a);gb_int(e.GetType());gb_bnd_curve_op(e,op,u1,u2,tol,gb_bnd_box2d);}
  else if(src==2){Geom2dAdaptor_Curve a(c,u1,u2);GeomBndLib_Curve2d e(a);gb_int(e.GetType());gb_bnd_curve_op(e,op,u1,u2,tol,gb_bnd_box2d);}
  else if(src==3){
   if(op==0)BndLib_Add2dCurve::Add(c,tol,b);else if(op==1)BndLib_Add2dCurve::Add(c,u1,u2,tol,b);
   else BndLib_Add2dCurve::AddOptimal(c,u1,u2,tol,b);
   gb_bnd_box2d(b);}
  else if(src==4){occ::handle<Adaptor2d_Curve2d> g=new Geom2dAdaptor_Curve(c);Adaptor2d_OffsetCurve a(g,0.5);
   if(op==0){BndLib_Add2dCurve::Add(a,tol,b);gb_bnd_box2d(b);}
   else if(op==1){BndLib_Add2dCurve::Add(a,u1,u2,tol,b);gb_bnd_box2d(b);}
   else{GeomBndLib_Curve2d e(a);gb_int(e.GetType());if(op==2)gb_bnd_box2d(e.Box(tol));else gb_bnd_box2d(e.BoxOptimal(u1,u2,tol));}}
  else{Geom2dAdaptor_Curve a(c);
   if(op==0)BndLib_Add2dCurve::Add(a,tol,b);else BndLib_Add2dCurve::Add(a,u1,u2,tol,b);
   gb_bnd_box2d(b);})
 gb_end();
}
// bnd_surface: mode = 4*source + op; source 0 the handle, 1 GeomAdaptor_Surface(S),
// 2 GeomAdaptor_Surface(S, U1, U2, V1, V2), 3 GeomAdaptor_TransformedSurface(S, T) with T
// a rotation about Ax1 (read after the patch) then a translation, 4 BndLib_AddSurface on
// GeomAdaptor_Surface(S) and a box holding [-1, 1]^3.
void gb_bnd_surface_case(std::istream& in){
 GbReader r{in};const int mode=r.integer();const double u1=r.real(),u2=r.real(),v1=r.real(),v2=r.real(),tol=r.real();
 const gp_Ax1 ax=r.ax1();const double ang=r.real();const gp_Vec tr=r.vec();
 occ::handle<Geom_Surface> s;try{s=gb_bnd_surface(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<'\n';return;}
 const int op=mode%4,src=mode/4;
 auto run=[&](const GeomBndLib_Surface& e){gb_int(e.GetType());
  if(op==0)gb_bnd_box(e.Box(tol));else if(op==1)gb_bnd_box(e.Box(u1,u2,v1,v2,tol));
  else if(op==2)gb_bnd_box(e.BoxOptimal(tol));else gb_bnd_box(e.BoxOptimal(u1,u2,v1,v2,tol));};
 GB_TRY(
  if(src==0){GeomBndLib_Surface e(s);run(e);}
  else if(src==1){GeomAdaptor_Surface a(s);GeomBndLib_Surface e(a);run(e);}
  else if(src==2){GeomAdaptor_Surface a(s,u1,u2,v1,v2);GeomBndLib_Surface e(a);run(e);}
  else if(src==3){gp_Trsf t;t.SetRotation(ax,ang);t.SetTranslationPart(tr);GeomAdaptor_TransformedSurface a(s,t);GeomBndLib_Surface e(a);run(e);}
  else{GeomAdaptor_Surface a(s);Bnd_Box b;b.Update(-1.,-1.,-1.,1.,1.,1.);
   if(op==0)BndLib_AddSurface::Add(a,tol,b);else if(op==1)BndLib_AddSurface::Add(a,u1,u2,v1,v2,tol,b);
   else if(op==2)BndLib_AddSurface::AddOptimal(a,tol,b);else BndLib_AddSurface::AddOptimal(a,u1,u2,v1,v2,tol,b);
   gb_bnd_box(b);})
 gb_end();
}
// bnd_gp: lib 0 BndLib::Add on a box prefilled with `npre` points, lib 1 the GeomBndLib
// static gp overload. Kinds: 0 lin, 1 lin2d, 2 circ, 3 circ arc, 4 circ2d, 5 circ2d arc,
// 6 elips, 7 elips arc, 8 elips2d, 9 elips2d arc, 10 parab, 11 parab2d, 12 hypr, 13 hypr2d
// (BndLib and GeomBndLib); 14 cylinder patch, 15 cylinder, 16 cone patch, 17 cone,
// 18 sphere, 19 sphere patch, 20 torus, 21 torus patch (BndLib only).
void gb_bnd_gp_case(std::istream& in){
 GbReader r{in};const int lib=r.integer(),kind=r.integer(),npre=r.integer();
 const bool flat=(kind==1||kind==4||kind==5||kind==8||kind==9||kind==11||kind==13);
 Bnd_Box b;Bnd_Box2d b2;
 for(int i=0;i<npre;++i){if(flat)b2.Add(r.pnt2d());else b.Add(r.pnt());}
 GB_TRY(
  if(kind==0){gp_Pnt p=r.pnt();gp_Lin l(p,r.dir());double p1=r.real(),p2=r.real(),t=r.real();
   if(lib==0)BndLib::Add(l,p1,p2,t,b);else b=GeomBndLib_Line::Box(l,p1,p2,t);}
  else if(kind==1){gp_Pnt2d p=r.pnt2d();gp_Lin2d l(p,r.dir2d());double p1=r.real(),p2=r.real(),t=r.real();
   if(lib==0)BndLib::Add(l,p1,p2,t,b2);else b2=GeomBndLib_Line2d::Box(l,p1,p2,t);}
  else if(kind==2||kind==3){gp_Ax2 a=r.ax2();gp_Circ c(a,r.real());double p1=0,p2=0;if(kind==3){p1=r.real();p2=r.real();}double t=r.real();
   if(lib==0){if(kind==2)BndLib::Add(c,t,b);else BndLib::Add(c,p1,p2,t,b);}
   else b=kind==2?GeomBndLib_Circle::Box(c,t):GeomBndLib_Circle::Box(c,p1,p2,t);}
  else if(kind==4||kind==5){gp_Ax2d a=r.ax2d();bool s=r.flag();gp_Circ2d c(a,r.real(),s);double p1=0,p2=0;if(kind==5){p1=r.real();p2=r.real();}double t=r.real();
   if(lib==0){if(kind==4)BndLib::Add(c,t,b2);else BndLib::Add(c,p1,p2,t,b2);}
   else b2=kind==4?GeomBndLib_Circle2d::Box(c,t):GeomBndLib_Circle2d::Box(c,p1,p2,t);}
  else if(kind==6||kind==7){gp_Ax2 a=r.ax2();double ra=r.real();gp_Elips c(a,ra,r.real());double p1=0,p2=0;if(kind==7){p1=r.real();p2=r.real();}double t=r.real();
   if(lib==0){if(kind==6)BndLib::Add(c,t,b);else BndLib::Add(c,p1,p2,t,b);}
   else b=kind==6?GeomBndLib_Ellipse::Box(c,t):GeomBndLib_Ellipse::Box(c,p1,p2,t);}
  else if(kind==8||kind==9){gp_Ax2d a=r.ax2d();bool s=r.flag();double ra=r.real();gp_Elips2d c(a,ra,r.real(),s);double p1=0,p2=0;if(kind==9){p1=r.real();p2=r.real();}double t=r.real();
   if(lib==0){if(kind==8)BndLib::Add(c,t,b2);else BndLib::Add(c,p1,p2,t,b2);}
   else b2=kind==8?GeomBndLib_Ellipse2d::Box(c,t):GeomBndLib_Ellipse2d::Box(c,p1,p2,t);}
  else if(kind==10){gp_Ax2 a=r.ax2();gp_Parab c(a,r.real());double p1=r.real(),p2=r.real(),t=r.real();
   if(lib==0)BndLib::Add(c,p1,p2,t,b);else b=GeomBndLib_Parabola::Box(c,p1,p2,t);}
  else if(kind==11){gp_Ax2d a=r.ax2d();bool s=r.flag();gp_Parab2d c(a,r.real(),s);double p1=r.real(),p2=r.real(),t=r.real();
   if(lib==0)BndLib::Add(c,p1,p2,t,b2);else b2=GeomBndLib_Parabola2d::Box(c,p1,p2,t);}
  else if(kind==12){gp_Ax2 a=r.ax2();double ra=r.real();gp_Hypr c(a,ra,r.real());double p1=r.real(),p2=r.real(),t=r.real();
   if(lib==0)BndLib::Add(c,p1,p2,t,b);else b=GeomBndLib_Hyperbola::Box(c,p1,p2,t);}
  else if(kind==13){gp_Ax2d a=r.ax2d();bool s=r.flag();double ra=r.real();gp_Hypr2d c(a,ra,r.real(),s);double p1=r.real(),p2=r.real(),t=r.real();
   if(lib==0)BndLib::Add(c,p1,p2,t,b2);else b2=GeomBndLib_Hyperbola2d::Box(c,p1,p2,t);}
  else if(kind==14||kind==15){gp_Ax3 a=r.ax3();gp_Cylinder c(a,r.real());double u1=0,u2=0;if(kind==14){u1=r.real();u2=r.real();}double v1=r.real(),v2=r.real(),t=r.real();
   if(kind==14)BndLib::Add(c,u1,u2,v1,v2,t,b);else BndLib::Add(c,v1,v2,t,b);}
  else if(kind==16||kind==17){gp_Ax3 a=r.ax3();double ang=r.real();gp_Cone c(a,ang,r.real());double u1=0,u2=0;if(kind==16){u1=r.real();u2=r.real();}double v1=r.real(),v2=r.real(),t=r.real();
   if(kind==16)BndLib::Add(c,u1,u2,v1,v2,t,b);else BndLib::Add(c,v1,v2,t,b);}
  else if(kind==18||kind==19){gp_Ax3 a=r.ax3();gp_Sphere c(a,r.real());
   if(kind==18)BndLib::Add(c,r.real(),b);else{double u1=r.real(),u2=r.real(),v1=r.real(),v2=r.real();BndLib::Add(c,u1,u2,v1,v2,r.real(),b);}}
  else{gp_Ax3 a=r.ax3();double ra=r.real();gp_Torus c(a,ra,r.real());
   if(kind==20)BndLib::Add(c,r.real(),b);else{double u1=r.real(),u2=r.real(),v1=r.real(),v2=r.real();BndLib::Add(c,u1,u2,v1,v2,r.real(),b);}}
  if(flat)gb_bnd_box2d(b2);else gb_bnd_box(b);)
 gb_end();
}
// bnd_helpers: the .pxx helpers the upstream tests reach. mode 0 ComputeNbSamples,
// 1 AdjustExtrCurve (min then max of coordinate `index`), 2 CurveBoxOptimal, 3 FillBox
// with `n` sample pairs (3D curve spec); 4 ComputeNbSamples2d, 5 AdjustExtrCurve2d,
// 6 CurveBoxOptimal (2D curve spec); 7 ComputeNbUSamples/VSamples ranged and full,
// 8 AdjustExtrSurf (min then max), with a surface spec.
void gb_bnd_helpers_case(std::istream& in){
 GbReader r{in};const int mode=r.integer();const double u1=r.real(),u2=r.real(),v1=r.real(),v2=r.real();
 const double extr=r.real(),tol=r.real();const int index=r.integer(),n=r.integer();
 GB_TRY(
  if(mode<=3){GeomAdaptor_Curve a(gb_lp_curve3d(r));
   if(mode==0)gb_int(GeomBndLib_SamplingHelpers::ComputeNbSamples(a,u1,u2));
   else if(mode==1){gb_real(GeomBndLib_OptimizationHelpers::AdjustExtrCurve(a,u1,u2,extr,index,tol,true));
    gb_real(GeomBndLib_OptimizationHelpers::AdjustExtrCurve(a,u1,u2,extr,index,tol,false));}
   else if(mode==2)gb_bnd_box(GeomBndLib_SplineHelpers::CurveBoxOptimal<GeomAdaptor_Curve,Bnd_Box,gp_Pnt>(a,u1,u2,tol));
   else{Bnd_Box b;gb_real(GeomBndLib_SplineHelpers::FillBox<Bnd_Box,GeomAdaptor_Curve,gp_Pnt>(b,a,u1,u2,n));gb_bnd_box(b);}}
  else if(mode<=6){Geom2dAdaptor_Curve a(gb_lp_curve2d(r));
   if(mode==4)gb_int(GeomBndLib_SamplingHelpers::ComputeNbSamples2d(a,u1,u2));
   else if(mode==5){gb_real(GeomBndLib_OptimizationHelpers::AdjustExtrCurve2d(a,u1,u2,extr,index,tol,true));
    gb_real(GeomBndLib_OptimizationHelpers::AdjustExtrCurve2d(a,u1,u2,extr,index,tol,false));}
   else gb_bnd_box2d(GeomBndLib_SplineHelpers::CurveBoxOptimal<Geom2dAdaptor_Curve,Bnd_Box2d,gp_Pnt2d>(a,u1,u2,tol));}
  else{GeomAdaptor_Surface a(gb_bnd_surface(r));
   if(mode==7){gb_int(GeomBndLib_SamplingHelpers::ComputeNbUSamples(a,u1,u2));gb_int(GeomBndLib_SamplingHelpers::ComputeNbVSamples(a,v1,v2));
    gb_int(GeomBndLib_SamplingHelpers::ComputeNbUSamples(a));gb_int(GeomBndLib_SamplingHelpers::ComputeNbVSamples(a));}
   else{gb_real(GeomBndLib_OptimizationHelpers::AdjustExtrSurf(a,u1,u2,v1,v2,extr,index,tol,true));
    gb_real(GeomBndLib_OptimizationHelpers::AdjustExtrSurf(a,u1,u2,v1,v2,extr,index,tol,false));}})
 gb_end();
}
