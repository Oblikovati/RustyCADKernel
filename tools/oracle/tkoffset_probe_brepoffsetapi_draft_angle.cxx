// BRepOffsetAPI_DraftAngle and Draft_Modification probe operations; the Rust twin is
// crates/tkoffset/examples/of_probe/draft_angle.rs.
//
// The shapes are primitives (ot_primitive of tkoffset_probe_brepoffset_tool.cxx); shapes
// print through pr_shape (tkprim_probe_shape.cxx).
#include <BRepOffsetAPI_DraftAngle.hxx>
#include <gp_Pln.hxx>
// `oa_draft_angle prim nadd (face dx dy dz angle px py pz nx ny nz flag)... remove`: the
// draft of faces of the primitive (explorer indices) about the neutral planes, then the
// removal of a face (-1: none). Prints the exception or AddDone after each Add and the
// removal, the exception of Build, IsDone, Status, the type and map index of the
// problematic shape (-1 for none), the shape when done, the sizes of Generated and
// Modified of every face, the number of modified faces and of the faces connected to
// the first drafted face.
void of_oa_draft_angle_case(std::istream& input){
 GaReader r{input};TopoDS_Shape prim;
 try{prim=ot_primitive(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 BRepOffsetAPI_DraftAngle draft(prim);
 int nadd=r.integer();TopoDS_Face first;
 for(int i=0;i<nadd;++i){
  TopoDS_Face f=TopoDS::Face(ot_sub(prim,TopAbs_FACE,r.integer()));if(first.IsNull())first=f;
  gp_Dir d=r.dir();double angle=r.real();gp_Pnt p=r.pnt();gp_Dir n=r.dir();bool flag=r.flag();
  GA_TRY(draft.Add(f,d,angle,gp_Pln(p,n),flag);ga_flag(draft.AddDone()))}
 int removed=r.integer();
 if(removed>=0)GA_TRY(draft.Remove(TopoDS::Face(ot_sub(prim,TopAbs_FACE,removed)));ga_flag(draft.AddDone()))
 GA_TRY(draft.Build())
 ga_flag(draft.IsDone());ga_int(draft.Status());
 NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> map;TopExp::MapShapes(prim,map);
 const TopoDS_Shape& bad=draft.ProblematicShape();
 if(bad.IsNull())ga_int(-1);else{ga_int(bad.ShapeType());ga_int(map.FindIndex(bad));}
 if(draft.IsDone())GA_TRY(pr_shape(draft.Shape()))
 for(TopExp_Explorer ex(prim,TopAbs_FACE);ex.More();ex.Next()){
  GA_TRY(ga_int(draft.Generated(ex.Current()).Extent()))
  GA_TRY(ga_int(draft.Modified(ex.Current()).Extent()))}
 GA_TRY(ga_int(draft.ModifiedFaces().Extent()))
 if(!first.IsNull())GA_TRY(ga_int(draft.ConnectedFaces(first).Extent()))
 ga_end();}
