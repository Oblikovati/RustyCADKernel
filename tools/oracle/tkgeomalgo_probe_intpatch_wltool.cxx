// IntPatch_WLineTool and IntPatch_LineConstructor probe operations (tkgeomalgo_probe.cxx);
// the Rust twin is crates/tkgeomalgo/examples/ga_probe/int_patch_wltool.rs. Each request
// reads two surfaces (ga_ics_surface), their domains and one or more lines, runs one
// algorithm and prints the lines it gives. Needs tkgeomalgo_probe_intpatch.cxx,
// tkgeomalgo_probe_intpatch_lines.cxx and tkgeomalgo_probe_intcurvesurface.cxx.
#include <Adaptor3d_TopolTool.hxx>
#include <IntPatch_LineConstructor.hxx>
#include <IntPatch_WLineTool.hxx>
// A domain `narcs [ranks] tol`: the domain of the surface (Adaptor3d_TopolTool) whose arcs
// are the arcs of the fixture of the given ranks, or its own restrictions for `narcs < 0`;
// with a 3d representation of tolerance `tol` when `tol >= 0`.
class GaWtDomain : public Adaptor3d_TopolTool{
public:
 GaWtDomain(const occ::handle<Adaptor3d_Surface>& S,std::vector<occ::handle<Adaptor2d_Curve2d>> A,bool own,double tol)
 :Adaptor3d_TopolTool(S),myArcs(std::move(A)),myOwn(own),myTol(tol),myIdx(0){}
 void Init() override{if(myOwn)Adaptor3d_TopolTool::Init();else myIdx=0;}
 bool More() override{return myOwn?Adaptor3d_TopolTool::More():myIdx<myArcs.size();}
 occ::handle<Adaptor2d_Curve2d> Value() override{return myOwn?Adaptor3d_TopolTool::Value():myArcs[myIdx];}
 void Next() override{if(myOwn)Adaptor3d_TopolTool::Next();else ++myIdx;}
 using Adaptor3d_TopolTool::Tol3d;
 bool Has3d() const override{return myTol>=0.;}
 double Tol3d(const occ::handle<Adaptor2d_Curve2d>& C) const override{
  if(myTol<0.)return Adaptor3d_TopolTool::Tol3d(C);return myTol;}
private:
 std::vector<occ::handle<Adaptor2d_Curve2d>> myArcs;bool myOwn;double myTol;size_t myIdx;
};
occ::handle<Adaptor3d_TopolTool> ga_wt_domain(GaReader& r,const GaIpFixture& fx,const occ::handle<Adaptor3d_Surface>& s){
 int n=r.integer();std::vector<occ::handle<Adaptor2d_Curve2d>> arcs;for(int i=0;i<n;++i)arcs.push_back(fx.arc(r.integer()));
 double tol=r.real();return new GaWtDomain(s,arcs,n<0,tol);}
