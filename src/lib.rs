pub mod packer;
pub use crate::packer::*;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageLayoutInfo {
    pub name: String,
    pub texture: usize,
    pub position: [usize; 2],
    pub size: [usize; 2],
    pub rotated: bool,
}

impl ImageLayoutInfo {
    pub fn empty() -> Self {
        ImageLayoutInfo {
            name: String::from(""),
            texture: 0,
            position: [0, 0],
            size: [0, 0],
            rotated: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OutputData {
    pub image_layouts: Vec<ImageLayoutInfo>,
    pub textures: Vec<String>,
}