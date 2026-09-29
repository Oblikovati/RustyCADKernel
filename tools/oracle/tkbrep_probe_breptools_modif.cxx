// BRepTools oracle, part 2: BRepTools_Modifier with each modification, and the wire
// explorer, over a scene built by tkbrep_probe_brep.cxx. Modified shapes are printed as
// their text in the current format (tkbrep_probe_breptools.cxx); the Rust twin is
// crates/tkbrep/examples/br_probe/brep_tools_modif.rs.
#include <BRepTools_CopyModification.hxx>
#include <BRepTools_GTrsfModification.hxx>
#include <BRepTools_Modifier.hxx>
#include <BRepTools_NurbsConvertModification.hxx>
#include <BRepTools_TrsfModification.hxx>
#include <BRepTools_WireExplorer.hxx>
#include <gp_GTrsf.hxx>
#include <gp_Mat.hxx>
// The modification of a request: 0 Trsf l copyMesh, 1 GTrsf (a11..a33 x y z),
// 2 Copy copyGeom copyMesh, 3 NurbsConvert.
occ::handle<BRepTools_Modification> bt_modification(const BrGraph& g,const std::vector<double>& p,int kind){
 if(kind==0){occ::handle<BRepTools_TrsfModification> m=new BRepTools_TrsfModification(g.loc(int(p[0])).Transformation());
  m->IsCopyMesh()=p[1]!=0;return m;}
 if(kind==1)return new BRepTools_GTrsfModification(gp_GTrsf(gp_Mat(p[0],p[1],p[2],p[3],p[4],p[5],p[6],p[7],p[8]),gp_XYZ(p[9],p[10],p[11])));
 if(kind==2)return new BRepTools_CopyModification(p[0]!=0,p[1]!=0);
 return new BRepTools_NurbsConvertModification();}
// Whether the modifier kept each scene shape (ModifiedShape(x).IsSame(x)), or its exception.
void bt_modified_shapes(const BrGraph& g,const BRepTools_Modifier& m){
 for(const TopoDS_Shape& x:g.shapes){if(x.IsNull())continue;BR_TRY(br_flag(m.ModifiedShape(x).IsSame(x)));}}
// One modifier run on a shape: done flag, result text, kept flags, updated edges.
void bt_modify(const BrGraph& g,const TopoDS_Shape& sh,const occ::handle<BRepTools_Modification>& mod,bool mutableInput){
 BRepTools_Modifier m(mutableInput);br_flag(m.IsMutableInput());
 m.Init(sh);m.Perform(mod);br_flag(m.IsDone());
 br_text(bt_written(m.ModifiedShape(sh),true,true,3));bt_modified_shapes(g,m);
 occ::handle<BRepTools_NurbsConvertModification> n=occ::down_cast<BRepTools_NurbsConvertModification>(mod);
 if(!n.IsNull())br_int(n->GetUpdatedEdges().Extent());}
// `br_tools_modif script kind params... mutable`: every non-null scene shape through a
// fresh modification and modifier; a mutable run modifies the scene shapes in place.
void br_tools_modif_case(std::istream& in){
 BrReader r{in};BrScene s=br_scene(r);int kind=r.integer();std::vector<double> params;
 int count=kind==0?2:kind==1?12:kind==2?2:0;for(int i=0;i<count;++i)params.push_back(r.real());bool mutableInput=r.flag();
 for(const TopoDS_Shape& sh:s.graph.shapes){
  if(sh.IsNull())continue;
  BR_TRY(occ::handle<BRepTools_Modification> mod=bt_modification(s.graph,params,kind);bt_modify(s.graph,sh,mod,mutableInput));}
 br_end();}
// Every edge of a wire exploration: the edge, its orientation and the current vertex.
void bt_explore(const BrGraph& g,BRepTools_WireExplorer& x){
 int n=0;
 for(;x.More()&&n<64;++n){br_desc(g,x.Current());BR_TRY(br_int(x.Orientation()));br_desc(g,x.CurrentVertex());x.Next();}
 br_int(n);br_flag(x.More());x.Clear();br_flag(x.More());}
// `br_tools_wire script`: each scene wire alone, on each scene face, and on each face
// with its UV bounds.
void br_tools_wire_case(std::istream& in){
 BrReader r{in};BrScene s=br_scene(r);const BrGraph& g=s.graph;
 for(const TopoDS_Shape& w:g.shapes){
  if(w.IsNull()||w.ShapeType()!=TopAbs_WIRE)continue;const TopoDS_Wire& wire=TopoDS::Wire(w);
  BR_TRY(BRepTools_WireExplorer x(wire);bt_explore(g,x));
  for(const TopoDS_Shape& f:g.shapes){
   if(f.IsNull()||f.ShapeType()!=TopAbs_FACE)continue;const TopoDS_Face& face=TopoDS::Face(f);
   BR_TRY(BRepTools_WireExplorer x(wire,face);bt_explore(g,x));
   double a,b,c,d;BR_TRY(BRepTools::UVBounds(face,a,b,c,d);BRepTools_WireExplorer x;x.Init(wire,face,a,b,c,d);bt_explore(g,x));}}
 BRepTools_WireExplorer empty;br_flag(empty.More());
 br_end();}
