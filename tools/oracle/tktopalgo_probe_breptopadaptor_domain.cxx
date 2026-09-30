// BRepTopAdaptor_TopolTool as the domain of IntStart and IntPatch (docs/topalgo-design.md
// section 8): the boundary and inside searches and IntPatch_Intersection run on faces read
// by tp_face (tktopalgo_probe_face.cxx) through BRepAdaptor_Surface and
// BRepTopAdaptor_TopolTool, so every vertex is a BRepTopAdaptor_HVertex reached through
// its Adaptor3d_HVertex base. The Rust twin is
// crates/tktopalgo/examples/tp_probe/brep_top_adaptor_domain.rs. Needs the quadric reader
// of tkgeomalgo_probe_intsurf.cxx and the line printers of
// tkgeomalgo_probe_intpatch_report.cxx.
#include <IntPatch_ArcFunction.hxx>
#include <IntPatch_Intersection.hxx>
#include <IntPatch_TheSOnBounds.hxx>
#include <IntPatch_TheSearchInside.hxx>
#include <IntPatch_TheSurfFunction.hxx>
#include <IntSurf_InteriorPoint.hxx>
// `face restr`: the face, its adaptor (restricted or not) and its tool.
struct TpBtaDomain{TopoDS_Face face;occ::handle<Adaptor3d_Surface> s;occ::handle<BRepTopAdaptor_TopolTool> d;};
TpBtaDomain tp_bta_domain(GaReader& r){
 TpFace f=tp_face(r);bool restr=r.flag();occ::handle<BRepAdaptor_Surface> s=new BRepAdaptor_Surface(f.face,restr);
 occ::handle<BRepTopAdaptor_TopolTool> d=new BRepTopAdaptor_TopolTool(s);return TpBtaDomain{f.face,s,d};}
// The edge index of an arc of the domain (-3 when the arc is not a BRepAdaptor_Curve2d).
int tp_bta_arc_edge(const TpBtaDomain& dm,const occ::handle<Adaptor2d_Curve2d>& arc){
 occ::handle<BRepAdaptor_Curve2d> c=occ::down_cast<BRepAdaptor_Curve2d>(arc);
 return c.IsNull()?-3:tp_edge_index(dm.face,c->Edge());}
// A vertex through its base handle (the virtual members of BRepTopAdaptor_HVertex) and
// through the tool (its 3d tolerance and point).
void tp_bta_domain_vertex(const TpBtaDomain& dm,const occ::handle<Adaptor3d_HVertex>& v,const occ::handle<Adaptor2d_Curve2d>& arc){
 ga_pnt2d(v->Value());ga_int(v->Orientation());GA_TRY(ga_real(v->Parameter(arc)))GA_TRY(ga_real(v->Resolution(arc)))
 GA_TRY(ga_real(dm.d->Tol3d(v)))GA_TRY(ga_pnt(dm.d->Pnt(v)))}
void tp_bta_path_point(const TpBtaDomain& dm,const IntPatch_ThePathPointOfTheSOnBounds& p){
 ga_pnt(p.Value());ga_real(p.Tolerance());ga_flag(p.IsNew());ga_real(p.Parameter());ga_int(tp_bta_arc_edge(dm,p.Arc()));
 GA_TRY(tp_bta_domain_vertex(dm,p.Vertex(),p.Arc()))}
// `face restr quadric tolb tolt recheck`: IntPatch_TheSOnBounds (IntStart_SearchOnBoundaries)
// of the quadric on the edges of the face.
void tp_bta_sob_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpBtaDomain dm=tp_bta_domain(r);IntSurf_Quadric q=ga_intsurf_quadric(r);double tolb=r.real(),tolt=r.real();bool recheck=r.flag();
  IntPatch_ArcFunction f;f.SetQuadric(q);f.Set(dm.s);IntPatch_TheSOnBounds sob;GA_TRY(sob.Perform(f,dm.d,tolb,tolt,recheck));
  ga_flag(sob.IsDone());GA_TRY(ga_flag(sob.AllArcSolution()));
  GA_TRY(int n=sob.NbPoints();ga_int(n);for(int i=1;i<=n;++i)tp_bta_path_point(dm,sob.Point(i)));
  GA_TRY(int n=sob.NbSegments();ga_int(n);for(int i=1;i<=n;++i){const IntPatch_TheSegmentOfTheSOnBounds& g=sob.Segment(i);
   ga_int(tp_bta_arc_edge(dm,g.Curve()));ga_flag(g.HasFirstPoint());GA_TRY(tp_bta_path_point(dm,g.FirstPoint()));
   ga_flag(g.HasLastPoint());GA_TRY(tp_bta_path_point(dm,g.LastPoint()));}))
 ga_end();}
// `face restr quadric tol eps`: IntPatch_TheSearchInside (IntStart_SearchInside) of the
// quadric on the face, sampled and classified by the tool.
void tp_bta_inside_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpBtaDomain dm=tp_bta_domain(r);IntSurf_Quadric q=ga_intsurf_quadric(r);double tol=r.real(),eps=r.real();
  IntPatch_TheSurfFunction f(dm.s,q);f.Set(tol);IntPatch_TheSearchInside w(f,dm.s,dm.d,eps);ga_flag(w.IsDone());
  GA_TRY(int n=w.NbPoints();ga_int(n);for(int i=1;i<=n;++i){const IntSurf_InteriorPoint& p=w.Value(i);
   ga_pnt(p.Value());ga_real(p.UParameter());ga_real(p.VParameter());ga_xyz(p.Direction().XYZ());ga_xy(p.Direction2d().XY());}))
 ga_end();}
// `face1 restr1 face2 restr2 tolarc toltang geomint keeprline postwl`:
// IntPatch_Intersection::Perform on the two face domains, then its points and lines
// (arcs printed by their rank in the domains, 1.. and 101..).
void tp_bta_intersection_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpBtaDomain d1=tp_bta_domain(r);TpBtaDomain d2=tp_bta_domain(r);double ta=r.real(),tt=r.real();
  bool g=r.flag(),k=r.flag(),p=r.flag();GaIplDomains dm(d1.d,d2.d);IntPatch_Intersection in;
  GA_TRY(in.Perform(d1.s,d1.d,d2.s,d2.d,ta,tt,g,k,p))
  ga_flag(in.IsDone());GA_TRY(ga_flag(in.IsEmpty()));GA_TRY(ga_flag(in.TangentFaces()));GA_TRY(ga_flag(in.OppositeFaces()));
  GA_TRY(int n=in.NbPnts();ga_int(n);for(int i=1;i<=n;++i)ga_ipl_point(dm,in.Point(i)));
  GA_TRY(int n=in.NbLines();ga_int(n);for(int i=1;i<=n;++i)ga_ipl_line(dm,in.Line(i))))
 ga_end();}
