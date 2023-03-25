pragma solidity >=0.8.4;

contract From {
    constructor() {}

    function one() public {
        require(msg.sender != address(0), "this is expensive");
    }

    function two() public {
        require(msg.sender != address(0), "this is expensive");
    }
}

contract To {
    constructor() {}

    function _msg() internal {
        require(msg.sender != address(0), "this is expensive");
    }

    function one() public {
        _msg();
    }

    function two() public {
        _msg();
    }
}


