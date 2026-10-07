// BRepBlend_Walking probe of tools/oracle/tkfillet_probe.cxx (included after the BRepBlend
// probes, whose readers it uses); the Rust twin is crates/tkfillet/examples/fl_probe/walking.rs.
#include <Adaptor3d_TopolTool.hxx>
#include <BRepBlend_Walking.hxx>
#include <BlendFunc_ConstRadInv.hxx>
#include <ChFiDS_ElSpine.hxx>
#include <IntSurf_Transition.hxx>

void fl_transition(const IntSurf_Transition& t){
 ga_int(int(t.TransitionType()));GA_TRY(ga_flag(t.IsTangent()));
 GA_TRY(ga_int(int(t.Situation())));GA_TRY(ga_flag(t.IsOpposite()));}

void fl_extremity(const BRepBlend_Extremity& e){
 ga_pnt(e.Value());double u,v;e.Parameters(u,v);ga_real(u);ga_real(v);
 ga_real(e.Parameter());ga_real(e.ParameterOnGuide());ga_real(e.Tolerance());
 ga_flag(e.IsVertex());ga_flag(e.HasTangent());GA_TRY(fl_vec(e.Tangent()));
 ga_int(e.NbPointOnRst());
 for(int i=1;i<=e.NbPointOnRst();++i){const BRepBlend_PointOnRst& p=e.PointOnRst(i);
  ga_real(p.ParameterOnArc());fl_transition(p.TransitionOnLine());fl_transition(p.TransitionOnArc());}}

void fl_blend_point(const Blend_Point& p){
 ga_real(p.Parameter());ga_pnt(p.PointOnS1());ga_pnt(p.PointOnS2());ga_flag(p.IsTangencyPoint());
 GA_TRY(double u,v;p.ParametersOnS1(u,v);ga_real(u);ga_real(v));
 GA_TRY(double u,v;p.ParametersOnS2(u,v);ga_real(u);ga_real(v));
 if(p.IsTangencyPoint())return;
 fl_vec(p.TangentOnS1());fl_vec(p.TangentOnS2());fl_vec2d(p.Tangent2dOnS1());fl_vec2d(p.Tangent2dOnS2());}

// The walking state: done, twists, and the line (points, extremities, transitions).
void fl_walk(const BRepBlend_Walking& w){
 ga_flag(w.IsDone());ga_flag(w.TwistOnS1());ga_flag(w.TwistOnS2());
 if(!w.IsDone())return;
 const occ::handle<BRepBlend_Line>& line=w.Line();
 ga_int(line->NbPoints());
 for(int i=1;i<=line->NbPoints();++i)fl_blend_point(line->Point(i));
 fl_extremity(line->StartPointOnFirst());fl_extremity(line->StartPointOnSecond());
 fl_extremity(line->EndPointOnFirst());fl_extremity(line->EndPointOnSecond());
 GA_TRY(ga_int(int(line->TransitionOnS1())));GA_TRY(ga_int(int(line->TransitionOnS2())));}

// A trimmed surface (fl_surface then U1 U2 V1 V2).
occ::handle<Adaptor3d_Surface> fl_trimmed_surface(GaReader& r){
 occ::handle<GeomAdaptor_Surface> s=occ::down_cast<GeomAdaptor_Surface>(fl_surface(r));
 double u1=r.real(),u2=r.real(),v1=r.real(),v2=r.real();
 return new GeomAdaptor_Surface(s->Surface(),u1,u2,v1,v2);}

// The guide: fl_guide's curve, its bounds, its vertices with tangents and the saved bounds.
occ::handle<ChFiDS_ElSpine> fl_el_spine(GaReader& r){
 occ::handle<GeomAdaptor_Curve> c=occ::down_cast<GeomAdaptor_Curve>(fl_guide(r));
 occ::handle<ChFiDS_ElSpine> spine=new ChFiDS_ElSpine();spine->SetCurve(c->Curve());
 spine->FirstParameter(r.real());spine->LastParameter(r.real());
 int nv=r.integer();for(int i=0;i<nv;++i){gp_Pnt p=r.pnt();spine->AddVertexWithTangent(gp_Ax1(p,r.dir()));}
 if(r.flag())spine->SaveFirstParameter();
 if(r.flag())spine->SaveLastParameter();
 return spine;}

