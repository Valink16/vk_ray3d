
// Checks if the ray collides with the given triangle, give ABC in counterclockwise order viewed from the exterior for culling, return -1.0 for no collision
float Ray_dist_to_Triangle(Ray r, vec3 A, vec3 B, vec3 C) {
	/*
	Derived from a linear system of equation solved using cramer:
	x: (u, v, t)
	M: (
		(AB.x, AB.y, AB.z)
		(AC.x, AC.y, AC.z)
		(-d.x, -d.y, -d.z)
	)
	y: (P.x, P.y, P.z)
	where:
		d is the directional vector of the ray
		AB and AC are (B - A) and (C - A) where A, B, C are the points of the triangle
		P is (d0 - A) where d0 is the origin of the ray
	*/
	
	vec3 d = r.dir.xyz;
	vec3 AB = B - A;
	vec3 AC = C - A;

	vec3 AC_cross_neg_d = cross(AC, -d);
	float detm = dot(AB, AC_cross_neg_d);

	if (detm <= 0) { // Ray is parallel to the triangle
		return -1.0;
	}

	vec3 P = r.origin.xyz - A;
	float detu = dot(P, AC_cross_neg_d);
	float u = detu / detm;
	
	if (u >= 0 && u <= 1) {
		vec3 P_cross_AB = cross(P, AB);
		float detv = dot(d, P_cross_AB);
		float v = detv / detm;

		if ((v >= 0 && v <= 1) && (u + v) <= 1) {
			float dett = dot(P_cross_AB, AC);

			float t = dett / detm;
			if (t > RAY_COLLISION_PRECISION) {
				return t;
			}
		}
	}

	return -1.0;
}

// Returns the dist to a model for a specific ray, returns -1.0 if no collision
float Ray_dist_to_Model(Ray r, Model model, out uint closest_tri_index) {
	float closest_tri_dist = 1.0 / 0.0;
	vec3 pos = model.pos.xyz;

	for (uint i = model.indices_start; i < model.indices_end; i++) {
		uvec3 indexed_tri = indices[i];
		vec3 A = vertices[indexed_tri[0]] + pos;
		vec3 B = vertices[indexed_tri[1]] + pos;
		vec3 C = vertices[indexed_tri[2]] + pos;

		float d = Ray_dist_to_Triangle(r, A, B, C);
		if (d != -1.0 && d < closest_tri_dist) {
			closest_tri_dist = d;
			closest_tri_index = i;
		}
	}

	if (closest_tri_dist == 1.0 / 0.0) {
		return -1.0;
	} else {
		return closest_tri_dist;
	}
}

// Traces the given ray to all spheres on the scene and returns the closest_d and writes to closest_mi the index of the detected model
// closest_mi == MODELS_LENGTH if no ray collisions
float Ray_trace_to_Models(Ray r, out uint closest_mi, out uint closest_tri_index) {
	closest_mi = MODELS_LENGTH;
	
	float closest_model_dist = 1.0 / 0.0;

	uint _closest_tri_index = 0;

	for (int i = 0; i < MODELS_LENGTH; i++) {
		float d = Ray_dist_to_Model(r, models[i], _closest_tri_index);
		if (d != -1.0 && d < closest_model_dist) {
			closest_model_dist = d;
			closest_tri_index = _closest_tri_index;
			closest_mi = i;
		}
	}

	return closest_model_dist;
}