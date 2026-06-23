// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

contract B2AStaking {
    mapping(address => uint256) public balances;
    address public owner;

    event Deposited(address indexed user, uint256 amount);
    event Slashed(address indexed user, uint256 amount);
    event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);

    constructor() {
        owner = msg.sender;
    }

    modifier onlyOwner() {
        require(msg.sender == owner, "Not authorized");
        _;
    }

    function transferOwnership(address newOwner) public onlyOwner {
        require(newOwner != address(0), "New owner is zero address");
        emit OwnershipTransferred(owner, newOwner);
        owner = newOwner;
    }

    function deposit() public payable {
        balances[msg.sender] += msg.value;
        emit Deposited(msg.sender, msg.value);
    }

    /// Solo el owner (API / Slasher autorizado) puede llamar slash
    function slash(address user, uint256 amount) public onlyOwner {
        require(balances[user] >= amount, "Insufficient balance");
        balances[user] -= amount;
        emit Slashed(user, amount);
    }
}
