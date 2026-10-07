// BRepBlend probes of tools/oracle/tkfillet_probe.cxx (included after the BlendFunc
// probes, whose readers they use); the Rust twin is
// crates/tkfillet/examples/fl_probe/brep_blend.rs.
#include <BRepBlend_AppFunc.hxx>
#include <BRepBlend_AppSurf.hxx>
#include <BRepBlend_AppSurface.hxx>
#include <BRepBlend_BlendTool.hxx>
#include <BRepBlend_Line.hxx>
#include <BlendFunc_ConstRad.hxx>
#include <BRepBlend_CurvPointRadInv.hxx>
#include <BRepBlend_HCurve2dTool.hxx>
#include <BRepBlend_HCurveTool.hxx>
#include <BRepBlend_SurfCurvConstRadInv.hxx>
#include <BRepBlend_SurfCurvEvolRadInv.hxx>
#include <BRepBlend_SurfPointConstRadInv.hxx>
#include <BRepBlend_SurfPointEvolRadInv.hxx>
#include <BRepBlend_RstRstConstRad.hxx>
#include <BRepBlend_RstRstEvolRad.hxx>
#include <BRepBlend_SurfRstConstRad.hxx>
#include <BRepBlend_SurfRstEvolRad.hxx>
#include <Geom2d_Circle.hxx>
#include <gp_Circ2d.hxx>

// A 2d curve: kind 0 the line through (a, b) along (c, d), 1 the circle of center (a, b)
// and radius c (d unused); 5 numbers.
occ::handle<Adaptor2d_Curve2d> fl_curve2d_kind(GaReader& r){
 int kind=r.integer();double a=r.real(),b=r.real(),c=r.real(),d=r.real();
 if(kind==1)return new Geom2dAdaptor_Curve(new Geom2d_Circle(gp_Circ2d(gp_Ax2d(gp_Pnt2d(a,b),gp_Dir2d(1,0)),c)));
 return new Geom2dAdaptor_Curve(new Geom2d_Line(gp_Pnt2d(a,b),gp_Dir2d(c,d)));}

// `fl_rad_inv kind S C C2 P2d D2d Law radius choix P X1..X3 tol`: kind 0
// SurfPointConstRadInv(S, C), 1 SurfPointEvolRadInv(S, C, Law), 2 SurfCurvConstRadInv(S, C2,
// C), 3 SurfCurvEvolRadInv(S, C2, C, Law), 4 CurvPointRadInv(C, C2) on (X1, X2).
void fl_rad_inv_case(std::istream& in){
 GaReader r{in};int kind=r.integer();
 occ::handle<Adaptor3d_Surface> s=fl_surface(r);occ::handle<Adaptor3d_Curve> c=fl_guide(r),c2=fl_guide(r);
 occ::handle<Adaptor2d_Curve2d> cs=fl_curve2d(r);occ::handle<Law_Function> law=fl_law(r);
 double radius=r.real();int choix=r.integer();gp_Pnt p=r.pnt();
 math_Vector x3(1,3);for(int i=1;i<=3;++i)x3(i)=r.real();double tol=r.real();
 if(kind==0){BRepBlend_SurfPointConstRadInv f(s,c);f.Set(radius,choix);f.Set(p);fl_inverse(f,x3,tol);}
 else if(kind==1){BRepBlend_SurfPointEvolRadInv f(s,c,law);f.Set(choix);f.Set(p);fl_inverse(f,x3,tol);}
 else if(kind==2){BRepBlend_SurfCurvConstRadInv f(s,c2,c);f.Set(radius,choix);f.Set(cs);fl_inverse(f,x3,tol);}
 else if(kind==3){BRepBlend_SurfCurvEvolRadInv f(s,c2,c,law);f.Set(choix);f.Set(cs);fl_inverse(f,x3,tol);}
 else{math_Vector x2(1,2);x2(1)=x3(1);x2(2)=x3(2);BRepBlend_CurvPointRadInv f(c,c2);f.Set(choix);f.Set(p);fl_inverse(f,x2,tol);}
 ga_end();}

