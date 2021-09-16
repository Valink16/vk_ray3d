
// Checks if the ray collides with the given triangle, give ABC in counterclockwise order viewed from the exterior for culling, return -1.0 for no collision
float Ray_dist_to_Triangle(Ray r, vec3 A, vec3 B, vec3 C, out vec2 uv) {
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

	if (detm == 0) { // Ray is parallel to the triangle
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
				uv = vec2(u, v);
				// debugPrintfEXT("Base UV: (%f; %f)", uv.x, uv.y);
				return t;
			}
		}
	}

	return -1.0;
}

// Returns the dist to a model for a specific ray, returns -1.0 if no collision
float Ray_dist_to_Model(Ray r, Model model, out uint closest_tri_index, out vec2 uv) {
	float closest_tri_dist = 1.0 / 0.0;
	vec2 temp_uv = vec2(0.0);
	vec3 pos = model.pos.xyz;

	for (uint i = model.indices_start; i < model.indices_end; i++) {
		uvec3 indexed_tri = indices[i];
		vec3 A = vertices[indexed_tri.x] + pos;
		vec3 B = vertices[indexed_tri.y] + pos;
		vec3 C = vertices[indexed_tri.z] + pos;

		float d = Ray_dist_to_Triangle(r, A, B, C, temp_uv);
		if (d != -1.0 && d < closest_tri_dist) {
			closest_tri_dist = d;
			closest_tri_index = i;
			uv = temp_uv;
			// debugPrintfEXT("Base UV 2: (%f; %f)", uv.x, uv.y);
		}
	}

	if (closest_tri_dist == 1.0 / 0.0) {
		return -1.0;
	} else {
		return closest_tri_dist;
	}
}

// Traces the given ray to all spheres on the scene and returns the closest_d and writes to closest_mi the index of the detected model, uv is not interpolated
// closest_mi == MODELS_LENGTH if no ray collisions
float Ray_trace_to_Models(Ray r, out uint closest_mi, out uint closest_tri_index, out vec2 uv) {
	closest_mi = MODELS_LENGTH;
	
	float closest_model_dist = 1.0 / 0.0;

	uint _closest_tri_index = 0;
	vec2 temp_uv = vec2(0.0);

	for (int i = 0; i < MODELS_LENGTH; i++) {
		float d = Ray_dist_to_Model(r, models[i], _closest_tri_index, temp_uv);
		if (d != -1.0 && d < closest_model_dist) {
			closest_model_dist = d;
			closest_tri_index = _closest_tri_index;
			closest_mi = i;
			uv = temp_uv;
			//debugPrintfEXT("Base UV 3: (%f; %f)", uv.x, uv.y);
		}
	}

	return closest_model_dist;
}

vec3 Model_texture_value(Model m, vec2 uv) {
    return texture(textures[m.texture_index], uv).xyz;
}

vec3 get_color(Model m, uint tri_index, vec2 uv) {
	if (m.texture_index == -1) {
		return m.col.xyz;
	} else {
		uvec3 indexed_tri = indices[tri_index];
		vec2 tex_A = uvs[indexed_tri.x];
		vec2 tex_B = uvs[indexed_tri.y];
		vec2 tex_C = uvs[indexed_tri.z];

		vec2 tex_AB = tex_B - tex_A;
		vec2 tex_AC = tex_C - tex_A;

		vec2 tex_uv = tex_A + uv.x * tex_AB + uv.y * tex_AC;
		return Model_texture_value(m, tex_uv);
	}
}

vec4 get_normal(uint tri_index, vec2 uv) {
	/*
	vec3 n_A = normals[indices[tri_index][0]];
	vec3 n_B = normals[indices[tri_index][1]];
	vec3 n_C = normals[indices[tri_index][2]];
	return vec4(n_A + (n_B - n_A) * uv.x + (n_C - n_A) * uv.y, 0.0); // UV interpolated normal for smoooooth shading
	*/

	// Compute normal with the triangle
	vec3 AB = vertices[indices[tri_index][1]] - vertices[indices[tri_index][0]]; // B - A
	vec3 AC = vertices[indices[tri_index][2]] - vertices[indices[tri_index][0]]; // C - A
	vec4 normal = vec4(normalize(cross(AB, AC)), 0.0);
	return normal;
}