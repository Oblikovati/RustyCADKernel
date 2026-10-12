#pragma once
static void fixed_cap_light_source(const occ::handle<V3d_View>& view,const std::string& mode) {
  if(mode.find("source-")==std::string::npos)return;
  const auto kind=mode.find("source-point")!=std::string::npos?Graphic3d_TypeOfLightSource_Positional:
    mode.find("source-spot")!=std::string::npos?Graphic3d_TypeOfLightSource_Spot:Graphic3d_TypeOfLightSource_Directional;
  const occ::handle<Graphic3d_CLight> light=new Graphic3d_CLight(kind);
  light->SetColor(texture_rgb(.75f,.5f,.25f));
  if(kind!=Graphic3d_TypeOfLightSource_Positional)light->SetDirection(gp_Dir(0,0,-1));
  if(kind!=Graphic3d_TypeOfLightSource_Directional) {light->SetPosition(gp_Pnt(0,0,12));light->SetAttenuation(.5f,.125f);}
  if(kind==Graphic3d_TypeOfLightSource_Spot) {light->SetAngle(.75f);light->SetConcentration(.125f);}
  view->View()->Lights()->Add(light);
}
