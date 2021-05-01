float PREC = 0.001;

float Ray_dist_to_Sphere(uint ri, uint si) { // Returns from the origin to a given Sphere for a Ray
    Ray r = rays[ri];
    Sphere s = spheres[si];

    float a = r.dir.x*r.dir.x + r.dir.y*r.dir.y + r.dir.z*r.dir.z;
    float b = 2.0 * ((r.dir.x*r.origin.x + r.dir.y*r.origin.y + r.dir.z*r.origin.z) - (r.dir.x*s.pos.x + r.dir.y*s.pos.y + r.dir.z*s.pos.z));
    float c = (s.pos.x*s.pos.x + s.pos.y*s.pos.y + s.pos.z*s.pos.z) - 2.0 * (r.origin.x*s.pos.x + r.origin.y*s.pos.y + r.origin.z*s.pos.z) + (r.origin.x*r.origin.x + r.origin.y*r.origin.y + r.origin.z*r.origin.z) - s.r*s.r;
    float delta = b*b - 4.0*a*c;

    if (delta >= 0.0) {
        float sq_delta = sqrt(delta);

        float t1 = (-b - sq_delta) / (2.0 * a);
        float t2 = (-b + sq_delta) / (2.0 * a);

        if (t1 > PREC) { // Do not detect objects behind the ray
            return t1;
        } else if (t2 > PREC) {
            return t2;
        }
    }

    return -1.0;
}