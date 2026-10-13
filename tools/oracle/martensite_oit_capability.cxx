#include "martensite_oit_fixture.hxx"
#include "martensite_oit_draw_trace.hxx"

struct NativeOitCapabilityControl {
  occ::handle<OpenGl_Context> context;
  OpenGl_FeatureFlag half, floating, samples, buffers;
  NativeOitCapabilityControl(const occ::handle<OpenGl_Context>& ctx):context(ctx),
    half(ctx->hasHalfFloatBuffer),floating(ctx->hasFloatBuffer),samples(ctx->hasSampleVariables),buffers(ctx->hasDrawBuffers) {}
  void restore() {
    context->hasHalfFloatBuffer=half;context->hasFloatBuffer=floating;
    context->hasSampleVariables=samples;context->hasDrawBuffers=buffers;
  }
  void apply(const std::string& mode) {
    if(mode=="float32")context->hasHalfFloatBuffer=OpenGl_FeatureNotAvailable;
    else if(mode=="nofloat") {context->hasHalfFloatBuffer=OpenGl_FeatureNotAvailable;context->hasFloatBuffer=OpenGl_FeatureNotAvailable;}
    else if(mode=="nosamples")context->hasSampleVariables=OpenGl_FeatureNotAvailable;
    else if(mode=="nobuffers")context->hasDrawBuffers=OpenGl_FeatureNotAvailable;
    else throw Standard_Failure(("received capability="+mode+"; expected float32, nofloat, nosamples or nobuffers").c_str());
  }
};
static void capture_capability_stage(const occ::handle<V3d_View>& view,
                                    const occ::handle<OpenGl_Context>& context,const std::string& prefix) {
  NativeOitDrawTrace trace(context,prefix);
  view->Redraw();texture_capture(view,prefix);
}
int main(int argc,char** argv) {
  if(argc!=3)throw Standard_Failure(("received argc="+std::to_string(argc)+"; expected capability and prefix").c_str());
  auto probe=create_policy_probe();texture_camera(probe.view);
  const auto structure=oit_native_geometry(probe.viewer,false);
  oit_native_parameters(probe.view,"weighted",4,0.f,4);
  NativeOitCapabilityControl control(probe.workspace->GetGlContext());
  control.apply(argv[1]);capture_capability_stage(probe.view,control.context,std::string(argv[2])+"-restricted");
  control.restore();capture_capability_stage(probe.view,control.context,std::string(argv[2])+"-restored");
  probe.view->Remove();return 0;
}
