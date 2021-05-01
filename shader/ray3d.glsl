#version 450

layout(local_size_x = 8, local_size_y = 8, local_size_z = 1) in;

layout(set = 0, binding = 0, r8) uniform writeonly image2D img;

struct Ray {
    vec4 origin;
    vec4 dir;
};

struct Sphere {
    vec4 pos;
    vec3 col;
    float r;
};

layout(set = 0, binding = 1, std430) buffer Rays {
    Ray rays[];
};

layout(set = 0, binding = 2, std430) buffer Spheres {
    Sphere spheres[];
};

#include "sphere.glsl"

uint SPHERES_LENGTH = spheres.length();

void main() {
    ivec2 img_size = imageSize(img);
    uint ri = gl_GlobalInvocationID.y * img_size.x + gl_GlobalInvocationID.x;

    vec4 col = vec4(0.0, 0.0, 0.0, 1.0);

    uint closest_si = SPHERES_LENGTH;
    float closest_dist = 1.0 / 0.0;
    
    for (int i = 0; i <= SPHERES_LENGTH; i++) {
        float dist = Ray_dist_to_Sphere(ri, i);

        if ((dist != -1.0) && (dist < closest_dist)) {
            closest_dist = dist;
            closest_si = i;
        }        
    }
    
    /*
    if (rays.length() == 120000) {
        col = vec4(1.0);
    }
    */

    if (closest_si != SPHERES_LENGTH) {
        col = vec4(1.0);
    }

    // col = vec4(abs(r.dir.x), abs(r.dir.y), abs(r.dir.z), 1.0);

    imageStore(img, ivec2(gl_GlobalInvocationID.xy), col);
}

