// This files provides function used to rotate vectors with quaternions

// Returns a quaternion which rotates vectors around the given axis by the given angle
// Assumes normalized axis vector otherwise the returned quaternion is not unit
vec4 from_axis(vec3 axis, float angle) {
	angle /= 2.0;
	return vec4(axis * sin(angle), cos(angle));
}

vec4 multiply(vec4 quat, vec4 other) {
	return vec4(
		quat.w * other.xyz + other.w * quat.xyz + cross(quat.xyz, other.xyz),
		quat.w * other.w - dot(quat.xyz, other.xyz)
	);
}

// Returns the given point rotated by the quaternion
vec3 transform_point(vec4 quat, vec3 point) {
	vec4 inv = vec4(-quat.xyz, quat.w);
	return multiply(multiply(quat, vec4(point, 0.0)), inv).xyz;
}