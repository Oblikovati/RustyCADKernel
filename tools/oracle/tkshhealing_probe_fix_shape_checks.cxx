// Analysis and wire-repair operations of the `sh_fix_shape` probe
// (tkshhealing_probe_fix_shape.cxx); the Rust twin is
// crates/tkshhealing/examples/sh_probe/fix_shape_checks.rs.
//
// Extra inputs: family 3 is a face on a Bezier surface (`nu nv (pole)*`, poles row by row
// in U), family 4 a face on the XY plane bounded by a wire whose edges are only added by
// BRep_Builder, so it may have gaps (`n (sh_unify edge)*`; no wire at all for n = 0), and
// family 5 a face on the unit sphere whose wire collects circle arcs without pcurves
// (`n (cx cy cz nx ny nz r a1 a2)*`).
//  16 ShapeAnalysis_CheckSmallFace (tolerance prec) on each face: CheckSingleStrip,
//     FindStripEdges, CheckPin, CheckTwisted, CheckSplittingVertices, CheckPinFace with prec
//     and -1, then CheckStripEdges (prec and -1) and CheckPinEdges on each pair of consecutive
//     edges with 3D curves, and the strip, pin, twisted and split-vertex statuses;
//  17 ShapeAnalysis_FreeBoundsProperties on the actual free bounds, 18 on the bounds
//     forecast with tolerance maxtol and both splits: the flag, then each closed and open
//     record (contour, area, perimeter, ratio, width, notches and their widths);
//  19..30 ShapeFix_Wire (precision prec, maximal tolerance maxtol, a context) on each wire
//     of each face, then on each free wire: 19 FixTails (width maxtol, angle 0.3), 20
//     FixNotchedEdges, 21 FixSelfIntersection, 22 the same with ModifyRemoveLoopMode 1, 23
//     FixGaps3d, 24 FixGaps2d, 25 FixGaps3d by ranges, 26 FixEdgeCurves, 27 FixClosed(prec),
//     28 Perform, 29 FixGaps2d by ranges, 30 FixReorder, FixGaps3d then FixGaps2d:
//     the flag, the status flags of the pass, StatusRemovedSegment and the wire;
//  32 ShapeFix_SplitTool (tolerances prec and 1e-9) on each edge of each face with a 3D
//     curve and a pcurve on range [a, b]: SplitEdge at the middle with a new vertex there,
//     SplitEdge around [a+0.3d, a+0.6d], the sequence SplitEdge over [a+0.2d, a+0.7d],
//     over [a, a+0.5d] and over [a+0.5d, b] with a context, then CutEdge from a to a+0.25d
//     and from b to a+0.75d: each flag with the outputs.
#include <BRepBuilderAPI_MakeFace.hxx>
#include <Geom_BezierSurface.hxx>
#include <Geom_Circle.hxx>
#include <Geom_Plane.hxx>
#include <Geom_SphericalSurface.hxx>
#include <ShapeAnalysis_FreeBoundData.hxx>
#include <ShapeAnalysis_FreeBoundsProperties.hxx>
#include <ShapeFix_SplitTool.hxx>
#include <ShapeFix_Wire.hxx>
TopoDS_Shape sfs_bezier_face(GaReader& r){
 int nu=r.integer(),nv=r.integer();NCollection_Array2<gp_Pnt> poles(1,nu,1,nv);
 for(int i=1;i<=nu;++i)for(int j=1;j<=nv;++j)poles(i,j)=r.pnt();
 occ::handle<Geom_Surface> s=new Geom_BezierSurface(poles);return BRepBuilderAPI_MakeFace(s,1e-7).Face();}
TopoDS_Shape sfs_raw_face(GaReader& r){
 int n=r.integer();BRep_Builder B;TopoDS_Wire w;B.MakeWire(w);
 for(int k=0;k<n;++k)B.Add(w,su_edge(r));
 TopoDS_Face f;B.MakeFace(f,new Geom_Plane(gp_Pln()),1e-7);if(n>0)B.Add(f,w);return f;}
