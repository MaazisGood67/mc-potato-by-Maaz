#version 120

attribute vec3 in_Pos;
attribute vec3 in_Norm;
attribute vec2 in_Uv;
attribute float in_Occlusion;

varying vec4 out_Pos;
varying vec3 out_Norm;
varying vec2 out_Uv;
varying float out_Occlusion;

uniform mat4 _Projection;
uniform mat4 _View;

void main() {
    mat4 mvp = _Projection * _View;

    out_Pos = vec4(in_Pos, 1.0);
    out_Norm = normalize(in_Norm);
    out_Uv = in_Uv;
    out_Occlusion = 1.0 - in_Occlusion;

    gl_Position = mvp * out_Pos;
}