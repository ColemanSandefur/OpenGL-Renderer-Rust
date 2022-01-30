use std::borrow::Cow;
use std::path::PathBuf;
use std::error::Error;

use glium::Texture2d;
use glium::backend::Facade;
use glium::texture::Texture2dDataSink;
use glium::texture::{Texture2dDataSource, RawImage2d};
use image::io::Reader as ImageReader;
use image::{DynamicImage, GenericImageView, ImageBuffer};

pub struct TextureLoader {}

impl TextureLoader {
    pub fn from_fs(facade: &impl Facade, path: &PathBuf) -> Result<Texture2d, Box<dyn Error>> {
        Self::from_image(facade, ImageLoader::load_from_fs(path)?)
    }

    pub fn from_image(facade: &impl Facade, image: Image) -> Result<Texture2d, Box<dyn Error>> {
        Ok(Texture2d::new(facade, image)?)
    }

    pub fn from_memory_rgb8(facade: &impl Facade, buffer: Vec<u8>, width: u32, height: u32) -> Result<Texture2d, Box<dyn Error>>{
        Ok(
            Texture2d::new(
                facade,
                RawImage2d::from_raw_rgb(buffer, (width, height))
            )?
        )
    }

    pub fn from_memory_rgbf32(facade: &impl Facade, buffer: Vec<f32>, width: u32, height: u32) -> Result<Texture2d, Box<dyn Error>> {
        let buffer_grouped = buffer.chunks_exact(3).map(|chunk| {
            return (chunk[0], chunk[1], chunk[2])
        });

        Ok(
            Texture2d::new(
                facade,
                RawImage2d::from_raw(Cow::from_iter(buffer_grouped), width, height)
            )?
        )
    }
}

// Still deciding on whether to use a custom image struct or just use DynamicImage,
// for now I will just wrap a DynamicImage
pub struct Image {
    image: DynamicImage,
}

impl<'a> Texture2dDataSource<'a> for Image {
    type Data = u8;

    fn into_raw(self) -> RawImage2d<'a, Self::Data> {
        let dimensions = self.image.dimensions();
        let data = self.image.into_rgba8().into_raw();

        RawImage2d::from_raw_rgba(data, dimensions)
    }
}

impl From<DynamicImage> for Image {
    fn from(d: DynamicImage) -> Self {
        Self {
            image: d
        }
    }
}

pub struct ImageLoader {}

impl ImageLoader {
    pub fn load_from_fs(path: &PathBuf) -> Result<Image, Box<dyn Error>> {
        let dynamic = ImageReader::open(&path)?.decode()?;

        Ok(Image {
            image: dynamic,
        })
    }

    pub fn from_memory_u8(data: Vec<u8>, width: u32, height: u32) -> Result<Image, Box<dyn Error>>{
        let img_buffer = ImageBuffer::from_raw(width, height, data).ok_or("Container was not large enough")?;

        Ok(
            Image {
                image: DynamicImage::ImageRgb8(img_buffer)
            }
        )
    }
}

// To be implemented
pub struct TextureSaver {}

// To be implemented
pub struct ImageSaver {}
