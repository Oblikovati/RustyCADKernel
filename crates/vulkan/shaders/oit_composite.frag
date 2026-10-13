#version 450
#extension GL_GOOGLE_include_directive : require
#include "native_fragment_alpha.glsl"
layout(set=0,binding=2,std430) readonly buffer CompositeAlphaParameters { vec4 settings; } compositeAlpha;
#ifdef MARTENSITE_OIT_MSAA
layout(set=0,binding=0) uniform sampler2DMS firstImage;
layout(set=0,binding=1) uniform sampler2DMS secondImage;
#define OIT_FETCH(image) texelFetch(image,ivec2(gl_FragCoord.xy),gl_SampleID)
#else
layout(set=0,binding=0) uniform sampler2D firstImage;
layout(set=0,binding=1) uniform sampler2D secondImage;
#define OIT_FETCH(image) texelFetch(image,ivec2(gl_FragCoord.xy),0)
#endif
layout(location=0) out vec4 fragmentColor;
void main() {
    vec4 first=OIT_FETCH(firstImage);
#ifdef MARTENSITE_OIT_BACK_BLEND
    if(first.a==0.0)discard;
    fragmentColor=first;
#elif defined(MARTENSITE_OIT_PEELING)
    vec4 back=OIT_FETCH(secondImage);
    // Native flush deliberately sums alpha without multiplying back alpha by transmittance.
    fragmentColor=vec4(first.rgb+(1.0-first.a)*back.rgb,first.a+back.a);
#else
    float weight=OIT_FETCH(secondImage).r;
    fragmentColor=vec4(first.rgb/max(weight,0.00001),first.a);
#endif
    nativeFragmentAlpha(fragmentColor.a,compositeAlpha.settings);
}
