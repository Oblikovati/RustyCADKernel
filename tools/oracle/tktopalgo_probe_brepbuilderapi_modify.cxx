// BRepBuilderAPI modification probe operations (ModifyShape, Transform, GTransform, Copy,
// NurbsConvert, Collect, FindPlane); the Rust twin is
// crates/tktopalgo/examples/tp_probe/brep_builder_api_modify.rs.
//
// Shapes are the shape records of the BRepLib probes (tp_shape_record); `mesh` n > 0 gives
// the faces an n x n grid triangulation (tp_mesh) before the operation.
// A transformation `trsf` is `kind ...`: 0 translation V; 1 rotation P D angle; 2 scale
// P factor; 3 mirror through P; 4 mirror through the axis P D; 5 mirror through the plane
// P N; 6 SetTransformation(gp_Ax3(P, N)); 7 the product of two transformations; 8 the
// identity. A general one `gtrsf` is `kind ...`: 0 a transformation; 1 an affinity of the
// axis P D and a ratio; 2 an affinity of the plane P N and a ratio; 3 a 3x3 matrix (row by
// row) and a translation.
// Every modification prints `tp_bba_modified`: tp_bba_made, then for each sub-shape of the
// input (TopExp::MapShapes order) ModifiedShape and the Modified list (each image as its
// index in the MapShapes map of the result, 0 when absent, -1 for a null shape, and its
// orientation), Generated count and IsDeleted; the image of a foreign vertex; whether each
// face and edge of the input shares its surface or 3d curve with its image; and the
// triangulation of each face of the result (-1, or the node count and the first node).
// `bba_transform shape mesh trsf copygeom copymesh mode`: mode 0 the constructor, 1
//   Transform(T) then Perform, 2 Transform(T), Perform, then Perform with !copygeom.
// `bba_gtransform shape gtrsf copy mode`: 0 the constructor, 1 GTransform(T) then
//   Perform, 2 the constructor then Perform again.
// `bba_copy shape mesh copygeom copymesh mode`: 0 the constructor, 1 the default one then
//   Perform, 2 the constructor then Perform again.
// `bba_nurbs shape copy mode`: 0 the constructor, 1 the default one then Perform, 2 the
//   constructor then Perform again.
// `bba_modify shape mode`: a class derived from ModifyShape (its constructors and DoModif
//   are protected) with a BRepTools_CopyModification: 0 (S) then DoModif(M); 1 (M) then
//   DoModif(S); 2 (S, M); 3 () then DoModif(S, M); 4 (M), DoModif(S) twice (whether the
//   result is kept); 5 () then DoModif(S); 6 (M) then DoModif of a null shape; 7 (S) only.
// `bba_findplane shape tol`: Found and the plane frame (location, axis, X direction).
// `bba_collect shape nops (op)...`: a Collect over steps on the current shape: op 0 a
//   copying translation (1, 2, 3), 1 NurbsConvert, 2 Copy, 3 a translation by location
//   (each Add(current, algorithm), the result becoming the current shape); 4 i j
//   AddGenerated and 5 i j AddModif of the sub-shapes i and j of the current shape; 6
//   Filter(current). Prints, for each sub-shape of the initial shape, its Modification
//   and Generated lists (bound flag, then the images as indices in the current shape).
#include <BRepBuilderAPI_Collect.hxx>
#include <BRepBuilderAPI_Copy.hxx>
#include <BRepBuilderAPI_FindPlane.hxx>
#include <BRepBuilderAPI_GTransform.hxx>
#include <BRepBuilderAPI_ModifyShape.hxx>
#include <BRepBuilderAPI_NurbsConvert.hxx>
#include <BRepBuilderAPI_Transform.hxx>
#include <BRepTools_CopyModification.hxx>
#include <gp_GTrsf.hxx>
#include <gp_Trsf.hxx>
gp_Trsf tp_bba_trsf(GaReader& r){
 gp_Trsf t;int kind=r.integer();
 if(kind==0)t.SetTranslation(gp_Vec(r.pnt().XYZ()));
 else if(kind==1){gp_Pnt p=r.pnt();gp_Dir d=r.dir();t.SetRotation(gp_Ax1(p,d),r.real());}
 else if(kind==2){gp_Pnt p=r.pnt();t.SetScale(p,r.real());}
 else if(kind==3)t.SetMirror(r.pnt());
 else if(kind==4){gp_Pnt p=r.pnt();t.SetMirror(gp_Ax1(p,r.dir()));}
 else if(kind==5){gp_Pnt p=r.pnt();t.SetMirror(gp_Ax2(p,r.dir()));}
 else if(kind==6){gp_Pnt p=r.pnt();t.SetTransformation(gp_Ax3(p,r.dir()));}
 else if(kind==7){gp_Trsf a=tp_bba_trsf(r);gp_Trsf b=tp_bba_trsf(r);t=a*b;}
 return t;}
