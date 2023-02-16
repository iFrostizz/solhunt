import { Signer } from "ethers";
import type { Provider } from "@ethersproject/providers";
import type { IAggregatedAccount, IAggregatedAccountInterface } from "../IAggregatedAccount";
export declare class IAggregatedAccount__factory {
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
    static createInterface(): IAggregatedAccountInterface;
    static connect(address: string, signerOrProvider: Signer | Provider): IAggregatedAccount;
}
