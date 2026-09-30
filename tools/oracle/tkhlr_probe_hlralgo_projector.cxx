// HLRAlgo_Projector, the Rust twin of crates/tkhlr/examples/hl_probe/hlr_algo_projector.rs:
// the five constructors, Set and Scaled, and every projection.
#include <HLRAlgo_Projector.hxx>
#include <gp_Ax2.hxx>
#include <gp_Ax3.hxx>
#include <gp_Lin.hxx>
#include <gp_Trsf.hxx>
#include <gp_Vec.hxx>
#include <gp_Vec2d.hxx>
// `P N X k V`: SetTransformation(gp_Ax3(gp_Ax2(P, N, X))), SetScaleFactor(k),
// SetTranslationPart(V).
gp_Trsf hl_read_trsf(GaReader& r){
 gp_Pnt p=r.pnt();gp_Dir n=r.dir();gp_Dir x=r.dir();double k=r.real();gp_XYZ v=r.xyz();
 gp_Trsf t;t.SetTransformation(gp_Ax3(gp_Ax2(p,n,x)));t.SetScaleFactor(k);t.SetTranslationPart(gp_Vec(v));return t;}
void hl_print_trsf(const gp_Trsf& t){for(int i=1;i<=3;++i)for(int j=1;j<=4;++j)ga_real(t.Value(i,j));}
void hl_print_projector(const HLRAlgo_Projector& pr){
 ga_flag(pr.Perspective());hl_print_trsf(pr.Transformation());hl_print_trsf(pr.InvertedTransformation());hl_print_trsf(pr.FullTransformation());
 gp_Vec2d d1,d2,d3;pr.Directions(d1,d2,d3);ga_xy(d1.XY());ga_xy(d2.XY());ga_xy(d3.XY());GA_TRY(ga_real(pr.Focus()))}
// `kind ... n (P D x y)*n [setkind ...] scaled`: kind 0 the default projector, 1 CS, 2 CS and
// focus, 3 T persp focus, 4 T persp focus v1 v2 v3; then optionally Set(T, persp, focus)
// and Scaled(on); the projector, then per point every projection of P (Pout starting at
// (7, 8)), P with D, Shoot(x, y), Transform(P) and Transform(D).
void hl_ha_projector_case(std::istream& input){
 GaReader r{input};int kind=r.integer();HLRAlgo_Projector pr;
 GA_TRY(
  if(kind==1){gp_Pnt p=r.pnt();gp_Dir n=r.dir();pr=HLRAlgo_Projector(gp_Ax2(p,n,r.dir()));}
  else if(kind==2){gp_Pnt p=r.pnt();gp_Dir n=r.dir();gp_Dir x=r.dir();pr=HLRAlgo_Projector(gp_Ax2(p,n,x),r.real());}
  else if(kind==3){gp_Trsf t=hl_read_trsf(r);bool ps=r.flag();pr=HLRAlgo_Projector(t,ps,r.real());}
  else if(kind==4){gp_Trsf t=hl_read_trsf(r);bool ps=r.flag();double f=r.real();gp_Pnt2d a=r.pnt2d(),b=r.pnt2d(),c=r.pnt2d();
   pr=HLRAlgo_Projector(t,ps,f,gp_Vec2d(a.XY()),gp_Vec2d(b.XY()),gp_Vec2d(c.XY()));}
  if(r.flag()){gp_Trsf t=hl_read_trsf(r);bool ps=r.flag();pr.Set(t,ps,r.real());}
  int scaled=r.integer();if(scaled>=0)pr.Scaled(scaled!=0);
  hl_print_projector(pr);
  for(int n=r.integer();n>0;--n){gp_Pnt p=r.pnt();gp_XYZ d=r.xyz();double x=r.real(),y=r.real();
   gp_Pnt2d out(7,8);pr.Project(p,out);ga_pnt2d(out);double X,Y,Z;pr.Project(p,X,Y,Z);ga_real(X);ga_real(Y);ga_real(Z);
   gp_Pnt2d o2;gp_Vec2d v2;pr.Project(p,gp_Vec(d),o2,v2);ga_pnt2d(o2);ga_xy(v2.XY());
   GA_TRY(gp_Lin l=pr.Shoot(x,y);ga_pnt(l.Location());ga_dir(l.Direction()))
   gp_Pnt tp=p;pr.Transform(tp);ga_pnt(tp);gp_Vec tv(d);pr.Transform(tv);ga_xyz(tv.XYZ());})
 ga_end();}
