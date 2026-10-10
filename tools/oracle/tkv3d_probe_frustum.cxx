// Protected SelectMgr_Frustum methods at OCCT 3d097a0; raw caches are identical inputs on both sides.
#include <SelectMgr_TriangularFrustum.hxx>
#include <SelectMgr_RectangularFrustum.hxx>
template<class Parent,int N> class V3RawFrustum : public Parent {
public:
  using Parent::hasBoxOverlap;using Parent::hasPointOverlap;using Parent::hasSegmentOverlap;
  using Parent::hasPolygonOverlap;using Parent::hasTriangleOverlap;using Parent::hasSphereOverlap;
  using Parent::hasCylinderOverlap;using Parent::hasCircleOverlap;
  using Parent::isDotInside;using Parent::isInsideCylinderEndFace;
  V3RawFrustum(GaReader& r,bool ortho) {
    occ::handle<Graphic3d_Camera> camera=new Graphic3d_Camera;
    camera->SetProjectionType(ortho?Graphic3d_Camera::Projection_Orthographic:Graphic3d_Camera::Projection_Perspective);
    this->SetCamera(camera);
    for(int i=0;i<N+2;++i)this->myPlanes[i]=gp_Vec(r.xyz());
    for(int i=0;i<2*N;++i)this->myVertices[i]=r.pnt();
    for(int i=0;i<N+2;++i)this->myMaxVertsProjections[i]=r.real();
    for(int i=0;i<N+2;++i)this->myMinVertsProjections[i]=r.real();
    for(int i=0;i<3;++i)this->myMaxOrthoVertsProjections[i]=r.real();
    for(int i=0;i<3;++i)this->myMinOrthoVertsProjections[i]=r.real();
    for(int i=0;i<6;++i)this->myEdgeDirs[i]=gp_Vec(r.xyz());
  }
  void CoreDump(std::ostream& output,int depth) { SelectMgr_Frustum<N>::DumpJson(output,depth); }
  void PrintIdentity() { ga_flag(!this->myBuilder.IsNull());ga_flag(this->myCamera==this->myBuilder->Camera()); }
};
NCollection_Array1<gp_Pnt> v3_frustum_points(GaReader& r) {
  int lower=r.integer(),count=r.integer();NCollection_Array1<gp_Pnt> points;
  if(count>0)points.Resize(lower,lower+count-1,false);
  for(int i=0;i<count;++i)points.SetValue(lower+i,r.pnt());return points;
}
gp_Trsf v3_frustum_transform(GaReader& r) {
  gp_Dir axis=r.dir();double angle=r.real();gp_Vec shift(r.xyz());double scale=r.real();
  gp_Trsf transform;transform.SetRotation(gp_Ax1(gp::Origin(),axis),angle);
  transform.SetTranslationPart(shift);transform.SetScaleFactor(scale);return transform;
}
std::string v3_named_dump_pointers(std::string text,std::initializer_list<const char*> fields) {
  // Only process-local addresses are canonicalized; identities are checked separately.
  for(const std::string field:fields) {
    std::string prefix="\""+field+"\": \"";auto start=text.find(prefix);
    if(start==std::string::npos)throw std::runtime_error("missing pointer field "+field);
    start+=prefix.size();auto end=text.find('"',start);
    text.replace(start,end-start,end==start?"null":"present");
  }
  return text;
}
std::string v3_frustum_dump_pointers(std::string text) {
  return v3_named_dump_pointers(std::move(text),{"Builder","Camera"});
}
template<class Volume> void v3_frustum_sat_query(Volume& volume,GaReader& r) {
  int kind=r.integer();gp_Vec normal(r.xyz());
  if(kind==0){gp_Pnt point=r.pnt();ga_flag(volume.hasPointOverlap(point));}
  if(kind==1){gp_Pnt a=r.pnt(),b=r.pnt();ga_flag(volume.hasSegmentOverlap(a,b));}
  if(kind==2){gp_XYZ a=r.xyz(),b=r.xyz();int mode=r.integer();bool inside=mode==1;GA_TRY(ga_flag(volume.hasBoxOverlap(NCollection_Vec3<double>(a.X(),a.Y(),a.Z()),NCollection_Vec3<double>(b.X(),b.Y(),b.Z()),mode<0?nullptr:&inside)))ga_flag(inside);}
  if(kind==3){auto points=v3_frustum_points(r);GA_TRY(ga_flag(volume.hasPolygonOverlap(points,normal)))ga_xyz(normal.XYZ());}
  if(kind==4){gp_Pnt a=r.pnt(),b=r.pnt(),c=r.pnt();GA_TRY(ga_flag(volume.hasTriangleOverlap(a,b,c,normal)))ga_xyz(normal.XYZ());}
}
template<class Volume> void v3_frustum_quadric_query(Volume& volume,GaReader& r) {
  int kind=r.integer(),mode=r.integer();bool inside=mode==1;
  if(kind==0){gp_Pnt center=r.pnt();double radius=r.real();GA_TRY(ga_flag(volume.hasSphereOverlap(center,radius,mode<0?nullptr:&inside)))}
  if(kind==1){double bottom=r.real(),top=r.real(),height=r.real();bool hollow=r.flag();auto transform=v3_frustum_transform(r);GA_TRY(ga_flag(volume.hasCylinderOverlap(bottom,top,height,transform,hollow,mode<0?nullptr:&inside)))}
  if(kind==2){double radius=r.real();bool filled=r.flag();auto transform=v3_frustum_transform(r);GA_TRY(ga_flag(volume.hasCircleOverlap(radius,transform,filled,mode<0?nullptr:&inside)))}
  if(kind==3){gp_Pnt point=r.pnt();auto vertices=v3_frustum_points(r);GA_TRY(ga_flag(volume.isDotInside(point,vertices)))}
  if(kind==4){double bottom=r.real(),top=r.real(),height=r.real();auto transform=v3_frustum_transform(r);auto vertices=v3_frustum_points(r);GA_TRY(ga_flag(volume.isInsideCylinderEndFace(bottom,top,height,transform,vertices)))}
  ga_flag(inside);
}
template<class Parent,int N> void v3_frustum_dispatch(GaReader& r,bool ortho,int family) {
  V3RawFrustum<Parent,N> volume(r,ortho);
  if(family==0)v3_frustum_sat_query(volume,r);
  if(family==1)v3_frustum_quadric_query(volume,r);
  if(family==2){int depth=r.integer();volume.PrintIdentity();std::ostringstream output;volume.CoreDump(output,depth);ga_text(v3_frustum_dump_pointers(output.str()));}
}
void v3_frustum_core_case(std::istream& input) {
  GaReader r{input};int n=r.integer();bool ortho=r.flag();int family=r.integer();
  GA_TRY(if(n==3){v3_frustum_dispatch<SelectMgr_TriangularFrustum,3>(r,ortho,family);}else{v3_frustum_dispatch<SelectMgr_RectangularFrustum,4>(r,ortho,family);})
  ga_end();
}
