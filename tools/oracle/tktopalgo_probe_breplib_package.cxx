// BRepLib package-function probe operations (the BRepLib class, BRepLib_MakeShell,
// BRepLib_CheckCurveOnSurface and BRepLib_ToolTriangulatedShape); the Rust twin is
// crates/tktopalgo/examples/tp_probe/brep_lib_package.rs. Every operation reads a shape
// record (tktopalgo_probe_breplib_shapes.cxx) unless stated, prints its results and the
// digest of the shape after it.
//
// `blp_build3d shape mode tol cont maxdeg maxseg`: mode 0 BuildCurves3d(S, tol, cont,
//   maxdeg, maxseg), 1 BuildCurves3d(S), 2 BuildCurve3d on each edge (MapShapes order).
// `blp_samerange shape confusion tol`: per edge CheckSameRange, SameRange, CheckSameRange.
// `blp_edgetol shape mode min max`: mode 0 UpdateEdgeTolerance(S), 1 UpdateEdgeTol per edge.
// `blp_regularity shape mode tolang n (edge index)...`: mode 0 EncodeRegularity(S),
//   1 EncodeRegularity(S, edges), 2 EncodeRegularity(E, F1, F2) per edge; then the
//   continuity of every edge between its faces.
// `blp_continuity shape tolang`: ContinuityOfFaces per edge between its faces.
// `blp_inner shape`: UpdateInnerTolerances.
// `blp_sort shape`: SortFaces and ReverseSortFaces (face indices).
// `blp_bounding vertices n (index)...`: BoundingVertex.
// `blp_validrange shape`: FindValidRange per edge.
// `blp_validrange2 curve first last tolE (par x y z tol) (par x y z tol)`: FindValidRange on
//   the GeomAdaptor_Curve of a Geom curve record between two parameters.
// `blp_pcurveplane shape mode`: per face and edge BuildPCurveForEdgeOnPlane (mode 0 the
//   query, 1 the update, 2 the template on the edges of the face).
// `blp_ccos shape parallel`: BRepLib_CheckCurveOnSurface per edge and face.
// `blp_shell surface bounded (u1 u2 v1 v2) segment mode`: BRepLib_MakeShell (mode 0 the
//   constructors, 1 the default one then Init).
// `blp_normals shape n uv angtol force`: the mesh then EnsureNormalConsistency.
// `blp_deflection shape n`: the mesh then UpdateDeflection.
// `blp_trinormals shape n uv connect`: the mesh then ComputeNormals per face.
#include <BRepLib_CheckCurveOnSurface.hxx>
#include <BRepLib_ToolTriangulatedShape.hxx>
#include <Poly_Connect.hxx>
using TpShapeMap=NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher>;
TpShapeMap tp_sub_shapes(const TopoDS_Shape& s,TopAbs_ShapeEnum t){TpShapeMap m;if(!s.IsNull())TopExp::MapShapes(s,t,m);return m;}
// The two faces of an edge as EncodeRegularity finds them (the first two distinct
// ancestors, or the first one twice for a seam); false when there are none.
bool tp_edge_faces(const TopoDS_Edge& e,const NCollection_List<TopoDS_Shape>& anc,TopoDS_Face& f1,TopoDS_Face& f2){
 f1.Nullify();
 for(NCollection_List<TopoDS_Shape>::Iterator it(anc);it.More();it.Next()){
  if(f1.IsNull())f1=TopoDS::Face(it.Value());else if(!f1.IsSame(it.Value())){f2=TopoDS::Face(it.Value());return true;}}
 if(f1.IsNull())return false;
 for(TopExp_Explorer x(f1,TopAbs_EDGE);x.More();x.Next())if(e.IsSame(x.Current())&&e.Orientation()!=x.Current().Orientation()){f2=f1;return true;}
 return false;}
