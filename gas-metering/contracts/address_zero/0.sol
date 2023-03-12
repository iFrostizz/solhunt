pragma solidity >=0.6.0;

contract From {
    function gasMeter() public {
        if (msg.sender == address(0)) {
        }
    }
}

contract To {
    function gasMeter() public {
        assembly {
            if iszero(caller()) {
            }
        }
    }
}
