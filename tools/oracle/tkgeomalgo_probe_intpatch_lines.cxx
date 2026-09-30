// IntPatch line probe operations (tkgeomalgo_probe.cxx); the Rust twin is
// crates/tkgeomalgo/examples/ga_probe/int_patch_lines.rs. Each request builds one line
// (walking, restriction, conic or analytic) with its vertices, runs a command script of
// edits (vertices added, inserted, replaced, removed, first and last points,
// ComputeVertexParameters...) and prints the line. Needs tkgeomalgo_probe_intpatch.cxx.
#include <IntAna_IntQuadQuad.hxx>
#include <IntAna_Quadric.hxx>
#include <IntPatch_ALine.hxx>
#include <IntPatch_GLine.hxx>
#include <IntPatch_RLine.hxx>
#include <IntPatch_WLine.hxx>
#include <gp_Circ.hxx>
#include <gp_Cylinder.hxx>
#include <gp_Elips.hxx>
#include <gp_Hypr.hxx>
#include <gp_Lin.hxx>
#include <gp_Parab.hxx>
#include <gp_Sphere.hxx>
// The transitions of a line: `kind tang [a b]`, kind 0 undecided, 1 the IN/OUT types a b,
// 2 the TOUCH situations a b.
struct GaIpTrans{int kind;bool tang;int a;int b;};
GaIpTrans ga_ip_read_trans(GaReader& r){
 GaIpTrans t{r.integer(),r.flag(),0,0};if(t.kind!=0){t.a=r.integer();t.b=r.integer();}return t;}
template<class L,class... A> occ::handle<L> ga_ip_make(const GaIpTrans& t,const A&... args){
 if(t.kind==1)return new L(args...,t.tang,IntSurf_TypeTrans(t.a),IntSurf_TypeTrans(t.b));
 if(t.kind==2)return new L(args...,t.tang,IntSurf_Situation(t.a),IntSurf_Situation(t.b));
 return new L(args...,t.tang);}
void ga_ip_line_base(const occ::handle<IntPatch_Line>& l){
 ga_text(l->DynamicType()->Name());ga_int(l->ArcType());ga_flag(l->IsTangent());ga_int(l->TransitionOnS1());ga_int(l->TransitionOnS2());
 GA_TRY(ga_int(l->SituationS1()));GA_TRY(ga_int(l->SituationS2()));
 ga_flag(l->IsUIsoOnS1());ga_flag(l->IsVIsoOnS1());ga_flag(l->IsUIsoOnS2());ga_flag(l->IsVIsoOnS2());
 ga_flag(l->IsKind(STANDARD_TYPE(IntPatch_PointLine)));}
template<class L> void ga_ip_vertices(const GaIpFixture& fx,const occ::handle<L>& l){
 int n=l->NbVertex();ga_int(n);for(int i=1;i<=n;++i)ga_ip_point(fx,l->Vertex(i));
 ga_flag(l->HasFirstPoint());ga_flag(l->HasLastPoint());GA_TRY(ga_ip_point(fx,l->FirstPoint()));GA_TRY(ga_ip_point(fx,l->LastPoint()));
 GA_TRY(l->Vertex(0));GA_TRY(l->Vertex(n+1));}
// The points and the boxes of a point line, when it has points.
void ga_ip_points(const occ::handle<IntPatch_PointLine>& l){
 GA_TRY(int n=l->NbPnts();ga_int(n);for(int i=1;i<=n;++i)ga_iw_pnt2s(l->Point(i));GA_TRY(l->Point(0));GA_TRY(l->Point(n+1));
  ga_flag(l->IsOutSurf1Box(gp_Pnt2d(0.5,0.5)));ga_flag(l->IsOutSurf2Box(gp_Pnt2d(100,100)));
  ga_flag(l->IsOutBox(gp_Pnt(0,0,0)));ga_flag(l->IsOutBox(gp_Pnt(100,100,100))));
 ga_flag(l->Curve().IsNull());}
