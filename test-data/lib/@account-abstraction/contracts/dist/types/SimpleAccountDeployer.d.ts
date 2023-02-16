import type { BaseContract, BigNumber, BigNumberish, BytesLike, CallOverrides, ContractTransaction, Overrides, PopulatedTransaction, Signer, utils } from "ethers";
import type { FunctionFragment, Result } from "@ethersproject/abi";
import type { Listener, Provider } from "@ethersproject/providers";
import type { TypedEventFilter, TypedEvent, TypedListener, OnEvent, PromiseOrValue } from "./common";
export interface SimpleAccountDeployerInterface extends utils.Interface {
    functions: {
        "deployAccount(address,address,uint256)": FunctionFragment;
        "getAddress(address,address,uint256)": FunctionFragment;
    };
    getFunction(nameOrSignatureOrTopic: "deployAccount" | "getAddress"): FunctionFragment;
    encodeFunctionData(functionFragment: "deployAccount", values: [
        PromiseOrValue<string>,
        PromiseOrValue<string>,
        PromiseOrValue<BigNumberish>
    ]): string;
    encodeFunctionData(functionFragment: "getAddress", values: [
        PromiseOrValue<string>,
        PromiseOrValue<string>,
        PromiseOrValue<BigNumberish>
    ]): string;
    decodeFunctionResult(functionFragment: "deployAccount", data: BytesLike): Result;
    decodeFunctionResult(functionFragment: "getAddress", data: BytesLike): Result;
    events: {};
}
export interface SimpleAccountDeployer extends BaseContract {
    connect(signerOrProvider: Signer | Provider | string): this;
    attach(addressOrName: string): this;
    deployed(): Promise<this>;
    interface: SimpleAccountDeployerInterface;
    queryFilter<TEvent extends TypedEvent>(event: TypedEventFilter<TEvent>, fromBlockOrBlockhash?: string | number | undefined, toBlock?: string | number | undefined): Promise<Array<TEvent>>;
    listeners<TEvent extends TypedEvent>(eventFilter?: TypedEventFilter<TEvent>): Array<TypedListener<TEvent>>;
    listeners(eventName?: string): Array<Listener>;
    removeAllListeners<TEvent extends TypedEvent>(eventFilter: TypedEventFilter<TEvent>): this;
    removeAllListeners(eventName?: string): this;
    off: OnEvent<this>;
    on: OnEvent<this>;
    once: OnEvent<this>;
    removeListener: OnEvent<this>;
    functions: {
        deployAccount(entryPoint: PromiseOrValue<string>, owner: PromiseOrValue<string>, salt: PromiseOrValue<BigNumberish>, overrides?: Overrides & {
            from?: PromiseOrValue<string>;
        }): Promise<ContractTransaction>;
        getAddress(entryPoint: PromiseOrValue<string>, owner: PromiseOrValue<string>, salt: PromiseOrValue<BigNumberish>, overrides?: CallOverrides): Promise<[string]>;
    };
    deployAccount(entryPoint: PromiseOrValue<string>, owner: PromiseOrValue<string>, salt: PromiseOrValue<BigNumberish>, overrides?: Overrides & {
        from?: PromiseOrValue<string>;
    }): Promise<ContractTransaction>;
    getAddress(entryPoint: PromiseOrValue<string>, owner: PromiseOrValue<string>, salt: PromiseOrValue<BigNumberish>, overrides?: CallOverrides): Promise<string>;
    callStatic: {
        deployAccount(entryPoint: PromiseOrValue<string>, owner: PromiseOrValue<string>, salt: PromiseOrValue<BigNumberish>, overrides?: CallOverrides): Promise<string>;
        getAddress(entryPoint: PromiseOrValue<string>, owner: PromiseOrValue<string>, salt: PromiseOrValue<BigNumberish>, overrides?: CallOverrides): Promise<string>;
    };
    filters: {};
    estimateGas: {
        deployAccount(entryPoint: PromiseOrValue<string>, owner: PromiseOrValue<string>, salt: PromiseOrValue<BigNumberish>, overrides?: Overrides & {
            from?: PromiseOrValue<string>;
        }): Promise<BigNumber>;
        getAddress(entryPoint: PromiseOrValue<string>, owner: PromiseOrValue<string>, salt: PromiseOrValue<BigNumberish>, overrides?: CallOverrides): Promise<BigNumber>;
    };
    populateTransaction: {
        deployAccount(entryPoint: PromiseOrValue<string>, owner: PromiseOrValue<string>, salt: PromiseOrValue<BigNumberish>, overrides?: Overrides & {
            from?: PromiseOrValue<string>;
        }): Promise<PopulatedTransaction>;
        getAddress(entryPoint: PromiseOrValue<string>, owner: PromiseOrValue<string>, salt: PromiseOrValue<BigNumberish>, overrides?: CallOverrides): Promise<PopulatedTransaction>;
    };
}
