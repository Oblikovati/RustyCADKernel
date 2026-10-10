// Read the actual X11 drawable independently of the OpenGL offscreen capture.
#include <X11/Xutil.h>
#include <chrono>
#include <iterator>
#include <thread>
static void map_test_window(const occ::handle<Xw_Window> &window) {
  auto display = reinterpret_cast<Display *>(
      window->DisplayConnection()->GetDisplayAspect());
  XSetWindowAttributes attributes{};
  // Prevent desktop minimum sizing and placement from racing capture geometry.
  attributes.override_redirect = True;
  XChangeWindowAttributes(display, window->NativeHandle(), CWOverrideRedirect,
                          &attributes);
  window->Map();
  XSync(display, False);
}
static std::vector<unsigned char>
window_pixels(const occ::handle<Xw_Window> &window) {
  auto display = reinterpret_cast<Display *>(
      window->DisplayConnection()->GetDisplayAspect());
  int width, height;
  window->Size(width, height);
  XSync(display, False);
  auto image = XGetImage(display, window->NativeHandle(), 0, 0, width, height,
                         AllPlanes, ZPixmap);
  if (!image)
    throw std::runtime_error("XGetImage returned null; expected mapped window");
  std::vector<unsigned char> rgba;
  rgba.reserve(width * height * 4);
  for (int y = 0; y < height; ++y)
    for (int x = 0; x < width; ++x) {
      auto pixel = XGetPixel(image, x, y);
      for (auto mask : {image->red_mask, image->green_mask, image->blue_mask}) {
        unsigned shift = 0;
        while (mask && !(mask & 1)) {
          mask >>= 1;
          ++shift;
        }
        rgba.push_back(mask ? ((pixel >> shift) & mask) * 255 / mask : 0);
      }
      rgba.push_back(255);
    }
  XDestroyImage(image);
  return rgba;
}
static void resize_window(const occ::handle<Xw_Window> &window,
                          const occ::handle<V3d_View> &view, int width,
                          int height) {
  auto display = reinterpret_cast<Display *>(
      window->DisplayConnection()->GetDisplayAspect());
  XResizeWindow(display, window->NativeHandle(), width, height);
  XSync(display, False);
  window->DoResize();
  view->MustBeResized();
}
static void capture_window_pixels(const occ::handle<Xw_Window> &window,
                                  const std::string &prefix) {
  std::ifstream expectedFile(prefix + ".rgba", std::ios::binary);
  std::vector<unsigned char> expected(
      (std::istreambuf_iterator<char>(expectedFile)), {}),
      actual;
  auto deadline = std::chrono::steady_clock::now() + std::chrono::seconds(3);
  do {
    actual = window_pixels(window);
    if (actual == expected)
      break;
    std::this_thread::sleep_for(std::chrono::milliseconds(5));
  } while (std::chrono::steady_clock::now() < deadline);
  if (actual != expected)
    throw std::runtime_error("native window pixels differ from OpenGL dump");
  std::ofstream output(prefix + ".window.rgba", std::ios::binary);
  output.write(reinterpret_cast<const char *>(actual.data()), actual.size());
  if (!output)
    throw std::runtime_error("window capture file failed");
}
static void
run_window_scene(const occ::handle<Aspect_DisplayConnection> &display,
                 const occ::handle<OpenGl_GraphicDriver> &driver,
                 const occ::handle<V3d_Viewer> &viewer,
                 const occ::handle<V3d_View> &view, int width, int height,
                 double offset, const std::string &mode,
                 const std::string &prefix) {
  auto window = occ::down_cast<Xw_Window>(view->Window());
  window->SetVirtual(false);
  map_test_window(window);
  XSync(reinterpret_cast<Display *>(display->GetDisplayAspect()), False);
  MartensiteOracleAisScene scene(viewer, offset);
  view->Redraw();
  if (mode == "window-resize") {
    resize_window(window, view, width + 16, height + 8);
    view->Redraw();
    resize_window(window, view, width, height);
  }
  if (mode == "window-rebind") {
    window = new Xw_Window(display, "Martensite rebound oracle", 100, 100,
                           width, height);
    map_test_window(window);
    view->SetWindow(window);
  }
  if (mode == "window-vsync") {
    driver->SetVerticalSync(true);
    view->Redraw();
    driver->SetVerticalSync(false);
  }
  if (mode == "window-remap") {
    XUnmapWindow(reinterpret_cast<Display *>(display->GetDisplayAspect()),
                 window->NativeHandle());
    XSync(reinterpret_cast<Display *>(display->GetDisplayAspect()), False);
    map_test_window(window);
    XSync(reinterpret_cast<Display *>(display->GetDisplayAspect()), False);
  }
  if (mode == "window-erased")
    scene.context->Erase(scene.object, false);
  write_capture(view, width, height, prefix, false);
  view->Redraw();
  capture_window_pixels(window, prefix);
  print_device(view);
  scene.context->Remove(scene.object, false);
  view->Remove();
}
