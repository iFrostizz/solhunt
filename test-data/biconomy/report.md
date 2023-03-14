# Solhunt report
## Findings summary
Name | Finding | Instances
--- | --- | ---
[M-1] | Centralization of power | 21
[M-2] | extcodesize for EOA test | 1
[L-1] | Unlocked compiler pragma | 13
[L-2] | Usage of abi.encodePacked() with dynamic types | 10
[G-1] | Avoid using public for state variables | 16
[G-2] | Avoid using compound operators with state variables | 7
[G-3] | Using a state variable in an event emission wastes gas | 7
[G-4] | `abi.encode()` is less efficient than `abi.encodepacked()` | 7
[G-5] | address(0) check | 49
[G-6] | Setting the constructor to payable | 6
[G-7] | Use custom errors instead of revert strings | 94
[G-8] | Duplicated require()/revert() Checks Should Be Refactored To A Modifier Or an internal function | 28
[G-9] | Use `require` instead of `assert` | 2
[G-10] | State variables that never change should be directly inlined in the bytecode | 2
[G-11] | `internal` functions only called once can be inlined to save gas | 34

## Findings details
### [M-1] Centralization of power


Contracts have owners with privileged rights to perform admin tasks and need to be trusted to not perform malicious updates or drain funds.

`paymasters/BasePaymaster.sol`
0:0

```solidity
```

`paymasters/BasePaymaster.sol`
0:0

```solidity
```

`paymasters/BasePaymaster.sol`
0:0

```solidity
```

`paymasters/BasePaymaster.sol`
0:0

```solidity
```

`paymasters/BasePaymaster.sol`
0:0

```solidity
```

`SmartAccount.sol`
0:0

```solidity
```

`SmartAccount.sol`
0:0

```solidity
```

`SmartAccount.sol`
0:0

```solidity
```

`SmartAccount.sol`
0:0

```solidity
```

`SmartAccount.sol`
0:0

```solidity
```

### [M-2] extcodesize for EOA test


using extcodesize. Can be an issue if determining if EOA.

`libs/LibAddress.sol`
0:0

```solidity
```

### [L-1] Unlocked compiler pragma


Unspecific compiler version pragma. Please lock the compiler version to avoid unexpected compilation results

`aa-4337/core/BaseAccount.sol`
0:0

```solidity
```

`paymasters/BasePaymaster.sol`
0:0

```solidity
```

`aa-4337/core/EntryPoint.sol`
0:0

```solidity
```

`aa-4337/utils/Exec.sol`
0:0

```solidity
```

`aa-4337/interfaces/IAccount.sol`
0:0

```solidity
```

`aa-4337/interfaces/IAggregatedAccount.sol`
0:0

```solidity
```

`aa-4337/interfaces/IEntryPoint.sol`
0:0

```solidity
```

`aa-4337/interfaces/IPaymaster.sol`
0:0

```solidity
```

`aa-4337/interfaces/IStakeManager.sol`
0:0

```solidity
```

`aa-4337/core/SenderCreator.sol`
0:0

```solidity
```

### [L-2] Usage of abi.encodePacked() with dynamic types


