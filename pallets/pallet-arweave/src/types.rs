use super::*;
use polkadot_sdk::*;

use scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

use sp_debug_derive::RuntimeDebug;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

pub type TaskFor<T, I = ()> = Task<
    <T as Config<I>>::TaskId,
    <T as frame_system::Config>::AccountId,
    <T as Config<I>>::MaxDataLength,
    <T as Config<I>>::MaxTxHashLength,
>;

pub type TaskResultFor<T, I = ()> = TaskResult<
    <T as Config<I>>::TaskId,
    <T as frame_system::Config>::AccountId,
    <T as Config<I>>::MaxTxHashLength,
>;

pub type WorkerInfoFor<T> = WorkerInfo<<T as frame_system::Config>::AccountId>;

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(DataLimit, TxHashLimit))]
pub struct Task<TaskID, WorkerAddress, DataLimit: Get<u32>, TxHashLimit: Get<u32>> {
    // unique task id
    pub task_id: TaskID,

    // address of worker for whom the task is intended
    pub worker_address: WorkerAddress,

    // some data to be stored in arweave
    pub data: BoundedVec<u8, DataLimit>,
    // task current state
    pub state: TaskState,

    // arweave tx hash
    pub tx_hash: Option<BoundedVec<u8, TxHashLimit>>,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum TaskState {
    Sign,
    Upload,
    Validate,
    Clear,
}

impl TaskState {
    pub fn next_state(&self) -> TaskState {
        match self {
            TaskState::Sign => TaskState::Upload,
            TaskState::Upload => TaskState::Validate,
            TaskState::Validate => TaskState::Clear,
            TaskState::Clear => TaskState::Clear,
        }
    }

    pub fn prev_state(&self) -> TaskState {
        match self {
            TaskState::Sign => TaskState::Sign,
            TaskState::Upload => TaskState::Sign,
            TaskState::Validate => TaskState::Upload,
            TaskState::Clear => TaskState::Validate,
        }
    }
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(TxHashLimit))]
pub struct TaskResult<TaskID, WorkerAddress, TxHashLimit: Get<u32>> {
    pub task_id: TaskID,
    pub worker_address: WorkerAddress,
    pub tx_hash: BoundedVec<u8, TxHashLimit>,
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct WorkerInfo<WorkerAddress> {
    pub worker_address: WorkerAddress,
}
