// BRepExtrema probe operations, proximity group (docs/topalgo-design.md section 13):
// TriangleSet (with its BVH), OverlapTool, SelfIntersection (with its protected filter),
// ProximityDistTool, ProximityValueTool, ShapeProximity and VertexInspector, on the meshes
// of shapes read by tp_shape. Maps are printed in their iteration order. Needs
// tktopalgo_probe_brepextrema.cxx (sub-shape readers). The Rust twin is
// crates/tktopalgo/examples/tp_probe/brep_extrema_prox.rs.
#include <BRepExtrema_OverlapTool.hxx>
#include <BRepExtrema_ProximityDistTool.hxx>
#include <BRepExtrema_ProximityValueTool.hxx>
#include <BRepExtrema_SelfIntersection.hxx>
#include <BRepExtrema_ShapeProximity.hxx>
#include <BRepExtrema_TriangleSet.hxx>
void tp_bxp_vec(const BVH_Vec3d& v){ga_real(v.x());ga_real(v.y());ga_real(v.z());}
// `n (kind index)...`: a list of sub-shapes.
NCollection_DynamicArray<TopoDS_Shape> tp_bxp_list(GaReader& r,const TpShape& s){
 NCollection_DynamicArray<TopoDS_Shape> list;int n=r.integer();for(int i=0;i<n;++i)list.Append(tp_bxe_sub(r,s));return list;}
// An overlap map: its size, then each key with the size and the items of its set.
void tp_bxp_map(const NCollection_DataMap<int,TColStd_PackedMapOfInteger>& m){
 ga_int(m.Extent());
 for(NCollection_DataMap<int,TColStd_PackedMapOfInteger>::Iterator it(m);it.More();it.Next()){
  ga_int(it.Key());ga_int(it.Value().Extent());
  for(TColStd_PackedMapOfInteger::Iterator k(it.Value());k.More();k.Next())ga_int(k.Key());}}
// Whether the shapes 0..=id of the list are all faces (GetTrgIdxInShape reads the triangle
// counts of the shapes before id unchecked; only faces store one).
bool tp_bxp_faces_before(const NCollection_DynamicArray<TopoDS_Shape>& list,int id){
 for(int i=0;i<=id;++i)if(i>=int(list.Size())||list(i).ShapeType()!=TopAbs_FACE)return false;return true;}
// Everything a triangle set tells: sizes, boxes, centers, vertices and indices of each
// triangle, the shape of each vertex, and the BVH nodes.
void tp_bxp_set(const occ::handle<BRepExtrema_TriangleSet>& set,const NCollection_DynamicArray<TopoDS_Shape>& list){
 ga_int(set->Size());int nv=set->GetVertices().Size();ga_int(nv);
 BVH_Box<double,3> all=set->Box();tp_bxp_vec(all.CornerMin());tp_bxp_vec(all.CornerMax());
 for(int i=0;i<set->Size();++i){BVH_Box<double,3> b=set->Box(i);tp_bxp_vec(b.CornerMin());tp_bxp_vec(b.CornerMax());
  for(int axis=0;axis<3;++axis)ga_real(set->Center(i,axis));
  BVH_Vec3d v1,v2,v3;set->GetVertices(i,v1,v2,v3);tp_bxp_vec(v1);tp_bxp_vec(v2);tp_bxp_vec(v3);
  NCollection_Array1<int> idx;set->GetVtxIndices(i,idx);for(int k=0;k<3;++k)ga_int(idx(k));
  ga_int(set->GetFaceID(i));if(tp_bxp_faces_before(list,set->GetFaceID(i)))ga_int(set->GetTrgIdxInShape(i));}
 for(int i=0;i<nv;++i){ga_int(set->GetShapeIDOfVtx(i));ga_int(set->GetVtxIdxInShape(i));}
 const occ::handle<BVH_Tree<double,3>>& tree=set->BVH();ga_int(tree->Length());
 for(int i=0;i<tree->Length();++i){const BVH_Vec4i& info=tree->NodeInfoBuffer()[i];
  ga_int(info.x());ga_int(info.y());ga_int(info.z());tp_bxp_vec(tree->MinPoint(i));tp_bxp_vec(tree->MaxPoint(i));}}
