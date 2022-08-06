pragma solidity ^0.8.0;

import {Ownable} from "./Ownable.sol";

contract Lock is Ownable {
    mapping(address => uint256) funds;

    function deposit() external payable {
        funds[msg.sender] += msg.value;
    }

    function withdraw(address user) external onlyOwner {
        funds[user] = 0;
        (bool success,) = owner.call{value : funds[user]}("");
        require(success, "!success");
    }
}
