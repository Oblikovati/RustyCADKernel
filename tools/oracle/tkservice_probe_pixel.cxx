// Image_PixMap pixel conversions, the Rust twin of crates/tkservice/examples/sv_probe/pixel.rs.
#include <Image_PixMap.hxx>
void sv_rgba_print(const Quantity_ColorRGBA& c){const NCollection_Vec4<float>& v=c;for(int i=0;i<4;++i)ga_real(v[i]);}
// `Format ToLinear R G B A`: ColorToRawPixel bytes of the color (built from the four floats
// without validation of the Vec4 path), then ColorFromRawPixel of those bytes.
void sv_pixel_case(std::istream& input){
 GaReader r{input};Image_Format format=(Image_Format)r.integer();bool linear=r.flag();
 float c[4];for(float& x:c)x=(float)r.real();
 GA_TRY(Quantity_ColorRGBA color(NCollection_Vec4<float>(c[0],c[1],c[2],c[3]));uint8_t raw[16]={0};
  Image_PixMap::ColorToRawPixel(raw,format,color,linear);for(int i=0;i<16;++i)ga_int(raw[i]);
  sv_rgba_print(Image_PixMap::ColorFromRawPixel(raw,format,linear)))
 ga_end();}
// `Format byte...16`: ColorFromRawPixel of raw bytes (float formats may raise).
void sv_rawpixel_case(std::istream& input){
 GaReader r{input};Image_Format format=(Image_Format)r.integer();bool linear=r.flag();
 uint8_t raw[16];for(uint8_t& b:raw)b=(uint8_t)r.integer();
 GA_TRY(sv_rgba_print(Image_PixMap::ColorFromRawPixel(raw,format,linear)))
 ga_end();}
// `Half Float`: ConvertFromHalfFloat(Half) and ConvertToHalfFloat(Float).
void sv_half_case(std::istream& input){
 GaReader r{input};uint16_t half=(uint16_t)r.integer();float value=(float)r.real();
 ga_real(Image_PixMap::ConvertFromHalfFloat(half));ga_int(Image_PixMap::ConvertToHalfFloat(value));
 ga_end();}
