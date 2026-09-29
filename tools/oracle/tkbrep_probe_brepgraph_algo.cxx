// BRepGraph algorithms (phase 2, wave 2c, area C2): the `br_bedit` actions of
// BRepGraph_Validate and BRepGraph_Transform, and raw field corruptions written through
// a mutation guard's Internal() so the validator sees broken graphs. Included by
// tkbrep_probe_brepgraph_edit_rest.cxx before the action dispatch. The Rust twin is
// crates/tkbrep/examples/br_probe/brep_graph_algo.rs.
#include <BRepGraph_Transform.hxx>
#include <BRepGraph_Validate.hxx>
// `ba_trsf`: 0 translation (vector), 1 rotation (point, direction, angle), 2 scale
// (point, factor), 3 point mirror, 4 axis mirror (point, direction), 5 plane mirror
// (point, normal, X direction), 6 SetValues (the 12 coefficients row by row).
gp_Trsf ba_trsf(BrReader& r){
 gp_Trsf t;int k=r.integer();
 switch(k){
  case 0:{gp_Pnt v=br_point(r);t.SetTranslation(gp_Vec(v.XYZ()));break;}
  case 1:{gp_Pnt p=br_point(r);gp_Dir d=br_direction(r);t.SetRotation(gp_Ax1(p,d),r.real());break;}
  case 2:{gp_Pnt p=br_point(r);t.SetScale(p,r.real());break;}
  case 3:t.SetMirror(br_point(r));break;
  case 4:{gp_Pnt p=br_point(r);t.SetMirror(gp_Ax1(p,br_direction(r)));break;}
  case 5:{gp_Pnt p=br_point(r);gp_Dir n=br_direction(r);t.SetMirror(gp_Ax2(p,n,br_direction(r)));break;}
  default:{double v[12];for(double& x:v)x=r.real();
   t.SetValues(v[0],v[1],v[2],v[3],v[4],v[5],v[6],v[7],v[8],v[9],v[10],v[11]);break;}}
 return t;}
// A validation report: IsValid, the warning and error counts, then each issue
// (severity, node, description) in report order.
void ba_report(const BRepGraph_Validate::Result& v){
 br_flag(v.IsValid());br_int(v.NbIssues(BRepGraph_Validate::Severity::Warning));
 br_int(v.NbIssues(BRepGraph_Validate::Severity::Error));br_int(int(v.Issues.Size()));
 for(const auto& i:v.Issues){br_int(int(i.Sev));bg_node(i.NodeId);bl_text(i.Description);}}
BRepGraph_Validate::Result ba_validate(const BRepGraph& g,int how){
 using V=BRepGraph_Validate;
 switch(how){
  case 0:return V::Perform(g);
  case 1:return V::Perform(g,V::Mode::Lightweight);
  case 2:return V::Perform(g,V::Mode::Audit);
  case 3:return V::Perform(g,V::Options::Lightweight());
  default:return V::Perform(g,V::Options::Audit());}}
// 244-249: raw writes through Internal(). 244 edge (e, field, value): 0 StartVertexRefId,
// 1 EndVertexRefId, 2 Curve3DRepId, 3 Polygon3DRepId; 245 coedge (c, field, value): 0
// ChildEdgeId, 1 FaceId, 2 Curve2DRepId, 3 Polygon2DRepId, 4 PolygonOnTriRepId, 5
// ParentWireId; 246 face (f, field, value): 0 SurfaceRepId, 1 TriangulationRepId; 247
// occurrence ChildNodeId (o, node); 248 vertex reference ChildVertexId (vref, v); 249
// child reference ChildNodeId (cref, node). Every argument is read before the guard.
void ba_edge_field(BRepGraph& g,BrReader& r){
 auto e=be_id<BRepGraph_EdgeId>(r);int f=r.integer();uint32_t v=uint32_t(r.integer());
 auto m=g.Editor().Edges().Mut(e);BRepGraphInc::EdgeDef& d=m.Internal();
 switch(f){
  case 0:d.StartVertexRefId=BRepGraph_VertexRefId(v);break;
  case 1:d.EndVertexRefId=BRepGraph_VertexRefId(v);break;
  case 2:d.Curve3DRepId=BRepGraph_EdgeCurve3DRepId(v);break;
  default:d.Polygon3DRepId=BRepGraph_EdgePolygon3DRepId(v);break;}}
