// BRepBuilderAPI_Sewing probe operations; the Rust twin is
// crates/tktopalgo/examples/tp_probe/sewing.rs. The shapes come from the shape reader
// (tktopalgo_probe_shape.cxx) and are printed with the BRepLib digest (tp_shape of
// tktopalgo_probe_breplib_common.cxx).
//
// `bba_sew shape nnm (edge t)... tol o1 o2 o3 o4 facemode floating samepar mintol maxtol localtol mode
//   context summary`: each (edge t) adds an INTERNAL vertex (tolerance 1e-7) at the
//   point of parameter t of the 3d curve of edge `edge` of the record (the edge is freed
//   for the addition); then BRepBuilderAPI_Sewing(tol, o1, o2, o3, o4), SetContext(new
//   BRepTools_ReShape) when `context`, the modes (a negative mintol or maxtol keeps the
//   default), then the input: mode 0 Add of each child of the shape (the shape itself
//   when it is not a compound), 1 Add of the shape, 2 Load of the shape, 3 Load of the
//   first child and Add of the others. Perform prints 1 or its exception, then:
//   the tolerances and modes; the counts (free, multiple, contiguous, degenerated,
//   deleted); the sewn shape digest; each free, multiple, contiguous (with its couple:
//   each section located, IsSectionBound and SectionToBoundary), degenerated shape and
//   deleted face located; the out-of-range accessors; for each sub-shape of the input
//   (TopExp::MapShapes order) IsModified, Modified, IsModifiedSubShape,
//   ModifiedSubShape and IsDegenerated; WhichFace 1 and 2 of each input edge; and the
//   Dump text. A shape is located by `input sewn orientation` (its indices in the
//   MapShapes maps of the input and of the sewn shape, 0 when absent), -1 when null.
// A last field `summary` set replaces the order of the sewn shape by an order-free
// summary (tp_sew_summary) and its index by a presence flag: BRepTools_Quilt appends the
// bound shapes that are not faces (floating edges) in the order of a hashed map, which
// follows the TShape addresses.
#include <BRepBuilderAPI_Sewing.hxx>
#include <BRepTools_ReShape.hxx>
#include <algorithm>
struct TpSewMaps{TpShapeMap input,sewn;bool summary;};
void tp_sew_locate(const TpSewMaps& m,const TopoDS_Shape& s){
 if(s.IsNull()){ga_int(-1);return;}
 ga_int(m.input.FindIndex(s));ga_int(m.summary?int(m.sewn.Contains(s)):m.sewn.FindIndex(s));ga_int(s.Orientation());}
std::vector<uint64_t> tp_sew_point(const TopoDS_Vertex& v){
 if(v.IsNull())return {0};gp_Pnt p=BRep_Tool::Pnt(v);
 return {1,ga_bits(p.X()),ga_bits(p.Y()),ga_bits(p.Z()),ga_bits(BRep_Tool::Tolerance(v))};}
// The order-free summary of a shape: its counts of vertices, edges and faces, its type,
// then the sorted tuples of its vertices (point, tolerance) and of its edges (tolerance,
// degenerated flag, range, sorted end tuples).
void tp_sew_summary(const TopoDS_Shape& s){
 if(s.IsNull()){ga_int(-1);return;}
 TpShapeMap mv=tp_sub_shapes(s,TopAbs_VERTEX),me=tp_sub_shapes(s,TopAbs_EDGE),mf=tp_sub_shapes(s,TopAbs_FACE);
 ga_int(mv.Extent());ga_int(me.Extent());ga_int(mf.Extent());ga_int(s.ShapeType());
 std::vector<std::vector<uint64_t>> vs,es;
 for(int i=1;i<=mv.Extent();++i)vs.push_back(tp_sew_point(TopoDS::Vertex(mv(i))));
 for(int i=1;i<=me.Extent();++i){const TopoDS_Edge& e=TopoDS::Edge(me(i));double f,l;BRep_Tool::Range(e,f,l);
  std::vector<uint64_t> t{ga_bits(BRep_Tool::Tolerance(e)),uint64_t(BRep_Tool::Degenerated(e)),ga_bits(f),ga_bits(l)};
  TopoDS_Vertex a,b;TopExp::Vertices(e,a,b);std::vector<uint64_t> pa=tp_sew_point(a),pb=tp_sew_point(b);
  if(pb<pa)std::swap(pa,pb);t.insert(t.end(),pa.begin(),pa.end());t.insert(t.end(),pb.begin(),pb.end());es.push_back(t);}
 std::sort(vs.begin(),vs.end());std::sort(es.begin(),es.end());
 for(const auto& t:vs)for(uint64_t u:t)std::cout<<u<<' ';
 for(const auto& t:es)for(uint64_t u:t)std::cout<<u<<' ';}
