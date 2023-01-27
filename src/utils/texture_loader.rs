use glium::texture::Texture2dDataSink;
use glium::Texture2d;
use glium::{backend::Facade, texture::RawImage2d};
use image::codecs::hdr::HdrDecoder;
use image::io::Reader as ImageReader;
use rayon::prelude::IntoParallelRefIterator;
use rayon::{prelude::ParallelIterator, slice::ParallelSlice};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::{borrow::Cow, error::Error};

pub struct TextureLoader {}

impl TextureLoader {
    pub fn from_memory_f32(
        facade: &impl Facade,
        buffer: &[f32],
        width: u32,
        height: u32,
    ) -> Result<Texture2d, Box<dyn Error>> {
        let buffer_grouped = buffer
            .par_chunks_exact(3)
            .map(|chunk| return (chunk[0], chunk[1], chunk[2]))
            .collect::<Vec<_>>();

        Ok(Texture2d::with_format(
            facade,
            RawImage2d::from_raw(Cow::from(buffer_grouped), width, height),
            glium::texture::UncompressedFloatFormat::F16F16F16,
            glium::texture::MipmapsOption::NoMipmap,
        )?)
    }

    pub fn from_fs(
        facade: &impl Facade,
        path: impl AsRef<Path>,
    ) -> Result<Texture2d, Box<dyn Error>> {
        let img = ImageReader::open(path)?.decode()?.into_rgb32f();
        let (width, height) = img.dimensions();
        let img_data = img.into_raw();

        Self::from_memory_f32(facade, &img_data, width, height)
    }

    pub fn from_fs_hdr(
        facade: &impl Facade,
        path: impl AsRef<Path>,
    ) -> Result<Texture2d, Box<dyn Error>> {
        let buf = BufReader::new(File::open(path)?);

        let hdr_image = HdrDecoder::new(buf)?;

        let width = hdr_image.metadata().width;
        let height = hdr_image.metadata().height;

        let mut pixels = hdr_image.read_image_hdr()?;

        for w in 0..width / 2 {
            for h in 0..height {
                let index = (w + (h * width)) as usize;
                let index2 = ((h * width) + (width - w - 1)) as usize;
                let p = pixels[index];
                let p2 = pixels[index2];

                pixels[index] = p2;
                pixels[index2] = p;
            }
        }

        let data = pixels
            .par_iter()
            .flat_map(|pixel| return pixel.0)
            .collect::<Vec<_>>();

        Self::from_memory_f32(facade, &data, width, height)
    }
}
