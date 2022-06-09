
use gru_opengl::{resource::load::{Model, TextureLoadConfig} , gl::{Shader, Texture, TextureChannel, TextureWrap}, impl_ResourceSystem};
impl_ResourceSystem!(CubeResources = 
    (model, Model<CubeVertex>, "cube", ()),
    (shader, Shader<CubeVertex>, "cube", ()),
    (texture, Texture<false>, "cube", TextureLoadConfig { channel: TextureChannel::RGB, mipmap: true, wrap: TextureWrap::Repeat })
);

use gru_opengl::gl::{AttributesReprCpacked, BufferType};
use gru_opengl::resource::load::{BuildFromGltf, VertexData};

#[repr(C, packed)]
pub struct CubeVertex
{
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub tex_coords: [f32; 2],
}

impl AttributesReprCpacked for CubeVertex
{
    const ATTRIBUTES: &'static [(BufferType, &'static str)] =
    &[
        (BufferType::Float { size: 3 }, "position"),
        (BufferType::Float { size: 3 }, "color"),
        (BufferType::Float { size: 2 }, "tex_coords")
    ];
}

impl BuildFromGltf for CubeVertex 
{
    fn build(vd: VertexData) -> Self {
        Self 
        {
            position: vd.position,
            color: vd.color,
            tex_coords: [vd.tex_coord[0] * 0.6, vd.tex_coord[1] * 0.6],  
        }
    }            
}

/*
pub const fn cube() -> ([u16; 36], [CubeVertex; 24])
{
    let indices =
    [
        0, 1, 3, 3, 2, 0, //left
        4, 6, 7, 7, 5, 4, //right
        8, 9, 11, 11, 10, 8, //bottom
        12, 14, 15, 15, 13, 12, //front
        16, 18, 19, 19, 17, 16, //down
        20, 21, 23, 23, 22, 20 //up
    ];
    let vertices =
    [
        //left
        CubeVertex { position: [-1.0, -1.0, -1.0], color: [0.0, 0.0, 0.0], tex_coords: [0.0, 0.0] }, //0 ldb
        CubeVertex { position: [-1.0, -1.0, 1.0], color: [0.0, 0.0, 1.0], tex_coords: [1.0, 0.0] }, //1 ldf
        CubeVertex { position: [-1.0, 1.0, -1.0], color: [0.0, 1.0, 0.0], tex_coords: [0.0, 1.0] }, //2 lub
        CubeVertex { position: [-1.0, 1.0, 1.0], color: [0.0, 1.0, 1.0], tex_coords: [1.0, 1.0] }, //3 luf
        //right
        CubeVertex { position: [1.0, -1.0, -1.0], color: [1.0, 0.0, 0.0], tex_coords: [0.0, 0.0] }, //4 rdb
        CubeVertex { position: [1.0, -1.0, 1.0], color: [1.0, 0.0, 1.0], tex_coords: [0.0, 1.0] }, //5 rdf
        CubeVertex { position: [1.0, 1.0, -1.0], color: [1.0, 1.0, 0.0], tex_coords: [1.0, 0.0] }, //6 rub
        CubeVertex { position: [1.0, 1.0, 1.0], color: [1.0, 1.0, 1.0], tex_coords: [1.0, 1.0] },  //7 ruf
        //bottom
        CubeVertex { position: [-1.0, -1.0, -1.0], color: [0.0, 0.0, 0.0], tex_coords: [0.0, 0.0] }, //8 ldb
        CubeVertex { position: [-1.0, 1.0, -1.0], color: [0.0, 1.0, 0.0], tex_coords: [1.0, 0.0] }, //9 lub
        CubeVertex { position: [1.0, -1.0, -1.0], color: [1.0, 0.0, 0.0], tex_coords: [0.0, 1.0] }, //10 rdb
        CubeVertex { position: [1.0, 1.0, -1.0], color: [1.0, 1.0, 0.0], tex_coords: [1.0, 1.0] }, //11 rub
        //front
        CubeVertex { position: [-1.0, -1.0, 1.0], color: [0.0, 0.0, 1.0], tex_coords: [0.0, 0.0] }, //12 ldf
        CubeVertex { position: [-1.0, 1.0, 1.0], color: [0.0, 1.0, 1.0], tex_coords: [0.0, 1.0] }, //13 luf
        CubeVertex { position: [1.0, -1.0, 1.0], color: [1.0, 0.0, 1.0], tex_coords: [1.0, 0.0] }, //14 rdf
        CubeVertex { position: [1.0, 1.0, 1.0], color: [1.0, 1.0, 1.0], tex_coords: [1.0, 1.0] } , //15 ruf
        //down
        CubeVertex { position: [-1.0, -1.0, -1.0], color: [0.0, 0.0, 0.0], tex_coords: [0.0, 0.0] }, //16 ldb
        CubeVertex { position: [-1.0, -1.0, 1.0], color: [0.0, 0.0, 1.0], tex_coords: [0.0, 1.0] }, //17 ldf
        CubeVertex { position: [1.0, -1.0, -1.0], color: [1.0, 0.0, 0.0], tex_coords: [1.0, 0.0] }, //18 rdb
        CubeVertex { position: [1.0, -1.0, 1.0], color: [1.0, 0.0, 1.0], tex_coords: [1.0, 1.0] }, //19 rdf
        //up
        CubeVertex { position: [-1.0, 1.0, -1.0], color: [0.0, 1.0, 0.0], tex_coords: [0.0, 0.0] }, //20 lub
        CubeVertex { position: [-1.0, 1.0, 1.0], color: [0.0, 1.0, 1.0], tex_coords: [1.0, 0.0] }, //21 luf
        CubeVertex { position: [1.0, 1.0, -1.0], color: [1.0, 1.0, 0.0], tex_coords: [0.0, 1.0] }, //22 rub
        CubeVertex { position: [1.0, 1.0, 1.0], color: [1.0, 1.0, 1.0], tex_coords: [1.0, 1.0] }  //23 ruf
    ];
    (indices, vertices)
}*/