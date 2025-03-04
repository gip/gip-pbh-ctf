use alloy_sol_types::sol;

sol! {
    #[sol(rpc)]
    interface ConsumeGas {
        function consumeGas(address address, uint256 iterations) external;
    }

    #[sol(rpc)]
    interface IPBHEntryPoint {
        function numPbhPerMonth() external view returns (uint16);
        function nullifierHashes(uint256) external view returns (bool);
    }
}
