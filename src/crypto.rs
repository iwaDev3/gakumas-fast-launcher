use crate::error::Error;
use aes_gcm::aead::Aead;
use aes_gcm::{Aes128Gcm, Aes256Gcm, Key, KeyInit, Nonce};
use zeroize::Zeroizing;

type V10Parts<'a> = (&'a [u8], &'a [u8], &'a [u8]);

pub fn split_v10(blob: &[u8]) -> Result<V10Parts<'_>, Error> {
    if blob.len() < 3 {
        return Err(Error::AuthDecryptFailed);
    }
    let prefix = &blob[..3];
    if prefix != b"v10" {
        return Err(Error::AuthPrefixUnsupported {
            prefix: String::from_utf8_lossy(prefix).into_owned(),
        });
    }
    if blob.len() < 31 {
        return Err(Error::AuthDecryptFailed);
    }
    let nonce = &blob[3..15];
    let tag = &blob[blob.len() - 16..];
    let ciphertext = &blob[15..blob.len() - 16];
    Ok((nonce, ciphertext, tag))
}

pub fn decrypt_v10(key: &[u8], blob: &[u8]) -> Result<Zeroizing<Vec<u8>>, Error> {
    let (nonce, ciphertext, tag) = split_v10(blob)?;
    match key.len() {
        32 => decrypt_aes256(key, nonce, ciphertext, tag),
        16 => decrypt_aes128(key, nonce, ciphertext, tag),
        _ => Err(Error::EncryptedKeyInvalid),
    }
}

fn decrypt_aes256(
    key: &[u8],
    nonce: &[u8],
    ciphertext: &[u8],
    tag: &[u8],
) -> Result<Zeroizing<Vec<u8>>, Error> {
    let key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);
    let mut ct = ciphertext.to_vec();
    ct.extend_from_slice(tag);
    cipher
        .decrypt(Nonce::from_slice(nonce), ct.as_ref())
        .map(Zeroizing::new)
        .map_err(|_| Error::AuthDecryptFailed)
}

fn decrypt_aes128(
    key: &[u8],
    nonce: &[u8],
    ciphertext: &[u8],
    tag: &[u8],
) -> Result<Zeroizing<Vec<u8>>, Error> {
    let key = Key::<Aes128Gcm>::from_slice(key);
    let cipher = Aes128Gcm::new(key);
    let mut ct = ciphertext.to_vec();
    ct.extend_from_slice(tag);
    cipher
        .decrypt(Nonce::from_slice(nonce), ct.as_ref())
        .map(Zeroizing::new)
        .map_err(|_| Error::AuthDecryptFailed)
}

#[cfg(windows)]
pub fn dpapi_unprotect(data: &[u8]) -> Result<Zeroizing<Vec<u8>>, Error> {
    use windows::Win32::Foundation::{LocalFree, HLOCAL};
    use windows::Win32::Security::Cryptography::{
        CryptUnprotectData, CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB,
    };
    use zeroize::Zeroize;

    if data.is_empty() {
        return Err(Error::DpapiFailed);
    }

    let mut input = data.to_vec();
    let in_blob = CRYPT_INTEGER_BLOB {
        cbData: input.len() as u32,
        pbData: input.as_mut_ptr(),
    };
    let mut out_blob = CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: std::ptr::null_mut(),
    };

    let result = unsafe {
        CryptUnprotectData(
            &in_blob,
            None,
            None,
            None,
            None,
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut out_blob,
        )
    };

    input.zeroize();

    match result {
        Ok(()) => {
            let plain =
                unsafe { std::slice::from_raw_parts(out_blob.pbData, out_blob.cbData as usize) }
                    .to_vec();
            if !out_blob.pbData.is_null() {
                unsafe {
                    let _ = LocalFree(Some(HLOCAL(out_blob.pbData.cast())));
                }
            }
            Ok(Zeroizing::new(plain))
        }
        Err(_) => Err(Error::DpapiFailed),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aes_gcm::aead::Aead;
    use aes_gcm::{Aes256Gcm, KeyInit, Nonce};

    fn encrypt_v10(key: &[u8; 32], nonce: &[u8; 12], plaintext: &[u8]) -> Vec<u8> {
        let cipher = Aes256Gcm::new(key.into());
        let ct = cipher
            .encrypt(Nonce::from_slice(nonce), plaintext)
            .expect("encrypt");
        let mut blob = Vec::with_capacity(3 + 12 + ct.len());
        blob.extend_from_slice(b"v10");
        blob.extend_from_slice(nonce);
        blob.extend_from_slice(&ct);
        blob
    }

    #[test]
    fn decrypt_roundtrip() {
        let key = [0x42u8; 32];
        let nonce = [0x11u8; 12];
        let plaintext = b"{\"accessToken\":\"abc\"}";
        let blob = encrypt_v10(&key, &nonce, plaintext);
        let out = decrypt_v10(&key, &blob).expect("decrypt");
        assert_eq!(&out[..], plaintext);
    }

    #[test]
    fn split_and_decrypt_match() {
        let key = [7u8; 32];
        let nonce = [9u8; 12];
        let blob = encrypt_v10(&key, &nonce, b"hello");
        let (n, ct, tag) = split_v10(&blob).expect("split");
        assert_eq!(n, &nonce);
        assert_eq!(tag.len(), 16);
        assert_eq!(ct.len() + tag.len() + 15, blob.len());
        assert_eq!(&decrypt_v10(&key, &blob).unwrap()[..], b"hello");
    }

    #[test]
    fn v20_prefix_unsupported() {
        let mut blob = encrypt_v10(&[1u8; 32], &[2u8; 12], b"x");
        blob[0..3].copy_from_slice(b"v20");
        match split_v10(&blob) {
            Err(Error::AuthPrefixUnsupported { prefix }) => assert_eq!(prefix, "v20"),
            other => panic!("expected AuthPrefixUnsupported, got {other:?}"),
        }
        match decrypt_v10(&[1u8; 32], &blob) {
            Err(Error::AuthPrefixUnsupported { prefix }) => assert_eq!(prefix, "v20"),
            other => panic!("expected AuthPrefixUnsupported, got {other:?}"),
        }
    }

    #[test]
    fn truncated_blob_errors() {
        assert!(matches!(split_v10(b"v1"), Err(Error::AuthDecryptFailed)));
        assert!(matches!(split_v10(b"v10"), Err(Error::AuthDecryptFailed)));
        let short = [b'v', b'1', b'0', 0, 1, 2];
        assert!(matches!(split_v10(&short), Err(Error::AuthDecryptFailed)));
    }
}
