import { Signer, ContractFactory, Overrides } from "ethers";
import type { Provider, TransactionRequest } from "@ethersproject/providers";
import type { PromiseOrValue } from "../../../common";
import type { SingletonFactory, SingletonFactoryInterface } from "../../../contracts/test/SingletonFactory";
type SingletonFactoryConstructorParams = [signer?: Signer] | ConstructorParameters<typeof ContractFactory>;
export declare class SingletonFactory__factory extends ContractFactory {
    constructor(...args: SingletonFactoryConstructorParams);
    deploy(overrides?: Overrides & {
        from?: PromiseOrValue<string>;
    }): Promise<SingletonFactory>;
    getDeployTransaction(overrides?: Overrides & {
        from?: PromiseOrValue<string>;
    }): TransactionRequest;
    attach(address: string): SingletonFactory;
    connect(signer: Signer): SingletonFactory__factory;
    static readonly bytecode = "0x608060405234801561001057600080fd5b50610173806100206000396000f3fe608060405234801561001057600080fd5b506004361061002b5760003560e01c80634af63f0214610030575b600080fd5b61004361003e366004610088565b61005f565b6040516001600160a01b03909116815260200160405180910390f35b6000818351602085016000f59392505050565b634e487b7160e01b600052604160045260246000fd5b6000806040838503121561009b57600080fd5b823567ffffffffffffffff808211156100b357600080fd5b818501915085601f8301126100c757600080fd5b8135818111156100d9576100d9610072565b604051601f8201601f19908116603f0116810190838211818310171561010157610101610072565b8160405282815288602084870101111561011a57600080fd5b82602086016020830137600060209382018401529896909101359650505050505056fea2646970667358221220942f273f033772d703ba03c258b70c24fa13dd7248a7d6289b6f626bb6a62c5d64736f6c634300080f0033";
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
    static createInterface(): SingletonFactoryInterface;
    static connect(address: string, signerOrProvider: Signer | Provider): SingletonFactory;
}
export {};
