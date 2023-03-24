pragma solidity >=0.5.0;

contract From {
    event Hello(uint256);
    uint256 val;

    function gasMeter() public {
        val = 0;
        emit Hello(val);
    }
}

contract To {
    event Hello(uint256);
    uint256 val;

    function gasMeter() public {
        uint256 val_ = 0;
        val = val_;
        emit Hello(val_);
    }
}
