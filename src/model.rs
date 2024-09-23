#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
}

fn create_vertex(position: [f32; 3], normal: [f32; 3]) -> Vertex {
    Vertex { position, normal }
}

impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

pub struct Model {
    pub vertices: Vec<Vertex>,
    pub vertex_indices: Vec<u16>,
}

pub fn cube() -> Model {
    /*
       7--------6
      / |      /|
     /  |     / |
    2--------3  |
    |   5----|--4    y  z
    |  /     | /     | /
    | /      |/      |/
    0--------1       o----> x

     */
    // Vertices
    let v0 = [-1.0, -1.0, -1.0];
    let v1 = [1.0, -1.0, -1.0];
    let v2 = [-1.0, 1.0, -1.0];
    let v3 = [1.0, 1.0, -1.0];
    let v4 = [1.0, -1.0, 1.0];
    let v5 = [-1.0, -1.0, 1.0];
    let v6 = [1.0, 1.0, 1.0];
    let v7 = [-1.0, 1.0, 1.0];

    // Normals
    let front = [0.0, 0.0, -1.0];
    let back = [0.0, 0.0, 1.0];
    let left = [0.0, -1.0, 0.0];
    let right = [0.0, 1.0, 0.0];
    let top = [-1.0, 0.0, 0.0];
    let bottom = [1.0, 0.0, 0.0];

    Model {
        vertices: vec![
            // Front
            create_vertex(v0, front), // 0
            create_vertex(v1, front), // 1
            create_vertex(v2, front), // 2
            create_vertex(v3, front), // 3
            // Right
            create_vertex(v1, right), // 4
            create_vertex(v4, right), // 5
            create_vertex(v3, right), // 6
            create_vertex(v6, right), // 7
            // Back
            create_vertex(v4, back), // 8
            create_vertex(v5, back), // 9
            create_vertex(v6, back), // 10
            create_vertex(v7, back), // 11
            // Left
            create_vertex(v5, left), // 12
            create_vertex(v0, left), // 13
            create_vertex(v7, left), // 14
            create_vertex(v2, left), // 15
            // Top
            create_vertex(v2, top), // 16
            create_vertex(v3, top), // 17
            create_vertex(v7, top), // 18
            create_vertex(v6, top), // 19
            // Bottom
            create_vertex(v5, bottom), // 20
            create_vertex(v4, bottom), // 21
            create_vertex(v0, bottom), // 22
            create_vertex(v1, bottom), // 23
        ],
        vertex_indices: vec![
            0, 2, 1, 2, 3, 1, // Front
            4, 6, 5, 6, 7, 5, // Right
            8, 10, 9, 10, 11, 9, // Back
            12, 14, 13, 14, 15, 13, // Left
            16, 18, 17, 18, 19, 17, // Top
            20, 22, 21, 22, 23, 21, // Bottom
        ],
    }
}
