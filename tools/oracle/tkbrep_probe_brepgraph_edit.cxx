// BRepGraph editor (phase 2, wave 2b editor area): a graph script drives the editor
// operation groups, the mutation guards, deferred invalidation, the supplement editor and
// RemoveShape, mixed with the `br_blayer` layer and cache actions, then reports the graph,
// the editor state, the logging layers, history, supplement, caches, the mesh cache and
// every node's shapes. The Rust twin is crates/tkbrep/examples/br_probe/brep_graph_edit*.rs.
#include <BRepGraph_DeferredScope.hxx>
#include <BRepGraph_EditorView.hxx>
#include <BRepGraph_MutGuard.hxx>
#include <BRepGraph_SupplementEditor.hxx>
// Typed ids as `index`, orientations as a TopAbs value (INTERNAL and EXTERNAL raise).
template<class T> T be_id(BrReader& r){return T(uint32_t(r.integer()));}
BRepGraphInc::ParityOrientation be_parity(int o){return BRepGraphInc::ParityOrientation(TopAbs_Orientation(o));}
BRepGraphInc::ParityOrientation be_o(BrReader& r){return be_parity(r.integer());}
// An owning copy: LinearVector::ToArray1 wraps the vector's buffer, which a returned
// array would outlive.
template<class T> NCollection_Array1<T> be_array(const NCollection_LinearVector<T>& v){
 if(v.IsEmpty())return NCollection_Array1<T>();
 NCollection_Array1<T> a(0,int(v.Size())-1);for(size_t i=0;i<v.Size();++i)a.SetValue(int(i),v.Value(i));return a;}
template<class T> NCollection_Array1<T> be_ids(BrReader& r){return be_array(bl_list<T>(r,be_id<T>));}
NCollection_Array1<BRepGraphInc::ParityOrientation> be_os(BrReader& r){return be_array(bl_list<BRepGraphInc::ParityOrientation>(r,be_o));}
template<class T> void be_print(const T& id){br_int(id.Index);}
template<class A> void be_prints(const A& v){br_int(v.Size());for(const auto& id:v)be_print(id);}
void be_action(BRepGraph& g,const BrScene& s,BlState& st,BrReader& r);
// 120-134: node creation. 120 Vertices().Add(P, tol), 121 Edges().Add(v1, v2, curve,
// first, last, tol), 122 CoEdges().Add(edge, o), 123 CoEdges().Add(edge, face, pcurve,
// first, last, o), 124 Wires().Add(coedges), 125 Faces().Add(surface, outer, inners, tol),
// 126 Shells().Add(), 127 Solids().Add(), 128 Compounds().Add(nodes), 129
// CompSolids().Add(solids), 130 Products().Add(root, location), 131 Products().Add(),
// 132 Products().Append(parent, child, location, parent occurrence) prints the occurrence
// and its reference, 133 Products().Append(parent, children, locations), 134
// AppendDocumentRoot. Geometry and locations are scene registry indices.
bool be_add_action(BRepGraph& g,const BrScene& s,BrReader& r,int code){
 BRepGraph::EditorView& e=g.Editor();const BrGeometry& G=s.geometry;
 switch(code){
  case 120:{gp_Pnt p=br_point(r);be_print(e.Vertices().Add(p,r.real()));break;}
  case 121:{auto a=be_id<BRepGraph_VertexId>(r),b=be_id<BRepGraph_VertexId>(r);const auto& c=br_at(G.curves,r.integer());
   double f=r.real(),l=r.real();be_print(e.Edges().Add(a,b,c,f,l,r.real()));break;}
  case 122:{auto x=be_id<BRepGraph_EdgeId>(r);be_print(e.CoEdges().Add(x,be_o(r)));break;}
  case 123:{auto x=be_id<BRepGraph_EdgeId>(r);auto f=be_id<BRepGraph_FaceId>(r);const auto& c=br_at(G.pcurves,r.integer());
   double a=r.real(),b=r.real();be_print(e.CoEdges().Add(x,f,c,a,b,be_o(r)));break;}
  case 124:be_print(e.Wires().Add(be_ids<BRepGraph_CoEdgeId>(r)));break;
  case 125:{const auto& c=br_at(G.surfaces,r.integer());auto w=be_id<BRepGraph_WireId>(r);auto in=be_ids<BRepGraph_WireId>(r);
   be_print(e.Faces().Add(c,w,in,r.real()));break;}
  case 126:be_print(e.Shells().Add());break;
  case 127:be_print(e.Solids().Add());break;
  case 128:be_print(e.Compounds().Add(bl_list<BRepGraph_NodeId>(r,bl_node).ToArray1()));break;
  case 129:be_print(e.CompSolids().Add(be_ids<BRepGraph_SolidId>(r)));break;
  case 130:{BRepGraph_NodeId n=bl_node(r);be_print(e.Products().Add(n,s.graph.loc(r.integer())));break;}
  case 131:be_print(e.Products().Add());break;
  case 132:{auto p=be_id<BRepGraph_ProductId>(r),c=be_id<BRepGraph_ProductId>(r);TopLoc_Location l=s.graph.loc(r.integer());
   auto o=be_id<BRepGraph_OccurrenceId>(r);BRepGraph_OccurrenceRefId x;be_print(e.Products().Append(p,c,l,o,&x));be_print(x);break;}
  case 133:{auto p=be_id<BRepGraph_ProductId>(r);auto c=be_ids<BRepGraph_ProductId>(r);
   auto l=bl_list<TopLoc_Location>(r,[&s](BrReader& x){return s.graph.loc(x.integer());});be_prints(e.Products().Append(p,c,l.ToArray1()));break;}
  case 134:e.Products().AppendDocumentRoot(be_id<BRepGraph_ProductId>(r));br_word("-");break;
  default:return false;}
 return true;}
