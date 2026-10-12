// Native OCCT 3d097a0 DeclarationsImpl: peeling depth classification precedes clipping.
layout(set=3,binding=0,std430) readonly buffer OitParameters { vec4 settings; } oit;
#include "native_fragment_alpha.glsl"
#if defined(MARTENSITE_OIT_WEIGHTED)
layout(location=0) out vec4 oitAccumulation;
layout(location=1) out vec4 oitCoverage;
bool oitEarlyReturn() { return false; }
void oitWriteColor(vec4 color) {
#ifdef MARTENSITE_OIT_FIXED_OUTPUT
    oitAccumulation=color;oitCoverage=color;
#else
    float weight=color.a*clamp(100.0*pow(1.0-gl_FragCoord.z*oit.settings.x,3.0),0.01,100.0);
    oitAccumulation=vec4(color.rgb*color.a*weight,color.a);
    oitCoverage=vec4(color.a*weight,0.0,0.0,0.0);
#endif
    nativeFragmentAlpha(oitAccumulation.a,oit.settings);
}
#elif defined(MARTENSITE_OIT_PEELING)
layout(set=3,binding=1) uniform sampler2D previousDepth;
layout(set=3,binding=2) uniform sampler2D previousFront;
layout(location=0) out vec4 oitDepth;
#ifndef MARTENSITE_OIT_INITIAL
layout(location=1) out vec4 oitFront;
layout(location=2) out vec4 oitBack;
int oitFrontLayer=-1;
#endif
bool oitReturnDepth() {
    nativeFragmentAlpha(oitDepth.a,oit.settings);
    return true;
}
bool oitEarlyReturn() {
#ifdef MARTENSITE_OIT_FIXED_OUTPUT
    return false;
#elif defined(MARTENSITE_OIT_INITIAL)
    // The initial native previous range is [-0,1]; exact boundary fragments don't extend it.
    float depth=gl_FragCoord.z;
    oitDepth=depth>0.0&&depth<1.0?vec4(-depth,depth,0.0,0.0):vec4(-1e15,-1e15,0.0,0.0);
    return oitReturnDepth();
#else
    ivec2 pixel=ivec2(gl_FragCoord.xy);
    vec2 bounds=texelFetch(previousDepth,pixel,0).rg;
    oitFront=texelFetch(previousFront,pixel,0);
    oitDepth=vec4(-1e15,-1e15,0.0,0.0);oitBack=vec4(0.0);
    float nearDepth=-bounds.x;float farDepth=bounds.y;float depth=gl_FragCoord.z;
    if(depth<nearDepth||depth>farDepth)return oitReturnDepth();
    if(depth>nearDepth&&depth<farDepth){oitDepth.rg=vec2(-depth,depth);return oitReturnDepth();}
    oitFrontLayer=depth==nearDepth?1:0;return false;
#endif
}
void oitWriteColor(vec4 color) {
#ifdef MARTENSITE_OIT_FIXED_OUTPUT
    oitDepth=color;
#ifndef MARTENSITE_OIT_INITIAL
    oitFront=color;oitBack=color;
#endif
#elif !defined(MARTENSITE_OIT_INITIAL)
    if(oitFrontLayer==1){
        float transmittance=1.0-oitFront.a;
        oitFront.rgb+=color.rgb*color.a*transmittance;
        oitFront.a=1.0-transmittance*(1.0-color.a);
    }else if(oitFrontLayer==0){oitBack=color;}
#endif
    nativeFragmentAlpha(oitDepth.a,oit.settings);
}
#endif