// `fl_blend_tool C2d P1 P2 C U0 U1`: the projection of P1 on the 2d curve, the
// intersection of [P1, P2] with it, its bounds, and the sample counts of the tools.
void fl_blend_tool_case(std::istream& in){
 GaReader r{in};
 occ::handle<Adaptor2d_Curve2d> arc=fl_curve2d_kind(r);gp_Pnt2d p1=r.pnt2d(),p2=r.pnt2d();
 occ::handle<Adaptor3d_Curve> c=fl_guide(r);double u0=r.real(),u1=r.real();
 occ::handle<Adaptor3d_Surface> none;
 GA_TRY(double prm,dist;ga_flag(BRepBlend_BlendTool::Project(p1,none,arc,prm,dist));ga_real(prm);ga_real(dist));
 GA_TRY(double prm=0,dist=0;ga_flag(BRepBlend_BlendTool::Inters(p1,p2,none,arc,prm,dist));ga_real(prm);ga_real(dist));
 GA_TRY(double a,b;BRepBlend_BlendTool::Bounds(arc,a,b);ga_real(a);ga_real(b));
 GA_TRY(ga_int(BRepBlend_HCurve2dTool::NbSamples(arc,u0,u1)));
 GA_TRY(ga_int(BRepBlend_HCurveTool::NbSamples(c,u0,u1)));
 ga_end();}

// The members of a surface-restriction fillet after Set(t), at X.
template<class F> void fl_surf_rst_members(F& f,const math_Vector& x,double t,double tol){
 math_Vector fv(1,3);math_Matrix d(1,3,1,3);fv.Init(0);d.Init(0);
 GA_TRY(ga_flag(f.Value(x,fv));fl_vector(fv));
 GA_TRY(ga_flag(f.Derivatives(x,d));fl_matrix(d));
 GA_TRY(ga_flag(f.Values(x,fv,d));fl_vector(fv);fl_matrix(d));
 bool solved=false;GA_TRY(solved=f.IsSolution(x,tol);ga_flag(solved));
 ga_flag(f.IsTangencyPoint());GA_TRY(fl_vec(f.TangentOnS()));GA_TRY(fl_vec2d(f.Tangent2dOnS()));
 GA_TRY(fl_vec(f.TangentOnRst()));GA_TRY(fl_vec2d(f.Tangent2dOnRst()));
 ga_pnt(f.PointOnS());ga_pnt(f.PointOnRst());ga_pnt2d(f.Pnt2dOnS());ga_pnt2d(f.Pnt2dOnRst());
 // The evolutive class leaves the restriction parameter uninitialized until a solution.
 if(solved)ga_real(f.ParameterOnRst());
 GA_TRY(gp_Vec ns,tg;ga_flag(f.Decroch(x,ns,tg));fl_vec(ns);fl_vec(tg));
 GA_TRY(double pd,pf;gp_Circ k;f.Section(t,x(1),x(2),x(3),pd,pf,k);ga_real(pd);ga_real(pf);
        ga_pnt(k.Location());ga_dir(k.Axis().Direction());ga_dir(k.XAxis().Direction());ga_real(k.Radius()));
 fl_sections_at(f,Blend_Point(gp_Pnt(),gp_Pnt(),t,x(1),x(2),x(3)));
 fl_app_queries(f,tol);}

template<class F> void fl_surf_rst_run(F& f,const occ::handle<Adaptor3d_Surface>& srst,const occ::handle<Adaptor2d_Curve2d>& rst,
                                       int shape,double t,bool trim,double first,double last,const math_Vector& x,double tol){
 // myTConv is uninitialized until GetShape meets a non-linear shape: set it first.
 {int a,b,c,d;f.Set(BlendFunc_Rational);f.GetShape(a,b,c,d);}
 f.Set(BlendFunc_SectionShape(shape));f.Set(srst,rst);
 if(trim)GA_TRY(f.Set(first,last));
 GA_TRY(f.Set(t));
 fl_surf_rst_members(f,x,t,tol);}

