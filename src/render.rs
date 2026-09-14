use crate::{assets::AssetCatalog, background::TileMap, game::World};
use std::borrow::Cow;
use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::{dpi::PhysicalSize, window::Window};

pub trait PlatformWindow {}
pub trait Renderer {
    fn render(&mut self, world: &World);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RenderCommand {
    SolidRect {
        layer: u8,
        x: i16,
        y: i16,
        width: u16,
        height: u16,
    },
    GeometryRect {
        layer: u8,
        x: i16,
        y: i16,
        width: u16,
        height: u16,
    },
    Texture {
        layer: u8,
        texture_id: u16,
        x: i16,
        y: i16,
        width: u16,
        height: u16,
    },
}

impl RenderCommand {
    fn layer(self) -> u8 {
        match self {
            Self::SolidRect { layer, .. }
            | Self::GeometryRect { layer, .. }
            | Self::Texture { layer, .. } => layer,
        }
    }
}

pub fn sort_render_commands(commands: &mut [RenderCommand]) {
    commands.sort_by_key(|command| match command {
        RenderCommand::SolidRect { layer, .. }
        | RenderCommand::GeometryRect { layer, .. }
        | RenderCommand::Texture { layer, .. } => *layer,
    });
}

pub struct GpuRenderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    texture_pipeline: wgpu::RenderPipeline,
    texture_bind_group: wgpu::BindGroup,
    texture_vertex_buffer: wgpu::Buffer,
    background_bind_group: wgpu::BindGroup,
    background_vertex_buffer: wgpu::Buffer,
    background_map: TileMap,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 3],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct TextureVertex {
    position: [f32; 2],
    uv: [f32; 2],
}

const LOGICAL_WIDTH: f32 = 640.0;
const LOGICAL_HEIGHT: f32 = 320.0;

