#version 140

in vec3 v_normal;
in vec3 v_frag_pos;

out vec4 f_color;

//uniform mat4 view;
//uniform vec3 camera_pos;
uniform vec3 material_color;

void main() {
    f_color = vec4(material_color, 1.0);
}