// `fl_walking mode S1 U1 U2 V1 V2 S2 U1 U2 V1 V2 G first last nv (P D)* savefirst savelast
// radius choix pdep pmax maxstep tol3d tolguide fleche appro X1..X4 cls1 cls2 check2d check
// pextra njalons (t u1 v1 u2 v2)*`: the walking of BlendFunc_ConstRad between the two
// trimmed surfaces (domains Adaptor3d_TopolTool) along the guide. Mode 0 Perform, 1 then
// Complete(pextra), 2 then Continu(pextra), 3 then Continu(pextra, OnS1), 4 then
// Continu(pextra, OnS2), 5 PerformFirstSection(F, ...), 6 PerformFirstSection(F, FInv,
// ...) on both restrictions then Perform from its section.
void fl_walking_case(std::istream& in){
 GaReader r{in};int mode=r.integer();
 occ::handle<Adaptor3d_Surface> s1=fl_trimmed_surface(r),s2=fl_trimmed_surface(r);
 occ::handle<ChFiDS_ElSpine> spine=fl_el_spine(r);
 double radius=r.real();int choix=r.integer();
 double pdep=r.real(),pmax=r.real(),maxstep=r.real(),tol3d=r.real(),tolguide=r.real(),fleche=r.real();
 bool appro=r.flag();math_Vector x(1,4);for(int i=1;i<=4;++i)x(i)=r.real();
 bool cls1=r.flag(),cls2=r.flag(),chk2d=r.flag(),chk=r.flag();double pextra=r.real();
 occ::handle<Adaptor3d_TopolTool> d1=new Adaptor3d_TopolTool(s1),d2=new Adaptor3d_TopolTool(s2);
 BlendFunc_ConstRad f(s1,s2,spine);f.Set(radius,choix);
 BlendFunc_ConstRadInv finv(s1,s2,spine);finv.Set(radius,choix);
 BRepBlend_Walking w(s1,s2,d1,d2,spine);
 int nj=r.integer();
 for(int i=0;i<nj;++i){double t=r.real(),u1=r.real(),v1=r.real(),u2=r.real(),v2=r.real();
  w.AddSingularPoint(Blend_Point(s1->Value(u1,v1),s2->Value(u2,v2),t,u1,v1,u2,v2));}
 w.ClassificationOnS1(cls1);w.ClassificationOnS2(cls2);w.Check2d(chk2d);w.Check(chk);
 GA_TRY(w.Line());
 if(mode==5){
  TopAbs_State p1,p2;
  GA_TRY(ga_flag(w.PerformFirstSection(f,pdep,x,tol3d,tolguide,p1,p2));ga_int(int(p1));ga_int(int(p2));fl_vector(x));
  ga_end();return;}
 if(mode==6){
  double psol=0;math_Vector sol(1,4);sol.Init(0);bool ok=false;
  GA_TRY(ok=w.PerformFirstSection(f,finv,pdep,pmax,x,tol3d,tolguide,true,true,psol,sol);ga_flag(ok);ga_real(psol);fl_vector(sol));
  if(ok)GA_TRY(w.Perform(f,finv,psol,pextra,maxstep,tol3d,tolguide,sol,fleche,false));
  fl_walk(w);ga_end();return;}
 GA_TRY(w.Perform(f,finv,pdep,pmax,maxstep,tol3d,tolguide,x,fleche,appro));
 fl_walk(w);
 if(mode==1)GA_TRY(ga_flag(w.Complete(f,finv,pextra)));
 if(mode==2)GA_TRY(ga_flag(w.Continu(f,finv,pextra)));
 if(mode==3)GA_TRY(ga_flag(w.Continu(f,finv,pextra,true)));
 if(mode==4)GA_TRY(ga_flag(w.Continu(f,finv,pextra,false)));
 if(mode>=1&&mode<=4)fl_walk(w);
 ga_end();}
