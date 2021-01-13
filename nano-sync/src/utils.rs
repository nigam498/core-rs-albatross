use std::cmp::min;

use ark_ff::{Field, PrimeField};
use ark_r1cs_std::fields::fp::FpVar;
use ark_r1cs_std::prelude::{Boolean, ToBitsGadget};
use ark_relations::r1cs::SynthesisError;

// Re-export bls utility functions.
pub use nimiq_bls::utils::*;

/// Takes multiple bit representations of a point (Fp/Fp2/Fp3).
/// Its length must be a multiple of the field size (in bits).
/// None of the underlying points can be zero!
/// This function pads each chunk of `MODULUS_BITS` to full bytes, prepending the `y_bit`
/// in the very front.
/// This maintains *Big-Endian* representation.
// TODO: Make this handle infinity!
pub fn pad_point_bits<F: PrimeField>(
    mut bits: Vec<Boolean<F>>,
    y_bit: Boolean<F>,
) -> Vec<Boolean<F>> {
    let point_len = F::size_in_bits();

    let padding = 8 - (point_len % 8);

    assert_eq!(
        bits.len() % point_len as usize,
        0,
        "Can only pad multiples of point size"
    );

    let mut serialization = vec![];

    // Start with y_bit.
    serialization.push(y_bit);

    let mut first = true;

    while !bits.is_empty() {
        // First, add padding.
        // If we are in the first round, skip one bit of padding.
        // The serialization begins with the y_bit, followed by the infinity flag.
        // By definition, the point must not be infinity, thus we can skip this flag.
        let padding_len = if first {
            first = false;
            padding - 1
        } else {
            padding
        };

        for _ in 0..padding_len {
            serialization.push(Boolean::constant(false));
        }

        // Then, split bits at `MODULUS_BITS`:
        // `new_bits` contains the elements in the range [MODULUS, len).
        let new_bits = bits.split_off(point_len as usize);

        serialization.append(&mut bits);

        bits = new_bits;
    }

    serialization
}

/// Takes a vector of Booleans and transforms it into a vector of a vector of Booleans, ready to be
/// transformed into field elements, which is the way we represent inputs to circuits. This assumes
/// that both the constraint field and the target field have the same size in bits (which is true
/// for the MNT curves).
/// Each field element has his last bit set to zero (since the capacity of a field is always one bit
/// less than its size). We also pad the last field element with zeros so that it has the correct
/// size.
pub fn pack_inputs<F: PrimeField>(mut input: Vec<Boolean<F>>) -> Vec<Vec<Boolean<F>>> {
    let capacity = F::size_in_bits() - 1;

    let mut result = vec![];

    while !input.is_empty() {
        let length = min(input.len(), capacity);

        let padding = F::size_in_bits() - length;

        let new_input = input.split_off(length);

        for _ in 0..padding {
            input.push(Boolean::constant(false));
        }

        result.push(input);

        input = new_input;
    }

    result
}

/// Takes a vector of public inputs to a circuit, represented as field elements, and converts it
/// to the canonical representation of a vector of Booleans. Internally, it just converts the field
/// elements to bits and discards the most significant bit (which never contains any data).
pub fn unpack_inputs<F: PrimeField>(
    inputs: Vec<FpVar<F>>,
) -> Result<Vec<Boolean<F>>, SynthesisError> {
    let mut result = vec![];

    for elem in inputs {
        let mut bits = elem.to_bits_le()?;
        bits.pop();
        result.append(&mut bits);
    }

    Ok(result)
}

/// Takes a data vector in *Big-Endian* representation and transforms it,
/// such that each byte starts with the least significant bit (as expected by blake2 gadgets).
/// b0 b1 b2 b3 b4 b5 b6 b7 b8 -> b8 b7 b6 b5 b4 b3 b2 b1 b0
pub fn reverse_inner_byte_order<F: Field>(data: &[Boolean<F>]) -> Vec<Boolean<F>> {
    assert_eq!(data.len() % 8, 0);

    data.chunks(8)
        // Reverse each 8 bit chunk.
        .flat_map(|chunk| chunk.iter().rev().cloned())
        .collect::<Vec<Boolean<F>>>()
}

/// Transforms a vector of little endian bits into a u8.
pub fn byte_from_le_bits(bits: &[bool]) -> u8 {
    assert!(bits.len() <= 8);

    let mut byte = 0;
    let mut base = 1;

    for i in 0..bits.len() {
        if bits[i] {
            byte += base;
        }
        base *= 2;
    }

    byte
}
