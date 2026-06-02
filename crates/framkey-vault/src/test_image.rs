use framkey_core::{FramkeyError, Generation, Result};

use crate::{SaveSlot, save_image::build_save_image_with_payload, types::TestVaultPayload};

pub fn build_test_save_image(
    image_size: usize,
    generation: Generation,
    label: &str,
) -> Result<Vec<u8>> {
    let payload = TestVaultPayload {
        kind: "framkey.test_vault",
        label,
        generation: generation.0,
        note: "hardware smoke test only; contains no wallet secret",
    };
    let payload = serde_json::to_vec_pretty(&payload)
        .map_err(|error| FramkeyError::invalid_data(error.to_string()))?;

    build_save_image_with_payload(image_size, SaveSlot::A, generation, &payload)
}
