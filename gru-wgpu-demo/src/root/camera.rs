use gru_misc::math::*;
use gru_wgpu::ui::event;

pub struct Cam
{
    theta: f32,
    phi: f32,
    dist: f32,
    moving: bool
}

impl Cam
{
    pub fn new() -> Self
    {
        Self
        {
            theta: std::f32::consts::FRAC_PI_2,
            phi: -std::f32::consts::FRAC_PI_2,
            dist: 3.0,
            moving: false
        }
    }

    pub fn input<T>(&mut self, events: &[event::Event<T>])
    {
        for event in events
        {
            match event
            {
                event::Event::Hardware(event::EventPod { event, used: false }) => match event
                {
                    event::HardwareEvent::PointerClicked { pos: _, button: event::MouseButton::Secondary, pressed } => self.moving = *pressed,
                    event::HardwareEvent::PointerMoved { pos: _, delta } => if self.moving
                    {
                        self.theta = (self.theta - 0.01 * delta.1).max(0.1).min(std::f32::consts::PI - 0.1);
                        self.phi -= 0.01 * delta.0;
                    },
                    _ => {}
                },
                _ => {}
            }
        }
    }

    pub fn build(&self, dims: (f32, f32)) -> (Vec3, Mat4)
    {
        let (sin_theta, cos_theta) = self.theta.sin_cos();
        let (sin_phi, cos_phi) = self.phi.sin_cos();
        let c_front = -Vec3(sin_theta * cos_phi, sin_theta * sin_phi, cos_theta);
        let c_up = (Vec3::e_z() - c_front * c_front.2).unit();
        let position = -c_front * self.dist + Vec3(0.0, 0.0, 1.0);
        let view_world = Rotor::from_coords((c_front, c_up), (-Vec3::e_z(), Vec3::e_y())).to_mat4() * Mat4::translation(-position);
        let clip_view = Mat4::perspective_wgpu(dims.0 / dims.1, std::f32::consts::FRAC_PI_4, 0.1, 10.0);
        let clip_world = clip_view * view_world;
        (position, clip_world)
    }
}