TopoDS_Shape sfs_sphere_face(GaReader& r){
 int n=r.integer();BRep_Builder B;TopoDS_Wire w;B.MakeWire(w);
 for(int k=0;k<n;++k){gp_Pnt c=r.pnt();gp_Dir d=r.dir();double R=r.real(),a1=r.real(),a2=r.real();
  occ::handle<Geom_Circle> circ=new Geom_Circle(gp_Ax2(c,d),R);B.Add(w,BRepBuilderAPI_MakeEdge(circ,a1,a2).Edge());}
 TopoDS_Face f;B.MakeFace(f,new Geom_SphericalSurface(gp_Ax3(),1.0),1e-7);B.Add(f,w);return f;}
void sfs_status_range(bool (*get)(const ShapeAnalysis_CheckSmallFace&,ShapeExtend_Status),
                      const ShapeAnalysis_CheckSmallFace& a){
 for(ShapeExtend_Status x:sfs_statuses)ga_flag(get(a,x));}
void sfs_check_statuses(const ShapeAnalysis_CheckSmallFace& a){
 sfs_status_range([](const ShapeAnalysis_CheckSmallFace& c,ShapeExtend_Status x){return c.StatusStrip(x);},a);
 sfs_status_range([](const ShapeAnalysis_CheckSmallFace& c,ShapeExtend_Status x){return c.StatusPin(x);},a);
 sfs_status_range([](const ShapeAnalysis_CheckSmallFace& c,ShapeExtend_Status x){return c.StatusTwisted(x);},a);
 sfs_status_range([](const ShapeAnalysis_CheckSmallFace& c,ShapeExtend_Status x){return c.StatusSplitVert(x);},a);}
void sfs_split_vertices(ShapeAnalysis_CheckSmallFace& a,const TopoDS_Face& f){
 NCollection_DataMap<TopoDS_Shape,NCollection_List<TopoDS_Shape>,TopTools_ShapeMapHasher> edges;
 NCollection_DataMap<TopoDS_Shape,NCollection_List<double>,TopTools_ShapeMapHasher> params;
 TopoDS_Compound all;BRep_Builder().MakeCompound(all);
 ga_int(a.CheckSplittingVertices(f,edges,params,all));tp_shape(all);
 for(TopExp_Explorer v(f,TopAbs_VERTEX);v.More();v.Next()){
  ga_flag(edges.IsBound(v.Current()));if(!edges.IsBound(v.Current()))continue;
  for(NCollection_List<TopoDS_Shape>::Iterator it(edges(v.Current()));it.More();it.Next())tp_shape(it.Value());
  for(NCollection_List<double>::Iterator it(params(v.Current()));it.More();it.Next())ga_real(it.Value());}}
void sfs_pin_face(ShapeAnalysis_CheckSmallFace& a,const TopoDS_Face& f,double tol){
 NCollection_DataMap<TopoDS_Shape,TopoDS_Shape,TopTools_ShapeMapHasher> map;
 GA_TRY(ga_flag(a.CheckPinFace(f,map,tol));ga_int(map.Extent()))}
bool sfs_has_curve(const TopoDS_Edge& e){double f,l;return !BRep_Tool::Curve(e,f,l).IsNull();}
void sfs_edge_pairs(const ShapeAnalysis_CheckSmallFace& a,const TopoDS_Face& f,double prec){
 std::vector<TopoDS_Edge> es;for(TopExp_Explorer e(f,TopAbs_EDGE);e.More();e.Next())es.push_back(TopoDS::Edge(e.Current()));
 for(size_t i=0;i+1<es.size();++i){
  if(!sfs_has_curve(es[i])||!sfs_has_curve(es[i+1]))continue;double d=-1;
  GA_TRY(ga_flag(a.CheckStripEdges(es[i],es[i+1],prec,d));ga_real(d))
  GA_TRY(ga_flag(a.CheckStripEdges(es[i],es[i+1],-1,d));ga_real(d))
  GA_TRY(ga_flag(a.CheckPinEdges(es[i],es[i+1],5.,7.,prec)))
  GA_TRY(ga_flag(a.CheckPinEdges(es[i+1],es[i],4.,40.,-1)))}}
