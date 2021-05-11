#version 450
layout(local_size_x = 8, local_size_y = 8, local_size_z = 1) in;
layout(set = 0, binding = 0, r8) uniform writeonly image2D img;

struct Ray {
    vec4 origin;
    vec4 dir;
};

struct Sphere {
    vec4 pos;
    vec4 col;
    float r;
};

struct PointLight {
    vec4 pos;
    vec3 col;
    float intensity;
};

struct DirectionalLight {
    vec4 dir;
    vec3 col;
};

layout(set = 0, binding = 1, std430) buffer Rays {
    Ray rays[];
};

layout(set = 0, binding = 2, std430) buffer Spheres {
    Sphere spheres[];
};

layout(set = 0, binding = 3, std430) buffer PointLights {
    PointLight point_lights[];
};

layout(set = 0, binding = 4, std430) buffer DirectionalLights {
    DirectionalLight directional_lights[];
};

layout(push_constant) uniform Camera {
    vec4 pos;
    vec4 orientation; // Quaternion
} camera;

#include "consts.glsl"
#include "quaternion.glsl"
#include "sphere.glsl"
#include "light.glsl"

void main() {
    ivec2 img_size = imageSize(img);
    uint ri = gl_GlobalInvocationID.y * img_size.x + gl_GlobalInvocationID.x;
    Ray r = rays[ri];

    if (length(camera.pos) == 0.1) {
        imageStore(img, ivec2(gl_GlobalInvocationID.xy), vec4(1.0));
        return;
    }

    r.origin += camera.pos;
    r.dir.xyz = transform_point(camera.orientation, r.dir.xyz);

    vec4 col = vec4(0.0, 0.0, 0.0, 1.0);

    uint closest_si;
    float closest_d = Ray_trace_to_Spheres(r, closest_si);

    if (closest_si != SPHERES_LENGTH) {
        Sphere closest_s = spheres[closest_si];
        vec4 impact_point = r.origin + r.dir * closest_d;
        // float df = 1 / (closest_d * closest_d);
        // col = vec4(PointLights_to_Sphere(impact_point, closest_s, r), 1.0) * df;
        col = vec4(PointLights_to_Sphere(impact_point, closest_s, r), 1.0);

        // col = spheres[closest_si].col;
    }

    /*
        uint closest_si;
        float closest_d;

        Ray reflected = r; // Used to iterate through reflections

        // The following arrays store the data about the reflection to then bactrace from the last impact and find the final color
        vec4 impact_points[REFLECT_DEPTH];
        uint impact_sindices[REFLECT_DEPTH];
        float impact_distances[REFLECT_DEPTH];

        int i; // So we can keep track of when the for loop stopped for later
        for (i = 0; i < REFLECT_DEPTH; i++) {
            closest_d = Ray_trace_to_Spheres(reflected, closest_si);
            
            if (closest_si == SPHERES_LENGTH) {
                break;
            }

            impact_points[i] = reflected.origin + reflected.dir * closest_d;
            impact_sindices[i] = closest_si;
            impact_distances[i] = closest_d;
            
            vec4 normal = normalize(impact_points[i] - spheres[closest_si].pos);

            reflected = Ray (
                impact_points[i], // The new origin is the current impact point
                reflect(-reflected.dir, normal)
            );
        }
        
        if (i > 0) { // Means the ray collided at least once
            --i; // or index out of range

            Sphere closest_s = spheres[impact_sindices[i]];
            vec4 impact_point = impact_points[i];
            vec3 diffused_color = PointLights_to_Sphere(impact_points[i], closest_s, reflected);

            float total_dist = impact_distances[i];
            
            for (int a = i - 1; a > 0; a++) {
                total_dist += impact_distances[a];
            }

            float df = 1 / (total_dist * total_dist); // diffused color already factors in the light - object distance, this is the object camera distance factor
            col = vec4(diffused_color * closest_s.col.xyz * df, 1.0);
        }
    */

    imageStore(img, ivec2(gl_GlobalInvocationID.xy), col);
}

