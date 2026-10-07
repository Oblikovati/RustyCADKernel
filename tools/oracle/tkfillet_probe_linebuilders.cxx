// BRepBlend line builder probes of tools/oracle/tkfillet_probe.cxx (included after the
// walking probe, whose printers they use); the Rust twin is
// crates/tkfillet/examples/fl_probe/line_builders.rs.
#include <Adaptor3d_CurveOnSurface.hxx>
#include <BRepBlend_RstRstLineBuilder.hxx>
#include <BRepBlend_SurfRstLineBuilder.hxx>

// A 2d curve of fl_curve2d_kind on the range [first, last].
occ::handle<Adaptor2d_Curve2d> fl_trimmed_curve2d(GaReader& r){
 occ::handle<Geom2dAdaptor_Curve> c=occ::down_cast<Geom2dAdaptor_Curve>(fl_curve2d_kind(r));
 double first=r.real(),last=r.real();
 return new Geom2dAdaptor_Curve(c->Curve(),first,last);}

// The line of a builder: the points with their restriction parameters, the extremities
// and the transitions.
void fl_builder_line(const occ::handle<BRepBlend_Line>& line){
 ga_int(line->NbPoints());
 for(int i=1;i<=line->NbPoints();++i){const Blend_Point& p=line->Point(i);fl_blend_point(p);
  GA_TRY(ga_real(p.ParameterOnC1()));GA_TRY(ga_real(p.ParameterOnC2()));}
 fl_extremity(line->StartPointOnFirst());fl_extremity(line->StartPointOnSecond());
 fl_extremity(line->EndPointOnFirst());fl_extremity(line->EndPointOnSecond());
 GA_TRY(ga_int(int(line->TransitionOnS1())));GA_TRY(ga_int(int(line->TransitionOnS2())));}

void fl_surf_rst_walk_state(const BRepBlend_SurfRstLineBuilder& b){
 ga_flag(b.IsDone());ga_flag(b.DecrochStart());ga_flag(b.DecrochEnd());
 if(b.IsDone())fl_builder_line(b.Line());}

// `fl_surf_rst_walk mode S1 U1 U2 V1 V2 S2 U1 U2 V1 V2 Rst(kind a b c d first last)
// Sref PCref(kind a b c d first last) G first last nv (P D)* savefirst savelast radius
// choix choixinv pdep pmax maxstep tol3d tol2d tolguide fleche appro X1..X3 recrst recp
// recs pextra`: the marching of BRepBlend_SurfRstConstRad between the trimmed surface S1
// and the restriction Rst of the trimmed surface S2 (Decroch against Sref and PCref),
// with the inverses of ChFi3d_FilBuilder (SurfCurvConstRadInv on the curve on surface,
// SurfPointConstRadInv, ConstRadInv on Sref). Mode 0 Perform, 1 then Complete(pextra), 2
// then ArcToRecadre at the last point, 3 PerformFirstSection then Perform from it to
// pextra.
void fl_surf_rst_walk_case(std::istream& in){
 GaReader r{in};int mode=r.integer();
 occ::handle<Adaptor3d_Surface> s1=fl_trimmed_surface(r),s2=fl_trimmed_surface(r);
 occ::handle<Adaptor2d_Curve2d> rst=fl_trimmed_curve2d(r);
 occ::handle<Adaptor3d_Surface> sref=fl_surface(r);occ::handle<Adaptor2d_Curve2d> pcref=fl_trimmed_curve2d(r);
 occ::handle<ChFiDS_ElSpine> spine=fl_el_spine(r);
 double radius=r.real();int choix=r.integer(),choixinv=r.integer();
 double pdep=r.real(),pmax=r.real(),maxstep=r.real(),tol3d=r.real(),tol2d=r.real(),tolguide=r.real(),fleche=r.real();
 bool appro=r.flag();math_Vector x(1,3);for(int i=1;i<=3;++i)x(i)=r.real();
 bool recrst=r.flag(),recp=r.flag(),recs=r.flag();double pextra=r.real();
 occ::handle<Adaptor3d_TopolTool> d1=new Adaptor3d_TopolTool(s1),d2=new Adaptor3d_TopolTool(s2);
 BRepBlend_SurfRstConstRad f(s1,s2,rst,spine);f.Set(sref,pcref);f.Set(radius,choix);
 occ::handle<Adaptor3d_CurveOnSurface> hc=new Adaptor3d_CurveOnSurface();hc->Load(rst,s2);
 BRepBlend_SurfCurvConstRadInv finvc(s1,hc,spine);finvc.Set(radius,choix);
 BRepBlend_SurfPointConstRadInv finvp(s1,spine);finvp.Set(radius,choix);
 BlendFunc_ConstRadInv finv(s1,sref,spine);finv.Set(false,pcref);finv.Set(radius,choixinv);
 BRepBlend_SurfRstLineBuilder b(s1,d1,s2,rst,d2);
 GA_TRY(b.Line());
 if(mode==3){
  double psol=0;math_Vector sol(1,3);sol.Init(0);bool ok=false;
  GA_TRY(ok=b.PerformFirstSection(f,finv,finvp,finvc,pdep,pmax,x,tol3d,tol2d,tolguide,recrst,recp,recs,psol,sol);
         ga_flag(ok);ga_real(psol);fl_vector(sol));
  fl_surf_rst_walk_state(b);
  if(ok)GA_TRY(b.Perform(f,finv,finvp,finvc,psol,pextra,maxstep,tol3d,tol2d,tolguide,sol,fleche,false));
  fl_surf_rst_walk_state(b);ga_end();return;}
 GA_TRY(b.Perform(f,finv,finvp,finvc,pdep,pmax,maxstep,tol3d,tol2d,tolguide,x,fleche,appro));
 fl_surf_rst_walk_state(b);
 if(mode==1){GA_TRY(ga_flag(b.Complete(f,finv,finvp,finvc,pextra)));fl_surf_rst_walk_state(b);}
 if(mode==2&&b.IsDone()){
  const Blend_Point& p=b.Line()->Point(b.Line()->NbPoints());double u,v;p.ParametersOnS(u,v);
  math_Vector s(1,3);s(1)=u+pextra;s(2)=v+pextra;s(3)=p.ParameterOnC();
  gp_Pnt2d a,c;double pon=0;GA_TRY(ga_int(b.ArcToRecadre(s,0,a,c,pon));ga_pnt2d(a);ga_pnt2d(c);ga_real(pon));}
 ga_end();}

