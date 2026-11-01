#version 120

varying vec4 out_Pos;
varying vec3 out_Norm;
varying vec2 out_Uv;
varying float out_Occlusion;

uniform sampler2D _Text;

void main() {
    vec4 albedo = texture2D(_Text, out_Uv);
    vec3 lightDir = normalize(vec3(-20.0, 10.0, 20.0) - out_Pos.xyz);
    float diffuse = max(dot(normalize(out_Norm), lightDir), 0.0);
    float ambient = 0.3 * out_Occlusion;

    gl_FragColor = vec4(albedo.rgb * (ambient + diffuse), albedo.a);
}