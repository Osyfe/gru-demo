use gru_opengl::{resource::load::Model, gl::{Shader, Texture}, impl_ResourceSystem};
use super::cube::CubeVertex; //, sound::SoundData};

impl_ResourceSystem!(Resources = 
    (cube_model, Model<CubeVertex>, "cube"),
    (cube_shader, Shader<CubeVertex>, "cube"),
    (cube_texture, Texture<false>, "cube")
);