// `shape list mode [list2]`: mode 0 the constructor on the list, 1 Init twice (the list,
// then list2, whose vertex data keep entries of the first one), 2 Init then Swap(0, 1)
// and Clear.
void tp_bxp_tset_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpShape s=tp_shape(r);NCollection_DynamicArray<TopoDS_Shape> list=tp_bxp_list(r,s);int mode=r.integer();
  occ::handle<BRepExtrema_TriangleSet> set;
  if(mode==0){set=new BRepExtrema_TriangleSet(list);tp_bxp_set(set,list);}
  else if(mode==1){NCollection_DynamicArray<TopoDS_Shape> list2=tp_bxp_list(r,s);set=new BRepExtrema_TriangleSet;
   ga_flag(set->Init(list));tp_bxp_set(set,list);ga_flag(set->Init(list2));tp_bxp_set(set,list2);}
  else{set=new BRepExtrema_TriangleSet;ga_flag(set->Init(list));if(set->Size()>1)set->Swap(0,1);tp_bxp_set(set,list);
   set->Clear();ga_int(set->Size());ga_int(set->GetVertices().Size());});
 ga_end();}
// `shape1 list1 shape2 list2 n (tolerance)...`: an overlap tool on the two sets performed
// with each tolerance in turn (the maps accumulate); the second set is the first one when
// list2 is empty.
void tp_bxp_overlap_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpShape a=tp_shape(r);NCollection_DynamicArray<TopoDS_Shape> l1=tp_bxp_list(r,a);TpShape b=tp_shape(r);
  NCollection_DynamicArray<TopoDS_Shape> l2=tp_bxp_list(r,b);
  occ::handle<BRepExtrema_TriangleSet> s1=new BRepExtrema_TriangleSet(l1);
  occ::handle<BRepExtrema_TriangleSet> s2=l2.IsEmpty()?s1:new BRepExtrema_TriangleSet(l2);
  BRepExtrema_OverlapTool tool(s1,s2);ga_flag(tool.IsDone());int n=r.integer();
  for(int i=0;i<n;++i){double tol=r.real();GA_TRY(tool.Perform(tol));ga_flag(tool.IsDone());
   tp_bxp_map(tool.OverlapSubShapes1());tp_bxp_map(tool.OverlapSubShapes2());tool.MarkDirty();ga_flag(tool.IsDone());});
 ga_end();}
// The protected filter of the self-intersection, made public for the probe.
struct TpSelfIntersection: public BRepExtrema_SelfIntersection{
 using BRepExtrema_SelfIntersection::BRepExtrema_SelfIntersection;
 using BRepExtrema_SelfIntersection::PreCheckElements;
 using BRepExtrema_SelfIntersection::isRegularSharedVertex;
 using BRepExtrema_SelfIntersection::isRegularSharedEdge;};
// `shape tolerance mode npairs`: mode 0 the constructor on the shape, 1 LoadShape on an
// empty tool, Perform twice (the second one does nothing), then a new tolerance; then
// PreCheckElements for the first `npairs` pairs of triangles (i < j).
void tp_bxp_self_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpShape s=tp_shape(r);double tol=r.real();int mode=r.integer();int npairs=r.integer();
  std::unique_ptr<TpSelfIntersection> t;
  if(mode==0)t.reset(new TpSelfIntersection(s.shape,tol));
  else{t.reset(new TpSelfIntersection(tol));ga_flag(t->LoadShape(s.shape));}
  ga_real(t->Tolerance());ga_flag(t->IsDone());GA_TRY(t->Perform());ga_flag(t->IsDone());tp_bxp_map(t->OverlapElements());
  if(mode==1){GA_TRY(t->Perform());tp_bxp_map(t->OverlapElements());t->SetTolerance(tol+0.25);ga_real(t->Tolerance());}
  const occ::handle<BRepExtrema_TriangleSet>& set=t->ElementSet();ga_int(set->Size());
  for(int i=0;i<t->OverlapElements().Extent()&&i<3;++i)tp_bxe_id(s,s,t->GetSubShape(i));
  int k=0;
  for(int i=0;i<set->Size()&&k<npairs;++i)for(int j=i+1;j<set->Size()&&k<npairs;++j,++k){
   GA_TRY(ga_int(int(t->PreCheckElements(i,j))));});
 ga_end();}