gp_GTrsf tp_bba_gtrsf(GaReader& r){
 gp_GTrsf g;int kind=r.integer();
 if(kind==0)g=gp_GTrsf(tp_bba_trsf(r));
 else if(kind==1){gp_Pnt p=r.pnt();gp_Dir d=r.dir();g.SetAffinity(gp_Ax1(p,d),r.real());}
 else if(kind==2){gp_Pnt p=r.pnt();gp_Dir d=r.dir();g.SetAffinity(gp_Ax2(p,d),r.real());}
 else{gp_Mat m;for(int i=1;i<=3;++i)for(int j=1;j<=3;++j)m.SetValue(i,j,r.real());g.SetVectorialPart(m);g.SetTranslationPart(r.pnt().XYZ());}
 return g;}
TpShapeMap tp_bba_all(const TopoDS_Shape& s){TpShapeMap m;if(!s.IsNull())TopExp::MapShapes(s,m);return m;}
// An image: its index in the result map (0 when absent) and its orientation; -1 when null.
void tp_bba_image(const TopoDS_Shape& image,const TpShapeMap& out){
 if(image.IsNull()){ga_int(-1);return;}ga_int(out.FindIndex(image));ga_int(image.Orientation());}
template<class A> void tp_bba_images(A& a,const TpShapeMap& in,const TpShapeMap& out){
 ga_int(in.Extent());
 for(int i=1;i<=in.Extent();++i){
  GA_TRY(tp_bba_image(a.ModifiedShape(in(i)),out))
  GA_TRY(const NCollection_List<TopoDS_Shape>& l=a.Modified(in(i));ga_int(l.Extent());for(const TopoDS_Shape& s:l)tp_bba_image(s,out))
  GA_TRY(ga_int(a.Generated(in(i)).Extent());ga_flag(a.IsDeleted(in(i))))}
 TopoDS_Vertex foreign;BRep_Builder().MakeVertex(foreign,gp_Pnt(1.,2.,3.),1e-7);
 GA_TRY(tp_bba_image(a.ModifiedShape(foreign),out))}
// Whether each face (edge) of the input shares its surface (3d curve) with its image.
template<class A> void tp_bba_shared(A& a,const TopoDS_Shape& input){
 TpShapeMap faces=tp_sub_shapes(input,TopAbs_FACE),edges=tp_sub_shapes(input,TopAbs_EDGE);
 for(int i=1;i<=faces.Extent();++i)GA_TRY(TopoDS_Shape im=a.ModifiedShape(faces(i));TopLoc_Location l1,l2;
  ga_flag(BRep_Tool::Surface(TopoDS::Face(faces(i)),l1)==BRep_Tool::Surface(TopoDS::Face(im),l2)))
 for(int i=1;i<=edges.Extent();++i)GA_TRY(TopoDS_Shape im=a.ModifiedShape(edges(i));TopLoc_Location l1,l2;double f,l;
  ga_flag(BRep_Tool::Curve(TopoDS::Edge(edges(i)),l1,f,l)==BRep_Tool::Curve(TopoDS::Edge(im),l2,f,l)))}
