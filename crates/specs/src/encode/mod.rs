use std::ops::Add;
use std::ops::Mul;

use halo2_proofs::halo2curves::ff::PrimeField;
use halo2_proofs::plonk::Expression;
use num_bigint::BigUint;

pub mod br_table;
pub mod frame_table;
pub mod image_table;
pub mod init_memory_table;
pub mod instruction_table;
pub mod memory_table;
pub mod opcode;

pub(crate) const COMMON_RANGE_OFFSET: u32 = 32;

pub trait FromBn: Sized + Add<Self, Output = Self> + Mul<Self, Output = Self> {
    fn zero() -> Self;
    fn from_bn(bn: &BigUint) -> Self;
}

impl FromBn for BigUint {
    fn zero() -> Self {
        BigUint::from(0u64)
    }

    fn from_bn(bn: &BigUint) -> Self {
        bn.clone()
    }
}

fn bn_to_field<F: PrimeField>(bn: &BigUint) -> F {
    let mut bytes = bn.to_bytes_le();
    bytes.resize(32, 0);
    let bytes = &bytes[..];
    let mut repr = F::Repr::default();
    repr.as_mut().copy_from_slice(bytes);
    F::from_repr(repr).unwrap()
}

impl<F: PrimeField> FromBn for Expression<F> {
    fn from_bn(bn: &BigUint) -> Self {
        halo2_proofs::plonk::Expression::Constant(bn_to_field(bn))
    }

    fn zero() -> Self {
        halo2_proofs::plonk::Expression::Constant(F::ZERO)
    }
}
