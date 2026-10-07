// WNT_HIDSpaceMouse decoding, the Rust twin of crates/tkservice/examples/sv_probe/hid.rs.
#include <WNT_HIDSpaceMouse.hxx>
void sv_hid_print(const NCollection_Vec3<double>& v,bool idle){ga_real(v.x());ga_real(v.y());ga_real(v.z());ga_flag(idle);}
// `ProductId Quadric N byte...`: the chunk flags, translation, rotation (each with its idle
// flag), the raw range after both reads, the key state when it is a key chunk and the
// virtual key of every key bit 0..33.
void sv_hid_case(std::istream& input){
 GaReader r{input};unsigned long pid=(unsigned long)r.real();bool quadric=r.flag();int n=r.integer();
 std::vector<uint8_t> bytes(n);for(uint8_t& b:bytes)b=(uint8_t)r.integer();
 WNT_HIDSpaceMouse mouse(pid,bytes.data(),bytes.size());
 ga_flag(WNT_HIDSpaceMouse::IsKnownProduct(pid));ga_flag(mouse.IsTranslation());ga_flag(mouse.IsRotation());ga_flag(mouse.IsKeyState());
 bool idle=false;NCollection_Vec3<double> t=mouse.Translation(idle,quadric);sv_hid_print(t,idle);
 NCollection_Vec3<double> rot=mouse.Rotation(idle,quadric);sv_hid_print(rot,idle);
 ga_int(mouse.RawValueRange());
 if(mouse.IsKeyState()&&n>=5)ga_real((double)mouse.KeyState());
 for(unsigned short bit=0;bit<34;++bit)ga_int((int)mouse.HidToSpaceKey(bit));
 ga_end();}
