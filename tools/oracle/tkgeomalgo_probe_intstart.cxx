// IntStart probe operations (tkgeomalgo_probe.cxx); the Rust twin is
// crates/tkgeomalgo/examples/ga_probe/int_start.rs. Both templates run through their
// IntPatch instances, IntPatch_TheSOnBounds and IntPatch_TheSearchInside, on the real
// IntPatch classes. The boundary search is also instantiated here on a probe tool whose
// arcs "have been seen" (in its own namespace, as its file-static helpers would collide
// in this single translation unit), and both searches run on a probe domain whose Edge()
// returns a real edge, so that the branches unreachable with IntPatch_HInterTool and
// Adaptor3d_TopolTool run too. Needs the surface readers of
// tkgeomalgo_probe_intcurvesurface.cxx and the quadric reader of tkgeomalgo_probe_intsurf.cxx.
#include <Adaptor2d_Curve2d.hxx>
#include <Adaptor3d_CurveOnSurface.hxx>
#include <Adaptor3d_HSurfaceTool.hxx>
#include <Adaptor3d_HVertex.hxx>
#include <Adaptor3d_TopolTool.hxx>
#include <BRepAdaptor_Curve.hxx>
#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <Extrema_ExtCC.hxx>
#include <Extrema_POnCurv.hxx>
#include <GeomAdaptor_Curve.hxx>
#include <Geom_Line.hxx>
#include <Geom_Plane.hxx>
#include <TopLoc_Location.hxx>
#include <GeomAdaptor_Surface.hxx>
#include <IntCurveSurface_HInter.hxx>
#include <IntPatch_ArcFunction.hxx>
#include <IntPatch_HCurve2dTool.hxx>
#include <IntPatch_HInterTool.hxx>
#include <IntPatch_TheSOnBounds.hxx>
#include <IntPatch_TheSearchInside.hxx>
#include <IntPatch_TheSurfFunction.hxx>
#include <IntPatch_ThePathPointOfTheSOnBounds.hxx>
#include <IntPatch_TheSegmentOfTheSOnBounds.hxx>
#include <IntSurf_InteriorPoint.hxx>
#include <IntSurf_Quadric.hxx>
#include <TopoDS_Edge.hxx>
#include <algorithm>
#include <math_BrentMinimum.hxx>
#include <math_Function.hxx>
#include <math_FunctionAllRoots.hxx>
#include <math_FunctionSample.hxx>
#include <math_FunctionSetRoot.hxx>
#include <math_Matrix.hxx>
#include <math_Vector.hxx>
#include <NCollection_Array1.hxx>
#include <NCollection_Sequence.hxx>
#include <StdFail_NotDone.hxx>
#include <TopAbs_State.hxx>
// The known points and segments of the seen arcs: `(x y z tol prm isvertex)` and the
// 1-based first/last point indices (0 for none), set by each request.
struct GaIsSeenTable{std::vector<std::array<double,6>> pts;std::vector<std::pair<int,int>> segs;bool all=false;};
static GaIsSeenTable ga_is_seen;
// TheSOBTool whose every arc has been seen: its known points come from ga_is_seen.
class GaProbeSeenTool{
public:
 static double Parameter(const occ::handle<Adaptor3d_HVertex>& V,const occ::handle<Adaptor2d_Curve2d>& C){return IntPatch_HInterTool::Parameter(V,C);}
 static double Tolerance(const occ::handle<Adaptor3d_HVertex>& V,const occ::handle<Adaptor2d_Curve2d>& C){return IntPatch_HInterTool::Tolerance(V,C);}
 static bool HasBeenSeen(const occ::handle<Adaptor2d_Curve2d>&){return true;}
 static void Bounds(const occ::handle<Adaptor2d_Curve2d>& A,double& f,double& l){IntPatch_HInterTool::Bounds(A,f,l);}
 static int NbPoints(const occ::handle<Adaptor2d_Curve2d>&){return int(ga_is_seen.pts.size());}
 static const std::array<double,6>& Known(int i){if(i<1||i>int(ga_is_seen.pts.size()))throw Standard_OutOfRange();return ga_is_seen.pts[i-1];}
 static void Value(const occ::handle<Adaptor2d_Curve2d>&,const int i,gp_Pnt& P,double& tol,double& U){
  const std::array<double,6>& k=Known(i);P.SetCoord(k[0],k[1],k[2]);tol=k[3];U=k[4];}
 static bool IsVertex(const occ::handle<Adaptor2d_Curve2d>&,const int i){return Known(i)[5]!=0;}
 static void Vertex(const occ::handle<Adaptor2d_Curve2d>&,const int i,occ::handle<Adaptor3d_HVertex>& V){
  const std::array<double,6>& k=Known(i);V=new Adaptor3d_HVertex(gp_Pnt2d(k[4],0.),TopAbs_FORWARD,k[3]);}
 static int NbSegments(const occ::handle<Adaptor2d_Curve2d>&){return int(ga_is_seen.segs.size());}
 static const std::pair<int,int>& Seg(int i){if(i<1||i>int(ga_is_seen.segs.size()))throw Standard_OutOfRange();return ga_is_seen.segs[i-1];}
 static bool HasFirstPoint(const occ::handle<Adaptor2d_Curve2d>&,const int i,int& index){index=Seg(i).first;return index!=0;}
 static bool HasLastPoint(const occ::handle<Adaptor2d_Curve2d>&,const int i,int& index){index=Seg(i).second;return index!=0;}
 static bool IsAllSolution(const occ::handle<Adaptor2d_Curve2d>&){return ga_is_seen.all;}
};
#define GA_IS_SOB_CLASS(NAME) \
class NAME{ \
public: \
 NAME(); \
 void Perform(IntPatch_ArcFunction& F,const occ::handle<Adaptor3d_TopolTool>& Domain,const double TolBoundary,const double TolTangency,const bool RecheckOnRegularity=false); \
 bool IsDone() const; bool AllArcSolution() const; int NbPoints() const; \
 const IntPatch_ThePathPointOfTheSOnBounds& Point(const int Index) const; int NbSegments() const; \
 const IntPatch_TheSegmentOfTheSOnBounds& Segment(const int Index) const; \
private: \
 bool done; bool all; NCollection_Sequence<IntPatch_TheSegmentOfTheSOnBounds> sseg; \
 NCollection_Sequence<IntPatch_ThePathPointOfTheSOnBounds> spnt; \
};
#define TheVertex occ::handle<Adaptor3d_HVertex>
#define TheArc occ::handle<Adaptor2d_Curve2d>
#define TheArcTool IntPatch_HCurve2dTool
#define Handle_TheTopolTool occ::handle<Adaptor3d_TopolTool>
#define TheTopolTool Adaptor3d_TopolTool
#define TheFunction IntPatch_ArcFunction
#define IntStart_ThePathPoint IntPatch_ThePathPointOfTheSOnBounds
#define IntStart_SequenceOfPathPoint NCollection_Sequence<IntPatch_ThePathPointOfTheSOnBounds>
#define IntStart_TheSegment IntPatch_TheSegmentOfTheSOnBounds
#define IntStart_SequenceOfSegment NCollection_Sequence<IntPatch_TheSegmentOfTheSOnBounds>
#define IntStart_SearchOnBoundaries GaProbeSOnBounds
namespace ga_is_seen_ns{
GA_IS_SOB_CLASS(GaProbeSOnBounds)
#define TheSOBTool GaProbeSeenTool
#include <IntStart_SearchOnBoundaries.lxx>
#include <IntStart_SearchOnBoundaries.gxx>
#undef TheSOBTool
}
#undef TheVertex
#undef TheArc
#undef TheArcTool
#undef Handle_TheTopolTool
#undef TheTopolTool
#undef TheFunction
#undef IntStart_ThePathPoint
#undef IntStart_SequenceOfPathPoint
#undef IntStart_TheSegment
#undef IntStart_SequenceOfSegment
#undef IntStart_SearchOnBoundaries
// A domain whose Edge() is a real edge, as BRepTopAdaptor_TopolTool gives one.
class GaProbeEdgeTopolTool : public Adaptor3d_TopolTool{
public:
 GaProbeEdgeTopolTool(const occ::handle<Adaptor3d_Surface>& S,const TopoDS_Edge& E):Adaptor3d_TopolTool(S),myEdge(E){}
 void* Edge() const override{return myEdge.IsNull()?nullptr:(void*)&myEdge;}
private:
 TopoDS_Edge myEdge;
};
// `edge 0` (no edge) or `edge 1 p d first last degenerated regular`: a straight edge,
// with a C1 regularity between two planes when `regular`.
TopoDS_Edge ga_is_edge(GaReader& r){
 TopoDS_Edge e;if(!r.flag())return e;gp_Pnt p=r.pnt();gp_Dir d=r.dir();double f=r.real(),l=r.real();bool deg=r.flag(),reg=r.flag();
 BRep_Builder b;b.MakeEdge(e,new Geom_Line(p,d),1.e-7);b.Range(e,f,l);b.Degenerated(e,deg);
 if(reg){occ::handle<Geom_Surface> s1=new Geom_Plane(gp_Ax3(p,gp_Dir(0,0,1))),s2=new Geom_Plane(gp_Ax3(p,gp_Dir(0,1,0)));
  b.Continuity(e,s1,s2,TopLoc_Location(),TopLoc_Location(),GeomAbs_C1);}
 return e;
}
// The 1-based rank of `arc` among the restrictions of `domain`, 0 when it is none.
int ga_is_arc_rank(const occ::handle<Adaptor3d_TopolTool>& domain,const occ::handle<Adaptor2d_Curve2d>& arc){
 int rank=0;for(domain->Init();domain->More();domain->Next()){++rank;if(domain->Value()==arc)return rank;}return 0;}
