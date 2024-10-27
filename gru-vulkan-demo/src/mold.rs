use gru_misc::math::Vec3;

pub struct Vertex
{
    pub position: Vec3,
    pub normal: Vec3,
    pub coords: (f32, f32)
}

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
