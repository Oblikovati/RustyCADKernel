// ChFiDS probes of the TKFillet oracle (included by tkfillet_probe.cxx): the guideline
// abscissa and evaluation of ChFiDS_Spine, and the radius laws of ChFiDS_FilSpine.
// The Rust twin is crates/tkfillet/examples/fl_probe/chfi_ds.rs.
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <ChFiDS_ElSpine.hxx>
#include <ChFiDS_FilSpine.hxx>
#include <ChFiDS_Spine.hxx>
#include <GeomAbs_CurveType.hxx>
#include <Geom_Line.hxx>
#include <Law_Composite.hxx>
#include <TopoDS_Edge.hxx>
#include <gp_Ax2.hxx>
#include <gp_Circ.hxx>
#include <gp_Lin.hxx>
#include <gp_Vec.hxx>
#include <gp_XY.hxx>

void fl_vec(const gp_Vec& v){ga_xyz(v.XYZ());}

// A segment edge from `a` to `b`; with `reversed`, the edge from `b` to `a`, reversed.
TopoDS_Edge fl_segment(const gp_Pnt& a,const gp_Pnt& b,bool reversed){
 if(!reversed)return BRepBuilderAPI_MakeEdge(a,b).Edge();
 TopoDS_Edge e=BRepBuilderAPI_MakeEdge(b,a).Edge();e.Reverse();return e;}

// The extremity setup shared by both spine probes: closed state, then Load.
void fl_load(const occ::handle<ChFiDS_Spine>& spine,bool closed){
 if(closed)spine->SetFirstStatus(ChFiDS_Closed);
 GA_TRY(spine->Load());
 GA_TRY(ga_real(spine->LastParameter()));ga_real(spine->GetTolesp());}

// Index, D2, D1 and both Parameter flavours, then Absc of the raw parameter, at `w`.
void fl_spine_at(const occ::handle<ChFiDS_Spine>& spine,double w){
 GA_TRY(ga_int(spine->Index(w,true)));GA_TRY(ga_int(spine->Index(w,false)));
 GA_TRY(gp_Pnt p;gp_Vec v1,v2;spine->D2(w,p,v1,v2);ga_pnt(p);fl_vec(v1);fl_vec(v2));
 GA_TRY(gp_Pnt p;gp_Vec v1;spine->D1(w,p,v1);ga_pnt(p);fl_vec(v1));
 GA_TRY(double u;spine->Parameter(w,u,true);ga_real(u));
 GA_TRY(double u;spine->Parameter(w,u,false);ga_real(u);ga_real(spine->Absc(u)));}

// `fl_spine N P1..PN R closed tgtFirst tgtLast refMode ref M W1..WM`: a guideline of the
// N-1 segments Pi-Pi+1 (segment i reversed when bit i-1 of R is set), loaded, optionally
// closed, extended by tangents and given a reference (1: abscissa, 2: edge index), then
// evaluated at each W.
void fl_spine_case(std::istream& in){
 GaReader r{in};int n=r.integer();std::vector<gp_Pnt> pts;for(int i=0;i<n;++i)pts.push_back(r.pnt());
 int rev=r.integer();bool closed=r.flag(),tf=r.flag(),tl=r.flag();int refMode=r.integer();double ref=r.real();
 std::vector<double> ws=r.reals();
 occ::handle<ChFiDS_Spine> spine=new ChFiDS_Spine();
 for(int i=0;i+1<n;++i)spine->SetEdges(fl_segment(pts[i],pts[i+1],(rev>>i)&1));
 fl_load(spine,closed);
 if(tf)GA_TRY(spine->SetFirstTgt(spine->FirstParameter()));
 if(tl)GA_TRY(spine->SetLastTgt(spine->LastParameter()));
 if(refMode==1)GA_TRY(spine->SetReference(ref));
 if(refMode==2)GA_TRY(spine->SetReference(int(ref)));
 for(double w:ws)fl_spine_at(spine,w);
 ga_end();}

// `fl_arc_spine C r a0 a1 reversed M W1..WM`: a guideline of one arc of the circle of
// center C and radius r in a plane parallel to XOY, from angle a0 to a1; its Circle()
// and type, then each W as in fl_spine.
void fl_arc_spine_case(std::istream& in){
 GaReader r{in};gp_Pnt c=r.pnt();double rad=r.real(),a0=r.real(),a1=r.real();bool rev=r.flag();
 std::vector<double> ws=r.reals();
 occ::handle<ChFiDS_Spine> spine=new ChFiDS_Spine();
 gp_Circ circ(gp_Ax2(c,gp_Dir(0,0,1)),rad);
 TopoDS_Edge e=BRepBuilderAPI_MakeEdge(circ,a0,a1).Edge();if(rev)e.Reverse();
 spine->SetEdges(e);fl_load(spine,false);
 GA_TRY(spine->CurrentElementarySpine(1));ga_int(int(spine->GetType()));
 GA_TRY(gp_Circ k=spine->Circle();ga_pnt(k.Location());ga_dir(k.Axis().Direction());
        ga_dir(k.XAxis().Direction());ga_real(k.Radius()));
 for(double w:ws)fl_spine_at(spine,w);
 ga_end();}

// The radius reads of a fillet guideline: IsConstant, then per edge IsConstant and Radius.
void fl_radii(const occ::handle<ChFiDS_FilSpine>& spine){
 ga_flag(spine->IsConstant());GA_TRY(ga_real(spine->Radius()));
 for(int i=1;i<=spine->NbEdges();++i){GA_TRY(ga_flag(spine->IsConstant(i)));GA_TRY(ga_real(spine->Radius(i)));}}

// `fl_fil_law N closed P (U R)*P E (I R)*E deb fin periodic M X1..XM`: a fillet guideline
// of N unit segments along X, optionally closed, with the radii R at abscissae U
// (SetRadius(gp_XY, 0)) and constant radii on edges I; the law of an elementary spine on
// [deb, fin] (a Geom_Line, optionally periodic) sampled at each X, then the radii after
// Reset(false).
void fl_fil_law_case(std::istream& in){
 GaReader r{in};int n=r.integer();bool closed=r.flag();
 occ::handle<ChFiDS_FilSpine> spine=new ChFiDS_FilSpine();
 for(int i=0;i<n;++i)spine->SetEdges(fl_segment(gp_Pnt(i,0,0),gp_Pnt(i+1,0,0),false));
 fl_load(spine,closed);
 int np=r.integer();for(int i=0;i<np;++i){double u=r.real(),rr=r.real();GA_TRY(spine->SetRadius(gp_XY(u,rr),0));}
 int ne=r.integer();for(int i=0;i<ne;++i){int k=r.integer();double rr=r.real();GA_TRY(spine->SetRadius(rr,spine->Edges(k)));}
 double deb=r.real(),fin=r.real();bool per=r.flag();std::vector<double> xs=r.reals();
 fl_radii(spine);
 occ::handle<ChFiDS_ElSpine> els=new ChFiDS_ElSpine();
 els->SetCurve(new Geom_Line(gp_Pnt(0,0,0),gp_Dir(1,0,0)));
 els->FirstParameter(deb);els->LastParameter(fin);els->SetPeriodic(per);
 GA_TRY(spine->AppendElSpine(els);occ::handle<Law_Composite> law=spine->Law(els);ga_flag(law.IsNull());
        if(!law.IsNull())for(double x:xs)GA_TRY(ga_real(law->Value(x)));
        ga_real(spine->MaxRadFromSeqAndLaws()));
 GA_TRY(spine->Reset(false));fl_radii(spine);
 ga_end();}
