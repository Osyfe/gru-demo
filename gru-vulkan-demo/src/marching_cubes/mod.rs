use gru_misc::math::*;

mod table;

pub trait Mold
{
	fn value(&self, pos: Vec3) -> f32;
	fn gradient(&self, pos: Vec3) -> Vec3;
	fn color(&self, pos: Vec3) -> Vec3;

	fn new_vertex(&self, pos: Vec3) -> Vertex
	{
		let normal = self.gradient(pos).unit();
		let phi = f32::atan2(normal.1, normal.0);
		let cos_theta = normal.2;
		Vertex
		{
			position: pos,
			normal: normal * (-1.0),
			coords: (phi * std::f32::consts::FRAC_1_PI, cos_theta)
		}
	}
}

pub struct Vertex
{
	pub position: Vec3,
	pub normal: Vec3,
	pub coords: (f32, f32)
}

pub fn mesh_builder<T: Mold>(offset: Vec3, radii: Vec3, resolutions: (u32, u32, u32), mold: &T) -> (Vec<Vertex>, Vec<u32>)
{
	use std::collections::HashMap;
	let mut done_vertices = Vec::new();
	done_vertices.resize(((resolutions.0 + 1) * (resolutions.1 + 1)) as usize, f32::NAN);
	let mut doing_vertices = Vec::new();
	doing_vertices.resize(((resolutions.0 + 1) * (resolutions.1 + 1)) as usize, f32::NAN);
	let mut done_xy_edges = HashMap::new();
	let mut doing_xy_edges = HashMap::new();
	let mut doing_z_edges = HashMap::new();

	let mut vertices = vec![];
	let mut indices = vec![];

	let step_sizes = (2.0 * radii.0 / resolutions.0 as f32, 2.0 * radii.1 / resolutions.1 as f32, 2.0 * radii.2 / resolutions.2 as f32);
	for zi in 0..resolutions.2
	{
		let cur_z = zi as f32 * step_sizes.2 - radii.2 + offset.2;
		for yi in 0..resolutions.1
		{
			let cur_y = yi as f32 * step_sizes.1 - radii.1 + offset.1;
			for xi in 0..resolutions.0
			{
				let cur_x = xi as f32 * step_sizes.0 - radii.0 + offset.0;

				let corners =
				[
					Vec3(cur_x, cur_y + step_sizes.1, cur_z + step_sizes.2),
					Vec3(cur_x + step_sizes.0, cur_y + step_sizes.1, cur_z + step_sizes.2),
					Vec3(cur_x + step_sizes.0, cur_y + step_sizes.1, cur_z),
					Vec3(cur_x, cur_y + step_sizes.1, cur_z),
					Vec3(cur_x, cur_y, cur_z + step_sizes.2),
					Vec3(cur_x + step_sizes.0, cur_y, cur_z + step_sizes.2),
					Vec3(cur_x + step_sizes.0, cur_y, cur_z),
					Vec3(cur_x, cur_y, cur_z)
				];

				let corner_values =
				[
					if xi != 0 { doing_vertices[(xi * (resolutions.0 + 1) + yi + 1) as usize] }
					else
					{
						let value = mold.value(corners[0]);
						doing_vertices[(xi * (resolutions.0 + 1) + yi + 1) as usize] = value;
						value
					},
					if false { doing_vertices[((xi + 1) * (resolutions.0 + 1) + yi + 1) as usize] }
					else
					{
						let value = mold.value(corners[1]);
						doing_vertices[((xi + 1) * (resolutions.0 + 1) + yi + 1) as usize] = value;
						value
					},
					if zi != 0 { done_vertices[((xi + 1) * (resolutions.0 + 1) + yi + 1) as usize] }
					else
					{
						let value = mold.value(corners[2]);
						done_vertices[((xi + 1) * (resolutions.0 + 1) + yi + 1) as usize] = value;
						value
					},
					if xi != 0 || zi != 0 { done_vertices[(xi * (resolutions.0 + 1) + yi + 1) as usize] }
					else
					{
						let value = mold.value(corners[3]);
						done_vertices[(xi * (resolutions.0 + 1) + yi + 1) as usize] = value;
						value
					},
					if xi != 0 || yi != 0 { doing_vertices[(xi * (resolutions.0 + 1) + yi) as usize] }
					else
					{
						let value = mold.value(corners[4]);
						doing_vertices[(xi * (resolutions.0 + 1) + yi) as usize] = value;
						value
					},
					if yi != 0 { doing_vertices[((xi + 1) * (resolutions.0 + 1) + yi) as usize] }
					else
					{
						let value = mold.value(corners[5]);
						doing_vertices[((xi + 1) * (resolutions.0 + 1) + yi) as usize] = value;
						value
					},
					if yi != 0 || zi != 0 { done_vertices[((xi + 1) * (resolutions.0 + 1) + yi) as usize] }
					else
					{
						let value = mold.value(corners[6]);
						done_vertices[((xi + 1) * (resolutions.0 + 1) + yi) as usize] = value;
						value
					},
					if xi != 0 || yi != 0 || zi != 0 { done_vertices[(xi * (resolutions.0 + 1) + yi) as usize] }
					else
					{
						let value = mold.value(corners[7]);
						done_vertices[(xi * (resolutions.0 + 1) + yi) as usize] = value;
						value
					},
				];

				let table_code =
					((if corner_values[0] >= 0.0 { 1 } else { 0 }) << 0)
				  | ((if corner_values[1] >= 0.0 { 1 } else { 0 }) << 1)
				  | ((if corner_values[2] >= 0.0 { 1 } else { 0 }) << 2)
				  | ((if corner_values[3] >= 0.0 { 1 } else { 0 }) << 3)
				  | ((if corner_values[4] >= 0.0 { 1 } else { 0 }) << 4)
				  | ((if corner_values[5] >= 0.0 { 1 } else { 0 }) << 5)
				  | ((if corner_values[6] >= 0.0 { 1 } else { 0 }) << 6)
				  | ((if corner_values[7] >= 0.0 { 1 } else { 0 }) << 7);

				for &edge in table::TRIANGULATION[table_code].iter()
				{
					if edge == -1
					{
						break;
					} else
					{
						let map = match edge
						{
							1 | 3 | 5 | 7 => &mut doing_z_edges,
							2 | 6 | 10 | 11 => &mut done_xy_edges,
							_ => &mut doing_xy_edges
						};
						let key = match edge
						{
							7 => (xi, yi),
							5 => (xi + 1, yi),
							3 => (xi, yi + 1),
							1 => (xi + 1, yi + 1),
							11 | 8 => (2 * xi, yi),
							10 | 9 => (2 * (xi + 1), yi),
							6 | 4 => (2 * xi + 1, yi),
							2 | 0 => (2 * xi + 1, yi + 1),
							_ => unreachable!()
						};
						//let certainly_new = edge == 0 || edge == 1 || edge == 9;
						let index = map.entry(key).or_insert_with(||
						{
							let corner_index_a = table::CORNER_INDEX_A_FROM_EDGE[edge as usize];
							let corner_index_b = table::CORNER_INDEX_B_FROM_EDGE[edge as usize];
							let corner_a = corners[corner_index_a];
							let corner_b = corners[corner_index_b];
							let corner_value_a = corner_values[corner_index_a];
							let corner_value_b = corner_values[corner_index_b];
							
							let factor_b = corner_value_a / (corner_value_a - corner_value_b);
							let factor_a = 1.0 - factor_b;
							let vertex = mold.new_vertex(corner_a * factor_a + corner_b * factor_b);
							let index = vertices.len();
							vertices.push(vertex);
							index
						});
						indices.push(*index as u32);
					}
				}
			}
		}
		std::mem::swap(&mut doing_vertices, &mut done_vertices);
		std::mem::swap(&mut doing_xy_edges, &mut done_xy_edges);
		doing_xy_edges.clear();
		doing_z_edges.clear();
	}
	(vertices, indices)
}
