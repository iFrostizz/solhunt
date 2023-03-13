pragma solidity >=0.6.0;

contract From {
    event MyGas();

    function gasMeter() public {
        _onlyInternal();
    }

    function _onlyInternal() internal {
         if (msg.sender == address(0)) {
              emit MyGas();
         } else {
              emit MyGas();
         }
    }
}

contract To {
    event MyGas();

    function gasMeter() public {
       if (msg.sender == address(0)) {
          emit MyGas();
       } else {
          emit MyGas();
       }
    }
}
