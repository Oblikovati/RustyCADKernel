// Actual AIS_PointCloud geometry and subset owners, paired with the Martensite fixture.
#include <AIS_PointCloud.hxx>
#include <TColStd_HPackedMapOfInteger.hxx>
struct MartensiteOraclePointCloudScene {
  occ::handle<AIS_InteractiveContext> context;
  occ::handle<AIS_PointCloud> cloud;
  double offset;
  MartensiteOraclePointCloudScene(const occ::handle<V3d_Viewer>& viewer,const std::string& mode,double shift):offset(shift) {
    context=new AIS_InteractiveContext(viewer);cloud=new AIS_PointCloud();
    cloud->Attributes()->ShadingAspect()->Aspect()->SetShadingModel(Graphic3d_TypeOfShadingModel_Unlit);
    cloud->SetPoints(points(5,offset,mode=="cloud-colors",mode=="cloud-normals-unlit"));
    context->Display(cloud,mode=="cloud-box"?2:0,mode.find("subset")!=std::string::npos?1:0,false);
    if(mode=="cloud-selected")context->SetSelected(cloud,false);
    if(mode=="cloud-erased")context->Erase(cloud,false);
    if(mode=="cloud-redisplayed"){context->Erase(cloud,false);context->Display(cloud,false);}
    if(mode=="cloud-unset-color"){cloud->SetColor(Quantity_NOC_RED);cloud->UnsetColor();}
    if(mode=="cloud-moved"){cloud->SetPoints(points(7,offset+1,false,false));context->Redisplay(cloud,false);}
  }
  static occ::handle<Graphic3d_ArrayOfPoints> points(int side,double offset,bool colors,bool normals) {
    occ::handle<Graphic3d_ArrayOfPoints> array=new Graphic3d_ArrayOfPoints(side*side,colors,normals);
    for(int row=0;row<side;++row)for(int col=0;col<side;++col) {
      int index=array->AddVertex(gp_Pnt((col-(side-1)/2.)*1.25+offset,(row-(side-1)/2.)*1.25,0));
      if(colors)array->SetVertexColor(index,Quantity_Color((row+col)%2==0?Quantity_NOC_RED:Quantity_NOC_GREEN));
      if(normals)array->SetVertexNormal(index,gp_Dir(0,0,1));
    }
    return array;
  }
  void detect(const occ::handle<V3d_View>& view,bool selected) {
    int x,y;view->Convert(offset,0,0,x,y);context->MoveTo(x,y,view,false);
    auto owner=occ::down_cast<AIS_PointCloudOwner>(context->DetectedOwner());
    if(owner.IsNull())throw std::runtime_error("expected AIS_PointCloudOwner at central point");
    if(selected)context->SelectDetected(AIS_SelectionScheme_Replace);
    auto subset=selected?owner->SelectedPoints():owner->DetectedPoints();
    if(subset->Map().IsEmpty())throw std::runtime_error("expected nonempty zero-based point cloud subset");
  }
};
