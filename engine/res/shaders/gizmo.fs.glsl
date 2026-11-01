#version 120

varying vec3 fs_Normal;

void main() {
    gl_FragColor = vec4(fs_Normal, 1.0);
}