use crate::{
    assets::AssetCatalog,
    background::TileMap,
    data::{
        HUD_Y, LOGICAL_HEIGHT, LOGICAL_WIDTH, SCREEN_HEIGHT, SCREEN_WIDTH, enemy_visual_data,
        player_growth_data, stage_enemy_character_ids,
    },
    game::{
        GameState, World,
        name_entry::{CHAR_MATRIX, GridKey},
    },
};
use std::borrow::Cow;
use std::sync::Arc;
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

const TILE_SIZE: f32 = 16.0;
const TILE_COLUMNS: usize = (SCREEN_WIDTH / 16) as usize; // 30
const TILE_ROWS_VISIBLE: usize = (SCREEN_HEIGHT / 16) as usize + 2; // 42
const MAX_SOLID_VERTICES: usize = 65_536;
const MAX_BACKGROUND_VERTICES: usize = 6 * TILE_COLUMNS * TILE_ROWS_VISIBLE;

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
    player_bind_groups: Vec<[wgpu::BindGroup; 2]>,
    enemy_bind_groups: Vec<Vec<[wgpu::BindGroup; 2]>>,
    enemy_character_ids: Vec<Vec<u16>>,
    boss_bind_groups: [wgpu::BindGroup; 2],
    item_bind_groups: [wgpu::BindGroup; 2],
    player_bullet_bind_groups: Vec<[wgpu::BindGroup; 2]>,
    enemy_bullet_bind_groups: Vec<[wgpu::BindGroup; 2]>,
    sprite_vertex_buffer: wgpu::Buffer,
    background_bind_groups: Vec<wgpu::BindGroup>,
    background_vertex_buffer: wgpu::Buffer,
    background_maps: Vec<TileMap>,
}

fn create_background_bind_group(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
    sampler: &wgpu::Sampler,
    image: &image::RgbaImage,
    stage_id: u8,
) -> wgpu::BindGroup {
    let (width, height) = image.dimensions();
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(&format!("stage {stage_id} background atlas")),
        size: wgpu::Extent3d {
            width,
            height,
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
        image.as_raw(),
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(width * 4),
            rows_per_image: Some(height),
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(&format!("stage {stage_id} background bind group")),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
        ],
    })
}

