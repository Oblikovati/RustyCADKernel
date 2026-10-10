// TopOpeBRepBuild face chain probe operations (WireEdgeClassifier, FaceBuilder, the
// MakeFaces of Builder and WireToFace); the Rust twin is
// crates/tkbool/examples/bl_probe/top_ope_brep_build.rs.
//
// The shapes are shapes of a shape program (tktopalgo_probe_shapes.cxx); shapes print
// through pr_shape (tkprim_probe_shape.cxx), states as TopAbs_State values.
#include <TopOpeBRepBuild_BlockBuilder.hxx>
#include <TopOpeBRepBuild_Builder.hxx>
#include <TopOpeBRepBuild_FaceBuilder.hxx>
#include <TopOpeBRepBuild_WireEdgeClassifier.hxx>
#include <TopOpeBRepBuild_WireEdgeSet.hxx>
#include <TopOpeBRepBuild_WireToFace.hxx>
#include <TopOpeBRepDS_BuildTool.hxx>
// `program face n wire...`: WireToFace::MakeFaces of the wires on the face; prints the
// face count and each face.
void bl_tb_wire_to_face_case(std::istream& input){
 GaReader r{input};TpShapes t;
 try{t=tp_shapes(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 const TopoDS_Face f=TopoDS::Face(t.shapes.at(r.integer()));int n=r.integer();
 TopOpeBRepBuild_WireToFace wtf;
 for(int i=0;i<n;i++)wtf.AddWire(TopoDS::Wire(t.shapes.at(r.integer())));
 NCollection_List<TopoDS_Shape> lf;
 GA_TRY(wtf.MakeFaces(f,lf);ga_int(lf.Extent());for(NCollection_List<TopoDS_Shape>::Iterator it(lf);it.More();it.Next())pr_shape(it.Value()))
 ga_end();}
// The faces, wires and edges of a face builder: per face its wire count, per wire the
// old flag, then the index of the old wire or the block extent and each valid edge
// (index, orientation) closed by -3.
void bl_tb_dump_face_builder(TopOpeBRepBuild_FaceBuilder& fb,const TpShapes& t){
 ga_int(fb.InitFace());
 for(;fb.MoreFace();fb.NextFace()){
  ga_int(fb.InitWire());
  for(;fb.MoreWire();fb.NextWire()){
   bool old=fb.IsOldWire();ga_flag(old);
   if(old){ga_int(tp_same_index(t,fb.OldWire()));continue;}
   ga_int(fb.InitEdge());
   for(;fb.MoreEdge();fb.NextEdge()){ga_int(tp_same_index(t,fb.Edge()));ga_int(fb.Edge().Orientation());}
   ga_int(-3);}}}
// `program face force n (kind shape)...`: a wire edge set on the face fed by AddShape
// (kind 0), AddStartElement (1) or AddElement (2), a face builder on it (dumped), then
// DetectPseudoInternalEdge (count and indices), DetectUnclosedWire (map extents), the
// dump again and the faces of Builder::MakeFaces with an APPROX build tool.
void bl_tb_face_builder_case(std::istream& input){
 GaReader r{input};TpShapes t;
 try{t=tp_shapes(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 const TopoDS_Face f=TopoDS::Face(t.shapes.at(r.integer()));bool force=r.flag();int n=r.integer();
 TopOpeBRepBuild_WireEdgeSet wes(f);
 for(int i=0;i<n;i++){int kind=r.integer();const TopoDS_Shape s=t.shapes.at(r.integer());
  GA_TRY(if(kind==0)wes.AddShape(s);else if(kind==1)wes.AddStartElement(s);else wes.AddElement(s))}
 TopOpeBRepBuild_FaceBuilder fb;
 try{fb.InitFaceBuilder(wes,f,force);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 GA_TRY(bl_tb_dump_face_builder(fb,t))
 GA_TRY(NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> me;fb.DetectPseudoInternalEdge(me);ga_int(me.Extent());
  for(int i=1;i<=me.Extent();i++)ga_int(tp_same_index(t,me(i))))
 GA_TRY(NCollection_IndexedDataMap<TopoDS_Shape,TopoDS_Shape,TopTools_ShapeMapHasher> vv,v1e;fb.DetectUnclosedWire(vv,v1e);
  ga_int(vv.Extent());ga_int(v1e.Extent()))
 GA_TRY(bl_tb_dump_face_builder(fb,t))
 GA_TRY(TopOpeBRepDS_BuildTool bt(TopOpeBRepTool_APPROX);TopOpeBRepBuild_Builder b(bt);NCollection_List<TopoDS_Shape> lf;
  b.MakeFaces(f,fb,lf);ga_int(lf.Extent());for(NCollection_List<TopoDS_Shape>::Iterator it(lf);it.More();it.Next())pr_shape(it.Value()))
 ga_end();}
// `program face s1 s2`: a WireEdgeClassifier on the face with an empty block builder:
// CompareShapes(s1, s2), CompareElementToShape(first edge of s1, s2), and ResetShape(s1)
// then CompareElement of each edge of s2 with the State.
void bl_tb_wec_case(std::istream& input){
 GaReader r{input};TpShapes t;
 try{t=tp_shapes(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 const TopoDS_Shape f=t.shapes.at(r.integer()),s1=t.shapes.at(r.integer()),s2=t.shapes.at(r.integer());
 TopOpeBRepBuild_BlockBuilder bb;TopOpeBRepBuild_WireEdgeClassifier wec(f,bb);
 GA_TRY(ga_int(wec.CompareShapes(s1,s2)))
 GA_TRY(TopExp_Explorer ex(s1,TopAbs_EDGE);ga_int(wec.CompareElementToShape(ex.Current(),s2)))
 GA_TRY(wec.ResetShape(s1);for(TopExp_Explorer ex(s2,TopAbs_EDGE);ex.More();ex.Next())ga_flag(wec.CompareElement(ex.Current()));ga_int(wec.State()))
 ga_end();}
