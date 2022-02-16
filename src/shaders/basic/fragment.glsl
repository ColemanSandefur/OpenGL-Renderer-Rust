#version 330 core

struct Material {
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
    float shininess;
};

in vec3 v_normal;
in vec3 v_position;
flat in Material v_material;

out vec4 color;

uniform vec3 u_light;
uniform vec3 camera_pos;

const vec3 ambient_color = vec3(0.2, 0.2, 0.2);
const vec3 diffuse_color = vec3(0.6, 0.6, 0.6);
const vec3 specular_color = vec3(1.0, 1.0, 1.0);

void main() {
    vec3 ambient = ambient_color * v_material.ambient;
    
    // diffuse
    vec3 norm = normalize(v_normal);
    vec3 light_dir = normalize(u_light - v_position);
    float diff = max(dot(norm, light_dir), 0.0);
    vec3 diffuse = diffuse_color * (diff * v_material.diffuse);

    // specular
    vec3 view_dir = normalize(camera_pos - v_position);
    vec3 reflect_dir = reflect(-light_dir, norm);
    float spec = pow(max(dot(view_dir, reflect_dir), 0.0), v_material.shininess);
    vec3 specular = specular_color * (spec * v_material.specular);

    vec3 result = ambient + diffuse + specular;

    color = vec4(result, 1.0);
}
