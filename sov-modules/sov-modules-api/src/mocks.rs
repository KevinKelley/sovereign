use std::convert::Infallible;

use crate::{Context, Spec};
use borsh::{BorshDeserialize, BorshSerialize};
use sov_state::{JmtStorage, ZkStorage};

/// Mock for Spec::PublicKey, useful for testing.
#[derive(PartialEq, Eq, Clone, BorshDeserialize, BorshSerialize, Debug)]
pub struct MockPublicKey {
    pub_key: Vec<u8>,
}

impl MockPublicKey {
    pub fn new(pub_key: Vec<u8>) -> Self {
        Self { pub_key }
    }
}

impl TryFrom<&'static str> for MockPublicKey {
    type Error = Infallible;

    fn try_from(key: &'static str) -> Result<Self, Self::Error> {
        let key = key.as_bytes().to_vec();
        Ok(Self { pub_key: key })
    }
}

/// Mock for Spec::Signature, useful for testing.
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, PartialEq, Eq, Debug, Clone)]
pub struct MockSignature {
    sig: Vec<u8>,
}

impl MockSignature {
    pub fn new(sig: Vec<u8>) -> Self {
        Self { sig }
    }
}

/// Mock for Context, useful for testing.
#[derive(Clone)]
pub struct MockContext {
    pub sender: MockPublicKey,
}

impl Spec for MockContext {
    type Storage = JmtStorage;
    type PublicKey = MockPublicKey;
    type Signature = MockSignature;
}

impl Context for MockContext {
    fn sender(&self) -> &Self::PublicKey {
        &self.sender
    }

    fn new(sender: Self::PublicKey) -> Self {
        Self { sender }
    }
}

#[derive(Clone)]
pub struct ZkMockContext {
    pub sender: MockPublicKey,
}

impl Spec for ZkMockContext {
    type Storage = ZkStorage;
    type PublicKey = MockPublicKey;
    type Signature = MockSignature;
}

impl Context for ZkMockContext {
    fn sender(&self) -> &Self::PublicKey {
        &self.sender
    }

    fn new(sender: Self::PublicKey) -> Self {
        Self { sender }
    }
}
