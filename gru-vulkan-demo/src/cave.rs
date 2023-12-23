use super::*;
use super::Vertex;
use super::marching_cubes::Mold;
use rand::distributions::{Distribution, Uniform};

pub struct CylinderBlock
{
	pub buffer: Buffer,
	pub vertex_view: BufferView<Vertex>,
	pub index_view: BufferView<u32>,
	pub z: i32,
    pub flashes: Vec<(Vec3, Vec3)>
}

impl CylinderBlock
{
    pub fn new(device: &Device, command_pool: &CommandPool, queue: &Arc<Mutex<Queue>>, mold: &impl Mold, z: i32) -> Self
    {
        let (vert, indices) = marching_cubes::mesh_builder
        (
            Vec3(0.0, 0.0, z as f32 * consts::BLOCK_LENGTH),
            Vec3(consts::CAVE_RADIUS * 2.0, consts::CAVE_RADIUS * 2.0, consts::BLOCK_LENGTH / 2.0),
            (consts::CAVE_RESOLUTION, consts::CAVE_RESOLUTION, (consts::CAVE_RESOLUTION as f32 / consts::CAVE_RADIUS * consts::BLOCK_LENGTH / 4.0) as u32),
            mold
        );
        let vertices: Vec<_> = vert.iter().map(|v| Vertex { position: v.position.into(), normal: v.normal.into(), tex_coords: v.coords.into() }).collect();
        let mut vertex_buffer_layout = device.new_buffer_type();
        let vertex_view = vertex_buffer_layout.add_attributes(vertices.len() as u32);
        let index_view = vertex_buffer_layout.add_indices(indices.len() as u32);
        let vertex_buffer_layout = vertex_buffer_layout.build();
        let buffer =
        {
            let mut vertex_buffer_temp = device.new_buffer(&vertex_buffer_layout, BufferUsage::Stage);
            {
               let mut buffer_map = vertex_buffer_temp.map();
               buffer_map.write_attributes(&vertex_view, 0, &vertices);
               buffer_map.write_indices(&index_view, 0, &indices);
            }
            let vertex_buffer = device.new_buffer(&vertex_buffer_layout, BufferUsage::Static);
            command_pool.new_command_buffer().copy_buffer(&queue.lock().unwrap(), &vertex_buffer_temp, &vertex_buffer, device.new_fence(false)).mark.wait();
            vertex_buffer
        };
        //Generate Flashes
        let mut flashes = Vec::with_capacity(1);
        let range = Uniform::from(-consts::CAVE_RADIUS..consts::CAVE_RADIUS);
        let mut rng = rand::thread_rng();
        if range.sample(&mut rng) / consts::CAVE_RADIUS / 2.0 + 0.5 < consts::FLASH_BLOCK_PROB
        {
            let mut pos = Vec3(range.sample(&mut rng), range.sample(&mut rng), (z as f32 + range.sample(&mut rng) / consts::CAVE_RADIUS / 2.0) * consts::BLOCK_LENGTH);
            while mold.value(pos) < 0.0 &&  mold.value(pos) < -0.1
            {
                pos.1 += 0.001;
            }
            if mold.value(pos) < 0.0 { flashes.push((pos, flash::FlashMold().color(pos))); }
        };
        Self { buffer, vertex_view, index_view, z, flashes }
    }
}

pub struct BlockGenerator
{
    t_request: mpsc::Sender<i32>,
    r_block: mpsc::Receiver<CylinderBlock>
}

impl BlockGenerator
{
    pub fn new(device: &Device, queue_family_info: &QueueFamilyInfo, mold: impl Mold + std::marker::Send + 'static) -> Self
    {
        let (t_request, r_request) = std::sync::mpsc::channel();
        let (t_block, r_block) = std::sync::mpsc::channel();
        let device = device.clone();
        let queue_family_info = queue_family_info.clone();
        std::thread::spawn(move ||
        {
            let graphic_queue_family = device.get_queue_family(&queue_family_info);
            let command_pool = device.new_command_pool(graphic_queue_family);
            let queue = graphic_queue_family.get_queue(0);

            for request in r_request.iter()
            {
                let block = CylinderBlock::new(&device, &command_pool, &queue, &mold, request);
                t_block.send(block).ok();
            }
        });
        Self { t_request, r_block }
    }

    pub fn request(&self, z: i32) { self.t_request.send(z).ok(); }
    pub fn receive(&self) -> mpsc::TryIter<CylinderBlock> { self.r_block.try_iter() }

    pub fn shutdown(self)
    {
        std::mem::drop(self.t_request);
        for _ in self.r_block.into_iter() {}
    }
}

pub struct Cave<T: noise::NoiseFn<[f64; 3]>>
{
    pub fun: T,
    pub perlin: noise::Perlin,
    pub bias: f32,
    x0: f32,
    y0: f32
}

impl<T: noise::NoiseFn<[f64; 3]>> Cave<T>
{
    pub fn new(fun: T, perlin: noise::Perlin, bias: f32) -> Self
    {
        let (x0, y0) = (consts::CAVE_RADIUS * perlin.get([0.0, 0.0]) as f32, consts::CAVE_RADIUS * perlin.get([0.0, 0.0]) as f32);
        Self { fun, perlin, bias, x0, y0 }
    }

    pub fn x0(&self) -> f32 { self.x0 }
    pub fn y0(&self) -> f32 { self.y0 }
}

impl<T: noise::NoiseFn<[f64; 3]>> Mold for Cave<T>
{
    fn value(&self, Vec3(x, y, z): Vec3) -> f32
    {
        self.fun.get([x as f64, y as f64, z as f64]) as f32
      + ((x - consts::CAVE_RADIUS * self.perlin.get([z as f64 * 0.01, 0.0]) as f32).abs().max((y - consts::CAVE_RADIUS * self.perlin.get([0.0, z as f64 * 0.01]) as f32).abs()).max(-z) / consts::CAVE_RADIUS).powi(consts::CAVE_GEN_BORDER_POWER) * consts::CAVE_GEN_BORDER_STRENGTH
      - consts::CAVE_GEN_SPAWN_STRENGTH * (consts::CAVE_GEN_SPAWN_DECAY_RATE * ((x - self.x0)*(x - self.x0) + (y - self.y0)*(y - self.y0) + z*z)).exp()
      + self.bias
    }

    fn gradient(&self, Vec3(x, y, z): Vec3) -> Vec3
    {
        Vec3
        (
            (self.value(Vec3(x + consts::CAVE_GEN_GRADIENT_EPSILON, y, z)) - self.value(Vec3(x - consts::CAVE_GEN_GRADIENT_EPSILON, y, z))) / consts::CAVE_GEN_GRADIENT_EPSILON_2,
            (self.value(Vec3(x, y + consts::CAVE_GEN_GRADIENT_EPSILON, z)) - self.value(Vec3(x, y - consts::CAVE_GEN_GRADIENT_EPSILON, z))) / consts::CAVE_GEN_GRADIENT_EPSILON_2,
            (self.value(Vec3(x, y, z + consts::CAVE_GEN_GRADIENT_EPSILON)) - self.value(Vec3(x, y, z - consts::CAVE_GEN_GRADIENT_EPSILON))) / consts::CAVE_GEN_GRADIENT_EPSILON_2
        )
    }

    fn color(&self, pos: Vec3) -> Vec3
    {
        Vec3(0.6, 0.5, 0.2) * (self.fun.get([2.0 * pos.0 as f64, 2.0 * pos.1 as f64, 2.0 * pos.2 as f64]) as f32 + 1.0)
    }
}
