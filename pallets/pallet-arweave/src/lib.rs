#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]
extern crate alloc;
extern crate core;

use alloc::format;
use alloc::vec;
use alloc::vec::Vec;

use polkadot_sdk::polkadot_sdk_frame as frame;
use polkadot_sdk::{sp_core, sp_io};

use frame::prelude::*;
use frame_system::{
    offchain::{AppCrypto, CreateSignedTransaction, SendSignedTransaction, Signer},
    pallet_prelude::BlockNumberFor,
};

use polkadot_sdk::frame_support::traits::Incrementable;

pub use errors::*;
pub use pallet::*;
pub use types::*;

pub mod arweave;
pub mod crypto;
pub mod weights;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
mod errors;
#[cfg(test)]
mod mock;
mod offchain;
mod tasks;
#[cfg(test)]
mod tests;
mod types;

pub const LOG_TARGET: &str = "runtime::pallet-arweave";

// syntactic sugar for logging.
#[macro_export]
macro_rules! log {
	($level:tt, $patter:expr $(, $values:expr)* $(,)?) => {
		log::$level!(
			target: crate::LOG_TARGET,
			concat!("[{}] 📦  ", $patter), crate::LOG_TARGET $(, $values)*
		)
	};
}

#[frame::pallet(dev_mode)]
pub mod pallet {
    use super::*;

    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T, I = ()>(PhantomData<(T, I)>);

    #[pallet::config]
    pub trait Config<I: 'static = ()>:
        CreateSignedTransaction<Call<Self, I>> + polkadot_sdk::frame_system::Config
    {
        type AuthorityId: AppCrypto<Self::Public, Self::Signature>;

        type TaskId: Member
            + Parameter
            + MaxEncodedLen
            + Copy
            + Incrementable
            + CheckedAdd
            + CheckedSub
            + PartialOrd;

        #[pallet::constant]
        type MaxDataLength: Get<u32>;

        #[pallet::constant]
        type MaxTxHashLength: Get<u32>;

        #[pallet::constant]
        type MaxSignedDataLength: Get<u32>;

        type RuntimeEvent: From<Event<Self, I>>
            + IsType<<Self as polkadot_sdk::frame_system::Config>::RuntimeEvent>;

        type WeightInfo: crate::weights::WeightInfo;
    }

    #[pallet::storage]
    pub type NextTaskId<T: Config<I>, I: 'static = ()> = StorageValue<_, T::TaskId, OptionQuery>;

    #[pallet::storage]
    pub(super) type Tasks<T: Config<I>, I: 'static = ()> =
        StorageMap<_, Blake2_128Concat, T::TaskId, TaskFor<T, I>>;

    #[pallet::storage]
    pub(super) type TasksResults<T: Config<I>, I: 'static = ()> =
        StorageMap<_, Blake2_128Concat, T::TaskId, TaskResultFor<T, I>>;

    #[pallet::storage]
    pub(super) type TasksSignedData<T: Config<I>, I: 'static = ()> =
        StorageMap<_, Blake2_128Concat, T::TaskId, BoundedVec<u8, T::MaxSignedDataLength>>;

