// TKGeomBase oracle: the elementary closed forms of the Extrema package (ExtPElC,
// ExtPElC2d, ExtElC, ExtElC2d, ExtPElS, ExtElCS, ExtElSS). Twin:
// crates/tkgeombase/examples/gb_probe/extrema_el.rs.
//
// A 3D conic is `kind` then 0 line P D | 1 circle Ax2 R | 2 ellipse Ax2 A B |
// 3 hyperbola Ax2 A B | 4 parabola Ax2 F; a 2D conic the same with Ax22d frames
// (P X Y) and a 2D line P D. A surface is 0 plane Ax3 | 1 cylinder Ax3 R |
// 2 cone Ax3 Ang R | 3 sphere Ax3 R | 4 torus Ax3 R r.
#include <Extrema_ExtElC.hxx>
#include <Extrema_ExtElC2d.hxx>
#include <Extrema_ExtElCS.hxx>
#include <Extrema_ExtElSS.hxx>
#include <Extrema_ExtPElC.hxx>
#include <Extrema_ExtPElC2d.hxx>
#include <Extrema_ExtPElS.hxx>
#include <Extrema_POnCurv.hxx>
#include <Extrema_POnCurv2d.hxx>
#include <Extrema_POnSurf.hxx>
#include <Extrema_POnSurfParams.hxx>
#include <gp_Sphere.hxx>
#include <gp_Torus.hxx>
void gb_poncurv(const Extrema_POnCurv& p){gb_real(p.Parameter());gb_pnt(p.Value());}
void gb_poncurv2d(const Extrema_POnCurv2d& p){gb_real(p.Parameter());gb_pnt2d(p.Value());}
void gb_ponsurf(const Extrema_POnSurf& p){double u,v;p.Parameter(u,v);gb_real(u);gb_real(v);gb_pnt(p.Value());}
// The result of an ExtPElC-like class (IsDone, NbExt, each extremum, bad indexes).
template<class E,class F> void gb_pelc_result(const E& e,F point){
 gb_flag(e.IsDone());
 GB_TRY(int n=e.NbExt();gb_int(n);for(int i=1;i<=n;++i){gb_real(e.SquareDistance(i));gb_flag(e.IsMin(i));point(e.Point(i));});
 GB_TRY(gb_real(e.SquareDistance(0)));GB_TRY(gb_flag(e.IsMin(5)));
}
// The result of an ExtElC-like class.
template<class E,class P,class F> void gb_elc_result(const E& e,F point){
 gb_flag(e.IsDone());GB_TRY(gb_flag(e.IsParallel()));
 GB_TRY(int n=e.NbExt();gb_int(n);for(int i=1;i<=n;++i){gb_real(e.SquareDistance(i));P a,b;e.Points(i,a,b);point(a);point(b);});
 GB_TRY(gb_real(e.SquareDistance(0)));GB_TRY(gb_real(e.SquareDistance(9)));
}
// ex_pelc: kind conic nq (x y z tol uinf usup)*: constructor, then Perform on one
// reused object.
void gb_ex_pelc_case(std::istream& in){
 GbReader r{in};
 try{
  int kind=r.integer();gp_Lin l;gp_Circ c;gp_Elips e;gp_Hypr h;gp_Parab p;
  if(kind==0){gp_Pnt o=r.pnt();l=gp_Lin(o,r.dir());}
  else{gp_Ax2 a=r.ax2();double x=r.real();
   if(kind==1)c=gp_Circ(a,x);if(kind==4)p=gp_Parab(a,x);
   if(kind==2||kind==3){double y=r.real();if(kind==2)e=gp_Elips(a,x,y);else h=gp_Hypr(a,x,y);}}
  int n=r.integer();Extrema_ExtPElC reused;gb_pelc_result(reused,gb_poncurv);
  for(int q=0;q<n;++q){gp_Pnt pt=r.pnt();double tol=r.real(),u1=r.real(),u2=r.real();
   GB_TRY(if(kind==0){Extrema_ExtPElC x(pt,l,tol,u1,u2);gb_pelc_result(x,gb_poncurv);reused.Perform(pt,l,tol,u1,u2);}
    if(kind==1){Extrema_ExtPElC x(pt,c,tol,u1,u2);gb_pelc_result(x,gb_poncurv);reused.Perform(pt,c,tol,u1,u2);}
    if(kind==2){Extrema_ExtPElC x(pt,e,tol,u1,u2);gb_pelc_result(x,gb_poncurv);reused.Perform(pt,e,tol,u1,u2);}
    if(kind==3){Extrema_ExtPElC x(pt,h,tol,u1,u2);gb_pelc_result(x,gb_poncurv);reused.Perform(pt,h,tol,u1,u2);}
    if(kind==4){Extrema_ExtPElC x(pt,p,tol,u1,u2);gb_pelc_result(x,gb_poncurv);reused.Perform(pt,p,tol,u1,u2);});
   gb_pelc_result(reused,gb_poncurv);}
 }catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
 gb_end();
}
// ex_pelc2d: kind conic2d nq (x y tol uinf usup)*.
void gb_ex_pelc2d_case(std::istream& in){
 GbReader r{in};
 try{
  int kind=r.integer();gp_Lin2d l;gp_Circ2d c;gp_Elips2d e;gp_Hypr2d h;gp_Parab2d p;
  if(kind==0){gp_Pnt2d o=r.pnt2d();l=gp_Lin2d(o,r.dir2d());}
  else{gp_Ax22d a=r.ax22d();double x=r.real();
   if(kind==1)c=gp_Circ2d(a,x);if(kind==4)p=gp_Parab2d(a,x);
   if(kind==2||kind==3){double y=r.real();if(kind==2)e=gp_Elips2d(a,x,y);else h=gp_Hypr2d(a,x,y);}}
  int n=r.integer();Extrema_ExtPElC2d reused;gb_pelc_result(reused,gb_poncurv2d);
  for(int q=0;q<n;++q){gp_Pnt2d pt=r.pnt2d();double tol=r.real(),u1=r.real(),u2=r.real();
   GB_TRY(if(kind==0){Extrema_ExtPElC2d x(pt,l,tol,u1,u2);gb_pelc_result(x,gb_poncurv2d);reused.Perform(pt,l,tol,u1,u2);}
    if(kind==1){Extrema_ExtPElC2d x(pt,c,tol,u1,u2);gb_pelc_result(x,gb_poncurv2d);reused.Perform(pt,c,tol,u1,u2);}
    if(kind==2){Extrema_ExtPElC2d x(pt,e,tol,u1,u2);gb_pelc_result(x,gb_poncurv2d);reused.Perform(pt,e,tol,u1,u2);}
    if(kind==3){Extrema_ExtPElC2d x(pt,h,tol,u1,u2);gb_pelc_result(x,gb_poncurv2d);reused.Perform(pt,h,tol,u1,u2);}
    if(kind==4){Extrema_ExtPElC2d x(pt,p,tol,u1,u2);gb_pelc_result(x,gb_poncurv2d);reused.Perform(pt,p,tol,u1,u2);});
   gb_pelc_result(reused,gb_poncurv2d);}
 }catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
 gb_end();
}
gp_Circ gb_ex_circ(GbReader& r){gp_Ax2 a=r.ax2();return gp_Circ(a,r.real());}
gp_Lin gb_ex_lin(GbReader& r){gp_Pnt o=r.pnt();return gp_Lin(o,r.dir());}
gp_Circ2d gb_ex_circ2d(GbReader& r){gp_Ax22d a=r.ax22d();return gp_Circ2d(a,r.real());}
gp_Lin2d gb_ex_lin2d(GbReader& r){gp_Pnt2d o=r.pnt2d();return gp_Lin2d(o,r.dir2d());}
// ex_elc: pair <first> <second> [tol]: 0 lin-lin, 1 lin-circ, 2 lin-elips, 3 lin-hypr,
// 4 lin-parab, 5 circ-circ.
void gb_ex_elc_case(std::istream& in){
 GbReader r{in};
 try{
  int pair=r.integer();
  if(pair==5){gp_Circ a=gb_ex_circ(r);gp_Circ b=gb_ex_circ(r);Extrema_ExtElC x(a,b);gb_elc_result<Extrema_ExtElC,Extrema_POnCurv>(x,gb_poncurv);}
  else{gp_Lin l=gb_ex_lin(r);gp_Ax2 a=r.ax2();double x1=r.real();
   if(pair==0){gp_Lin m(a.Location(),a.Direction());Extrema_ExtElC x(l,m,x1);gb_elc_result<Extrema_ExtElC,Extrema_POnCurv>(x,gb_poncurv);}
   if(pair==1){double tol=r.real();Extrema_ExtElC x(l,gp_Circ(a,x1),tol);gb_elc_result<Extrema_ExtElC,Extrema_POnCurv>(x,gb_poncurv);}
   if(pair==2){double y=r.real();Extrema_ExtElC x(l,gp_Elips(a,x1,y));gb_elc_result<Extrema_ExtElC,Extrema_POnCurv>(x,gb_poncurv);}
   if(pair==3){double y=r.real();Extrema_ExtElC x(l,gp_Hypr(a,x1,y));gb_elc_result<Extrema_ExtElC,Extrema_POnCurv>(x,gb_poncurv);}
   if(pair==4){Extrema_ExtElC x(l,gp_Parab(a,x1));gb_elc_result<Extrema_ExtElC,Extrema_POnCurv>(x,gb_poncurv);}}
 }catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
 gb_end();
}
// ex_elc2d: pair: 0 lin-lin (lin lin tol), 1 lin-circ, 2 lin-elips, 3 lin-hypr,
// 4 lin-parab (lin Ax22d x [y] [tol]), 5 circ-circ, 6 circ-elips, 7 circ-hypr,
// 8 circ-parab (circ Ax22d x [y]).
void gb_ex_elc2d_case(std::istream& in){
 GbReader r{in};
 auto show=[](const Extrema_ExtElC2d& x){gb_elc_result<Extrema_ExtElC2d,Extrema_POnCurv2d>(x,gb_poncurv2d);};
 try{
  int pair=r.integer();
  if(pair==0){gp_Lin2d a=gb_ex_lin2d(r);gp_Lin2d b=gb_ex_lin2d(r);show(Extrema_ExtElC2d(a,b,r.real()));}
  else if(pair<=4){gp_Lin2d l=gb_ex_lin2d(r);gp_Ax22d a=r.ax22d();double x1=r.real();
   if(pair==1)show(Extrema_ExtElC2d(l,gp_Circ2d(a,x1),r.real()));
   if(pair==2){double y=r.real();show(Extrema_ExtElC2d(l,gp_Elips2d(a,x1,y)));}
   if(pair==3){double y=r.real();show(Extrema_ExtElC2d(l,gp_Hypr2d(a,x1,y)));}
   if(pair==4)show(Extrema_ExtElC2d(l,gp_Parab2d(a,x1)));}
  else{gp_Circ2d c=gb_ex_circ2d(r);gp_Ax22d a=r.ax22d();double x1=r.real();
   if(pair==5)show(Extrema_ExtElC2d(c,gp_Circ2d(a,x1)));
   if(pair==6){double y=r.real();show(Extrema_ExtElC2d(c,gp_Elips2d(a,x1,y)));}
   if(pair==7){double y=r.real();show(Extrema_ExtElC2d(c,gp_Hypr2d(a,x1,y)));}
   if(pair==8)show(Extrema_ExtElC2d(c,gp_Parab2d(a,x1)));}
 }catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
 gb_end();
}
// ex_pels: kind surface nq (x y z tol)*: constructor, then Perform on one object.
void gb_ex_pels_case(std::istream& in){
 GbReader r{in};
 try{
  int kind=r.integer();gp_Ax3 a=r.ax3();gp_Pln pl(a);gp_Cylinder cy;gp_Cone co;gp_Sphere sp;gp_Torus to;
  if(kind==1)cy=gp_Cylinder(a,r.real());
  if(kind==2){double ang=r.real();co=gp_Cone(a,ang,r.real());}
  if(kind==3)sp=gp_Sphere(a,r.real());
  if(kind==4){double big=r.real();to=gp_Torus(a,big,r.real());}
  auto show=[](const Extrema_ExtPElS& e){gb_flag(e.IsDone());
   GB_TRY(int n=e.NbExt();gb_int(n);for(int i=1;i<=n;++i){gb_real(e.SquareDistance(i));gb_ponsurf(e.Point(i));});
   GB_TRY(gb_real(e.SquareDistance(0)));GB_TRY(gb_ponsurf(e.Point(5)));};
  int n=r.integer();Extrema_ExtPElS reused;show(reused);
  for(int q=0;q<n;++q){gp_Pnt pt=r.pnt();double tol=r.real();
   GB_TRY(if(kind==0){show(Extrema_ExtPElS(pt,pl,tol));reused.Perform(pt,pl,tol);}
    if(kind==1){show(Extrema_ExtPElS(pt,cy,tol));reused.Perform(pt,cy,tol);}
    if(kind==2){show(Extrema_ExtPElS(pt,co,tol));reused.Perform(pt,co,tol);}
    if(kind==3){show(Extrema_ExtPElS(pt,sp,tol));reused.Perform(pt,sp,tol);}
    if(kind==4){show(Extrema_ExtPElS(pt,to,tol));reused.Perform(pt,to,tol);});
   show(reused);}
 }catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
 gb_end();
}
// ex_elcs: n steps of `kind curve surface` performed on one object: kind is
// curve*5+surface with curve 0 line, 1 circle, 2 hyperbola (surface plane only).
// After each step the result, and Points(1) of a parallel result once an earlier
// step allocated the point arrays (the stale handles).
void gb_ex_elcs_case(std::istream& in){
 GbReader r{in};
 try{
  int n=r.integer();Extrema_ExtElCS e;bool allocated=false;
  for(int q=0;q<n;++q){int kind=r.integer();int curve=kind/5,surf=kind%5;
   gp_Lin l;gp_Circ c;gp_Hypr h;
   if(curve==0)l=gb_ex_lin(r);
   if(curve==1)c=gb_ex_circ(r);
   if(curve==2){gp_Ax2 a=r.ax2();double x=r.real();h=gp_Hypr(a,x,r.real());}
   gp_Ax3 a=r.ax3();gp_Pln pl(a);gp_Cylinder cy;gp_Cone co;gp_Sphere sp;gp_Torus to;
   if(surf==1)cy=gp_Cylinder(a,r.real());
   if(surf==2){double ang=r.real();co=gp_Cone(a,ang,r.real());}
   if(surf==3)sp=gp_Sphere(a,r.real());
   if(surf==4){double big=r.real();to=gp_Torus(a,big,r.real());}
   GB_TRY(if(curve==0&&surf==0)e.Perform(l,pl);if(curve==0&&surf==1)e.Perform(l,cy);if(curve==0&&surf==2)e.Perform(l,co);
    if(curve==0&&surf==3)e.Perform(l,sp);if(curve==0&&surf==4)e.Perform(l,to);
    if(curve==1&&surf==0)e.Perform(c,pl);if(curve==1&&surf==1)e.Perform(c,cy);if(curve==1&&surf==2)e.Perform(c,co);
    if(curve==1&&surf==3)e.Perform(c,sp);if(curve==1&&surf==4)e.Perform(c,to);if(curve==2)e.Perform(h,pl));
   gb_flag(e.IsDone());bool par=false;GB_TRY(par=e.IsParallel();gb_flag(par));
   GB_TRY(int m=e.NbExt();gb_int(m);
    if(!par){for(int i=1;i<=m;++i){gb_real(e.SquareDistance(i));Extrema_POnCurv p1;Extrema_POnSurf p2;e.Points(i,p1,p2);gb_poncurv(p1);gb_ponsurf(p2);}if(m>0)allocated=true;}
    else{gb_real(e.SquareDistance(1));if(allocated){Extrema_POnCurv p1;Extrema_POnSurf p2;e.Points(1,p1,p2);gb_poncurv(p1);gb_ponsurf(p2);}});
   GB_TRY(gb_real(e.SquareDistance(0)));}
 }catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
 gb_end();
}
// ex_elss: n steps of `kind S1 S2` on one object: 0 pln-pln, 1 pln-sphere,
// 2 sphere-sphere, 3 sphere-cylinder, 4 sphere-cone, 5 sphere-torus. Points is never
// called: the native point arrays are never allocated (a null dereference).
void gb_ex_elss_case(std::istream& in){
 GbReader r{in};
 try{
  int n=r.integer();Extrema_ExtElSS e;
  for(int q=0;q<n;++q){int kind=r.integer();gp_Ax3 a=r.ax3();
   if(kind==0){gp_Ax3 b=r.ax3();GB_TRY(e.Perform(gp_Pln(a),gp_Pln(b)));}
   else if(kind==1){gp_Ax3 b=r.ax3();GB_TRY(e.Perform(gp_Pln(a),gp_Sphere(b,r.real())));}
   else{gp_Sphere s(a,r.real());gp_Ax3 b=r.ax3();double x=r.real();
    GB_TRY(if(kind==2)e.Perform(s,gp_Sphere(b,x));if(kind==3)e.Perform(s,gp_Cylinder(b,x));
     if(kind==4)e.Perform(s,gp_Cone(b,x,r.real()));if(kind==5)e.Perform(s,gp_Torus(b,x,r.real())));}
   gb_flag(e.IsDone());GB_TRY(gb_flag(e.IsParallel()));
   GB_TRY(int m=e.NbExt();gb_int(m);for(int i=1;i<=m;++i)gb_real(e.SquareDistance(i)));
   GB_TRY(gb_real(e.SquareDistance(0)));}
 }catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
 gb_end();
}
// ex_pon: POnCurv, POnCurv2d, POnSurf and POnSurfParams: default values, construction
// and setters. Input: u v x y z sq type iu iv.
void gb_ex_pon_case(std::istream& in){
 GbReader r{in};
 double u=r.real(),v=r.real();gp_Pnt p=r.pnt();double sq=r.real();int type=r.integer(),iu=r.integer(),iv=r.integer();
 Extrema_POnCurv c;gb_poncurv(c);c.SetValues(u,p);gb_poncurv(c);gb_poncurv(Extrema_POnCurv(v,p));
 Extrema_POnCurv2d c2;gb_poncurv2d(c2);c2.SetValues(u,gp_Pnt2d(p.X(),p.Y()));gb_poncurv2d(c2);
 Extrema_POnSurf s;gb_ponsurf(s);s.SetParameters(u,v,p);gb_ponsurf(s);gb_ponsurf(Extrema_POnSurf(v,u,p));
 Extrema_POnSurfParams sp;gb_ponsurf(sp);gb_real(sp.GetSqrDistance());gb_int(sp.GetElementType());
 int i1,i2;sp.GetIndices(i1,i2);gb_int(i1);gb_int(i2);
 Extrema_POnSurfParams sp2(u,v,p);sp2.SetSqrDistance(sq);sp2.SetElementType(Extrema_ElementType(type));sp2.SetIndices(iu,iv);
 gb_ponsurf(sp2);gb_real(sp2.GetSqrDistance());gb_int(sp2.GetElementType());sp2.GetIndices(i1,i2);gb_int(i1);gb_int(i2);
 gb_end();
}