// `fl_surf_rst kind S SRst Rst C Law radius choix shape t trim first last X1..X3 tol`: kind 0
// SurfRstConstRad, 1 SurfRstEvolRad; the reference of Decroch is the restriction itself.
void fl_surf_rst_case(std::istream& in){
 GaReader r{in};int kind=r.integer();
 occ::handle<Adaptor3d_Surface> s=fl_surface(r),srst=fl_surface(r);occ::handle<Adaptor2d_Curve2d> rst=fl_curve2d(r);
 occ::handle<Adaptor3d_Curve> c=fl_guide(r);occ::handle<Law_Function> law=fl_law(r);
 double radius=r.real();int choix=r.integer(),shape=r.integer();double t=r.real();bool trim=r.flag();
 double first=r.real(),last=r.real();math_Vector x(1,3);for(int i=1;i<=3;++i)x(i)=r.real();double tol=r.real();
 if(kind==0){BRepBlend_SurfRstConstRad f(s,srst,rst,c);f.Set(radius,choix);fl_surf_rst_run(f,srst,rst,shape,t,trim,first,last,x,tol);}
 else{BRepBlend_SurfRstEvolRad f(s,srst,rst,c,law);f.Set(choix);fl_surf_rst_run(f,srst,rst,shape,t,trim,first,last,x,tol);}
 ga_end();}

// The members of a restriction-restriction fillet after Set(t), at X.
template<class F> void fl_rst_rst_members(F& f,const math_Vector& x,double t,double tol){
 math_Vector fv(1,2);math_Matrix d(1,2,1,2);fv.Init(0);d.Init(0);
 GA_TRY(ga_flag(f.Value(x,fv));fl_vector(fv));
 GA_TRY(ga_flag(f.Derivatives(x,d));fl_matrix(d));
 GA_TRY(ga_flag(f.Values(x,fv,d));fl_vector(fv);fl_matrix(d));
 bool solved=false;GA_TRY(solved=f.IsSolution(x,tol);ga_flag(solved));
 ga_flag(f.IsTangencyPoint());GA_TRY(fl_vec(f.TangentOnRst1()));GA_TRY(fl_vec2d(f.Tangent2dOnRst1()));
 GA_TRY(fl_vec(f.TangentOnRst2()));GA_TRY(fl_vec2d(f.Tangent2dOnRst2()));
 ga_pnt(f.PointOnRst1());ga_pnt(f.PointOnRst2());ga_pnt2d(f.Pnt2dOnRst1());ga_pnt2d(f.Pnt2dOnRst2());
 // The evolutive class leaves the restriction parameters uninitialized until a solution.
 if(solved){ga_real(f.ParameterOnRst1());ga_real(f.ParameterOnRst2());}
 GA_TRY(gp_Vec a,b,c,e;ga_int(int(f.Decroch(x,a,b,c,e)));fl_vec(a);fl_vec(b);fl_vec(c);fl_vec(e));
 GA_TRY(gp_Pnt ctr;gp_Vec med;ga_flag(f.CenterCircleRst1Rst2(f.PointOnRst1(),f.PointOnRst2(),gp_Vec(1,0,0),ctr,med));ga_pnt(ctr);fl_vec(med));
 GA_TRY(double pd,pf;gp_Circ k;f.Section(t,x(1),x(2),pd,pf,k);ga_real(pd);ga_real(pf);
        ga_pnt(k.Location());ga_dir(k.Axis().Direction());ga_dir(k.XAxis().Direction());ga_real(k.Radius()));
 fl_sections_at(f,Blend_Point(gp_Pnt(),gp_Pnt(),t,0.,0.,0.,0.,x(1),x(2)));
 fl_app_queries(f,tol);}

