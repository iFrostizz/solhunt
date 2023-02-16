"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.wrapProvider = void 0;
const contracts_1 = require("@account-abstraction/contracts");
const SimpleAccountAPI_1 = require("./SimpleAccountAPI");
const ERC4337EthersProvider_1 = require("./ERC4337EthersProvider");
const HttpRpcClient_1 = require("./HttpRpcClient");
const DeterministicDeployer_1 = require("./DeterministicDeployer");
const debug_1 = __importDefault(require("debug"));
const debug = (0, debug_1.default)('aa.wrapProvider');
/**
 * wrap an existing provider to tunnel requests through Account Abstraction.
 * @param originalProvider the normal provider
 * @param config see ClientConfig for more info
 * @param originalSigner use this signer as the owner. of this wallet. By default, use the provider's signer
 */
async function wrapProvider(originalProvider, config, originalSigner = originalProvider.getSigner()) {
    const entryPoint = contracts_1.EntryPoint__factory.connect(config.entryPointAddress, originalProvider);
    // Initial SimpleAccount instance is not deployed and exists just for the interface
    const detDeployer = new DeterministicDeployer_1.DeterministicDeployer(originalProvider);
    const simpleAccountDeployer = await detDeployer.deterministicDeploy(contracts_1.SimpleAccountDeployer__factory.bytecode);
    const smartAccountAPI = new SimpleAccountAPI_1.SimpleAccountAPI({
        provider: originalProvider,
        entryPointAddress: entryPoint.address,
        owner: originalSigner,
        factoryAddress: simpleAccountDeployer,
        paymasterAPI: config.paymasterAPI
    });
    debug('config=', config);
    const chainId = await originalProvider.getNetwork().then(net => net.chainId);
    const httpRpcClient = new HttpRpcClient_1.HttpRpcClient(config.bundlerUrl, config.entryPointAddress, chainId);
    return await new ERC4337EthersProvider_1.ERC4337EthersProvider(chainId, config, originalSigner, originalProvider, httpRpcClient, entryPoint, smartAccountAPI).init();
}
exports.wrapProvider = wrapProvider;
//# sourceMappingURL=Provider.js.map