// 135-155: containers. 135 Faces().Append(face, wire, o), 136 Shells().Append(shell, face,
// o), 137 the batch form (shell, faces, os), 138 Solids().Append(solid, shell, o), 139 the
// batch form, 140 Compounds().Append(compound, node, o), 141 the batch form, 142
// CompSolids().Append(compsolid, solid, o), 143 the batch form, 144 Faces().RemoveWire,
// 145 Shells().RemoveFace, 146 RemoveFaces, 147 Solids().RemoveShell, 148 RemoveShells,
// 149 Compounds().RemoveChild, 150 RemoveChildren, 151 CompSolids().RemoveSolid, 152
// RemoveSolids, 153 Products().RemoveOccurrence, 154 RemoveOccurrences, 155 RemoveShapeRoot.
bool be_container_action(BRepGraph& g,BrReader& r,int code){
 BRepGraph::EditorView& e=g.Editor();
 switch(code){
  case 135:{auto f=be_id<BRepGraph_FaceId>(r);auto w=be_id<BRepGraph_WireId>(r);be_print(e.Faces().Append(f,w,be_o(r)));break;}
  case 136:{auto x=be_id<BRepGraph_ShellId>(r);auto f=be_id<BRepGraph_FaceId>(r);be_print(e.Shells().Append(x,f,be_o(r)));break;}
  case 137:{auto x=be_id<BRepGraph_ShellId>(r);auto f=be_ids<BRepGraph_FaceId>(r);be_prints(e.Shells().Append(x,f,be_os(r)));break;}
  case 138:{auto x=be_id<BRepGraph_SolidId>(r);auto f=be_id<BRepGraph_ShellId>(r);be_print(e.Solids().Append(x,f,be_o(r)));break;}
  case 139:{auto x=be_id<BRepGraph_SolidId>(r);auto f=be_ids<BRepGraph_ShellId>(r);be_prints(e.Solids().Append(x,f,be_os(r)));break;}
  case 140:{auto x=be_id<BRepGraph_CompoundId>(r);BRepGraph_NodeId n=bl_node(r);be_print(e.Compounds().Append(x,n,be_o(r)));break;}
  case 141:{auto x=be_id<BRepGraph_CompoundId>(r);auto n=bl_list<BRepGraph_NodeId>(r,bl_node);be_prints(e.Compounds().Append(x,n.ToArray1(),be_os(r)));break;}
  case 142:{auto x=be_id<BRepGraph_CompSolidId>(r);auto f=be_id<BRepGraph_SolidId>(r);be_print(e.CompSolids().Append(x,f,be_o(r)));break;}
  case 143:{auto x=be_id<BRepGraph_CompSolidId>(r);auto f=be_ids<BRepGraph_SolidId>(r);be_prints(e.CompSolids().Append(x,f,be_os(r)));break;}
  case 144:{auto x=be_id<BRepGraph_FaceId>(r);br_flag(e.Faces().RemoveWire(x,be_id<BRepGraph_WireRefId>(r)));break;}
  case 145:{auto x=be_id<BRepGraph_ShellId>(r);br_flag(e.Shells().RemoveFace(x,be_id<BRepGraph_FaceRefId>(r)));break;}
  case 146:{auto x=be_id<BRepGraph_ShellId>(r);br_flag(e.Shells().RemoveFaces(x,be_ids<BRepGraph_FaceRefId>(r)));break;}
  case 147:{auto x=be_id<BRepGraph_SolidId>(r);br_flag(e.Solids().RemoveShell(x,be_id<BRepGraph_ShellRefId>(r)));break;}
  case 148:{auto x=be_id<BRepGraph_SolidId>(r);br_flag(e.Solids().RemoveShells(x,be_ids<BRepGraph_ShellRefId>(r)));break;}
  case 149:{auto x=be_id<BRepGraph_CompoundId>(r);br_flag(e.Compounds().RemoveChild(x,be_id<BRepGraph_ChildRefId>(r)));break;}
  case 150:{auto x=be_id<BRepGraph_CompoundId>(r);br_flag(e.Compounds().RemoveChildren(x,be_ids<BRepGraph_ChildRefId>(r)));break;}
  case 151:{auto x=be_id<BRepGraph_CompSolidId>(r);br_flag(e.CompSolids().RemoveSolid(x,be_id<BRepGraph_SolidRefId>(r)));break;}
  case 152:{auto x=be_id<BRepGraph_CompSolidId>(r);br_flag(e.CompSolids().RemoveSolids(x,be_ids<BRepGraph_SolidRefId>(r)));break;}
  case 153:{auto x=be_id<BRepGraph_ProductId>(r);br_flag(e.Products().RemoveOccurrence(x,be_id<BRepGraph_OccurrenceRefId>(r)));break;}
  case 154:{auto x=be_id<BRepGraph_ProductId>(r);br_flag(e.Products().RemoveOccurrences(x,be_ids<BRepGraph_OccurrenceRefId>(r)));break;}
  case 155:br_flag(e.Products().RemoveShapeRoot(be_id<BRepGraph_ProductId>(r)));break;
  default:return false;}
 return true;}
