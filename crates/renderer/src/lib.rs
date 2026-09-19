//! Planning and assembling a tiled Mandelbrot image.

use std::collections::{HashMap, HashSet};

use showcase_api::model::{RenderTileRequest, RenderTileResponse};

const IMAGE_REGION_MIN_RE: f64 = -2.0;
const IMAGE_REGION_MAX_RE: f64 = 1.0;
const IMAGE_REGION_MIN_IM: f64 = -1.0;
const IMAGE_REGION_MAX_IM: f64 = 1.0;

/// A tile request together with its pixel offset in the final image.
#[derive(Debug, Clone, PartialEq)]
pub struct TilePlan {
    /// Horizontal pixel offset from the image's left edge.
    pub x: u32,
    /// Vertical pixel offset from the image's top edge.
    pub y: u32,
    /// Request sent to a render worker.
    pub request: RenderTileRequest,
}

/// Errors raised while planning, assembling, or encoding the image.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderError {
    /// Image dimensions and tile size must be non-zero.
    EmptyDimensions,
    /// The image or tile count cannot be represented on this platform.
    DimensionsTooLarge,
    /// A response refers to a tile that was not requested.
    UnknownTile,
    /// A tile response has an invalid or duplicate identifier.
    InvalidTileResponse,
    /// A requested tile has no response.
    MissingTile,
}

impl std::fmt::Display for RenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyDimensions => f.write_str("image dimensions and tile size must be non-zero"),
            Self::DimensionsTooLarge => f.write_str("image dimensions or tile count are too large"),
            Self::UnknownTile => f.write_str("received a response for an unknown tile"),
            Self::InvalidTileResponse => {
                f.write_str("received an invalid or duplicate tile response")
            }
            Self::MissingTile => f.write_str("one or more tile responses are missing"),
        }
    }
}

impl std::error::Error for RenderError {}

/// Divide a full Mandelbrot view into row-major tiles.
pub fn plan_tiles(
    width: u32,
    height: u32,
    tile_size: u32,
    max_iterations: u32,
) -> Result<Vec<TilePlan>, RenderError> {
    if width == 0 || height == 0 || tile_size == 0 {
        return Err(RenderError::EmptyDimensions);
    }

    let columns = width.div_ceil(tile_size);
    let rows = height.div_ceil(tile_size);
    let tile_count = columns
        .checked_mul(rows)
        .ok_or(RenderError::DimensionsTooLarge)?;
    let tile_capacity = usize::try_from(tile_count).map_err(|_| RenderError::DimensionsTooLarge)?;
    let mut tiles = Vec::new();
    tiles
        .try_reserve_exact(tile_capacity)
        .map_err(|_| RenderError::DimensionsTooLarge)?;

    for y in (0..height).step_by(tile_size as usize) {
        for x in (0..width).step_by(tile_size as usize) {
            let tile_width = tile_size.min(width - x);
            let tile_height = tile_size.min(height - y);
            let min_re = IMAGE_REGION_MIN_RE
                + f64::from(x) / f64::from(width) * (IMAGE_REGION_MAX_RE - IMAGE_REGION_MIN_RE);
            let max_re = IMAGE_REGION_MIN_RE
                + f64::from(x + tile_width) / f64::from(width)
                    * (IMAGE_REGION_MAX_RE - IMAGE_REGION_MIN_RE);
            let max_im = IMAGE_REGION_MAX_IM
                - f64::from(y) / f64::from(height) * (IMAGE_REGION_MAX_IM - IMAGE_REGION_MIN_IM);
            let min_im = IMAGE_REGION_MAX_IM
                - f64::from(y + tile_height) / f64::from(height)
                    * (IMAGE_REGION_MAX_IM - IMAGE_REGION_MIN_IM);

            let tile_id =
                u32::try_from(tiles.len()).map_err(|_| RenderError::DimensionsTooLarge)?;
            tiles.push(TilePlan {
                x,
                y,
                request: RenderTileRequest {
                    tile_id,
                    region: fractal_core::ComplexRegion {
                        min_re,
                        max_re,
                        min_im,
                        max_im,
                    },
                    width: tile_width,
                    height: tile_height,
                    max_iterations,
                },
            });
        }
    }

    Ok(tiles)
}