// A line `kind ...`: 0 walking `trans way pu1 pv1 pu2 pv2 points nvertex vertex-scripts arc1
// arc2`, 1 restriction `trans haspoints [points] nvertex vertex-scripts arc1 arc2`, 2 conic
// `ckind trans p d xd r1 r2 nvertex vertex-scripts`, 3 analytic `p d rc sp sr tol trans
// nvertex vertex-scripts` (the first curve of the cylinder and the sphere).
occ::handle<IntPatch_Line> ga_wt_line(GaReader& r,const GaIpFixture& fx){
 int kind=r.integer();
 if(kind==0){GaIpTrans t=ga_ip_read_trans(r);int way=r.integer();double p1=r.real(),p2=r.real(),p3=r.real(),p4=r.real();
  occ::handle<IntPatch_WLine> l=ga_ip_make<IntPatch_WLine>(t,ga_ip_read_line2s(r));l->SetCreatingWayInfo(IntPatch_WLine::IntPatch_WLType(way));
  l->SetPeriod(p1,p2,p3,p4);int nv=r.integer();for(int i=0;i<nv;++i)l->AddVertex(ga_ip_read_point(r,fx));
  int a1=r.integer(),a2=r.integer();if(a1)l->SetArcOnS1(fx.arc(a1));if(a2)l->SetArcOnS2(fx.arc(a2));return l;}
 if(kind==1){GaIpTrans t=ga_ip_read_trans(r);occ::handle<IntPatch_RLine> l=ga_ip_make<IntPatch_RLine>(t);if(r.flag())l->Add(ga_ip_read_line2s(r));
  int nv=r.integer();for(int i=0;i<nv;++i)l->AddVertex(ga_ip_read_point(r,fx));
  int a1=r.integer(),a2=r.integer();if(a1)l->SetArcOnS1(fx.arc(a1));if(a2)l->SetArcOnS2(fx.arc(a2));return l;}
 if(kind==2){int ck=r.integer();GaIpTrans t=ga_ip_read_trans(r);gp_Pnt p=r.pnt();gp_Dir d=r.dir(),xd=r.dir();double r1=r.real(),r2=r.real();
  gp_Ax2 ax(p,d,xd);occ::handle<IntPatch_GLine> l;
  if(ck==0)l=ga_ip_make<IntPatch_GLine>(t,gp_Lin(p,d));else if(ck==1)l=ga_ip_make<IntPatch_GLine>(t,gp_Circ(ax,r1));
  else if(ck==2)l=ga_ip_make<IntPatch_GLine>(t,gp_Elips(ax,r1,r2));else if(ck==3)l=ga_ip_make<IntPatch_GLine>(t,gp_Parab(ax,r1));
  else l=ga_ip_make<IntPatch_GLine>(t,gp_Hypr(ax,r1,r2));
  int nv=r.integer();for(int i=0;i<nv;++i)l->AddVertex(ga_ip_read_point(r,fx));return l;}
 gp_Pnt p=r.pnt();gp_Dir d=r.dir();double rc=r.real();gp_Pnt sp=r.pnt();double sr=r.real(),tol=r.real();GaIpTrans t=ga_ip_read_trans(r);
 IntAna_IntQuadQuad q(gp_Cylinder(gp_Ax3(p,d),rc),IntAna_Quadric(gp_Sphere(gp_Ax3(sp,gp_Dir(0,0,1)),sr)),tol);
 occ::handle<IntPatch_ALine> l=ga_ip_make<IntPatch_ALine>(t,q.NbCurve()>0?q.Curve(1):IntAna_Curve());
 int nv=r.integer();for(int i=0;i<nv;++i)l->AddVertex(ga_ip_read_point(r,fx));return l;
}
// A line of any kind: its base, its points (point lines), its vertices and its arcs.
void ga_wt_report(const GaIpFixture& fx,const occ::handle<IntPatch_Line>& l){
 ga_ip_line_base(l);
 occ::handle<IntPatch_WLine> w=occ::down_cast<IntPatch_WLine>(l);occ::handle<IntPatch_RLine> rl=occ::down_cast<IntPatch_RLine>(l);
 occ::handle<IntPatch_GLine> g=occ::down_cast<IntPatch_GLine>(l);occ::handle<IntPatch_ALine> a=occ::down_cast<IntPatch_ALine>(l);
 if(!w.IsNull()){ga_ip_points(w);ga_ip_vertices(fx,w);ga_int(fx.arc_rank(w->GetArcOnS1()));ga_int(fx.arc_rank(w->GetArcOnS2()));
  ga_int(w->GetCreatingWay());ga_flag(w->IsPurgingAllowed());ga_real(w->U1Period());ga_real(w->V2Period());}
 if(!rl.IsNull()){ga_ip_points(rl);ga_ip_vertices(fx,rl);ga_int(fx.arc_rank(rl->ArcOnS1()));ga_int(fx.arc_rank(rl->ArcOnS2()));}
 if(!g.IsNull())ga_ip_vertices(fx,g);
 if(!a.IsNull())ga_ip_vertices(fx,a);
}
// `s1 s2 dom1 dom2 wline`: IntPatch_WLineTool::ComputePurgedWLine; the result or a null flag.
void ga_ip_purge_case(std::istream& input){
 GaReader r{input};GaIpFixture fx;
 GA_TRY(occ::handle<Adaptor3d_Surface> s1=ga_ics_surface(r);occ::handle<Adaptor3d_Surface> s2=ga_ics_surface(r);
  occ::handle<Adaptor3d_TopolTool> d1=ga_wt_domain(r,fx,s1);occ::handle<Adaptor3d_TopolTool> d2=ga_wt_domain(r,fx,s2);
  occ::handle<IntPatch_WLine> w=occ::down_cast<IntPatch_WLine>(ga_wt_line(r,fx));
  occ::handle<IntPatch_WLine> p=IntPatch_WLineTool::ComputePurgedWLine(w,s1,s2,d1,d2);ga_flag(p.IsNull());if(!p.IsNull())ga_wt_report(fx,p);
  ga_int(w->NbPnts());ga_int(w->NbVertex()));
 ga_end();
}
// `s1 s2 tol nlines lines npoints point-scripts`: IntPatch_WLineTool::JoinWLines.
void ga_ip_join_case(std::istream& input){
 GaReader r{input};GaIpFixture fx;ga_real(IntPatch_WLineTool::myMaxConcatAngle);
 GA_TRY(occ::handle<Adaptor3d_Surface> s1=ga_ics_surface(r);occ::handle<Adaptor3d_Surface> s2=ga_ics_surface(r);double tol=r.real();
  NCollection_Sequence<occ::handle<IntPatch_Line>> lines;int nl=r.integer();for(int i=0;i<nl;++i)lines.Append(ga_wt_line(r,fx));
  NCollection_Sequence<IntPatch_Point> pts;int np=r.integer();for(int i=0;i<np;++i)pts.Append(ga_ip_read_point(r,fx));
  GA_TRY(IntPatch_WLineTool::JoinWLines(lines,pts,s1,s2,tol));
  ga_int(lines.Length());for(int i=1;i<=lines.Length();++i)ga_wt_report(fx,lines(i));
  ga_int(pts.Length());for(int i=1;i<=pts.Length();++i)ga_ip_point(fx,pts(i)));
 ga_end();
}
// `s1 s2 dom1 dom2 tol nref ref-lines line`: IntPatch_LineConstructor::Perform.
void ga_ip_lc_case(std::istream& input){
 GaReader r{input};GaIpFixture fx;
 GA_TRY(occ::handle<Adaptor3d_Surface> s1=ga_ics_surface(r);occ::handle<Adaptor3d_Surface> s2=ga_ics_surface(r);
  occ::handle<Adaptor3d_TopolTool> d1=ga_wt_domain(r,fx,s1);occ::handle<Adaptor3d_TopolTool> d2=ga_wt_domain(r,fx,s2);double tol=r.real();
  NCollection_Sequence<occ::handle<IntPatch_Line>> refs;int nr=r.integer();for(int i=0;i<nr;++i)refs.Append(ga_wt_line(r,fx));
  occ::handle<IntPatch_Line> l=ga_wt_line(r,fx);IntPatch_LineConstructor lc(2);
  GA_TRY(lc.Perform(refs,l,s1,d1,s2,d2,tol));
  int n=lc.NbLines();ga_int(n);for(int i=1;i<=n;++i)ga_wt_report(fx,lc.Line(i));GA_TRY(lc.Line(0));GA_TRY(lc.Line(n+1)));
 ga_end();
}