// 156-171: edges and vertices. 156 Split(edge, vertex, t) prints both halves, 157
// RemoveVertex(edge, vref), 158 ReplaceVertex(edge, vref, vertex), 159 Reverse, 160
// SetTolerance, 161 SetParamRange, 162 SetCurve(edge, curve, first, last), 163 ClearCurve,
// 164 SetPersistentPolygon3D(edge, polygon), 165 ClearPersistentPolygon3D, 166
// SetStartVertexRefId, 167 SetEndVertexRefId, 168 Vertices().SetPoint(v, P), 169
// SetTolerance, 170 SetRefOrientation(vref, o), 171 SetRefChildVertexId(vref, v).
bool be_edge_action(BRepGraph& g,const BrScene& s,BrReader& r,int code){
 if(code<156||code>171)return false;
 BRepGraph::EditorView::EdgeOps& e=g.Editor().Edges();BRepGraph::EditorView::VertexOps& v=g.Editor().Vertices();const BrGeometry& G=s.geometry;
 if(code==156){auto x=be_id<BRepGraph_EdgeId>(r);auto p=be_id<BRepGraph_VertexId>(r);BRepGraph_EdgeId a,b;e.Split(x,p,r.real(),a,b);be_print(a);be_print(b);return true;}
 if(code==157){auto x=be_id<BRepGraph_EdgeId>(r);br_flag(e.RemoveVertex(x,be_id<BRepGraph_VertexRefId>(r)));return true;}
 if(code==158){auto x=be_id<BRepGraph_EdgeId>(r);auto y=be_id<BRepGraph_VertexRefId>(r);be_print(e.ReplaceVertex(x,y,be_id<BRepGraph_VertexId>(r)));return true;}
 switch(code){
  case 159:e.Reverse(be_id<BRepGraph_EdgeId>(r));break;
  case 160:{auto x=be_id<BRepGraph_EdgeId>(r);e.SetTolerance(x,r.real());break;}
  case 161:{auto x=be_id<BRepGraph_EdgeId>(r);double f=r.real();e.SetParamRange(x,f,r.real());break;}
  case 162:{auto x=be_id<BRepGraph_EdgeId>(r);const auto& c=br_at(G.curves,r.integer());double f=r.real();e.SetCurve(x,c,f,r.real());break;}
  case 163:e.ClearCurve(be_id<BRepGraph_EdgeId>(r));break;
  case 164:{auto x=be_id<BRepGraph_EdgeId>(r);e.SetPersistentPolygon3D(x,br_at(G.polygons3d,r.integer()));break;}
  case 165:e.ClearPersistentPolygon3D(be_id<BRepGraph_EdgeId>(r));break;
  case 166:{auto x=be_id<BRepGraph_EdgeId>(r);e.SetStartVertexRefId(x,be_id<BRepGraph_VertexRefId>(r));break;}
  case 167:{auto x=be_id<BRepGraph_EdgeId>(r);e.SetEndVertexRefId(x,be_id<BRepGraph_VertexRefId>(r));break;}
  case 168:{auto x=be_id<BRepGraph_VertexId>(r);v.SetPoint(x,br_point(r));break;}
  case 169:{auto x=be_id<BRepGraph_VertexId>(r);v.SetTolerance(x,r.real());break;}
  case 170:{auto x=be_id<BRepGraph_VertexRefId>(r);v.SetRefOrientation(x,be_o(r));break;}
  default:{auto x=be_id<BRepGraph_VertexRefId>(r);v.SetRefChildVertexId(x,be_id<BRepGraph_VertexId>(r));break;}}
 br_word("-");return true;}
