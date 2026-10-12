#version 450
#extension GL_GOOGLE_include_directive : require
#define MARTENSITE_FRAGMENT
#include "lighting.glsl"
#include "clipping.glsl"
layout(location=0) in vec4 vertexColor;
layout(location=1) in vec3 worldNormal;
layout(location=2) in vec3 viewDirection;
layout(location=4) in vec4 worldPosition;
layout(location=5) in vec4 frontColor;
layout(location=6) in vec4 backColor;
#ifdef MARTENSITE_FIXED_LIGHTING
layout(location=9) flat in vec4 fixedFrontColor;
#endif
#ifdef MARTENSITE_FIXED_LIGHTING
layout(location=10) flat in vec4 fixedBackColor;
#endif
#if defined(MARTENSITE_OIT_WEIGHTED) || defined(MARTENSITE_OIT_PEELING)
#include "oit.glsl"
#else
#include "native_fragment_alpha.glsl"
layout(location=0) out vec4 fragmentColor;
#endif
#ifdef MARTENSITE_MARKER_LIGHTING
#define MARTENSITE_SPRITE_SET 2
#include "marker.glsl"
layout(push_constant) uniform Transform {
    mat4 clip;float size;uint width;uint height;uint policy;
#ifdef MARTENSITE_FIXED_LIGHTING
    float materialCutoff;
#endif
} transform;
#endif
#ifdef MARTENSITE_LINE_LIGHTING
layout(push_constant) uniform LineTransform { mat4 clip; vec2 viewport; uint pattern; uint factor; } transform;
#endif
#include "lit_fragment.glsl"

void main() {
#if defined(MARTENSITE_OIT_WEIGHTED) || defined(MARTENSITE_OIT_PEELING)
    if(oitEarlyReturn())return;
    vec4 fragmentColor;
#endif
#ifdef MARTENSITE_CLIPPING
    discardClippedFragment();
#endif
// OCCT 3d097a0 installs stippling only in its unlit shader. Nodal lit lines
    // intentionally retain solid coverage even when their retained aspect is dashed.
    fragmentColor = litFragmentColor();
#ifdef MARTENSITE_MARKER_LIGHTING
    fragmentColor = litMarkerColor(markerTexel(uvec2(transform.width, transform.height)), fragmentColor, transform.policy);
#ifdef MARTENSITE_FIXED_LIGHTING
    // GL_POINTS bypasses sprite alpha testing; sprite GL_GEQUAL accepts the exact cutoff.
    float cutoff=transform.policy==4u?transform.materialCutoff:0.1;
    if (fragmentColor.a < cutoff) discard;
#else
    if (fragmentColor.a <= 0.1) discard;
#endif
#endif
#if defined(MARTENSITE_OIT_WEIGHTED) || defined(MARTENSITE_OIT_PEELING)
    oitWriteColor(fragmentColor);
#else
    nativePostAlpha(fragmentColor.a);
#endif
}
