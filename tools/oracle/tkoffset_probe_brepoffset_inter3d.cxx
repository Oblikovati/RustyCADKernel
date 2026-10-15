// BRepOffset_Inter3d probe operations; the Rust twin is crates/tkoffset/examples/of_probe/inter3d.rs.
// The primitives are those of tkoffset_probe_brepoffset_tool.cxx (ot_primitive).
#include <BRepAlgo_AsDes.hxx>
#include <BRepAlgo_Image.hxx>
#include <BRepOffset_Analyse.hxx>
#include <BRepOffset_Inter3d.hxx>
#include <BRepOffset_Inter2d.hxx>
// The descendants of a shape in the AsDes: their number and digests.
static void oi_descendants(const occ::handle<BRepAlgo_AsDes>& asDes,const TopoDS_Shape& s){
 const NCollection_List<TopoDS_Shape>& l=asDes->Descendant(s);ga_int(l.Extent());
 for(const TopoDS_Shape& d:l){ga_int(d.Orientation());pr_shape(d);}}
// `oi_inter prim offset angle op planar`: the analyse of the primitive (offset value
// set), the offset of each face, then op 0 ConnexIntByInt (planar or not), op 1
// CompletInt of the offset faces, op 2 ConnexIntByInt then ContextIntByInt with the
// first face as context (extended when `planar`), op 3 ConnexIntByInt then the edge
// intersections of BRepOffset_MakeOffset::IntersectEdges (BRepOffset_Inter2d), op 4 op 3
// then Inter2d::Compute and FuseVertices on the enlarged faces (with, for ops 3 and 4,
// the extended edges, vertices and vertex images); for each face its enlarged face and
// the descendants of the enlarged and offset faces, the Build compound of each edge and
// vertex, the failed shapes and the new and touched counts.
void of_oi_inter_case(std::istream& input){
 GaReader r{input};
 try{
  TopoDS_Shape p=ot_primitive(r);double off=r.real(),angle=r.real();int op=r.integer();bool planar=r.flag();
  BRepOffset_Analyse a;a.SetOffsetValue(off);a.Perform(p,angle);
  NCollection_DataMap<TopoDS_Shape,BRepOffset_Offset,TopTools_ShapeMapHasher> mapSF;
  BRepAlgo_Image image;NCollection_List<TopoDS_Shape> offsetFaces;
  for(TopExp_Explorer ef(p,TopAbs_FACE);ef.More();ef.Next()){
   BRepOffset_Offset o(TopoDS::Face(ef.Current()),off,true,GeomAbs_Intersection);mapSF.Bind(ef.Current(),o);
   image.Bind(ef.Current(),o.Face());offsetFaces.Append(o.Face());}
  occ::handle<BRepAlgo_AsDes> asDes=new BRepAlgo_AsDes();
  BRepOffset_Inter3d inter(asDes,off>0?TopAbs_OUT:TopAbs_IN,1e-7);
  NCollection_DataMap<TopoDS_Shape,TopoDS_Shape,TopTools_ShapeMapHasher> mes,build;NCollection_List<TopoDS_Shape> failed;
  if(op==1)inter.CompletInt(offsetFaces,image,Message_ProgressRange());
  else inter.ConnexIntByInt(p,mapSF,a,mes,build,failed,Message_ProgressRange(),planar);
  occ::handle<BRepAlgo_AsDes> asDes2d=new BRepAlgo_AsDes();BRepAlgo_Image imageVV;
  NCollection_DataMap<TopoDS_Shape,NCollection_List<TopoDS_Shape>,TopTools_ShapeMapHasher> edgeInt;
  if(op>=3){
   // BRepOffset_MakeOffset::IntersectEdges, then for op 4 Compute on each enlarged face.
   NCollection_IndexedDataMap<TopoDS_Shape,NCollection_List<TopoDS_Shape>,TopTools_ShapeMapHasher> dmvv;
   NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> mfv;
   for(TopExp_Explorer ef(p,TopAbs_FACE);ef.More();ef.Next()){
    const TopoDS_Face& f=TopoDS::Face(ef.Current());
    ga_flag(BRepOffset_Inter2d::ConnexIntByInt(f,mapSF(f),mes,build,asDes,asDes2d,off,BRep_Tool::Tolerance(f),a,mfv,imageVV,edgeInt,dmvv,Message_ProgressRange()));}
   ga_int(mfv.Extent());
   for(int i=1;i<=mfv.Extent();i++){const TopoDS_Face& f=TopoDS::Face(mfv(i));
    BRepOffset_Inter2d::ConnexIntByIntInVert(f,mapSF(f),mes,build,asDes,asDes2d,BRep_Tool::Tolerance(f),a,dmvv,Message_ProgressRange());}
   ga_flag(BRepOffset_Inter2d::FuseVertices(dmvv,asDes2d,imageVV));
   if(op==4){
    NCollection_IndexedDataMap<TopoDS_Shape,NCollection_List<TopoDS_Shape>,TopTools_ShapeMapHasher> dmvv2;
    for(TopExp_Explorer ef(p,TopAbs_FACE);ef.More();ef.Next()){
     const TopoDS_Shape* nf=mes.Seek(mapSF(ef.Current()).Face());
     if(nf)BRepOffset_Inter2d::Compute(asDes,TopoDS::Face(*nf),inter.NewEdges(),BRep_Tool::Tolerance(TopoDS::Face(*nf)),edgeInt,dmvv2,Message_ProgressRange());}
    ga_flag(BRepOffset_Inter2d::FuseVertices(dmvv2,asDes,imageVV));}
   for(TopExp_Explorer ef(p,TopAbs_FACE);ef.More();ef.Next()){
    const BRepOffset_Offset& o=mapSF(ef.Current());
    for(TopExp_Explorer ee(ef.Current(),TopAbs_EDGE);ee.More();ee.Next()){
     const TopoDS_Shape* ne=mes.Seek(o.Generated(ee.Current()));ga_flag(ne!=nullptr);
     if(ne){pr_shape(*ne);oi_descendants(asDes2d,*ne);}}
    const TopoDS_Shape* nf=mes.Seek(o.Face());
    if(nf)for(const TopoDS_Shape& e:asDes->Descendant(*nf)){oi_descendants(asDes2d,e);oi_descendants(asDes,e);}}
   for(TopExp_Explorer ev(p,TopAbs_VERTEX);ev.More();ev.Next()){
    ga_flag(imageVV.HasImage(ev.Current()));if(imageVV.HasImage(ev.Current())){const NCollection_List<TopoDS_Shape>& l=imageVV.Image(ev.Current());ga_int(l.Extent());for(const TopoDS_Shape& v:l)pr_shape(v);}}}
  if(op==2){
   NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> ctx;TopExp_Explorer ef(p,TopAbs_FACE);ctx.Add(ef.Current());
   inter.ContextIntByInt(ctx,planar,mapSF,a,mes,build,failed,Message_ProgressRange(),false);}
  for(TopExp_Explorer ef(p,TopAbs_FACE);ef.More();ef.Next()){
   const TopoDS_Shape& of=mapSF(ef.Current()).Face();const TopoDS_Shape* nf=mes.Seek(of);
   ga_flag(nf!=nullptr);if(nf){pr_shape(*nf);oi_descendants(asDes,*nf);}
   oi_descendants(asDes,of);oi_descendants(asDes,ef.Current());}
  NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> ve;TopExp::MapShapes(p,TopAbs_EDGE,ve);TopExp::MapShapes(p,TopAbs_VERTEX,ve);
  for(int i=1;i<=ve.Extent();i++){const TopoDS_Shape* b=build.Seek(ve(i));ga_flag(b!=nullptr);if(b)pr_shape(*b);}
  ga_int(failed.Extent());for(const TopoDS_Shape& s:failed)ga_int(ve.FindIndex(s));
  ga_int(inter.NewEdges().Extent());ga_int(inter.TouchedFaces().Extent());}
 catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 ga_end();}
