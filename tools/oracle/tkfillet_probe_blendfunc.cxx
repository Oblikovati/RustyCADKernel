// BlendFunc probes of the TKFillet oracle (included by tkfillet_probe.cxx): every member
// of the two-surface blend functions on elementary surfaces along a line or a circle.
// The Rust twin is crates/tkfillet/examples/fl_probe/blend_func.rs.
#include <BlendFunc.hxx>
#include <BlendFunc_ConstRad.hxx>
#include <BlendFunc_ConstRadInv.hxx>
#include <BlendFunc_EvolRad.hxx>
#include <BlendFunc_EvolRadInv.hxx>
#include <BlendFunc_ChAsym.hxx>
#include <BlendFunc_ChAsymInv.hxx>
#include <BlendFunc_ChamfInv.hxx>
#include <BlendFunc_Chamfer.hxx>
#include <BlendFunc_ConstThroat.hxx>
#include <BlendFunc_ConstThroatInv.hxx>
#include <BlendFunc_ConstThroatWithPenetration.hxx>
#include <BlendFunc_ConstThroatWithPenetrationInv.hxx>
#include <gp_Lin.hxx>
#include <memory>
#include <Law_Constant.hxx>
#include <Law_Interpol.hxx>
#include <Law_Linear.hxx>
#include <Law_S.hxx>
#include <Geom2dAdaptor_Curve.hxx>
#include <Geom2d_Line.hxx>
#include <Blend_Point.hxx>
#include <GeomAdaptor_Curve.hxx>
#include <GeomAdaptor_Surface.hxx>
#include <Geom_Circle.hxx>
#include <Geom_ConicalSurface.hxx>
#include <Geom_CylindricalSurface.hxx>
#include <Geom_Line.hxx>
#include <Geom_Plane.hxx>
#include <Geom_SphericalSurface.hxx>
#include <Geom_ToroidalSurface.hxx>
#include <gp_Ax1.hxx>
#include <gp_Ax3.hxx>
#include <gp_Circ.hxx>
#include <gp_Vec2d.hxx>
#include <math_Matrix.hxx>
#include <math_Vector.hxx>

void fl_vec2d(const gp_Vec2d& v){ga_real(v.X());ga_real(v.Y());}

// A surface spec: kind (0 plane, 1 cylinder, 2 sphere, 3 cone, 4 torus), the frame P N X
// and two reals (radii or angle and radius); 12 numbers.
occ::handle<Adaptor3d_Surface> fl_surface(GaReader& r){
 int kind=r.integer();gp_Pnt p=r.pnt();gp_Dir n=r.dir(),x=r.dir();double a=r.real(),b=r.real();
 gp_Ax3 frame(p,n,x);occ::handle<Geom_Surface> s;
 switch(kind){
  case 0:s=new Geom_Plane(frame);break;
  case 1:s=new Geom_CylindricalSurface(frame,a);break;
  case 2:s=new Geom_SphericalSurface(frame,a);break;
  case 3:s=new Geom_ConicalSurface(frame,a,b);break;
  default:s=new Geom_ToroidalSurface(frame,a,b);break;}
 return new GeomAdaptor_Surface(s);}

// A guide spec: kind (0 line through P along N, 1 circle of frame P N X and radius a), the
// frame and one real; 11 numbers.
occ::handle<Adaptor3d_Curve> fl_guide(GaReader& r){
 int kind=r.integer();gp_Pnt p=r.pnt();gp_Dir n=r.dir(),x=r.dir();double a=r.real();
 occ::handle<Geom_Curve> c;
 if(kind==0)c=new Geom_Line(p,n);else c=new Geom_Circle(gp_Ax2(p,n,x),a);
 return new GeomAdaptor_Curve(c);}

void fl_vector(const math_Vector& v){for(int i=v.Lower();i<=v.Upper();++i)ga_real(v(i));}
void fl_matrix(const math_Matrix& m){
 for(int i=m.LowerRow();i<=m.UpperRow();++i)for(int j=m.LowerCol();j<=m.UpperCol();++j)ga_real(m(i,j));}