void sfs_face_checks(ShapeAnalysis_CheckSmallFace& a,const TopoDS_Face& f,double prec){
 TopoDS_Edge e1,e2;GA_TRY(ga_flag(a.CheckSingleStrip(f,e1,e2,prec));sfs_edge(e1);sfs_edge(e2))
 double d=-1;GA_TRY(ga_flag(a.FindStripEdges(f,e1,e2,prec,d));ga_real(d);sfs_edge(e1);sfs_edge(e2))
 int row=-7,sens=-7;GA_TRY(ga_flag(a.CheckPin(f,row,sens));ga_int(row);ga_int(sens))
 double u=-7,v=-7;GA_TRY(ga_flag(a.CheckTwisted(f,u,v));ga_real(u);ga_real(v))
 GA_TRY(sfs_split_vertices(a,f))
 sfs_pin_face(a,f,prec);sfs_pin_face(a,f,-1);
 sfs_edge_pairs(a,f,prec);sfs_check_statuses(a);}
void sfs_small_face_checks(const TopoDS_Shape& s,double prec){
 ShapeAnalysis_CheckSmallFace a;a.SetTolerance(prec);
 for(TopExp_Explorer e(s,TopAbs_FACE);e.More();e.Next())sfs_face_checks(a,TopoDS::Face(e.Current()),prec);}
void sfs_bound_record(const occ::handle<ShapeAnalysis_FreeBoundData>& d){
 tp_shape(d->FreeBound());ga_real(d->Area());ga_real(d->Perimeter());ga_real(d->Ratio());ga_real(d->Width());
 ga_int(d->NbNotches());for(int i=1;i<=d->NbNotches();++i){tp_shape(d->Notch(i));ga_real(d->NotchWidth(i));}}
void sfs_free_properties(const TopoDS_Shape& s,double maxtol,bool forecast){
 ShapeAnalysis_FreeBoundsProperties p;
 if(forecast)p.Init(s,maxtol,true,true);else p.Init(s,false,false);
 ga_flag(p.Perform());ga_int(p.NbClosedFreeBounds());ga_int(p.NbOpenFreeBounds());
 for(int i=1;i<=p.NbClosedFreeBounds();++i)sfs_bound_record(p.ClosedFreeBound(i));
 for(int i=1;i<=p.NbOpenFreeBounds();++i)sfs_bound_record(p.OpenFreeBound(i));}
bool sfs_wire_pass(ShapeFix_Wire& w,int op,double prec,double maxtol){
 if(op==19){w.SetMaxTailWidth(maxtol);w.SetMaxTailAngle(0.3);return w.FixTails();}
 if(op==20)return w.FixNotchedEdges();
 if(op==21||op==22){if(op==22)w.ModifyRemoveLoopMode()=1;return w.FixSelfIntersection();}
 if(op==23)return w.FixGaps3d();
 if(op==24)return w.FixGaps2d();
 if(op==25){w.FixGapsByRangesMode()=true;return w.FixGaps3d();}
 if(op==26)return w.FixEdgeCurves();
 if(op==27)return w.FixClosed(prec);
 if(op==29){w.FixGapsByRangesMode()=true;return w.FixGaps2d();}
 if(op==30){w.FixReorder();bool a=w.FixGaps3d();return w.FixGaps2d()||a;}
 return w.Perform();}
void sfs_wire_report(const ShapeFix_Wire& w){
 for(ShapeExtend_Status x:sfs_statuses){ga_flag(w.StatusFixTails(x));ga_flag(w.StatusNotches(x));
  ga_flag(w.StatusSelfIntersection(x));ga_flag(w.StatusGaps3d(x));ga_flag(w.StatusGaps2d(x));
  ga_flag(w.StatusEdgeCurves(x));ga_flag(w.StatusClosed(x));}
 ga_flag(w.StatusRemovedSegment());ga_int(w.NbEdges());tp_shape(w.Wire());}
void sfs_wire_fix(ShapeFix_Wire& w,int op,double prec,double maxtol){
 w.SetContext(new ShapeBuild_ReShape);w.SetPrecision(prec);w.SetMaxTolerance(maxtol);
 GA_TRY(ga_flag(sfs_wire_pass(w,op,prec,maxtol));sfs_wire_report(w))}