/// Assemble tile responses into a row-major RGB8 image, independent of response order.
pub fn assemble_rgb(
    width: u32,
    height: u32,
    tiles: &[TilePlan],
    responses: &[RenderTileResponse],
) -> Result<Vec<u8>, RenderError> {
    if width == 0 || height == 0 {
        return Err(RenderError::EmptyDimensions);
    }

    let image_len = (width as usize)
        .checked_mul(height as usize)
        .and_then(|pixels| pixels.checked_mul(3))
        .ok_or(RenderError::DimensionsTooLarge)?;
    let mut pixels = Vec::new();
    pixels
        .try_reserve_exact(image_len)
        .map_err(|_| RenderError::DimensionsTooLarge)?;
    pixels.resize(image_len, 0);

    let by_id: HashMap<_, _> = tiles
        .iter()
        .map(|tile| (tile.request.tile_id, tile))
        .collect();
    if by_id.len() != tiles.len() {
        return Err(RenderError::InvalidTileResponse);
    }

    let mut seen = HashSet::with_capacity(responses.len());
    for response in responses {
        let tile = by_id
            .get(&response.tile_id)
            .ok_or(RenderError::UnknownTile)?;
        let request = &tile.request;
        let expected_len = (request.width as usize)
            .checked_mul(request.height as usize)
            .and_then(|pixels| pixels.checked_mul(3))
            .ok_or(RenderError::DimensionsTooLarge)?;
        if !seen.insert(response.tile_id)
            || response.width != request.width
            || response.height != request.height
            || response.pixels.len() != expected_len
            || tile
                .x
                .checked_add(request.width)
                .is_none_or(|end| end > width)
            || tile
                .y
                .checked_add(request.height)
                .is_none_or(|end| end > height)
        {
            return Err(RenderError::InvalidTileResponse);
        }

        let row_bytes = request.width as usize * 3;
        for row in 0..request.height as usize {
            let source_start = row * row_bytes;
            let target_start = ((tile.y as usize + row) * width as usize + tile.x as usize) * 3;
            pixels[target_start..target_start + row_bytes]
                .copy_from_slice(&response.pixels[source_start..source_start + row_bytes]);
        }
    }

    if seen.len() != tiles.len() {
        return Err(RenderError::MissingTile);
    }

    Ok(pixels)
}

/// Encode an RGB8 buffer as a PNG image.
pub fn write_png<W: std::io::Write>(
    output: W,
    width: u32,
    height: u32,
    pixels: &[u8],
) -> Result<(), png::EncodingError> {
    let mut encoder = png::Encoder::new(output, width, height);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.write_header()?.write_image_data(pixels)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use showcase_api::model::RuntimeKind;

    use super::*;

    fn response(tile: &TilePlan, color: [u8; 3]) -> RenderTileResponse {
        let pixels = color.repeat((tile.request.width * tile.request.height) as usize);
        RenderTileResponse {
            tile_id: tile.request.tile_id,
            worker_id: "test-worker".to_owned(),
            runtime: RuntimeKind::Tokio,
            width: tile.request.width,
            height: tile.request.height,
            pixels,
            render_duration_ms: 1.0,
            artificial_delay_ms: 0.0,
        }
    }

    #[test]
    fn planning_covers_non_divisible_image_dimensions() {
        let tiles = plan_tiles(5, 3, 2, 64).unwrap();

        assert_eq!(tiles.len(), 6);
        assert_eq!((tiles[0].request.width, tiles[0].request.height), (2, 2));
        assert_eq!((tiles[2].x, tiles[2].request.width), (4, 1));
        assert_eq!((tiles[5].y, tiles[5].request.height), (2, 1));
    }

    #[test]
    fn assembly_uses_tile_coordinates_not_completion_order() {
        let tiles = plan_tiles(2, 2, 1, 16).unwrap();
        let colors = [[1, 0, 0], [2, 0, 0], [3, 0, 0], [4, 0, 0]];
        let responses: Vec<_> = tiles
            .iter()
            .zip(colors)
            .rev()
            .map(|(tile, color)| response(tile, color))
            .collect();

        let image = assemble_rgb(2, 2, &tiles, &responses).unwrap();

        assert_eq!(image, [1, 0, 0, 2, 0, 0, 3, 0, 0, 4, 0, 0]);
    }

    #[test]
    fn png_output_has_expected_dimensions_and_rgb_pixels() {
        let pixels = [7, 8, 9, 10, 11, 12];
        let mut encoded = Vec::new();
        write_png(&mut encoded, 2, 1, &pixels).unwrap();

        let decoder = png::Decoder::new(Cursor::new(encoded));
        let mut reader = decoder.read_info().unwrap();
        let mut decoded = vec![0; reader.output_buffer_size().unwrap()];
        let info = reader.next_frame(&mut decoded).unwrap();

        assert_eq!((info.width, info.height), (2, 1));
        assert_eq!(&decoded[..info.buffer_size()], &pixels);
    }
}