template<class T> void fl_reals(const NCollection_Array1<T>& a){for(int i=a.Lower();i<=a.Upper();++i)ga_real(double(a(i)));}
void fl_pnts(const NCollection_Array1<gp_Pnt>& a){for(int i=a.Lower();i<=a.Upper();++i)ga_pnt(a(i));}
void fl_vecs(const NCollection_Array1<gp_Vec>& a){for(int i=a.Lower();i<=a.Upper();++i)fl_vec(a(i));}
void fl_pnt2ds(const NCollection_Array1<gp_Pnt2d>& a){for(int i=a.Lower();i<=a.Upper();++i)ga_pnt2d(a(i));}
void fl_vec2ds(const NCollection_Array1<gp_Vec2d>& a){for(int i=a.Lower();i<=a.Upper();++i)fl_vec2d(a(i));}

// The tangent queries of a Blend_Function after IsSolution.
void fl_tangents(Blend_Function& f){
 ga_flag(f.IsTangencyPoint());
 GA_TRY(fl_vec(f.TangentOnS1()));GA_TRY(fl_vec2d(f.Tangent2dOnS1()));
 GA_TRY(fl_vec(f.TangentOnS2()));GA_TRY(fl_vec2d(f.Tangent2dOnS2()));
 GA_TRY(ga_flag(f.TwistOnS1()));GA_TRY(ga_flag(f.TwistOnS2()));}

// The sections of every order at the point X of parameter t, arrays sized by GetShape.
void fl_sections(Blend_AppFunction& f,const math_Vector& x,double t){
 int np=0,nk=0,deg=0,np2=0;GA_TRY(f.GetShape(np,nk,deg,np2);ga_int(np);ga_int(nk);ga_int(deg);ga_int(np2));
 if(np<1||np>64||nk<1||nk>64)return;
 NCollection_Array1<double> knots(1,nk),weights(1,np),dw(1,np),d2w(1,np);
 NCollection_Array1<int> mults(1,nk);
 knots.Init(0);mults.Init(0);weights.Init(0);dw.Init(0);d2w.Init(0);
 GA_TRY(f.Knots(knots);fl_reals(knots));GA_TRY(f.Mults(mults);fl_reals(mults));
 GA_TRY(f.GetMinimalWeight(weights);fl_reals(weights));
 math_Vector tol3d(1,np),tol1d(1,np2);tol3d.Init(0);tol1d.Init(0);
 GA_TRY(f.GetTolerance(1.e-4,1.e-5,1.e-3,tol3d,tol1d);fl_vector(tol3d);fl_vector(tol1d));
 Blend_Point p(gp_Pnt(),gp_Pnt(),t,x(1),x(2),x(3),x(4));
 NCollection_Array1<gp_Pnt> poles(1,np);NCollection_Array1<gp_Vec> dp(1,np),d2p(1,np);
 NCollection_Array1<gp_Pnt2d> p2(1,np2);NCollection_Array1<gp_Vec2d> dp2(1,np2),d2p2(1,np2);
 weights.Init(0);
 GA_TRY(f.Section(p,poles,p2,weights);fl_pnts(poles);fl_pnt2ds(p2);fl_reals(weights));
 GA_TRY(ga_flag(f.Section(p,poles,dp,p2,dp2,weights,dw));fl_pnts(poles);fl_vecs(dp);fl_pnt2ds(p2);
        fl_vec2ds(dp2);fl_reals(weights);fl_reals(dw));
 GA_TRY(ga_flag(f.Section(p,poles,dp,d2p,p2,dp2,d2p2,weights,dw,d2w));fl_pnts(poles);fl_vecs(dp);fl_vecs(d2p);
        fl_pnt2ds(p2);fl_vec2ds(dp2);fl_vec2ds(d2p2);fl_reals(weights);fl_reals(dw);fl_reals(d2w));}