    #[pallet::storage]
    pub(super) type Workers<T: Config<I>, I: 'static = ()> =
        StorageMap<_, Blake2_128Concat, T::AccountId, WorkerInfoFor<T>>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config<I>, I: 'static = ()> {
        TaskAdded { task_id: T::TaskId },
        TaskChanged { task_id: T::TaskId },
        TaskCleared { task_id: T::TaskId },
    }

    #[pallet::error]
    pub enum Error<T, I = ()> {
        TaskIdIncrementFailed,
        TaskAlreadyExists,
        TaskNotExist,
        TaskMustBeInClearState,
        TaskHasNoResult,
        TaskDataInvalidJson,
    }

    #[pallet::hooks]
    impl<T: Config<I>, I: 'static> Hooks<BlockNumberFor<T>> for Pallet<T, I> {
        fn offchain_worker(block_number: BlockNumberFor<T>) {
            let n: U256 = block_number.into();

            if n % 2 != U256::from(0) {
                return;
            }

            if !arweave_rust::ar_runtime_interface::arweave_extension::enabled() {
                return;
            }

            log!(debug, "offchain_worker job");

            let mut commit_tasks = vec![];

            match Self::sign_tasks() {
                Ok(r) => commit_tasks.extend(r),
                Err(e) => log!(error, "sign_tasks: {:?}", e),
            };

            match Self::upload_tasks() {
                Ok(r) => commit_tasks.extend(r),
                Err(e) => log!(error, "upload_tasks: {:?}", e),
            };

            match Self::validate_tasks() {
                Ok(r) => commit_tasks.extend(r),
                Err(e) => log!(error, "validate_tasks: {:?}", e),
            };

            let signer = Signer::<T, T::AuthorityId>::all_accounts();
            if !signer.can_sign() {
                log!(
                    error,
                    "No local accounts available. Consider adding one via `author_insertKey` RPC."
                );
                return;
            }

            for (commit_task, data) in commit_tasks {
                let results = if let Some(data) = data {
                    log!(debug, "commit task with data");

                    let bytes = data.tx.as_bytes().to_vec();
                    match BoundedVec::<u8, T::MaxSignedDataLength>::try_from(bytes) {
                        Ok(v) => signer.send_signed_transaction(|_account| Call::sign_task_data {
                            task_id: commit_task.task_id,
                            signed_data: v.clone(),
                            tx_hash: commit_task.tx_hash.clone(),
                        }),
                        Err(e) => {
                            log!(error,
                                "can not convert data to bounded vector: data len={} max={} task_id={:?}",
                                e.len(),
                                T::MaxSignedDataLength::get(),
                                commit_task.task_id
                            );
                            continue;
                        }
                    }
                } else {
                    log!(debug, "commit task without data");

                    signer.send_signed_transaction(|_account| Call::update_task {
                        task_id: commit_task.task_id,
                        state: commit_task.state.clone(),
                    })
                };

                for (acc, res) in &results {
                    match res {
                        Ok(()) => log!(debug, "[{:?}] Submitted", acc.id),
                        Err(e) => {
                            log!(
                                error,
                                "[{:?}] Failed to submit transaction: {:?}",
                                acc.id,
                                e
                            )
                        }
                    }
                }
            }

            let tasks_to_clear = Self::get_tasks_by_state(TaskState::Clear);
            for task_to_clear in tasks_to_clear {
                log!(
                    info,
                    "commit task to clear: task_id={:?}",
                    task_to_clear.task_id
                );

                let results = signer.send_signed_transaction(|_account| Call::clear_task {
                    task_id: task_to_clear.task_id,
                });

                for (acc, res) in &results {
                    match res {
                        Ok(()) => log!(debug, "[{:?}] Submitted", acc.id),
                        Err(e) => {
                            log!(
                                error,
                                "[{:?}] Failed to submit transaction: {:?}",
                                acc.id,
                                e
                            )
                        }
                    }
                }
            }
        }
    }

    #[pallet::call]
    impl<T: Config<I>, I: 'static> Pallet<T, I> {
        #[pallet::call_index(0)]
        #[pallet::weight(10)]
        pub fn create_task(
            origin: OriginFor<T>,
            worker_address: T::AccountId,
            data: BoundedVec<u8, T::MaxDataLength>,
        ) -> DispatchResult {
            let origin = ensure_signed(origin)?;
            Self::add_new_task(origin, worker_address, data)?;
            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(10)]
        pub fn update_task(
            origin: OriginFor<T>,
            task_id: T::TaskId,
            state: TaskState,
        ) -> DispatchResult {
            let origin = ensure_signed(origin)?;
            Self::change_state_task(origin, task_id, state)?;
            Ok(())
        }

        #[pallet::call_index(2)]
        #[pallet::weight(10)]
        pub fn sign_task_data(
            origin: OriginFor<T>,
            task_id: T::TaskId,
            signed_data: BoundedVec<u8, T::MaxSignedDataLength>,
            tx_hash: Option<BoundedVec<u8, T::MaxTxHashLength>>,
        ) -> DispatchResult {
            let origin = ensure_signed(origin)?;
            ensure!(tx_hash.is_some(), Error::<T, I>::TaskHasNoResult);
            Self::sign_task_with_data(origin, task_id, signed_data, tx_hash.unwrap())?;
            Ok(())
        }

        #[pallet::call_index(3)]
        #[pallet::weight(10)]
        pub fn clear_task(origin: OriginFor<T>, task_id: T::TaskId) -> DispatchResult {
            let origin = ensure_signed(origin)?;
            Self::move_task_to_result(origin, task_id)?;
            Ok(())
        }
    }
}
