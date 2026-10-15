// BRepOffset_MakeOffset probe operations; the Rust twin is crates/tkoffset/examples/of_probe/maker.rs.
// The primitives are those of tkoffset_probe_brepoffset_tool.cxx (ot_primitive). The
// order in which the algorithm visits its hashed maps follows the shape addresses, so
// the results are printed as order-independent digests: the counts of the sub-shapes,
// the sorted per-face records (surface type, orientation, rounded area) and the sorted
// rounded vertex points, the rounded volume, and the history counts of the source.
#include <BRepOffset_MakeOffset.hxx>
#include <BRepOffset_MakeSimpleOffset.hxx>
#include <BRepGProp.hxx>
#include <GProp_GProps.hxx>
#include <BRepAdaptor_Surface.hxx>
#include <algorithm>
#include <cmath>
// A value rounded to 1e-6.
static long long om_round(double v){return std::llround(v*1e6);}
// The order-independent digest of a result.
static void om_digest(const TopoDS_Shape& s){
 ga_flag(s.IsNull());if(s.IsNull())return;
 ga_int(s.ShapeType());
 for(TopAbs_ShapeEnum t:{TopAbs_SOLID,TopAbs_SHELL,TopAbs_FACE,TopAbs_EDGE,TopAbs_VERTEX}){
  NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> m;TopExp::MapShapes(s,t,m);ga_int(m.Extent());}
 std::vector<std::vector<long long>> faces;
 NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> fm;TopExp::MapShapes(s,TopAbs_FACE,fm);
 for(int i=1;i<=fm.Extent();i++){
  const TopoDS_Face& f=TopoDS::Face(fm(i));GProp_GProps p;BRepGProp::SurfaceProperties(f,p);
  BRepAdaptor_Surface a(f,false);
  NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> em;TopExp::MapShapes(f,TopAbs_EDGE,em);
  faces.push_back({(long long)a.GetType(),(long long)f.Orientation(),om_round(p.Mass()),(long long)em.Extent()});}
 std::sort(faces.begin(),faces.end());
 for(const auto& r:faces)for(long long v:r)std::cout<<v<<' ';
 std::vector<std::vector<long long>> pts;
 NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> vm;TopExp::MapShapes(s,TopAbs_VERTEX,vm);
 for(int i=1;i<=vm.Extent();i++){gp_Pnt p=BRep_Tool::Pnt(TopoDS::Vertex(vm(i)));pts.push_back({om_round(p.X()),om_round(p.Y()),om_round(p.Z())});}
 std::sort(pts.begin(),pts.end());
 for(const auto& r:pts)for(long long v:r)std::cout<<v<<' ';
 GProp_GProps vp;BRepGProp::VolumeProperties(s,vp);std::cout<<om_round(vp.Mass())<<' ';}
// The shell of faces of a primitive.
static TopoDS_Shape om_shell(const TopoDS_Shape& p,const std::vector<int>& idx){
 BRep_Builder b;TopoDS_Shell sh;b.MakeShell(sh);
 for(int i:idx){TopoDS_Shape f=ot_sub(p,TopAbs_FACE,i);if(!f.IsNull())b.Add(sh,f);}
 return sh;}
// The count and types of a history list.
static void om_list(const NCollection_List<TopoDS_Shape>& l){ga_int(l.Extent());for(const TopoDS_Shape& s:l)ga_int(s.ShapeType());}
// `om_offset prim sub removed offset join inter thick remove mode`: the offset of the
// primitive (`sub` -1), of its face `sub`, of the shell of its faces 0 and 2 (`sub` -2)
// or 0, 2 and 4 (`sub` -3), MakeThickSolid with the face `removed`
// removed (MakeOffsetShape for -1), join 0 arc 1 intersection, the intersection,
// thickening and internal-edge removal flags and the mode; prints the error, done flag,
// result digest and the history counts (Generated, Modified, IsDeleted) of the faces,
// edges and vertices of the source.
void of_om_offset_case(std::istream& input){
 GaReader r{input};
 try{
  TopoDS_Shape p=ot_primitive(r);int sub=r.integer(),removed=r.integer();double off=r.real();
  GeomAbs_JoinType join=r.flag()?GeomAbs_Intersection:GeomAbs_Arc;bool inter=r.flag(),thick=r.flag(),rem=r.flag();
  BRepOffset_Mode mode=BRepOffset_Mode(r.integer());
  TopoDS_Shape src=sub==-1?p:sub>=0?ot_sub(p,TopAbs_FACE,sub):om_shell(p,sub==-2?std::vector<int>{0,2}:std::vector<int>{0,2,4});
  BRepOffset_MakeOffset m;m.Initialize(src,off,1e-7,mode,inter,false,join,thick,rem);
  if(removed>=0){m.AddFace(TopoDS::Face(ot_sub(src,TopAbs_FACE,removed)));m.MakeThickSolid();}
  else m.MakeOffsetShape();
  ga_int(m.Error());ga_flag(m.IsDone());om_digest(m.Shape());
  for(TopAbs_ShapeEnum t:{TopAbs_FACE,TopAbs_EDGE,TopAbs_VERTEX})
   for(TopExp_Explorer ex(src,t);ex.More();ex.Next()){
    GA_TRY(om_list(m.Generated(ex.Current())))GA_TRY(om_list(m.Modified(ex.Current())))GA_TRY(ga_flag(m.IsDeleted(ex.Current())))}}
 catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 ga_end();}
// `om_simple prim sub offset solid`: BRepOffset_MakeSimpleOffset of the primitive or its
// face (as om_offset), with the solid flag; prints the error, done flag and digest.
void of_om_simple_case(std::istream& input){
 GaReader r{input};
 try{
  TopoDS_Shape p=ot_primitive(r);int sub=r.integer();double off=r.real();bool solid=r.flag();
  TopoDS_Shape src=sub==-1?p:sub>=0?ot_sub(p,TopAbs_FACE,sub):om_shell(p,sub==-2?std::vector<int>{0,2}:std::vector<int>{0,2,4});
  BRepOffset_MakeSimpleOffset m(src,off);m.SetBuildSolidFlag(solid);m.Perform();
  ga_int(m.GetError());ga_flag(m.IsDone());if(m.IsDone())om_digest(m.GetResultShape());}
 catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 ga_end();}