// The bounds, tolerances, intervals and resolutions of a Blend_AppFunction.
void fl_app_queries(Blend_AppFunction& f,double tol){
 math_Vector inf(1,4),sup(1,4),tv(1,4);inf.Init(0);sup.Init(0);tv.Init(0);
 GA_TRY(f.GetBounds(inf,sup);fl_vector(inf);fl_vector(sup));
 GA_TRY(f.GetTolerance(tv,tol);fl_vector(tv));
 GA_TRY(int n=f.NbIntervals(GeomAbs_C1);ga_int(n);NCollection_Array1<double> iv(1,n+1);iv.Init(0);f.Intervals(iv,GeomAbs_C1);fl_reals(iv));
 GA_TRY(double u,v;f.Resolution(1,tol,u,v);ga_real(u);ga_real(v));
 GA_TRY(double u,v;f.Resolution(2,tol,u,v);ga_real(u);ga_real(v));
 GA_TRY(ga_real(f.GetMinimalDistance()));GA_TRY(ga_real(f.GetSectionSize()));GA_TRY(ga_flag(f.IsRational()));}

// The members a two-surface blend function shares, after Set(t), at the point X; `section`
// prints the section of the class (a circle for the fillets, a line for the chamfers).
template<class F,class S> void fl_blend_members(F& f,const math_Vector& x,double t,double tol,S section){
 math_Vector fv(1,4);math_Matrix d(1,4,1,4);fv.Init(0);d.Init(0);
 GA_TRY(ga_flag(f.Value(x,fv));fl_vector(fv));
 GA_TRY(ga_flag(f.Derivatives(x,d));fl_matrix(d));
 GA_TRY(ga_flag(f.Values(x,fv,d));fl_vector(fv);fl_matrix(d));
 GA_TRY(ga_flag(f.IsSolution(x,tol)));fl_tangents(f);
 GA_TRY(gp_Vec a,b,cc,dd;f.Tangent(x(1),x(2),x(3),x(4),a,b,cc,dd);fl_vec(a);fl_vec(b);fl_vec(cc);fl_vec(dd));
 GA_TRY(section());
 fl_sections(f,x,t);
 fl_app_queries(f,tol);}

// The circular section of a fillet function.
template<class F> void fl_fillet_members(F& f,const math_Vector& x,double t,double tol){
 fl_blend_members(f,x,t,tol,[&]{double pd,pf;gp_Circ k;f.Section(t,x(1),x(2),x(3),x(4),pd,pf,k);ga_real(pd);ga_real(pf);
        ga_pnt(k.Location());ga_dir(k.Axis().Direction());ga_dir(k.XAxis().Direction());ga_real(k.Radius());});}

// `fl_const_rad S1 S2 C radius choix shape t trim first last X1..X4 tol`.
void fl_const_rad_case(std::istream& in){
 GaReader r{in};
 occ::handle<Adaptor3d_Surface> s1=fl_surface(r),s2=fl_surface(r);occ::handle<Adaptor3d_Curve> c=fl_guide(r);
 double radius=r.real();int choix=r.integer(),shape=r.integer();double t=r.real();bool trim=r.flag();
 double first=r.real(),last=r.real();math_Vector x(1,4);for(int i=1;i<=4;++i)x(i)=r.real();double tol=r.real();
 BlendFunc_ConstRad f(s1,s2,c);f.Set(radius,choix);f.Set(BlendFunc_SectionShape(shape));
 if(trim)GA_TRY(f.Set(first,last));
 f.Set(t);
 fl_fillet_members(f,x,t,tol);
 GA_TRY(gp_Ax1 ax=f.AxeRot(t);ga_pnt(ax.Location());ga_dir(ax.Direction()));
 ga_end();}

// A radius law: kind (0 constant r1 on [p1, p2], 1 linear, 2 S, 3 interpolation through
// (p1, r1), ((p1 + p2) / 2, r3), (p2, r2)) then p1 r1 p2 r2 r3 and one unused real; 7 numbers.
occ::handle<Law_Function> fl_law(GaReader& r){
 int kind=r.integer();double p1=r.real(),r1=r.real(),p2=r.real(),r2=r.real(),r3=r.real();r.real();
 if(kind==0){occ::handle<Law_Constant> l=new Law_Constant();l->Set(r1,p1,p2);return l;}
 if(kind==1){occ::handle<Law_Linear> l=new Law_Linear();l->Set(p1,r1,p2,r2);return l;}
 if(kind==2){occ::handle<Law_S> l=new Law_S();l->Set(p1,r1,p2,r2);return l;}
 NCollection_Array1<gp_Pnt2d> pts(1,3);pts(1)=gp_Pnt2d(p1,r1);pts(2)=gp_Pnt2d((p1+p2)/2,r3);pts(3)=gp_Pnt2d(p2,r2);
 occ::handle<Law_Interpol> l=new Law_Interpol();l->Set(pts,false);return l;}

