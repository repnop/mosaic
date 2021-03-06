#![deny(missing_docs)]

//! Mosiac: A crate for easily drawing tiles from a tilesheet with
//! [`ggez`](https://crates.io/crates/ggez).
//!

extern crate ggez;
extern crate mint;

use ggez::graphics::{self, spritebatch::SpriteBatch, Color, DrawParam, Image, Rect};
use mint::{Point2, Vector2};
use std::{collections::HashMap, hash::Hash};

/// A set of tiles made from a tilesheet image.
pub struct TileSet<Key: Hash + Eq> {
    tile_size: Vector2<i32>,
    tile_cache: HashMap<Key, Point2<i32>>,
    sheet_dimensions: Vector2<i32>,
    spritebatch: SpriteBatch,
}

impl<Key: Hash + Eq> TileSet<Key> {
    /// Create a new `TileSet` from an image and tile size.
    pub fn new<S: Into<Vector2<i32>>>(sheet: Image, tile_size: S) -> Self {
        let tile_size = tile_size.into();
        let sheet_dimensions = [
            sheet.width() as i32 / tile_size.x,
            sheet.height() as i32 / tile_size.y,
        ].into();

        Self {
            tile_size,
            tile_cache: HashMap::new(),
            sheet_dimensions,
            spritebatch: SpriteBatch::new(sheet),
        }
    }

    /// Register a tile from the tilesheet to the `TileSet` with the lookup
    /// value of `key`.
    pub fn register_tile<I: Into<Point2<i32>>>(
        &mut self,
        key: Key,
        index: I,
    ) -> Result<(), TileSetError> {
        let index = index.into();

        if index.x > self.sheet_dimensions.x || index.y > self.sheet_dimensions.y {
            return Err(TileSetError::OutOfRange);
        }

        self.tile_cache.insert(key, index);

        Ok(())
    }

    /// Queue a tile with the lookup value `key` to be drawn at `draw_location`,
    /// with optional drawing options.
    pub fn queue_tile<P: Into<Point2<i32>>, TP: Into<TileParams>>(
        &mut self,
        key: Key,
        draw_location: P,
        options: Option<TP>,
    ) -> Result<(), TileSetError> {
        let tile = self.tile_cache.get(&key).ok_or(TileSetError::TileNotFound)?;

        let options = options.map(|tp| tp.into()).unwrap_or(TileParams {
            color: None,
            scale: None,
        });

        let coords = draw_location.into();
        let normal_x = 1.0 / self.sheet_dimensions.x as f32;
        let normal_y = 1.0 / self.sheet_dimensions.y as f32;

        self.spritebatch.add(DrawParam {
            src: Rect::new(
                normal_x * tile.x as f32,
                normal_y * tile.y as f32,
                normal_x,
                normal_y,
            ),
            dest: graphics::Point2::new(
                (coords.x * self.tile_size.x) as f32,
                (coords.y * self.tile_size.y) as f32,
            ),
            color: options.color,
            scale: options.scale.unwrap_or(graphics::Point2::new(1.0, 1.0)),
            ..Default::default()
        });

        Ok(())
    }

    /// Clear the tile queue.
    pub fn clear_queue(&mut self) {
        self.spritebatch.clear();
    }

    /// Draw the tiles using `ctx`.
    pub fn draw(&self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        graphics::draw_ex(ctx, &self.spritebatch, Default::default())
    }
}

/// Additional parameters for drawing tiles.
pub struct TileParams {
    /// The optional color to draw the tile with.
    pub color: Option<Color>,
    /// Scale factor for drawing. Default is `1.0` (no scaling).
    pub scale: Option<graphics::Point2>,
}

impl From<(Option<Color>, Option<graphics::Point2>)> for TileParams {
    fn from((color, scale): (Option<Color>, Option<graphics::Point2>)) -> TileParams {
        TileParams { color, scale }
    }
}

impl From<(Option<Color>, graphics::Point2)> for TileParams {
    fn from((color, scale): (Option<Color>, graphics::Point2)) -> TileParams {
        TileParams {
            color,
            scale: Some(scale),
        }
    }
}

impl From<(Color, Option<graphics::Point2>)> for TileParams {
    fn from((color, scale): (Color, Option<graphics::Point2>)) -> TileParams {
        TileParams {
            color: Some(color),
            scale,
        }
    }
}

impl From<(Color, graphics::Point2)> for TileParams {
    fn from((color, scale): (Color, graphics::Point2)) -> TileParams {
        TileParams {
            color: Some(color),
            scale: Some(scale),
        }
    }
}

/// Possible errors from `TileSet` operations.
#[derive(Debug, Clone, Copy)]
pub enum TileSetError {
    /// The tile position to register was outside the tilesheet bounds.
    OutOfRange,
    /// Tile not found.
    TileNotFound,
}

impl std::fmt::Display for TileSetError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TileSetError::OutOfRange => "Position out of range of tilesheet dimensions",
                TileSetError::TileNotFound => "Tile not found during lookup",
            }
        )
    }
}

impl std::error::Error for TileSetError {}
