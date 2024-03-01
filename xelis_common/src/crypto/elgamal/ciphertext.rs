use std::ops::{Add, AddAssign, Sub, SubAssign};

use curve25519_dalek::{traits::Identity, RistrettoPoint, Scalar};
use super::{pedersen::{DecryptHandle, PedersenCommitment}, CompressedCiphertext, CompressedCommitment};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ciphertext {
    commitment: PedersenCommitment,
    handle: DecryptHandle,
}

impl Ciphertext {
    pub fn new(commitment: PedersenCommitment, handle: DecryptHandle) -> Self {
        Self { commitment, handle }
    }
    
    // Create a ciphertext with a zero value
    pub fn zero() -> Self {
        Self {
            commitment: PedersenCommitment::from_point(RistrettoPoint::identity()),
            handle: DecryptHandle::from_point(RistrettoPoint::identity()),
        }
    }

    pub fn commitment(&self) -> &PedersenCommitment {
        &self.commitment
    }

    pub fn handle(&self) -> &DecryptHandle {
        &self.handle
    }

    pub fn compress(&self) -> CompressedCiphertext {
        CompressedCiphertext::new(
            CompressedCommitment::new(self.commitment.as_point().compress()),
            self.handle.as_point().compress()
        )
    }
}

// ADD TRAITS

impl Add for Ciphertext {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self {
            commitment: self.commitment + rhs.commitment,
            handle: self.handle + rhs.handle,
        }
    }
}

impl Add<&Ciphertext> for Ciphertext {
    type Output = Self;

    fn add(self, rhs: &Self) -> Self {
        Self {
            commitment: self.commitment + &rhs.commitment,
            handle: self.handle + &rhs.handle,
        }
    }
}

impl Add<Scalar> for Ciphertext {
    type Output = Self;
    fn add(self, rhs: Scalar) -> Self {
        Self {
            commitment: self.commitment + rhs,
            handle: self.handle,
        }
    }
}

impl Add<&Scalar> for Ciphertext {
    type Output = Self;
    fn add(self, rhs: &Scalar) -> Self {
        Self {
            commitment: self.commitment + rhs,
            handle: self.handle,
        }
    }
}

// ADD ASSIGN TRAITS

impl AddAssign for Ciphertext {
    fn add_assign(&mut self, rhs: Self) {
        self.commitment += rhs.commitment;
        self.handle += rhs.handle;
    }
}

impl AddAssign<&Ciphertext> for Ciphertext {
    fn add_assign(&mut self, rhs: &Self) {
        self.commitment += &rhs.commitment;
        self.handle += &rhs.handle;
    }
}

impl AddAssign<Scalar> for Ciphertext {
    fn add_assign(&mut self, rhs: Scalar) {
        self.commitment += rhs;
    }
}

impl AddAssign<&Scalar> for Ciphertext {
    fn add_assign(&mut self, rhs: &Scalar) {
        self.commitment += rhs;
    }
}

// SUB TRAITS

impl Sub for Ciphertext {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self {
            commitment: self.commitment - rhs.commitment,
            handle: self.handle - rhs.handle,
        }
    }
}

impl Sub<&Ciphertext> for Ciphertext {
    type Output = Self;

    fn sub(self, rhs: &Self) -> Self {
        Self {
            commitment: self.commitment - &rhs.commitment,
            handle: self.handle - &rhs.handle,
        }
    }
}

impl Sub<Scalar> for Ciphertext {
    type Output = Self;
    fn sub(self, rhs: Scalar) -> Self {
        Self {
            commitment: self.commitment - rhs,
            handle: self.handle,
        }
    }
}

impl Sub<&Scalar> for Ciphertext {
    type Output = Self;
    fn sub(self, rhs: &Scalar) -> Self {
        Self {
            commitment: self.commitment - rhs,
            handle: self.handle,
        }
    }
}

// SUB ASSIGN TRAITS

impl SubAssign for Ciphertext {
    fn sub_assign(&mut self, rhs: Self) {
        self.commitment -= rhs.commitment;
        self.handle -= rhs.handle;
    }
}

impl SubAssign<&Ciphertext> for Ciphertext {
    fn sub_assign(&mut self, rhs: &Self) {
        self.commitment -= &rhs.commitment;
        self.handle -= &rhs.handle;
    }
}

impl SubAssign<Scalar> for Ciphertext {
    fn sub_assign(&mut self, rhs: Scalar) {
        self.commitment -= rhs;
    }
}

impl SubAssign<&Scalar> for Ciphertext {
    fn sub_assign(&mut self, rhs: &Scalar) {
        self.commitment -= rhs;
    }
}