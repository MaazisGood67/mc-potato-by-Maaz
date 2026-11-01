#version 120

attribute vec3 in_Pos;
attribute vec3 in_Normal;

varying vec3 fs_Normal;

uniform mat4 _RotX;
uniform mat4 _RotY;

void main() {
    fs_Normal = in_Normal;

    mat4 mat = _RotY * _RotX;

    gl_Position = mat * vec4(in_Pos, 1.0);
    gl_Position.z = -0.2;
}