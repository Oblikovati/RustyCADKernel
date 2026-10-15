// BRepOffset_Offset probe operations; the Rust twin is crates/tkoffset/examples/of_probe/offset.rs.
// The primitives are those of tkoffset_probe_brepoffset_tool.cxx (ot_primitive).
#include <BRepOffset_Offset.hxx>
#include <TopExp.hxx>
// The status and face of an offset, with the image of each edge of its initial face.
static void oo_print(const BRepOffset_Offset& o,const TopoDS_Shape& source){
 ga_int(o.Status());pr_shape(o.Face());
 for(TopExp_Explorer ex(source,TopAbs_EDGE);ex.More();ex.Next()){
  TopoDS_Shape g=o.Generated(ex.Current());ga_flag(g.IsNull());if(!g.IsNull())ga_int(g.Orientation());}}
// `oo_faces prim offset outside join shared`: the offset of each face of the primitive
// (with the images of the earlier faces as Created when `shared`): its status, face and
// edge images.
void of_oo_faces_case(std::istream& input){
 GaReader r{input};
 try{
  TopoDS_Shape p=ot_primitive(r);double off=r.real();bool outside=r.flag();
  GeomAbs_JoinType join=r.flag()?GeomAbs_Intersection:GeomAbs_Arc;bool shared=r.flag();
  NCollection_DataMap<TopoDS_Shape,TopoDS_Shape,TopTools_ShapeMapHasher> created;
  for(TopExp_Explorer ef(p,TopAbs_FACE);ef.More();ef.Next()){
   const TopoDS_Face& f=TopoDS::Face(ef.Current());
   try{
    BRepOffset_Offset o(f,off,created,outside,join);oo_print(o,f);
    if(shared)for(TopExp_Explorer ee(f,TopAbs_EDGE);ee.More();ee.Next()){
     TopoDS_Shape g=o.Generated(ee.Current());if(!g.IsNull()&&!created.IsBound(ee.Current()))created.Bind(ee.Current(),g);}}
   catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}}}
 catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 ga_end();}
// `oo_pipes prim offset polynomial tol conti connectors`: the faces offset apart, then the pipe of each edge between two distinct faces along the images of the
// edge (once more with the connectors of the first pipe when `connectors`), then the
// sphere of each vertex of three pipes from their connectors.
void of_oo_pipes_case(std::istream& input){
 GaReader r{input};
 try{
  TopoDS_Shape p=ot_primitive(r);double off=r.real();bool poly=r.flag();double tol=r.real();
  GeomAbs_Shape conti=GeomAbs_Shape(r.integer());bool connectors=r.flag();
  NCollection_DataMap<TopoDS_Shape,TopoDS_Shape,TopTools_ShapeMapHasher> created;
  NCollection_DataMap<TopoDS_Shape,BRepOffset_Offset,TopTools_ShapeMapHasher> faces;
  for(TopExp_Explorer ef(p,TopAbs_FACE);ef.More();ef.Next()){
   const TopoDS_Face& f=TopoDS::Face(ef.Current());
   BRepOffset_Offset o(f,off,created,true,GeomAbs_Arc);faces.Bind(f,o);}
  NCollection_IndexedDataMap<TopoDS_Shape,NCollection_List<TopoDS_Shape>,TopTools_ShapeMapHasher> ef;
  TopExp::MapShapesAndAncestors(p,TopAbs_EDGE,TopAbs_FACE,ef);
  NCollection_DataMap<TopoDS_Shape,NCollection_List<TopoDS_Shape>,TopTools_ShapeMapHasher> vconn;
  for(int i=1;i<=ef.Extent();i++){
   const TopoDS_Edge& e=TopoDS::Edge(ef.FindKey(i));const NCollection_List<TopoDS_Shape>& lf=ef(i);
   if(lf.Extent()!=2||lf.First().IsSame(lf.Last())||BRep_Tool::Degenerated(e)){ga_int(-1);continue;}
   try{
    TopoDS_Edge r1=TopoDS::Edge(faces(lf.First()).Generated(e)),r2=TopoDS::Edge(faces(lf.Last()).Generated(e));
    BRepOffset_Offset o(e,r1,r2,off,poly,tol,conti);oo_print(o,TopoDS_Shape());
    TopoDS_Vertex v1,v2;TopExp::Vertices(e,v1,v2);
    TopoDS_Shape g1=o.Generated(v1),g2=o.Generated(v2);pr_shape(g1);pr_shape(g2);
    if(!g1.IsNull())vconn.TryBound(v1,NCollection_List<TopoDS_Shape>()).Append(g1);
    if(!g2.IsNull()&&!v2.IsSame(v1))vconn.TryBound(v2,NCollection_List<TopoDS_Shape>()).Append(g2);
    if(connectors){
     BRepOffset_Offset o2(e,r1,r2,off,TopoDS::Edge(g1),TopoDS::Edge(g2),poly,tol,conti);oo_print(o2,TopoDS_Shape());}}
   catch(const Standard_Failure& ex){std::cout<<ex.ExceptionType()<<' ';}}
  NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> vs;TopExp::MapShapes(p,TopAbs_VERTEX,vs);
  for(int i=1;i<=vs.Extent();i++){
   const NCollection_List<TopoDS_Shape>* l=vconn.Seek(vs(i));
   if(l==nullptr||l->Extent()<3){ga_int(-1);continue;}
   try{BRepOffset_Offset o(TopoDS::Vertex(vs(i)),*l,off,poly,tol,conti);ga_int(o.Status());pr_shape(o.Face());}
   catch(const Standard_Failure& ex){std::cout<<ex.ExceptionType()<<' ';}}}
 catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 ga_end();}
// `oo_tubes prim offset`: the tube of each non-degenerated edge of the primitive.
void of_oo_tubes_case(std::istream& input){
 GaReader r{input};
 try{
  TopoDS_Shape p=ot_primitive(r);double off=r.real();
  NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> es;TopExp::MapShapes(p,TopAbs_EDGE,es);
  for(int i=1;i<=es.Extent();i++){
   const TopoDS_Edge& e=TopoDS::Edge(es(i));if(BRep_Tool::Degenerated(e)){ga_int(-1);continue;}
   try{BRepOffset_Offset o;o.Init(e,off);pr_shape(o.Face());
    TopoDS_Vertex v1,v2;TopExp::Vertices(e,v1,v2);pr_shape(o.Generated(v1));pr_shape(o.Generated(v2));}
   catch(const Standard_Failure& ex){std::cout<<ex.ExceptionType()<<' ';}}}
 catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 ga_end();}
