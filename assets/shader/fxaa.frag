#version 450

#include "header/math.frag"

#define FXAA_PC 1
#define FXAA_GLSL_130 1
#define FXAA_GREEN_AS_LUMA 1
#define FXAA_QUALITY_PRESET 23

#include "header/fxaa.frag"

layout(std140, set = 0, binding = 0) uniform FXAAUniformArgs {
    uniform float screen_width;
    uniform float screen_height;
};

//layout(set = 1, binding = 0) uniform sampler2D color;
layout(set = 1, binding = 0) uniform sampler2D color;

layout(location = 0) in VertexData {
    vec3 position;
    vec2 tex_coord;
} vertex;

layout(location = 0) out vec4 out_color;

void main(){
    vec4 zero = vec4(0.0, 0.0, 0.0, 0.0);
    out_color = FxaaPixelShader(vertex.tex_coord, zero, color, color, color, vec2(1.0/screen_width,1.0/screen_height), zero, zero, zero, 0.75, 0.125, 0.0833, 8.0, 0.125, 0.05, zero);

    //vec4 input_color = texture(color, vec2(vertex.tex_coord.x,vertex.tex_coord.y));
    // simply output the texture coordinate for now
    //out_color = input_color;
    //out_color = vec4(vertex.tex_coord, 0.0, 1.0);
}