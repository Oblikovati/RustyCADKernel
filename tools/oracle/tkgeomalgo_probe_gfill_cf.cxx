// GeomFill filling-part probe operations: boundaries, the Coons patch, tangent fields,
// corner states and the constrained filling (tkgeomalgo_probe.cxx). The Rust twin is
// crates/tkgeomalgo/examples/ga_probe/gfill_cf.rs; the curve reader and the surface
// dump are in tkgeomalgo_probe_gfill.cxx and tkgeomalgo_probe_gfill_fill.cxx.
#include <Adaptor3d_CurveOnSurface.hxx>
#include <GeomAdaptor_Curve.hxx>
#include <GeomAdaptor_Surface.hxx>
#include <Geom2dAdaptor_Curve.hxx>
#include <Geom2d_Line.hxx>
#include <GeomFill_BoundWithSurf.hxx>
#include <GeomFill_ConstrainedFilling.hxx>
#include <GeomFill_CoonsAlgPatch.hxx>
#include <GeomFill_CornerState.hxx>
#include <GeomFill_DegeneratedBound.hxx>
#include <GeomFill_SimpleBound.hxx>
#include <GeomFill_TgtOnCoons.hxx>
#include <Geom_CylindricalSurface.hxx>
#include <Geom_Plane.hxx>
#include <Geom_SphericalSurface.hxx>
#include <Law_Function.hxx>
#include <Law_Linear.hxx>
void ga_vec(const gp_Vec& v){ga_xyz(v.XYZ());}
// A surface: `0 P N X` (plane), `1 P N X r` (cylinder), `2 P N X r` (sphere).
occ::handle<Geom_Surface> ga_gf_surface(GaReader& r){
 int kind=r.integer();gp_Pnt p=r.pnt();gp_Dir n=r.dir();gp_Dir x=r.dir();gp_Ax3 ax(p,n,x);
 if(kind==0)return new Geom_Plane(ax);
 if(kind==1)return new Geom_CylindricalSurface(ax,r.real());
 return new Geom_SphericalSurface(ax,r.real());
}
// A boundary: `0 tol3d tolang first last <curve>` (SimpleBound on the curve, restricted
// to [first, last] when first < last), `1 tol3d tolang point first last`
// (DegeneratedBound), `2 tol3d tolang <surface> p2d d2d first last` (BoundWithSurf on a
// 2d line segment of the surface).
occ::handle<GeomFill_Boundary> ga_gf_boundary(GaReader& r){
 int kind=r.integer();double t3=r.real(),ta=r.real();
 if(kind==0){double f=r.real(),l=r.real();occ::handle<Geom_Curve> c=ga_gf_curve(r);
  occ::handle<Adaptor3d_Curve> a=f<l?new GeomAdaptor_Curve(c,f,l):new GeomAdaptor_Curve(c);return new GeomFill_SimpleBound(a,t3,ta);}
 if(kind==1){gp_Pnt p=r.pnt();double f=r.real();return new GeomFill_DegeneratedBound(p,f,r.real(),t3,ta);}
 occ::handle<Geom_Surface> s=ga_gf_surface(r);gp_Pnt2d p=r.pnt2d();gp_Dir2d d=r.dir2d();double f=r.real(),l=r.real();
 occ::handle<Geom2dAdaptor_Curve> c2=new Geom2dAdaptor_Curve(new Geom2d_Line(p,d),f,l);
 occ::handle<GeomAdaptor_Surface> as=new GeomAdaptor_Surface(s);
 Adaptor3d_CurveOnSurface cos(c2,as);
 return new GeomFill_BoundWithSurf(cos,t3,ta);
}
// The evaluations of a boundary at the relative parameters `ts` of its bounds.
void ga_gf_bound_report(const occ::handle<GeomFill_Boundary>& b,const std::vector<double>& ts){
 ga_flag(b->IsDegenerated());ga_flag(b->HasNormals());ga_real(b->Tol3d());ga_real(b->Tolang());
 double f=0,l=0;GA_TRY(b->Bounds(f,l);ga_real(f);ga_real(l));
 GA_TRY(gp_Pnt p1,p2;b->Points(p1,p2);ga_pnt(p1);ga_pnt(p2));
 for(double t:ts){double w=f+t*(l-f);
  GA_TRY(ga_pnt(b->Value(w)));GA_TRY(gp_Pnt p;gp_Vec v;b->D1(w,p,v);ga_pnt(p);ga_vec(v));
  GA_TRY(ga_vec(b->Norm(w)));GA_TRY(gp_Vec n,dn;b->D1Norm(w,n,dn);ga_vec(n);ga_vec(dn));}
}
// `<boundary> ts... first last hasdf hasdl df dl rev t3 ta`: a boundary evaluated,
// reparametrized, evaluated again, then given new tolerances.
void ga_gfill_bound_case(std::istream& input){
 GaReader r{input};occ::handle<GeomFill_Boundary> b=ga_gf_boundary(r);std::vector<double> ts=r.reals();
 double f=r.real(),l=r.real();bool hf=r.flag(),hl=r.flag();double df=r.real(),dl=r.real();bool rev=r.flag();
 double t3=r.real(),ta=r.real();
 ga_gf_bound_report(b,ts);GA_TRY(b->Reparametrize(f,l,hf,hl,df,dl,rev));ga_gf_bound_report(b,ts);
 b->Tol3d(t3);b->Tolang(ta);ga_real(b->Tol3d());ga_real(b->Tolang());
 ga_end();
}
// `ops...`: a corner state driven by (op, value) pairs: 0 Gap, 1 TgtAng, 2 NorAng,
// 3 Constraint, 4 DoKill; every query after each step.
void ga_gfill_corner_case(std::istream& input){
 GaReader r{input};std::vector<double> ops=r.reals();GeomFill_CornerState c;
 for(size_t k=0;k+1<ops.size();k+=2){int op=int(ops[k]);double v=ops[k+1];
  if(op==0)c.Gap(v);else if(op==1)c.TgtAng(v);else if(op==2)c.NorAng(v);else if(op==3)c.Constraint();else c.DoKill(v);
  ga_real(c.Gap());ga_real(c.TgtAng());ga_real(c.NorAng());ga_flag(c.HasConstraint());double s=0;ga_flag(c.IsToKill(s));ga_real(s);}
 ga_end();
}
// `4 boundaries, uvs...`: the Coons patch of the boundaries at relative (u, v) pairs,
// its corners and laws, the tangent fields of its sides, and the patch after
// swapping its laws.
void ga_gfill_patch_case(std::istream& input){
 GaReader r{input};occ::handle<GeomFill_Boundary> b[4];for(auto& x:b)x=ga_gf_boundary(r);std::vector<double> uv=r.reals();
 occ::handle<GeomFill_CoonsAlgPatch> k;
 try{k=new GeomFill_CoonsAlgPatch(b[0],b[1],b[2],b[3]);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 double u0=0,u1=0,v0=0,v1=0;b[0]->Bounds(u0,u1);b[1]->Bounds(v0,v1);
 for(int i=0;i<4;i++)ga_pnt(k->Corner(i));
 for(size_t n=0;n+1<uv.size();n+=2){double u=u0+uv[n]*(u1-u0),v=v0+uv[n+1]*(v1-v0);
  GA_TRY(ga_pnt(k->Value(u,v)));GA_TRY(ga_vec(k->D1U(u,v)));GA_TRY(ga_vec(k->D1V(u,v)));GA_TRY(ga_vec(k->DUV(u,v)));
  GA_TRY(ga_real(k->Func(0)->Value(v)));GA_TRY(ga_real(k->Func(1)->Value(u)));}
 for(int i=0;i<4;i++){occ::handle<GeomFill_TgtOnCoons> t=new GeomFill_TgtOnCoons(k,i);ga_flag(t->IsScalable());
  double f=0,l=0;b[i]->Bounds(f,l);
  for(double s:{0.,0.3,1.}){double w=f+s*(l-f);GA_TRY(ga_vec(t->Value(w)));GA_TRY(ga_vec(t->D1(w)));
   GA_TRY(gp_Vec a,d;t->D1(w,a,d);ga_vec(a);ga_vec(d));}}
 occ::handle<Law_Function> f1,f2;k->Func(f1,f2);k->SetFunc(f2,f1);
 GA_TRY(ga_pnt(k->Value(0.5*(u0+u1),0.5*(v0+v1))));
 ga_end();
}
// The filled surface, and the input index of each sorted boundary (-1 for the apex).
void ga_gf_cfill_report(GeomFill_ConstrainedFilling& f,const std::vector<occ::handle<GeomFill_Boundary>>& b){
 ga_gf_dump_bsurface(f.Surface());
 for(int i=0;i<4;i++){int found=-1;occ::handle<GeomFill_Boundary> s=f.Boundary(i);
  for(size_t k=0;k<b.size();k++)if(s==b[k])found=int(k);ga_int(found);}
 for(double w:{0.,0.3,1.})for(int ord=0;ord<2;ord++){double res[12]={0};ga_int(f.Eval(w,ord,res[0]));for(double x:res)ga_real(x);}
}
// `maxdeg maxseg nocheck checks n boundaries... doms... again [n2 boundaries2...]`: the
// constrained filling of 3 or 4 boundaries; `doms` are (input index, l) pairs for
// SetDomain before a ReBuild; with `checks`, the Check members' text; with `again`, a
// second Init of the same framework.
void ga_gfill_cfill_case(std::istream& input){
 GaReader r{input};int maxdeg=r.integer(),maxseg=r.integer();bool nocheck=r.flag(),checks=r.flag();int n=r.integer();
 std::vector<occ::handle<GeomFill_Boundary>> b;for(int k=0;k<n;k++)b.push_back(ga_gf_boundary(r));
 std::vector<double> doms=r.reals();bool again=r.flag();
 GeomFill_ConstrainedFilling f(maxdeg,maxseg);ga_flag(f.Surface().IsNull());GA_TRY(f.ReBuild());
 std::string error;
 try{if(n==3)f.Init(b[0],b[1],b[2],nocheck);else f.Init(b[0],b[1],b[2],b[3],nocheck);}
 catch(const Standard_Failure& e){error=e.ExceptionType();std::cout<<error<<' ';}
 if(!error.empty()){ga_end();return;}
 ga_gf_cfill_report(f,b);
 if(checks)for(int i=0;i<4;i++){
  ga_captured([&]{f.CheckCoonsAlgPatch(i);});ga_captured([&]{f.CheckTgteField(i);});
  ga_captured([&]{f.CheckApprox(i);});ga_captured([&]{f.CheckResult(i);});}
 if(!doms.empty()){
  for(size_t k=0;k+1<doms.size();k+=2)f.SetDomain(doms[k+1],occ::down_cast<GeomFill_BoundWithSurf>(b[int(doms[k])]));
  GA_TRY(f.ReBuild());ga_gf_dump_bsurface(f.Surface());}
 if(again){int n2=r.integer();std::vector<occ::handle<GeomFill_Boundary>> b2;for(int k=0;k<n2;k++)b2.push_back(ga_gf_boundary(r));
  GA_TRY(if(n2==3)f.Init(b2[0],b2[1],b2[2],nocheck);else f.Init(b2[0],b2[1],b2[2],b2[3],nocheck);ga_gf_dump_bsurface(f.Surface()));}
 ga_end();
}
