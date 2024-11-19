use gru_misc::math::*;
use super::render::UniformFragment;
use gru_wgpu::ui::lens::Lens;

#[derive(Clone, Copy, PartialEq)]
#[repr(u32)]
pub enum Present
{
    Full = 0,
    Normal = 1,
    Roughness = 2
}

#[derive(Lens)]
pub struct Light
{
    theta: f32,
    phi: f32,
    rot_speed: f32,
    ambient_color: (f32, f32, f32),
    sun_color: (f32, f32, f32),
    present: Present
}

impl Light
{
    pub fn new() -> Self
    {
        Self
        {
            theta: std::f32::consts::FRAC_PI_2,
            phi: -std::f32::consts::FRAC_PI_2,
            rot_speed: 0.0,
            ambient_color: (0.3, 0.3, 0.3),
            sun_color: (3.0, 3.0, 3.0),
            present: Present::Full
        }
    }

    pub fn build(&mut self, dt: f32, cam_pos: Vec3) -> UniformFragment
    {
        self.phi = (self.phi + self.rot_speed * dt) % std::f32::consts::TAU;
        let (sin_theta, cos_theta) = self.theta.sin_cos();
        let (sin_phi, cos_phi) = self.phi.sin_cos();
        let sun_dir = Vec3(sin_theta * cos_phi, sin_theta * sin_phi, cos_theta);
        UniformFragment
        {
            cam_pos: cam_pos.with_w0(),
            ambient_color: Vec3::from(self.ambient_color).with_w0(),
            sun_dir: sun_dir.with_w0(),
            sun_color: Vec3::from(self.sun_color),
            present: self.present as u32
        }
    }
}
