// Graphic3d_MarkerImage, the Rust twin of crates/tkservice/examples/sv_probe/marker.rs.
#include <Graphic3d_MarkerImage.hxx>
// An image: presence, size, format, row order and the pixel bytes of each row from the top.
void sv_marker_pixmap(const occ::handle<Image_PixMap>& image){
 ga_flag(!image.IsNull());if(image.IsNull())return;
 ga_int((int)image->SizeX());ga_int((int)image->SizeY());ga_int(image->Format());ga_flag(image->IsTopDown());
 for(size_t row=0;row<image->SizeY();++row){const unsigned char* p=image->Row(row);
  for(size_t i=0;i<image->SizeX()*image->SizePixelBytes();++i)ga_int(p[i]);}}
void sv_marker_bits(const occ::handle<NCollection_HArray1<uint8_t>>& bits){
 ga_flag(!bits.IsNull());if(bits.IsNull())return;ga_int(bits->Lower());ga_int(bits->Length());
 for(int i=bits->Lower();i<=bits->Upper();++i)ga_int(bits->Value(i));}
// `Type Scale R G B`: StandardMarker, then every member of the marker.
void sv_marker_case(std::istream& input){
 GaReader r{input};int type=r.integer();float scale=(float)r.real();float c[3];for(float& x:c)x=(float)r.real();
 occ::handle<Graphic3d_MarkerImage> m=Graphic3d_MarkerImage::StandardMarker((Aspect_TypeOfMarker)type,scale,NCollection_Vec4<float>(c[0],c[1],c[2],1.0f));
 ga_flag(!m.IsNull());
 if(!m.IsNull()){ga_text(m->GetImageId().ToCString());ga_text(m->GetImageAlphaId().ToCString());
  int w=0,h=0;m->GetTextureSize(w,h);ga_int(w);ga_int(h);ga_flag(m->IsColoredImage());
  sv_marker_bits(m->GetBitMapArray(0.5,false));sv_marker_bits(m->GetBitMapArray(0.5,true));
  sv_marker_pixmap(m->GetImage());sv_marker_pixmap(m->GetImageAlpha());ga_flag(m->IsColoredImage());}
 ga_end();}
