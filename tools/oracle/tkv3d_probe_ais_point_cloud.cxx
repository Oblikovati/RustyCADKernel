// Martensite point cloud retained/copy geometry and appearance oracle at OCCT 3d097a0.
#include <AIS_PointCloud.hxx>
#include <Select3D_SensitivePrimitiveArray.hxx>
#include <TColStd_HPackedMapOfInteger.hxx>
static void cloud_color(const Quantity_Color& value) {ga_real(value.Red());ga_real(value.Green());ga_real(value.Blue());}
static void cloud_snapshot(const occ::handle<AIS_PointCloud>& cloud) {
  ga_int(static_cast<PrsMgr_PresentableObject*>(cloud.get())->DisplayMode());ga_int(cloud->HilightMode());
  ga_flag(cloud->GetPoints().IsNull());ga_flag(cloud->GetBoundingBox().IsVoid());
  if(!cloud->GetBoundingBox().IsVoid()) {double a,b,c,d,e,f;cloud->GetBoundingBox().Get(a,b,c,d,e,f);for(double v:{a,b,c,d,e,f})ga_real(v);}
  auto points=cloud->GetPoints();if(points.IsNull())return;
  ga_int(points->VertexNumber());ga_flag(points->HasVertexColors());ga_flag(points->HasVertexNormals());
  for(int i=1;i<=points->VertexNumber();++i) {
    ga_pnt(points->Vertice(i));if(points->HasVertexColors())cloud_color(points->VertexColor(i));
    if(points->HasVertexNormals()){double x,y,z;points->VertexNormal(i,x,y,z);ga_real(x);ga_real(y);ga_real(z);}
  }
}
static void cloud_appearance(const occ::handle<AIS_PointCloud>& cloud) {
  ga_flag(cloud->HasColor());ga_flag(cloud->HasMaterial());cloud_color(cloud->Attributes()->Color());ga_real(cloud->Transparency());
  auto shading=cloud->Attributes()->ShadingAspect();ga_flag(cloud->Attributes()->HasOwnShadingAspect());ga_flag(shading->Aspect()->Distinguish());
  for(auto facing:{Aspect_TOFM_FRONT_SIDE,Aspect_TOFM_BACK_SIDE}) {
    cloud_color(shading->Color(facing));ga_int(shading->Material(facing).Name());cloud_color(shading->Material(facing).Color());ga_real(shading->Transparency(facing));
  }
  cloud_color(shading->Aspect()->InteriorColor());cloud_color(shading->Aspect()->BackInteriorColor());
}
static void cloud_arrays(const occ::handle<AIS_PointCloud>& cloud,int colors,int normals) {
  occ::handle<NCollection_HArray1<gp_Pnt>> coords=new NCollection_HArray1<gp_Pnt>(-7,-5);
  coords->SetValue(-7,gp_Pnt(.1,2,3));coords->SetValue(-6,gp_Pnt(4,-5,6));coords->SetValue(-5,gp_Pnt(-7,8,-9));
  occ::handle<NCollection_HArray1<Quantity_Color>> cols;
  if(colors!=0)cols=new NCollection_HArray1<Quantity_Color>(8,8+(colors==1?2:0),Quantity_Color(Quantity_NOC_RED));
  occ::handle<NCollection_HArray1<gp_Dir>> norms;
  if(normals!=0)norms=new NCollection_HArray1<gp_Dir>(42,42+(normals==1?2:0),gp_Dir(0,0,1));
  cloud->SetPoints(coords,cols,norms);
}
static void cloud_operation(const occ::handle<AIS_PointCloud>& cloud,int op) {
  switch(op) {
    case 0:cloud->SetColor(Quantity_Color(Quantity_NOC_RED));break;
    case 1:cloud->UnsetColor();break;
    case 2:cloud->SetMaterial(Graphic3d_MaterialAspect(Graphic3d_NameOfMaterial_Gold));break;
    case 3:cloud->UnsetMaterial();break;
    case 4:cloud->SetTransparency(.25);break;
    case 5:cloud->SetCurrentFacingModel(Aspect_TOFM_FRONT_SIDE);break;
    case 6:cloud->SetCurrentFacingModel(Aspect_TOFM_BACK_SIDE);break;
    case 7:cloud->SetColor(Quantity_Color(Quantity_NOC_BLUE));break;
  }
}
static void cloud_selection(const occ::handle<AIS_PointCloud>& cloud,int mode) {
  occ::handle<SelectMgr_Selection> selection=new SelectMgr_Selection(mode);
  static_cast<SelectMgr_SelectableObject*>(cloud.get())->ComputeSelection(selection,mode);
  ga_int(selection->Entities().Size());
  for(const auto& entity:selection->Entities()) {
    auto sensitive=entity->BaseSensitive();auto owner=sensitive->OwnerId();
    ga_int(owner->Priority());ga_flag(owner->IsForcedHilight());ga_flag(owner->Selectable()==cloud);ga_int(sensitive->SensitivityFactor());
    auto array=occ::down_cast<Select3D_SensitivePrimitiveArray>(sensitive);ga_flag(!array.IsNull());if(!array.IsNull())ga_int(array->NbSubElements());
  }
}
void v3_ais_point_cloud(std::istream& input) {
  int linked,colors,normals,count;input>>linked>>colors>>normals>>count;
  occ::handle<AIS_PointCloud> cloud=new AIS_PointCloud();
  if(linked!=0){occ::handle<Prs3d_Drawer> drawer=new Prs3d_Drawer();drawer->SetupOwnDefaults();drawer->ShadingAspect()->SetColor(Quantity_Color(Quantity_NOC_GREEN));cloud->Attributes()->Link(drawer);}
  GA_TRY(cloud_arrays(cloud,colors,normals));GA_TRY(cloud_snapshot(cloud));cloud_appearance(cloud);
  for(int i=0;i<count;++i){int op;input>>op;GA_TRY(cloud_operation(cloud,op));cloud_appearance(cloud);}
  for(int mode:{-1,0,1,2,99}){GA_TRY(cloud_selection(cloud,mode));}ga_end();
}
void v3_point_cloud_alias(std::istream&) {
  occ::handle<AIS_PointCloud> cloud=new AIS_PointCloud();occ::handle<Graphic3d_ArrayOfPoints> points=new Graphic3d_ArrayOfPoints(3);
  points->AddVertex(gp_Pnt(.1,2,3));points->AddVertex(gp_Pnt(4,-5,6));
  GA_TRY(cloud->SetPoints(points));GA_TRY(cloud_snapshot(cloud));points->SetVertice(1,gp_Pnt(-9,-8,-7));
  GA_TRY(cloud_snapshot(cloud));GA_TRY(cloud->SetPoints(points));GA_TRY(cloud_snapshot(cloud));
  GA_TRY(cloud->SetPoints(occ::handle<Graphic3d_ArrayOfPoints>()));GA_TRY(cloud_snapshot(cloud));
  GA_TRY(cloud->SetPoints(new NCollection_HArray1<gp_Pnt>(),nullptr,nullptr));GA_TRY(cloud_snapshot(cloud));GA_TRY(cloud_selection(cloud,0));ga_end();
}
static void cloud_owner_snapshot(const occ::handle<AIS_PointCloudOwner>& owner,const occ::handle<AIS_PointCloud>& cloud) {
  ga_int(owner->Priority());ga_flag(owner->IsForcedHilight());
  for(auto subset:{owner->SelectedPoints(),owner->DetectedPoints()}) {
    ga_int(subset->Map().Extent());for(int i=0;i<3;++i)ga_flag(subset->Map().Contains(i));
  }
  auto selected=cloud->GetSelectPresentation(nullptr);auto dynamic=cloud->GetHilightPresentation(nullptr);
  for(auto prs:{selected,dynamic}) {
    ga_flag(!prs.IsNull());if(!prs.IsNull()){ga_flag(prs->IsDisplayed());ga_int(prs->GetZLayer());ga_int(prs->Groups().Size());}
  }
}
static void cloud_owner_detect(const occ::handle<Select3D_SensitivePrimitiveArray>& sensitive,double x) {
  SelectMgr_SelectingVolumeManager volume;volume.InitAxisSelectingVolume(gp_Ax1(gp_Pnt(x,0,10),gp_Dir(0,0,-1)));volume.BuildSelectingVolume();
  SelectBasics_PickResult pick;ga_flag(sensitive->Matches(volume,pick));
}
void v3_point_cloud_owner(std::istream& input) {
  int layer;input>>layer;V3DisplayTrace trace;
  occ::handle<V3d_Viewer> viewer=new V3d_Viewer(new V3DisplayDriver(trace));
  occ::handle<PrsMgr_PresentationManager> manager=new PrsMgr_PresentationManager(viewer->StructureManager());
  occ::handle<AIS_PointCloud> cloud=new AIS_PointCloud();occ::handle<Graphic3d_ArrayOfPoints> points=new Graphic3d_ArrayOfPoints(3);
  for(int i=0;i<3;++i)points->AddVertex(gp_Pnt(i*2,0,0));cloud->SetPoints(points);
  occ::handle<SelectMgr_Selection> selection=new SelectMgr_Selection(1);cloud->AddSelection(selection,1);
  auto sensitive=occ::down_cast<Select3D_SensitivePrimitiveArray>(selection->Entities().First()->BaseSensitive());
  auto owner=occ::down_cast<AIS_PointCloudOwner>(sensitive->OwnerId());
  occ::handle<Prs3d_Drawer> style=new Prs3d_Drawer();style->SetupOwnDefaults();style->SetZLayer(layer);
  cloud_owner_snapshot(owner,cloud);
  GA_TRY(cloud_owner_detect(sensitive,2));GA_TRY(owner->HilightWithColor(manager,style,99));cloud_owner_snapshot(owner,cloud);
  owner->Clear(manager,0);cloud_owner_snapshot(owner,cloud);owner->Unhilight(manager,0);cloud_owner_snapshot(owner,cloud);
  manager->BeginImmediateDraw();GA_TRY(cloud_owner_detect(sensitive,4));GA_TRY(owner->HilightWithColor(manager,style,0));cloud_owner_snapshot(owner,cloud);ga_end();
}
