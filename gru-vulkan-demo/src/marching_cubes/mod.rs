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

// Works in z slices: done ~ current z, doing ~ next z
pub fn mesh_builder<T: Mold>(offset: Vec3, radii: Vec3, resolutions: (u32, u32, u32), mold: &T) -> (Vec<Vertex>, Vec<u32>)
{
    use ahash::AHashMap as HashMap;
    use std::time::Instant;

    let before = Instant::now();

    let corner_counts = (resolutions.0 + 1, resolutions.1 + 1);
    let step_sizes = (2.0 * radii.0 / resolutions.0 as f32, 2.0 * radii.1 / resolutions.1 as f32, 2.0 * radii.2 / resolutions.2 as f32);
    let origin = offset - radii;

    let mut done_vertices = vec![f32::NAN; (corner_counts.0 * corner_counts.1) as usize];
    let mut doing_vertices = vec![f32::NAN; (corner_counts.0 * corner_counts.1) as usize];
    let mut done_xy_edges = HashMap::new();
    let mut doing_xy_edges = HashMap::new();
    let mut doing_z_edges = HashMap::new();

    let mut vertices = vec![];
    let mut indices = vec![];

    fn compute_corners<T: Mold>(vertices: &mut [f32], cur_z: f32, origin: Vec3, step_sizes: (f32, f32, f32), corner_counts: (u32, u32), mold: &T)
    {
        let cur_z = cur_z + origin.2;
        for yi in 0..corner_counts.1
        {
            let cur_y = yi as f32 * step_sizes.1 + origin.1;
            let vertices = &mut vertices[(yi  * corner_counts.1) as usize..];
            for xi in 0..corner_counts.0
            {
                let cur_x = xi as f32 * step_sizes.0 + origin.0;
                let value = mold.value(Vec3(cur_x, cur_y, cur_z));
                vertices[xi as usize] = value;
            }
        }
    }
    compute_corners(&mut done_vertices, 0.0, origin, step_sizes, corner_counts, mold);
    
    for zi in 0..resolutions.2
    {
        compute_corners(&mut doing_vertices, (zi + 1) as f32 * step_sizes.2, origin, step_sizes, corner_counts, mold);

        let cur_z = zi as f32 * step_sizes.2 + origin.2;
        let next_z = cur_z + step_sizes.2;
        for yi in 0..resolutions.1
        {	
            let cur_y = yi as f32 * step_sizes.1 + origin.1;
            let next_y = cur_y + step_sizes.1;
            for xi in 0..resolutions.0
            {
                let cur_x = xi as f32 * step_sizes.0 + origin.0;
                let next_x = cur_x + step_sizes.0;

                let corners =
                [
                    Vec3(cur_x, next_y, next_z),
                    Vec3(next_x, next_y, next_z),
                    Vec3(next_x, next_y, cur_z),
                    Vec3(cur_x, next_y, cur_z),
                    Vec3(cur_x, cur_y, next_z),
                    Vec3(next_x, cur_y, next_z),
                    Vec3(next_x, cur_y, cur_z),
                    Vec3(cur_x, cur_y, cur_z)
                ];

                let corner_values =
                [
                    doing_vertices[((yi + 1) * corner_counts.1 + xi) as usize],
                    doing_vertices[((yi + 1) * corner_counts.1 + xi + 1) as usize],
                    done_vertices[((yi + 1) * corner_counts.1 + xi + 1) as usize],
                    done_vertices[((yi + 1) * corner_counts.1 + xi) as usize],
                    doing_vertices[(yi * corner_counts.1 + xi) as usize],
                    doing_vertices[(yi * corner_counts.1 + xi + 1) as usize],
                    done_vertices[(yi * corner_counts.1 + xi + 1) as usize],
                    done_vertices[(yi * corner_counts.1 + xi) as usize]
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
                    }
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
        std::mem::swap(&mut doing_vertices, &mut done_vertices);
        std::mem::swap(&mut doing_xy_edges, &mut done_xy_edges);
        doing_xy_edges.clear();
        doing_z_edges.clear();
    }
    let after = std::time::Instant::now();
    let _duration = after - before;
    //println!("time: {} ms, vertices: {}", duration.as_millis(), vertices.len());
    (vertices, indices)
}
