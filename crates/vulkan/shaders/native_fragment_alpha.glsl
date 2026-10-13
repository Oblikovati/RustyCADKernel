// OCCT3d097a0 drawMarkers restores GL_GEQUAL even while subsequent shaders are programmable.
void nativeFragmentAlpha(float alpha, vec4 settings) {
    if(settings.z!=0.0 && clamp(alpha,0.0,1.0)<settings.y) discard;
}
#ifdef MARTENSITE_NATIVE_FRAGMENT_ALPHA
layout(set=3,binding=0,std430) readonly buffer NativeAlphaParameters { vec4 settings; } nativeAlpha;
#define nativePostAlpha(alpha) nativeFragmentAlpha(alpha,nativeAlpha.settings)
#else
#define nativePostAlpha(alpha)
#endif
