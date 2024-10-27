use super::*;
use crate::Vec3;
use rand::distributions::{Distribution, Uniform};

#[derive(VertexAttributeGroupReprCpacked)]
#[repr(C, packed)]
pub struct FlashVertex
{
    #[location = 0]
    pub pos: F3
}

#[derive(InstanceAttributeGroupReprCpacked)]
#[repr(C, packed)]
pub struct FlashInstance
{
    #[location = 1]
    pub offset: F3,
    #[location = 2]
    pub color: F3
}

pub struct FlashMold;

impl mold::Mold for FlashMold
{
    fn value(&self, pos: Vec3) -> f32
    {
        let rad = consts::FLASH_RADIUS_SQ - pos.2 * pos.2 - pos.1 * pos.1;
        let hei = consts::FLASH_HEIGHT - pos.0.abs();
        if rad < 0.0 && hei < 0.0 { rad.max(hei) }
        else if rad < 0.0 && hei > 0.0 { rad }
        else if rad > 0.0 && hei < 0.0 { hei }
        else { rad.min(hei) }
    }

    fn gradient(&self, _: Vec3) -> Vec3 { Vec3(0.8, 0.1, 0.1) }
    
    fn color(&self, _: Vec3) -> Vec3 
    {
        let range = Uniform::from(0.0..1.0);
        let mut rng = rand::thread_rng();
        Vec3(range.sample(&mut rng), range.sample(&mut rng), range.sample(&mut rng)).unit()
    }
}
