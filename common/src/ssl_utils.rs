use anyhow::{anyhow, Result};
use openssl::rsa::{Padding, Rsa};
use openssl::symm::{encrypt, Cipher};
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub struct Aes128Keychain {
    key: Vec<u8>,
    iv: Vec<u8>,
}

impl Display for Aes128Keychain {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Aes128Keychain [ KEY '{}'; IV '{}' ]",
            hex::encode(self.key()),
            hex::encode(self.iv())
        )
    }
}

impl Aes128Keychain {
    pub fn empty() -> Self {
        Aes128Keychain {
            key: Vec::new(),
            iv: Vec::new(),
        }
    }

    pub fn new(key: &[u8], iv: &[u8]) -> Result<Self> {
        let kc = Aes128Keychain {
            key: key.to_vec(),
            iv: iv.to_vec(),
        };
        kc.is_valid()?;
        Ok(kc)
    }

    pub fn is_valid(&self) -> Result<()> {
        if self.key.len() == 16 && self.iv.len() == 16 {
            Ok(())
        } else {
            Err(anyhow!("Keychain is invalid"))
        }
    }

    pub fn key(&self) -> &[u8] {
        self.key.as_slice()
    }

    pub fn iv(&self) -> &[u8] {
        self.iv.as_slice()
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        [self.key(), self.iv()].concat()
    }
}

pub fn aes_encrypt(keychain: &Aes128Keychain, payload: &[u8]) -> Result<Vec<u8>> {
    keychain.is_valid()?;
    // let pad = (0..16 - payload.len() % 16)
    //     .map(|_| 4u8)
    //     .collect::<Vec<u8>>();
    // let buf = [payload, &pad].concat();
    Ok(encrypt(
        Cipher::aes_128_cbc(),
        keychain.key(),
        Some(keychain.iv()),
        payload,
        // buf.as_slice(),
    )?)
}

pub fn aes_decrypt(keychain: &Aes128Keychain, payload: &[u8]) -> Result<Vec<u8>> {
    keychain.is_valid()?;
    Ok(openssl::symm::decrypt(
        Cipher::aes_128_cbc(),
        keychain.key(),
        Some(keychain.iv()),
        payload,
    )?)
}

pub fn generate_rsa_keypair() -> Result<(Vec<u8>, Vec<u8>)> {
    let rsa = Rsa::generate(1024)?;
    let private_key: Vec<u8> = rsa.private_key_to_pem()?;
    let public_key: Vec<u8> = rsa.public_key_to_pem()?;
    Ok((private_key, public_key))
}

pub fn rsa_encrypt(rsa_pub: &[u8], plain_payload: &[u8]) -> Result<Vec<u8>> {
    let rsa = Rsa::public_key_from_pem(rsa_pub)?;
    let mut cipher_payload: Vec<u8> = vec![0; rsa.size() as usize];
    rsa.public_encrypt(plain_payload, &mut cipher_payload, Padding::PKCS1)?;
    Ok(cipher_payload)
}

pub fn rsa_decrypt(rsa: &[u8], cipher_payload: &[u8]) -> Result<Vec<u8>> {
    let rsa = Rsa::private_key_from_pem(rsa)?;
    let mut plain_payload: Vec<u8> = vec![0; rsa.size() as usize];
    rsa.private_decrypt(cipher_payload, &mut plain_payload, Padding::PKCS1)?;
    Ok(plain_payload)
}
