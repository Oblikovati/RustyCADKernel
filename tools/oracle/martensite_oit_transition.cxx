#include "martensite_oit_fixture.hxx"
#include "martensite_oit_draw_trace.hxx"

// A single retained view distinguishes persistent fallback from a fresh-view request.
static void capture_stage(const occ::handle<V3d_View>& view,
                          const occ::handle<OpenGl_Context>& context,
                          const std::string& prefix, const std::string& mode, int samples) {
  oit_native_parameters(view, mode, samples, 0.f, 4);
  NativeOitDrawTrace trace(context, prefix);
  texture_capture(view, prefix);
}

int main(int argc, char** argv) {
  if (argc != 2) {
    throw Standard_Failure(("received argc=" + std::to_string(argc)
      + "; expected executable and output prefix").c_str());
  }
  auto probe = create_policy_probe();
  texture_camera(probe.view);
  const auto structure = oit_native_geometry(probe.viewer, false);
  const std::string prefix = argv[1];
  for (const int samples : {0, 1, 2, 3, 4, 6, 8}) {
    capture_stage(probe.view, probe.workspace->GetGlContext(),
      prefix + "-weighted-" + std::to_string(samples), "weighted", samples);
  }
  capture_stage(probe.view, probe.workspace->GetGlContext(), prefix + "-peeling-4", "peeling", 4);
  capture_stage(probe.view, probe.workspace->GetGlContext(), prefix + "-after-weighted-4", "weighted", 4);
  probe.view->Remove();
  return 0;
}