void tp_sew_nm_vertices(GaReader& r,const TpShape& shape){
 int n=r.integer();BRep_Builder B;
 for(int i=0;i<n;++i){TopoDS_Edge e=shape.edges.at(r.integer());double t=r.real(),f,l;
  occ::handle<Geom_Curve> c=BRep_Tool::Curve(e,f,l);TopoDS_Vertex v;B.MakeVertex(v,c->Value(t),1e-7);
  e.Free(true);B.Add(e,v.Oriented(TopAbs_INTERNAL));e.Free(false);}}
void tp_sew_input(BRepBuilderAPI_Sewing& sew,const TopoDS_Shape& s,int mode){
 if(mode==1){sew.Add(s);return;}
 if(mode==2){sew.Load(s);return;}
 if(s.ShapeType()!=TopAbs_COMPOUND){sew.Add(s);return;}
 bool first=true;
 for(TopoDS_Iterator it(s);it.More();it.Next()){
  if(first&&mode==3)sew.Load(it.Value());else sew.Add(it.Value());first=false;}}
void tp_sew_settings(const BRepBuilderAPI_Sewing& sew){
 ga_real(sew.Tolerance());ga_real(sew.MinTolerance());ga_real(sew.MaxTolerance());
 ga_flag(sew.FaceMode());ga_flag(sew.FloatingEdgesMode());ga_flag(sew.SameParameterMode());
 ga_flag(sew.LocalTolerancesMode());ga_flag(sew.NonManifoldMode());}
void tp_sew_counts(const BRepBuilderAPI_Sewing& sew){
 ga_int(sew.NbFreeEdges());ga_int(sew.NbMultipleEdges());ga_int(sew.NbContigousEdges());
 ga_int(sew.NbDegeneratedShapes());ga_int(sew.NbDeletedFaces());}
void tp_sew_couple(const BRepBuilderAPI_Sewing& sew,const TpSewMaps& m,int i){
 const NCollection_List<TopoDS_Shape>& couple=sew.ContigousEdgeCouple(i);ga_int(couple.Extent());
 for(NCollection_List<TopoDS_Shape>::Iterator it(couple);it.More();it.Next()){
  tp_sew_locate(m,it.Value());const TopoDS_Edge& e=TopoDS::Edge(it.Value());ga_flag(sew.IsSectionBound(e));
  GA_TRY(tp_sew_locate(m,sew.SectionToBoundary(e)))}}
void tp_sew_lists(const BRepBuilderAPI_Sewing& sew,const TpSewMaps& m){
 for(int i=1;i<=sew.NbFreeEdges();++i)tp_sew_locate(m,sew.FreeEdge(i));
 for(int i=1;i<=sew.NbMultipleEdges();++i)tp_sew_locate(m,sew.MultipleEdge(i));
 for(int i=1;i<=sew.NbContigousEdges();++i){tp_sew_locate(m,sew.ContigousEdge(i));tp_sew_couple(sew,m,i);}
 for(int i=1;i<=sew.NbDegeneratedShapes();++i)tp_sew_locate(m,sew.DegeneratedShape(i));
 for(int i=1;i<=sew.NbDeletedFaces();++i)tp_sew_locate(m,sew.DeletedFace(i));}
