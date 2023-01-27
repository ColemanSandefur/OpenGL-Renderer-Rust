use glium::texture::RawImage2d;
use glium::{pixel_buffer::PixelBuffer, texture::CubeLayer};
use image::DynamicImage;
use std::error::Error;

type Pixel = (u8, u8, u8, u8);

pub struct CubemapLayoutBuffer<'a> {
    pub x_pos: &'a PixelBuffer<Pixel>,
    pub x_neg: &'a PixelBuffer<Pixel>,
    pub y_pos: &'a PixelBuffer<Pixel>,
    pub y_neg: &'a PixelBuffer<Pixel>,
    pub z_pos: &'a PixelBuffer<Pixel>,
    pub z_neg: &'a PixelBuffer<Pixel>,
}

impl<'a> CubemapLayoutBuffer<'a> {
    pub fn get_from_gl_enum(&self, layer: CubeLayer) -> &'a PixelBuffer<Pixel> {
        match layer {
            CubeLayer::PositiveX => self.x_pos,
            CubeLayer::NegativeX => self.x_neg,
            CubeLayer::PositiveY => self.y_pos,
            CubeLayer::NegativeY => self.y_neg,
            CubeLayer::PositiveZ => self.z_pos,
            CubeLayer::NegativeZ => self.z_neg,
        }
    }

    pub fn to_cubemap(&self) {}
}

pub struct CubemapLayout {
    pub x_pos: DynamicImage,
    pub x_neg: DynamicImage,
    pub y_pos: DynamicImage,
    pub y_neg: DynamicImage,
    pub z_pos: DynamicImage,
    pub z_neg: DynamicImage,
}

impl CubemapLayout {
    pub fn get_from_gl_enum(&self, layer: CubeLayer) -> &DynamicImage {
        match layer {
            CubeLayer::PositiveX => &self.x_pos,
            CubeLayer::NegativeX => &self.x_neg,
            CubeLayer::PositiveY => &self.y_pos,
            CubeLayer::NegativeY => &self.y_neg,
            CubeLayer::PositiveZ => &self.z_pos,
            CubeLayer::NegativeZ => &self.z_neg,
        }
    }

    pub fn to_cubemap(self) {}
}

impl<'a> TryFrom<CubemapLayoutBuffer<'a>> for CubemapLayout {
    type Error = Box<dyn std::error::Error>;

    fn try_from(other: CubemapLayoutBuffer) -> Result<Self, Self::Error> {
        // quick conversion function from PixelBuffer to DynamicImage
        let convert_tex = |texture: &PixelBuffer<Pixel>| -> Result<DynamicImage, Self::Error> {
            let data: RawImage2d<'_, u8> = texture.read_as_texture_2d()?;

            let (width, height) = (data.width, data.height);

            let image =
                image::ImageBuffer::from_raw(width as u32, height as u32, data.data.into_owned())
                    .ok_or("failed to create image")?;

            let image = image::DynamicImage::ImageRgba8(image);

            Ok(image)
        };

        Ok(Self {
            x_pos: convert_tex(other.x_pos)?,
            x_neg: convert_tex(other.x_neg)?,
            y_pos: convert_tex(other.y_pos)?,
            y_neg: convert_tex(other.y_neg)?,
            z_pos: convert_tex(other.z_pos)?,
            z_neg: convert_tex(other.z_neg)?,
        })
    }
}
