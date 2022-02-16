#version 330 core
out vec4 color;

in vec3 v_tex_coords;

uniform samplerCube skybox;

void main() {
    color = texture(skybox, v_tex_coords);
}