void tp_sew_ranges(const BRepBuilderAPI_Sewing& sew,const TpSewMaps& m){
 GA_TRY(tp_sew_locate(m,sew.FreeEdge(0)))GA_TRY(tp_sew_locate(m,sew.FreeEdge(sew.NbFreeEdges()+1)))
 GA_TRY(tp_sew_locate(m,sew.FreeEdge(-1)))GA_TRY(tp_sew_locate(m,sew.MultipleEdge(sew.NbMultipleEdges()+1)))
 GA_TRY(tp_sew_locate(m,sew.ContigousEdge(0)))GA_TRY(ga_int(sew.ContigousEdgeCouple(sew.NbContigousEdges()+1).Extent()))
 GA_TRY(tp_sew_locate(m,sew.DegeneratedShape(sew.NbDegeneratedShapes()+1)))
 GA_TRY(tp_sew_locate(m,sew.DeletedFace(0)))GA_TRY(tp_sew_locate(m,sew.DeletedFace(sew.NbDeletedFaces()+1)))}
void tp_sew_history(const BRepBuilderAPI_Sewing& sew,const TpSewMaps& m){
 for(int i=1;i<=m.input.Extent();++i){const TopoDS_Shape& s=m.input(i);
  ga_flag(sew.IsModified(s));tp_sew_locate(m,sew.Modified(s));
  GA_TRY(ga_flag(sew.IsModifiedSubShape(s)))GA_TRY(tp_sew_locate(m,sew.ModifiedSubShape(s)))
  GA_TRY(ga_flag(sew.IsDegenerated(s)))
  if(s.ShapeType()==TopAbs_EDGE){GA_TRY(tp_sew_locate(m,sew.WhichFace(TopoDS::Edge(s),1)))
   GA_TRY(tp_sew_locate(m,sew.WhichFace(TopoDS::Edge(s),2)))}}}
void tp_bba_sew_case(std::istream& input){
 GaReader r{input};
 try{TpShape shape=tp_shape(r);tp_sew_nm_vertices(r,shape);double tol=r.real();bool o1=r.flag(),o2=r.flag(),o3=r.flag(),o4=r.flag();
  bool face=r.flag(),floating=r.flag(),samepar=r.flag();double mintol=r.real(),maxtol=r.real();bool local=r.flag();
  int mode=r.integer();bool context=r.flag(),summary=r.flag();
  occ::handle<BRepBuilderAPI_Sewing> sew=new BRepBuilderAPI_Sewing(tol,o1,o2,o3,o4);
  occ::handle<BRepTools_ReShape> ctx=new BRepTools_ReShape;if(context)sew->SetContext(ctx);
  sew->SetFaceMode(face);sew->SetFloatingEdgesMode(floating);sew->SetSameParameterMode(samepar);
  if(mintol>=0)sew->SetMinTolerance(mintol);if(maxtol>=0)sew->SetMaxTolerance(maxtol);sew->SetLocalTolerancesMode(local);
  tp_sew_input(*sew,shape.shape,mode);ga_flag(sew->GetContext()==ctx);
  GA_TRY(sew->Perform();ga_int(1))
  tp_sew_settings(*sew);tp_sew_counts(*sew);if(summary)tp_sew_summary(sew->SewedShape());else tp_shape(sew->SewedShape());
  TpSewMaps m;m.summary=summary;TopExp::MapShapes(shape.shape,m.input);if(!sew->SewedShape().IsNull())TopExp::MapShapes(sew->SewedShape(),m.sewn);
  tp_sew_lists(*sew,m);tp_sew_ranges(*sew,m);tp_sew_history(*sew,m);
  ga_captured([&]{sew->Dump();});
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 ga_end();}