impl GpuRenderer {
    pub async fn new(window: Arc<Window>) -> Result<Self, String> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let surface = instance
            .create_surface(window)
            .map_err(|error| format!("failed to create surface: {error}"))?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .map_err(|error| format!("failed to request adapter: {error}"))?;
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .map_err(|error| format!("failed to request device: {error}"))?;
        let capabilities = surface.get_capabilities(&adapter);
        let format = capabilities
            .formats
            .first()
            .copied()
            .ok_or_else(|| String::from("surface has no supported formats"))?;
        let size = window_size(&surface);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: capabilities.present_modes[0],
            alpha_mode: capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("solid rectangle shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(
                r#"
                struct VertexOutput {
                    @builtin(position) position: vec4<f32>,
                    @location(0) color: vec3<f32>,
                };

                @vertex
                fn vs_main(@location(0) position: vec2<f32>, @location(1) color: vec3<f32>) -> VertexOutput {
                    var output: VertexOutput;
                    output.position = vec4<f32>(position, 0.0, 1.0);
                    output.color = color;
                    return output;
                }

                @fragment
                fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
                    return vec4<f32>(input.color, 1.0);
                }
                "#,
            )),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("solid rectangle pipeline layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("solid rectangle pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x3,
                            offset: std::mem::size_of::<[f32; 2]>() as u64,
                            shader_location: 1,
                        },
                    ],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });
        let vertices = rectangle_vertices(280.0, 120.0, 80.0, 80.0);
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("solid rectangle vertices"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let stage_assets = AssetCatalog::load_stage_one()
            .map_err(|error| format!("failed to load stage one assets: {error}"))?;
        let sprite_sheet = stage_assets.player_sheet;
        let sprite_frame = sprite_sheet
            .frame(0, 0)
            .ok_or_else(|| String::from("placeholder sprite frame is missing"))?;
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("player sprite texture"),
            size: wgpu::Extent3d {
                width: sprite_frame.width,
                height: sprite_frame.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &sprite_frame.rgba8,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(sprite_frame.width * 4),
                rows_per_image: Some(sprite_frame.height),
            },
            wgpu::Extent3d {
                width: sprite_frame.width,
                height: sprite_frame.height,
                depth_or_array_layers: 1,
            },
        );
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("nearest sprite sampler"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("sprite texture bind group layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });
        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("player sprite bind group"),
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });
        let background_image = stage_assets.background_atlas;
        let (background_width, background_height) = background_image.dimensions();
        let background_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("stage 1 background atlas"),
            size: wgpu::Extent3d {
                width: background_width,
                height: background_height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &background_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            background_image.as_raw(),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(background_width * 4),
                rows_per_image: Some(background_height),
            },
            wgpu::Extent3d {
                width: background_width,
                height: background_height,
                depth_or_array_layers: 1,
            },
        );
        let background_view =
            background_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let background_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("stage 1 background bind group"),
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&background_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });
        let texture_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("sprite texture shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(
                r#"
                struct VertexOutput {
                    @builtin(position) position: vec4<f32>,
                    @location(0) uv: vec2<f32>,
                };

                @vertex
                fn vs_main(@location(0) position: vec2<f32>, @location(1) uv: vec2<f32>) -> VertexOutput {
                    var output: VertexOutput;
                    output.position = vec4<f32>(position, 0.0, 1.0);
                    output.uv = uv;
                    return output;
                }

                @group(0) @binding(0) var sprite_texture: texture_2d<f32>;
                @group(0) @binding(1) var sprite_sampler: sampler;

                @fragment
                fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
                    return textureSample(sprite_texture, sprite_sampler, input.uv);
                }
                "#,
            )),
        });
        let texture_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("sprite texture pipeline layout"),
                bind_group_layouts: &[&texture_bind_group_layout],
                push_constant_ranges: &[],
            });
        let texture_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("sprite texture pipeline"),
            layout: Some(&texture_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &texture_shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<TextureVertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: std::mem::size_of::<[f32; 2]>() as u64,
                            shader_location: 1,
                        },
                    ],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &texture_shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });
        let texture_vertices = texture_vertices(304.0, 144.0, 32.0, 32.0);
        let texture_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("player sprite vertices"),
            contents: bytemuck::cast_slice(&texture_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let background_map = TileMap::from_text_file("data/stage01_tilemap.txt")
            .map_err(|error| format!("failed to load stage one tile map: {error}"))?;
        let background_vertices = background_tile_vertices(0, &background_map);
        let background_vertex_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("stage 1 background vertices"),
                contents: bytemuck::cast_slice(&background_vertices),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });
        Ok(Self {
            surface,
            device,
            queue,
            config,
            pipeline,
            vertex_buffer,
            texture_pipeline,
            texture_bind_group,
            texture_vertex_buffer,
            background_bind_group,
            background_vertex_buffer,
            background_map,
        })
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 {
            return;
        }
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
    }

    pub fn render(&mut self, world: &World) -> Result<(), wgpu::SurfaceError> {
        let mut commands = vec![
            RenderCommand::Texture {
                layer: 0,
                texture_id: 1,
                x: 0,
                y: 0,
                width: 640,
                height: 320,
            },
            RenderCommand::SolidRect {
                layer: 1,
                x: 280,
                y: 120,
                width: 80,
                height: 80,
            },
            RenderCommand::GeometryRect {
                layer: 1,
                x: 0,
                y: 0,
                width: 640,
                height: 2,
            },
            RenderCommand::Texture {
                layer: 2,
                texture_id: 2,
                x: 304,
                y: 144,
                width: 32,
                height: 32,
            },
        ];
        sort_render_commands(&mut commands);
        let output = self.surface.get_current_texture()?;
        let background_vertices =
            background_tile_vertices(world.background_scroll.raw(), &self.background_map);
        self.queue.write_buffer(
            &self.background_vertex_buffer,
            0,
            bytemuck::cast_slice(&background_vertices),
        );
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("clear encoder"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("clear pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.015,
                            g: 0.02,
                            b: 0.06,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            let (viewport_x, viewport_y, viewport_width, viewport_height) =
                logical_viewport(self.config.width, self.config.height);
            pass.set_viewport(
                viewport_x,
                viewport_y,
                viewport_width,
                viewport_height,
                0.0,
                1.0,
            );
            pass.set_pipeline(&self.pipeline);
            pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            for command in commands {
                match command {
                    RenderCommand::Texture { texture_id: 1, .. } => {
                        pass.set_pipeline(&self.texture_pipeline);
                        pass.set_bind_group(0, &self.background_bind_group, &[]);
                        pass.set_vertex_buffer(0, self.background_vertex_buffer.slice(..));
                        pass.draw(0..(6 * 40 * 20), 0..1);
                    }
                    RenderCommand::Texture { texture_id: 2, .. } => {
                        pass.set_pipeline(&self.texture_pipeline);
                        pass.set_bind_group(0, &self.texture_bind_group, &[]);
                        pass.set_vertex_buffer(0, self.texture_vertex_buffer.slice(..));
                        pass.draw(0..6, 0..1);
                    }
                    RenderCommand::SolidRect { .. } | RenderCommand::GeometryRect { .. } => {
                        pass.set_pipeline(&self.pipeline);
                        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                        pass.draw(0..6, 0..1);
                    }
                    RenderCommand::Texture { .. } => {}
                }
            }
        }
        self.queue.submit(Some(encoder.finish()));
        output.present();
        Ok(())
    }
}

