// TKGeomBase oracle: the IntAna2d package (tkgeombase_probe.cxx). A 2D element is read as
// its kind then its data: 0 line (P, D), 1 circle (Ax22d, R), 2 ellipse (Ax22d, Maj, Min),
// 3 parabola (Ax22d, Focal), 4 hyperbola (Ax22d, Maj, Min). An intersection prints IsDone
// then, when done, IsEmpty, IdenticalElements, ParallelElements, NbPoints, each point
// (value, ParamOnFirst, SecondIsImplicit, ParamOnSecond or its exception) and Point(N+1).
// The circle-conic Perform prints a message on std::cout when its solver fails; the probe
// discards std::cout around every intersection so that only its own tokens remain.
#include <IntAna2d_AnaIntersection.hxx>
#include <IntAna2d_Conic.hxx>
#include <IntAna2d_IntPoint.hxx>
#include <IntAna2d_Outils.hxx>
struct GbCoutSink{
 std::ostringstream sink;std::streambuf* old;
 GbCoutSink():old(std::cout.rdbuf(sink.rdbuf())){}
 ~GbCoutSink(){std::cout.rdbuf(old);}
};
struct GbConic2d{int kind;gp_Lin2d l;gp_Circ2d c;gp_Elips2d e;gp_Parab2d p;gp_Hypr2d h;};
GbConic2d gb_conic2d(GbReader& r){
 GbConic2d g{};g.kind=r.integer();
 if(g.kind==0){gp_Pnt2d p=r.pnt2d();g.l=gp_Lin2d(p,r.dir2d());}
 if(g.kind==1){gp_Ax22d a=r.ax22d();g.c=gp_Circ2d(a,r.real());}
 if(g.kind==2){gp_Ax22d a=r.ax22d();double M=r.real();g.e=gp_Elips2d(a,M,r.real());}
 if(g.kind==3){gp_Ax22d a=r.ax22d();g.p=gp_Parab2d(a,r.real());}
 if(g.kind==4){gp_Ax22d a=r.ax22d();double M=r.real();g.h=gp_Hypr2d(a,M,r.real());}
 return g;
}
IntAna2d_Conic gb_implicit(const GbConic2d& g){
 if(g.kind==0)return IntAna2d_Conic(g.l);
 if(g.kind==1)return IntAna2d_Conic(g.c);
 if(g.kind==2)return IntAna2d_Conic(g.e);
 if(g.kind==3)return IntAna2d_Conic(g.p);
 return IntAna2d_Conic(g.h);
}
void gb_int_point(const IntAna2d_IntPoint& p){
 gb_pnt2d(p.Value());gb_real(p.ParamOnFirst());gb_flag(p.SecondIsImplicit());GB_TRY(gb_real(p.ParamOnSecond()));
}
void gb_ana(const IntAna2d_AnaIntersection& a){
 gb_flag(a.IsDone());if(!a.IsDone()){GB_TRY(gb_int(a.NbPoints()));return;}
 gb_flag(a.IsEmpty());gb_flag(a.IdenticalElements());gb_flag(a.ParallelElements());
 int n=a.NbPoints();gb_int(n);
 for(int i=1;i<=n;++i)gb_int_point(a.Point(i));
 GB_TRY(gb_int_point(a.Point(n+1)));
}
// ia2_lines: L1 (P, D), L2 (P, D).
void gb_ia2_lines(GbReader& r){
 gp_Pnt2d p1=r.pnt2d();gp_Lin2d l1(p1,r.dir2d());gp_Pnt2d p2=r.pnt2d();gp_Lin2d l2(p2,r.dir2d());
 IntAna2d_AnaIntersection a(l1,l2);gb_ana(a);
}
void gb_ia2_lines_case(std::istream& in){GbReader r{in};GB_TRY(gb_ia2_lines(r));gb_end();}
// ia2_circles: C1 (Ax22d, R), C2 (Ax22d, R).
void gb_ia2_circles(GbReader& r){
 gp_Ax22d a1=r.ax22d();gp_Circ2d c1(a1,r.real());gp_Ax22d a2=r.ax22d();gp_Circ2d c2(a2,r.real());
 IntAna2d_AnaIntersection a;{GbCoutSink s;a.Perform(c1,c2);}gb_ana(a);
}
void gb_ia2_circles_case(std::istream& in){GbReader r{in};GB_TRY(gb_ia2_circles(r));gb_end();}
// ia2_line_circle: L (P, D), C (Ax22d, R).
void gb_ia2_line_circle(GbReader& r){
 gp_Pnt2d p=r.pnt2d();gp_Lin2d l(p,r.dir2d());gp_Ax22d a1=r.ax22d();gp_Circ2d c(a1,r.real());
 IntAna2d_AnaIntersection a;{GbCoutSink s;a.Perform(l,c);}gb_ana(a);
}
void gb_ia2_line_circle_case(std::istream& in){GbReader r{in};GB_TRY(gb_ia2_line_circle(r));gb_end();}
// ia2_conic: the first element (line, circle, ellipse, parabola, hyperbola), then the
// implicit conic.
void gb_ia2_conic(GbReader& r){
 GbConic2d g=gb_conic2d(r);IntAna2d_Conic c=gb_implicit(gb_conic2d(r));
 IntAna2d_AnaIntersection a;
 {GbCoutSink s;
  if(g.kind==0)a.Perform(g.l,c);
  if(g.kind==1)a.Perform(g.c,c);
  if(g.kind==2)a.Perform(g.e,c);
  if(g.kind==3)a.Perform(g.p,c);
  if(g.kind==4)a.Perform(g.h,c);}
 gb_ana(a);
}
void gb_ia2_conic_case(std::istream& in){GbReader r{in};GB_TRY(gb_ia2_conic(r));gb_end();}
// ia2_conic_eval: a conic, X, Y and an Ax2d: coefficients, Value, Grad, ValAndGrad and the
// coefficients in the Ax2d frame.
void gb_ia2_conic_eval(GbReader& r){
 IntAna2d_Conic c=gb_implicit(gb_conic2d(r));double x=r.real(),y=r.real();gp_Ax2d ax=r.ax2d();
 double A,B,C,D,E,F;c.Coefficients(A,B,C,D,E,F);
 for(double v:{A,B,C,D,E,F})gb_real(v);
 gb_real(c.Value(x,y));gb_xy(c.Grad(x,y));
 double val;gp_XY g;c.ValAndGrad(x,y,val,g);gb_real(val);gb_xy(g);
 c.NewCoefficients(A,B,C,D,E,F,ax);
 for(double v:{A,B,C,D,E,F})gb_real(v);
}
void gb_ia2_conic_eval_case(std::istream& in){GbReader r{in};GB_TRY(gb_ia2_conic_eval(r));gb_end();}
// ia2_roots k: 0 quartic (A4..A0), 1 quadratic (A2..A0) of MyDirectPolynomialRoots.
void gb_ia2_roots_case(std::istream& in){
 GbReader r{in};int k=r.integer();
 auto print=[](const MyDirectPolynomialRoots& m){
  gb_flag(m.IsDone()!=0.0);gb_flag(m.InfiniteRoots());int n=m.NbSolutions();gb_int(n);
  for(int i=1;i<=n;++i)gb_real(m.Value(i));};
 if(k==0){double a4=r.real(),a3=r.real(),a2=r.real(),a1=r.real(),a0=r.real();print(MyDirectPolynomialRoots(a4,a3,a2,a1,a0));}
 else{double a2=r.real(),a1=r.real(),a0=r.real();print(MyDirectPolynomialRoots(a2,a1,a0));}
 gb_end();
}
// ia2_outils: Points_Confondus(x1, y1, x2, y2), Coord_Ancien_Repere(x, y, Ax2d), then
// Traitement_Points_Confondus on N points (X, Y, U1, U2).
void gb_ia2_outils_case(std::istream& in){
 GbReader r{in};double x1=r.real(),y1=r.real(),x2=r.real(),y2=r.real();
 gb_flag(Points_Confondus(x1,y1,x2,y2));
 double x=r.real(),y=r.real();gp_Ax2d ax=r.ax2d();Coord_Ancien_Repere(x,y,ax);gb_real(x);gb_real(y);
 int n=r.integer();IntAna2d_IntPoint pts[4];
 for(int i=0;i<n;++i){double px=r.real(),py=r.real(),u1=r.real(),u2=r.real();pts[i].SetValue(px,py,u1,u2);}
 Traitement_Points_Confondus(n,pts);gb_int(n);
 for(int i=0;i<4;++i)gb_int_point(pts[i]);
 gb_end();
}
// ia2_int_point: X, Y, U1, U2: the default point, both constructors and both SetValue.
void gb_ia2_int_point_case(std::istream& in){
 GbReader r{in};double x=r.real(),y=r.real(),u1=r.real(),u2=r.real();
 IntAna2d_IntPoint d;gb_int_point(d);
 gb_int_point(IntAna2d_IntPoint(x,y,u1,u2));gb_int_point(IntAna2d_IntPoint(x,y,u1));
 d.SetValue(y,x,u2);gb_int_point(d);d.SetValue(y,x,u2,u1);gb_int_point(d);
 IntAna2d_AnaIntersection empty;gb_ana(empty);
 GB_TRY(gb_flag(empty.IsEmpty()));GB_TRY(gb_flag(empty.IdenticalElements()));
 GB_TRY(gb_flag(empty.ParallelElements()));GB_TRY(gb_int_point(empty.Point(1)));
 gb_end();
}
