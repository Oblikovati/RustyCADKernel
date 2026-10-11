// Actual AIS_PointCloud geometry and subset owners, paired with the Martensite fixture.
#include <AIS_PointCloud.hxx>
#include <TColStd_HPackedMapOfInteger.hxx>
struct MartensiteOraclePointCloudScene {
  occ::handle<AIS_InteractiveContext> context;
  occ::handle<AIS_PointCloud> cloud;
  double offset;
  MartensiteOraclePointCloudScene(const occ::handle<V3d_Viewer>& viewer,const std::string& mode,double shift):offset(shift) {
    context=new AIS_InteractiveContext(viewer);cloud=new AIS_PointCloud();
    cloud->Attributes()->ShadingAspect()->Aspect()->SetShadingModel(mode=="cloud-lit-gouraud"?Graphic3d_TypeOfShadingModel_Gouraud:mode.rfind("cloud-lit-",0)==0?Graphic3d_TypeOfShadingModel_Phong:Graphic3d_TypeOfShadingModel_Unlit);
    cloud->SetPoints(points(5,offset,mode=="cloud-colors"||mode=="cloud-lit-colors",mode=="cloud-normals-unlit"||mode.rfind("cloud-lit-",0)==0));
    context->Display(cloud,mode=="cloud-box"?2:0,mode.find("subset")!=std::string::npos?1:0,false);
    if(mode=="cloud-lit-normal-updated"){cloud->GetPoints()->SetVertexNormal(13,gp_Dir(1,0,0));context->Redisplay(cloud,false);}
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
      if(normals)array->SetVertexNormal(index,gp_Dir(col%2==0?0:.6,0,col%2==0?1:.8));
    }
    return array;
  }
  void configure_lights(const occ::handle<V3d_View>& view,const std::string& mode) {
    if(mode.rfind("cloud-lit-",0)!=0)return;
    view->ChangeRenderingParams().ShadingModel=Graphic3d_TypeOfShadingModel_Phong;
    occ::handle<Graphic3d_CLight> ambient=new Graphic3d_CLight(Graphic3d_TypeOfLightSource_Ambient);
    ambient->SetColor(Quantity_Color(.125,.25,.5,Quantity_TOC_RGB));ambient->SetIntensity(2);
    occ::handle<Graphic3d_CLight> directional=new Graphic3d_CLight(Graphic3d_TypeOfLightSource_Directional);
    directional->SetDirection(gp_Dir(0,0,-1));directional->SetColor(Quantity_Color(.75,.5,.25,Quantity_TOC_RGB));
    occ::handle<Graphic3d_LightSet> lights=new Graphic3d_LightSet();lights->Add(ambient);lights->Add(directional);view->View()->SetLights(lights);
    if(mode!="cloud-lit-clipped")return;
    occ::handle<Graphic3d_SequenceOfHClipPlane> planes=new Graphic3d_SequenceOfHClipPlane();
    planes->Append(new Graphic3d_ClipPlane(NCollection_Vec4<double>(1,0,0,-offset)));view->SetClipPlanes(planes);
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