// `n (shared v1 v2 w1 w2)... m (v0 v1 v2 w2)...`: the shared-vertex and shared-edge tests.
void tp_bxp_shared_case(std::istream& input){
 GaReader r{input};TpSelfIntersection t;
 auto vec=[&]{double x=r.real(),y=r.real();return BVH_Vec3d(x,y,r.real());};
 int n=r.integer();
 for(int i=0;i<n;++i){BVH_Vec3d a=vec(),b=vec(),c=vec(),d=vec(),e=vec();GA_TRY(ga_int(int(t.isRegularSharedVertex(a,b,c,d,e))));}
 int m=r.integer();
 for(int i=0;i<m;++i){BVH_Vec3d a=vec(),b=vec(),c=vec(),d=vec();ga_int(int(t.isRegularSharedEdge(a,b,c,d)));}
 ga_end();}
// `shape1 list1 shape2 list2 nsamples n (x y z status)... mode`: a proximity distance tool
// from the first set (with additional vertices) to the second; mode 0 the loading
// constructor, 1 the default one then the loaders.
void tp_bxp_pdist_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpShape a=tp_shape(r);NCollection_DynamicArray<TopoDS_Shape> l1=tp_bxp_list(r,a);TpShape b=tp_shape(r);
  NCollection_DynamicArray<TopoDS_Shape> l2=tp_bxp_list(r,b);int ns=r.integer();int n=r.integer();BVH_Array3d add;
  NCollection_DynamicArray<ProxPnt_Status> st;
  for(int i=0;i<n;++i){double x=r.real(),y=r.real(),z=r.real();add.EmplaceAppend(x,y,z);st.Append(ProxPnt_Status(r.integer()));}
  int mode=r.integer();
  occ::handle<BRepExtrema_TriangleSet> s1=new BRepExtrema_TriangleSet(l1),s2=new BRepExtrema_TriangleSet(l2);
  std::unique_ptr<BRepExtrema_ProximityDistTool> t;
  if(mode==0)t.reset(new BRepExtrema_ProximityDistTool(s1,ns,add,st,s2,l1,l2));
  else{t.reset(new BRepExtrema_ProximityDistTool());ga_real(t->ProximityDistance());t->LoadTriangleSets(s1,s2);
   t->LoadShapeLists(l1,l2);t->LoadAdditionalPointsFirstSet(add,st);}
  GA_TRY(t->Perform());ga_flag(t->IsDone());ga_real(t->ProximityDistance());BVH_Vec3d p1,p2;t->ProximityPoints(p1,p2);
  tp_bxp_vec(p1);tp_bxp_vec(p2);ProxPnt_Status q1,q2;t->ProximityPointsStatus(q1,q2);ga_int(int(q1));ga_int(int(q2)););
 ga_end();}
// `shape face`: IsNodeOnBorder for each node and IsEdgeOnBorder for each triangle and each
// pair of its nodes (from 0) of the triangulation of the face.
void tp_bxp_border_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpShape s=tp_shape(r);TopoDS_Face f=TopoDS::Face(tp_bxe_sub(r,s));TopLoc_Location l;
  occ::handle<Poly_Triangulation> tr=BRep_Tool::Triangulation(f,l);
  for(int i=1;i<=tr->NbNodes();++i)GA_TRY(ga_flag(BRepExtrema_ProximityDistTool::IsNodeOnBorder(i,tr)));
  for(int t=1;t<=tr->NbTriangles();++t)for(int j=0;j<3;++j)for(int k=0;k<3;++k)
   GA_TRY(ga_flag(BRepExtrema_ProximityDistTool::IsEdgeOnBorder(t,j,k,tr))););
 ga_end();}
// The results of a proximity value tool.
void tp_bxp_pval_state(const BRepExtrema_ProximityValueTool& t){
 ga_flag(t.IsDone());ga_real(t.Distance());gp_Pnt p1,p2;t.ProximityPoints(p1,p2);ga_pnt(p1);ga_pnt(p2);
 if(t.IsDone()){ProxPnt_Status q1,q2;t.ProximityPointsStatus(q1,q2);ga_int(int(q1));ga_int(int(q2));}}