// `fl_evol_rad S1 S2 C Law choix shape t trim first last X1..X4 tol`.
void fl_evol_rad_case(std::istream& in){
 GaReader r{in};
 occ::handle<Adaptor3d_Surface> s1=fl_surface(r),s2=fl_surface(r);occ::handle<Adaptor3d_Curve> c=fl_guide(r);
 occ::handle<Law_Function> law=fl_law(r);
 int choix=r.integer(),shape=r.integer();double t=r.real();bool trim=r.flag();
 double first=r.real(),last=r.real();math_Vector x(1,4);for(int i=1;i<=4;++i)x(i)=r.real();double tol=r.real();
 BlendFunc_EvolRad f(s1,s2,c,law);f.Set(choix);f.Set(BlendFunc_SectionShape(shape));
 if(trim)GA_TRY(f.Set(first,last));
 f.Set(t);
 fl_fillet_members(f,x,t,tol);
 ga_end();}

// A curve on surface: the line of the parametric plane through (u, v) along (du, dv).
occ::handle<Adaptor2d_Curve2d> fl_curve2d(GaReader& r){
 gp_Pnt2d p=r.pnt2d();gp_Dir2d d=r.dir2d();return new Geom2dAdaptor_Curve(new Geom2d_Line(p,d));}

// The math members and the bounds of an inverse function at X.
void fl_inverse(Blend_FuncInv& f,const math_Vector& x,double tol){
 math_Vector fv(1,4),inf(1,4),sup(1,4),tv(1,4);math_Matrix d(1,4,1,4);
 fv.Init(0);inf.Init(0);sup.Init(0);tv.Init(0);d.Init(0);
 ga_int(f.NbVariables());ga_int(f.NbEquations());
 GA_TRY(ga_flag(f.Value(x,fv));fl_vector(fv));
 GA_TRY(ga_flag(f.Derivatives(x,d));fl_matrix(d));
 GA_TRY(ga_flag(f.Values(x,fv,d));fl_vector(fv);fl_matrix(d));
 GA_TRY(ga_flag(f.IsSolution(x,tol)));
 GA_TRY(f.GetTolerance(tv,tol);fl_vector(tv));
 GA_TRY(f.GetBounds(inf,sup);fl_vector(inf);fl_vector(sup));}

// `fl_const_rad_inv S1 S2 C P2d D2d radius choix first X1..X4 tol`.
void fl_const_rad_inv_case(std::istream& in){
 GaReader r{in};
 occ::handle<Adaptor3d_Surface> s1=fl_surface(r),s2=fl_surface(r);occ::handle<Adaptor3d_Curve> c=fl_guide(r);
 occ::handle<Adaptor2d_Curve2d> cs=fl_curve2d(r);
 double radius=r.real();int choix=r.integer();bool first=r.flag();
 math_Vector x(1,4);for(int i=1;i<=4;++i)x(i)=r.real();double tol=r.real();
 BlendFunc_ConstRadInv f(s1,s2,c);f.Set(radius,choix);f.Set(first,cs);
 fl_inverse(f,x,tol);
 ga_end();}

// `fl_evol_rad_inv S1 S2 C P2d D2d Law choix first X1..X4 tol`.
void fl_evol_rad_inv_case(std::istream& in){
 GaReader r{in};
 occ::handle<Adaptor3d_Surface> s1=fl_surface(r),s2=fl_surface(r);occ::handle<Adaptor3d_Curve> c=fl_guide(r);
 occ::handle<Adaptor2d_Curve2d> cs=fl_curve2d(r);occ::handle<Law_Function> law=fl_law(r);
 int choix=r.integer();bool first=r.flag();
 math_Vector x(1,4);for(int i=1;i<=4;++i)x(i)=r.real();double tol=r.real();
 BlendFunc_EvolRadInv f(s1,s2,c,law);f.Set(choix);f.Set(first,cs);
 fl_inverse(f,x,tol);
 ga_end();}

