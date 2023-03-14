pragma solidity >=0.6.0;

contract From {
    constructor() public {}

    fallback() external {}
}

contract To {
    constructor() payable public {}

    fallback() external {}
}
