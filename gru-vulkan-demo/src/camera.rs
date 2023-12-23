use crate::marching_cubes::Mold;
use gru_misc::math::*;
use crate::consts;

pub struct Camera
{
    pub proj: Mat4,
    pub theta: f32,
    pub phi: f32,
    pub pos: Vec3,
    pub vel: Vec3,
    pub acc: Vec3,
    pub drag: f32,
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub jump_cooldown: f32,
    pub control_time: f32,
    pub does_physics: bool,
}

impl Camera
{
    pub fn new() -> Self
    {
        Camera
        {
            proj: Mat4::identity(),
            theta: 0.0,
            phi: 0.0,
            pos: Vec3(0.0, 0.0, 0.0),
            vel: Vec3(0.0, 0.0, 0.0),
            acc: Vec3(0.0, 0.0, 0.0),
            drag: consts::AIR_DRAG,
            forward: false,
            backward: false,
            left: false,
            right: false,
            jump_cooldown: 0.0,
            control_time: consts::CONTROL_TIME,
            does_physics: true
        }
    }

    pub fn corners(&self, center: Vec3) -> [Vec3; 12]
    {
        [
            center + Vec3( consts::FIGUR_WIDTH,   consts::FIGUR_HEIGHT,  consts::FIGUR_WIDTH),
            center + Vec3( consts::FIGUR_WIDTH,   consts::FIGUR_HEIGHT, -consts::FIGUR_WIDTH),
            center + Vec3( consts::FIGUR_WIDTH, - consts::FIGUR_HEIGHT,  consts::FIGUR_WIDTH),
            center + Vec3( consts::FIGUR_WIDTH, - consts::FIGUR_HEIGHT, -consts::FIGUR_WIDTH),
            center + Vec3(-consts::FIGUR_WIDTH,   consts::FIGUR_HEIGHT,  consts::FIGUR_WIDTH),
            center + Vec3(-consts::FIGUR_WIDTH,   consts::FIGUR_HEIGHT, -consts::FIGUR_WIDTH),
            center + Vec3(-consts::FIGUR_WIDTH, - consts::FIGUR_HEIGHT,  consts::FIGUR_WIDTH),
            center + Vec3(-consts::FIGUR_WIDTH, - consts::FIGUR_HEIGHT, -consts::FIGUR_WIDTH),
            center + Vec3(-consts::FIGUR_WIDTH,                    0.0,  consts::FIGUR_WIDTH),
            center + Vec3(-consts::FIGUR_WIDTH,                    0.0, -consts::FIGUR_WIDTH),
            center + Vec3( consts::FIGUR_WIDTH,                    0.0,  consts::FIGUR_WIDTH),
            center + Vec3( consts::FIGUR_WIDTH,                    0.0, -consts::FIGUR_WIDTH)
        ]
    }

    pub fn build_projection(&mut self, aspect: f32)
    {
        self.proj = Mat4::perspective_vulkan(aspect, consts::CAM_ANGLE, consts::CAM_NEAR, consts::CAM_FAR);
    }

    pub fn get_acc(&mut self, dt: f32, mold: &impl Mold)
    {
        //acc
        if self.does_physics
        {
            self.acc = (Vec3(0.0, consts::GRAV, 0.0)
            //input
            + self.input() * consts::ACCELERATION * (self.control_time / consts::CONTROL_TIME)//.ceil()
            ).into();
        } else 
        {
            self.acc = (self.input() * consts::ACCELERATION).into();
        }
        self.drag = consts::AIR_DRAG;
        //collision
        if self.does_physics
        {
            let assumed_vel = (self.vel + self.acc * dt) * self.drag.powf(dt);
            let mut collision = false;
            let mut collision_force = Vec3(0.0,0.0,0.0);
            for corner in self.corners(self.pos + assumed_vel * dt).iter()
            {
                let force = (|v: f32| { collision = collision || v > 0.0; v.max(0.0)})(mold.value(*corner)) * consts::COLLISION_FORCE;
                collision_force = collision_force - mold.gradient(*corner).unit() * force;
            }
            if collision
            {
                self.control_time = consts::CONTROL_TIME;
                let vel_norm = self.vel.norm();
                let collision_force_norm = collision_force.norm();
                if vel_norm > 0.0 && collision_force_norm > 0.0
                {
                    let cos = -Vec3::dot(self.vel, collision_force) / vel_norm / collision_force.norm();
                    //self.drag *= consts::COLLISION_DRAG;
                    self.drag *= cos.signum().max(0.0) * cos.powi(2) * consts::COLLISION_DRAG + (1.0 - cos.powi(2)) * consts::SURFACE_DRAG;
                    self.acc = self.acc + collision_force;
                }
            }
        }
    }

    pub fn logic(&mut self, dt: f32, mold: &impl Mold)
    {
        self.get_acc(dt, mold);
        if self.acc.norm() < consts::MIN_ACCELERATION
        {
            self.acc = Vec3(0.0, 0.0, 0.0);
        }
        self.vel = (self.vel + self.acc * dt) * self.drag.powf(dt);
        //pos update
        self.pos = self.pos + self.vel * dt;

        self.jump_cooldown = (self.jump_cooldown - dt).max(0.0);
        self.control_time = (self.control_time - dt).max(0.0);
    }

    pub fn input(&mut self) -> Vec3
    {
        //tasten
        let mut dir = Vec3(0.0, 0.0, 0.0); 
        if self.forward { dir.2 += 1.0; }
        if self.backward { dir.2 -= 1.0; }
        if self.left { dir.0 -= 1.0; }
        if self.right { dir.0 += 1.0; }
        dir = if dir.norm() == 0.0 { Vec3(0.0, 0.0, 0.0) } else { dir.unit() };
        let mut rot = Mat4::rotation_y(self.phi);
        if !self.does_physics
        {
            rot = rot * Mat4::rotation_x(self.theta);
        }
        let input_acc = rot * Vec4(dir.0, dir.1, dir.2, 1.0);
        Vec3(input_acc.0, input_acc.1, input_acc.2)
    }

    pub fn jump(&mut self)
    {
        if self.jump_cooldown == 0.0 && self.control_time > 0.0
        {
            self.vel.1 -= consts::JUMP;
            self.jump_cooldown = consts::JUMP_COOLDOWN;
        }
    }

    pub fn mats(&self) -> (Mat4, Mat4)
    {
        let rot = Mat4::rotation_x(-self.theta) * Mat4::rotation_y(-self.phi);
        let trans = Mat4::translation(Vec3(-self.pos.0, -self.pos.1 + consts::EYE_HEIGHT, -self.pos.2));
        (self.proj, rot * trans)
    }
}
