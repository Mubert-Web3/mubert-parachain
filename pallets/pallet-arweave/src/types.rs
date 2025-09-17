use super::*;
use polkadot_sdk::*;

use polkadot_sdk::frame_support::traits::Currency;
use scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

use sp_debug_derive::RuntimeDebug;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

/// Type alias for the balance type used in the pallet, derived from the `Currency` trait.
pub type BalanceOf<T, I = ()> =
    <<T as Config<I>>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

/// Type alias for a `Task` structure specific to the pallet configuration.
pub type TaskFor<T, I = ()> = Task<
    <T as Config<I>>::TaskId,
    <T as frame_system::Config>::AccountId,
    <T as Config<I>>::MaxDataLength,
    <T as Config<I>>::MaxTxHashLength,
    BalanceOf<T, I>,
>;

/// Type alias for a `TaskResult` structure specific to the pallet configuration.
pub type TaskResultFor<T, I = ()> = TaskResult<
    <T as Config<I>>::TaskId,
    <T as frame_system::Config>::AccountId,
    <T as Config<I>>::MaxTxHashLength,
>;

/// Type alias for `WorkerInfo` structure specific to the pallet configuration.
pub type WorkerInfoFor<T> = WorkerInfo<<T as frame_system::Config>::AccountId>;

/// Represents a task to be processed and stored in Arweave.
///
/// # Parameters
/// - `TaskID`: The unique identifier for the task.
/// - `WorkerAddress`: The address of the worker assigned to the task.
/// - `DataLimit`: The maximum size of the data to be stored.
/// - `TxHashLimit`: The maximum size of the transaction hash.
/// - `Currency`: The type representing the currency used for deposits and tips.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(DataLimit, TxHashLimit))]
pub struct Task<TaskID, WorkerAddress, DataLimit: Get<u32>, TxHashLimit: Get<u32>, Currency> {
    /// Unique task ID.
    pub task_id: TaskID,
    /// Address of the worker for whom the task is intended.
    pub worker_address: WorkerAddress,
    /// Data to be stored in Arweave.
    pub data: BoundedVec<u8, DataLimit>,
    /// Current state of the task.
    pub state: TaskState,
    /// Arweave transaction hash.
    pub tx_hash: Option<BoundedVec<u8, TxHashLimit>>,
    /// Deposit for the task, returned to the worker after task completion.
    pub amount: Currency,
    /// Tips for the transaction with signed data.
    pub tips: Currency,
}

/// Represents the state of a task in its lifecycle.
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum TaskState {
    /// Task is in the signing stage.
    Sign,
    /// Task is in the uploading stage.
    Upload,
    /// Task is in the validation stage.
    Validate,
    /// Task is in the clearing stage.
    Clear,
}

impl TaskState {
    /// Returns the next state in the task lifecycle.
    pub fn next_state(&self) -> TaskState {
        match self {
            TaskState::Sign => TaskState::Upload,
            TaskState::Upload => TaskState::Validate,
            TaskState::Validate => TaskState::Clear,
            TaskState::Clear => TaskState::Clear,
        }
    }

    /// Returns the previous state in the task lifecycle.
    pub fn prev_state(&self) -> TaskState {
        match self {
            TaskState::Sign => TaskState::Sign,
            TaskState::Upload => TaskState::Sign,
            TaskState::Validate => TaskState::Upload,
            TaskState::Clear => TaskState::Validate,
        }
    }
}

/// Represents the result of a completed task.
///
/// # Parameters
/// - `TaskID`: The unique identifier for the task.
/// - `WorkerAddress`: The address of the worker who completed the task.
/// - `TxHashLimit`: The maximum size of the transaction hash.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(TxHashLimit))]
pub struct TaskResult<TaskID, WorkerAddress, TxHashLimit: Get<u32>> {
    /// Unique task ID.
    pub task_id: TaskID,
    /// Address of the worker who completed the task.
    pub worker_address: WorkerAddress,
    /// Arweave transaction hash.
    pub tx_hash: BoundedVec<u8, TxHashLimit>,
}

/// Represents information about a worker.
///
/// # Parameters
/// - `WorkerAddress`: The address of the worker.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct WorkerInfo<WorkerAddress> {
    /// Address of the worker.
    pub worker_address: WorkerAddress,
}
