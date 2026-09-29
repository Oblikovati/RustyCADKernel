// The mesh view report of `br_bcache` (tkbrep_probe_brepgraph_cache.cxx): the counts,
// then the cache, persistent and effective queries of every face, edge and coedge (one
// past each end too), the persistent (edge, face) polygons on triangulation included.
void bc_view_cache(const BrScene& s,const BRepGraph& g){
 const BrGeometry& G=s.geometry;const auto& v=g.Mesh().Cache();
 for(uint32_t i=0;i<=g.Topo().Faces().Nb();++i){BRepGraph_FaceId f(i);br_flag(v.Faces().Has(f));br_int(br_id(G.triangulations,v.Faces().Triangulation(f)));br_flag(v.Faces().Entry(f)!=nullptr);}
 for(uint32_t i=0;i<=g.Topo().Edges().Nb();++i){BRepGraph_EdgeId e(i);br_flag(v.Edges().Has(e));br_int(br_id(G.polygons3d,v.Edges().Polygon3D(e)));br_flag(v.Edges().Entry(e)!=nullptr);}
 for(uint32_t i=0;i<=g.Topo().CoEdges().Nb();++i){BRepGraph_CoEdgeId x(i);br_flag(v.CoEdges().Has(x));
  br_flag(v.CoEdges().FindPolygon2D(x)!=nullptr);br_flag(v.CoEdges().FindPolygonOnTri(x)!=nullptr);br_flag(v.CoEdges().FindRaw(x)!=nullptr);}}
void bc_view_persistent(const BrScene& s,const BRepGraph& g){
 const BrGeometry& G=s.geometry;const auto& v=g.Mesh().Persistent();
 for(uint32_t i=0;i<=g.Topo().Faces().Nb();++i){BRepGraph_FaceId f(i);br_flag(v.Faces().Has(f));br_int(br_id(G.triangulations,v.Faces().Triangulation(f)));}
 for(uint32_t i=0;i<=g.Topo().Edges().Nb();++i){BRepGraph_EdgeId e(i);br_flag(v.Edges().Has(e));br_int(br_id(G.polygons3d,v.Edges().Polygon3D(e)));
  for(uint32_t j=0;j<g.Topo().Faces().Nb();++j){BRepGraph_FaceId f(j);br_flag(v.Edges().HasPolygonOnTriangulation(e,f));br_int(br_id(G.nodes,v.Edges().PolygonOnTriangulation(e,f)));}}
 for(uint32_t i=0;i<=g.Topo().CoEdges().Nb();++i){BRepGraph_CoEdgeId x(i);br_flag(v.CoEdges().Has(x));br_int(br_id(G.polygons2d,v.CoEdges().PolygonOnSurface(x)));
  br_flag(v.CoEdges().HasPolygonOnTriangulation(x));br_int(br_id(G.nodes,v.CoEdges().PolygonOnTriangulation(x)));}}
void bc_view_effective(const BrScene& s,const BRepGraph& g){
 const BrGeometry& G=s.geometry;const auto& v=g.Mesh().Effective();
 for(uint32_t i=0;i<=g.Topo().Faces().Nb();++i){BRepGraph_FaceId f(i);br_flag(v.Faces().Has(f));br_int(br_id(G.triangulations,v.Faces().Triangulation(f)));}
 for(uint32_t i=0;i<=g.Topo().Edges().Nb();++i){BRepGraph_EdgeId e(i);br_flag(v.Edges().Has(e));br_int(br_id(G.polygons3d,v.Edges().Polygon3D(e)));}
 for(uint32_t i=0;i<=g.Topo().CoEdges().Nb();++i){BRepGraph_CoEdgeId x(i);br_flag(v.CoEdges().Has(x));
  br_flag(v.CoEdges().HasPolygonOnSurface(x));br_int(br_id(G.polygons2d,v.CoEdges().PolygonOnSurface(x)));
  br_flag(v.CoEdges().HasPolygonOnTriangulation(x));br_int(br_id(G.nodes,v.CoEdges().PolygonOnTriangulation(x)));}}
void bc_view_report(const BrScene& s,const BRepGraph& g){
 const auto& p=g.Mesh().Poly();
 br_int(p.NbFaceTriangulations());br_int(p.NbEdgePolygons3D());br_int(p.NbCoEdgePolygons2D());br_int(p.NbCoEdgePolygonsOnTri());
 br_int(p.NbActiveTriangulations());br_int(p.NbActivePolygons3D());br_int(p.NbActivePolygons2D());br_int(p.NbActivePolygonsOnTri());
 bc_view_cache(s,g);bc_view_persistent(s,g);bc_view_effective(s,g);}
// `br_bcache script nactions (action)...`: the actions, the reports of both graphs,
// then every driver's Fill log.
void br_bcache_case(std::istream& in){
 BrReader r{in};BrScene s=br_scene(r);BcState st;
 {BcGraphs gs;int n=r.integer();for(int i=0;i<n;++i)bc_action(gs,s,st,r);
  for(BRepGraph* g:{&gs.g,&gs.t}){bc_registries(*g);bc_lock_report(*g);bc_deferred_report(*g);bc_derived_report(*g);
   bc_mesh_report(s,*g);bc_view_report(s,*g);}}
 for(const auto& d:st.drivers){br_int(int(d->Events.size()));for(long long e:d->Events)br_int(e);}
 br_end();}
