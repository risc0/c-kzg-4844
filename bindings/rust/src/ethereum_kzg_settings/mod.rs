use crate::KzgSettings;
use once_cell::sync::Lazy;

/// A lazily initialized global instance of Ethereum mainnet KZG settings.
static ETHEREUM_KZG_SETTINGS: Lazy<&'static KzgSettings> = Lazy::new(load_kzg_settings);

/// Returns default Ethereum mainnet KZG settings.
///
/// Note: Precompute values are ignored.
#[inline]
pub fn ethereum_kzg_settings(_precompute: u64) -> &'static KzgSettings {
    *ETHEREUM_KZG_SETTINGS
}

#[cfg(feature = "generate_ethereum_kzg_settings")]
fn generate_kzg_settings() -> std::io::Result<&'static crate::bindings::RawKzgSettings> {
    use std::{fs::File, io::Write, path::PathBuf};

    let trusted_setup = include_str!("../../../../src/trusted_setup.txt");
    let settings = KzgSettings::parse_kzg_trusted_setup(trusted_setup, 0).unwrap();

    let raw = settings.to_raw();

    let mut root_dir = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
    root_dir.push("bindings/rust/src/ethereum_kzg_settings");

    File::create(root_dir.join("roots_of_unity.bin"))?.write_all(&raw.roots_of_unity)?;
    File::create(root_dir.join("brp_roots_of_unity.bin"))?.write_all(&raw.brp_roots_of_unity)?;
    File::create(root_dir.join("reverse_roots_of_unity.bin"))?
        .write_all(&raw.reverse_roots_of_unity)?;
    File::create(root_dir.join("g1_values_monomial.bin"))?.write_all(&raw.g1_values_monomial)?;
    File::create(root_dir.join("g1_values_lagrange_brp.bin"))?
        .write_all(&raw.g1_values_lagrange_brp)?;
    File::create(root_dir.join("g2_values_monomial.bin"))?.write_all(&raw.g2_values_monomial)?;
    File::create(root_dir.join("x_ext_fft_columns.bin"))?.write_all(&raw.x_ext_fft_columns)?;

    Ok(Box::leak(raw))
}

fn load_kzg_settings() -> &'static KzgSettings {
    #[cfg(feature = "generate_ethereum_kzg_settings")]
    return KzgSettings::from_raw(generate_kzg_settings().expect("failed to write KZG settings"))
        .unwrap();
    #[cfg(not(feature = "generate_ethereum_kzg_settings"))]
    {
        use crate::bindings::RawKzgSettings;
        static RAW_KZG_SETTINGS: RawKzgSettings = RawKzgSettings {
            roots_of_unity: *include_bytes!("roots_of_unity.bin"),
            brp_roots_of_unity: *include_bytes!("brp_roots_of_unity.bin"),
            reverse_roots_of_unity: *include_bytes!("reverse_roots_of_unity.bin"),
            g1_values_monomial: *include_bytes!("g1_values_monomial.bin"),
            g1_values_lagrange_brp: *include_bytes!("g1_values_lagrange_brp.bin"),
            g2_values_monomial: *include_bytes!("g2_values_monomial.bin"),
            #[cfg(feature = "eip-7594")]
            x_ext_fft_columns: *include_bytes!("x_ext_fft_columns.bin"),
            #[cfg(not(feature = "eip-7594"))]
            x_ext_fft_columns: [],
            scratch_size: 0,
        };

        KzgSettings::from_raw(&RAW_KZG_SETTINGS).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{bindings::BYTES_PER_BLOB, Blob, KzgSettings};
    use std::path::Path;

    #[test]
    pub fn compare_default_with_file() {
        let precompute = 0;
        let ts_settings =
            KzgSettings::load_trusted_setup_file(Path::new("src/trusted_setup.txt"), precompute)
                .unwrap();
        let eth_settings = ethereum_kzg_settings(precompute);
        let blob = Blob::new([1u8; BYTES_PER_BLOB]);

        // generate commitment
        let ts_commitment = ts_settings
            .blob_to_kzg_commitment(&blob)
            .unwrap()
            .to_bytes();
        let eth_commitment = eth_settings
            .blob_to_kzg_commitment(&blob)
            .unwrap()
            .to_bytes();
        assert_eq!(ts_commitment, eth_commitment);

        // generate proof
        let ts_proof = ts_settings
            .compute_blob_kzg_proof(&blob, &ts_commitment)
            .unwrap()
            .to_bytes();
        let eth_proof = eth_settings
            .compute_blob_kzg_proof(&blob, &eth_commitment)
            .unwrap()
            .to_bytes();
        assert_eq!(ts_proof, eth_proof);
    }
}
