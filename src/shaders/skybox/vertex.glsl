#version 330 core
in vec3 position;

out vec3 v_tex_coords;

uniform mat4 projection;
uniform mat4 view;

void main() {
    v_tex_coords = position;
    vec4 pos = projection * mat4(mat3(view ))* vec4(position, 1.0);
    gl_Position = pos.xyww;
}
