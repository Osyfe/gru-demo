#version 450 core

#include "text.glsl"

layout (location=0) in vec3 coords;

layout (location=0) out vec4 frag_color;

layout (set=0, binding=1) uniform sampler2DArray atlas;

float contour(float d, float w)
{
    return smoothstep(0.5 - w, 0.5 + w, d);
}

float samp(vec3 uv, float w)
{
    return contour(texture(atlas, uv).r, w);
}

void main()
{
    if(coords.p < -0.5) frag_color = vec4(0.2, 0.2, 0.6, 1.0);
    else
    {
        vec3 uv = coords;
        float dist = texture(atlas, uv).r;
        float width = fwidth(dist);
        float alpha = contour(dist, width);
        float dscale = 0.354; //half of 1/sqrt2
        vec2 duv = dscale * (dFdx(uv.st) + dFdy(uv.st));
        vec4 box = vec4(uv.st - duv, uv.st + duv);
        float asum =
            samp(vec3(box.xy, uv.p), width)
          + samp(vec3(box.zw, uv.p), width)
          + samp(vec3(box.xw, uv.p), width)
          + samp(vec3(box.zy, uv.p), width);
        alpha = (alpha + 0.5 * asum) / 3.0;
        frag_color = vec4(0.0, 0.1, 0.8, alpha);

        //float sd = texture(tex, coords).r;
        //float alpha = smoothstep(0.5 - SIG, 0.5 + SIG, sd);
        //frag_color = vec4(0.0, 0.1, 0.8, alpha);
    }
}
