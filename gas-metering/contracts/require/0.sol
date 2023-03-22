pragma solidity >=0.8.4;

contract From {
    function gasMeter() public {
        require(msg.sender != address(0), "this is expensive");
    }
}

contract To {
    error Unauthorized();

    function gasMeter() public {
        if (msg.sender == address(0)) 
            revert Unauthorized();
    }
}

