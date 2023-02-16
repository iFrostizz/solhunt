"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.postExecutionCheck = exports.postExecutionDump = void 0;
const utils_1 = require("ethers/lib/utils");
async function postExecutionDump(entryPoint, userOpHash) {
    const { gasPaid, gasUsed, success, userOp } = await postExecutionCheck(entryPoint, userOpHash);
    /// / debug dump:
    console.log('==== used=', gasUsed, 'paid', gasPaid, 'over=', gasPaid - gasUsed, 'callLen=', userOp.callData.length, 'initLen=', userOp.initCode.length, success ? 'success' : 'failed');
}
exports.postExecutionDump = postExecutionDump;
/**
 * check whether an already executed UserOperation paid enough
 * (the only field that EntryPoint can check is the preVerificationGas.
 * There is no "view-mode" way to determine the actual gas cost of a given transaction,
 * so we must do it after mining it.
 * @param entryPoint
 * @param userOpHash
 */
async function postExecutionCheck(entryPoint, userOpHash) {
    const req = await entryPoint.queryFilter(entryPoint.filters.UserOperationEvent(userOpHash));
    if (req.length === 0) {
        console.log('postExecutionCheck: failed to read event (not mined)');
        // @ts-ignore
        return { gasUsed: 0, gasPaid: 0, success: false, userOp: {} };
    }
    const transactionReceipt = await req[0].getTransactionReceipt();
    const tx = await req[0].getTransaction();
    const { ops } = entryPoint.interface.decodeFunctionData('handleOps', tx.data);
    const userOp = await (0, utils_1.resolveProperties)(ops[0]);
    const { actualGasPrice, actualGasCost, success } = req[0].args;
    const gasPaid = actualGasCost.div(actualGasPrice).toNumber();
    const gasUsed = transactionReceipt.gasUsed.toNumber();
    return {
        gasUsed,
        gasPaid,
        success,
        userOp
    };
}
exports.postExecutionCheck = postExecutionCheck;
//# sourceMappingURL=postExecCheck.js.map