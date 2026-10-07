// BRepFill_OffsetWire probe operations (with BRepFill_TrimEdgeTool, which trims the
// parallels, and BRepFill_OffsetAncestors); the Rust twin is
// crates/tkbool/examples/bl_probe/brep_fill_offset_wire.rs.
//
// The spine face is a shape of a shape program (tktopalgo_probe_shapes.cxx); shapes print
// through pr_shape (tkprim_probe_shape.cxx).
#include <algorithm>
#include <BRepFill_OffsetAncestors.hxx>
#include <BRepFill_OffsetWire.hxx>
// The indices (in the map of the result) and orientations of a list of shapes, sorted:
// GeneratedShapes merges the parallels of cut parts in the iteration order of a map
// hashed on the shape addresses.
static void bl_sorted(const NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher>& m,
                      const NCollection_List<TopoDS_Shape>& l){
 std::vector<std::pair<int,int>> ids;
 for(const TopoDS_Shape& s:l)ids.emplace_back(m.FindIndex(s),(int)s.Orientation());
 std::sort(ids.begin(),ids.end());ga_int((int)ids.size());
 for(const auto& id:ids){ga_int(id.first);ga_int(id.second);}}
// For each edge of the result: the index of its ancestor in the map of the spine, or -1.
static void bl_ancestors(BRepFill_OffsetWire& ow){
 BRepFill_OffsetAncestors anc(ow);ga_flag(anc.IsDone());
 NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> sm;TopExp::MapShapes(ow.Spine(),sm);
 for(TopExp_Explorer ex(ow.Shape(),TopAbs_EDGE);ex.More();ex.Next()){
  const TopoDS_Edge& e=TopoDS::Edge(ex.Current());
  ga_int(anc.HasAncestor(e)?sm.FindIndex(anc.Ancestor(e)):-1);}}
// `program face join open offset alt`: a BRepFill_OffsetWire of the face (join 0 arc,
// 2 intersection) performed at the offset; prints IsDone, the shape, JoinType and, when
// done, GeneratedShapes of every shape of the program and of the spine, and the
// ancestors of the edges of the result.
void bl_bf_offset_wire_case(std::istream& input){
 GaReader r{input};TpShapes t;
 try{t=tp_shapes(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 const TopoDS_Face face=TopoDS::Face(t.shapes.at(r.integer()));
 GeomAbs_JoinType join=(GeomAbs_JoinType)r.integer();bool open=r.flag();
 double offset=r.real(),alt=r.real();
 BRepFill_OffsetWire ow;
 try{ow.Init(face,join,open);ow.Perform(offset,alt);}
 catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 ga_flag(ow.IsDone());ga_int((int)ow.JoinType());
 GA_TRY(pr_shape(ow.Shape()))
 if(!ow.IsDone()){ga_end();return;}
 NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> map;
 TopExp::MapShapes(ow.Shape(),map);
 for(const TopoDS_Shape& g:t.shapes)GA_TRY(bl_sorted(map,ow.GeneratedShapes(g)))
 for(TopExp_Explorer ex(ow.Spine(),TopAbs_EDGE);ex.More();ex.Next())GA_TRY(bl_sorted(map,ow.GeneratedShapes(ex.Current())))
 GA_TRY(bl_ancestors(ow))
 ga_end();}
