#pragma once
#include <Graphic3d_ArrayOfPoints.hxx>
static occ::handle<Graphic3d_ArrayOfPrimitives> fixed_marker_array(const std::string& mode) {
  const bool colored=mode.find("vertices")!=std::string::npos;
  const auto flags=Graphic3d_ArrayFlags(Graphic3d_ArrayFlags_VertexNormal
    |(colored?Graphic3d_ArrayFlags_VertexColor:0));
  const occ::handle<Graphic3d_ArrayOfPrimitives> points=new Graphic3d_ArrayOfPoints(3,flags);
  const float normals[][3]={{0,0,1},{.6f,0,.8f},{0,.6f,.8f}};
  for(int index=0;index<3;++index) {
    points->SetVertice(index+1,-6.125f+6.25f*index,.375f,0);
    points->SetVertexNormal(index+1,normals[index][0],normals[index][1],normals[index][2]);
    if(colored)points->SetVertexColor(index+1,lighting_rgb(.5f,.75f,.25f));
  }
  return points;
}
static void fixed_marker_aspect(const occ::handle<Graphic3d_Aspects>& aspect,const std::string& mode) {
  aspect->SetMarkerScale(5);
  aspect->SetMarkerType(mode.find("ball")!=std::string::npos?Aspect_TOM_BALL:
    mode.find("plus")!=std::string::npos?Aspect_TOM_PLUS:Aspect_TOM_POINT);
  if(mode.find("opacity")!=std::string::npos) {
    auto material=aspect->FrontMaterial();material.SetTransparency(.5f);
    aspect->SetFrontMaterial(material);
  }
  if(mode.find("user")!=std::string::npos) {
    const occ::handle<Image_PixMap> image=new Image_PixMap;image->InitZero(Image_Format_RGBA,4,4);
    image->SetTopDown(mode.find("bottom")==std::string::npos);
    const uint8_t rows[][4]={{128,64,32,255},{32,128,64,128},{64,32,128,0},{255,128,64,255}};
    for(int row=0;row<4;++row)for(int column=0;column<4;++column)
      std::memcpy(image->ChangeData()+row*16+column*4,rows[row],4);
    aspect->SetMarkerType(Aspect_TOM_USERDEFINED);aspect->SetMarkerImage(new Graphic3d_MarkerImage(image));
  }
}
