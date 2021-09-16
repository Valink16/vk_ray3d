// Returns the total quantity of light on a sphere using point_lights
vec3 PointLights_to_Sphere(vec4 impact_point, Sphere closest_s, Ray r) {
	vec3 final_color = vec3(0.0);
	vec4 normal = normalize(impact_point - closest_s.pos);
	impact_point += normal * RAY_COLLISION_PRECISION;

	for (int i = 0; i < POINT_LIGHTS_COUNT; i++) {
		PointLight light = point_lights[i];
		vec4 to_light = (light.pos - impact_point);

		Ray ray_to_light = { // Create a new ray going from the impact point to the point light
			impact_point,
			to_light, // No need to normalize, because we don't need the exact distance if there is a collision
		};

		uint closest_si;
		Ray_trace_to_Spheres(ray_to_light, closest_si);

		if (closest_si == SPHERES_LENGTH || closest_si == 0) { // Means no collisions detected, thus no shadow, so we need to compute the color for the current light, 0 means skybox
			float to_light_sq_dist = dot(to_light, to_light);
			
			/*
			vec4 normal = impact_point - closest_s.pos;

			float a = acos(dot(to_light, normal) / (length(to_light) * length(normal)));

			float w = clamp(a / (PI / 2), 0.0, 1.0);

			float diffusion_factor = mix(1.0, 0.0, w);
			*/

			float diffusion_factor = clamp(dot(normal, to_light) / length(to_light), 0.0, 1.0);

			float distance_factor = (1 / to_light_sq_dist); // df is the distance factor, light intensity is proportional to the inverse of the distance squared
			final_color += light.col * light.intensity * distance_factor * diffusion_factor;
		}
	}

	for (int li = 0; li < DIR_LIGHTS_COUNT; li++) {
		DirectionalLight light = directional_lights[li];
		vec4 to_light = -light.dir;

		Ray ray_to_light = { // Create a new ray going from the impact point to the point light
			impact_point,
			to_light, // No need to normalize, because we don't need the exact distance if there is a collision
		};

		uint closest_si;
		Ray_trace_to_Spheres(ray_to_light, closest_si);
		
		if (closest_si == SPHERES_LENGTH) { // Means no collisions detected, thus no shadow, so we need to compute the color for the current light
			/*
			vec4 normal = impact_point - closest_s.pos;

			float a = acos(dot(to_light, normal) / (length(to_light) * length(normal)));

			float w = clamp(a / (PI / 2), 0.0, 1.0);

			float diffusion_factor = mix(1.0, 0.0, w);
			*/

			/*
			vec4 normal = vec4(normalize(cross(
                vertices[indices[_tri_index][1]] - vertices[indices[_tri_index][0]], // AB
                vertices[indices[_tri_index][2]] - vertices[indices[_tri_index][0]] // AC
			)), 0.0);
			*/

			float diffusion_factor = clamp(dot(normal, to_light) / length(to_light), 0.0, 1.0);

			final_color += light.col * diffusion_factor;
		}
	}

	return final_color;
}

// Returns the total quantity of light on a model using point_lights
vec3 PointLights_to_Model(vec4 impact_point, Model closest_m, Ray r, uint tri_index, vec2 uv) {
	vec3 final_color = vec3(0.0);

	// Compute normal with the triangle
	// vec3 AB = vertices[indices[tri_index][1]] - vertices[indices[tri_index][0]]; // B - A
	// vec3 AC = vertices[indices[tri_index][2]] - vertices[indices[tri_index][0]]; // C - A
	// vec4 _normal = vec4(normalize(cross(AB, AC)), 0.0);

	// vec4 normal = vec4((normals[indices[tri_index][0]] + normals[indices[tri_index][1]] + normals[indices[tri_index][2]]) / 3.0, 0.0); // Average normal
	
	vec4 normal = get_normal(tri_index, uv); // UV interpolated normal for smoooooth shading

	// debugPrintfEXT("Cross n: (%f; %f; %f)", _normal.x, _normal.y, _normal.z);
	// debugPrintfEXT("n: (%f; %f; %f)", normal.x, normal.y, normal.z);

	impact_point += normal * RAY_COLLISION_PRECISION; // Shift the impact point a bit outward to limit the dotty effect

	for (int li = 0; li < POINT_LIGHTS_COUNT; li++) {
		PointLight light = point_lights[li];
		vec4 to_light = (light.pos - impact_point);

		Ray ray_to_light = { // Create a new ray going from the impact point to the point light
			impact_point,
			to_light, // No need to normalize, because we don't need the exact distance if there is a collision
		};

		uint closest_mi;
		uint _tri_index;
		vec2 uv;
		Ray_trace_to_Models(ray_to_light, closest_mi, _tri_index, uv);
		
		if (closest_mi == MODELS_LENGTH) { // Means no collisions detected, thus no shadow, so we need to compute the color for the current light
			float to_light_sq_dist = dot(to_light, to_light);
			
			/*
			vec4 normal = impact_point - closest_s.pos;

			float a = acos(dot(to_light, normal) / (length(to_light) * length(normal)));

			float w = clamp(a / (PI / 2), 0.0, 1.0);

			float diffusion_factor = mix(1.0, 0.0, w);
			*/


			/*
			vec4 normal = vec4(normalize(cross(
                vertices[indices[_tri_index][1]] - vertices[indices[_tri_index][0]], // AB
                vertices[indices[_tri_index][2]] - vertices[indices[_tri_index][0]] // AC
			)), 0.0);
			*/

			float diffusion_factor = clamp(dot(normal, to_light) / length(to_light), 0.0, 1.0);

			float distance_factor = (1 / to_light_sq_dist); // df is the distance factor, light intensity is proportional to the inverse of the distance squared
			final_color += light.col * light.intensity * distance_factor * diffusion_factor;
		}
	}

	for (int li = 0; li < DIR_LIGHTS_COUNT; li++) {
		DirectionalLight light = directional_lights[li];
		vec4 to_light = -light.dir;

		Ray ray_to_light = { // Create a new ray going from the impact point to the point light
			impact_point,
			to_light, // No need to normalize, because we don't need the exact distance if there is a collision
		};

		uint closest_mi;
		uint _tri_index;
		vec2 uv;
		Ray_trace_to_Models(ray_to_light, closest_mi, _tri_index, uv);
		
		if (closest_mi == MODELS_LENGTH) { // Means no collisions detected, thus no shadow, so we need to compute the color for the current light
			float to_light_sq_dist = dot(to_light, to_light);
			
			/*
			vec4 normal = impact_point - closest_s.pos;

			float a = acos(dot(to_light, normal) / (length(to_light) * length(normal)));

			float w = clamp(a / (PI / 2), 0.0, 1.0);

			float diffusion_factor = mix(1.0, 0.0, w);
			*/


			/*
			vec4 normal = vec4(normalize(cross(
                vertices[indices[_tri_index][1]] - vertices[indices[_tri_index][0]], // AB
                vertices[indices[_tri_index][2]] - vertices[indices[_tri_index][0]] // AC
			)), 0.0);
			*/

			float diffusion_factor = clamp(dot(normal, to_light) / length(to_light), 0.0, 1.0);

			final_color += light.col * light.intensity * diffusion_factor;
		}
	}

	return final_color;
}
