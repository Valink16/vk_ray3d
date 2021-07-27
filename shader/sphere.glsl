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