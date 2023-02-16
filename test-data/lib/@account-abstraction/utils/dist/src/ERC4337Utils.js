"use strict";
var _a;
Object.defineProperty(exports, "__esModule", { value: true });
exports.rethrowError = exports.decodeErrorReason = exports.getUserOpHash = exports.packUserOp = exports.AddressZero = void 0;
const utils_1 = require("ethers/lib/utils");
const IEntryPoint_json_1 = require("@account-abstraction/contracts/artifacts/IEntryPoint.json");
const ethers_1 = require("ethers");
exports.AddressZero = ethers_1.ethers.constants.AddressZero;
// UserOperation is the first parameter of simulateValidation
const UserOpType = (_a = IEntryPoint_json_1.abi.find(entry => entry.name === 'simulateValidation')) === null || _a === void 0 ? void 0 : _a.inputs[0];
function encode(typevalues, forSignature) {
    const types = typevalues.map(typevalue => typevalue.type === 'bytes' && forSignature ? 'bytes32' : typevalue.type);
    const values = typevalues.map((typevalue) => typevalue.type === 'bytes' && forSignature ? (0, utils_1.keccak256)(typevalue.val) : typevalue.val);
    return utils_1.defaultAbiCoder.encode(types, values);
}
/**
 * pack the userOperation
 * @param op
 * @param forSignature "true" if the hash is needed to calculate the getUserOpHash()
 *  "false" to pack entire UserOp, for calculating the calldata cost of putting it on-chain.
 */
function packUserOp(op, forSignature = true) {
    if (forSignature) {
        // lighter signature scheme (must match UserOperation#pack): do encode a zero-length signature, but strip afterwards the appended zero-length value
        const userOpType = {
            components: [
                {
                    type: 'address',
                    name: 'sender'
                },
                {
                    type: 'uint256',
                    name: 'nonce'
                },
                {
                    type: 'bytes',
                    name: 'initCode'
                },
                {
                    type: 'bytes',
                    name: 'callData'
                },
                {
                    type: 'uint256',
                    name: 'callGasLimit'
                },
                {
                    type: 'uint256',
                    name: 'verificationGasLimit'
                },
                {
                    type: 'uint256',
                    name: 'preVerificationGas'
                },
                {
                    type: 'uint256',
                    name: 'maxFeePerGas'
                },
                {
                    type: 'uint256',
                    name: 'maxPriorityFeePerGas'
                },
                {
                    type: 'bytes',
                    name: 'paymasterAndData'
                },
                {
                    type: 'bytes',
                    name: 'signature'
                }
            ],
            name: 'userOp',
            type: 'tuple'
        };
        // console.log('hard-coded userOpType', userOpType)
        // console.log('from ABI userOpType', UserOpType)
        let encoded = utils_1.defaultAbiCoder.encode([userOpType], [Object.assign(Object.assign({}, op), { signature: '0x' })]);
        // remove leading word (total length) and trailing word (zero-length signature)
        encoded = '0x' + encoded.slice(66, encoded.length - 64);
        return encoded;
    }
    const typevalues = UserOpType.components.map((c) => ({
        type: c.type,
        val: op[c.name]
    }));
    // const typevalues = [
    //   {
    //     type: 'address',
    //     val: op.sender
    //   },
    //   {
    //     type: 'uint256',
    //     val: op.nonce
    //   },
    //   {
    //     type: 'bytes',
    //     val: op.initCode
    //   },
    //   {
    //     type: 'bytes',
    //     val: op.callData
    //   },
    //   {
    //     type: 'uint256',
    //     val: op.callGasLimit
    //   },
    //   {
    //     type: 'uint256',
    //     val: op.verificationGasLimit
    //   },
    //   {
    //     type: 'uint256',
    //     val: op.preVerificationGas
    //   },
    //   {
    //     type: 'uint256',
    //     val: op.maxFeePerGas
    //   },
    //   {
    //     type: 'uint256',
    //     val: op.maxPriorityFeePerGas
    //   },
    //   {
    //     type: 'bytes',
    //     val: op.paymasterAndData
    //   }
    // ]
    // console.log('hard-coded typedvalues', typevalues)
    // console.log('from ABI typedValues', typedValues)
    if (!forSignature) {
        // for the purpose of calculating gas cost, also hash signature
        typevalues.push({
            type: 'bytes',
            val: op.signature
        });
    }
    return encode(typevalues, forSignature);
}
exports.packUserOp = packUserOp;
/**
 * calculate the userOpHash of a given userOperation.
 * The userOpHash is a hash of all UserOperation fields, except the "signature" field.
 * The entryPoint uses this value in the emitted UserOperationEvent.
 * A wallet may use this value as the hash to sign (the SampleWallet uses this method)
 * @param op
 * @param entryPoint
 * @param chainId
 */
function getUserOpHash(op, entryPoint, chainId) {
    const userOpHash = (0, utils_1.keccak256)(packUserOp(op, true));
    const enc = utils_1.defaultAbiCoder.encode(['bytes32', 'address', 'uint256'], [userOpHash, entryPoint, chainId]);
    return (0, utils_1.keccak256)(enc);
}
exports.getUserOpHash = getUserOpHash;
const ErrorSig = (0, utils_1.keccak256)(Buffer.from('Error(string)')).slice(0, 10); // 0x08c379a0
const FailedOpSig = (0, utils_1.keccak256)(Buffer.from('FailedOp(uint256,address,string)')).slice(0, 10); // 0x00fa072b
/**
 * decode bytes thrown by revert as Error(message) or FailedOp(opIndex,paymaster,message)
 */
function decodeErrorReason(error) {
    // console.log('decoding', error)
    if (error.startsWith(ErrorSig)) {
        const [message] = utils_1.defaultAbiCoder.decode(['string'], '0x' + error.substring(10));
        return { message };
    }
    else if (error.startsWith(FailedOpSig)) {
        let [opIndex, paymaster, message] = utils_1.defaultAbiCoder.decode(['uint256', 'address', 'string'], '0x' + error.substring(10));
        message = `FailedOp: ${message}`;
        if (paymaster.toString() !== ethers_1.ethers.constants.AddressZero) {
            message = `${message} (paymaster ${paymaster})`;
        }
        else {
            paymaster = undefined;
        }
        return {
            message,
            opIndex,
            paymaster
        };
    }
}
exports.decodeErrorReason = decodeErrorReason;
/**
 * update thrown Error object with our custom FailedOp message, and re-throw it.
 * updated both "message" and inner encoded "data"
 * tested on geth, hardhat-node
 * usage: entryPoint.handleOps().catch(decodeError)
 */
function rethrowError(e) {
    let error = e;
    let parent = e;
    if ((error === null || error === void 0 ? void 0 : error.error) != null) {
        error = error.error;
    }
    while ((error === null || error === void 0 ? void 0 : error.data) != null) {
        parent = error;
        error = error.data;
    }
    const decoded = typeof error === 'string' && error.length > 2 ? decodeErrorReason(error) : undefined;
    if (decoded != null) {
        e.message = decoded.message;
        if (decoded.opIndex != null) {
            // helper for chai: convert our FailedOp error into "Error(msg)"
            const errorWithMsg = (0, utils_1.hexConcat)([ErrorSig, utils_1.defaultAbiCoder.encode(['string'], [decoded.message])]);
            // modify in-place the error object:
            parent.data = errorWithMsg;
        }
    }
    throw e;
}
exports.rethrowError = rethrowError;
//# sourceMappingURL=ERC4337Utils.js.map