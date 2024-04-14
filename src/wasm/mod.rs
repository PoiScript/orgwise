mod backend;
mod lsp_backend;

use serde_wasm_bindgen::Serializer;

pub const SERIALIZER: Serializer = Serializer::new().serialize_maps_as_objects(true);