void sfs_wire_fixes(const TopoDS_Shape& s,double prec,double maxtol,int op){
 for(TopExp_Explorer f(s,TopAbs_FACE);f.More();f.Next())
  for(TopExp_Explorer e(f.Current(),TopAbs_WIRE);e.More();e.Next()){
   ShapeFix_Wire w(TopoDS::Wire(e.Current()),TopoDS::Face(f.Current()),prec);sfs_wire_fix(w,op,prec,maxtol);}
 for(TopExp_Explorer e(s,TopAbs_WIRE,TopAbs_FACE);e.More();e.Next()){
  ShapeFix_Wire w;w.Load(TopoDS::Wire(e.Current()));sfs_wire_fix(w,op,prec,maxtol);}}

TopoDS_Vertex sfs_vertex_at(const occ::handle<Geom_Curve>& c,double t,double tol){
 TopoDS_Vertex v;BRep_Builder().MakeVertex(v,c->Value(t),tol);return v;}
void sfs_pair(bool ok,const TopoDS_Edge& e1,const TopoDS_Edge& e2,const TopoDS_Vertex& v){
 ga_flag(ok);if(!ok)return;tp_shape(e1);tp_shape(e2);ga_real(BRep_Tool::Tolerance(v));}
void sfs_sequence(const ShapeFix_SplitTool& t,const TopoDS_Edge& e,const occ::handle<Geom_Curve>& c,double f,double l,
                  const TopoDS_Face& face,double prec){
 TopoDS_Vertex v1=sfs_vertex_at(c,f,prec),v2=sfs_vertex_at(c,l,prec);
 occ::handle<ShapeBuild_ReShape> ctx=new ShapeBuild_ReShape;NCollection_Sequence<TopoDS_Shape> seq;int num=-1;
 GA_TRY(bool ok=t.SplitEdge(e,f,v1,l,v2,face,seq,num,ctx,prec,1e-9);ga_flag(ok);ga_int(num);ga_int(seq.Length());
  for(int i=1;i<=seq.Length();i++)tp_shape(seq(i));tp_shape(ctx->Apply(e)))}
void sfs_split_edge(const ShapeFix_SplitTool& t,const TopoDS_Edge& e,const TopoDS_Face& face,double prec){
 double f,l,a,b;occ::handle<Geom_Curve> c=BRep_Tool::Curve(e,f,l);occ::handle<Geom2d_Curve> c2d;
 if(c.IsNull()||!ShapeAnalysis_Edge().PCurve(e,face,c2d,a,b,true))return;
 double d=b-a;TopoDS_Edge e1,e2;
 TopoDS_Vertex v=sfs_vertex_at(c,(f+l)/2,prec);GA_TRY(bool ok=t.SplitEdge(e,a+0.5*d,v,face,e1,e2,prec,1e-9);sfs_pair(ok,e1,e2,v))
 v=sfs_vertex_at(c,(f+l)/2,prec);GA_TRY(bool ok=t.SplitEdge(e,a+0.3*d,a+0.6*d,v,face,e1,e2,prec,1e-9);sfs_pair(ok,e1,e2,v))
 sfs_sequence(t,e,c,a+0.2*d,a+0.7*d,face,prec);sfs_sequence(t,e,c,a,a+0.5*d,face,prec);sfs_sequence(t,e,c,a+0.5*d,b,face,prec);
 bool line=false;GA_TRY(ga_flag(t.CutEdge(e,a,a+0.25*d,face,line));ga_flag(line);tp_shape(e))
 GA_TRY(ga_flag(t.CutEdge(e,b,a+0.75*d,face,line));ga_flag(line);tp_shape(e))}
void sfs_split_tool(const TopoDS_Shape& s,double prec){
 ShapeFix_SplitTool t;
 for(TopExp_Explorer f(s,TopAbs_FACE);f.More();f.Next())
  for(TopExp_Explorer e(f.Current(),TopAbs_EDGE);e.More();e.Next())
   sfs_split_edge(t,TopoDS::Edge(e.Current()),TopoDS::Face(f.Current()),prec);}
