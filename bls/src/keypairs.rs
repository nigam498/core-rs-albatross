use super::*;

// Is this even used?
#[derive(Clone, PartialEq, Eq)]
pub struct KeyPair {
    pub secret_key: SecretKey,
    pub public_key: PublicKey,
}

impl KeyPair {
    pub fn from_secret(secret: &SecretKey) -> Self {
        KeyPair::from(secret.clone())
    }

    pub fn sign<M: Hash>(&self, msg: &M) -> Signature {
        self.secret_key.sign::<M>(msg)
    }

    pub fn sign_hash(&self, hash: SigHash) -> Signature {
        self.secret_key.sign_hash(hash)
    }

    pub fn verify<M: Hash>(&self, msg: &M, signature: &Signature) -> bool {
        self.public_key.verify::<M>(msg, signature)
    }

    pub fn verify_hash(&self, hash: SigHash, signature: &Signature) -> bool {
        self.public_key.verify_hash(hash, signature)
    }
}

impl SecureGenerate for KeyPair {
    fn generate<R: Rng + CryptoRng>(rng: &mut R) -> Self {
        let secret = SecretKey::generate(rng);
        KeyPair::from(secret)
    }
}

impl From<SecretKey> for KeyPair {
    fn from(secret: SecretKey) -> Self {
        let public = PublicKey::from_secret(&secret);
        return KeyPair {
            secret_key: secret,
            public_key: public,
        };
    }
}
