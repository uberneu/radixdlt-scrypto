use sbor::*;

use super::{EcdsaPublicKey, EcdsaSignature, Ed25519PublicKey, Ed25519Signature};

/// Represents any natively supported public key.
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "public_key")
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, TypeId, Encode, Decode)]
pub enum PublicKey {
    Ecdsa(EcdsaPublicKey),
    Ed25519(Ed25519PublicKey),
}

/// Represents any natively supported signature.
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "signature")
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, TypeId, Encode, Decode, Hash)]
pub enum Signature {
    Ecdsa(EcdsaSignature),
    Ed25519(Ed25519Signature),
}

/// Represents any natively supported signature, including public key.
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type")
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, TypeId, Encode, Decode, Hash)]
pub enum SignatureWithPublicKey {
    Ecdsa {
        signature: EcdsaSignature,
    },
    Ed25519 {
        public_key: Ed25519PublicKey,
        signature: Ed25519Signature,
    },
}

impl SignatureWithPublicKey {
    pub fn signature(&self) -> Signature {
        match &self {
            SignatureWithPublicKey::Ecdsa { signature } => signature.clone().into(),
            SignatureWithPublicKey::Ed25519 { signature, .. } => signature.clone().into(),
        }
    }
}

impl From<EcdsaPublicKey> for PublicKey {
    fn from(public_key: EcdsaPublicKey) -> Self {
        Self::Ecdsa(public_key)
    }
}

impl From<Ed25519PublicKey> for PublicKey {
    fn from(public_key: Ed25519PublicKey) -> Self {
        Self::Ed25519(public_key)
    }
}

impl From<EcdsaSignature> for Signature {
    fn from(signature: EcdsaSignature) -> Self {
        Self::Ecdsa(signature)
    }
}

impl From<Ed25519Signature> for Signature {
    fn from(signature: Ed25519Signature) -> Self {
        Self::Ed25519(signature)
    }
}

impl From<EcdsaSignature> for SignatureWithPublicKey {
    fn from(signature: EcdsaSignature) -> Self {
        Self::Ecdsa { signature }
    }
}

impl From<(Ed25519PublicKey, Ed25519Signature)> for SignatureWithPublicKey {
    fn from((public_key, signature): (Ed25519PublicKey, Ed25519Signature)) -> Self {
        Self::Ed25519 {
            public_key,
            signature,
        }
    }
}
