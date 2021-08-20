extern crate web_sys;

mod utils;

use image::imageops;
use image::RgbaImage;

use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn shrink_by_half(original_image: Vec<u8>, width: u32, height: u32) -> Vec<u8> {
    let image: RgbaImage = image::ImageBuffer::from_vec(width, height, original_image).unwrap();

    let output_image =
        imageops::resize(&image, width / 2, height / 2, imageops::FilterType::Nearest);

    output_image.into_vec()
}

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, wasm-image-processing!");
}