template<class F> void fl_rst_rst_run(F& f,const occ::handle<Adaptor3d_Surface> s[2],const occ::handle<Adaptor2d_Curve2d> rst[2],
                                      int shape,double t,bool trim,double first,double last,const math_Vector& x,double tol){
 // myTConv is uninitialized until GetShape meets a non-linear shape: set it first.
 {int a,b,c,d;f.Set(BlendFunc_Rational);f.GetShape(a,b,c,d);}
 f.Set(BlendFunc_SectionShape(shape));f.Set(s[0],rst[0],s[1],rst[1]);
 if(trim)GA_TRY(f.Set(first,last));
 GA_TRY(f.Set(t));
 fl_rst_rst_members(f,x,t,tol);}

// `fl_rst_rst kind S1 Rst1 S2 Rst2 C Law radius choix shape t trim first last X1 X2 tol`:
// kind 0 RstRstConstRad, 1 RstRstEvolRad; the references of Decroch are the restrictions.
void fl_rst_rst_case(std::istream& in){
 GaReader r{in};int kind=r.integer();
 occ::handle<Adaptor3d_Surface> s[2];occ::handle<Adaptor2d_Curve2d> rst[2];
 s[0]=fl_surface(r);rst[0]=fl_curve2d(r);s[1]=fl_surface(r);rst[1]=fl_curve2d(r);
 occ::handle<Adaptor3d_Curve> c=fl_guide(r);occ::handle<Law_Function> law=fl_law(r);
 double radius=r.real();int choix=r.integer(),shape=r.integer();double t=r.real();bool trim=r.flag();
 double first=r.real(),last=r.real();math_Vector x(1,2);x(1)=r.real();x(2)=r.real();double tol=r.real();
 if(kind==0){BRepBlend_RstRstConstRad f(s[0],rst[0],s[1],rst[1],c);f.Set(radius,choix);fl_rst_rst_run(f,s,rst,shape,t,trim,first,last,x,tol);}
 else{BRepBlend_RstRstEvolRad f(s[0],rst[0],s[1],rst[1],c,law);f.Set(choix);fl_rst_rst_run(f,s,rst,shape,t,trim,first,last,x,tol);}
 ga_end();}

// The surface of an approximation: its shape, poles, weights, knots and errors.
template<class A> void fl_approx(const A& a){
 ga_flag(a.IsDone());if(!a.IsDone())return;
 GA_TRY(int ud,vd,nu,nv,ku,kv;a.SurfShape(ud,vd,nu,nv,ku,kv);ga_int(ud);ga_int(vd);ga_int(nu);ga_int(nv);ga_int(ku);ga_int(kv));
 GA_TRY(const NCollection_Array2<gp_Pnt>& p=a.SurfPoles();for(int i=p.LowerRow();i<=p.UpperRow();++i)for(int j=p.LowerCol();j<=p.UpperCol();++j)ga_pnt(p(i,j)));
 GA_TRY(const NCollection_Array2<double>& w=a.SurfWeights();for(int i=w.LowerRow();i<=w.UpperRow();++i)for(int j=w.LowerCol();j<=w.UpperCol();++j)ga_real(w(i,j)));
 GA_TRY(fl_reals(a.SurfUKnots()));GA_TRY(fl_reals(a.SurfVKnots()));GA_TRY(fl_reals(a.SurfUMults()));GA_TRY(fl_reals(a.SurfVMults()));
 GA_TRY(ga_int(a.NbCurves2d()));
 GA_TRY(for(int k=1;k<=a.NbCurves2d();++k)fl_pnt2ds(a.Curve2dPoles(k)));
 GA_TRY(double t3,t2;a.TolReached(t3,t2);ga_real(t3);ga_real(t2));}

