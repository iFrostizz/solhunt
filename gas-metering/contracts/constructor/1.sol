pragma solidity >=0.5.0;

contract From {
    bytes32 zero = bytes32(0);
    uint256 num = 0;
    bytes val = hex"";

    constructor() public {}
}

contract To {
    bytes32 zero;
    uint256 num;
    bytes val;

    constructor() public {}
}