void ga_is_path_point(const occ::handle<Adaptor3d_TopolTool>& d,const IntPatch_ThePathPointOfTheSOnBounds& p){
 ga_pnt(p.Value());ga_real(p.Tolerance());ga_flag(p.IsNew());ga_real(p.Parameter());ga_int(ga_is_arc_rank(d,p.Arc()));
 GA_TRY(const occ::handle<Adaptor3d_HVertex>& v=p.Vertex();ga_pnt2d(v->Value());ga_real(v->Resolution(p.Arc())));
}
template<class S> void ga_is_sob_report(const S& s,const occ::handle<Adaptor3d_TopolTool>& d){
 ga_flag(s.IsDone());GA_TRY(ga_flag(s.AllArcSolution()));
 GA_TRY(int n=s.NbPoints();ga_int(n);for(int i=1;i<=n;++i)ga_is_path_point(d,s.Point(i)));
 GA_TRY(int n=s.NbSegments();ga_int(n);for(int i=1;i<=n;++i){const IntPatch_TheSegmentOfTheSOnBounds& g=s.Segment(i);
  ga_int(ga_is_arc_rank(d,g.Curve()));ga_flag(g.HasFirstPoint());GA_TRY(ga_is_path_point(d,g.FirstPoint()));
  ga_flag(g.HasLastPoint());GA_TRY(ga_is_path_point(d,g.LastPoint()));});
 GA_TRY(s.Point(0));GA_TRY(s.Point(s.NbPoints()+1));GA_TRY(s.Segment(0));GA_TRY(s.Segment(s.NbSegments()+1));
}
// `seen surface quadric tolb tolt recheck mode edge [npts (x y z tol prm isv)* nseg (f l)*
// all]`: IntStart_SearchOnBoundaries.gxx on the arcs of the surface domain, with the
// IntPatch tools (seen 0) or the seen tool and its table (seen 1); mode 0 performs, 1
// performs twice, 2 reads the consultation without a Perform.
void ga_is_sob_case(std::istream& input){
 GaReader r{input};bool seen=r.flag();occ::handle<Adaptor3d_Surface> s=ga_ics_surface(r);IntSurf_Quadric q=ga_intsurf_quadric(r);
 double tolb=r.real(),tolt=r.real();bool recheck=r.flag();int mode=r.integer();TopoDS_Edge e=ga_is_edge(r);
 if(seen){ga_is_seen=GaIsSeenTable();int np=r.integer();for(int i=0;i<np;++i){std::array<double,6> k;for(double& x:k)x=r.real();ga_is_seen.pts.push_back(k);}
  int ns=r.integer();for(int i=0;i<ns;++i){int f=r.integer();ga_is_seen.segs.push_back({f,r.integer()});}ga_is_seen.all=r.flag();}
 int nperf=mode==0?1:(mode==1?2:0);
 GA_TRY(occ::handle<Adaptor3d_TopolTool> d=new GaProbeEdgeTopolTool(s,e);IntPatch_ArcFunction f;f.SetQuadric(q);f.Set(s);
  if(seen){ga_is_seen_ns::GaProbeSOnBounds sob;for(int k=0;k<nperf;++k)GA_TRY(sob.Perform(f,d,tolb,tolt,recheck));ga_is_sob_report(sob,d);}
  else{IntPatch_TheSOnBounds sob;for(int k=0;k<nperf;++k)GA_TRY(sob.Perform(f,d,tolb,tolt,recheck));ga_is_sob_report(sob,d);});
 ga_end();
}
void ga_is_inside_report(const IntPatch_TheSearchInside& s){
 ga_flag(s.IsDone());
 GA_TRY(int n=s.NbPoints();ga_int(n);for(int i=1;i<=n;++i){const IntSurf_InteriorPoint& p=s.Value(i);
  ga_pnt(p.Value());ga_real(p.UParameter());ga_real(p.VParameter());ga_xyz(p.Direction().XYZ());ga_xy(p.Direction2d().XY());});
 GA_TRY(s.Value(0));GA_TRY(s.Value(s.NbPoints()+1));
}
// `surface quadric tol eps mode [u v]`: IntStart_SearchInside.gxx on the surface domain
// with IntPatch_TheSurfFunction; mode 0 the sampled Perform (by the performing
// constructor), 1 the Perform from (u, v), 2 the consultation without a Perform.
void ga_is_inside_case(std::istream& input){
 GaReader r{input};occ::handle<Adaptor3d_Surface> s=ga_ics_surface(r);IntSurf_Quadric q=ga_intsurf_quadric(r);
 double tol=r.real(),eps=r.real();int mode=r.integer();
 GA_TRY(IntPatch_TheSurfFunction f(s,q);f.Set(tol);occ::handle<Adaptor3d_TopolTool> d=new Adaptor3d_TopolTool(s);
  if(mode==0){GA_TRY(IntPatch_TheSearchInside w(f,s,d,eps);ga_is_inside_report(w));}
  else if(mode==1){double u=r.real(),v=r.real();IntPatch_TheSearchInside w;GA_TRY(w.Perform(f,s,u,v));ga_is_inside_report(w);}
  else{IntPatch_TheSearchInside w;ga_is_inside_report(w);});
 ga_end();
}
