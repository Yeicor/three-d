//!
//! Different types of textures used by the GPU to read from and write to.
//!
mod texture2d;
#[doc(inline)]
pub use texture2d::*;

mod texture_cube_map;
#[doc(inline)]
pub use texture_cube_map::*;

mod depth_target_texture2d;
#[doc(inline)]
pub use depth_target_texture2d::*;

mod texture2d_array;
#[doc(inline)]
pub use texture2d_array::*;

mod texture3d;
#[doc(inline)]
pub use texture3d::*;

mod depth_target_texture2d_array;
#[doc(inline)]
pub use depth_target_texture2d_array::*;

mod depth_target_texture_cube_map;
#[doc(inline)]
pub use depth_target_texture_cube_map::*;

use data_type::*;
pub use three_d_asset::texture::{
    Interpolation, Texture2D as CpuTexture, Texture3D as CpuTexture3D, TextureData, Wrapping,
};

/// The basic data type used for each channel of each pixel in a texture.
pub trait TextureDataType: DataType {}
impl TextureDataType for u8 {}
impl TextureDataType for f16 {}
impl TextureDataType for f32 {}

impl<T: TextureDataType + PrimitiveDataType> TextureDataType for Vector2<T> {}
impl<T: TextureDataType + PrimitiveDataType> TextureDataType for Vector3<T> {}
impl<T: TextureDataType + PrimitiveDataType> TextureDataType for Vector4<T> {}
impl<T: TextureDataType + PrimitiveDataType> TextureDataType for [T; 2] {}
impl<T: TextureDataType + PrimitiveDataType> TextureDataType for [T; 3] {}
impl<T: TextureDataType + PrimitiveDataType> TextureDataType for [T; 4] {}

impl TextureDataType for Color {}
impl TextureDataType for Quat {}

impl<T: TextureDataType + ?Sized> TextureDataType for &T {}

use crate::core::*;

// COMMON TEXTURE FUNCTIONS

fn generate(context: &Context) -> ThreeDResult<crate::context::Texture> {
    unsafe {
        Ok(context
            .create_texture()
            .map_err(|e| CoreError::TextureCreation(e))?)
    }
}

fn set_parameters(
    context: &Context,
    target: u32,
    min_filter: Interpolation,
    mag_filter: Interpolation,
    mip_map_filter: Option<Interpolation>,
    wrap_s: Wrapping,
    wrap_t: Wrapping,
    wrap_r: Option<Wrapping>,
) -> ThreeDResult<()> {
    unsafe {
        match mip_map_filter {
            None => context.tex_parameter_i32(
                target,
                crate::context::TEXTURE_MIN_FILTER,
                interpolation_from(min_filter),
            ),
            Some(Interpolation::Nearest) => {
                if min_filter == Interpolation::Nearest {
                    context.tex_parameter_i32(
                        target,
                        crate::context::TEXTURE_MIN_FILTER,
                        crate::context::NEAREST_MIPMAP_NEAREST as i32,
                    );
                } else {
                    context.tex_parameter_i32(
                        target,
                        crate::context::TEXTURE_MIN_FILTER,
                        crate::context::LINEAR_MIPMAP_NEAREST as i32,
                    )
                }
            }
            Some(Interpolation::Linear) => {
                if min_filter == Interpolation::Nearest {
                    context.tex_parameter_i32(
                        target,
                        crate::context::TEXTURE_MIN_FILTER,
                        crate::context::NEAREST_MIPMAP_LINEAR as i32,
                    );
                } else {
                    context.tex_parameter_i32(
                        target,
                        crate::context::TEXTURE_MIN_FILTER,
                        crate::context::LINEAR_MIPMAP_LINEAR as i32,
                    )
                }
            }
        }
        context.tex_parameter_i32(
            target,
            crate::context::TEXTURE_MAG_FILTER,
            interpolation_from(mag_filter),
        );
        context.tex_parameter_i32(
            target,
            crate::context::TEXTURE_WRAP_S,
            wrapping_from(wrap_s),
        );
        context.tex_parameter_i32(
            target,
            crate::context::TEXTURE_WRAP_T,
            wrapping_from(wrap_t),
        );
        if let Some(r) = wrap_r {
            context.tex_parameter_i32(target, crate::context::TEXTURE_WRAP_R, wrapping_from(r));
        }
    }
    context.error_check()
}

fn calculate_number_of_mip_maps(
    mip_map_filter: Option<Interpolation>,
    width: u32,
    height: u32,
    depth: Option<u32>,
) -> u32 {
    if mip_map_filter.is_some()
        && width == height
        && depth.map(|d| d == width).unwrap_or(true)
        && width.is_power_of_two()
    {
        (width as f64).log2() as u32 + 1
    } else {
        1
    }
}

fn internal_format_from_depth(format: DepthFormat) -> u32 {
    match format {
        DepthFormat::Depth16 => crate::context::DEPTH_COMPONENT16,
        DepthFormat::Depth24 => crate::context::DEPTH_COMPONENT24,
        DepthFormat::Depth32F => crate::context::DEPTH_COMPONENT32F,
    }
}

fn wrapping_from(wrapping: Wrapping) -> i32 {
    (match wrapping {
        Wrapping::Repeat => crate::context::REPEAT,
        Wrapping::MirroredRepeat => crate::context::MIRRORED_REPEAT,
        Wrapping::ClampToEdge => crate::context::CLAMP_TO_EDGE,
    }) as i32
}

fn interpolation_from(interpolation: Interpolation) -> i32 {
    (match interpolation {
        Interpolation::Nearest => crate::context::NEAREST,
        Interpolation::Linear => crate::context::LINEAR,
    }) as i32
}

fn check_data_length<T: TextureDataType>(
    width: u32,
    height: u32,
    depth: u32,
    data_byte_size: usize,
    data: &[T],
) -> ThreeDResult<()> {
    let expected_bytes = width as usize * height as usize * depth as usize * data_byte_size;
    let actual_bytes = data.len() * std::mem::size_of::<T>();
    if expected_bytes != actual_bytes {
        Err(CoreError::InvalidTextureLength(
            actual_bytes,
            expected_bytes,
        ))?;
    }
    Ok(())
}
