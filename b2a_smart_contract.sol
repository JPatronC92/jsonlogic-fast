// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

contract B2AStaking {
    mapping(address => uint256) public balances;

    event Deposited(address indexed user, uint256 amount);
    event Slashed(address indexed user, uint256 amount);

    function deposit() public payable {
        balances[msg.sender] += msg.value;
        emit Deposited(msg.sender, msg.value);
    }

    // Only owner/API can call this in reality, but simplified for mock
    function slash(address user, uint256 amount) public {
        require(balances[user] >= amount, "Insufficient balance");
        balances[user] -= amount;
        emit Slashed(user, amount);
    }
}