fn create_frame_bind_group(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
    sampler: &wgpu::Sampler,
    label: &str,
    frame: &crate::sprite::SpriteFrame,
) -> wgpu::BindGroup {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width: frame.width,
            height: frame.height,
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
        &frame.rgba8,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(frame.width * 4),
            rows_per_image: Some(frame.height),
        },
        wgpu::Extent3d {
            width: frame.width,
            height: frame.height,
            depth_or_array_layers: 1,
        },
    );
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(label),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
        ],
    })
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
        let sprite_vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("dynamic sprite vertices"),
            size: (MAX_SOLID_VERTICES * std::mem::size_of::<TextureVertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let stage_assets = AssetCatalog::load_stage_one()
            .map_err(|error| format!("failed to load stage one assets: {error}"))?;
        let enemy_character_ids = (1..=6).map(stage_enemy_character_ids).collect();

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

        let player_bind_groups = stage_assets
            .player_variant_sheets
            .iter()
            .enumerate()
            .map(|(level, sheet)| {
                let frame_0 = sheet
                    .frame(0, 0)
                    .ok_or(format!("player level {level} frame 0 missing"))?;
                let frame_1 = sheet.frame(0, 1).unwrap_or(frame_0);
                Ok([
                    create_frame_bind_group(
                        &device,
                        &queue,
                        &texture_bind_group_layout,
                        &sampler,
                        &format!("player level {level} frame 0"),
                        frame_0,
                    ),
                    create_frame_bind_group(
                        &device,
                        &queue,
                        &texture_bind_group_layout,
                        &sampler,
                        &format!("player level {level} frame 1"),
                        frame_1,
                    ),
                ])
            })
            .collect::<Result<Vec<_>, String>>()?;

        let enemy_bind_groups = stage_assets
            .stage_enemy_sheets
            .iter()
            .enumerate()
            .map(|(stage_index, sheets)| {
                sheets
                    .iter()
                    .enumerate()
                    .map(|(variant, sheet)| {
                        let frame_0 = sheet.frame(0, 0).ok_or(format!(
                            "stage {stage_index} enemy variant {variant} frame 0 missing"
                        ))?;
                        let frame_1 = sheet.frame(0, 1).unwrap_or(frame_0);
                        Ok([
                            create_frame_bind_group(
                                &device,
                                &queue,
                                &texture_bind_group_layout,
                                &sampler,
                                &format!("stage {stage_index} enemy variant {variant} frame 0"),
                                frame_0,
                            ),
                            create_frame_bind_group(
                                &device,
                                &queue,
                                &texture_bind_group_layout,
                                &sampler,
                                &format!("stage {stage_index} enemy variant {variant} frame 1"),
                                frame_1,
                            ),
                        ])
                    })
                    .collect::<Result<Vec<_>, String>>()
            })
            .collect::<Result<Vec<_>, String>>()?;

        let boss_frame_0 = stage_assets
            .boss_sheet
            .frame(0, 0)
            .ok_or("boss frame 0 missing")?;
        let boss_frame_1 = stage_assets.boss_sheet.frame(0, 1).unwrap_or(boss_frame_0);
        let boss_bind_groups = [
            create_frame_bind_group(
                &device,
                &queue,
                &texture_bind_group_layout,
                &sampler,
                "boss 0",
                boss_frame_0,
            ),
            create_frame_bind_group(
                &device,
                &queue,
                &texture_bind_group_layout,
                &sampler,
                "boss 1",
                boss_frame_1,
            ),
        ];

        let item_frame_0 = stage_assets
            .item_sheet
            .frame(0, 0)
            .ok_or("item frame 0 missing")?;
        let item_frame_1 = stage_assets.item_sheet.frame(0, 1).unwrap_or(item_frame_0);
        let item_bind_groups = [
            create_frame_bind_group(
                &device,
                &queue,
                &texture_bind_group_layout,
                &sampler,
                "item 0",
                item_frame_0,
            ),
            create_frame_bind_group(
                &device,
                &queue,
                &texture_bind_group_layout,
                &sampler,
                "item 1",
                item_frame_1,
            ),
        ];

        let player_bullet_bind_groups = stage_assets
            .player_bullet_sheets
            .iter()
            .enumerate()
            .map(|(variant, sheet)| {
                let frame_0 = sheet
                    .frame(0, 0)
                    .ok_or(format!("player bullet {variant} frame 0 missing"))?;
                let frame_1 = sheet.frame(0, 1).unwrap_or(frame_0);
                Ok([
                    create_frame_bind_group(
                        &device,
                        &queue,
                        &texture_bind_group_layout,
                        &sampler,
                        &format!("player bullet {variant} frame 0"),
                        frame_0,
                    ),
                    create_frame_bind_group(
                        &device,
                        &queue,
                        &texture_bind_group_layout,
                        &sampler,
                        &format!("player bullet {variant} frame 1"),
                        frame_1,
                    ),
                ])
            })
            .collect::<Result<Vec<_>, String>>()?;
        let enemy_bullet_bind_groups = stage_assets
            .enemy_bullet_variant_sheets
            .iter()
            .enumerate()
            .map(|(variant, sheet)| {
                let frame_0 = sheet
                    .frame(0, 0)
                    .ok_or(format!("enemy bullet variant {variant} frame 0 missing"))?;
                let frame_1 = sheet.frame(0, 1).unwrap_or(frame_0);
                Ok([
                    create_frame_bind_group(
                        &device,
                        &queue,
                        &texture_bind_group_layout,
                        &sampler,
                        &format!("enemy bullet variant {variant} frame 0"),
                        frame_0,
                    ),
                    create_frame_bind_group(
                        &device,
                        &queue,
                        &texture_bind_group_layout,
                        &sampler,
                        &format!("enemy bullet variant {variant} frame 1"),
                        frame_1,
                    ),
                ])
            })
            .collect::<Result<Vec<_>, String>>()?;
        let mut background_bind_groups = Vec::with_capacity(6);
        let mut background_maps = Vec::with_capacity(6);
        for stage_id in 1..=6 {
            let assets = AssetCatalog::load_stage_id(stage_id)
                .map_err(|error| format!("failed to load stage {stage_id} assets: {error}"))?;
            background_bind_groups.push(create_background_bind_group(
                &device,
                &queue,
                &texture_bind_group_layout,
                &sampler,
                &assets.background_atlas,
                stage_id,
            ));
            background_maps.push(
                TileMap::from_text_file(format!("data/stage{:02}_tilemap.txt", stage_id)).map_err(
                    |error| format!("failed to load stage {stage_id} tile map: {error}"),
                )?,
            );
        }
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
        let background_vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("stage 1 background vertices"),
            size: (MAX_BACKGROUND_VERTICES * std::mem::size_of::<TextureVertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Ok(Self {
            surface,
            device,
            queue,
            config,
            pipeline,
            vertex_buffer,
            texture_pipeline,
            player_bind_groups,
            enemy_bind_groups,
            enemy_character_ids,
            boss_bind_groups,
            item_bind_groups,
            player_bullet_bind_groups,
            enemy_bullet_bind_groups,
            sprite_vertex_buffer,
            background_bind_groups,
            background_vertex_buffer,
            background_maps,
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

    pub fn render(
        &mut self,
        world: &World,
        paused: bool,
        pause_frame: u32,
    ) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;

        // Generate solid vertices for UI, HUD, and effects
        let mut solid_vertices = Vec::new();
        build_world_vertices(world, &mut solid_vertices);
        if paused && (pause_frame / 20).is_multiple_of(2) {
            push_text_centered(
                &mut solid_vertices,
                LOGICAL_HEIGHT / 2.0,
                "PAUSE",
                [1.0, 0.85, 0.2],
                5.0,
            );
        }

        if !solid_vertices.is_empty() {
            let write_len = solid_vertices.len().min(MAX_SOLID_VERTICES);
            self.queue.write_buffer(
                &self.vertex_buffer,
                0,
                bytemuck::cast_slice(&solid_vertices[..write_len]),
            );
        }

        // Update background scroll
        let background_index = usize::from(world.stage_id.saturating_sub(1)).min(5);
        let background_vertices = background_tile_vertices(
            world.background_scroll.raw(),
            &self.background_maps[background_index],
        );
        if !background_vertices.is_empty() {
            let write_len = background_vertices.len().min(MAX_BACKGROUND_VERTICES);
            self.queue.write_buffer(
                &self.background_vertex_buffer,
                0,
                bytemuck::cast_slice(&background_vertices[..write_len]),
            );
        }

        // Animation frame indices
        let anim_frame = if (world.frame / 8).is_multiple_of(2) {
            1
        } else {
            0
        };
        let player_anim = if (world.frame / 6).is_multiple_of(2) {
            1
        } else {
            0
        };
        let boss_anim = if (world.frame / 10).is_multiple_of(2) {
            1
        } else {
            0
        };

        // Collect sprite batches
        let mut all_sprite_vertices: Vec<TextureVertex> = Vec::new();

        // 1. Enemy sprites
        let stage_index = usize::from(world.stage_id.saturating_sub(1)).min(5);
        let stage_enemy_ids = &self.enemy_character_ids[stage_index];
        let mut enemy_ranges = vec![(0usize, 0usize); stage_enemy_ids.len()];
        for (variant, character_id) in stage_enemy_ids.iter().copied().enumerate() {
            let start = all_sprite_vertices.len();
            world.enemies.for_each_active(|e| {
                if e.character_id == character_id {
                    let x = e.x.raw() as f32 / 16.0;
                    let y = e.y.raw() as f32 / 16.0;
                    let visual = enemy_visual_data(character_id);
                    let width = visual.width as f32;
                    let height = visual.height as f32;
                    push_texture_quad(
                        &mut all_sprite_vertices,
                        x - width / 2.0,
                        y - height / 2.0,
                        width,
                        height,
                    );
                }
            });
            enemy_ranges[variant] = (start, all_sprite_vertices.len());
        }

        // 2. Boss sprites
        let boss_start = all_sprite_vertices.len();
        world.enemies.for_each_active(|e| {
            if e.character_id >= 100 && e.character_id <= 105 {
                let x = e.x.raw() as f32 / 16.0;
                let y = e.y.raw() as f32 / 16.0;
                let visual = enemy_visual_data(e.character_id);
                let width = visual.width as f32;
                let height = visual.height as f32;
                push_texture_quad(
                    &mut all_sprite_vertices,
                    x - width / 2.0,
                    y - height / 2.0,
                    width,
                    height,
                );
            }
        });
        let boss_end = all_sprite_vertices.len();

        // 3. Item sprites
        let item_start = all_sprite_vertices.len();
        world.items.for_each_active(|item| {
            let x = item.x.raw() as f32 / 16.0;
            let y = item.y.raw() as f32 / 16.0;
            push_texture_quad(&mut all_sprite_vertices, x - 16.0, y - 16.0, 32.0, 32.0);
        });
        let item_end = all_sprite_vertices.len();

        // 4. Bullet sprites (player variants, enemy)
        let player_bullet_ids = [1u16, 2, 3, 4, 5];
        let mut player_bullet_ranges = [(0usize, 0usize); 5];
        for (variant, character_id) in player_bullet_ids.into_iter().enumerate() {
            let start = all_sprite_vertices.len();
            world.player_bullets.for_each_active(|b| {
                if b.character_id == character_id {
                    let x = b.x.raw() as f32 / 16.0;
                    let y = b.y.raw() as f32 / 16.0;
                    push_texture_quad(&mut all_sprite_vertices, x - 8.0, y - 8.0, 16.0, 16.0);
                }
            });
            player_bullet_ranges[variant] = (start, all_sprite_vertices.len());
        }

        let enemy_bullet_ids = [6u16, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let mut enemy_bullet_ranges = [(0usize, 0usize); 11];
        for (variant, character_id) in enemy_bullet_ids.into_iter().enumerate() {
            let start = all_sprite_vertices.len();
            world.enemy_bullets.for_each_active(|b| {
                if b.character_id == character_id {
                    let x = b.x.raw() as f32 / 16.0;
                    let y = b.y.raw() as f32 / 16.0;
                    push_texture_quad(&mut all_sprite_vertices, x - 4.0, y - 4.0, 8.0, 8.0);
                }
            });
            enemy_bullet_ranges[variant] = (start, all_sprite_vertices.len());
        }

        // 5. Player sprite
        let player_start = all_sprite_vertices.len();
        let show_player = world.invincible_frames == 0 || (world.invincible_frames % 4 < 2);
        if show_player && world.hp > 0 && world.lives > 0 {
            let visual = player_growth_data(world.growth_level);
            let width = visual.visual_width as f32;
            let height = visual.visual_height as f32;
            let px = world.player_x.raw() as f32 / 16.0 - width / 2.0;
            let py = world.player_y.raw() as f32 / 16.0 - height / 2.0;
            push_texture_quad(&mut all_sprite_vertices, px, py, width, height);
        }
        let player_end = all_sprite_vertices.len();

        // Write all sprite vertices
        if !all_sprite_vertices.is_empty() {
            let write_len = all_sprite_vertices.len().min(MAX_SOLID_VERTICES);
            self.queue.write_buffer(
                &self.sprite_vertex_buffer,
                0,
                bytemuck::cast_slice(&all_sprite_vertices[..write_len]),
            );
        }

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
            ) && !background_vertices.is_empty()
            {
                let count = background_vertices.len().min(MAX_BACKGROUND_VERTICES) as u32;
                pass.set_pipeline(&self.texture_pipeline);
                pass.set_bind_group(0, &self.background_bind_groups[background_index], &[]);
                pass.set_vertex_buffer(0, self.background_vertex_buffer.slice(..));
                pass.draw(0..count, 0..1);

                // 2. Draw active sprites
                if world.state != GameState::StageClear {
                    pass.set_pipeline(&self.texture_pipeline);
                    pass.set_vertex_buffer(0, self.sprite_vertex_buffer.slice(..));

                    for (variant, &(start, end)) in enemy_ranges.iter().enumerate() {
                        if start < end {
                            pass.set_bind_group(
                                0,
                                &self.enemy_bind_groups[stage_index][variant][anim_frame],
                                &[],
                            );
                            pass.draw(start as u32..end as u32, 0..1);
                        }
                    }
                    if boss_start < boss_end {
                        pass.set_bind_group(0, &self.boss_bind_groups[boss_anim], &[]);
                        pass.draw(boss_start as u32..boss_end as u32, 0..1);
                    }
                    if item_start < item_end {
                        pass.set_bind_group(0, &self.item_bind_groups[anim_frame], &[]);
                        pass.draw(item_start as u32..item_end as u32, 0..1);
                    }
                    for (variant, &(start, end)) in player_bullet_ranges.iter().enumerate() {
                        if start < end {
                            pass.set_bind_group(
                                0,
                                &self.player_bullet_bind_groups[variant][player_anim],
                                &[],
                            );
                            pass.draw(start as u32..end as u32, 0..1);
                        }
                    }
                    for (variant, &(start, end)) in enemy_bullet_ranges.iter().enumerate() {
                        if start < end {
                            pass.set_bind_group(
                                0,
                                &self.enemy_bullet_bind_groups[variant][anim_frame],
                                &[],
                            );
                            pass.draw(start as u32..end as u32, 0..1);
                        }
                    }
                    if player_start < player_end {
                        let player_level = usize::from(world.growth_level.min(4));
                        pass.set_bind_group(
                            0,
                            &self.player_bind_groups[player_level][player_anim],
                            &[],
                        );
                        pass.draw(player_start as u32..player_end as u32, 0..1);
                    }
                }
            }

            // 3. Draw solid vertices (HUD, UI, Effects, Name Entry, Rankings)
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

pub fn text_width(text: &str, scale: f32) -> f32 {
    let char_count = text.chars().filter(|&ch| ch != '\n').count();
    char_count as f32 * 6.0 * scale
}

pub fn text_height(scale: f32) -> f32 {
    7.0 * scale
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

pub fn push_text_centered_x(
    vertices: &mut Vec<Vertex>,
    y: f32,
    text: &str,
    color: [f32; 3],
    scale: f32,
) {
    let width = text_width(text, scale);
    let x = (LOGICAL_WIDTH - width) / 2.0;
    push_text_colored(vertices, x, y, text, color, scale);
}

pub fn push_text_centered(
    vertices: &mut Vec<Vertex>,
    center_y: f32,
    text: &str,
    color: [f32; 3],
    scale: f32,
) {
    let width = text_width(text, scale);
    let height = text_height(scale);
    let x = (LOGICAL_WIDTH - width) / 2.0;
    let y = center_y - height / 2.0;
    push_text_colored(vertices, x, y, text, color, scale);
}

pub fn build_world_vertices(world: &World, vertices: &mut Vec<Vertex>) {
    match world.state {
        GameState::Title => {
            let center_y = LOGICAL_HEIGHT / 2.0;
            push_text_centered(vertices, center_y, "RUST SHOOTING", [0.3, 0.8, 1.0], 4.0);
            if (world.frame / 20).is_multiple_of(2) {
                push_text_centered_x(
                    vertices,
                    center_y + 80.0,
                    "PRESS SPACE TO START",
                    [1.0, 1.0, 1.0],
                    1.6,
                );
            }
            push_text_centered_x(
                vertices,
                LOGICAL_HEIGHT - 40.0,
                "COPYRIGHT (C) 2026 RUST SHOOTING TEAM",
                [0.5, 0.5, 0.6],
                0.9,
            );
        }
        GameState::Demo => {
            push_text_centered_x(vertices, 40.0, "TOP 10 RANKINGS", [1.0, 0.85, 0.2], 2.5);
            for (i, entry) in world.ranking.entries.iter().enumerate() {
                let y = 90.0 + i as f32 * 36.0;
                let rank_color = match i {
                    0 => [1.0, 0.85, 0.2],
                    1 => [0.85, 0.85, 0.9],
                    2 => [0.8, 0.55, 0.3],
                    _ => [0.7, 0.8, 0.9],
                };
                let text = format!("{:2}.   {}   {:8}", i + 1, entry.name_str(), entry.score);
                push_text_centered_x(vertices, y, &text, rank_color, 1.6);
            }
            if (world.frame / 20).is_multiple_of(2) {
                push_text_centered_x(
                    vertices,
                    LOGICAL_HEIGHT - 60.0,
                    "PRESS SPACE TO START",
                    [1.0, 1.0, 1.0],
                    1.6,
                );
            }
        }
        GameState::NameEntry => {
            push_text_centered_x(vertices, 40.0, "NAME ENTRY", [1.0, 0.85, 0.2], 2.5);
            let score_text = format!("SCORE: {:08}", world.score);
            push_text_centered_x(vertices, 85.0, &score_text, [1.0, 1.0, 1.0], 1.6);

            let name_str = std::str::from_utf8(&world.name_entry.name).unwrap_or("???");
            let name_display = format!("[ {} ]", name_str);
            push_text_centered_x(vertices, 125.0, &name_display, [0.3, 0.9, 1.0], 2.5);

            // Matrix rendering (7 cols x 6 rows)
            let col_w = 46.0;
            let row_h = 38.0;
            let matrix_w = 7.0 * col_w;
            let start_x = (LOGICAL_WIDTH - matrix_w) / 2.0 + 4.0;
            let start_y = 200.0;

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

            push_text_centered_x(
                vertices,
                LOGICAL_HEIGHT - 70.0,
                "MOVE: ARROWS   SELECT: SPACE",
                [0.7, 0.7, 0.7],
                1.2,
            );
        }
        GameState::GameOver => {
            push_text_centered(
                vertices,
                LOGICAL_HEIGHT / 2.0,
                "GAME OVER",
                [0.9, 0.2, 0.2],
                3.5,
            );
            let score_text = format!("FINAL SCORE: {:08}", world.score);
            push_text_centered_x(
                vertices,
                LOGICAL_HEIGHT / 2.0 + 70.0,
                &score_text,
                [1.0, 1.0, 1.0],
                1.5,
            );
        }
        GameState::Ending => {
            let base_y = LOGICAL_HEIGHT * 0.32;
            push_text_centered_x(
                vertices,
                base_y,
                "ALL STAGES CLEARED!",
                [1.0, 0.85, 0.2],
                2.5,
            );
            push_text_centered_x(
                vertices,
                base_y + 50.0,
                "CONGRATULATIONS!",
                [0.3, 0.9, 0.5],
                2.5,
            );
            let score_text = format!("FINAL SCORE: {:08}", world.score);
            push_text_centered_x(vertices, base_y + 110.0, &score_text, [1.0, 1.0, 1.0], 1.6);
        }
        GameState::Playing | GameState::StageIntro | GameState::StageClear => {
            // Destruction and growth effects (rendered with solid vertices)
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
                push_text_colored(vertices, 175.0, 260.0, &text, [1.0, 1.0, 1.0], 2.5);
                let subtext = match world.stage_id {
                    1 => "ASTEROID BELT",
                    2 => "SPACE STATION",
                    3 => "NEBULA STORM",
                    4 => "CYBER MATRIX",
                    5 => "FORTRESS APPROACH",
                    6 => "FINAL: CORE INTRUSION",
                    _ => "UNKNOWN REGION",
                };
                push_text_colored(vertices, 140.0, 310.0, subtext, [0.3, 0.85, 1.0], 1.5);
                push_text_colored(vertices, 195.0, 350.0, "READY!", [1.0, 0.85, 0.2], 2.0);
            } else if world.state == GameState::StageClear {
                let text = format!("STAGE {:02} CLEAR!", world.stage_id);
                push_text_colored(vertices, 125.0, 280.0, &text, [1.0, 0.85, 0.2], 2.2);
            }

            // HUD rendering at bottom line
            append_hud_vertices(world, vertices);
        }
    }
}

fn push_texture_quad(vertices: &mut Vec<TextureVertex>, x: f32, y: f32, width: f32, height: f32) {
    let left = x / LOGICAL_WIDTH * 2.0 - 1.0;
    let right = (x + width) / LOGICAL_WIDTH * 2.0 - 1.0;
    let top = 1.0 - y / LOGICAL_HEIGHT * 2.0;
    let bottom = 1.0 - (y + height) / LOGICAL_HEIGHT * 2.0;
    vertices.extend([
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
    ]);
}

fn append_hud_vertices(world: &World, vertices: &mut Vec<Vertex>) {
    push_rect_colored(vertices, 8.0, HUD_Y + 3.0, 120.0, 10.0, [0.2, 0.15, 0.15]);
    let hp_ratio = (world.hp as f32 / 100.0).clamp(0.0, 1.0);
    let hp_color = if world.hp > 50 {
        [0.15, 0.9, 0.3]
    } else if world.hp > 25 {
        [0.95, 0.85, 0.15]
    } else {
        [0.95, 0.2, 0.2]
    };
    push_rect_colored(vertices, 8.0, HUD_Y + 3.0, 120.0 * hp_ratio, 10.0, hp_color);

    // 2. Score
    let score_str = format!("{:08}", world.score);
    push_text_colored(
        vertices,
        140.0,
        HUD_Y + 4.0,
        &score_str,
        [1.0, 1.0, 1.0],
        1.0,
    );

    // 3. Lives
    let lives_str = format!("L:{:02}", world.lives);
    push_text_colored(
        vertices,
        280.0,
        HUD_Y + 4.0,
        &lives_str,
        [0.3, 0.85, 1.0],
        1.0,
    );

    // 4. Stock / Growth
    let stock_str = if world.growth_level < 4 {
        format!("LV:{}", world.growth_level)
    } else {
        format!("ST:+{}", world.recovery_stock)
    };
    push_text_colored(
        vertices,
        380.0,
        HUD_Y + 4.0,
        &stock_str,
        [1.0, 0.85, 0.2],
        1.0,
    );

    // 5. Stage number at the lower right
    let stage_str = format!("S:{:02}", world.stage_id);
    push_text_colored(
        vertices,
        440.0,
        HUD_Y + 4.0,
        &stage_str,
        [0.95, 0.95, 1.0],
        1.0,
    );
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

fn background_tile_vertices(scroll_y_raw: i16, tile_map: &TileMap) -> Vec<TextureVertex> {
    let mut vertices = Vec::with_capacity(MAX_BACKGROUND_VERTICES);
    let scroll_y = f32::from(scroll_y_raw) / 16.0;
    let tile_map_width = tile_map.width.max(1);
    let tile_map_height = tile_map.height.max(1);

    let first_row = (-scroll_y / TILE_SIZE).floor() as i32;
    for row_idx in 0..TILE_ROWS_VISIBLE {
        let row = first_row + row_idx as i32;
        let map_row = row.rem_euclid(tile_map_height as i32);
        let screen_y = row as f32 * TILE_SIZE + scroll_y;
        for column in 0..TILE_COLUMNS {
            let tile_id = tile_map.tile_id(map_row, (column as u16) % tile_map_width);
            if tile_id == 0 {
                continue;
            }
            let atlas_x = (tile_id % 4) as f32;
            let atlas_y = (tile_id / 4) as f32;
            let u0 = atlas_x * 0.25;
            let v0 = atlas_y * 0.25;
            let u1 = u0 + 0.25;
            let v1 = v0 + 0.25;
            let left = column as f32 * TILE_SIZE / LOGICAL_WIDTH * 2.0 - 1.0;
            let right = (column as f32 * TILE_SIZE + TILE_SIZE) / LOGICAL_WIDTH * 2.0 - 1.0;
            let top = 1.0 - screen_y / LOGICAL_HEIGHT * 2.0;
            let bottom = 1.0 - (screen_y + TILE_SIZE) / LOGICAL_HEIGHT * 2.0;
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
    PhysicalSize::new(u32::from(SCREEN_WIDTH) * 2, u32::from(SCREEN_HEIGHT) * 2)
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
