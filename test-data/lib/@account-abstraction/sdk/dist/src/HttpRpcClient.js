"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.HttpRpcClient = void 0;
const ethers_1 = require("ethers");
const utils_1 = require("ethers/lib/utils");
const debug_1 = __importDefault(require("debug"));
const debug = (0, debug_1.default)('aa.rpc');
class HttpRpcClient {
    constructor(bundlerUrl, entryPointAddress, chainId) {
        this.bundlerUrl = bundlerUrl;
        this.entryPointAddress = entryPointAddress;
        this.chainId = chainId;
        this.userOpJsonRpcProvider = new ethers_1.ethers.providers.JsonRpcProvider(this.bundlerUrl, {
            name: 'Connected bundler network',
            chainId
        });
        this.initializing = this.validateChainId();
    }
    async validateChainId() {
        // validate chainId is in sync with expected chainid
        const chain = await this.userOpJsonRpcProvider.send('eth_chainId', []);
        const bundlerChain = parseInt(chain);
        if (bundlerChain !== this.chainId) {
            throw new Error(`bundler ${this.bundlerUrl} is on chainId ${bundlerChain}, but provider is on chainId ${this.chainId}`);
        }
    }
    /**
     * send a UserOperation to the bundler
     * @param userOp1
     * @return userOpHash the id of this operation, for getUserOperationTransaction
     */
    async sendUserOpToBundler(userOp1) {
        await this.initializing;
        const userOp = await (0, utils_1.resolveProperties)(userOp1);
        const hexifiedUserOp = Object.keys(userOp)
            .map(key => {
            let val = userOp[key];
            if (typeof val !== 'string' || !val.startsWith('0x')) {
                val = (0, utils_1.hexValue)(val);
            }
            return [key, val];
        })
            .reduce((set, [k, v]) => (Object.assign(Object.assign({}, set), { [k]: v })), {});
        const jsonRequestData = [hexifiedUserOp, this.entryPointAddress];
        await this.printUserOperation(jsonRequestData);
        return await this.userOpJsonRpcProvider
            .send('eth_sendUserOperation', [hexifiedUserOp, this.entryPointAddress]);
    }
    async printUserOperation([userOp1, entryPointAddress]) {
        const userOp = await (0, utils_1.resolveProperties)(userOp1);
        debug('sending eth_sendUserOperation', Object.assign({}, userOp
        // initCode: (userOp.initCode ?? '').length,
        // callData: (userOp.callData ?? '').length
        ), entryPointAddress);
    }
}
exports.HttpRpcClient = HttpRpcClient;
//# sourceMappingURL=HttpRpcClient.js.map