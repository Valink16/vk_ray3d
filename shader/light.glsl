// Returns the total quantity of light on the closest_si using point_lights
// # Arguments
// 	- closest_d: distance from the ray origin to the impact point
// 	- closest_si: index of the sphere with which the ray collided
// 	- ri: index of the current ray
vec3 PointLights_to_Sphere(vec4 impact_point, Sphere closest_s, Ray r) {
	vec3 final_color = vec3(0.0);

	for (int i = 0; i < LIGHTS_COUNT; i++) {
		PointLight light = point_lights[i];
		vec4 to_light = (light.pos - impact_point);

		Ray ray_to_light = { // Create a new ray going from the impact point to the point light
			impact_point,
			to_light, // No need to normalize, because we don't need the exact distance if there is a collision
		};

		uint closest_si;
		Ray_trace_to_Spheres(ray_to_light, closest_si);

		if (closest_si == SPHERES_LENGTH) { // Means no collisions detected, thus no shadow, so we need to compute the color for the current light
			float to_light_sq_dist = dot(to_light, to_light);
			
			/*
			vec4 normal = impact_point - closest_s.pos;

			float a = acos(dot(to_light, normal) / (length(to_light) * length(normal)));

			float w = clamp(a / (PI / 2), 0.0, 1.0);

			float diffusion_factor = mix(1.0, 0.0, w);
			*/

			vec4 normal = normalize(impact_point - closest_s.pos);
			float diffusion_factor = clamp(dot(normal, to_light) / length(to_light), 0.2, 1.0);

			float distance_factor = (1 / to_light_sq_dist); // df is the distance factor, light intensity is proportional to the inverse of the distance squared
			final_color += light.col * light.intensity * distance_factor * diffusion_factor;
		}
	}

	return final_color;
}