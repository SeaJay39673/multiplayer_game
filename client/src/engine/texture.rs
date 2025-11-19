use std::{collections::HashMap, hash::Hash, sync::{Arc, LazyLock, RwLock}};

use anyhow::{Result, anyhow};
use shared::TileType;
use wgpu::{AddressMode, Device, Extent3d, FilterMode, Origin3d, Queue, Sampler, SamplerDescriptor, TexelCopyBufferLayout, TexelCopyTextureInfo, TextureAspect, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView, TextureViewDescriptor};

pub struct Texture {
    x_count: u8,
    y_count: u8,
    pub view: TextureView,
    pub sampler: Sampler,
}

impl Texture {
    pub fn from_file(
        device: &Device,
        queue: &Queue,
        path: &str,
        x_count: u8,
        y_count: u8,
    ) -> Result<Self> {
        let img = image::open(path)?.to_rgba8();
        let (width, height) = img.dimensions();

        let texture_size = Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&TextureDescriptor {
            label: Some("Texture"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            &img,
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(4 * height),
            },
            texture_size,
        );

        let texture_view = texture.create_view(&TextureViewDescriptor::default());
        let sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });

        Ok(Self {
            view: texture_view,
            sampler,
            x_count,
            y_count,
        })
    }

    pub fn from_color(device: &Device, queue: &Queue, color: [u8; 4]) -> Self {
        let size = Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&TextureDescriptor {
            label: Some("Fallback Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            &color,
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: Some(1),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());

        Self {
            view,
            sampler,
            x_count: 1,
            y_count: 1,
        }
    }
    
}


#[derive(Clone)]
pub struct TexInfo {
    pub texture: Arc<Texture>,
    pub index: [u8; 2],
}

impl TexInfo {
    pub fn map_uv(&self, coords: [f32; 2]) -> [f32;2] {
        let Texture {x_count, y_count, ..} = self.texture.as_ref();
        let x_map: f32 = (coords[0] / *x_count as f32) + (self.index[0] as f32 / *x_count as f32);
        let y_map: f32 = (coords[1] / *y_count as f32) + (self.index[1] as f32 / * y_count as f32);
        [x_map, y_map]
    }
}

pub static TEXTURE_MAP: LazyLock<RwLock<HashMap<TileType, TexInfo>>> = LazyLock::new(|| RwLock::new(HashMap::new()));


#[derive(Hash, PartialEq, Eq)]
pub enum PlayerTexture {
    North,
    South,
    East,
    West,
}

pub static PLAYER_TEXTURES: LazyLock<RwLock<HashMap<PlayerTexture, TexInfo>>> = LazyLock::new(|| RwLock::new(HashMap::new()));

pub fn init_textures(device: &Device, queue: &Queue) -> Result<()> {
    let mut textures = TEXTURE_MAP
        .write()
        .map_err(|e| anyhow!("Could not access TEXTURE_MAP for writing: {}", e))?;
    let texture = Arc::new(Texture::from_file(device, queue, "src/assets/isometric.png", 8,8)?);
    textures.insert(TileType::GrassBlock, TexInfo { texture: texture.clone(), index: [0,0] });
    textures.insert(TileType::GrassSlopeL, TexInfo { texture: texture.clone(), index: [1,0] });
    textures.insert(TileType::GrassSlopeR, TexInfo { texture: texture.clone(), index: [2,0] });

    let mut players = PLAYER_TEXTURES
        .write()
        .map_err(|e| anyhow!("Could not access PLAYER_TEXTURES for writing: {e}"))?;

    let sprites = Arc::new(Texture::from_file(device, queue, "src/assets/sprites.png", 8, 12)?);
    players.insert(PlayerTexture::South, TexInfo { texture: sprites.clone(), index: [4,3] });

    Ok(())
}