use gru_opengl::{resource::{Res, load::*, ResourceSystem, get_res_iter_mut}, gl::{Shader, Texture}};

use super::{cube::CubeVertex}; //, sound::SoundData};

pub struct Resources {
    pub cube_model: Res<Model<CubeVertex>>,
    pub cube_shader: Res<Shader<CubeVertex>>,
    pub cube_texture: Res<Texture<false>>,
    //weh: Res<SoundData>,
    //eh: Res<SoundData>,
}

impl ResourceSystem for Resources {
    fn empty() -> Self {
        Self
        {
            cube_model: Res::new("cube"),
            cube_shader: Res::new("cube"),
            cube_texture: Res::new("cube"),
            //weh: Res::new("weh"),
            //eh: Res::new("eh"),
        }
        
    }

    fn get_iter_mut(&mut self) -> gru_opengl::resource::ResIterMut {
        get_res_iter_mut([&mut self.cube_model, &mut self.cube_shader, &mut self.cube_texture])
    }
}