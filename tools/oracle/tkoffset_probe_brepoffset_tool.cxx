// BRepOffset_Tool probe operations; the Rust twin is crates/tkoffset/examples/of_probe/tool.rs.
//
// The faces and edges come from primitives (`kind a b c ox oy oz nx ny nz`: kind 0 a box
// a x b x c, 1 a cylinder of radius a and height b, 2 a cone of radii a, b and height c,
// 3 a sphere of radius a at the origin, 4 a torus of radii a, b; placed on the frame of
// the origin and normal) by their index in the explorer order; shapes print through
// pr_shape (tkprim_probe_shape.cxx).
#include <BRepOffset_Tool.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeCone.hxx>
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <BRepPrimAPI_MakeSphere.hxx>
#include <BRepPrimAPI_MakeTorus.hxx>
#include <BRep_Builder.hxx>
#include <TopoDS_Shell.hxx>
// The primitive of a request.
static TopoDS_Shape ot_primitive(GaReader& r){
 int kind=r.integer();double a=r.real(),b=r.real(),c=r.real();gp_Pnt o=r.pnt();gp_Dir n=r.dir();
 gp_Ax2 ax(o,n);
 if(kind==0)return BRepPrimAPI_MakeBox(ax,a,b,c).Shape();
 if(kind==1)return BRepPrimAPI_MakeCylinder(ax,a,b).Shape();
 if(kind==2)return BRepPrimAPI_MakeCone(ax,a,b,c).Shape();
 if(kind==3)return BRepPrimAPI_MakeSphere(a).Shape();
 return BRepPrimAPI_MakeTorus(ax,a,b).Shape();}
// The sub-shape of a type at an explorer index.
static TopoDS_Shape ot_sub(const TopoDS_Shape& s,TopAbs_ShapeEnum kind,int index){
 int i=0;for(TopExp_Explorer ex(s,kind);ex.More();ex.Next(),++i)if(i==index)return ex.Current();
 return TopoDS_Shape();}
// The face of a request (`index enlarge` of a primitive; the original face in `orig`),
// enlarged by EnLargeFace(F, NF, true) on request.
static TopoDS_Face ot_face(GaReader& r,const TopoDS_Shape& prim,TopoDS_Shape& orig){
 orig=ot_sub(prim,TopAbs_FACE,r.integer());TopoDS_Face f=TopoDS::Face(orig);
 if(r.flag()){TopoDS_Face nf;BRepOffset_Tool::EnLargeFace(f,nf,true);return nf;}
 return f;}
// The sections of a list pair: the size, then the orientations and the edge of each.
static void ot_sections(const NCollection_List<TopoDS_Shape>& l1,const NCollection_List<TopoDS_Shape>& l2){
 ga_int(l1.Extent());ga_int(l2.Extent());
 NCollection_List<TopoDS_Shape>::Iterator i1(l1),i2(l2);
 for(;i1.More()&&i2.More();i1.Next(),i2.Next()){ga_int(i1.Value().Orientation());ga_int(i2.Value().Orientation());pr_shape(i1.Value());}}
// `ot_inter3d op prim1 f1 enl1 same [prim2] f2 enl2 side ref`: the second face of the first
// primitive when `same`, else of its own; op 0 Inter3D (with the first common edge of the
// original faces as reference when `ref`), 1 PipeInter, 2 InterOrExtent.
void of_ot_inter3d_case(std::istream& input){
 GaReader r{input};int op=r.integer();
 try{
  TopoDS_Shape p1=ot_primitive(r),p1f,p2f;TopoDS_Face f1=ot_face(r,p1,p1f);
  TopoDS_Shape p2=r.flag()?p1:ot_primitive(r);TopoDS_Face f2=ot_face(r,p2,p2f);
  TopAbs_State side=r.flag()?TopAbs_OUT:TopAbs_IN;bool ref=r.flag();
  NCollection_List<TopoDS_Shape> l1,l2;
  if(op==0){
   TopoDS_Edge e;TopoDS_Face nf;
   if(ref){NCollection_List<TopoDS_Shape> le;BRepOffset_Tool::FindCommonShapes(p1f,p2f,TopAbs_EDGE,le);if(!le.IsEmpty())e=TopoDS::Edge(le.First());}
   BRepOffset_Tool::Inter3D(f1,f2,l1,l2,side,e,nf,nf);}
  else if(op==1)BRepOffset_Tool::PipeInter(f1,f2,l1,l2,side);
  else BRepOffset_Tool::InterOrExtent(f1,f2,l1,l2,side);
  ot_sections(l1,l2);}
 catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 ga_end();}
