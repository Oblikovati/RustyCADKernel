// Wasm_Window (built without Emscripten) and the display-free parts of Xw_Window, the Rust
// twin of crates/tkservice/examples/sv_probe/window.rs.
#include <Wasm_Window.hxx>
#include <Xw_Window.hxx>
// `Virtual Ratio W H LogicalW LogicalH Buttons Key KeySym(N)...`: a canvas window's geometry
// and conversions, then Xw_Window key translations and constructor checks.
void sv_window_case(std::istream& input)
{
  GaReader r{input};
  const bool isVirtual = r.flag(); const double aRatio = r.real(); const int aW = r.integer(), aH = r.integer();
  const double aLogW = r.real(), aLogH = r.real(); const int aButtons = r.integer(), aKey = r.integer();
  occ::handle<Wasm_Window> aWin = new Wasm_Window("#canvas", true);
  aWin->SetVirtual(isVirtual); ga_text(aWin->CanvasId().ToCString()); ga_flag(aWin->IsMapped());
  aWin->Unmap(); ga_flag(aWin->IsMapped()); aWin->Map(); ga_flag(aWin->IsMapped()); ga_flag(aWin->DoMapping());
  ga_int(aWin->DoResize()); ga_real(aWin->Ratio());
  aWin->SetSizeBacking(NCollection_Vec2<int>(aW, aH)); ga_real(aWin->Ratio());
  int aSize[2] = {-7, -7}; aWin->Size(aSize[0], aSize[1]); ga_int(isVirtual ? aSize[0] : 0); ga_int(isVirtual ? aSize[1] : 0);
  int aPos[4] = {-7, -7, -7, -7}; aWin->Position(aPos[0], aPos[1], aPos[2], aPos[3]);
  ga_int(aPos[0]); ga_int(aPos[1]); ga_int(isVirtual ? aPos[2] : 0); ga_int(isVirtual ? aPos[3] : 0);
  aWin->SetDevicePixelRatio(aRatio); ga_real(aWin->DevicePixelRatio());
  aWin->SetSizeLogical(NCollection_Vec2<double>(aLogW, aLogH)); ga_real(aWin->Ratio());
  ga_int((int)aWin->NativeHandle()); ga_int((int)aWin->NativeParentHandle()); ga_flag(aWin->NativeFBConfig() == nullptr);
  ga_int(Wasm_Window::MouseButtonsFromNative((unsigned short)aButtons)); ga_int(Wasm_Window::VirtualKeyFromNative(aKey));
  const int aNb = r.integer();
  for (int i = 0; i < aNb; ++i) ga_int(Xw_Window::VirtualKeyFromNative((unsigned long)r.real()));
  GA_TRY(Xw_Window(occ::handle<Aspect_DisplayConnection>(), "t", 0, 0, aW, aH));
  GA_TRY(Xw_Window(occ::handle<Aspect_DisplayConnection>(), (Aspect_Drawable)aKey));
  ga_end();
}
