pragma solidity ^0.8.0;

contract Storage {
    uint256 value;

    function store(uint256 val) public {
        value = val;
    }

    function get() public returns (uint256) {
        return value;
    }
}