void fl_rst_rst_walk_state(const BRepBlend_RstRstLineBuilder& b){
 ga_flag(b.IsDone());ga_flag(b.Decroch1Start());ga_flag(b.Decroch2Start());ga_flag(b.Decroch1End());ga_flag(b.Decroch2End());
 if(b.IsDone())fl_builder_line(b.Line());}

// `fl_rst_rst_walk mode S1 U1 U2 V1 V2 Rst1(kind a b c d first last) S2 U1 U2 V1 V2
// Rst2 Sref1 PCref1 Sref2 PCref2 G first last nv (P D)* savefirst savelast radius choix
// ch1 ch2 pdep pmax maxstep tol3d tolguide fleche appro X1 X2 recrst1 recp1 recrst2 recp2
// pextra`: the marching of BRepBlend_RstRstConstRad between the restrictions Rst1 and Rst2
// of the trimmed surfaces S1 and S2 (Decroch against the reference faces), with the
// inverses of ChFi3d_FilBuilder. Mode 0 Perform, 1 then Complete(pextra), 3
// PerformFirstSection then Perform from it to pextra.
void fl_rst_rst_walk_case(std::istream& in){
 GaReader r{in};int mode=r.integer();
 occ::handle<Adaptor3d_Surface> s1=fl_trimmed_surface(r);occ::handle<Adaptor2d_Curve2d> rst1=fl_trimmed_curve2d(r);
 occ::handle<Adaptor3d_Surface> s2=fl_trimmed_surface(r);occ::handle<Adaptor2d_Curve2d> rst2=fl_trimmed_curve2d(r);
 occ::handle<Adaptor3d_Surface> sref1=fl_surface(r);occ::handle<Adaptor2d_Curve2d> pcref1=fl_trimmed_curve2d(r);
 occ::handle<Adaptor3d_Surface> sref2=fl_surface(r);occ::handle<Adaptor2d_Curve2d> pcref2=fl_trimmed_curve2d(r);
 occ::handle<ChFiDS_ElSpine> spine=fl_el_spine(r);
 double radius=r.real();int choix=r.integer(),ch1=r.integer(),ch2=r.integer();
 double pdep=r.real(),pmax=r.real(),maxstep=r.real(),tol3d=r.real(),tolguide=r.real(),fleche=r.real();
 bool appro=r.flag();math_Vector x(1,2);x(1)=r.real();x(2)=r.real();
 bool recrst1=r.flag(),recp1=r.flag(),recrst2=r.flag(),recp2=r.flag();double pextra=r.real();
 occ::handle<Adaptor3d_TopolTool> d1=new Adaptor3d_TopolTool(s1),d2=new Adaptor3d_TopolTool(s2);
 BRepBlend_RstRstConstRad f(s1,rst1,s2,rst2,spine);f.Set(sref1,pcref1,sref2,pcref2);f.Set(radius,choix);
 occ::handle<Adaptor3d_CurveOnSurface> hc1=new Adaptor3d_CurveOnSurface();hc1->Load(rst1,s1);
 occ::handle<Adaptor3d_CurveOnSurface> hc2=new Adaptor3d_CurveOnSurface();hc2->Load(rst2,s2);
 BRepBlend_SurfCurvConstRadInv finv1(sref1,hc2,spine);BRepBlend_CurvPointRadInv finvp1(spine,hc2);
 BRepBlend_SurfCurvConstRadInv finv2(sref2,hc1,spine);BRepBlend_CurvPointRadInv finvp2(spine,hc1);
 finv1.Set(pcref1);finv2.Set(pcref2);finv1.Set(radius,ch1);finvp1.Set(choix);finv2.Set(radius,ch2);finvp2.Set(choix);
 BRepBlend_RstRstLineBuilder b(s1,rst1,d1,s2,rst2,d2);
 GA_TRY(b.Line());
 if(mode==3){
  double psol=0;math_Vector sol(1,2);sol.Init(0);bool ok=false;
  GA_TRY(ok=b.PerformFirstSection(f,finv1,finvp1,finv2,finvp2,pdep,pmax,x,tol3d,tolguide,recrst1,recp1,recrst2,recp2,psol,sol);
         ga_flag(ok);ga_real(psol);fl_vector(sol));
  fl_rst_rst_walk_state(b);
  if(ok)GA_TRY(b.Perform(f,finv1,finvp1,finv2,finvp2,psol,pextra,maxstep,tol3d,tolguide,sol,fleche,false));
  fl_rst_rst_walk_state(b);ga_end();return;}
 GA_TRY(b.Perform(f,finv1,finvp1,finv2,finvp2,pdep,pmax,maxstep,tol3d,tolguide,x,fleche,appro));
 fl_rst_rst_walk_state(b);
 if(mode==1){GA_TRY(ga_flag(b.Complete(f,finv1,finvp1,finv2,finvp2,pextra)));fl_rst_rst_walk_state(b);}
 ga_end();}
