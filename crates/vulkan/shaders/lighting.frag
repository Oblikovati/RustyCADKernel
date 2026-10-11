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
layout(location=0) out vec4 fragmentColor;
#ifdef MARTENSITE_MARKER_LIGHTING
#define MARTENSITE_SPRITE_SET 2
#include "marker.glsl"
layout(push_constant) uniform Transform { mat4 clip; float size; uint width; uint height; } transform;
#endif
vec4 litFragmentColor() {
    if (lighting.settings.x == 1u) {
        return gl_FrontFacing ? frontColor : backColor;
    }
    vec3 normal = normalize(worldNormal);
    if (lighting.settings.x == 0u) {
        vec3 point = worldPosition.xyz / worldPosition.w;
        // Negative Vulkan viewport reverses screen Y relative to OpenGL derivatives.
        normal = normalize(-cross(dFdx(point), dFdy(point)));
        normal = gl_FrontFacing ? normal : -normal;
    }
    return computeLighting(normal, normalize(viewDirection), worldPosition, gl_FrontFacing, vertexColor);
}

void main() {
#ifdef MARTENSITE_CLIPPING
    discardClippedFragment();
#endif
    fragmentColor = litFragmentColor();
#ifdef MARTENSITE_MARKER_LIGHTING
    fragmentColor *= markerTexel(uvec2(transform.width, transform.height));
    if (fragmentColor.a <= 0.1) discard;
#endif
}
