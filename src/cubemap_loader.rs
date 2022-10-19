use gl::types::GLenum;
use glium::backend::Facade;
use glium::texture::Cubemap;
use glium::texture::SrgbCubemap;
use glium::texture::{Dimensions, MipmapsOption};
use glium::uniforms::AsUniformValue;
use image::io::Reader as ImageReader;
use image::DynamicImage;
use image::GenericImageView;
use std::error::Error;
use std::ops::Index;
use std::path::PathBuf;
use std::ptr::null;

/// Different Cubemap types.
///
/// Intended to be used as the return type from [`CubemapLoader`] since it might generate different
/// cubemaps based on the input file.
pub enum CubemapType {
    Cubemap(Cubemap),
    SrgbCubemap(SrgbCubemap),
}

impl From<Cubemap> for CubemapType {
    fn from(c: Cubemap) -> Self {
        Self::Cubemap(c)
    }
}
impl From<SrgbCubemap> for CubemapType {
    fn from(c: SrgbCubemap) -> Self {
        Self::SrgbCubemap(c)
    }
}

impl AsUniformValue for &CubemapType {
    fn as_uniform_value(&self) -> glium::uniforms::UniformValue<'_> {
        match self {
            CubemapType::Cubemap(c) => c.as_uniform_value(),
            CubemapType::SrgbCubemap(c) => c.as_uniform_value(),
        }
    }
}

impl AsUniformValue for CubemapType {
    fn as_uniform_value(&self) -> glium::uniforms::UniformValue<'_> {
        match self {
            CubemapType::Cubemap(c) => c.as_uniform_value(),
            CubemapType::SrgbCubemap(c) => c.as_uniform_value(),
        }
    }
}

/// Loads an OpenGL [`Cubemap`] since [`glium`] doesn't provide a convenient way to create one.
///
/// When necessary, each method should have specific instructions on how to use it. This contains
/// unsafe code that is not rigorously tested, so it is likely that there will be some unexpected
/// behavior
pub struct CubemapLoader {}

impl CubemapLoader {
    fn create_paths(mut directory: PathBuf, extension: &str) -> Vec<PathBuf> {
        directory.push(format!("right.{}", extension));
        let path: PathBuf = directory.into();

        vec![
            path.with_file_name(format!("right.{}", extension)),
            path.with_file_name(format!("left.{}", extension)),
            path.with_file_name(format!("top.{}", extension)),
            path.with_file_name(format!("bottom.{}", extension)),
            path.with_file_name(format!("front.{}", extension)),
            path.with_file_name(format!("back.{}", extension)),
        ]
    }

    /// Loads the Cubemap from the directory provided.
    ///
    /// It will look for files named "right", "left", "top", "bottom", "front", "back" (with the
    /// provided extension) in the provided directory.
    pub fn load_from_fs(
        directory: PathBuf,
        extension: &str,
        facade: &impl Facade,
    ) -> Result<CubemapType, Box<dyn Error>> {
        let paths = Self::create_paths(directory, extension);
        let mut images = Vec::new();
        for path in paths {
            let image = ImageReader::open(&path)?.decode()?;

            images.push(image);
        }

        let orientation = CubeOrientation::from_array(images).unwrap();

        let cubemap = Self::load_cubemap(facade, vec![orientation]);

        Ok(CubemapType::Cubemap(cubemap))
    }

    /// Loads the Cubemap from the directory provided.
    ///
    /// It will look for folders named a number (ex. 0, 1, 2, 3) corresponding to mipmap layer
    /// (ex. 0 is the first layer, 1 is the second, ...) and in each folder it looks for files
    /// named "right", "left", "top", "bottom", "front", "back" (with the
    /// provided extension) in the provided directory.
    ///
    /// It will return Err if there were no cubemaps were found.
    pub fn load_mips_fs(
        mut directory: PathBuf,
        extension: &str,
        facade: &impl Facade,
    ) -> Result<CubemapType, Box<dyn Error>> {
        if directory.is_file() {
            directory.pop();
        }

        let mut cubes = Vec::new();
        let mut level = 0;
        while directory.join(format!("{}", level)).exists() {
            let sub_directory = directory.join(format!("{}", level));

            let paths = Self::create_paths(sub_directory, extension);
            let mut images = Vec::new();
            for path in paths {
                let image = ImageReader::open(path)?.decode()?;
                images.push(image);
            }

            let orientation = CubeOrientation::from_array(images)?;

            cubes.push(orientation);

            level += 1;
        }

        if cubes.len() == 0 {
            return Err(format!(
                "Unable to find any cubemaps in \"{}\".",
                directory.as_os_str().to_str().unwrap()
            )
            .into());
        }

        let cubemap = Self::load_cubemap(facade, cubes);

        Ok(CubemapType::Cubemap(cubemap))
    }

