import { Signer } from "ethers";
import type { Provider } from "@ethersproject/providers";
import type { IAccount, IAccountInterface } from "../../../../@account-abstraction/contracts/interfaces/IAccount";
export declare class IAccount__factory {
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
    static createInterface(): IAccountInterface;
    static connect(address: string, signerOrProvider: Signer | Provider): IAccount;
}
