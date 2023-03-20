pragma solidity >=0.6.0;

contract From {
    function gasMeter() public returns (bool) {
        if (true) {
            return false;
        } else {
            return true;
        }
    }
}

contract To {
    function gasMeter() public returns (bool) {
        return true ? false : true;
    }
}

