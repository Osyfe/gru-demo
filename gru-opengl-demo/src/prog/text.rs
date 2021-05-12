use gru_opengl::gl::{AttributesReprCpacked, BufferType};

pub const ATLAS_SIZE: u32 = 1024;

pub const TEXTS: [&'static str; 2] =
[
    "Hello Open GL",
    "Wuu huuu"
];

#[derive(PartialEq)]
pub enum Text
{
    None,
    Hello,
    Wuhu
}

impl Text
{
    pub fn get(&self) -> (&'static str, f32)
    {
        match self
        {
            Self::None => unreachable!(),
            Self::Hello => (TEXTS[0], 3.0),
            Self::Wuhu => (TEXTS[1], 2.0)
        }
    }
}

#[repr(C, packed)]
pub struct GlyphVertex
{
    pub position: [f32; 2],
    pub tex_coords: [f32; 2]
}

impl AttributesReprCpacked for GlyphVertex
{
    const ATTRIBUTES: &'static [(BufferType, &'static str)] =
    &[
        (BufferType::Float { size: 2 }, "position"),
        (BufferType::Float { size: 2 }, "tex_coords")
    ];
}
