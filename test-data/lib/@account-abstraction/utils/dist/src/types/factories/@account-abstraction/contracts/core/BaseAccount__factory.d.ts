import { Signer } from "ethers";
import type { Provider } from "@ethersproject/providers";
import type { BaseAccount, BaseAccountInterface } from "../../../../@account-abstraction/contracts/core/BaseAccount";
export declare class BaseAccount__factory {
    static readonly abi: {
        inputs: ({
            components: {
                internalType: string;
                name: string;
                type: string;
            }[];
            internalType: string;
            name: string;
            type: string;
        } | {
            internalType: string;
            name: string;
            type: string;
            components?: undefined;
        })[];
        name: string;
        outputs: {
            internalType: string;
            name: string;
            type: string;
        }[];
        stateMutability: string;
        type: string;
    }[];
    static createInterface(): BaseAccountInterface;
    static connect(address: string, signerOrProvider: Signer | Provider): BaseAccount;
}
