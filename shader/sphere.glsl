// Returns distance from the origin to a given Sphere for a Ray
float Ray_dist_to_Sphere(Ray r, Sphere s) {

    float a = r.dir.x*r.dir.x + r.dir.y*r.dir.y + r.dir.z*r.dir.z;
    float b = 2.0 * ((r.dir.x*r.origin.x + r.dir.y*r.origin.y + r.dir.z*r.origin.z) - (r.dir.x*s.pos.x + r.dir.y*s.pos.y + r.dir.z*s.pos.z));
    float c = (s.pos.x*s.pos.x + s.pos.y*s.pos.y + s.pos.z*s.pos.z) - 2.0 * (r.origin.x*s.pos.x + r.origin.y*s.pos.y + r.origin.z*s.pos.z) + (r.origin.x*r.origin.x + r.origin.y*r.origin.y + r.origin.z*r.origin.z) - s.r*s.r;
    float delta = b*b - 4.0*a*c;

    if (delta >= 0.0) {
        float sq_delta = sqrt(delta);

        float t1 = (-b - sq_delta) / (2.0 * a);
        float t2 = (-b + sq_delta) / (2.0 * a);

        if (t1 > RAY_COLLISION_PRECISION) { // Do not detect objects behind the ray
            return t1;
        } else if (t2 > RAY_COLLISION_PRECISION) {
            return t2;
        }
    }

    return -1.0;
}

// Traces the given ray to all spheres on the scene and returns the closest_d and writes to closest_si the index of the detected sphere
// closest_si == SPHERES_LENGTH if no ray collisions 
float Ray_trace_to_Spheres(Ray r, out uint closest_si) {
    closest_si = SPHERES_LENGTH; // Arbitrary default value, used to check if a collision has been detected
    float closest_d = 1.0 / 0.0; // infinity
    
    for (int i = 0; i < SPHERES_LENGTH; i++) { // Loop through objects in scene to find the closest ray collision
        float dist = Ray_dist_to_Sphere(r, spheres[i]);

        if ((dist != -1.0) && (dist < closest_d)) {
            closest_d = dist;
            closest_si = i;
        }        
    }

    return closest_d;
}

vec2 point_to_geo(vec4 point, Sphere s) {
    vec3 sphereNormal = normalize(point - s.pos).xyz;
            
    vec3 northVector = vec3(0, 1, 0);
    vec3 eastVector  = vec3(1, 0, 0);
    
    vec3 vertPoint = sphereNormal;
    
    float lat = acos(dot(northVector, sphereNormal));
    float v = lat / PI;
    float u;
    
    float lon = (acos( dot( vertPoint, eastVector) / sin(lat))) / (2.0 * PI);
    if(dot(cross(northVector, eastVector), vertPoint) > 0.0){
        u = lon;
    }
    else{
        u = 1.0 - lon;
    }

    return vec2(-u, v);
}

vec3 get_color(Sphere s, vec4 impact_point) {
    // Computing U, V coordinates for the sphere, https://en.wikipedia.org/wiki/UV_mapping
    if (s.texture_index != -1) {
        vec2 uv = point_to_geo(impact_point, s);

        return texture(textures[s.texture_index], uv).xyz;
    }
    return s.col.xyz;
}

vec4 get_normal(Sphere s, vec4 impact_point) {
    vec4 normal = normalize(impact_point - s.pos);
    return normal;
}