// `shape1 list1 shape2 list2 ns1 ns2 mode`: mode 0 the loading constructor then Perform, 1
// the default one, the loaders, SetNbSamplePoints and Perform twice (MarkDirty between,
// the refinement adds vertices again), 2 LoadShapeLists only (not performed).
void tp_bxp_pval_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpShape a=tp_shape(r);NCollection_DynamicArray<TopoDS_Shape> l1=tp_bxp_list(r,a);TpShape b=tp_shape(r);
  NCollection_DynamicArray<TopoDS_Shape> l2=tp_bxp_list(r,b);int ns1=r.integer(),ns2=r.integer(),mode=r.integer();
  occ::handle<BRepExtrema_TriangleSet> s1=new BRepExtrema_TriangleSet(l1),s2=new BRepExtrema_TriangleSet(l2);
  double tol=-1.;
  if(mode==0){BRepExtrema_ProximityValueTool t(s1,s2,l1,l2);t.SetNbSamplePoints(ns1,ns2);GA_TRY(t.Perform(tol));
   ga_real(tol);tp_bxp_pval_state(t);}
  else{BRepExtrema_ProximityValueTool t;tp_bxp_pval_state(t);GA_TRY(t.LoadShapeLists(l1,l2));t.LoadTriangleSets(s1,s2);
   t.SetNbSamplePoints(ns1,ns2);
   if(mode==1)for(int i=0;i<2;++i){GA_TRY(t.Perform(tol));ga_real(tol);tp_bxp_pval_state(t);t.MarkDirty();}
   tp_bxp_pval_state(t);});
 ga_end();}
// `shape1 shape2 tolerance ns1 ns2 mode`: mode 0 the constructor on the shapes, 1 the
// default one with LoadShape1/LoadShape2, 2 as 1 then LoadShape1 again and Perform (the
// set of the first shape keeps its vertex data); Perform twice (the second does nothing).
void tp_bxp_prox_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpShape a=tp_shape(r);TpShape b=tp_shape(r);double tol=r.real();int ns1=r.integer(),ns2=r.integer(),mode=r.integer();
  bool inf=tol==Precision::Infinite();
  std::unique_ptr<BRepExtrema_ShapeProximity> p;
  if(mode==0)p.reset(new BRepExtrema_ShapeProximity(a.shape,b.shape,tol));
  else{p.reset(new BRepExtrema_ShapeProximity(tol));ga_flag(p->LoadShape1(a.shape));ga_flag(p->LoadShape2(b.shape));}
  p->SetNbSamples1(ns1);p->SetNbSamples2(ns2);
  auto state=[&]{ga_flag(p->IsDone());ga_real(p->Proximity());ga_real(p->Tolerance());ga_pnt(p->ProximityPoint1());
   ga_pnt(p->ProximityPoint2());
   if(inf&&p->IsDone()){ga_int(int(p->ProxPntStatus1()));ga_int(int(p->ProxPntStatus2()));}
   tp_bxp_map(p->OverlapSubShapes1());tp_bxp_map(p->OverlapSubShapes2());
   ga_int(p->ElementSet1()->Size());ga_int(p->ElementSet2()->Size());};
  GA_TRY(p->Perform());state();GA_TRY(p->Perform());state();
  if(mode==2){ga_flag(p->LoadShape1(a.shape));GA_TRY(p->Perform());state();}
  GA_TRY(tp_bxe_id(a,b,p->GetSubShape1(0)));GA_TRY(tp_bxe_id(a,b,p->GetSubShape2(0))););
 ga_end();}
// `tol n (x y z)... m (cx cy cz ntargets target...)...`: a vertex inspector: Coord and Shift
// of each point, the points added, then each current point inspected against targets.
void tp_bxp_insp_case(std::istream& input){
 GaReader r{input};BRepExtrema_VertexInspector insp;ga_flag(insp.IsNeedAdd());
 double tol=r.real();if(tol>=0.)insp.SetTol(tol);int n=r.integer();
 for(int i=0;i<n;++i){gp_XYZ p=r.xyz();for(int k=0;k<3;++k)ga_real(BRepExtrema_VertexInspector::Coord(k,p));
  ga_xyz(BRepExtrema_VertexInspector::Shift(p,tol));insp.Add(p);}
 int m=r.integer();
 for(int i=0;i<m;++i){insp.SetCurrent(r.xyz());ga_flag(insp.IsNeedAdd());int nt=r.integer();
  for(int k=0;k<nt;++k){ga_int(int(insp.Inspect(r.integer())));ga_flag(insp.IsNeedAdd());}}
 ga_end();}
