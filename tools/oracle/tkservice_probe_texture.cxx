// Graphic3d textures, the Rust twin of crates/tkservice/examples/sv_probe/texture.rs.
#include <Graphic3d_CubeMapPacked.hxx>
#include <Graphic3d_Texture1Dsegment.hxx>
#include <Graphic3d_TextureParams.hxx>
// The image format of a corpus code: 0 Gray, 1 RGB, 2 RGBA, 3 BGR.
Image_Format sv_texture_format(int code){
 const Image_Format formats[]={Image_Format_Gray,Image_Format_RGB,Image_Format_RGBA,Image_Format_BGR};return formats[code];}
// `SizeX SizeY Format TopDown O0..O5`: a packed cube map over a patterned image; for each side,
// the tile wrapper's presence, size, row order and pixel bytes, then the texture row order.
void sv_cubemap_case(std::istream& input){
 GaReader r{input};int sx=r.integer(),sy=r.integer(),code=r.integer();bool topDown=r.flag();int o[6];for(int& v:o)v=r.integer();
 occ::handle<Image_PixMap> image=new Image_PixMap();image->SetTopDown(topDown);
 image->InitTrash(sv_texture_format(code),sx,sy);
 for(size_t i=0;i<image->SizeBytes();++i)image->ChangeData()[i]=(unsigned char)((i*31+7)&255);
 GA_TRY(Graphic3d_ValidatedCubeMapOrder order=Graphic3d_CubeMapOrder(o[0],o[1],o[2],o[3],o[4],o[5]).Validated();
  occ::handle<Graphic3d_CubeMapPacked> cube=new Graphic3d_CubeMapPacked(image,order);
  for(cube->Reset();cube->More();cube->Next()){
   occ::handle<Image_PixMap> tile=cube->Value(occ::handle<Image_SupportedFormats>());ga_flag(!tile.IsNull());
   if(tile.IsNull())continue;ga_int((int)tile->SizeX());ga_int((int)tile->SizeY());ga_flag(tile->IsTopDown());
   for(size_t row=0;row<tile->SizeY();++row)for(size_t col=0;col<tile->SizeX();++col){
    const unsigned char* p=tile->RawValue(row,col);for(size_t b=0;b<tile->SizePixelBytes();++b)ga_int(p[b]);}}
  ga_flag(cube->IsTopDown());ga_flag(image->IsTopDown()))
 ga_end();}
// `X1 Y1 Z1 X2 Y2 Z2`: the generation planes of a segment texture.
void sv_segment_case(std::istream& input){
 GaReader r{input};float v[6];for(float& x:v)x=(float)r.real();
 occ::handle<Graphic3d_Texture1Dsegment> t=new Graphic3d_Texture1Dsegment(TCollection_AsciiString("ramp.png"));
 t->SetSegment(v[0],v[1],v[2],v[3],v[4],v[5]);
 sv_vec4f(t->GetParams()->GenPlaneS());sv_vec4f(t->GetParams()->GenPlaneT());ga_int(t->GetParams()->GenMode());
 ga_end();}