// `ot_enlarge prim f change update eu evf evl mode l1 l2 l3 l4`: EnLargeFace with every
// option; prints its result, the enlarged face and the face (whose p-curves an update moves).
void of_ot_enlarge_case(std::istream& input){
 GaReader r{input};
 try{
  TopoDS_Shape p=ot_primitive(r);TopoDS_Face f=TopoDS::Face(ot_sub(p,TopAbs_FACE,r.integer()));
  bool change=r.flag(),update=r.flag(),eu=r.flag(),evf=r.flag(),evl=r.flag();int mode=r.integer();
  double l1=r.real(),l2=r.real(),l3=r.real(),l4=r.real();
  TopoDS_Face nf;
  ga_flag(BRepOffset_Tool::EnLargeFace(f,nf,change,update,eu,evf,evl,mode,l1,l2,l3,l4));
  pr_shape(nf);pr_shape(f);}
 catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 ga_end();}
// `ot_inter2d prim f enlarge e1 e2 tol`: Inter2d of two edges of the face (the edges of the
// original face); prints the vertices with their points and parameters on both edges.
void of_ot_inter2d_case(std::istream& input){
 GaReader r{input};
 try{
  TopoDS_Shape p=ot_primitive(r);int fi=r.integer();bool enl=r.flag();
  TopoDS_Face f=TopoDS::Face(ot_sub(p,TopAbs_FACE,fi));
  TopoDS_Edge e1=TopoDS::Edge(ot_sub(f,TopAbs_EDGE,r.integer())),e2=TopoDS::Edge(ot_sub(f,TopAbs_EDGE,r.integer()));
  double tol=r.real();
  if(enl){TopoDS_Face nf;BRepOffset_Tool::EnLargeFace(f,nf,true);f=nf;}
  NCollection_List<TopoDS_Shape> lv;
  BRepOffset_Tool::Inter2d(f,e1,e2,lv,tol);
  ga_int(lv.Extent());
  for(const TopoDS_Shape& v:lv){const TopoDS_Vertex& vx=TopoDS::Vertex(v);ga_pnt(BRep_Tool::Pnt(vx));ga_int(v.Orientation());
   GA_TRY(ga_real(BRep_Tool::Parameter(vx,TopoDS::Edge(e1.Oriented(TopAbs_FORWARD)))))
   GA_TRY(ga_real(BRep_Tool::Parameter(vx,TopoDS::Edge(e2.Oriented(TopAbs_FORWARD)))))}}
 catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 ga_end();}
// `ot_queries prim f1 f2`: FindCommonShapes, CheckPlanesNormals and OrientSection of the
// common edges of two faces, Gabarit of the edge curves of the first face.
void of_ot_queries_case(std::istream& input){
 GaReader r{input};
 try{
  TopoDS_Shape p=ot_primitive(r);
  TopoDS_Face f1=TopoDS::Face(ot_sub(p,TopAbs_FACE,r.integer())),f2=TopoDS::Face(ot_sub(p,TopAbs_FACE,r.integer()));
  NCollection_List<TopoDS_Shape> le,lv;
  ga_flag(BRepOffset_Tool::FindCommonShapes(f1,f2,le,lv));ga_int(le.Extent());ga_int(lv.Extent());
  ga_flag(BRepOffset_Tool::CheckPlanesNormals(f1,f2));
  for(const TopoDS_Shape& e:le){TopAbs_Orientation o1,o2;GA_TRY(BRepOffset_Tool::OrientSection(TopoDS::Edge(e),f1,f2,o1,o2);ga_int(o1);ga_int(o2))}
  for(TopExp_Explorer ex(f1,TopAbs_EDGE);ex.More();ex.Next()){
   const TopoDS_Edge& e=TopoDS::Edge(ex.Current());double a,b;occ::handle<Geom_Curve> c=BRep_Tool::Curve(e,a,b);
   if(!c.IsNull())GA_TRY(ga_real(BRepOffset_Tool::Gabarit(c)))}}
 catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 ga_end();}
// `ot_deboucle prim removed bound`: Deboucle3D of the shell of the primitive without its face
// `removed` (none for -1), the edges of that face as boundary when `bound`.
void of_ot_deboucle_case(std::istream& input){
 GaReader r{input};
 try{
  TopoDS_Shape p=ot_primitive(r);int removed=r.integer();bool bound=r.flag();
  BRep_Builder bb;TopoDS_Shell sh;bb.MakeShell(sh);
  NCollection_Map<TopoDS_Shape,TopTools_ShapeMapHasher> boundary;
  int i=0;for(TopExp_Explorer ex(p,TopAbs_FACE);ex.More();ex.Next(),++i){
   if(i==removed){if(bound)for(TopExp_Explorer ee(ex.Current(),TopAbs_EDGE);ee.More();ee.Next())boundary.Add(ee.Current());continue;}
   bb.Add(sh,ex.Current());}
  pr_shape(BRepOffset_Tool::Deboucle3D(sh,boundary));}
 catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 ga_end();}
