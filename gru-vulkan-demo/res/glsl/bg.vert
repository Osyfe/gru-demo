#version 450 core

#define RADIUS 30.0
#define BACK_DISTANCE 120.0
#define FRONT_DISTANCE 330.0

#include "cam.glsl"
#include "light.glsl"

layout (location=0) out float z;

vec3 luf() { return vec3(-RADIUS, -RADIUS, light.pos.z - BACK_DISTANCE); }
vec3 lub() { return vec3(-RADIUS, -RADIUS, light.pos.z + FRONT_DISTANCE); }
vec3 ldf() { return vec3(-RADIUS, +RADIUS, light.pos.z - BACK_DISTANCE); }
vec3 ldb() { return vec3(-RADIUS, +RADIUS, light.pos.z + FRONT_DISTANCE); }
vec3 ruf() { return vec3(+RADIUS, -RADIUS, light.pos.z - BACK_DISTANCE); }
vec3 rub() { return vec3(+RADIUS, -RADIUS, light.pos.z + FRONT_DISTANCE); }
vec3 rdf() { return vec3(+RADIUS, +RADIUS, light.pos.z - BACK_DISTANCE); }
vec3 rdb() { return vec3(+RADIUS, +RADIUS, light.pos.z + FRONT_DISTANCE); }

void main()
{
	vec3 position;
	switch(gl_VertexIndex)
	{
		//left
		case  0: position = luf(); break;
		case  1: position = ldf(); break;
		case  2: position = ldb(); break;
		case  3: position = ldb(); break;
		case  4: position = lub(); break;
		case  5: position = luf(); break;
		//right
		case  6: position = rub(); break;
		case  7: position = rdb(); break;
		case  8: position = rdf(); break;
		case  9: position = rdf(); break;
		case 10: position = ruf(); break;
		case 11: position = rub(); break;
		//up
		case 12: position = luf(); break;
		case 13: position = lub(); break;
		case 14: position = rub(); break;
		case 15: position = rub(); break;
		case 16: position = ruf(); break;
		case 17: position = luf(); break;
		//down
		case 18: position = ldb(); break;
		case 19: position = ldf(); break;
		case 20: position = rdf(); break;
		case 21: position = rdf(); break;
		case 22: position = rdb(); break;
		case 23: position = ldb(); break;
		//front
		case 24: position = ruf(); break;
		case 25: position = rdf(); break;
		case 26: position = ldf(); break;
		case 27: position = ldf(); break;
		case 28: position = luf(); break;
		case 29: position = ruf(); break;
		//back
		case 30: position = lub(); break;
		case 31: position = ldb(); break;
		case 32: position = rdb(); break;
		case 33: position = rdb(); break;
		case 34: position = rub(); break;
		case 35: position = lub(); break;
	}
	z = position.z;
	gl_Position = cam.proj * vec4(position, 1.0);
}