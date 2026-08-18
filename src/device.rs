use crate::error::Error;
use hmac::{Hmac, Mac};
use sha2::Sha256;

pub const KDF_KEY: &[u8] = b"gkmasfl/device/v1";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeviceFingerprint {
    pub mac_address: String,
    pub hdd_serial: String,
    pub motherboard: String,
}

pub fn fingerprint_from_ikm(ikm: &[u8]) -> DeviceFingerprint {
    let prk = hmac_sha256(KDF_KEY, ikm);
    let mut mac_raw = hmac_sha256(&prk, b"mac");
    let hdd = hmac_sha256(&prk, b"hdd");
    let mb = hmac_sha256(&prk, b"mb");
    mac_raw[0] = (mac_raw[0] & 0xFE) | 0x02;
    DeviceFingerprint {
        mac_address: format!(
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            mac_raw[0], mac_raw[1], mac_raw[2], mac_raw[3], mac_raw[4], mac_raw[5]
        ),
        hdd_serial: hex::encode(hdd),
        motherboard: hex::encode(mb),
    }
}

pub fn random_fingerprint() -> Result<DeviceFingerprint, Error> {
    let mut ikm = [0u8; 32];
    getrandom::getrandom(&mut ikm).map_err(|_| Error::DeviceRandomFailed)?;
    Ok(fingerprint_from_ikm(&ikm))
}

fn hmac_sha256(key: &[u8], data: &[u8]) -> [u8; 32] {
    let mut mac = Hmac::<Sha256>::new_from_slice(key).expect("HMAC-SHA256 accepts any key length");
    mac.update(data);
    mac.finalize().into_bytes().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fingerprint_stable_and_shaped() {
        let a = fingerprint_from_ikm(b"fixed");
        let b = fingerprint_from_ikm(b"fixed");
        assert_eq!(a.mac_address, b.mac_address);
        assert_eq!(a.hdd_serial, b.hdd_serial);
        assert_eq!(a.motherboard, b.motherboard);

        let parts: Vec<&str> = a.mac_address.split(':').collect();
        assert_eq!(parts.len(), 6);
        assert!(
            parts
                .iter()
                .all(|p| p.len() == 2 && p.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f')))
        );
        let first = u8::from_str_radix(&a.mac_address[0..2], 16).unwrap();
        assert_eq!(first & 0x01, 0);
        assert_eq!(first & 0x02, 0x02);

        assert_eq!(a.hdd_serial.len(), 64);
        assert_eq!(a.motherboard.len(), 64);
        assert!(a.hdd_serial.chars().all(|c| c.is_ascii_hexdigit()));
        assert!(a.motherboard.chars().all(|c| c.is_ascii_hexdigit()));
        assert_ne!(a.hdd_serial, a.motherboard);
    }

    #[test]
    fn random_fingerprints_differ() {
        let a = random_fingerprint().unwrap();
        let b = random_fingerprint().unwrap();
        assert_ne!(a, b);
    }
}
