use framkey_core::WalletType;
use framkey_ipc::SignerVaultMetadata;
use framkey_vault::{KeychainEncryptedVaultMetadata, KeychainVaultMetadata};

pub(crate) fn encrypted_metadata_to_ipc(
    metadata: KeychainEncryptedVaultMetadata,
) -> SignerVaultMetadata {
    SignerVaultMetadata {
        image_size: metadata.image_size,
        shard_size: metadata.shard_size,
        data_shards: metadata.data_shards,
        parity_shards: metadata.parity_shards,
        wallet_id: metadata.wallet_id,
        generation: metadata.generation,
        wallet_type: wallet_type_name(metadata.wallet_type).to_owned(),
        payload_hash_valid: metadata.payload_hash_valid,
        recovered_shard_count: metadata.recovered_shard_count,
        wallet_secret_hash: Some(metadata.wallet_secret_hash),
    }
}

pub(crate) fn metadata_to_ipc(
    metadata: KeychainVaultMetadata,
    wallet_secret_hash: Option<String>,
) -> SignerVaultMetadata {
    SignerVaultMetadata {
        image_size: metadata.image_size,
        shard_size: metadata.shard_size,
        data_shards: metadata.data_shards,
        parity_shards: metadata.parity_shards,
        wallet_id: metadata.wallet_id,
        generation: metadata.generation,
        wallet_type: wallet_type_name(metadata.wallet_type).to_owned(),
        payload_hash_valid: metadata.payload_hash_valid,
        recovered_shard_count: metadata.recovered_shard_count,
        wallet_secret_hash,
    }
}

fn wallet_type_name(wallet_type: WalletType) -> &'static str {
    match wallet_type {
        WalletType::EvmEoaSecp256k1 => "evm_eoa_secp256k1",
        WalletType::Secp256k1SingleKey => "secp256k1_single_key",
        _ => "unknown",
    }
}
