#version 450
// OCCT 3d097a0 Graphic3d_ShaderManager: native projected triangle altitudes.
layout(triangles) in;
layout(triangle_strip, max_vertices=3) out;
layout(location=0) in vec4 inputColor[];
layout(location=0) out vec4 vertexColor;
layout(location=7) in vec4 inputUV[];
layout(location=7) out vec4 surfaceUV;
layout(location=8) out vec3 edgeDistance;
#ifdef MARTENSITE_LIGHTING
layout(location=1) in vec3 inputNormal[];
layout(location=2) in vec3 inputDirection[];
layout(location=4) in vec4 inputPosition[];
layout(location=5) in vec4 inputFront[];
layout(location=6) in vec4 inputBack[];
#ifdef MARTENSITE_FIXED_LIGHTING
layout(location=9) flat in vec4 inputFixedFront[];
#endif
#ifdef MARTENSITE_FIXED_LIGHTING
layout(location=10) flat in vec4 inputFixedBack[];
#endif
layout(location=1) out vec3 worldNormal;
layout(location=2) out vec3 viewDirection;
layout(location=4) out vec4 worldPosition;
layout(location=5) out vec4 frontColor;
layout(location=6) out vec4 backColor;
#ifdef MARTENSITE_FIXED_LIGHTING
layout(location=9) flat out vec4 fixedFrontColor;
#endif
#ifdef MARTENSITE_FIXED_LIGHTING
layout(location=10) flat out vec4 fixedBackColor;
#endif
#endif
#ifdef MARTENSITE_CLIPPING
layout(location=3) in vec4 inputClipping[];
layout(location=3) out vec4 clippingWorldPosition;
#endif
layout(push_constant) uniform EdgeTransform {
    layout(offset=80) vec4 edgeColor;
    layout(offset=96) float edgeWidth;
    layout(offset=100) float edgeFeather;
    layout(offset=104) vec2 viewport;
    layout(offset=112) uint skipFirst;
} transform;
vec3 viewportPosition(vec4 position) {
    // Vertex shader has already mapped native clip depth to Vulkan's 0..1 range.
    vec3 coordinate=position.xyz/position.w;
    return vec3((coordinate.xy*0.5+0.5)*transform.viewport,coordinate.z);
}
void main() {
    vec3 a=viewportPosition(gl_in[2].gl_Position)-viewportPosition(gl_in[1].gl_Position);
    vec3 b=viewportPosition(gl_in[2].gl_Position)-viewportPosition(gl_in[0].gl_Position);
    vec3 c=viewportPosition(gl_in[1].gl_Position)-viewportPosition(gl_in[0].gl_Position);
    float area=abs(b.x*c.y-b.y*c.x);
    vec3 height=max(vec3(area)/vec3(length(a),length(b),length(c)),vec3(10.0*transform.edgeWidth));
    float skipped=transform.skipFirst!=0u?transform.edgeWidth+1.0:0.0;
    for(int index=0;index<3;++index) {
        vertexColor=inputColor[index]; surfaceUV=inputUV[index];
        edgeDistance=index==0?vec3(height.x,0.0,skipped):index==1?vec3(0.0,height.y,skipped):vec3(0.0,0.0,height.z);
#ifdef MARTENSITE_LIGHTING
        worldNormal=inputNormal[index]; viewDirection=inputDirection[index]; worldPosition=inputPosition[index];
        frontColor=inputFront[index]; backColor=inputBack[index];
#ifdef MARTENSITE_FIXED_LIGHTING
        fixedFrontColor=inputFixedFront[0]; fixedBackColor=inputFixedBack[0];
#endif
#endif
#ifdef MARTENSITE_CLIPPING
        clippingWorldPosition=inputClipping[index];
#endif
        gl_Position=gl_in[index].gl_Position; EmitVertex();
    }
    EndPrimitive();
}
