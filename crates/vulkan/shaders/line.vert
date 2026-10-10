#version 450
#extension GL_GOOGLE_include_directive : require
#include "clipping.glsl"
layout(location=0) in vec3 position;
layout(location=1) in vec4 color;
layout(location=0) out vec4 vertexColor;
layout(location=1) out vec2 screenSpaceCoord;
layout(push_constant) uniform LineTransform { mat4 clip; vec2 viewport; uint pattern; uint factor; } transform;
void main() {
#ifdef MARTENSITE_CLIPPING
    clippingWorldPosition = clipping.worldFromObject * vec4(position, 1.0);
#endif

    gl_Position = transform.clip * vec4(position, 1.0);
    screenSpaceCoord = (gl_Position.xy / gl_Position.w * 0.5 + 0.5) * transform.viewport;
    gl_Position.z = 0.5 * (gl_Position.z + gl_Position.w);
    vertexColor = color;
}