occ::handle<IntSurf_LineOn2S> ga_ip_read_line2s(GaReader& r){
 occ::handle<IntSurf_LineOn2S> c=new IntSurf_LineOn2S();int n=r.integer();
 for(int i=0;i<n;++i){gp_Pnt x=r.pnt();double a=r.real(),b=r.real(),e=r.real(),d=r.real();IntSurf_PntOn2S q;q.SetValue(x,a,b,e,d);c->Add(q);}
 return c;}
// The vertex commands shared by the point lines: 1 add (prepend flag, point), 3 replace,
// 4 first, 5 last, 6 compute, 7 remove, 9 clear, 12 iso flags, 14 parameter through
// ChangeVertex; returns false for another command.
template<class L> bool ga_ip_point_line_command(GaReader& r,const GaIpFixture& fx,const occ::handle<L>& l,int c){
 switch(c){
  case 1:{bool pre=r.flag();l->AddVertex(ga_ip_read_point(r,fx),pre);return true;}
  case 3:{int i=r.integer();l->Replace(i,ga_ip_read_point(r,fx));return true;}
  case 4:l->SetFirstPoint(r.integer());return true;
  case 5:l->SetLastPoint(r.integer());return true;
  case 6:l->ComputeVertexParameters(r.real());return true;
  case 7:l->RemoveVertex(r.integer());return true;
  case 9:l->ClearVertexes();return true;
  case 12:{bool a=r.flag(),b=r.flag(),e=r.flag();l->SetValue(a,b,e,r.flag());return true;}
  case 14:{int i=r.integer();double prm=r.real();l->ChangeVertex(i).SetParameter(prm);return true;}
  default:return false;
 }
}
// `dump trans creation-way pu1 pv1 pu2 pv2 npoints points nvertex vertex-scripts ncommands
// commands`: a walking line; the WLine commands add 2 set point, 8 insert before, 10/11
// the arcs, 13 purging; `dump` is the Dump mode, -1 for none.
void ga_ip_wline_case(std::istream& input){
 GaReader r{input};GaIpFixture fx;int dump=r.integer();GaIpTrans t=ga_ip_read_trans(r);int way=r.integer();
 double p1=r.real(),p2=r.real(),p3=r.real(),p4=r.real();occ::handle<IntSurf_LineOn2S> c=ga_ip_read_line2s(r);
 occ::handle<IntPatch_WLine> l=ga_ip_make<IntPatch_WLine>(t,c);l->SetCreatingWayInfo(IntPatch_WLine::IntPatch_WLType(way));
 l->SetPeriod(p1,p2,p3,p4);int nv=r.integer();for(int i=0;i<nv;++i)l->AddVertex(ga_ip_read_point(r,fx));
 int n=r.integer();
 for(int k=0;k<n;++k){int cmd=r.integer();GA_TRY(if(!ga_ip_point_line_command(r,fx,l,cmd)){
  if(cmd==2){int i=r.integer();l->SetPoint(i,ga_ip_read_point(r,fx));}
  else if(cmd==8){int i=r.integer();l->InsertVertexBefore(i,ga_ip_read_point(r,fx));}
  else if(cmd==10)l->SetArcOnS1(fx.arc(r.integer()));
  else if(cmd==11)l->SetArcOnS2(fx.arc(r.integer()));
  else l->EnablePurging(r.flag());});}
 ga_ip_line_base(l);ga_ip_points(l);ga_ip_vertices(fx,l);
 GA_TRY(int i=0;l->FirstPoint(i);ga_int(i));GA_TRY(int i=0;l->LastPoint(i);ga_int(i));
 ga_real(l->U1Period());ga_real(l->V1Period());ga_real(l->U2Period());ga_real(l->V2Period());
 ga_flag(l->HasArcOnS1());ga_int(fx.arc_rank(l->GetArcOnS1()));ga_flag(l->HasArcOnS2());ga_int(fx.arc_rank(l->GetArcOnS2()));
 ga_flag(l->IsPurgingAllowed());ga_int(l->GetCreatingWay());
 if(dump>=0)ga_ip_captured([&]{l->Dump(dump);});
 ga_end();
}
// `dump trans haspoints [points] nvertex vertex-scripts ncommands commands`: a restriction
// line; the RLine commands add 2 set point, 10/11 the arcs, 13 a new point set (Add), 15
// SetCurve with the null handle.
void ga_ip_rline_case(std::istream& input){
 GaReader r{input};GaIpFixture fx;int dump=r.integer();GaIpTrans t=ga_ip_read_trans(r);
 occ::handle<IntPatch_RLine> l=ga_ip_make<IntPatch_RLine>(t);if(r.flag())l->Add(ga_ip_read_line2s(r));
 int nv=r.integer();for(int i=0;i<nv;++i)l->AddVertex(ga_ip_read_point(r,fx));
 int n=r.integer();
 for(int k=0;k<n;++k){int cmd=r.integer();GA_TRY(if(!ga_ip_point_line_command(r,fx,l,cmd)){
  if(cmd==2){int i=r.integer();l->SetPoint(i,ga_ip_read_point(r,fx));}
  else if(cmd==10)l->SetArcOnS1(fx.arc(r.integer()));
  else if(cmd==11)l->SetArcOnS2(fx.arc(r.integer()));
  else if(cmd==13)l->SetCurve(ga_ip_read_line2s(r));
  else l->SetCurve(occ::handle<IntSurf_LineOn2S>());});}
 ga_ip_line_base(l);ga_ip_points(l);ga_ip_vertices(fx,l);ga_flag(l->HasPolygon());
 ga_flag(l->IsArcOnS1());ga_int(fx.arc_rank(l->ArcOnS1()));ga_flag(l->IsArcOnS2());ga_int(fx.arc_rank(l->ArcOnS2()));
 double a,b;l->ParamOnS1(a,b);ga_real(a);ga_real(b);l->ParamOnS2(a,b);ga_real(a);ga_real(b);
 if(dump>=0)ga_ip_captured([&]{l->Dump(dump);});
 ga_end();
}
// `kind trans p d xd r1 r2 nvertex vertex-scripts ncommands commands`: a conic line of the
// frame (p, d, xd) — kind 0 line, 1 circle (r1), 2 ellipse (r1, r2), 3 parabola (r1), 4
// hyperbola (r1, r2); commands 1 add, 3 replace, 4 first, 5 last, 6 compute.
void ga_ip_gline_case(std::istream& input){
 GaReader r{input};GaIpFixture fx;int kind=r.integer();GaIpTrans t=ga_ip_read_trans(r);gp_Pnt p=r.pnt();gp_Dir d=r.dir(),xd=r.dir();
 double r1=r.real(),r2=r.real();
 GA_TRY(gp_Ax2 ax(p,d,xd);occ::handle<IntPatch_GLine> l;
  if(kind==0)l=ga_ip_make<IntPatch_GLine>(t,gp_Lin(p,d));else if(kind==1)l=ga_ip_make<IntPatch_GLine>(t,gp_Circ(ax,r1));
  else if(kind==2)l=ga_ip_make<IntPatch_GLine>(t,gp_Elips(ax,r1,r2));else if(kind==3)l=ga_ip_make<IntPatch_GLine>(t,gp_Parab(ax,r1));
  else l=ga_ip_make<IntPatch_GLine>(t,gp_Hypr(ax,r1,r2));
  int nv=r.integer();for(int i=0;i<nv;++i)l->AddVertex(ga_ip_read_point(r,fx));
  int n=r.integer();
  for(int k=0;k<n;++k){int cmd=r.integer();GA_TRY(
   if(cmd==1)l->AddVertex(ga_ip_read_point(r,fx));
   else if(cmd==3){int i=r.integer();l->Replace(i,ga_ip_read_point(r,fx));}
   else if(cmd==4)l->SetFirstPoint(r.integer());else if(cmd==5)l->SetLastPoint(r.integer());
   else l->ComputeVertexParameters(r.real()));}
  ga_ip_line_base(l);ga_ip_vertices(fx,l);
  GA_TRY(gp_Lin x=l->Line();ga_pnt(x.Location());ga_dir(x.Direction()));
  GA_TRY(gp_Circ x=l->Circle();ga_pnt(x.Location());ga_dir(x.Axis().Direction());ga_dir(x.XAxis().Direction());ga_real(x.Radius()));
  GA_TRY(gp_Elips x=l->Ellipse();ga_pnt(x.Location());ga_dir(x.XAxis().Direction());ga_real(x.MajorRadius());ga_real(x.MinorRadius()));
  GA_TRY(gp_Parab x=l->Parabola();ga_pnt(x.Location());ga_dir(x.YAxis().Direction());ga_real(x.Focal()));
  GA_TRY(gp_Hypr x=l->Hyperbola();ga_pnt(x.Location());ga_dir(x.Axis().Direction());ga_real(x.MajorRadius());ga_real(x.MinorRadius())));
 ga_end();
}
// `p d rc sp sr tol trans nvertex vertex-scripts ncommands commands nparams params`: the
// first curve of the intersection of the cylinder (p, d, rc) and the sphere (sp, sr) as an
// analytic line; commands 1 add, 3 replace, 4 first, 5 last, 6 compute, 14 parameter
// through ChangeVertex; then the curve at the parameters.
void ga_ip_aline_case(std::istream& input){
 GaReader r{input};GaIpFixture fx;gp_Pnt p=r.pnt();gp_Dir d=r.dir();double rc=r.real();gp_Pnt sp=r.pnt();double sr=r.real(),tol=r.real();
 GaIpTrans t=ga_ip_read_trans(r);
 GA_TRY(IntAna_IntQuadQuad q(gp_Cylinder(gp_Ax3(p,d),rc),IntAna_Quadric(gp_Sphere(gp_Ax3(sp,gp_Dir(0,0,1)),sr)),tol);
  IntAna_Curve curve=q.NbCurve()>0?q.Curve(1):IntAna_Curve();occ::handle<IntPatch_ALine> l=ga_ip_make<IntPatch_ALine>(t,curve);
  int nv=r.integer();for(int i=0;i<nv;++i)l->AddVertex(ga_ip_read_point(r,fx));
  int n=r.integer();
  for(int k=0;k<n;++k){int cmd=r.integer();GA_TRY(
   if(cmd==1)l->AddVertex(ga_ip_read_point(r,fx));
   else if(cmd==3){int i=r.integer();l->Replace(i,ga_ip_read_point(r,fx));}
   else if(cmd==4)l->SetFirstPoint(r.integer());else if(cmd==5)l->SetLastPoint(r.integer());
   else if(cmd==6)l->ComputeVertexParameters(r.real());
   else{int i=r.integer();double prm=r.real();l->ChangeVertex(i).SetParameter(prm);});}
  ga_ip_line_base(l);ga_ip_vertices(fx,l);
  GA_TRY(bool inc;ga_real(l->FirstParameter(inc));ga_flag(inc));GA_TRY(bool inc;ga_real(l->LastParameter(inc));ga_flag(inc));
  int np=r.integer();
  for(int i=0;i<np;++i){double u=r.real();GA_TRY(ga_pnt(l->Value(u)));GA_TRY(gp_Pnt x;gp_Vec v(0,0,0);ga_flag(l->D1(u,x,v));ga_pnt(x);ga_xyz(v.XYZ()));
   GA_TRY(NCollection_List<double> ps;l->FindParameter(l->Value(u),ps);ga_int(ps.Size());for(double x:ps)ga_real(x));});
 ga_end();
}
