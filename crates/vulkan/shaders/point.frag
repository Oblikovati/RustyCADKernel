#version 450
#extension GL_GOOGLE_include_directive : require
#define MARTENSITE_FRAGMENT
#include "clipping.glsl"
#include "native_fragment_alpha.glsl"
layout(location=0) in vec4 vertexColor;
layout(location=0) out vec4 outColor;
#include "marker.glsl"
layout(push_constant) uniform Transform {
    mat4 clip;float size;uint width;uint height;uint policy;
#ifdef MARTENSITE_FIXED_LIGHTING
    float materialCutoff;
#endif
} transform;
void main() {
#ifdef MARTENSITE_CLIPPING
    discardClippedFragment();
#endif

    outColor = unlitMarkerColor(markerTexel(uvec2(transform.width, transform.height)), vertexColor, transform.policy);
#ifdef MARTENSITE_FIXED_LIGHTING
    float cutoff=transform.policy==4u?transform.materialCutoff:0.1;
    if (outColor.a < cutoff) discard;
#else
    if (outColor.a <= 0.1) discard;
#endif
    nativePostAlpha(outColor.a);
}
