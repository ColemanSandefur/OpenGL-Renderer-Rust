#version 330 core
out vec4 f_color;

in vec3 v_position;

uniform sampler2D equirectangular_map;

const vec2 invAtan = vec2(0.1591, 0.3183);
vec2 SampleSphericalMap(vec3 v) {
    vec2 uv = vec2(atan(v.z, v.x), asin(v.y));
    uv *= invAtan;
    uv += 0.5;
    return uv;
}

void main() {
    vec2 uv = SampleSphericalMap(normalize(v_position));
    vec3 color = texture(equirectangular_map, uv).rgb;

    f_color = vec4(color, 1.0);
}
