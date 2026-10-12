// OCCT3d097a0 PushState: GL_LIGHT_MODEL_LOCAL_VIEWER=false, TWO_SIDE=true.
vec3 fixedLightingNormal(vec3 objectNormal) {
    vec3 eyeNormal=(lighting.normalFromObject*vec4(objectNormal,0.0)).xyz;
    return (lighting.settings.w&2u)!=0u?normalize(eyeNormal):eyeNormal;
}
float fixedSpotAttenuation(vec3 direction,RenderLight light) {
    float cosine=dot(normalize(light.direction.xyz),-direction);
    if(cosine<cos(light.params.z))return 0.0;
    return pow(max(0.0,cosine),light.params.w*128.0);
}
vec3 fixedLightContribution(RenderLight light,vec3 faceNormal,vec4 eyePosition,vec4 diffuseMaterial,vec4 specularMaterial) {
    vec3 direction=light.position.xyz;
    float attenuation=1.0;
    if(light.position.w!=0.0) {
        direction=direction/light.position.w-eyePosition.xyz/eyePosition.w;
        attenuation=1.0/(light.params.x+light.params.y*length(direction));
    }
    direction=normalize(direction);
    if(light.kind.x==3u)attenuation*=fixedSpotAttenuation(direction,light);
    float diffuse=max(0.0,dot(faceNormal,direction));
    vec3 halfDirection=direction+vec3(0.0,0.0,1.0);
    float halfLength=length(halfDirection);
    float specular=diffuse>0.0&&halfLength>0.0
        ?pow(max(0.0,dot(faceNormal,halfDirection/halfLength)),specularMaterial.w):0.0;
    return light.color.rgb*attenuation*(diffuse*diffuseMaterial.rgb+specular*specularMaterial.rgb);
}
vec4 computeFixedLighting(vec3 normal,vec4 eyePosition,bool front,vec4 vertex) {
    uint face=front?0u:4u;
    bool colored=lighting.settings.z!=0u&&(front||(lighting.settings.w&1u)==0u);
    vec4 diffuseMaterial=colored?clamp(vertex,0.0,1.0):lighting.materials[face];
    vec3 ambientMaterial=colored?clamp(vertex.rgb,0.0,1.0):lighting.materials[face+3u].rgb;
    vec3 result=lighting.materials[face+1u].rgb+lighting.ambient.rgb*ambientMaterial;
    vec3 faceNormal=front?normal:-normal;
    for(uint i=0u;i<lighting.settings.y;++i) {
        result+=fixedLightContribution(lighting.sources[i],faceNormal,eyePosition,diffuseMaterial,lighting.materials[face+2u]);
    }
    return clamp(vec4(result,diffuseMaterial.a),0.0,1.0);
}
