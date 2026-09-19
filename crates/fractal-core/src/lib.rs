//! Deterministic, synchronous Mandelbrot tile rendering.

use serde::{Deserialize, Serialize};

/// A rectangular region of the complex plane.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ComplexRegion {
    /// Left edge of the region.
    pub min_re: f64,
    /// Right edge of the region.
    pub max_re: f64,
    /// Bottom edge of the region.
    pub min_im: f64,
    /// Top edge of the region.
    pub max_im: f64,
}

/// Rendering parameters for one rectangular image tile.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TileSpec {
    /// Complex-plane bounds sampled by this tile.
    pub region: ComplexRegion,
    /// Number of pixels across.
    pub width: u32,
    /// Number of pixels down.
    pub height: u32,
    /// Maximum Mandelbrot iterations per pixel.
    pub max_iterations: u32,
}

/// Rendered tile pixels in row-major RGB8 format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedTile {
    /// Number of pixels across.
    pub width: u32,
    /// Number of pixels down.
    pub height: u32,
    /// Three bytes (red, green, blue) per pixel, from top-left to bottom-right.
    pub pixels: Vec<u8>,
}

/// Reasons a tile cannot be represented or rendered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderError {
    /// A tile must have non-zero width and height.
    EmptyDimensions,
    /// Region bounds must be finite and ordered.
    InvalidRegion,
    /// The requested pixel buffer is too large to allocate.
    DimensionsTooLarge,
}

impl std::fmt::Display for RenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyDimensions => f.write_str("tile dimensions must be non-zero"),
            Self::InvalidRegion => f.write_str("complex region bounds must be finite and ordered"),
            Self::DimensionsTooLarge => f.write_str("tile pixel buffer is too large"),
        }
    }
}

impl std::error::Error for RenderError {}

/// Render a Mandelbrot tile as deterministic RGB8 pixels.
pub fn render_tile(spec: &TileSpec) -> Result<RenderedTile, RenderError> {
    if spec.width == 0 || spec.height == 0 {
        return Err(RenderError::EmptyDimensions);
    }

    let region = spec.region;
    if ![region.min_re, region.max_re, region.min_im, region.max_im]
        .into_iter()
        .all(f64::is_finite)
        || region.min_re > region.max_re
        || region.min_im > region.max_im
    {
        return Err(RenderError::InvalidRegion);
    }

    let capacity = (spec.width as usize)
        .checked_mul(spec.height as usize)
        .and_then(|pixels| pixels.checked_mul(3))
        .ok_or(RenderError::DimensionsTooLarge)?;
    let mut pixels = Vec::new();
    pixels
        .try_reserve_exact(capacity)
        .map_err(|_| RenderError::DimensionsTooLarge)?;

    for y in 0..spec.height {
        let im =
            region.max_im - (y as f64 + 0.5) / spec.height as f64 * (region.max_im - region.min_im);
        for x in 0..spec.width {
            let re = region.min_re
                + (x as f64 + 0.5) / spec.width as f64 * (region.max_re - region.min_re);
            let Some(iterations) = escape_iteration(re, im, spec.max_iterations) else {
                pixels.extend_from_slice(&[0, 0, 0]);
                continue;
            };

            let intensity =
                ((u64::from(iterations) * 255) / u64::from(spec.max_iterations.max(1))) as u8;
            pixels.extend_from_slice(&[intensity, intensity.saturating_mul(2), 255 - intensity]);
        }
    }

    Ok(RenderedTile {
        width: spec.width,
        height: spec.height,
        pixels,
    })
}

fn escape_iteration(c_re: f64, c_im: f64, max_iterations: u32) -> Option<u32> {
    let (mut z_re, mut z_im) = (0.0, 0.0);
    for iteration in 1..=max_iterations {
        let next_re = z_re * z_re - z_im * z_im + c_re;
        z_im = 2.0 * z_re * z_im + c_im;
        z_re = next_re;
        if z_re * z_re + z_im * z_im > 4.0 {
            return Some(iteration);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn point(re: f64, im: f64, max_iterations: u32) -> TileSpec {
        TileSpec {
            region: ComplexRegion {
                min_re: re,
                max_re: re,
                min_im: im,
                max_im: im,
            },
            width: 1,
            height: 1,
            max_iterations,
        }
    }

    #[test]
    fn rendered_tile_has_rgb8_dimensions_and_pixel_count() {
        let spec = TileSpec {
            region: ComplexRegion {
                min_re: -2.0,
                max_re: 1.0,
                min_im: -1.0,
                max_im: 1.0,
            },
            width: 3,
            height: 2,
            max_iterations: 64,
        };

        let tile = render_tile(&spec).unwrap();

        assert_eq!((tile.width, tile.height), (3, 2));
        assert_eq!(tile.pixels.len(), 3 * 2 * 3);
    }

    #[test]
    fn render_output_is_deterministic() {
        let spec = point(-0.5, 0.25, 100);

        assert_eq!(render_tile(&spec), render_tile(&spec));
    }

    #[test]
    fn reference_tile_has_expected_fnv1a_checksum() {
        let spec = TileSpec {
            region: ComplexRegion {
                min_re: -2.0,
                max_re: 1.0,
                min_im: -1.0,
                max_im: 1.0,
            },
            width: 4,
            height: 3,
            max_iterations: 32,
        };
        let tile = render_tile(&spec).unwrap();
        let checksum = tile.pixels.iter().fold(0x811c_9dc5_u32, |hash, byte| {
            (hash ^ u32::from(*byte)).wrapping_mul(0x0100_0193)
        });

        assert_eq!(checksum, 0xf4c9_220e);
    }

    #[test]
    fn points_inside_the_set_are_black() {
        assert_eq!(
            render_tile(&point(0.0, 0.0, 100)).unwrap().pixels,
            [0, 0, 0]
        );
    }

    #[test]
    fn points_outside_the_set_use_the_escape_gradient() {
        let pixel = &render_tile(&point(2.0, 0.0, 100)).unwrap().pixels;

        assert_ne!(pixel, &[0, 0, 0]);
        assert_eq!(pixel.len(), 3);
    }

    #[test]
    fn zero_dimensions_are_rejected() {
        let mut spec = point(0.0, 0.0, 100);
        spec.width = 0;

        assert_eq!(render_tile(&spec), Err(RenderError::EmptyDimensions));
    }

    #[test]
    fn non_finite_region_bounds_are_rejected() {
        let mut spec = point(0.0, 0.0, 100);
        spec.region.max_re = f64::INFINITY;

        assert_eq!(render_tile(&spec), Err(RenderError::InvalidRegion));
    }
}
