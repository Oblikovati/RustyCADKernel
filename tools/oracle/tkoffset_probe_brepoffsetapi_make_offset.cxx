// BRepOffsetAPI_MakeOffset probe operations; the Rust twin is
// crates/tkoffset/examples/of_probe/make_offset.rs.
//
// The spine face is a shape of a shape program (tktopalgo_probe_shapes.cxx); shapes print
// through pr_shape (tkprim_probe_shape.cxx).
#include <algorithm>
#include <BRepOffsetAPI_MakeOffset.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <TopTools_ShapeMapHasher.hxx>
// The indices (in the map of the result) and orientations of a list of shapes, sorted:
// BRepFill_OffsetWire::GeneratedShapes merges the parallels of cut parts in the
// iteration order of a map hashed on the shape addresses.
static void of_sorted(const NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher>& m,
                      const NCollection_List<TopoDS_Shape>& l){
 std::vector<std::pair<int,int>> ids;
 for(const TopoDS_Shape& s:l)ids.emplace_back(m.FindIndex(s),(int)s.Orientation());
 std::sort(ids.begin(),ids.end());ga_int((int)ids.size());
 for(const auto& id:ids){ga_int(id.first);ga_int(id.second);}}
// One Perform: the exception type, or IsDone and, when done, the shape and Generated of
// every shape of the program.
static void of_make_offset_perform(BRepOffsetAPI_MakeOffset& mo,const TpShapes& t,double offset,double alt){
 try{mo.Perform(offset,alt);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';return;}
 ga_flag(mo.IsDone());
 if(!mo.IsDone())return;
 GA_TRY(pr_shape(mo.Shape()))
 NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> map;
 TopExp::MapShapes(mo.Shape(),map);
 for(const TopoDS_Shape& g:t.shapes)GA_TRY(of_sorted(map,mo.Generated(g)))}
// `program face mode join open approx offset1 alt1 offset2 alt2`: a
// BRepOffsetAPI_MakeOffset of the face (mode 0: Init(face); mode 1: Init(join) and
// AddWire of each wire of the face; mode 2: no spine), with SetApprox, performed twice
// (the second Perform reuses the domains of its side).
void of_oa_make_offset_case(std::istream& input){
 GaReader r{input};TpShapes t;
 try{t=tp_shapes(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 const TopoDS_Face face=TopoDS::Face(t.shapes.at(r.integer()));
 int mode=r.integer();GeomAbs_JoinType join=(GeomAbs_JoinType)r.integer();
 bool open=r.flag(),approx=r.flag();
 double offset1=r.real(),alt1=r.real(),offset2=r.real(),alt2=r.real();
 BRepOffsetAPI_MakeOffset mo;
 try{
  if(mode==0)mo.Init(face,join,open);
  else{
   mo.Init(join,open);
   if(mode==1)for(TopExp_Explorer ex(face,TopAbs_WIRE);ex.More();ex.Next())mo.AddWire(TopoDS::Wire(ex.Current()));}
  mo.SetApprox(approx);}
 catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 of_make_offset_perform(mo,t,offset1,alt1);
 of_make_offset_perform(mo,t,offset2,alt2);
 ga_end();}
