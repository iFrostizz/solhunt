pragma solidity >=0.5.0;

contract From {
    uint private a;

    function gasMeter() public {
        a += 1;
    }
}

contract To {
    uint private a;

    function gasMeter() public {
        a = a + 1;
    }
}
