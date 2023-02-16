import { UserOperationStruct } from '@account-abstraction/contracts';
export declare class HttpRpcClient {
    readonly bundlerUrl: string;
    readonly entryPointAddress: string;
    readonly chainId: number;
    private readonly userOpJsonRpcProvider;
    initializing: Promise<void>;
    constructor(bundlerUrl: string, entryPointAddress: string, chainId: number);
    validateChainId(): Promise<void>;
    /**
     * send a UserOperation to the bundler
     * @param userOp1
     * @return userOpHash the id of this operation, for getUserOperationTransaction
     */
    sendUserOpToBundler(userOp1: UserOperationStruct): Promise<string>;
    private printUserOperation;
}