// 172-181: coedges. 172 SetParamRange, 173 SetOrientation, 174 SetPCurve(coedge, pcurve),
// 175 SetPCurve(coedge, pcurve, first, last), 176 ClearPCurve, 177
// SetPersistentPolygon2D, 178 SetPersistentPolygonOnTri, 179 ResetPCurveBinding, 180
// SetChildEdgeId, 181 SetFaceId.
bool be_co_edge_action(BRepGraph& g,const BrScene& s,BrReader& r,int code){
 if(code<172||code>181)return false;
 BRepGraph::EditorView::CoEdgeOps& e=g.Editor().CoEdges();const BrGeometry& G=s.geometry;auto x=be_id<BRepGraph_CoEdgeId>(r);
 switch(code){
  case 172:{double f=r.real();e.SetParamRange(x,f,r.real());break;}
  case 173:e.SetOrientation(x,be_o(r));break;
  case 174:e.SetPCurve(x,br_at(G.pcurves,r.integer()));break;
  case 175:{const auto& c=br_at(G.pcurves,r.integer());double f=r.real();e.SetPCurve(x,c,f,r.real());break;}
  case 176:e.ClearPCurve(x);break;
  case 177:e.SetPersistentPolygon2D(x,br_at(G.polygons2d,r.integer()));break;
  case 178:e.SetPersistentPolygonOnTri(x,br_at(G.nodes,r.integer()));break;
  case 179:e.ResetPCurveBinding(x);break;
  case 180:e.SetChildEdgeId(x,be_id<BRepGraph_EdgeId>(r));break;
  default:e.SetFaceId(x,be_id<BRepGraph_FaceId>(r));break;}
 br_word("-");return true;}
#include "tkbrep_probe_brepgraph_edit_rest.cxx"
