// TKGeomBase oracle: the IntAna package (tkgeombase_probe.cxx). A surface is read by kind:
// 0 plane (Ax3), 1 cylinder (Ax3, R), 2 sphere (Ax3, R), 3 cone (Ax3, Angle, R),
// 4 torus (Ax3, R, r); a conic by kind: 0 line (P, D), 1 circle (Ax2, R),
// 2 ellipse (Ax2, Maj, Min), 3 parabola (Ax2, Focal), 4 hyperbola (Ax2, Maj, Min).
// Every result is printed with its exceptions, including the out-of-range accessors.
#include <IntAna_Curve.hxx>
#include <IntAna_IntConicQuad.hxx>
#include <IntAna_IntLinTorus.hxx>
#include <IntAna_IntQuadQuad.hxx>
#include <IntAna_QuadQuadGeo.hxx>
#include <IntAna_Quadric.hxx>
#include <NCollection_List.hxx>
#include <gp_Sphere.hxx>
#include <gp_Torus.hxx>
struct GbSurface{int kind;gp_Pln pln;gp_Cylinder cyl;gp_Sphere sph;gp_Cone cone;gp_Torus tor;};
GbSurface gb_surface(GbReader& r){
 GbSurface s{};s.kind=r.integer();gp_Ax3 a=r.ax3();
 if(s.kind==0)s.pln=gp_Pln(a);
 if(s.kind==1)s.cyl=gp_Cylinder(a,r.real());
 if(s.kind==2)s.sph=gp_Sphere(a,r.real());
 if(s.kind==3){double ang=r.real();s.cone=gp_Cone(a,ang,r.real());}
 if(s.kind==4){double M=r.real();s.tor=gp_Torus(a,M,r.real());}
 return s;
}
IntAna_Quadric gb_quadric(const GbSurface& s){
 if(s.kind==0)return IntAna_Quadric(s.pln);
 if(s.kind==1)return IntAna_Quadric(s.cyl);
 if(s.kind==2)return IntAna_Quadric(s.sph);
 return IntAna_Quadric(s.cone);
}
struct GbConic3d{int kind;gp_Lin l;gp_Circ c;gp_Elips e;gp_Parab p;gp_Hypr h;};
GbConic3d gb_conic3d(GbReader& r){
 GbConic3d g{};g.kind=r.integer();
 if(g.kind==0){gp_Pnt p=r.pnt();g.l=gp_Lin(p,r.dir());return g;}
 gp_Ax2 a=r.ax2();
 if(g.kind==1)g.c=gp_Circ(a,r.real());
 if(g.kind==2){double M=r.real();g.e=gp_Elips(a,M,r.real());}
 if(g.kind==3)g.p=gp_Parab(a,r.real());
 if(g.kind==4){double M=r.real();g.h=gp_Hypr(a,M,r.real());}
 return g;
}
void gb_coefficients(const double (&q)[10]){for(double v:q)gb_real(v);}
// ia_quadric: a surface (plane, cylinder, sphere or cone) then an Ax3: the coefficients,
// the special points after one and two SetQuadric, and the coefficients in the Ax3.
void gb_ia_quadric(GbReader& r){
 GbSurface s=gb_surface(r);gp_Ax3 ax=r.ax3();
 IntAna_Quadric q=gb_quadric(s);
 double c[10];q.Coefficients(c[0],c[1],c[2],c[3],c[4],c[5],c[6],c[7],c[8],c[9]);gb_coefficients(c);
 if(s.kind==0)q.SetQuadric(s.pln);
 if(s.kind==1)q.SetQuadric(s.cyl);
 if(s.kind==2)q.SetQuadric(s.sph);
 if(s.kind==3)q.SetQuadric(s.cone);
 gb_int(q.SpecialPoints().Size());
 for(const gp_Pnt& p:q.SpecialPoints())gb_pnt(p);
 q.NewCoefficients(c[0],c[1],c[2],c[3],c[4],c[5],c[6],c[7],c[8],c[9],ax);gb_coefficients(c);
 IntAna_Quadric empty;empty.Coefficients(c[0],c[1],c[2],c[3],c[4],c[5],c[6],c[7],c[8],c[9]);gb_coefficients(c);
}
void gb_ia_quadric_case(std::istream& in){GbReader r{in};GB_TRY(gb_ia_quadric(r));gb_end();}
void gb_conic_quad(const IntAna_IntConicQuad& q){
 gb_flag(q.IsDone());if(!q.IsDone()){GB_TRY(gb_int(q.NbPoints()));return;}
 gb_flag(q.IsInQuadric());gb_flag(q.IsParallel());
 GB_TRY(int n=q.NbPoints();gb_int(n);for(int i=1;i<=n;++i){gb_pnt(q.Point(i));gb_real(q.ParamOnConic(i));}
        gb_real(q.ParamOnConic(n+1)));
}
// ia_conic_quad: a conic then a quadric surface.
void gb_ia_conic_quad(GbReader& r){
 GbConic3d g=gb_conic3d(r);IntAna_Quadric q=gb_quadric(gb_surface(r));IntAna_IntConicQuad i;
 if(g.kind==0)i.Perform(g.l,q);
 if(g.kind==1)i.Perform(g.c,q);
 if(g.kind==2)i.Perform(g.e,q);
 if(g.kind==3)i.Perform(g.p,q);
 if(g.kind==4)i.Perform(g.h,q);
 gb_conic_quad(i);
}
void gb_ia_conic_quad_case(std::istream& in){GbReader r{in};GB_TRY(gb_ia_conic_quad(r));gb_end();}
// ia_conic_plane: a conic, a plane (Ax3), Tolang, Tol, Len (the tolerances each overload
// takes, in order).
void gb_ia_conic_plane(GbReader& r){
 GbConic3d g=gb_conic3d(r);gp_Pln p(r.ax3());double ta=r.real(),t=r.real(),len=r.real();
 // The empty constructor then Perform: the other constructors leave the flags that
 // their Perform skips uninitialized.
 IntAna_IntConicQuad i;
 if(g.kind==0)i.Perform(g.l,p,ta,t,len);
 if(g.kind==1)i.Perform(g.c,p,ta,t);
 if(g.kind==2)i.Perform(g.e,p,ta,t);
 if(g.kind==3)i.Perform(g.p,p,ta);
 if(g.kind==4)i.Perform(g.h,p,ta);
 gb_conic_quad(i);
}
void gb_ia_conic_plane_case(std::istream& in){GbReader r{in};GB_TRY(gb_ia_conic_plane(r));gb_end();}
// ia_lin_torus: a line (P, D) and a torus (Ax3, R, r).
void gb_ia_lin_torus(GbReader& r){
 gp_Pnt p=r.pnt();gp_Lin l(p,r.dir());gp_Ax3 a=r.ax3();double M=r.real();gp_Torus t(a,M,r.real());
 IntAna_IntLinTorus i(l,t);gb_flag(i.IsDone());
 GB_TRY(int n=i.NbPoints();gb_int(n);
  for(int k=1;k<=n;++k){gb_pnt(i.Value(k));gb_real(i.ParamOnLine(k));double fi,th;i.ParamOnTorus(k,fi,th);gb_real(fi);gb_real(th);}
  gb_real(i.ParamOnLine(n+1)));
}
void gb_ia_lin_torus_case(std::istream& in){GbReader r{in};GB_TRY(gb_ia_lin_torus(r));gb_end();}
void gb_qqgeo_solution(const IntAna_QuadQuadGeo& q,int i){
 switch(q.TypeInter()){
  case IntAna_Point:GB_TRY(gb_pnt(q.Point(i)));break;
  case IntAna_Line:GB_TRY(gb_lin(q.Line(i)));break;
  case IntAna_Circle:GB_TRY(gb_circ(q.Circle(i)));break;
  case IntAna_PointAndCircle:GB_TRY(gb_pnt(q.Point(1)));GB_TRY(gb_circ(q.Circle(1)));break;
  case IntAna_Ellipse:GB_TRY(gb_elips(q.Ellipse(i)));break;
  case IntAna_Parabola:GB_TRY(gb_parab(q.Parabola(i)));break;
  case IntAna_Hyperbola:GB_TRY(gb_hypr(q.Hyperbola(i)));break;
  default:break;
 }
}
void gb_qqgeo(const IntAna_QuadQuadGeo& q){
 gb_flag(q.IsDone());gb_flag(q.HasCommonGen());gb_pnt(q.PChar());
 if(!q.IsDone()){GB_TRY(gb_int(q.TypeInter()));return;}
 gb_int(q.TypeInter());int n=q.NbSolutions();gb_int(n);
 for(int i=1;i<=n;++i)gb_qqgeo_solution(q,i);
 GB_TRY(gb_pnt(q.Point(1)));GB_TRY(gb_lin(q.Line(n+1)));GB_TRY(gb_circ(q.Circle(n+1)));
 GB_TRY(gb_elips(q.Ellipse(0)));GB_TRY(gb_parab(q.Parabola(2)));GB_TRY(gb_hypr(q.Hyperbola(n+1)));
}
// ia_qqgeo: two surfaces then the tolerances of the matching overload: plane-plane
// (TolAng, Tol), plane-cylinder (Tolang, Tol, H), plane-sphere (), plane-cone (Tolang, Tol),
// any other pair (Tol). The pair is taken in the order of the overloads.
void gb_ia_qqgeo(GbReader& r){
 GbSurface a=gb_surface(r),b=gb_surface(r);int ka=a.kind,kb=b.kind;IntAna_QuadQuadGeo q;
 if(ka==0&&kb==0){double ta=r.real();q.Perform(a.pln,b.pln,ta,r.real());}
 else if(ka==0&&kb==1){double ta=r.real(),t=r.real();q.Perform(a.pln,b.cyl,ta,t,r.real());}
 else if(ka==0&&kb==2){q.Perform(a.pln,b.sph);}
 else if(ka==0&&kb==3){double ta=r.real();q.Perform(a.pln,b.cone,ta,r.real());}
 else{double t=r.real();
  if(ka==1&&kb==1)q.Perform(a.cyl,b.cyl,t);
  if(ka==1&&kb==2)q.Perform(a.cyl,b.sph,t);
  if(ka==1&&kb==3)q.Perform(a.cyl,b.cone,t);
  if(ka==2&&kb==2)q.Perform(a.sph,b.sph,t);
  if(ka==2&&kb==3)q.Perform(a.sph,b.cone,t);
  if(ka==3&&kb==3)q.Perform(a.cone,b.cone,t);
  if(ka==0&&kb==4)q.Perform(a.pln,b.tor,t);
  if(ka==1&&kb==4)q.Perform(a.cyl,b.tor,t);
  if(ka==3&&kb==4)q.Perform(a.cone,b.tor,t);
  if(ka==2&&kb==4)q.Perform(a.sph,b.tor,t);
  if(ka==4&&kb==4)q.Perform(a.tor,b.tor,t);}
 gb_qqgeo(q);
}
void gb_ia_qqgeo_case(std::istream& in){GbReader r{in};GB_TRY(gb_ia_qqgeo(r));gb_end();}
void gb_ia_curve(const IntAna_Curve& source){
 IntAna_Curve c=source;
 gb_flag(c.IsOpen());gb_flag(c.IsConstant());gb_flag(c.IsFirstOpen());gb_flag(c.IsLastOpen());
 double f,l;c.Domain(f,l);gb_real(f);gb_real(l);
 for(int k=0;k<=4;++k){double t=f+(l-f)*k/4;GB_TRY(gb_pnt(c.Value(t)));}
 double t=f+(l-f)*0.37;gp_Pnt p(0,0,0);gp_Vec v(0,0,0);
 GB_TRY(gb_flag(c.D1u(t,p,v));gb_pnt(p);gb_vec(v));
 GB_TRY(gb_pnt(c.Value(l+1.0)));
 GB_TRY(gp_Pnt at=c.Value(t);NCollection_List<double> ps;c.FindParameter(at,ps);gb_int(ps.Size());for(double d:ps)gb_real(d));
 GB_TRY(gp_Pnt at=c.Value(f);NCollection_List<double> ps;c.FindParameter(at,ps);gb_int(ps.Size());for(double d:ps)gb_real(d));
}
// ia_qq k: k = 0 cylinder (Ax3, R), 1 cone (Ax3, Angle, R), 2 a new intersection; then a
// quadric surface and Tol. Prints every curve, its links and the out-of-range accessors.
void gb_ia_qq(GbReader& r){
 int k=r.integer();IntAna_IntQuadQuad q;
 if(k==2){gb_flag(q.IsDone());GB_TRY(gb_int(q.NbCurve()));GB_TRY(gb_flag(q.HasNextCurve(1)));return;}
 gp_Ax3 a=r.ax3();gp_Cylinder cyl;gp_Cone cone;
 if(k==0)cyl=gp_Cylinder(a,r.real());else{double ang=r.real();cone=gp_Cone(a,ang,r.real());}
 IntAna_Quadric quad=gb_quadric(gb_surface(r));double tol=r.real();
 if(k==0)q.Perform(cyl,quad,tol);else q.Perform(cone,quad,tol);
 gb_flag(q.IsDone());if(!q.IsDone()){GB_TRY(gb_int(q.NbCurve()));return;}
 gb_flag(q.IdenticalElements());int n=q.NbCurve();gb_int(n);gb_int(q.NbPnt());
 for(int i=1;i<=n;++i){
  gb_ia_curve(q.Curve(i));
  gb_flag(q.HasNextCurve(i));gb_flag(q.HasPreviousCurve(i));
  GB_TRY(bool o=false;int j=q.NextCurve(i,o);gb_int(j);gb_flag(o));
  GB_TRY(bool o=false;int j=q.PreviousCurve(i,o);gb_int(j);gb_flag(o));
 }
 GB_TRY(gb_ia_curve(q.Curve(n+1)));GB_TRY(gb_pnt(q.Point(1)));GB_TRY(gb_flag(q.HasPreviousCurve(0)));
}
void gb_ia_qq_case(std::istream& in){GbReader r{in};GB_TRY(gb_ia_qq(r));gb_end();}
// ia_curve: an IntAna_Curve set directly: k = 0 cylinder (Ax3, R), 1 cone (Ax3, Angle, R);
// the ten quadric coefficients, Tol, DomInf, DomSup, TwoCurves, ZIsPositive, then the
// trim (First, Last) of SetDomain and the open flags.
void gb_ia_curve_direct(GbReader& r){
 int k=r.integer();gp_Ax3 a=r.ax3();gp_Cylinder cyl;gp_Cone cone;
 if(k==0)cyl=gp_Cylinder(a,r.real());else{double ang=r.real();cone=gp_Cone(a,ang,r.real());}
 double q[10];for(double& v:q)v=r.real();
 double tol=r.real(),inf=r.real(),sup=r.real();bool two=r.flag(),pos=r.flag();
 double first=r.real(),last=r.real();bool fo=r.flag(),lo=r.flag();
 IntAna_Curve c;GB_TRY(double f,l;c.Domain(f,l));gb_flag(c.IsOpen());
 if(k==0)c.SetCylinderQuadValues(cyl,q[0],q[1],q[2],q[3],q[4],q[5],q[6],q[7],q[8],q[9],tol,inf,sup,two,pos);
 else c.SetConeQuadValues(cone,q[0],q[1],q[2],q[3],q[4],q[5],q[6],q[7],q[8],q[9],tol,inf,sup,two,pos);
 gb_ia_curve(c);
 GB_TRY(c.SetDomain(first,last);gb_ia_curve(c));
 c.SetIsFirstOpen(fo);c.SetIsLastOpen(lo);gb_ia_curve(c);
}
void gb_ia_curve_case(std::istream& in){GbReader r{in};GB_TRY(gb_ia_curve_direct(r));gb_end();}