void ba_co_edge_field(BRepGraph& g,BrReader& r){
 auto c=be_id<BRepGraph_CoEdgeId>(r);int f=r.integer();uint32_t v=uint32_t(r.integer());
 auto m=g.Editor().CoEdges().Mut(c);BRepGraphInc::CoEdgeDef& d=m.Internal();
 switch(f){
  case 0:d.ChildEdgeId=BRepGraph_EdgeId(v);break;
  case 1:d.FaceId=BRepGraph_FaceId(v);break;
  case 2:d.Curve2DRepId=BRepGraph_CoEdgeCurve2DRepId(v);break;
  case 3:d.Polygon2DRepId=BRepGraph_CoEdgePolygon2DRepId(v);break;
  case 4:d.PolygonOnTriRepId=BRepGraph_CoEdgePolygonOnTriRepId(v);break;
  default:d.ParentWireId=BRepGraph_WireId(v);break;}}
void ba_face_field(BRepGraph& g,BrReader& r){
 auto x=be_id<BRepGraph_FaceId>(r);int f=r.integer();uint32_t v=uint32_t(r.integer());
 auto m=g.Editor().Faces().Mut(x);BRepGraphInc::FaceDef& d=m.Internal();
 if(f==0)d.SurfaceRepId=BRepGraph_FaceSurfaceRepId(v);else d.TriangulationRepId=BRepGraph_FaceTriangulationRepId(v);}
bool ba_corrupt_action(BRepGraph& g,BrReader& r,int code){
 switch(code){
  case 244:ba_edge_field(g,r);break;
  case 245:ba_co_edge_field(g,r);break;
  case 246:ba_face_field(g,r);break;
  case 247:{auto o=be_id<BRepGraph_OccurrenceId>(r);BRepGraph_NodeId n=bl_node(r);
   auto m=g.Editor().Occurrences().Mut(o);m.Internal().ChildNodeId=n;break;}
  case 248:{auto x=be_id<BRepGraph_VertexRefId>(r);auto v=be_id<BRepGraph_VertexId>(r);
   auto m=g.Editor().Vertices().MutRef(x);m.Internal().ChildVertexId=v;break;}
  case 249:{auto x=be_id<BRepGraph_ChildRefId>(r);BRepGraph_NodeId n=bl_node(r);
   auto m=g.Editor().Gen().MutChildRef(x);m.Internal().ChildNodeId=n;break;}
  default:return false;}
 br_word("-");return true;}
// 250 Validate (how: 0 Perform(g), 1 Mode::Lightweight, 2 Mode::Audit, 3
// Options::Lightweight(), 4 Options::Audit()); 251 MoveRef(child reference, trsf); 252
// MoveRef(occurrence reference, trsf).
bool ba_algo_action(BRepGraph& g,BrReader& r,int code){
 if(ba_corrupt_action(g,r,code))return true;
 switch(code){
  case 250:ba_report(ba_validate(g,r.integer()));return true;
  case 251:{auto x=be_id<BRepGraph_ChildRefId>(r);gp_Trsf t=ba_trsf(r);br_flag(BRepGraph_Transform::MoveRef(g,x,t));return true;}
  case 252:{auto x=be_id<BRepGraph_OccurrenceRefId>(r);gp_Trsf t=ba_trsf(r);br_flag(BRepGraph_Transform::MoveRef(g,x,t));return true;}
  default:return false;}}
