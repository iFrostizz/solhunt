import { BigNumberish } from 'ethers';
import { TransactionRequest } from '@ethersproject/abstract-provider';
import { JsonRpcProvider } from '@ethersproject/providers';
/**
 * wrapper class for Arachnid's deterministic deployer
 * (deterministic deployer used by 'hardhat-deployer'. generates the same addresses as "hardhat-deploy")
 */
export declare class DeterministicDeployer {
    readonly provider: JsonRpcProvider;
    /**
     * return the address this code will get deployed to.
     * @param ctrCode constructor code to pass to CREATE2
     * @param salt optional salt. defaults to zero
     */
    static getAddress(ctrCode: string, salt?: BigNumberish): Promise<string>;
    /**
     * deploy the contract, unless already deployed
     * @param ctrCode constructor code to pass to CREATE2
     * @param salt optional salt. defaults to zero
     * @return the deployed address
     */
    static deploy(ctrCode: string, salt?: BigNumberish): Promise<string>;
    proxyAddress: string;
    deploymentTransaction: string;
    deploymentSignerAddress: string;
    deploymentGasPrice: number;
    deploymentGasLimit: number;
    constructor(provider: JsonRpcProvider);
    isContractDeployed(address: string): Promise<boolean>;
    isDeployerDeployed(): Promise<boolean>;
    deployDeployer(): Promise<void>;
    getDeployTransaction(ctrCode: string, salt?: BigNumberish): Promise<TransactionRequest>;
    getDeterministicDeployAddress(ctrCode: string, salt?: BigNumberish): Promise<string>;
    deterministicDeploy(ctrCode: string, salt?: BigNumberish): Promise<string>;
    private static _instance?;
    static init(provider: JsonRpcProvider): void;
    static get instance(): DeterministicDeployer;
}
