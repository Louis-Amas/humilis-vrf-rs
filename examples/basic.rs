//! # Basic example
//!
//! This example shows a basic usage of the `vrf-rs` crate:
//!
//! 1. Instantiate the `ECVRF` by specifying the `CipherSuite`
//! 2. Generate a VRF proof by using the `prove()` function
//! 3. (Optional) Convert the VRF proof to a hash (e.g. to be used as pseudo-random value)
//! 4. Verify a VRF proof by using `verify()` function

use std::str::FromStr;

use alloy::hex::FromHex;
use alloy::primitives::{keccak256, Address, Uint, U16, U256, U32, U64};
use openssl::bn::BigNum;
use vrf::openssl::{CipherSuite, ECVRF};
use vrf::VRF;

use alloy::dyn_abi::SolType;
use alloy::sol;
use alloy::sol_types::{sol_data, SolValue};

fn main() {
    let mut vrf = ECVRF::from_suite(CipherSuite::SECP256K1_SHA256_TAI).unwrap();
    // Inputs: Secret Key, Public Key (derived) & Message
    let secret_key =
        hex::decode("c9afa9d845ba75166b5c215767b1d6934e50c3db36e89b127b8a622b120f6721").unwrap();
    let public_key = vrf.derive_public_key(&secret_key).unwrap();

    let hex_string: String = public_key
        .iter()
        .map(|byte| format!("{:02X}", byte))
        .collect::<Vec<String>>()
        .join("");
    println!("{}", hex_string);
    let message: &[u8] =
        &hex::decode("1fb983642f14dbbed8a2dbd79a68c7cb33830bdf1b8474529e3b2aa0c6dc1f03").unwrap();

    type RandomRequest = sol! { (uint64, uint16, uint32, uint32, uint256, address) };

    let values = (
        1,
        0,
        0,
        1,
        U256::from_str_radix("1", 10).unwrap(),
        Address::from_hex("0x7FA9385bE102ac3EAc297483Dd6233D62b3e1496").unwrap(),
    );

    let encoded = RandomRequest::abi_encode_packed(&values);

    let hash = keccak256(&encoded);

    println!(
        "encoding: {:?}\nhash: {:?}",
        hex::encode(encoded),
        hex::encode(hash)
    );

    // VRF proof and hash output
    let pi = vrf.prove(&secret_key, &hash.to_vec()).unwrap();
    let hash = vrf.proof_to_hash(&pi).unwrap();
    println!("Generated VRF proof: {}", hex::encode(&pi));

    // VRF proof verification (returns VRF hash output)
    let beta = vrf.verify(&public_key, &pi, &hash.to_vec());

    match beta {
        Ok(beta) => {
            println!("VRF proof is valid!\nHash output: {}", hex::encode(&beta));
            assert_eq!(hash, beta);
        }
        Err(e) => {
            println!("VRF proof is not valid: {}", e);
        }
    }
}