fn rectangle_vertices(x: f32, y: f32, width: f32, height: f32) -> [Vertex; 6] {
    let left = x / LOGICAL_WIDTH * 2.0 - 1.0;
    let right = (x + width) / LOGICAL_WIDTH * 2.0 - 1.0;
    let top = 1.0 - y / LOGICAL_HEIGHT * 2.0;
    let bottom = 1.0 - (y + height) / LOGICAL_HEIGHT * 2.0;
    let color = [0.15, 0.75, 0.95];
    [
        Vertex {
            position: [left, top],
            color,
        },
        Vertex {
            position: [right, top],
            color,
        },
        Vertex {
            position: [right, bottom],
            color,
        },
        Vertex {
            position: [left, top],
            color,
        },
        Vertex {
            position: [right, bottom],
            color,
        },
        Vertex {
            position: [left, bottom],
            color,
        },
    ]
}

fn texture_vertices(x: f32, y: f32, width: f32, height: f32) -> [TextureVertex; 6] {
    let left = x / LOGICAL_WIDTH * 2.0 - 1.0;
    let right = (x + width) / LOGICAL_WIDTH * 2.0 - 1.0;
    let top = 1.0 - y / LOGICAL_HEIGHT * 2.0;
    let bottom = 1.0 - (y + height) / LOGICAL_HEIGHT * 2.0;
    [
        TextureVertex {
            position: [left, top],
            uv: [0.0, 0.0],
        },
        TextureVertex {
            position: [right, top],
            uv: [1.0, 0.0],
        },
        TextureVertex {
            position: [right, bottom],
            uv: [1.0, 1.0],
        },
        TextureVertex {
            position: [left, top],
            uv: [0.0, 0.0],
        },
        TextureVertex {
            position: [right, bottom],
            uv: [1.0, 1.0],
        },
        TextureVertex {
            position: [left, bottom],
            uv: [0.0, 1.0],
        },
    ]
}

fn background_tile_vertices(scroll_y_raw: i16, tile_map: &TileMap) -> Vec<TextureVertex> {
    let mut vertices = Vec::with_capacity(6 * usize::from(tile_map.width) * 20);
    let scroll_y = f32::from(scroll_y_raw) / 16.0;
    for row in 0..20 {
        for column in 0..tile_map.width {
            let tile_id = tile_map.tile_id(row % tile_map.height as i32, column);
            if tile_id == 0 {
                continue;
            }
            let atlas_x = (tile_id % 4) as f32;
            let atlas_y = (tile_id / 4) as f32;
            let u0 = atlas_x * 0.25;
            let v0 = atlas_y * 0.25;
            let u1 = u0 + 0.25;
            let v1 = v0 + 0.25;
            let left = column as f32 * 16.0 / LOGICAL_WIDTH * 2.0 - 1.0;
            let right = (column as f32 * 16.0 + 16.0) / LOGICAL_WIDTH * 2.0 - 1.0;
            let top = 1.0 - (row as f32 * 16.0 - scroll_y) / LOGICAL_HEIGHT * 2.0;
            let bottom = 1.0 - (row as f32 * 16.0 + 16.0 - scroll_y) / LOGICAL_HEIGHT * 2.0;
            vertices.extend([
                TextureVertex {
                    position: [left, top],
                    uv: [u0, v0],
                },
                TextureVertex {
                    position: [right, top],
                    uv: [u1, v0],
                },
                TextureVertex {
                    position: [right, bottom],
                    uv: [u1, v1],
                },
                TextureVertex {
                    position: [left, top],
                    uv: [u0, v0],
                },
                TextureVertex {
                    position: [right, bottom],
                    uv: [u1, v1],
                },
                TextureVertex {
                    position: [left, bottom],
                    uv: [u0, v1],
                },
            ]);
        }
    }
    vertices
}

fn logical_viewport(width: u32, height: u32) -> (f32, f32, f32, f32) {
    let scale = (width as f32 / LOGICAL_WIDTH).min(height as f32 / LOGICAL_HEIGHT);
    let viewport_width = LOGICAL_WIDTH * scale;
    let viewport_height = LOGICAL_HEIGHT * scale;
    (
        (width as f32 - viewport_width) / 2.0,
        (height as f32 - viewport_height) / 2.0,
        viewport_width,
        viewport_height,
    )
}

fn window_size(_surface: &wgpu::Surface<'static>) -> PhysicalSize<u32> {
    PhysicalSize::new(1280, 640)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_commands_are_sorted_by_layer() {
        let mut commands = [
            RenderCommand::Texture {
                layer: 3,
                texture_id: 1,
                x: 0,
                y: 0,
                width: 1,
                height: 1,
            },
            RenderCommand::SolidRect {
                layer: 1,
                x: 0,
                y: 0,
                width: 1,
                height: 1,
            },
        ];
        sort_render_commands(&mut commands);
        assert_eq!(commands[0].layer(), 1);
    }
}
