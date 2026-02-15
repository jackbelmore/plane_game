use bevy::{
    prelude::*,
    image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor},
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
};
use rand::Rng;

pub fn create_grass_texture() -> Image {
    const WIDTH: usize = 1024;
    const HEIGHT: usize = 1024;
    
    // 1. Generate Noise Data
    // We'll use a simple "white noise" with a green tint, but at a higher resolution
    // to simulate grass blades when viewed from a distance.
    let mut data = Vec::with_capacity(WIDTH * HEIGHT * 4);
    let mut rng = rand::thread_rng();

    for _y in 0..HEIGHT {
        for _x in 0..WIDTH {
            // Base Green: R=30-50, G=100-160, B=20-40
            // We vary the intensity to create "blades"
            let intensity = rng.gen_range(0.8..1.2);
            
            let r = (40.0 * intensity) as u8;
            let g = (130.0 * intensity) as u8;
            let b = (30.0 * intensity) as u8;
            
            data.push(r);
            data.push(g);
            data.push(b);
            data.push(255); // Alpha
        }
    }

    // 2. Create Image
    let mut image = Image::new(
        Extent3d {
            width: WIDTH as u32,
            height: HEIGHT as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    );

    // 3. Set Sampler to Repeat (Tiling)
    // Critical for terrain: The texture must tile seamlessly
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        address_mode_w: ImageAddressMode::Repeat,
        ..default()
    });

    image
}
