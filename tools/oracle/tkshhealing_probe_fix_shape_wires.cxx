// Multi-wire faces of the `sh_fix_shape` probe (tkshhealing_probe_fix_shape.cxx); the Rust
// twin is crates/tkshhealing/examples/sh_probe/fix_shape_wires.rs.
//
// Family 6 is a face on the XY plane with `k` wires added by BRep_Builder:
// `k (kind ...)*k`, kind 0 a closed polygon `m (point)*m` (BRepBuilderAPI_MakePolygon), kind 1
// a full circle `center r` (gp_Ax2(center, Z)) in a wire of its own.
//  33 ShapeFix_IntersectionTool (a context, precision prec, maximal tolerance maxtol)
//     FixIntersectingWires on each face: the flag, the face, then the context applied to
//     each direct child of the input face and to the input face.
#include <ShapeFix_IntersectionTool.hxx>
TopoDS_Wire sfs_multi_wire(GaReader& r){
 if(r.integer()==0){int m=r.integer();BRepBuilderAPI_MakePolygon poly;
  for(int i=0;i<m;++i)poly.Add(r.pnt());poly.Close();return poly.Wire();}
 gp_Pnt c=r.pnt();double R=r.real();BRep_Builder B;TopoDS_Wire w;B.MakeWire(w);
 B.Add(w,BRepBuilderAPI_MakeEdge(new Geom_Circle(gp_Ax2(c,gp::DZ()),R)).Edge());return w;}
TopoDS_Shape sfs_multi_wire_face(GaReader& r){
 int k=r.integer();BRep_Builder B;TopoDS_Face f;B.MakeFace(f,new Geom_Plane(gp_Pln()),1e-7);
 for(int i=0;i<k;++i)B.Add(f,sfs_multi_wire(r));return f;}
void sfs_intersecting_wires(const TopoDS_Shape& s,double prec,double maxtol){
 for(TopExp_Explorer e(s,TopAbs_FACE);e.More();e.Next()){
  occ::handle<ShapeBuild_ReShape> ctx=new ShapeBuild_ReShape;ShapeFix_IntersectionTool t(ctx,prec,maxtol);
  TopoDS_Face f=TopoDS::Face(e.Current());
  GA_TRY(ga_flag(t.FixIntersectingWires(f));tp_shape(f);
   for(TopoDS_Iterator w(e.Current(),false);w.More();w.Next())tp_shape(ctx->Apply(w.Value()));
   tp_shape(ctx->Apply(e.Current())))}}
