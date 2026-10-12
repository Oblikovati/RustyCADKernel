vec4 litFragmentColor() {
    // OCCT 3d097a0 line/marker shaders always evaluate the front material.
#if defined(MARTENSITE_LINE_LIGHTING) || defined(MARTENSITE_MARKER_LIGHTING)
    bool frontFacing = true;
#else
    bool frontFacing = gl_FrontFacing;
#endif
#ifdef MARTENSITE_FIXED_LIGHTING
    vec4 result=lighting.settings.x==4u?(frontFacing?fixedFrontColor:fixedBackColor)
        :(frontFacing?frontColor:backColor);
    bool colored=lighting.settings.z!=0u&&(frontFacing||(lighting.settings.w&1u)==0u);
    if(!colored)result.a=lighting.materials[frontFacing?0u:4u].a;
    return result;
#else
    if (lighting.settings.x == 1u) {
        return frontFacing ? frontColor : backColor;
    }
    vec3 normal = normalize(worldNormal);
    if (lighting.settings.x == 0u) {
        vec3 point = worldPosition.xyz / worldPosition.w;
        // Negative Vulkan viewport reverses screen Y relative to OpenGL derivatives.
        normal = normalize(-cross(dFdx(point), dFdy(point)));
        normal = gl_FrontFacing ? normal : -normal;
    }
    return computeLighting(normal, normalize(viewDirection), worldPosition, frontFacing, vertexColor);
#endif
}

