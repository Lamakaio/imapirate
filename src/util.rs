use std::hash::Hash;

use bevy::{
    prelude::{Texture, TextureAtlas},
    render::texture::TextureFormat,
};
use density_mesh_core::prelude::{DensityMap, DensityMeshGenerator, GenerateDensityMeshSettings};
use image::Pixel;
use parry2d::{math::Point, shape::TriMesh};
use seahash::SeaHasher;

//a struct to provide a seeded hasher. I doesn't expose the underlying hasher intentionnally to make sure nothing breaks it.
pub struct SeededHasher {
    hasher: SeaHasher,
}

impl SeededHasher {
    pub fn new(seed: &str) -> SeededHasher {
        let mut hasher = SeaHasher::new();
        seed.hash(&mut hasher);
        SeededHasher { hasher }
    }

    pub fn get_hasher(&self) -> SeaHasher {
        self.hasher
    }
}

pub(crate) fn texture_to_image(texture: &Texture) -> Option<image::DynamicImage> {
    match texture.format {
        TextureFormat::R8Unorm => image::ImageBuffer::from_raw(
            texture.size.width,
            texture.size.height,
            texture.data.clone(),
        )
        .map(image::DynamicImage::ImageLuma8),
        TextureFormat::Rg8Unorm => image::ImageBuffer::from_raw(
            texture.size.width,
            texture.size.height,
            texture.data.clone(),
        )
        .map(image::DynamicImage::ImageLumaA8),
        TextureFormat::Rgba8UnormSrgb => image::ImageBuffer::from_raw(
            texture.size.width,
            texture.size.height,
            texture.data.clone(),
        )
        .map(image::DynamicImage::ImageRgba8),
        TextureFormat::Bgra8UnormSrgb => image::ImageBuffer::from_raw(
            texture.size.width,
            texture.size.height,
            texture.data.clone(),
        )
        .map(image::DynamicImage::ImageBgra8),
        _ => None,
    }
}
pub fn texture_atlas_to_trimeshes(
    atlas: &TextureAtlas,
    texture: &Texture,
    scale: f32,
) -> Vec<TriMesh> {
    let mut meshes = Vec::new();
    for rect in &atlas.textures {
        let size = rect.max - rect.min;
        let image = texture_to_image(texture)
            .unwrap()
            .crop(
                rect.min.x as u32,
                rect.min.y as u32,
                size.x as u32,
                size.y as u32,
            )
            .into_luma8()
            .pixels()
            .map(|p| p.channels()[0])
            .collect();
        let map = DensityMap::new(size.x as usize, size.y as usize, 1, image).unwrap();

        let settings = GenerateDensityMeshSettings {
            points_separation: 4.0.into(),
            keep_invisible_triangles: false,
            ..Default::default()
        };
        let mut generator = DensityMeshGenerator::new(vec![], map, settings.clone());
        generator.process_wait().expect("Failed to process image");
        let mut mesh = generator.into_mesh().unwrap();
        let vertices = mesh
            .points
            .drain(..)
            .map(|coord| {
                Point::new(
                    (coord.x - size.x / 2.) * scale,
                    (coord.y - size.y / 2.) * scale,
                )
            })
            .collect();
        let indices = mesh
            .triangles
            .drain(..)
            .map(|triangle| [triangle.a as u32, triangle.b as u32, triangle.c as u32])
            .collect();
        let trimesh = TriMesh::new(vertices, indices);
        meshes.push(trimesh)
    }
    meshes
}
