#version 330 core
in vec2 v_tex_coords;
in vec3 v_world_pos;
in vec3 v_normal;

out vec4 f_color;

uniform vec3 camera_pos;
uniform vec3 light_pos;
uniform vec3 light_color;

uniform vec3 albedo;
uniform float metallic;
uniform float roughness;
uniform float ao;

uniform samplerCube skybox;

const float PI = 3.14159265359;

float DistributionGGX(vec3 N, vec3 H, float roughness) {
    float a = roughness * roughness;
    float a2 = a * a;
    float NdotH = max(dot(N, H), 0.0);
    float NdotH2 = NdotH * NdotH;

    float nom = a2;
    float denom = (NdotH2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;

    return nom/denom;
}

float GeometrySchlickGGX(float NdotV, float roughness) {
    float r = (roughness + 1.0);
    float k = (r*r) / 8.0;

    float nom   = NdotV;
    float denom = NdotV * (1.0 - k) + k;

    return nom / denom;
}

float GeometrySmith(vec3 N, vec3 V, vec3 L, float roughness)
{
    float NdotV = max(dot(N, V), 0.0);
    float NdotL = max(dot(N, L), 0.0);
    float ggx2 = GeometrySchlickGGX(NdotV, roughness);
    float ggx1 = GeometrySchlickGGX(NdotL, roughness);

    return ggx1 * ggx2;
}

vec3 fresnelSchlick(float cosTheta, vec3 F0)
{
    return F0 + (1.0 - F0) * pow(clamp(1.0 - cosTheta, 0.0, 1.0), 5.0);
}

void main() {
    vec3 N = normalize(v_normal);
    vec3 V = normalize(camera_pos - v_world_pos);

    vec3 F0 = vec3(0.04);
    F0 = mix(F0, albedo, metallic);
    
    vec3 Lo = vec3(0.0);

    // For each light
    {
        vec3 L = normalize(light_pos - v_world_pos);
        vec3 H = normalize(V + L);
        float distance = length(light_pos - v_world_pos);
        float attenuation = 1.0 / (distance * distance);
        vec3 radiance = light_color * attenuation;

        float NDF = DistributionGGX(N, H, roughness); // Normal distribution function
        float G = GeometrySmith(N, V, L, roughness); // Geometry function
        vec3 F = fresnelSchlick(clamp(dot(H, V), 0.0, 1.0), F0); // Fresnel equation

        vec3 numerator = NDF * G * F;
        float denominator = 4.0 * max(dot(N,V), 0.0) * max(dot(N, L), 0.0) + 0.0001; // +0.0001 prevents divide by zero
        vec3 specular = numerator / denominator;

        vec3 kS = F; // Specular
        vec3 kD = vec3(1.0) - kS; // Diffuse
        kD *= 1.0 - metallic;

        float NdotL = max(dot(N, L), 0.0);

        Lo += (kD * albedo / PI + specular) * radiance * NdotL;
    }

    vec3 ambient = vec3(0.03) * albedo * ao;

    vec3 color = ambient + Lo;

    color = color / (color + vec3(1.0));
    color = pow(color, vec3(1.0/2.2));

    f_color = vec4(color, 1.0);
}
