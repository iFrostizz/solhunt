pragma solidity >=0.5.0;

contract From {
    uint256 value = 1;

    function gasMeter() public returns (uint256) {
        return value;
    }
}

contract To {
    uint256 constant value = 1;

    function gasMeter() public returns (uint256) {
        return value;
    }
}
