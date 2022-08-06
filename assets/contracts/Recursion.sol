pragma solidity ^0.8.0;

contract Recursion {
    function inv(uint256[] calldata arr) external pure returns (uint256[] memory){
        uint256[] memory rra = new uint256[](arr.length);

        for (uint256 i; i < arr.length; i++) {
            rra[i] = arr[arr.length - i - 1];
        }

        return rra;
    }
}