// `fl_app_func radius choix shape npts t0 dt tparam tol3d tol2d continuity`: the
// constant-radius fillet of the right-angled planes z = 0 and y = 0 along the X axis, its
// exact line of npts points (t, R, t, -R), the sweep function at tparam, the sweep
// approximation (BRepBlend_AppSurface) and the AppBlend one (BRepBlend_AppSurf).
void fl_app_func_case(std::istream& in){
 GaReader r{in};
 double radius=r.real();int choix=r.integer(),shape=r.integer(),npts=r.integer();
 double t0=r.real(),dt=r.real(),tparam=r.real(),tol3d=r.real(),tol2d=r.real();int cont=r.integer();
 occ::handle<Adaptor3d_Surface> s1=new GeomAdaptor_Surface(new Geom_Plane(gp_Ax3(gp_Pnt(0,0,0),gp_Dir(0,0,1),gp_Dir(1,0,0))));
 occ::handle<Adaptor3d_Surface> s2=new GeomAdaptor_Surface(new Geom_Plane(gp_Ax3(gp_Pnt(0,0,0),gp_Dir(0,1,0),gp_Dir(1,0,0))));
 occ::handle<Adaptor3d_Curve> c=new GeomAdaptor_Curve(new Geom_Line(gp_Pnt(0,0,0),gp_Dir(1,0,0)));
 BlendFunc_ConstRad f(s1,s2,c);f.Set(radius,choix);f.Set(BlendFunc_SectionShape(shape));
 occ::handle<BRepBlend_Line> line=new BRepBlend_Line();
 for(int i=0;i<npts;++i){double t=t0+i*dt;line->Append(Blend_Point(gp_Pnt(t,radius,0),gp_Pnt(t,0,radius),t,t,radius,t,-radius));}
 GA_TRY({
  occ::handle<BRepBlend_AppFunc> af=new BRepBlend_AppFunc(line,f,tol3d,tol2d);
  int np=0,nk=0,deg=0;af->SectionShape(np,nk,deg);int n2=af->Nb2dCurves();
  ga_int(np);ga_int(nk);ga_int(deg);ga_int(n2);ga_flag(af->IsRational());ga_pnt(af->BarycentreOfSurf());
  GA_TRY(ga_real(af->MaximalSection()));GA_TRY(ga_int(af->NbIntervals(GeomAbs_C1)));
  NCollection_Array1<gp_Pnt> poles(1,np);NCollection_Array1<gp_Pnt2d> p2(1,n2);NCollection_Array1<double> w(1,np),dw(1,np);
  NCollection_Array1<gp_Vec> dp(1,np);NCollection_Array1<gp_Vec2d> dp2(1,n2);w.Init(0);dw.Init(0);
  GA_TRY(ga_flag(af->D0(tparam,t0,t0,poles,p2,w));fl_pnts(poles);fl_pnt2ds(p2);fl_reals(w));
  GA_TRY(ga_flag(af->D1(tparam,t0,t0,poles,dp,p2,dp2,w,dw));fl_pnts(poles);fl_vecs(dp);fl_pnt2ds(p2);fl_vec2ds(dp2);fl_reals(w);fl_reals(dw));
  ga_int(line->NbPoints());
  NCollection_Array1<double> tol(1,np);tol.Init(0);GA_TRY(af->GetTolerance(1.e-4,1.e-5,1.e-3,tol);fl_reals(tol));
  double first=t0,last=t0+(npts-1)*dt;
  GA_TRY(const GeomAbs_Shape conts[3]={GeomAbs_C0,GeomAbs_C1,GeomAbs_C2};BRepBlend_AppSurface app(af,first,last,tol3d,tol2d,1.e-2,conts[cont],11,50);fl_approx(app));
 });
 GA_TRY(BRepBlend_AppSurf app(3,8,tol3d,tol2d,5);app.Perform(line,f,true);fl_approx(app));
 ga_end();}