using TpAncestors=NCollection_IndexedDataMap<TopoDS_Shape,NCollection_List<TopoDS_Shape>,TopTools_ShapeMapHasher>;
TpAncestors tp_edge_ancestors(const TopoDS_Shape& s){TpAncestors m;if(!s.IsNull())TopExp::MapShapesAndAncestors(s,TopAbs_EDGE,TopAbs_FACE,m);return m;}
void tp_blp_build3d_case(std::istream& input){
 GaReader r{input};
 try{TopoDS_Shape s=tp_shape_record(r);int mode=r.integer();double tol=r.real();GeomAbs_Shape cont=GeomAbs_Shape(r.integer());int deg=r.integer(),seg=r.integer();
  if(mode==0)ga_flag(BRepLib::BuildCurves3d(s,tol,cont,deg,seg));
  else if(mode==1)ga_flag(BRepLib::BuildCurves3d(s));
  else{TpShapeMap m=tp_sub_shapes(s,TopAbs_EDGE);for(int i=1;i<=m.Extent();++i)GA_TRY(ga_flag(BRepLib::BuildCurve3d(TopoDS::Edge(m(i)),tol,cont,deg,seg)))}
  tp_shape(s);
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 ga_end();}
void tp_blp_samerange_case(std::istream& input){
 GaReader r{input};
 try{TopoDS_Shape s=tp_shape_record(r);double conf=r.real(),tol=r.real();TpShapeMap m=tp_sub_shapes(s,TopAbs_EDGE);
  for(int i=1;i<=m.Extent();++i){const TopoDS_Edge& e=TopoDS::Edge(m(i));
   GA_TRY(ga_flag(BRepLib::CheckSameRange(e,conf));BRepLib::SameRange(e,tol);ga_flag(BRepLib::CheckSameRange(e,conf)))}
  tp_shape(s);
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 ga_end();}
void tp_blp_edgetol_case(std::istream& input){
 GaReader r{input};
 try{TopoDS_Shape s=tp_shape_record(r);int mode=r.integer();double mn=r.real(),mx=r.real();
  if(mode==0)ga_flag(BRepLib::UpdateEdgeTolerance(s,mn,mx));
  else{TpShapeMap m=tp_sub_shapes(s,TopAbs_EDGE);for(int i=1;i<=m.Extent();++i)GA_TRY(ga_flag(BRepLib::UpdateEdgeTol(TopoDS::Edge(m(i)),mn,mx)))}
  tp_shape(s);
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 ga_end();}
// The continuity of every edge (MapShapesAndAncestors order) between its two faces.
void tp_edge_continuities(const TopoDS_Shape& s){
 TpAncestors m=tp_edge_ancestors(s);
 for(int i=1;i<=m.Extent();++i){TopoDS_Face f1,f2;const TopoDS_Edge& e=TopoDS::Edge(m.FindKey(i));
  if(!tp_edge_faces(e,m(i),f1,f2)){ga_int(-1);continue;}GA_TRY(ga_int(BRep_Tool::Continuity(e,f1,f2)))}}
void tp_blp_regularity_case(std::istream& input){
 GaReader r{input};
 try{TopoDS_Shape s=tp_shape_record(r);int mode=r.integer();double tol=r.real();int n=r.integer();
  TpShapeMap edges=tp_sub_shapes(s,TopAbs_EDGE);NCollection_List<TopoDS_Shape> list;
  for(int i=0;i<n;++i)list.Append(edges(r.integer()));
  if(mode==0)BRepLib::EncodeRegularity(s,tol);
  else if(mode==1)BRepLib::EncodeRegularity(s,list,tol);
  else{TpAncestors m=tp_edge_ancestors(s);for(int i=1;i<=m.Extent();++i){TopoDS_Face f1,f2;TopoDS_Edge e=TopoDS::Edge(m.FindKey(i));
   if(tp_edge_faces(e,m(i),f1,f2))GA_TRY(BRepLib::EncodeRegularity(e,f1,f2,tol))}}
  tp_edge_continuities(s);
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 ga_end();}
void tp_blp_continuity_case(std::istream& input){
 GaReader r{input};
 try{TopoDS_Shape s=tp_shape_record(r);double tol=r.real();TpAncestors m=tp_edge_ancestors(s);
  for(int i=1;i<=m.Extent();++i){TopoDS_Face f1,f2;const TopoDS_Edge& e=TopoDS::Edge(m.FindKey(i));
   if(!tp_edge_faces(e,m(i),f1,f2)){ga_int(-1);continue;}GA_TRY(ga_int(BRepLib::ContinuityOfFaces(e,f1,f2,tol)))}
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 ga_end();}
void tp_blp_inner_case(std::istream& input){
 GaReader r{input};
 try{TopoDS_Shape s=tp_shape_record(r);BRepLib::UpdateInnerTolerances(s);tp_shape(s);
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 ga_end();}
void tp_blp_sort_case(std::istream& input){
 GaReader r{input};
 try{TopoDS_Shape s=tp_shape_record(r);TpShapeMap faces=tp_sub_shapes(s,TopAbs_FACE);NCollection_List<TopoDS_Shape> lf;
  BRepLib::SortFaces(s,lf);ga_int(lf.Extent());for(NCollection_List<TopoDS_Shape>::Iterator it(lf);it.More();it.Next())ga_int(faces.FindIndex(it.Value()));
  BRepLib::ReverseSortFaces(s,lf);ga_int(lf.Extent());for(NCollection_List<TopoDS_Shape>::Iterator it(lf);it.More();it.Next())ga_int(faces.FindIndex(it.Value()));
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 ga_end();}
void tp_blp_bounding_case(std::istream& input){
 GaReader r{input};std::vector<TopoDS_Vertex> vs=tp_vertices(r);int n=r.integer();NCollection_List<TopoDS_Shape> list;
 for(int i=0;i<n;++i)list.Append(vs.at(r.integer()));
 gp_Pnt c(-7.,-7.,-7.);double tol=-7.;
 GA_TRY(BRepLib::BoundingVertex(list,c,tol);ga_pnt(c);ga_real(tol))
 ga_end();}
void tp_blp_validrange_case(std::istream& input){
 GaReader r{input};
 try{TopoDS_Shape s=tp_shape_record(r);TpShapeMap m=tp_sub_shapes(s,TopAbs_EDGE);
  for(int i=1;i<=m.Extent();++i){double f=-7.,l=-7.;GA_TRY(ga_flag(BRepLib::FindValidRange(TopoDS::Edge(m(i)),f,l));ga_real(f);ga_real(l))}
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 ga_end();}
void tp_blp_validrange2_case(std::istream& input){
 GaReader r{input};
 try{occ::handle<Geom_Curve> c=tp_geom_curve(r);double a=r.real(),b=r.real();GeomAdaptor_Curve ac(c,a,b);double tolE=r.real();
  double p1=r.real();gp_Pnt v1=r.pnt();double t1=r.real();double p2=r.real();gp_Pnt v2=r.pnt();double t2=r.real();
  double f=-7.,l=-7.;ga_flag(BRepLib::FindValidRange(ac,tolE,p1,v1,t1,p2,v2,t2,f,l));ga_real(f);ga_real(l);
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 ga_end();}
void tp_blp_pcurveplane_case(std::istream& input){
 GaReader r{input};
 try{TopoDS_Shape s=tp_shape_record(r);int mode=r.integer();TpShapeMap faces=tp_sub_shapes(s,TopAbs_FACE);
  for(int i=1;i<=faces.Extent();++i){const TopoDS_Face& f=TopoDS::Face(faces(i));TpShapeMap edges=tp_sub_shapes(f,TopAbs_EDGE);
   if(mode==2){NCollection_List<TopoDS_Shape> list;for(int k=1;k<=edges.Extent();++k)list.Append(edges(k));
    GA_TRY(BRepLib::BuildPCurveForEdgesOnPlane(list,f))continue;}
   for(int k=1;k<=edges.Extent();++k){const TopoDS_Edge& e=TopoDS::Edge(edges(k));
    if(mode==1){GA_TRY(BRepLib::BuildPCurveForEdgeOnPlane(e,f))continue;}
    occ::handle<Geom2d_Curve> c;bool upd=false;GA_TRY(BRepLib::BuildPCurveForEdgeOnPlane(e,f,c,upd);ga_flag(upd);tp_type(c))}}
  tp_shape(s);
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 ga_end();}
void tp_blp_ccos_case(std::istream& input){
 GaReader r{input};
 try{TopoDS_Shape s=tp_shape_record(r);bool parallel=r.flag();TpAncestors m=tp_edge_ancestors(s);BRepLib_CheckCurveOnSurface reused;
  for(int i=1;i<=m.Extent();++i){const TopoDS_Edge& e=TopoDS::Edge(m.FindKey(i));
   for(NCollection_List<TopoDS_Shape>::Iterator it(m(i));it.More();it.Next()){
    GA_TRY(BRepLib_CheckCurveOnSurface c(e,TopoDS::Face(it.Value()));c.SetParallel(parallel);ga_flag(c.IsParallel());c.Perform();
     ga_flag(c.IsDone());ga_int(c.ErrorStatus());ga_real(c.MaxDistance());ga_real(c.MaxParameter()))
    GA_TRY(reused.Init(e,TopoDS::Face(it.Value()));reused.Perform();ga_int(reused.ErrorStatus());ga_real(reused.MaxDistance()))}}
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 ga_end();}
void tp_blp_shell_case(std::istream& input){
 GaReader r{input};
 try{occ::handle<Geom_Surface> s=tp_surface(r);bool bounded=r.flag();double u1=0.,u2=0.,v1=0.,v2=0.;
  if(bounded){u1=r.real();u2=r.real();v1=r.real();v2=r.real();}bool segment=r.flag();int mode=r.integer();
  std::unique_ptr<BRepLib_MakeShell> m;
  if(mode==1){m.reset(new BRepLib_MakeShell());ga_flag(m->IsDone());ga_int(m->Error());
   if(!bounded)s->Bounds(u1,u2,v1,v2);m->Init(s,u1,u2,v1,v2,segment);}
  else if(bounded)m.reset(new BRepLib_MakeShell(s,u1,u2,v1,v2,segment));else m.reset(new BRepLib_MakeShell(s,segment));
  ga_flag(m->IsDone());ga_int(m->Error());tp_shape(m->Shell());
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 ga_end();}
void tp_blp_normals_case(std::istream& input){
 GaReader r{input};
 try{TopoDS_Shape s=tp_shape_record(r);int n=r.integer();bool uv=r.flag();double ang=r.real();bool force=r.flag();tp_mesh(s,n,uv);
  GA_TRY(ga_flag(BRepLib::EnsureNormalConsistency(s,ang,force)))tp_normals(s);
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 ga_end();}
void tp_blp_deflection_case(std::istream& input){
 GaReader r{input};
 try{TopoDS_Shape s=tp_shape_record(r);int n=r.integer();tp_mesh(s,n,r.flag());BRepLib::UpdateDeflection(s);TpShapeMap faces=tp_sub_shapes(s,TopAbs_FACE);
  for(int i=1;i<=faces.Extent();++i){TopLoc_Location l;occ::handle<Poly_Triangulation> t=BRep_Tool::Triangulation(TopoDS::Face(faces(i)),l);
   if(t.IsNull())ga_int(-1);else ga_real(t->Deflection());}
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 ga_end();}
void tp_blp_trinormals_case(std::istream& input){
 GaReader r{input};
 try{TopoDS_Shape s=tp_shape_record(r);int n=r.integer();bool uv=r.flag();bool connect=r.flag();tp_mesh(s,n,uv);TpShapeMap faces=tp_sub_shapes(s,TopAbs_FACE);
  Poly_Connect shared;
  for(int i=1;i<=faces.Extent();++i){const TopoDS_Face& f=TopoDS::Face(faces(i));TopLoc_Location l;occ::handle<Poly_Triangulation> t=BRep_Tool::Triangulation(f,l);
   if(connect)GA_TRY(BRepLib_ToolTriangulatedShape::ComputeNormals(f,t,shared))else GA_TRY(BRepLib_ToolTriangulatedShape::ComputeNormals(f,t))}
  tp_normals(s);
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 ga_end();}
