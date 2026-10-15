// TopOpeBRepBuild_Tools probe operations; the Rust twin is
// crates/tkbool/examples/bl_probe/top_ope_brep_build_tools.rs.
//
// The shapes are shapes of a shape program (tktopalgo_probe_shapes.cxx); shapes print
// through pr_shape (tkprim_probe_shape.cxx), states as TopAbs_State values, map keys as
// their index in the program (tp_same_index).
#include <TopOpeBRepBuild_Tools.hxx>
#include <TopOpeBRepDS_ShapeWithState.hxx>
#include <TopOpeBRepTool_ShapeClassifier.hxx>
// `program face edge edge2`: GetNormalToFaceOnEdge and GetNormalInNearestPoint of the
// face at the edge, GetTangentToEdge of the edge, GetTangentToEdgeEdge of the edge on
// edge2 (flag and tangent), IsDegEdgesTheSame(edge, edge2), CheckFaceClosed2d and
// NormalizeFace of the face.
void bl_tb_tools_geom_case(std::istream& input){
 GaReader r{input};TpShapes t;
 try{t=tp_shapes(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 const TopoDS_Shape f=t.shapes.at(r.integer()),e=t.shapes.at(r.integer()),e2=t.shapes.at(r.integer());
 GA_TRY(gp_Vec n;TopOpeBRepBuild_Tools::GetNormalToFaceOnEdge(TopoDS::Face(f),TopoDS::Edge(e),n);ga_xyz(n.XYZ()))
 GA_TRY(gp_Vec n;TopOpeBRepBuild_Tools::GetNormalInNearestPoint(TopoDS::Face(f),TopoDS::Edge(e),n);ga_xyz(n.XYZ()))
 GA_TRY(gp_Vec v;ga_flag(TopOpeBRepBuild_Tools::GetTangentToEdge(TopoDS::Edge(e),v));ga_xyz(v.XYZ()))
 GA_TRY(gp_Vec v;ga_flag(TopOpeBRepBuild_Tools::GetTangentToEdgeEdge(TopoDS::Face(f),TopoDS::Edge(e),TopoDS::Edge(e2),v));ga_xyz(v.XYZ()))
 GA_TRY(ga_flag(TopOpeBRepBuild_Tools::IsDegEdgesTheSame(e,e2)))
 GA_TRY(ga_flag(TopOpeBRepBuild_Tools::CheckFaceClosed2d(TopoDS::Face(f))))
 GA_TRY(TopoDS_Shape nf;TopOpeBRepBuild_Tools::NormalizeFace(f,nf);pr_shape(nf))
 ga_end();}
// The entries of a map of shapes with states: the extent, then per entry the program
// index of the key, its state and its split flag.
void bl_tb_dump_sws(const TpShapes& t,
                    const NCollection_IndexedDataMap<TopoDS_Shape,TopOpeBRepDS_ShapeWithState,TopTools_ShapeMapHasher>& m){
 ga_int(m.Extent());
 for(int i=1;i<=m.Extent();i++){ga_int(tp_same_index(t,m.FindKey(i)));ga_int(m(i).State());ga_flag(m(i).IsSplitted());}}
// `program shape ref avoid`: with a classifier on the reference, FindStateThroughVertex
// of the shape (avoiding the avoid shape when >= 0) printing its state and the map; then
// on a fresh map PropagateState from the first face of the shape IN to its other faces
// through their edges and PropagateStateForWires of those faces, printing the map.
void bl_tb_tools_states_case(std::istream& input){
 GaReader r{input};TpShapes t;
 try{t=tp_shapes(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 const TopoDS_Shape s=t.shapes.at(r.integer()),ref=t.shapes.at(r.integer());int av=r.integer();
 NCollection_Map<TopoDS_Shape,TopTools_ShapeMapHasher> avoid;
 if(av>=0)avoid.Add(t.shapes.at(av));
 GA_TRY(TopOpeBRepTool_ShapeClassifier sc;sc.SetReference(ref);
  NCollection_IndexedDataMap<TopoDS_Shape,TopOpeBRepDS_ShapeWithState,TopTools_ShapeMapHasher> m;
  ga_int(TopOpeBRepBuild_Tools::FindStateThroughVertex(s,sc,m,avoid));bl_tb_dump_sws(t,m))
 GA_TRY(TopOpeBRepTool_ShapeClassifier sc;sc.SetReference(ref);
  NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> faces,rest;TopExp::MapShapes(s,TopAbs_FACE,faces);
  NCollection_DataMap<TopoDS_Shape,TopAbs_State,TopTools_ShapeMapHasher> spl;spl.Bind(faces(1),TopAbs_IN);
  for(int i=2;i<=faces.Extent();i++)rest.Add(faces(i));
  NCollection_IndexedDataMap<TopoDS_Shape,TopOpeBRepDS_ShapeWithState,TopTools_ShapeMapHasher> m;
  TopOpeBRepBuild_Tools::PropagateState(spl,rest,TopAbs_EDGE,TopAbs_FACE,sc,m,avoid);
  TopOpeBRepBuild_Tools::PropagateStateForWires(rest,m);bl_tb_dump_sws(t,m))
 ga_end();}
// `program shape maxtol`: CorrectTolerances(shape, maxtol), then the tolerances of the
// vertices and the edges of the shape (TopExp::MapShapes order).
void bl_tb_tools_tolerances_case(std::istream& input){
 GaReader r{input};TpShapes t;
 try{t=tp_shapes(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 const TopoDS_Shape s=t.shapes.at(r.integer());double maxtol=r.real();
 GA_TRY(TopOpeBRepBuild_Tools::CorrectTolerances(s,maxtol))
 NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> vs,es;
 TopExp::MapShapes(s,TopAbs_VERTEX,vs);TopExp::MapShapes(s,TopAbs_EDGE,es);
 ga_int(vs.Extent());for(int i=1;i<=vs.Extent();i++)ga_real(BRep_Tool::Tolerance(TopoDS::Vertex(vs(i))));
 ga_int(es.Extent());for(int i=1;i<=es.Extent();i++)ga_real(BRep_Tool::Tolerance(TopoDS::Edge(es(i))));
 ga_end();}