    /// Loads a cubemap from memory
    ///
    /// cubes is basically a vector that holds each mipmap of the cubemap. So the first element is
    /// the main texture, second element is the first mipmap, etc.
    pub fn load_cubemap(facade: &impl Facade, mut cubes: Vec<CubeOrientation>) -> Cubemap {
        unsafe {
            let mut cubemap_id: u32 = 0;
            let num_mips = cubes.len() - 1;
            let largest_side = cubes[0][0].dimensions().0;

            gl::GenTextures(1, &mut cubemap_id);
            gl::BindTexture(gl::TEXTURE_CUBE_MAP, cubemap_id);

            gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_BASE_LEVEL, 0);
            gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MAX_LEVEL, num_mips as i32);

            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_MIN_FILTER,
                gl::LINEAR_MIPMAP_LINEAR as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_MAG_FILTER,
                gl::LINEAR as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_WRAP_S,
                gl::CLAMP_TO_EDGE as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_WRAP_T,
                gl::CLAMP_TO_EDGE as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_WRAP_R,
                gl::CLAMP_TO_EDGE as i32,
            );

            // generate textures
            for layer in 0..cubes.len() {
                let cube_orientation = cubes.remove(0);
                let dimensions = cube_orientation.front.dimensions();

                for side in 0..6 {
                    let mut pixels = cube_orientation[side].to_rgb8();

                    let flat_samples = pixels.as_flat_samples_mut();
                    let slice: &[u8] = flat_samples.as_slice();
                    let ptr: *const core::ffi::c_void =
                        slice as *const _ as *const core::ffi::c_void;
                    gl::TexImage2D(
                        gl::TEXTURE_CUBE_MAP_POSITIVE_X + side as u32,
                        layer as i32,
                        gl::RGB16F as i32,
                        dimensions.0 as i32,
                        dimensions.1 as i32,
                        0,
                        gl::RGB,
                        gl::UNSIGNED_BYTE,
                        ptr,
                    );
                }
            }

            // give the cubemap to glium::Cubemap
            let cubemap = Cubemap::from_id(
                facade,
                glium::texture::UncompressedFloatFormat::U8U8U8,
                cubemap_id,
                true,
                MipmapsOption::EmptyMipmapsMax(num_mips as u32),
                Dimensions::Cubemap {
                    dimension: largest_side,
                },
            );

            cubemap
        }
    }
}
/// Holds the sides of a cubemap
///
/// Mostly used for loading a cubemap via [`CubemapLoader`]. Makes sure that the textures are
/// loaded in the right direction.
pub struct CubeOrientation {
    pub front: DynamicImage,
    pub back: DynamicImage,
    pub left: DynamicImage,
    pub right: DynamicImage,
    pub top: DynamicImage,
    pub bottom: DynamicImage,
}

impl CubeOrientation {
    pub fn new(
        right: DynamicImage,
        left: DynamicImage,
        top: DynamicImage,
        bottom: DynamicImage,
        front: DynamicImage,
        back: DynamicImage,
    ) -> Self {
        CubeOrientation {
            front,
            back,
            left,
            right,
            top,
            bottom,
        }
    }

    /// Loads from a buffer sized 6
    ///
    /// Images should be provided in the order of right, left, top, bottom, front, back ([`the same
    /// order as face assignment in opengl`](https://www.khronos.org/opengl/wiki/Cubemap_Texture#Creation)). I'd
    /// recommend using [`new`] as this can be error prone, but this is here for convenience.
    ///
    /// Returns `Err` when there weren't enough elements
    ///
    /// [`new`]: Self::new
    pub fn from_array(
        faces: impl IntoIterator<Item = DynamicImage>,
    ) -> Result<Self, Box<dyn Error>> {
        let mut iter = faces.into_iter();
        let error_msg = "Vector didn't have enough textures, it should have 6 elements.";
        Ok(CubeOrientation {
            right: iter.next().ok_or(error_msg)?,
            left: iter.next().ok_or(error_msg)?,
            top: iter.next().ok_or(error_msg)?,
            bottom: iter.next().ok_or(error_msg)?,
            front: iter.next().ok_or(error_msg)?,
            back: iter.next().ok_or(error_msg)?,
        })
    }

    pub fn get_from_gl_enum(&self, side: GLenum) -> Option<&DynamicImage> {
        match side {
            gl::TEXTURE_CUBE_MAP_POSITIVE_X => Some(&self.right),
            gl::TEXTURE_CUBE_MAP_NEGATIVE_X => Some(&self.left),
            gl::TEXTURE_CUBE_MAP_POSITIVE_Y => Some(&self.top),
            gl::TEXTURE_CUBE_MAP_NEGATIVE_Y => Some(&self.bottom),
            gl::TEXTURE_CUBE_MAP_POSITIVE_Z => Some(&self.front),
            gl::TEXTURE_CUBE_MAP_NEGATIVE_Z => Some(&self.back),
            _ => None,
        }
    }
}

/// Get access to the textures via indexing
///
/// panics when index > 5 because CubeOrientation can only hold 6 textures
impl Index<usize> for CubeOrientation {
    type Output = DynamicImage;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.right,
            1 => &self.left,
            2 => &self.top,
            3 => &self.bottom,
            4 => &self.front,
            5 => &self.back,
            _ => panic!(
                "Index was out of bounds, CubeOrientation always has 6 elements, you accessed {}",
                index
            ),
        }
    }
}