void tp_bba_meshes(const TopoDS_Shape& s){
 TpShapeMap faces=tp_sub_shapes(s,TopAbs_FACE);
 for(int i=1;i<=faces.Extent();++i){TopLoc_Location l;occ::handle<Poly_Triangulation> t=BRep_Tool::Triangulation(TopoDS::Face(faces(i)),l);
  if(t.IsNull()){ga_int(-1);continue;}ga_int(t->NbNodes());ga_pnt(t->Node(1).Transformed(l.Transformation()));}}
template<class A> void tp_bba_modified(A& a,const TopoDS_Shape& input){
 tp_bba_made(a);if(!a.IsDone())return;
 TpShapeMap out=tp_bba_all(a.Shape());tp_bba_images(a,tp_bba_all(input),out);tp_bba_shared(a,input);tp_bba_meshes(a.Shape());}
TopoDS_Shape tp_bba_meshed_record(GaReader& r){TopoDS_Shape s=tp_shape_record(r);int n=r.integer();if(n>0)tp_mesh(s,n,false);return s;}
void tp_bba_transform_case(std::istream& input){
 GaReader r{input};
 try{TopoDS_Shape s=tp_bba_meshed_record(r);gp_Trsf t=tp_bba_trsf(r);bool g=r.flag(),m=r.flag();int mode=r.integer();
  std::unique_ptr<BRepBuilderAPI_Transform> a;
  if(mode==0)a.reset(new BRepBuilderAPI_Transform(s,t,g,m));
  else{a.reset(new BRepBuilderAPI_Transform(t));a->Perform(s,g,m);if(mode==2)a->Perform(s,!g,m);}
  tp_bba_modified(*a,s);
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 ga_end();}
void tp_bba_gtransform_case(std::istream& input){
 GaReader r{input};
 try{TopoDS_Shape s=tp_shape_record(r);gp_GTrsf t=tp_bba_gtrsf(r);bool c=r.flag();int mode=r.integer();
  std::unique_ptr<BRepBuilderAPI_GTransform> a;
  if(mode==1){a.reset(new BRepBuilderAPI_GTransform(t));a->Perform(s,c);}
  else{a.reset(new BRepBuilderAPI_GTransform(s,t,c));if(mode==2)a->Perform(s,c);}
  tp_bba_modified(*a,s);
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 ga_end();}
void tp_bba_copy_case(std::istream& input){
 GaReader r{input};
 try{TopoDS_Shape s=tp_bba_meshed_record(r);bool g=r.flag(),m=r.flag();int mode=r.integer();
  std::unique_ptr<BRepBuilderAPI_Copy> a;
  if(mode==1){a.reset(new BRepBuilderAPI_Copy());a->Perform(s,g,m);}
  else{a.reset(new BRepBuilderAPI_Copy(s,g,m));if(mode==2)a->Perform(s,g,m);}
  tp_bba_modified(*a,s);
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 ga_end();}
void tp_bba_nurbs_case(std::istream& input){
 GaReader r{input};
 try{TopoDS_Shape s=tp_shape_record(r);bool c=r.flag();int mode=r.integer();
  std::unique_ptr<BRepBuilderAPI_NurbsConvert> a;
  if(mode==1){a.reset(new BRepBuilderAPI_NurbsConvert());a->Perform(s,c);}
  else{a.reset(new BRepBuilderAPI_NurbsConvert(s,c));if(mode==2)a->Perform(s,c);}
  tp_bba_modified(*a,s);
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 ga_end();}
// ModifyShape with its protected members made reachable.
struct TpModifyShape:BRepBuilderAPI_ModifyShape{
 TpModifyShape()=default;
 explicit TpModifyShape(const TopoDS_Shape& s):BRepBuilderAPI_ModifyShape(s){}
 explicit TpModifyShape(const occ::handle<BRepTools_Modification>& m):BRepBuilderAPI_ModifyShape(m){}
 TpModifyShape(const TopoDS_Shape& s,const occ::handle<BRepTools_Modification>& m):BRepBuilderAPI_ModifyShape(s,m){}
 void ModifShape(const TopoDS_Shape& s){DoModif(s);}
 void ModifModification(const occ::handle<BRepTools_Modification>& m){DoModif(m);}
 void ModifBoth(const TopoDS_Shape& s,const occ::handle<BRepTools_Modification>& m){DoModif(s,m);}};
std::unique_ptr<TpModifyShape> tp_bba_modify_mode(int mode,const TopoDS_Shape& s){
 occ::handle<BRepTools_Modification> m=new BRepTools_CopyModification(true,false);std::unique_ptr<TpModifyShape> a;
 if(mode==0){a.reset(new TpModifyShape(s));a->ModifModification(m);}
 else if(mode==1||mode==4||mode==6){a.reset(new TpModifyShape(m));a->ModifShape(mode==6?TopoDS_Shape():s);}
 else if(mode==2)a.reset(new TpModifyShape(s,m));
 else if(mode==3){a.reset(new TpModifyShape());a->ModifBoth(s,m);}
 else if(mode==5){a.reset(new TpModifyShape());a->ModifShape(s);}
 else a.reset(new TpModifyShape(s));
 return a;}
void tp_bba_modify_case(std::istream& input){
 GaReader r{input};
 try{TopoDS_Shape s=tp_shape_record(r);int mode=r.integer();std::unique_ptr<TpModifyShape> a=tp_bba_modify_mode(mode,s);
  if(mode==4){TopoDS_Shape first=a->Shape();a->ModifShape(s);ga_flag(first.IsEqual(a->Shape()));}
  if(mode==7){ga_flag(a->IsDone());GA_TRY(tp_bba_image(a->ModifiedShape(s),TpShapeMap()))}
  tp_bba_modified(*a,s);
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 ga_end();}
void tp_bba_findplane_case(std::istream& input){
 GaReader r{input};
 try{TopoDS_Shape s=tp_shape_record(r);BRepBuilderAPI_FindPlane f(s,r.real());ga_flag(f.Found());
  if(f.Found()){const gp_Ax3& a=f.Plane()->Position();ga_pnt(a.Location());ga_dir(a.Direction());ga_dir(a.XDirection());}
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 ga_end();}
// One step of `bba_collect` on the current shape.
void tp_bba_collect_step(int op,GaReader& r,BRepBuilderAPI_Collect& c,TopoDS_Shape& cur){
 if(op>=4&&op<=5){TpShapeMap m=tp_bba_all(cur);const TopoDS_Shape& a=m(r.integer());const TopoDS_Shape& b=m(r.integer());
  if(op==4)c.AddGenerated(a,b);else c.AddModif(a,b);return;}
 if(op==6){c.Filter(cur);return;}
 gp_Trsf t;t.SetTranslation(gp_Vec(1.,2.,3.));std::unique_ptr<BRepBuilderAPI_MakeShape> a;
 if(op==0)a.reset(new BRepBuilderAPI_Transform(cur,t,true));
 else if(op==1)a.reset(new BRepBuilderAPI_NurbsConvert(cur));
 else if(op==2)a.reset(new BRepBuilderAPI_Copy(cur));
 else a.reset(new BRepBuilderAPI_Transform(cur,t));
 c.Add(cur,*a);cur=a->Shape();}
void tp_bba_collect_list(const NCollection_DataMap<TopoDS_Shape,NCollection_List<TopoDS_Shape>,TopTools_ShapeMapHasher>& h,
                         const TopoDS_Shape& key,const TpShapeMap& out){
 const NCollection_List<TopoDS_Shape>* l=h.Seek(key);ga_flag(l!=nullptr);if(l==nullptr)return;
 ga_int(l->Extent());for(const TopoDS_Shape& s:*l)tp_bba_image(s,out);}
void tp_bba_collect_case(std::istream& input){
 GaReader r{input};
 try{TopoDS_Shape s=tp_shape_record(r);TopoDS_Shape cur=s;BRepBuilderAPI_Collect c;int n=r.integer();
  for(int i=0;i<n;++i)tp_bba_collect_step(r.integer(),r,c,cur);
  TpShapeMap in=tp_bba_all(s),out=tp_bba_all(cur);
  for(int i=1;i<=in.Extent();++i){tp_bba_collect_list(c.Modification(),in(i),out);tp_bba_collect_list(c.Generated(),in(i),out);}
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 ga_end();}
