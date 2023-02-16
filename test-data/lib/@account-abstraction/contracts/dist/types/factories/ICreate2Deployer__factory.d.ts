import { Signer } from "ethers";
import type { Provider } from "@ethersproject/providers";
import type { ICreate2Deployer, ICreate2DeployerInterface } from "../ICreate2Deployer";
export declare class ICreate2Deployer__factory {
    static readonly abi: {
        inputs: {
            internalType: string;
            name: string;
            type: string;
        }[];
        name: string;
        outputs: {
            internalType: string;
            name: string;
            type: string;
        }[];
        stateMutability: string;
        type: string;
    }[];
    static createInterface(): ICreate2DeployerInterface;
    static connect(address: string, signerOrProvider: Signer | Provider): ICreate2Deployer;
}
