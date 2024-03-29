import type { BaseContract, BigNumber, BigNumberish, BytesLike, CallOverrides, ContractTransaction, Overrides, PayableOverrides, PopulatedTransaction, Signer, utils } from "ethers";
import type { FunctionFragment, Result, EventFragment } from "@ethersproject/abi";
import type { Listener, Provider } from "@ethersproject/providers";
import type { TypedEventFilter, TypedEvent, TypedListener, OnEvent, PromiseOrValue } from "./common";
export declare type UserOperationStruct = {
    sender: PromiseOrValue<string>;
    nonce: PromiseOrValue<BigNumberish>;
    initCode: PromiseOrValue<BytesLike>;
    callData: PromiseOrValue<BytesLike>;
    callGasLimit: PromiseOrValue<BigNumberish>;
    verificationGasLimit: PromiseOrValue<BigNumberish>;
    preVerificationGas: PromiseOrValue<BigNumberish>;
    maxFeePerGas: PromiseOrValue<BigNumberish>;
    maxPriorityFeePerGas: PromiseOrValue<BigNumberish>;
    paymasterAndData: PromiseOrValue<BytesLike>;
    signature: PromiseOrValue<BytesLike>;
};
export declare type UserOperationStructOutput = [
    string,
    BigNumber,
    string,
    string,
    BigNumber,
    BigNumber,
    BigNumber,
    BigNumber,
    BigNumber,
    string,
    string
] & {
    sender: string;
    nonce: BigNumber;
    initCode: string;
    callData: string;
    callGasLimit: BigNumber;
    verificationGasLimit: BigNumber;
    preVerificationGas: BigNumber;
    maxFeePerGas: BigNumber;
    maxPriorityFeePerGas: BigNumber;
    paymasterAndData: string;
    signature: string;
};
export declare namespace IStakeManager {
    type DepositInfoStruct = {
        deposit: PromiseOrValue<BigNumberish>;
        staked: PromiseOrValue<boolean>;
        stake: PromiseOrValue<BigNumberish>;
        unstakeDelaySec: PromiseOrValue<BigNumberish>;
        withdrawTime: PromiseOrValue<BigNumberish>;
    };
    type DepositInfoStructOutput = [
        BigNumber,
        boolean,
        BigNumber,
        number,
        BigNumber
    ] & {
        deposit: BigNumber;
        staked: boolean;
        stake: BigNumber;
        unstakeDelaySec: number;
        withdrawTime: BigNumber;
    };
}
export declare namespace IEntryPoint {
    type UserOpsPerAggregatorStruct = {
        userOps: UserOperationStruct[];
        aggregator: PromiseOrValue<string>;
        signature: PromiseOrValue<BytesLike>;
    };
    type UserOpsPerAggregatorStructOutput = [
        UserOperationStructOutput[],
        string,
        string
    ] & {
        userOps: UserOperationStructOutput[];
        aggregator: string;
        signature: string;
    };
}
export interface IEntryPointInterface extends utils.Interface {
    functions: {
        "addStake(uint32)": FunctionFragment;
        "balanceOf(address)": FunctionFragment;
        "depositTo(address)": FunctionFragment;
        "getDepositInfo(address)": FunctionFragment;
        "getSenderAddress(bytes)": FunctionFragment;
        "getUserOpHash((address,uint256,bytes,bytes,uint256,uint256,uint256,uint256,uint256,bytes,bytes))": FunctionFragment;
        "handleAggregatedOps(((address,uint256,bytes,bytes,uint256,uint256,uint256,uint256,uint256,bytes,bytes)[],address,bytes)[],address)": FunctionFragment;
        "handleOps((address,uint256,bytes,bytes,uint256,uint256,uint256,uint256,uint256,bytes,bytes)[],address)": FunctionFragment;
        "simulateValidation((address,uint256,bytes,bytes,uint256,uint256,uint256,uint256,uint256,bytes,bytes))": FunctionFragment;
        "unlockStake()": FunctionFragment;
        "withdrawStake(address)": FunctionFragment;
        "withdrawTo(address,uint256)": FunctionFragment;
    };
    getFunction(nameOrSignatureOrTopic: "addStake" | "balanceOf" | "depositTo" | "getDepositInfo" | "getSenderAddress" | "getUserOpHash" | "handleAggregatedOps" | "handleOps" | "simulateValidation" | "unlockStake" | "withdrawStake" | "withdrawTo"): FunctionFragment;
    encodeFunctionData(functionFragment: "addStake", values: [PromiseOrValue<BigNumberish>]): string;
    encodeFunctionData(functionFragment: "balanceOf", values: [PromiseOrValue<string>]): string;
    encodeFunctionData(functionFragment: "depositTo", values: [PromiseOrValue<string>]): string;
    encodeFunctionData(functionFragment: "getDepositInfo", values: [PromiseOrValue<string>]): string;
    encodeFunctionData(functionFragment: "getSenderAddress", values: [PromiseOrValue<BytesLike>]): string;
    encodeFunctionData(functionFragment: "getUserOpHash", values: [UserOperationStruct]): string;
    encodeFunctionData(functionFragment: "handleAggregatedOps", values: [IEntryPoint.UserOpsPerAggregatorStruct[], PromiseOrValue<string>]): string;
    encodeFunctionData(functionFragment: "handleOps", values: [UserOperationStruct[], PromiseOrValue<string>]): string;
    encodeFunctionData(functionFragment: "simulateValidation", values: [UserOperationStruct]): string;
    encodeFunctionData(functionFragment: "unlockStake", values?: undefined): string;
    encodeFunctionData(functionFragment: "withdrawStake", values: [PromiseOrValue<string>]): string;
    encodeFunctionData(functionFragment: "withdrawTo", values: [PromiseOrValue<string>, PromiseOrValue<BigNumberish>]): string;
    decodeFunctionResult(functionFragment: "addStake", data: BytesLike): Result;
    decodeFunctionResult(functionFragment: "balanceOf", data: BytesLike): Result;
    decodeFunctionResult(functionFragment: "depositTo", data: BytesLike): Result;
    decodeFunctionResult(functionFragment: "getDepositInfo", data: BytesLike): Result;
    decodeFunctionResult(functionFragment: "getSenderAddress", data: BytesLike): Result;
    decodeFunctionResult(functionFragment: "getUserOpHash", data: BytesLike): Result;
    decodeFunctionResult(functionFragment: "handleAggregatedOps", data: BytesLike): Result;
    decodeFunctionResult(functionFragment: "handleOps", data: BytesLike): Result;
    decodeFunctionResult(functionFragment: "simulateValidation", data: BytesLike): Result;
    decodeFunctionResult(functionFragment: "unlockStake", data: BytesLike): Result;
    decodeFunctionResult(functionFragment: "withdrawStake", data: BytesLike): Result;
    decodeFunctionResult(functionFragment: "withdrawTo", data: BytesLike): Result;
    events: {
        "Deposited(address,uint256)": EventFragment;
        "StakeLocked(address,uint256,uint256)": EventFragment;
        "StakeUnlocked(address,uint256)": EventFragment;
        "StakeWithdrawn(address,address,uint256)": EventFragment;
        "UserOperationEvent(bytes32,address,address,uint256,uint256,uint256,bool)": EventFragment;
        "UserOperationRevertReason(bytes32,address,uint256,bytes)": EventFragment;
        "Withdrawn(address,address,uint256)": EventFragment;
    };
    getEvent(nameOrSignatureOrTopic: "Deposited"): EventFragment;
    getEvent(nameOrSignatureOrTopic: "StakeLocked"): EventFragment;
    getEvent(nameOrSignatureOrTopic: "StakeUnlocked"): EventFragment;
    getEvent(nameOrSignatureOrTopic: "StakeWithdrawn"): EventFragment;
    getEvent(nameOrSignatureOrTopic: "UserOperationEvent"): EventFragment;
    getEvent(nameOrSignatureOrTopic: "UserOperationRevertReason"): EventFragment;
    getEvent(nameOrSignatureOrTopic: "Withdrawn"): EventFragment;
}
export interface DepositedEventObject {
    account: string;
    totalDeposit: BigNumber;
}
export declare type DepositedEvent = TypedEvent<[
    string,
    BigNumber
], DepositedEventObject>;
export declare type DepositedEventFilter = TypedEventFilter<DepositedEvent>;
export interface StakeLockedEventObject {
    account: string;
    totalStaked: BigNumber;
    withdrawTime: BigNumber;
}
export declare type StakeLockedEvent = TypedEvent<[
    string,
    BigNumber,
    BigNumber
], StakeLockedEventObject>;
export declare type StakeLockedEventFilter = TypedEventFilter<StakeLockedEvent>;
export interface StakeUnlockedEventObject {
    account: string;
    withdrawTime: BigNumber;
}
export declare type StakeUnlockedEvent = TypedEvent<[
    string,
    BigNumber
], StakeUnlockedEventObject>;
export declare type StakeUnlockedEventFilter = TypedEventFilter<StakeUnlockedEvent>;
export interface StakeWithdrawnEventObject {
    account: string;
    withdrawAddress: string;
    amount: BigNumber;
}
export declare type StakeWithdrawnEvent = TypedEvent<[
    string,
    string,
    BigNumber
], StakeWithdrawnEventObject>;
export declare type StakeWithdrawnEventFilter = TypedEventFilter<StakeWithdrawnEvent>;
export interface UserOperationEventEventObject {
    userOpHash: string;
    sender: string;
    paymaster: string;
    nonce: BigNumber;
    actualGasCost: BigNumber;
    actualGasPrice: BigNumber;
    success: boolean;
}
export declare type UserOperationEventEvent = TypedEvent<[
    string,
    string,
    string,
    BigNumber,
    BigNumber,
    BigNumber,
    boolean
], UserOperationEventEventObject>;
export declare type UserOperationEventEventFilter = TypedEventFilter<UserOperationEventEvent>;
export interface UserOperationRevertReasonEventObject {
    userOpHash: string;
    sender: string;
    nonce: BigNumber;
    revertReason: string;
}
export declare type UserOperationRevertReasonEvent = TypedEvent<[
    string,
    string,
    BigNumber,
    string
], UserOperationRevertReasonEventObject>;
export declare type UserOperationRevertReasonEventFilter = TypedEventFilter<UserOperationRevertReasonEvent>;
export interface WithdrawnEventObject {
    account: string;
    withdrawAddress: string;
    amount: BigNumber;
}
export declare type WithdrawnEvent = TypedEvent<[
    string,
    string,
    BigNumber
], WithdrawnEventObject>;
export declare type WithdrawnEventFilter = TypedEventFilter<WithdrawnEvent>;
export interface IEntryPoint extends BaseContract {
    connect(signerOrProvider: Signer | Provider | string): this;
    attach(addressOrName: string): this;
    deployed(): Promise<this>;
    interface: IEntryPointInterface;
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
        addStake(_unstakeDelaySec: PromiseOrValue<BigNumberish>, overrides?: PayableOverrides & {
            from?: PromiseOrValue<string>;
        }): Promise<ContractTransaction>;
        balanceOf(account: PromiseOrValue<string>, overrides?: CallOverrides): Promise<[BigNumber]>;
        depositTo(account: PromiseOrValue<string>, overrides?: PayableOverrides & {
            from?: PromiseOrValue<string>;
        }): Promise<ContractTransaction>;
        getDepositInfo(account: PromiseOrValue<string>, overrides?: CallOverrides): Promise<[
            IStakeManager.DepositInfoStructOutput
        ] & {
            info: IStakeManager.DepositInfoStructOutput;
        }>;
        getSenderAddress(initCode: PromiseOrValue<BytesLike>, overrides?: Overrides & {
            from?: PromiseOrValue<string>;
        }): Promise<ContractTransaction>;
        getUserOpHash(userOp: UserOperationStruct, overrides?: CallOverrides): Promise<[string]>;
        handleAggregatedOps(opsPerAggregator: IEntryPoint.UserOpsPerAggregatorStruct[], beneficiary: PromiseOrValue<string>, overrides?: Overrides & {
            from?: PromiseOrValue<string>;
        }): Promise<ContractTransaction>;
        handleOps(ops: UserOperationStruct[], beneficiary: PromiseOrValue<string>, overrides?: Overrides & {
            from?: PromiseOrValue<string>;
        }): Promise<ContractTransaction>;
        simulateValidation(userOp: UserOperationStruct, overrides?: Overrides & {
            from?: PromiseOrValue<string>;
        }): Promise<ContractTransaction>;
        unlockStake(overrides?: Overrides & {
            from?: PromiseOrValue<string>;
        }): Promise<ContractTransaction>;
        withdrawStake(withdrawAddress: PromiseOrValue<string>, overrides?: Overrides & {
            from?: PromiseOrValue<string>;
        }): Promise<ContractTransaction>;
        withdrawTo(withdrawAddress: PromiseOrValue<string>, withdrawAmount: PromiseOrValue<BigNumberish>, overrides?: Overrides & {
            from?: PromiseOrValue<string>;
        }): Promise<ContractTransaction>;
    };
    addStake(_unstakeDelaySec: PromiseOrValue<BigNumberish>, overrides?: PayableOverrides & {
        from?: PromiseOrValue<string>;
    }): Promise<ContractTransaction>;
    balanceOf(account: PromiseOrValue<string>, overrides?: CallOverrides): Promise<BigNumber>;
    depositTo(account: PromiseOrValue<string>, overrides?: PayableOverrides & {
        from?: PromiseOrValue<string>;
    }): Promise<ContractTransaction>;
    getDepositInfo(account: PromiseOrValue<string>, overrides?: CallOverrides): Promise<IStakeManager.DepositInfoStructOutput>;
    getSenderAddress(initCode: PromiseOrValue<BytesLike>, overrides?: Overrides & {
        from?: PromiseOrValue<string>;
    }): Promise<ContractTransaction>;
    getUserOpHash(userOp: UserOperationStruct, overrides?: CallOverrides): Promise<string>;
    handleAggregatedOps(opsPerAggregator: IEntryPoint.UserOpsPerAggregatorStruct[], beneficiary: PromiseOrValue<string>, overrides?: Overrides & {
        from?: PromiseOrValue<string>;
    }): Promise<ContractTransaction>;
    handleOps(ops: UserOperationStruct[], beneficiary: PromiseOrValue<string>, overrides?: Overrides & {
        from?: PromiseOrValue<string>;
    }): Promise<ContractTransaction>;
    simulateValidation(userOp: UserOperationStruct, overrides?: Overrides & {
        from?: PromiseOrValue<string>;
    }): Promise<ContractTransaction>;
    unlockStake(overrides?: Overrides & {
        from?: PromiseOrValue<string>;
    }): Promise<ContractTransaction>;
    withdrawStake(withdrawAddress: PromiseOrValue<string>, overrides?: Overrides & {
        from?: PromiseOrValue<string>;
    }): Promise<ContractTransaction>;
    withdrawTo(withdrawAddress: PromiseOrValue<string>, withdrawAmount: PromiseOrValue<BigNumberish>, overrides?: Overrides & {
        from?: PromiseOrValue<string>;
    }): Promise<ContractTransaction>;
    callStatic: {
        addStake(_unstakeDelaySec: PromiseOrValue<BigNumberish>, overrides?: CallOverrides): Promise<void>;
        balanceOf(account: PromiseOrValue<string>, overrides?: CallOverrides): Promise<BigNumber>;
        depositTo(account: PromiseOrValue<string>, overrides?: CallOverrides): Promise<void>;
        getDepositInfo(account: PromiseOrValue<string>, overrides?: CallOverrides): Promise<IStakeManager.DepositInfoStructOutput>;
        getSenderAddress(initCode: PromiseOrValue<BytesLike>, overrides?: CallOverrides): Promise<void>;
        getUserOpHash(userOp: UserOperationStruct, overrides?: CallOverrides): Promise<string>;
        handleAggregatedOps(opsPerAggregator: IEntryPoint.UserOpsPerAggregatorStruct[], beneficiary: PromiseOrValue<string>, overrides?: CallOverrides): Promise<void>;
        handleOps(ops: UserOperationStruct[], beneficiary: PromiseOrValue<string>, overrides?: CallOverrides): Promise<void>;
        simulateValidation(userOp: UserOperationStruct, overrides?: CallOverrides): Promise<void>;
        unlockStake(overrides?: CallOverrides): Promise<void>;
        withdrawStake(withdrawAddress: PromiseOrValue<string>, overrides?: CallOverrides): Promise<void>;
        withdrawTo(withdrawAddress: PromiseOrValue<string>, withdrawAmount: PromiseOrValue<BigNumberish>, overrides?: CallOverrides): Promise<void>;
    };
    filters: {
        "Deposited(address,uint256)"(account?: PromiseOrValue<string> | null, totalDeposit?: null): DepositedEventFilter;
        Deposited(account?: PromiseOrValue<string> | null, totalDeposit?: null): DepositedEventFilter;
        "StakeLocked(address,uint256,uint256)"(account?: PromiseOrValue<string> | null, totalStaked?: null, withdrawTime?: null): StakeLockedEventFilter;
        StakeLocked(account?: PromiseOrValue<string> | null, totalStaked?: null, withdrawTime?: null): StakeLockedEventFilter;
        "StakeUnlocked(address,uint256)"(account?: PromiseOrValue<string> | null, withdrawTime?: null): StakeUnlockedEventFilter;
        StakeUnlocked(account?: PromiseOrValue<string> | null, withdrawTime?: null): StakeUnlockedEventFilter;
        "StakeWithdrawn(address,address,uint256)"(account?: PromiseOrValue<string> | null, withdrawAddress?: null, amount?: null): StakeWithdrawnEventFilter;
        StakeWithdrawn(account?: PromiseOrValue<string> | null, withdrawAddress?: null, amount?: null): StakeWithdrawnEventFilter;
        "UserOperationEvent(bytes32,address,address,uint256,uint256,uint256,bool)"(userOpHash?: PromiseOrValue<BytesLike> | null, sender?: PromiseOrValue<string> | null, paymaster?: PromiseOrValue<string> | null, nonce?: null, actualGasCost?: null, actualGasPrice?: null, success?: null): UserOperationEventEventFilter;
        UserOperationEvent(userOpHash?: PromiseOrValue<BytesLike> | null, sender?: PromiseOrValue<string> | null, paymaster?: PromiseOrValue<string> | null, nonce?: null, actualGasCost?: null, actualGasPrice?: null, success?: null): UserOperationEventEventFilter;
        "UserOperationRevertReason(bytes32,address,uint256,bytes)"(userOpHash?: PromiseOrValue<BytesLike> | null, sender?: PromiseOrValue<string> | null, nonce?: null, revertReason?: null): UserOperationRevertReasonEventFilter;
        UserOperationRevertReason(userOpHash?: PromiseOrValue<BytesLike> | null, sender?: PromiseOrValue<string> | null, nonce?: null, revertReason?: null): UserOperationRevertReasonEventFilter;
        "Withdrawn(address,address,uint256)"(account?: PromiseOrValue<string> | null, withdrawAddress?: null, amount?: null): WithdrawnEventFilter;
        Withdrawn(account?: PromiseOrValue<string> | null, withdrawAddress?: null, amount?: null): WithdrawnEventFilter;
    };
    estimateGas: {
        addStake(_unstakeDelaySec: PromiseOrValue<BigNumberish>, overrides?: PayableOverrides & {
            from?: PromiseOrValue<string>;
        }): Promise<BigNumber>;
        balanceOf(account: PromiseOrValue<string>, overrides?: CallOverrides): Promise<BigNumber>;
        depositTo(account: PromiseOrValue<string>, overrides?: PayableOverrides & {
            from?: PromiseOrValue<string>;
        }): Promise<BigNumber>;
        getDepositInfo(account: PromiseOrValue<string>, overrides?: CallOverrides): Promise<BigNumber>;
        getSenderAddress(initCode: PromiseOrValue<BytesLike>, overrides?: Overrides & {
            from?: PromiseOrValue<string>;
        }): Promise<BigNumber>;
        getUserOpHash(userOp: UserOperationStruct, overrides?: CallOverrides): Promise<BigNumber>;
        handleAggregatedOps(opsPerAggregator: IEntryPoint.UserOpsPerAggregatorStruct[], beneficiary: PromiseOrValue<string>, overrides?: Overrides & {
            from?: PromiseOrValue<string>;
        }): Promise<BigNumber>;
        handleOps(ops: UserOperationStruct[], beneficiary: PromiseOrValue<string>, overrides?: Overrides & {
            from?: PromiseOrValue<string>;
        }): Promise<BigNumber>;
        simulateValidation(userOp: UserOperationStruct, overrides?: Overrides & {
            from?: PromiseOrValue<string>;
        }): Promise<BigNumber>;
        unlockStake(overrides?: Overrides & {
            from?: PromiseOrValue<string>;
        }): Promise<BigNumber>;
        withdrawStake(withdrawAddress: PromiseOrValue<string>, overrides?: Overrides & {
            from?: PromiseOrValue<string>;
        }): Promise<BigNumber>;
        withdrawTo(withdrawAddress: PromiseOrValue<string>, withdrawAmount: PromiseOrValue<BigNumberish>, overrides?: Overrides & {
            from?: PromiseOrValue<string>;
        }): Promise<BigNumber>;
    };
    populateTransaction: {
        addStake(_unstakeDelaySec: PromiseOrValue<BigNumberish>, overrides?: PayableOverrides & {
            from?: PromiseOrValue<string>;
        }): Promise<PopulatedTransaction>;
        balanceOf(account: PromiseOrValue<string>, overrides?: CallOverrides): Promise<PopulatedTransaction>;
        depositTo(account: PromiseOrValue<string>, overrides?: PayableOverrides & {
            from?: PromiseOrValue<string>;
        }): Promise<PopulatedTransaction>;
        getDepositInfo(account: PromiseOrValue<string>, overrides?: CallOverrides): Promise<PopulatedTransaction>;
        getSenderAddress(initCode: PromiseOrValue<BytesLike>, overrides?: Overrides & {
            from?: PromiseOrValue<string>;
        }): Promise<PopulatedTransaction>;
        getUserOpHash(userOp: UserOperationStruct, overrides?: CallOverrides): Promise<PopulatedTransaction>;
        handleAggregatedOps(opsPerAggregator: IEntryPoint.UserOpsPerAggregatorStruct[], beneficiary: PromiseOrValue<string>, overrides?: Overrides & {
            from?: PromiseOrValue<string>;
        }): Promise<PopulatedTransaction>;
        handleOps(ops: UserOperationStruct[], beneficiary: PromiseOrValue<string>, overrides?: Overrides & {
            from?: PromiseOrValue<string>;
        }): Promise<PopulatedTransaction>;
        simulateValidation(userOp: UserOperationStruct, overrides?: Overrides & {
            from?: PromiseOrValue<string>;
        }): Promise<PopulatedTransaction>;
        unlockStake(overrides?: Overrides & {
            from?: PromiseOrValue<string>;
        }): Promise<PopulatedTransaction>;
        withdrawStake(withdrawAddress: PromiseOrValue<string>, overrides?: Overrides & {
            from?: PromiseOrValue<string>;
        }): Promise<PopulatedTransaction>;
        withdrawTo(withdrawAddress: PromiseOrValue<string>, withdrawAmount: PromiseOrValue<BigNumberish>, overrides?: Overrides & {
            from?: PromiseOrValue<string>;
        }): Promise<PopulatedTransaction>;
    };
}
