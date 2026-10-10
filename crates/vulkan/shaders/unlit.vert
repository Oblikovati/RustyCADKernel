#version 450
layout(location=0) in vec3 position;
layout(location=1) in vec4 color;
layout(location=0) out vec4 vertexColor;
layout(push_constant) uniform Transform { mat4 clip; } transform;
void main() {
    gl_Position = transform.clip * vec4(position, 1.0);
    // Graphic3d cameras use OpenGL clip depth; Vulkan uses zero-to-one depth.
    gl_Position.z = 0.5 * (gl_Position.z + gl_Position.w);
    vertexColor = color;
}