// The linear section of a chamfer function.
template<class F> void fl_chamfer_members(F& f,const math_Vector& x,double t,double tol){
 fl_blend_members(f,x,t,tol,[&]{double pd,pf;gp_Lin l;f.Section(t,x(1),x(2),x(3),x(4),pd,pf,l);ga_real(pd);ga_real(pf);
        ga_pnt(l.Location());ga_dir(l.Direction());});}

template<class F> void fl_chamfer_run(F& f,double t,bool trim,double first,double last,const math_Vector& x,double tol){
 if(trim)GA_TRY(f.Set(first,last));
 f.Set(t);
 fl_chamfer_members(f,x,t,tol);}

// `fl_chamfer S1 S2 C kind d1 d2 choix t trim first last X1..X4 tol`: kind 0 Chamfer,
// 1 ConstThroat, 2 ConstThroatWithPenetration, 3 ChAsym (d2 is the angle).
void fl_chamfer_case(std::istream& in){
 GaReader r{in};
 occ::handle<Adaptor3d_Surface> s1=fl_surface(r),s2=fl_surface(r);occ::handle<Adaptor3d_Curve> c=fl_guide(r);
 int kind=r.integer();double d1=r.real(),d2=r.real();int choix=r.integer();double t=r.real();bool trim=r.flag();
 double first=r.real(),last=r.real();math_Vector x(1,4);for(int i=1;i<=4;++i)x(i)=r.real();double tol=r.real();
 if(kind==3){BlendFunc_ChAsym f(s1,s2,c);f.Set(d1,d2,choix);fl_chamfer_run(f,t,trim,first,last,x,tol);ga_end();return;}
 std::unique_ptr<BlendFunc_GenChamfer> f;
 if(kind==0)f.reset(new BlendFunc_Chamfer(s1,s2,c));
 else if(kind==1)f.reset(new BlendFunc_ConstThroat(s1,s2,c));
 else f.reset(new BlendFunc_ConstThroatWithPenetration(s1,s2,c));
 f->Set(d1,d2,choix);
 fl_chamfer_run(*f,t,trim,first,last,x,tol);
 ga_end();}

// `fl_chamfer_inv S1 S2 C P2d D2d kind d1 d2 choix first X1..X4 tol`: kind 0 ChamfInv,
// 1 ConstThroatInv, 2 ConstThroatWithPenetrationInv, 3 ChAsymInv (d2 is the angle).
void fl_chamfer_inv_case(std::istream& in){
 GaReader r{in};
 occ::handle<Adaptor3d_Surface> s1=fl_surface(r),s2=fl_surface(r);occ::handle<Adaptor3d_Curve> c=fl_guide(r);
 occ::handle<Adaptor2d_Curve2d> cs=fl_curve2d(r);
 int kind=r.integer();double d1=r.real(),d2=r.real();int choix=r.integer();bool first=r.flag();
 math_Vector x(1,4);for(int i=1;i<=4;++i)x(i)=r.real();double tol=r.real();
 if(kind==3){BlendFunc_ChAsymInv f(s1,s2,c);f.Set(d1,d2,choix);f.Set(first,cs);fl_inverse(f,x,tol);ga_end();return;}
 std::unique_ptr<BlendFunc_GenChamfInv> f;
 if(kind==0)f.reset(new BlendFunc_ChamfInv(s1,s2,c));
 else if(kind==1)f.reset(new BlendFunc_ConstThroatInv(s1,s2,c));
 else f.reset(new BlendFunc_ConstThroatWithPenetrationInv(s1,s2,c));
 f->Set(d1,d2,choix);f->Set(first,cs);
 fl_inverse(*f,x,tol);
 ga_end();}
