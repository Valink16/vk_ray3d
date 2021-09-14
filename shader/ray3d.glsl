#version 450
#extension GL_EXT_debug_printf : enable

layout(local_size_x = 8, local_size_y = 8, local_size_z = 1) in;
layout(set = 0, binding = 0) uniform writeonly image2D img;

#include "consts.glsl"

struct Ray {
    vec4 origin;
    vec4 dir;
};

struct Sphere {
    vec4 pos;
    vec4 col;
    float r;
    float reflexivity; // When computing reflections, factor of the incoming light reflected
    float diffuse_factor; // When computing reflections, factor of the added diffuse light to the incoming reflected light
    uint texture_index;
};

struct Model {
    vec4 pos;
    vec4 col;
    float reflexivity; // When computing reflections, factor of the incoming light reflected
    float diffuse_factor; // When computing reflections, factor of the added diffuse light to the incoming reflected light
    uint indices_start; // Index of the first indexed triangle of the model in the global indexed triangles array
    uint indices_end; // End of the indexed triangles
    int texture_index;
};

struct PointLight {
    vec4 pos;
    vec3 col;
    float intensity;
};

struct DirectionalLight {
    vec4 dir;
    vec3 col;
    float intensity;
};

layout(set = 0, binding = 1, std430) buffer Rays {
    Ray rays[];
};

layout(set = 0, binding = 2, std430) buffer Spheres {
    Sphere spheres[];
};

layout(set = 0, binding = 3, std430) buffer Models {
    Model models[];
};

layout(set = 0, binding = 4, std430) buffer Vertices {
    vec3 vertices[];
};

layout(set = 0, binding = 5, std430) buffer UVs {
    vec2 uvs[];
};

layout(set = 0, binding = 6, std430) buffer Indices {
    uvec3 indices[];
};

layout(set = 0, binding = 7, std430) buffer Normals {
    vec3 normals[];
};

layout(set = 0, binding = 8, std430) buffer PointLights {
    PointLight point_lights[];
};

layout(set = 0, binding = 9, std430) buffer DirectionalLights {
    DirectionalLight directional_lights[];
};

layout(set = 0, binding = 10) uniform sampler2D textures[2];

layout(push_constant) uniform Camera {
    vec4 pos;
    vec4 orientation; // Quaternion
} camera;

uint SPHERES_LENGTH = spheres.length();
uint MODELS_LENGTH = models.length();
uint POINT_LIGHTS_COUNT = point_lights.length();
uint DIR_LIGHTS_COUNT = directional_lights.length();

#include "quaternion.glsl"
#include "sphere.glsl"
#include "model.glsl"
#include "light.glsl"

