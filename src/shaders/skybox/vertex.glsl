#version 330 core
in vec3 position;

out vec3 v_tex_coords;

uniform mat4 projection;
uniform mat4 view;
uniform mat4 model;

void main() {
    v_tex_coords = position;
    //vec3 v_world_pos = vec3(mat4(mat3(model)) * vec4(position, 1.0));
    vec3 v_world_pos = vec3(model * vec4(position, 1.0));

    vec4 pos = projection * mat4(mat3(view)) * vec4(v_world_pos, 1.0);
    gl_Position = pos.xyww;
}
