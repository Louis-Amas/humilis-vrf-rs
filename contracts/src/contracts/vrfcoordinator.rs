use alloy::{
    primitives::{keccak256, B256},
    sol,
    sol_types::SolValue,
};

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    VRFCoordinator,
    "abi/vrfcoordinator.abi.json"
);

pub fn get_request_id(random_request: &VRFCoordinator::RandomRequest) -> B256 {
    let encoded = random_request.abi_encode_packed();

    keccak256(encoded)
}

#[cfg(test)]
mod tests {
    use alloy::{
        hex::FromHex,
        primitives::{Address, B256, U256},
    };

    use super::{get_request_id, VRFCoordinator::RandomRequest};

    #[test]
    fn test_verify_request_id() {
        let random_request = RandomRequest {
            subId: 1,
            minimumRequestConfirmations: 0,
            callbackGasLimit: 0,
            numWords: 1,
            blockNumber: U256::from_str_radix("1", 10).unwrap(),
            sender: Address::from_hex("0x7FA9385bE102ac3EAc297483Dd6233D62b3e1496").unwrap(),
        };

        let request_id = get_request_id(&random_request);

        assert_eq!(
            request_id,
            B256::from_hex("1fb983642f14dbbed8a2dbd79a68c7cb33830bdf1b8474529e3b2aa0c6dc1f03")
                .unwrap()
        )
    }
}
