use crate::bindings::RawKzgSettings;
use crate::KzgSettings;
use once_cell::sync::Lazy;

/// A lazily initialized global instance of Ethereum mainnet KZG settings.
static ETHEREUM_KZG_SETTINGS: Lazy<&'static KzgSettings> = Lazy::new(load_kzg_settings);

/// Returns default Ethereum mainnet KZG settings.
#[inline]
pub fn ethereum_kzg_settings() -> &'static KzgSettings {
    *ETHEREUM_KZG_SETTINGS
}

#[cfg(feature = "generate_ethereum_kzg_settings")]
fn generate_kzg_settings() -> std::io::Result<&'static RawKzgSettings> {
    use std::{fs::File, io::Write, path::PathBuf};

    let trusted_setup = include_str!("../../../../src/trusted_setup.txt");
    let settings = KzgSettings::parse_kzg_trusted_setup(trusted_setup).unwrap();

    let raw = settings.to_raw();

    let mut root_dir = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
    root_dir.push("bindings/rust/src/ethereum_kzg_settings");

    File::create(root_dir.join("roots_of_unity.bin"))?.write_all(&raw.roots_of_unity)?;
    File::create(root_dir.join("g1_points.bin"))?.write_all(&raw.g1_points)?;
    File::create(root_dir.join("g2_points.bin"))?.write_all(&raw.g2_points)?;

    Ok(Box::leak(raw))
}

fn load_kzg_settings() -> &'static KzgSettings {
    #[cfg(feature = "generate_ethereum_kzg_settings")]
    return KzgSettings::from_raw(generate_kzg_settings().expect("failed to write KZG settings"))
        .unwrap();

    #[cfg(not(feature = "generate_ethereum_kzg_settings"))]
    {
        static RAW_KZG_SETTINGS: RawKzgSettings = RawKzgSettings {
            roots_of_unity: *include_bytes!("roots_of_unity.bin"),
            g1_points: *include_bytes!("g1_points.bin"),
            g2_points: *include_bytes!("g2_points.bin"),
        };

        KzgSettings::from_raw(&RAW_KZG_SETTINGS).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{bindings::BYTES_PER_BLOB, Blob, KzgCommitment, KzgProof, KzgSettings};
    use std::path::Path;

    #[test]
    pub fn compare_default_with_file() {
        let ts_settings =
            KzgSettings::load_trusted_setup_file(Path::new("src/trusted_setup.txt")).unwrap();
        let eth_settings = ethereum_kzg_settings();
        let blob = Blob::new([1u8; BYTES_PER_BLOB]);

        // generate commitment
        let ts_commitment = KzgCommitment::blob_to_kzg_commitment(&blob, &ts_settings)
            .unwrap()
            .to_bytes();
        let eth_commitment = KzgCommitment::blob_to_kzg_commitment(&blob, eth_settings)
            .unwrap()
            .to_bytes();
        assert_eq!(ts_commitment, eth_commitment);

        // generate proof
        let ts_proof = KzgProof::compute_blob_kzg_proof(&blob, &ts_commitment, &ts_settings)
            .unwrap()
            .to_bytes();
        let eth_proof = KzgProof::compute_blob_kzg_proof(&blob, &eth_commitment, eth_settings)
            .unwrap()
            .to_bytes();
        assert_eq!(ts_proof, eth_proof);
    }
}
