use crate::{
    assets::AssetCatalog,
    background::TileMap,
    game::{
        GameState, World,
        name_entry::{CHAR_MATRIX, GridKey},
    },
};
use std::borrow::Cow;
use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::{dpi::PhysicalSize, window::Window};

#[allow(dead_code)]
pub trait PlatformWindow {}
#[allow(dead_code)]
pub trait Renderer {
    fn render(&mut self, world: &World);
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(dead_code)]
pub enum RenderCommand {
    SolidRect {
        layer: u8,
        x: i16,
        y: i16,
        width: u16,
        height: u16,
    },
    ColoredRect {
        layer: u8,
        x: i16,
        y: i16,
        width: u16,
        height: u16,
        color: [f32; 3],
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

#[allow(dead_code)]
impl RenderCommand {
    fn layer(self) -> u8 {
        match self {
            Self::SolidRect { layer, .. }
            | Self::ColoredRect { layer, .. }
            | Self::GeometryRect { layer, .. }
            | Self::Texture { layer, .. } => layer,
        }
    }
}

#[allow(dead_code)]
pub fn sort_render_commands(commands: &mut [RenderCommand]) {
    commands.sort_by_key(|command| command.layer());
}

const MAX_SOLID_VERTICES: usize = 65_536;
const LOGICAL_WIDTH: f32 = 640.0;
const LOGICAL_HEIGHT: f32 = 320.0;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
    pub color: [f32; 3],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct TextureVertex {
    position: [f32; 2],
    uv: [f32; 2],
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
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("dynamic solid rectangle vertices"),
            size: (MAX_SOLID_VERTICES * std::mem::size_of::<Vertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
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
        let output = self.surface.get_current_texture()?;

        // Generate solid vertices for UI, HUD, and game objects
        let mut solid_vertices = Vec::new();
        build_world_vertices(world, &mut solid_vertices);

        if !solid_vertices.is_empty() {
            let write_len = solid_vertices.len().min(MAX_SOLID_VERTICES);
            self.queue.write_buffer(
                &self.vertex_buffer,
                0,
                bytemuck::cast_slice(&solid_vertices[..write_len]),
            );
        }

        // Update background scroll
        let background_vertices =
            background_tile_vertices(world.background_scroll.raw(), &self.background_map);
        self.queue.write_buffer(
            &self.background_vertex_buffer,
            0,
            bytemuck::cast_slice(&background_vertices),
        );

        // Update player sprite position
        let px = (world.player_x.raw() as f32 / 16.0) - 16.0;
        let py = (world.player_y.raw() as f32 / 16.0) - 16.0;
        let p_vertices = texture_vertices(px, py, 32.0, 32.0);
        self.queue.write_buffer(
            &self.texture_vertex_buffer,
            0,
            bytemuck::cast_slice(&p_vertices),
        );

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render encoder"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("main render pass"),
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

            // 1. Draw background tiles if in gameplay states
            if matches!(
                world.state,
                GameState::Playing
                    | GameState::StageIntro
                    | GameState::StageClear
                    | GameState::GameOver
            ) {
                pass.set_pipeline(&self.texture_pipeline);
                pass.set_bind_group(0, &self.background_bind_group, &[]);
                pass.set_vertex_buffer(0, self.background_vertex_buffer.slice(..));
                pass.draw(0..(6 * 40 * 20), 0..1);

                // 2. Draw player sprite (blink if invincible)
                let show_player = world.invincible_frames == 0 || (world.invincible_frames % 4 < 2);
                if show_player && world.hp > 0 && world.lives > 0 {
                    pass.set_pipeline(&self.texture_pipeline);
                    pass.set_bind_group(0, &self.texture_bind_group, &[]);
                    pass.set_vertex_buffer(0, self.texture_vertex_buffer.slice(..));
                    pass.draw(0..6, 0..1);
                }
            }

            // 3. Draw solid vertices (HUD, UI, Bullets, Enemies, Name Entry, Rankings)
            if !solid_vertices.is_empty() {
                let count = solid_vertices.len().min(MAX_SOLID_VERTICES) as u32;
                pass.set_pipeline(&self.pipeline);
                pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                pass.draw(0..count, 0..1);
            }
        }
        self.queue.submit(Some(encoder.finish()));
        output.present();
        Ok(())
    }
}

pub fn push_rect_colored(
    vertices: &mut Vec<Vertex>,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    color: [f32; 3],
) {
    let left = x / LOGICAL_WIDTH * 2.0 - 1.0;
    let right = (x + width) / LOGICAL_WIDTH * 2.0 - 1.0;
    let top = 1.0 - y / LOGICAL_HEIGHT * 2.0;
    let bottom = 1.0 - (y + height) / LOGICAL_HEIGHT * 2.0;
    vertices.extend([
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
    ]);
}

pub fn push_char_colored(
    vertices: &mut Vec<Vertex>,
    x: f32,
    y: f32,
    ch: char,
    color: [f32; 3],
    scale: f32,
) {
    let bitmap = glyph_bitmap(ch);
    for (row_idx, &row_bits) in bitmap.iter().enumerate() {
        for col_idx in 0..5 {
            if (row_bits & (1 << (4 - col_idx))) != 0 {
                let px = x + col_idx as f32 * scale;
                let py = y + row_idx as f32 * scale;
                push_rect_colored(vertices, px, py, scale, scale, color);
            }
        }
    }
}

pub fn push_text_colored(
    vertices: &mut Vec<Vertex>,
    x: f32,
    y: f32,
    text: &str,
    color: [f32; 3],
    scale: f32,
) {
    let mut cur_x = x;
    let char_width = 6.0 * scale;
    for ch in text.chars() {
        if ch == '\n' {
            continue;
        }
        push_char_colored(vertices, cur_x, y, ch, color, scale);
        cur_x += char_width;
    }
}

pub fn build_world_vertices(world: &World, vertices: &mut Vec<Vertex>) {
    match world.state {
        GameState::Title => {
            push_text_colored(vertices, 160.0, 90.0, "RUST SHOOTING", [0.3, 0.8, 1.0], 4.0);
            if (world.frame / 20).is_multiple_of(2) {
                push_text_colored(
                    vertices,
                    200.0,
                    200.0,
                    "PRESS SPACE TO START",
                    [1.0, 1.0, 1.0],
                    2.0,
                );
            }
            push_text_colored(
                vertices,
                180.0,
                280.0,
                "COPYRIGHT (C) 2026 RUST SHOOTING TEAM",
                [0.5, 0.5, 0.6],
                1.0,
            );
        }
        GameState::Demo => {
            push_text_colored(
                vertices,
                200.0,
                20.0,
                "TOP 10 RANKINGS",
                [1.0, 0.85, 0.2],
                3.0,
            );
            for (i, entry) in world.ranking.entries.iter().enumerate() {
                let y = 65.0 + i as f32 * 20.0;
                let rank_color = match i {
                    0 => [1.0, 0.85, 0.2],
                    1 => [0.85, 0.85, 0.9],
                    2 => [0.8, 0.55, 0.3],
                    _ => [0.7, 0.8, 0.9],
                };
                let text = format!("{:2}.   {}   {:8}", i + 1, entry.name_str(), entry.score);
                push_text_colored(vertices, 180.0, y, &text, rank_color, 2.0);
            }
            if (world.frame / 20).is_multiple_of(2) {
                push_text_colored(
                    vertices,
                    200.0,
                    295.0,
                    "PRESS SPACE TO START",
                    [1.0, 1.0, 1.0],
                    2.0,
                );
            }
        }
        GameState::NameEntry => {
            push_text_colored(vertices, 220.0, 15.0, "NAME ENTRY", [1.0, 0.85, 0.2], 3.0);
            let score_text = format!("SCORE: {:08}", world.score);
            push_text_colored(vertices, 230.0, 50.0, &score_text, [1.0, 1.0, 1.0], 2.0);

            let name_str = std::str::from_utf8(&world.name_entry.name).unwrap_or("???");
            let name_display = format!("[ {} ]", name_str);
            push_text_colored(vertices, 260.0, 80.0, &name_display, [0.3, 0.9, 1.0], 3.0);

            // Matrix rendering (7 cols x 6 rows)
            let start_x = 160.0;
            let start_y = 120.0;
            let col_w = 46.0;
            let row_h = 24.0;

            for (r, row) in CHAR_MATRIX.iter().enumerate() {
                for (c, &key) in row.iter().enumerate() {
                    let x = start_x + c as f32 * col_w;
                    let y = start_y + r as f32 * row_h;

                    let is_cursor =
                        world.name_entry.cursor_col == c && world.name_entry.cursor_row == r;

                    if is_cursor {
                        // Highlight cursor box
                        push_rect_colored(
                            vertices,
                            x - 4.0,
                            y - 4.0,
                            col_w - 4.0,
                            row_h - 4.0,
                            [0.9, 0.8, 0.1],
                        );
                    }

                    let text_color = if is_cursor {
                        [0.0, 0.0, 0.0]
                    } else {
                        [0.9, 0.9, 0.9]
                    };

                    match key {
                        GridKey::Char(b) => {
                            let mut buf = [0u8; 4];
                            let s = (b as char).encode_utf8(&mut buf);
                            push_text_colored(vertices, x + 6.0, y, s, text_color, 2.0);
                        }
                        GridKey::Left => {
                            push_text_colored(vertices, x + 6.0, y, "<", text_color, 2.0);
                        }
                        GridKey::Right => {
                            push_text_colored(vertices, x + 6.0, y, ">", text_color, 2.0);
                        }
                        GridKey::Delete => {
                            push_text_colored(vertices, x, y + 2.0, "DEL", text_color, 1.5);
                        }
                        GridKey::Space => {
                            push_text_colored(vertices, x, y + 2.0, "SPC", text_color, 1.5);
                        }
                    }
                }
            }

            push_text_colored(
                vertices,
                160.0,
                295.0,
                "MOVE: ARROWS   SELECT: SPACE",
                [0.7, 0.7, 0.7],
                1.5,
            );
        }
        GameState::GameOver => {
            push_text_colored(vertices, 200.0, 100.0, "GAME OVER", [0.9, 0.2, 0.2], 4.0);
            let score_text = format!("FINAL SCORE: {:08}", world.score);
            push_text_colored(vertices, 200.0, 180.0, &score_text, [1.0, 1.0, 1.0], 2.0);
        }
        GameState::Ending => {
            push_text_colored(
                vertices,
                120.0,
                80.0,
                "ALL STAGES CLEARED!",
                [1.0, 0.85, 0.2],
                3.0,
            );
            push_text_colored(
                vertices,
                180.0,
                130.0,
                "CONGRATULATIONS!",
                [0.3, 0.9, 0.5],
                3.0,
            );
            let score_text = format!("FINAL SCORE: {:08}", world.score);
            push_text_colored(vertices, 200.0, 190.0, &score_text, [1.0, 1.0, 1.0], 2.0);
        }
        GameState::Playing | GameState::StageIntro | GameState::StageClear => {
            // Draw gameplay elements
            // Player bullets
            world.player_bullets.for_each_active(|b| {
                let x = b.x.raw() as f32 / 16.0 - 4.0;
                let y = b.y.raw() as f32 / 16.0 - 8.0;
                let color = if b.character_id == 4 {
                    [0.2, 1.0, 0.9] // Penetrating bullet (cyan-bright)
                } else {
                    [0.3, 0.85, 1.0] // Regular bullet
                };
                push_rect_colored(vertices, x, y, 8.0, 16.0, color);
            });

            // Enemies
            world.enemies.for_each_active(|e| {
                let x = e.x.raw() as f32 / 16.0;
                let y = e.y.raw() as f32 / 16.0;
                match e.character_id {
                    100..=105 => {
                        // Bosses
                        let boss_color = match e.character_id {
                            100 => [0.9, 0.2, 0.3], // Red
                            101 => [0.9, 0.5, 0.1], // Orange
                            102 => [0.7, 0.2, 0.9], // Purple
                            103 => [0.1, 0.8, 0.9], // Cyan
                            104 => [0.8, 0.8, 0.2], // Yellow
                            _ => [1.0, 0.1, 0.4],   // Omega Red-Pink
                        };
                        push_rect_colored(vertices, x - 32.0, y - 24.0, 64.0, 48.0, boss_color);
                        // Boss core
                        let pulse = (world.frame % 20 < 10) as i32;
                        let core_color = if pulse == 1 {
                            [1.0, 1.0, 1.0]
                        } else {
                            [1.0, 0.8, 0.0]
                        };
                        push_rect_colored(vertices, x - 12.0, y - 10.0, 24.0, 20.0, core_color);
                    }
                    6 => {
                        // Mid-boss Cruiser
                        push_rect_colored(
                            vertices,
                            x - 20.0,
                            y - 16.0,
                            40.0,
                            32.0,
                            [0.8, 0.3, 0.5],
                        );
                        push_rect_colored(vertices, x - 8.0, y - 6.0, 16.0, 12.0, [1.0, 0.9, 0.2]);
                    }
                    4 => {
                        // Heavy Turret Enemy
                        push_rect_colored(
                            vertices,
                            x - 12.0,
                            y - 12.0,
                            24.0,
                            24.0,
                            [0.2, 0.5, 0.9],
                        );
                    }
                    5 => {
                        // Spreader Enemy
                        push_rect_colored(
                            vertices,
                            x - 10.0,
                            y - 10.0,
                            20.0,
                            20.0,
                            [0.7, 0.3, 0.8],
                        );
                    }
                    3 => {
                        // Fast/Rush Enemy
                        push_rect_colored(vertices, x - 8.0, y - 8.0, 16.0, 16.0, [0.9, 0.8, 0.1]);
                    }
                    _ => {
                        // Standard Enemy
                        push_rect_colored(vertices, x - 8.0, y - 8.0, 16.0, 16.0, [0.9, 0.4, 0.1]);
                    }
                }
            });

            // Enemy bullets
            world.enemy_bullets.for_each_active(|b| {
                let x = b.x.raw() as f32 / 16.0;
                let y = b.y.raw() as f32 / 16.0;
                match b.character_id {
                    3 => {
                        // Heavy Bullet
                        push_rect_colored(vertices, x - 5.0, y - 5.0, 10.0, 10.0, [1.0, 0.4, 0.1]);
                        push_rect_colored(vertices, x - 2.0, y - 2.0, 4.0, 4.0, [1.0, 1.0, 0.8]);
                    }
                    5 => {
                        // Fast Bullet
                        push_rect_colored(vertices, x - 3.0, y - 3.0, 6.0, 6.0, [0.9, 0.1, 0.8]);
                    }
                    _ => {
                        // Standard Bullet
                        push_rect_colored(vertices, x - 3.0, y - 3.0, 6.0, 6.0, [1.0, 0.2, 0.3]);
                    }
                }
            });

            // Items
            world.items.for_each_active(|item| {
                let x = item.x.raw() as f32 / 16.0 - 6.0;
                let y = item.y.raw() as f32 / 16.0 - 6.0;
                let pulse = (world.frame % 10 < 5) as i32;
                let color = if pulse == 1 {
                    [0.3, 1.0, 0.5]
                } else {
                    [0.1, 0.8, 0.3]
                };
                push_rect_colored(vertices, x, y, 12.0, 12.0, color);
                push_text_colored(vertices, x + 2.0, y + 2.0, "P", [1.0, 1.0, 1.0], 1.2);
            });

            // Destruction and growth effects
            world.effects.for_each_active(|effect| {
                let x = effect.x.raw() as f32 / 16.0;
                let y = effect.y.raw() as f32 / 16.0;
                let progress = effect.frame as f32 / effect.max_frames.max(1) as f32;
                match effect.effect_id {
                    2 => {
                        // Boss Explosion: expanding geometric rings and flashes
                        let current_size = (effect.size as f32) * (0.5 + progress * 1.5);
                        let alpha = 1.0 - progress;
                        push_rect_colored(
                            vertices,
                            x - current_size / 2.0,
                            y - current_size / 2.0,
                            current_size,
                            current_size,
                            [1.0 * alpha, 0.6 * alpha, 0.1 * alpha],
                        );
                        let sub_size = current_size * 0.6;
                        push_rect_colored(
                            vertices,
                            x - sub_size / 2.0,
                            y - sub_size / 2.0,
                            sub_size,
                            sub_size,
                            [1.0 * alpha, 0.9 * alpha, 0.4 * alpha],
                        );
                    }
                    1 => {
                        // Item / Growth pickup effect: expanding green cross
                        let s = (effect.size as f32) * (0.2 + progress * 0.8);
                        let alpha = 1.0 - progress;
                        push_rect_colored(
                            vertices,
                            x - s / 2.0,
                            y - 2.0,
                            s,
                            4.0,
                            [0.2 * alpha, 1.0 * alpha, 0.4 * alpha],
                        );
                        push_rect_colored(
                            vertices,
                            x - 2.0,
                            y - s / 2.0,
                            4.0,
                            s,
                            [0.2 * alpha, 1.0 * alpha, 0.4 * alpha],
                        );
                    }
                    _ => {
                        // Standard enemy explosion: expanding diamond / rectangle
                        let current_size = (effect.size as f32) * (0.3 + progress * 0.7);
                        let alpha = 1.0 - progress;
                        push_rect_colored(
                            vertices,
                            x - current_size / 2.0,
                            y - current_size / 2.0,
                            current_size,
                            current_size,
                            [1.0 * alpha, 0.5 * alpha, 0.1 * alpha],
                        );
                    }
                }
            });

            // Stage Intro / Clear banners
            if world.state == GameState::StageIntro {
                let text = format!("STAGE {:02}", world.stage_id);
                push_text_colored(vertices, 240.0, 130.0, &text, [1.0, 1.0, 1.0], 3.0);
                let subtext = match world.stage_id {
                    1 => "ASTEROID BELT",
                    2 => "SPACE STATION",
                    3 => "NEBULA STORM",
                    4 => "CYBER MATRIX",
                    5 => "FORTRESS APPROACH",
                    6 => "FINAL: CORE INTRUSION",
                    _ => "UNKNOWN REGION",
                };
                push_text_colored(vertices, 230.0, 160.0, subtext, [0.3, 0.85, 1.0], 1.5);
                push_text_colored(vertices, 245.0, 185.0, "READY!", [1.0, 0.85, 0.2], 2.0);
            } else if world.state == GameState::StageClear {
                let text = format!("STAGE {:02} CLEAR!", world.stage_id);
                push_text_colored(vertices, 200.0, 140.0, &text, [1.0, 0.85, 0.2], 3.0);
            }

            // HUD rendering at bottom line (y=304..312)
            append_hud_vertices(world, vertices);
        }
    }
}

pub fn append_hud_vertices(world: &World, vertices: &mut Vec<Vertex>) {
    // HUD background bar (x=0, y=304, width=640, height=16)
    push_rect_colored(vertices, 0.0, 304.0, 640.0, 16.0, [0.03, 0.04, 0.08]);

    // 1. HP Gauge (x=8, y=304, width=160, height=8)
    push_rect_colored(vertices, 8.0, 306.0, 160.0, 10.0, [0.2, 0.15, 0.15]);
    let hp_ratio = (world.hp as f32 / 100.0).clamp(0.0, 1.0);
    let hp_color = if world.hp > 50 {
        [0.15, 0.9, 0.3]
    } else if world.hp > 25 {
        [0.95, 0.85, 0.15]
    } else {
        [0.95, 0.2, 0.2]
    };
    push_rect_colored(vertices, 8.0, 306.0, 160.0 * hp_ratio, 10.0, hp_color);

    // 2. Score (x=176, y=304, width=160, height=8)
    let score_str = format!("{:08}", world.score);
    push_text_colored(vertices, 176.0, 307.0, &score_str, [1.0, 1.0, 1.0], 1.2);

    // 3. Lives (x=344, y=304, width=96, height=8)
    let lives_str = format!("L:{:02}", world.lives);
    push_text_colored(vertices, 344.0, 307.0, &lives_str, [0.3, 0.85, 1.0], 1.2);

    // 4. Stock / Growth (x=448, y=304, width=184, height=8)
    let stock_str = if world.growth_level < 4 {
        format!("LV:{}", world.growth_level)
    } else {
        format!("ST:+{}", world.recovery_stock)
    };
    push_text_colored(vertices, 448.0, 307.0, &stock_str, [1.0, 0.85, 0.2], 1.2);
}

fn glyph_bitmap(ch: char) -> [u8; 7] {
    match ch.to_ascii_uppercase() {
        'A' => [
            0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
        'B' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110,
        ],
        'C' => [
            0b01111, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b01111,
        ],
        'D' => [
            0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110,
        ],
        'E' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111,
        ],
        'F' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000,
        ],
        'G' => [
            0b01111, 0b10000, 0b10000, 0b10111, 0b10001, 0b10001, 0b01110,
        ],
        'H' => [
            0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
        'I' => [
            0b01110, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
        ],
        'J' => [
            0b00001, 0b00001, 0b00001, 0b00001, 0b10001, 0b10001, 0b01110,
        ],
        'K' => [
            0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001,
        ],
        'L' => [
            0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111,
        ],
        'M' => [
            0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001,
        ],
        'N' => [
            0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001,
        ],
        'O' => [
            0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        'P' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000,
        ],
        'Q' => [
            0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10011, 0b01111,
        ],
        'R' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10001, 0b10001,
        ],
        'S' => [
            0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110,
        ],
        'T' => [
            0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        'U' => [
            0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        'V' => [
            0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100,
        ],
        'W' => [
            0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b11011, 0b10001,
        ],
        'X' => [
            0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001,
        ],
        'Y' => [
            0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        'Z' => [
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111,
        ],
        '0' => [
            0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110,
        ],
        '1' => [
            0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
        ],
        '2' => [
            0b01110, 0b10001, 0b00001, 0b00110, 0b01000, 0b10000, 0b11111,
        ],
        '3' => [
            0b11110, 0b00001, 0b00001, 0b01110, 0b00001, 0b00001, 0b11110,
        ],
        '4' => [
            0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010,
        ],
        '5' => [
            0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110,
        ],
        '6' => [
            0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110,
        ],
        '7' => [
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000,
        ],
        '8' => [
            0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110,
        ],
        '9' => [
            0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100,
        ],
        '.' => [
            0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00110, 0b00110,
        ],
        '-' => [
            0b00000, 0b00000, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000,
        ],
        '<' => [
            0b00010, 0b00100, 0b01000, 0b10000, 0b01000, 0b00100, 0b00010,
        ],
        '>' => [
            0b01000, 0b00100, 0b00010, 0b00001, 0b00010, 0b00100, 0b01000,
        ],
        '!' => [
            0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00000, 0b00100,
        ],
        ':' => [
            0b00000, 0b00110, 0b00110, 0b00000, 0b00110, 0b00110, 0b00000,
        ],
        '[' => [
            0b01110, 0b01000, 0b01000, 0b01000, 0b01000, 0b01000, 0b01110,
        ],
        ']' => [
            0b01110, 0b00010, 0b00010, 0b00010, 0b00010, 0b00010, 0b01110,
        ],
        '*' => [
            0b00000, 0b10101, 0b01110, 0b11111, 0b01110, 0b10101, 0b00000,
        ],
        '+' => [
            0b00000, 0b00100, 0b00100, 0b11111, 0b00100, 0b00100, 0b00000,
        ],
        '/' => [
            0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b00000, 0b00000,
        ],
        _ => [0b00000; 7],
    }
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
                width: 10,
                height: 10,
            },
            RenderCommand::SolidRect {
                layer: 1,
                x: 0,
                y: 0,
                width: 10,
                height: 10,
            },
        ];
        sort_render_commands(&mut commands);
        assert_eq!(commands[0].layer(), 1);
        assert_eq!(commands[1].layer(), 3);
    }

    #[test]
    fn hud_generates_vertices_for_hp_score_lives_stock() {
        let world = World {
            state: GameState::Playing,
            hp: 75,
            score: 12_340,
            lives: 2,
            growth_level: 3,
            ..World::default()
        };
        let mut vertices = Vec::new();
        append_hud_vertices(&world, &mut vertices);
        assert!(!vertices.is_empty());
    }

    #[test]
    fn demo_and_name_entry_generate_vertices() {
        let mut world = World::default();
        let mut demo_vertices = Vec::new();
        world.state = GameState::Demo;
        build_world_vertices(&world, &mut demo_vertices);
        assert!(!demo_vertices.is_empty());

        world.state = GameState::NameEntry;
        let mut entry_vertices = Vec::new();
        build_world_vertices(&world, &mut entry_vertices);
        assert!(!entry_vertices.is_empty());
    }
}