`abi.encodePacked()` should not be used with dynamic types when passing the result to a hash function such as `keccak256()`. Use `abi.encode()` instead which will pad items to 32 bytes, which will [prevent hash collisions](https://docs.soliditylang.org/en/v0.8.13/abi-spec.html#non-standard-packed-mode) (e.g. `abi.encodePacked(0x123,0x456)` => `0x123456` => `abi.encodePacked(0x1,0x23456)`, but `abi.encode(0x123,0x456)` => `0x0...1230...456`). "Unless there is a compelling reason, `abi.encode` should be preferred". 

`SmartAccount.sol`
0:0

```solidity
```

`SmartAccount.sol`
0:0

```solidity
```

`SmartAccount.sol`
0:0

```solidity
```

`SmartAccountFactory.sol`
0:0

```solidity
```

`SmartAccountFactory.sol`
0:0

```solidity
```

`SmartAccountFactory.sol`
0:0

```solidity
```

`SmartAccountFactory.sol`
0:0

```solidity
```

`SmartAccountFactory.sol`
0:0

```solidity
```

`SmartAccountFactory.sol`
0:0

```solidity
```

`SmartAccountNoAuth.sol`
0:0

```solidity
```

### [G-1] Avoid using public for state variables


Public state variable are generating a getter function which costs more gas on deployment. Some variables may not need to require a getter function.

`paymasters/BasePaymaster.sol`
0:0

```solidity
```

`handler/DefaultCallbackHandler.sol`
0:0

```solidity
```

`handler/DefaultCallbackHandler.sol`
0:0

```solidity
```

`SmartAccount.sol`
0:0

```solidity
```

`SmartAccount.sol`
0:0

```solidity
```

`SmartAccount.sol`
0:0

```solidity
```

`SmartAccountFactory.sol`
0:0

```solidity
```

`SmartAccountFactory.sol`
0:0

```solidity
```

`SmartAccountFactory.sol`
0:0

```solidity
```

`SmartAccountNoAuth.sol`
0:0

```solidity
```

### [G-2] Avoid using compound operators with state variables


+= and -= are more expensive than = + and = -

`aa-4337/core/EntryPoint.sol`
0:0

```solidity
```

`aa-4337/core/EntryPoint.sol`
0:0

```solidity
```

`aa-4337/core/EntryPoint.sol`
0:0

```solidity
```

`aa-4337/core/EntryPoint.sol`
0:0

```solidity
```

`paymasters/verifying/singleton/VerifyingSingletonPaymaster.sol`
0:0

```solidity
```

`paymasters/verifying/singleton/VerifyingSingletonPaymaster.sol`
0:0

```solidity
```

`paymasters/verifying/singleton/VerifyingSingletonPaymaster.sol`
0:0

```solidity
```

### [G-3] Using a state variable in an event emission wastes gas


A state variable should not be used in an event emission because it will load it from the storage. It should rather be loaded from the stack or the memory.

`SmartAccount.sol`
0:0

```solidity
```

`SmartAccountFactory.sol`
0:0

```solidity
```

`SmartAccountFactory.sol`
0:0

```solidity
```

`SmartAccountFactory.sol`
0:0

```solidity
```

`SmartAccountNoAuth.sol`
0:0

```solidity
```

`SmartAccountNoAuth.sol`
0:0

```solidity
```

`SmartAccountNoAuth.sol`
0:0

```solidity
```

### [G-4] `abi.encode()` is less efficient than `abi.encodepacked()`


see: https://github.com/ConnorBlockchain/Solidity-Encode-Gas-Comparison

`aa-4337/core/EntryPoint.sol`
0:0

```solidity
```

`paymasters/PaymasterHelpers.sol`
0:0

```solidity
```

`SmartAccount.sol`
0:0

```solidity
```

`SmartAccount.sol`
0:0

```solidity
```

`SmartAccountNoAuth.sol`
0:0

```solidity
```

`SmartAccountNoAuth.sol`
0:0

```solidity
```

`paymasters/verifying/singleton/VerifyingSingletonPaymaster.sol`
0:0

```solidity
```

### [G-5] address(0) check


Use assembly to check for `address(0)`, *Saves 6 gas per instance*

`aa-4337/core/EntryPoint.sol`
0:0

```solidity
```

`aa-4337/core/EntryPoint.sol`
0:0

```solidity
```

`aa-4337/core/EntryPoint.sol`
0:0

```solidity
```

`aa-4337/core/EntryPoint.sol`
0:0

```solidity
```

`aa-4337/core/EntryPoint.sol`
0:0

```solidity
```

`aa-4337/core/EntryPoint.sol`
0:0

```solidity
```

`aa-4337/core/EntryPoint.sol`
0:0

```solidity
```

`aa-4337/core/EntryPoint.sol`
0:0

```solidity
```

`base/ModuleManager.sol`
0:0

```solidity
```

`base/ModuleManager.sol`
0:0

```solidity
```

### [G-6] Setting the constructor to payable


Saves ~13 gas per instance

`paymasters/BasePaymaster.sol`
0:0

```solidity
```

`libs/MultiSend.sol`
0:0

```solidity
```

`Proxy.sol`
0:0

```solidity
```

`SmartAccountFactory.sol`
0:0

```solidity
```

`paymasters/verifying/singleton/VerifyingSingletonPaymaster.sol`
0:0

```solidity
```

`aa-4337/core/BasePaymaster.sol`
0:0

```solidity
```

### [G-7] Use custom errors instead of revert strings


Solidity 0.8.4 added the custom errors functionality, which can be use instead of revert strings, resulting in big gas savings on errors. Replace all revert statements with custom error ones

`aa-4337/core/BaseAccount.sol`
0:0

```solidity
```

`paymasters/BasePaymaster.sol`
0:0

```solidity
```

`BaseSmartAccount.sol`
0:0

```solidity
```

`utils/Decoder.sol`
0:0

```solidity
```

`aa-4337/core/EntryPoint.sol`
0:0

```solidity
```

`aa-4337/core/EntryPoint.sol`
0:0

```solidity
```

`aa-4337/core/EntryPoint.sol`
0:0

```solidity
```

`aa-4337/core/EntryPoint.sol`
0:0

```solidity
```

`aa-4337/core/EntryPoint.sol`
0:0

```solidity
```

`base/ModuleManager.sol`
0:0

```solidity
```

### [G-8] Duplicated require()/revert() Checks Should Be Refactored To A Modifier Or an internal function


Less code means a less costly deployment

`base/ModuleManager.sol`
0:0

```solidity
```

`base/ModuleManager.sol`
0:0

```solidity
```

`SmartAccount.sol`
0:0

```solidity
```

`SmartAccount.sol`
0:0

```solidity
```

`SmartAccount.sol`
0:0

```solidity
```

`SmartAccount.sol`
0:0

```solidity
```

`SmartAccount.sol`
0:0

```solidity
```

`SmartAccount.sol`
0:0

```solidity
```

`SmartAccount.sol`
0:0

```solidity
```

`SmartAccount.sol`
0:0

```solidity
```

### [G-9] Use `require` instead of `assert`


assert wastes all the transaction gas. Use require instead

`Proxy.sol`
0:0

```solidity
```

`common/Singleton.sol`
0:0

```solidity
```

### [G-10] State variables that never change should be directly inlined in the bytecode


When state variables are guaranteed to never change, they should be inlined in the bytecode of the contract by declaring them as immutables or constants to avoid paying the upfront cost of SLOAD which are expensive, mainly when the slot is cold.

`libs/MultiSend.sol`
0:0

```solidity
```

`SmartAccountFactory.sol`
0:0

```solidity
```

### [G-11] `internal` functions only called once can be inlined to save gas


Not inlining costs 20 to 40 gas because of two extra JUMP instructions and additional stack operations needed for function calls.

`aa-4337/core/BaseAccount.sol`
0:0

```solidity
```

`aa-4337/core/BaseAccount.sol`
0:0

```solidity
```

`aa-4337/core/BaseAccount.sol`
0:0

```solidity
```

`paymasters/BasePaymaster.sol`
0:0

```solidity
```

`paymasters/BasePaymaster.sol`
0:0

```solidity
```

`BaseSmartAccount.sol`
0:0

```solidity
```

`BaseSmartAccount.sol`
0:0

```solidity
```

`BaseSmartAccount.sol`
0:0

```solidity
```

`aa-4337/core/EntryPoint.sol`
0:0

```solidity
```

`aa-4337/core/EntryPoint.sol`
0:0

```solidity
```

