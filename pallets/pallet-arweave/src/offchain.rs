use crate::*;

use arweave_rust::*;
use polkadot_sdk::sp_runtime::offchain::http;

impl<T: Config<I>, I: 'static> Pallet<T, I> {
    pub fn sign_tasks(
    ) -> OffchainWorkerResult<Vec<(TaskFor<T, I>, Option<ar_substrate::Transaction>)>> {
        let mut commit_tasks = vec![];

        let tasks_to_sign = Self::get_tasks_by_state(TaskState::Sign);
        for mut task_to_sing in tasks_to_sign {
            log!(info, "sign: task_id={:?}", task_to_sing.task_id);

            let fee = Self::arweave_fee(task_to_sing.data.len())
                .map_err(OffchainWorkerError::HttpRequestError)?;
            let last_tx = Self::arweave_last_tx().map_err(OffchainWorkerError::HttpRequestError)?;

            let tx = ar_runtime_interface::arweave_extension::signed_transaction(
                task_to_sing.data.to_vec(),
                fee*2,
                last_tx.as_str(),
                vec![(
                    "Content-Type".as_bytes().to_vec(),
                    "application/json".as_bytes().to_vec(),
                )],
            )
            .map_err(OffchainWorkerError::ArweaveRustError)?;

            let bytes = tx.id.as_bytes().to_vec();
            let tx_hash = BoundedVec::<u8, T::MaxTxHashLength>::try_from(bytes)
                .map_err(OffchainWorkerError::BoundedVecError)?;

            task_to_sing.state = task_to_sing.state.next_state();
            task_to_sing.tx_hash = Some(tx_hash);

            commit_tasks.push((task_to_sing, Some(tx)))
        }

        Ok(commit_tasks)
    }

    pub fn upload_tasks(
    ) -> OffchainWorkerResult<Vec<(TaskFor<T, I>, Option<ar_substrate::Transaction>)>> {
        let mut commit_tasks = vec![];

        let task_to_upload = Self::get_tasks_by_state(TaskState::Upload).pop();
        if let Some(mut task_to_upload) = task_to_upload {
            log!(info, "upload: task_id={:?}", task_to_upload.task_id);

            if let Some(data) = TasksSignedData::<T, I>::get(task_to_upload.task_id) {
                let task_key = format!("lock::task::{:?}", task_to_upload.task_id);

                // check a task already uploaded
                if sp_io::offchain::local_storage_get(
                    sp_core::offchain::StorageKind::PERSISTENT,
                    task_key.as_bytes(),
                )
                .is_none()
                // upload task if its not in node store
                {
                    // set task id in node storage
                    sp_io::offchain::local_storage_set(
                        sp_core::offchain::StorageKind::PERSISTENT,
                        task_key.as_bytes(),
                        "".as_bytes(),
                    );

                    log!(
                        info,
                        "arweave_post_transaction: task_id={:?}",
                        task_to_upload.task_id
                    );

                    match Self::arweave_post_transaction(vec![&data.to_vec()]) {
                        Ok(_) => {
                            task_to_upload.state = task_to_upload.state.next_state();
                            commit_tasks.push((task_to_upload, None));
                        }
                        Err(e) => {
                            // clear task id from storage on error
                            sp_io::offchain::local_storage_clear(
                                sp_core::offchain::StorageKind::PERSISTENT,
                                task_key.as_bytes(),
                            );

                            log!(error, "arweave_post_transaction: {:?}", e);

                            // move task state to sign stage
                            // to prevent arweave errors
                            task_to_upload.state = task_to_upload.state.prev_state();
                            commit_tasks.push((task_to_upload, None));
                        }
                    };
                } else {
                    task_to_upload.state = TaskState::Validate;
                    commit_tasks.push((task_to_upload, None));
                }
            }
        }

        Ok(commit_tasks)
    }

    pub fn validate_tasks(
    ) -> OffchainWorkerResult<Vec<(TaskFor<T, I>, Option<ar_substrate::Transaction>)>> {
        let mut commit_tasks = vec![];

        let tasks_to_validate = Self::get_tasks_by_state(TaskState::Validate);
        for mut task_to_validate in tasks_to_validate {
            log!(info, "validate: task_id={:?}", task_to_validate.task_id);

            if let Some(tx_hash) = &task_to_validate.tx_hash {
                let status_code = Self::arweave_get_transaction(tx_hash.to_vec())?;

                match status_code {
                    200 => {
                        task_to_validate.state = task_to_validate.state.next_state();
                        commit_tasks.push((task_to_validate, None));
                    }
                    202 => {
                        return Err(OffchainWorkerError::ArweaveRustTransactionPending);
                    }
                    404 => {
                        log!(warn, "Received 404 status_code from arweave, resubmit to sign: {}", status_code);
                        task_to_validate.state = TaskState::Sign;
                        commit_tasks.push((task_to_validate, None));
                    }
                    _ => {
                        log!(warn, "Unexpected status code: {}", status_code);
                        return Err(OffchainWorkerError::HttpRequestError(http::Error::Unknown));
                    }
                };
            };
        }

        Ok(commit_tasks)
    }
}
