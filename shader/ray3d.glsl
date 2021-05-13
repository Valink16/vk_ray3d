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
    float reflexivity; // When computing reflections, factor of the incoming light reflected
    float diffuse_factor; // When computing reflections, factor of the added diffuse light to the incoming reflected light
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
        // The following arrays store the data about the reflection to then bactrace from the last impact and find the final color
        vec4 impact_points[REFLECT_DEPTH];
        uint impact_sindices[REFLECT_DEPTH];
        float impact_distance = 0.0;

        Sphere closest_s = spheres[closest_si];
        vec4 impact_point = r.origin + r.dir * closest_d;
        vec4 normal = normalize(impact_point - closest_s.pos);

        r.origin = impact_point;
        r.dir = reflect(r.dir, normal); // Used to iterate through reflections
        impact_points[0] = impact_point;
        impact_sindices[0] = closest_si;
        impact_distance += closest_d;

        int i; // So we can keep track of when the for loop stopped for later
        for (i = 1; i < REFLECT_DEPTH; i++) {
            closest_d = Ray_trace_to_Spheres(r, closest_si);
            
            if (closest_si == SPHERES_LENGTH) { // No collision, so stop the loop, because the ray "goes" into infinity
                break;
            }

            impact_points[i] = r.origin + r.dir * closest_d;
            impact_sindices[i] = closest_si;
            impact_distance += closest_d;
            
            normal = normalize(impact_points[i] - spheres[closest_si].pos);
            r.origin = impact_points[i]; // New origin is the impact point
            r.dir = reflect(r.dir, normal); // Used to iterate through reflections
        }

        if (i > 0) {
            --i; // or index out of range

            vec3 reflected_color = PointLights_to_Sphere(impact_points[i], spheres[impact_sindices[i]], r) * spheres[impact_sindices[i]].reflexivity; // This is the color on the "last" impact, we'll now compose it using the colors of the objects on our light path

            for (int a = i - 1; a >= 0; a--) {
                Sphere _sph = spheres[impact_sindices[a]];
                vec3 dif = PointLights_to_Sphere(impact_points[a], _sph, r);
                reflected_color *= _sph.col.xyz * _sph.reflexivity;
                reflected_color += dif * _sph.diffuse_factor;
            }
            col = vec4(reflected_color, 1.0);
        }
    }

    imageStore(img, ivec2(gl_GlobalInvocationID.xy), col);
}

