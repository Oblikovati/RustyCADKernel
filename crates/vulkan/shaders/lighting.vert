#version 450
#extension GL_GOOGLE_include_directive : require
#include "lighting.glsl"
#include "clipping.glsl"
layout(location=0) in vec4 position;
layout(location=1) in vec4 color;
layout(location=2) in vec3 normal;
layout(push_constant) uniform Transform {
    mat4 clipFromObject;
#ifdef MARTENSITE_LINE_LIGHTING
    vec2 viewport; uint pattern; uint factor;
#endif
#ifdef MARTENSITE_MARKER_LIGHTING
    float size; uint width; uint height;
#endif
} transform;
layout(location=0) out vec4 vertexColor;
layout(location=1) out vec3 worldNormal;
layout(location=2) out vec3 viewDirection;
layout(location=4) out vec4 worldPosition;
layout(location=5) out vec4 frontColor;
layout(location=6) out vec4 backColor;
void main() {
    worldPosition = lighting.worldFromObject * position;
    worldNormal = normalize((lighting.normalFromObject * vec4(normal, 0.0)).xyz);
    viewDirection = lighting.eye.w != 0.0 ? normalize(lighting.eye.xyz - worldPosition.xyz) : lighting.eye.xyz;
    vertexColor = color;
    frontColor = backColor = vec4(0.0);
    if (lighting.settings.x == 1u) {
        frontColor = computeLighting(worldNormal, viewDirection, worldPosition, true, color);
        backColor = computeLighting(worldNormal, viewDirection, worldPosition, false, color);
    }
#ifdef MARTENSITE_MARKER_LIGHTING
    gl_PointSize = transform.size;
#endif
    gl_Position = transform.clipFromObject * position;
    gl_Position.z = 0.5 * (gl_Position.z + gl_Position.w);
#ifdef MARTENSITE_CLIPPING
    clippingWorldPosition = clipping.worldFromObject * position;
#endif
}