void main() {
    ivec2 img_size = imageSize(img);
    uint ri = gl_GlobalInvocationID.y * img_size.x + gl_GlobalInvocationID.x;
    Ray r = rays[ri];

    r.origin += camera.pos;
    r.dir.xyz = transform_point(camera.orientation, r.dir.xyz);

    vec4 col = vec4(0.0, 0.0, 0.0, 1.0);

    /*
    uint closest_si;
    float closest_d = Ray_trace_to_Spheres(r, closest_si);

    if (closest_si != SPHERES_LENGTH) {
        // The following arrays store the data about the reflection to then bactrace from the last impact and find the final color
        vec4 impact_points[REFLECT_DEPTH];
        uint impact_sindices[REFLECT_DEPTH];
        float impact_distances[REFLECT_DEPTH];

        Sphere closest_s = spheres[closest_si];
        vec4 impact_point = r.origin + r.dir * closest_d;
        vec4 normal = normalize(impact_point - closest_s.pos);

        r.origin = impact_point;
        r.dir = reflect(r.dir, normal); // Used to iterate through reflections
        impact_points[0] = impact_point;
        impact_sindices[0] = closest_si;
        impact_distances[0] = closest_d; // We choose arbitrarily to ignore the camera

        vec3 skybox_emissive = vec3(0.0);
        
        int i; // So we can keep track of when the for loop stopped for later
        for (i = 1; i < REFLECT_DEPTH; i++) {
            closest_d = Ray_trace_to_Spheres(r, closest_si);
            
            if (closest_si == SPHERES_LENGTH) { // No collision, so stop the loop, because the ray "goes" into infinity
                break;
            }

            impact_points[i] = r.origin + r.dir * closest_d;
            impact_sindices[i] = closest_si;
            impact_distances[i] = closest_d;
            
            normal = normalize(impact_points[i] - spheres[closest_si].pos);
            r.origin = impact_points[i] + normal * RAY_COLLISION_PRECISION; // New origin is the impact point
            r.dir = reflect(r.dir, normal); // Used to iterate through reflections
        }

        if (i > 0) {
            --i; // or index out of range

            Sphere _sph = spheres[impact_sindices[i]];

            vec3 texture_color = Sphere_texture_value(_sph, impact_points[i]);
            vec3 reflected_color = PointLights_to_Sphere(impact_points[i], _sph, r) * texture_color * _sph.diffuse_factor;
            
            for (int a = i - 1; a >= 0; a--) {
                _sph = spheres[impact_sindices[a]];
                // vec3 dif = PointLights_to_Sphere(impact_points[a], _sph, r) * _sph.diffuse_factor;
                // Computing U, V coordinates for the sphere, https://en.wikipedia.org/wiki/UV_mapping
                
                vec3 texture_color = Sphere_texture_value(_sph, impact_points[a]);
                
                float impact_dist = impact_distances[a + 1];

                float df = 1 / (impact_dist * impact_dist);
                reflected_color *= (texture_color * _sph.reflexivity * df);
                reflected_color += (PointLights_to_Sphere(impact_points[a], _sph, r) * texture_color * _sph.diffuse_factor); // Add diffused light
            }

            float last_df = 1 / (impact_distances[0] * impact_distances[0]);
            col = vec4(reflected_color, 1.0);
            // col = texture(textures[0], vec2(0.05, 0.05));
        }
    }
    */

    uint closest_mi;
    uint closest_tri_index;
    vec2 uv;
    float closest_model_dist = Ray_trace_to_Models(r, closest_mi, closest_tri_index, uv);

    if (closest_mi != MODELS_LENGTH) {
        Model _mod = models[closest_mi];

        vec4 impact_point = r.origin + r.dir * closest_model_dist;

        if (_mod.texture_index == -1) {
            col.xyz = _mod.col.xyz;
        } else {
            uvec3 indexed_tri = indices[closest_tri_index];
            vec2 tex_A = uvs[indexed_tri.x];
            vec2 tex_B = uvs[indexed_tri.y];
            vec2 tex_C = uvs[indexed_tri.z];

            vec2 tex_AB = tex_B - tex_A;
            vec2 tex_AC = tex_C - tex_A;

            uv = tex_A + uv.x * tex_AB + uv.y * tex_AC;

            col.xyz = Model_texture_value(_mod, uv);
        }
        
        
        // 3col.xyz += vec3(0.0, 0.0, 0.5);
        
        // col.xyz = _mod.col.xyz;
        col.xyz *= PointLights_to_Model(impact_point, _mod, r, closest_tri_index) * _mod.diffuse_factor;
        /*
        vec4 impact_points[REFLECT_DEPTH];
        uint impact_sindices[REFLECT_DEPTH];
        uint tri_indices[REFLECT_DEPTH]; // Stores the triangle index for each reflection
        float impact_distances[REFLECT_DEPTH];

        Model closest_m = models[closest_mi];
        
        // vec4 normal = normalize(impact_point - closest_m.pos);
        
        // Compute normal with the triangle
        // vec3 AB = vertices[indices[closest_tri_index][1]] - vertices[indices[closest_tri_index][0]]; // B - A
        // vec3 AC = vertices[indices[closest_tri_index][2]] - vertices[indices[closest_tri_index][0]]; // C - A
        // vec4 normal = vec4(normalize(cross(AC, AB)), 0.0);
        vec4 normal = vec4(normals[closest_tri_index], 0.0);

        r.origin = impact_point;
        r.dir = reflect(r.dir, normal); // Used to iterate through reflections
        impact_points[0] = impact_point;
        impact_sindices[0] = closest_mi;
        tri_indices[0] = closest_tri_index;
        impact_distances[0] = closest_model_dist; // We choose arbitrarily to ignore the camera
        float total_distance = closest_model_dist;

        int i; // So we can keep track of when the for loop stopped for later
        for (i = 1; i < REFLECT_DEPTH; i++) {
            closest_model_dist = Ray_trace_to_Models(r, closest_mi, closest_tri_index);
            
            if (closest_mi == MODELS_LENGTH) { // No collision, so stop the loop, because the ray "goes" into infinity
                break;
            }

            impact_points[i] = r.origin + r.dir * closest_model_dist;
            impact_sindices[i] = closest_mi;
            tri_indices[i] = closest_tri_index;
            impact_distances[i] = closest_model_dist;
            total_distance += closest_model_dist;
            
            // AB = vertices[indices[closest_tri_index][1]] - vertices[indices[closest_tri_index][0]]; // B - A
            // AC = vertices[indices[closest_tri_index][2]] - vertices[indices[closest_tri_index][0]]; // C - A
            // normal = vec4(normalize(cross(AC, AB)), 0.0);
            normal = vec4(normals[closest_tri_index], 0.0);

            r.origin = impact_points[i] + normal * RAY_COLLISION_PRECISION; // New origin is the impact point
            r.dir = reflect(r.dir, normal); // Used to iterate through reflections
        }


        if (i > 0) {
            --i; // or index out of range

            Model _mod = models[impact_sindices[i]];
            vec3 reflected_color = PointLights_to_Model(impact_points[i], _mod, r, tri_indices[i]) * _mod.col.xyz * _mod.diffuse_factor;

            for (int a = i - 1; a >= 0; a--) {
                _mod = models[impact_sindices[a]];
                // vec3 dif = PointLights_to_Model(impact_points[a], _mod, r, closest_tri_index) * _mod.diffuse_factor;
                
                float impact_dist = impact_distances[a + 1]; // 
                if (impact_dist < 0.8) {
                    impact_dist = 0.8;
                }

                float df = 1 / (impact_dist * impact_dist);

                reflected_color *= _mod.col.xyz * _mod.reflexivity * df;
                reflected_color += PointLights_to_Model(impact_points[a], _mod, r, tri_indices[a]) * _mod.col.xyz * _mod.diffuse_factor; // Add diffused light
            }
            col = vec4(reflected_color, 1.0);
        }
        */
    }
    
    imageStore(img, ivec2(gl_GlobalInvocationID.xy), col);
}

