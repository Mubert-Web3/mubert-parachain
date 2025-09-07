use crate::*;

impl<T: Config<I>, I: 'static> Pallet<T, I> {
    pub(crate) fn add_new_task(
        origin: T::AccountId,
        worker_address: T::AccountId,
        data: BoundedVec<u8, T::MaxDataLength>,
        amount: BalanceOf<T, I>,
        tips: BalanceOf<T, I>,
    ) -> DispatchResult {
        let pallet_account = Self::account_id();

        let ed = T::Currency::minimum_balance();
        if T::Currency::free_balance(&pallet_account).is_zero() {
            // If pallet account doesn't exist yet, the very first inbound transfer
            // must be >= ED to create it via transfer.
            ensure!(amount >= ed, Error::<T, I>::DepositTooSmallForNewAccount);
        }

        let deposit = amount
            .checked_add(&tips)
            .ok_or(Error::<T, I>::DepositOverflow)?;

        T::Currency::transfer(
            &origin,
            &pallet_account,
            deposit,
            ExistenceRequirement::KeepAlive,
        )?;

        NextTaskId::<T, I>::try_mutate(|maybe_task_id| -> DispatchResult {
            let task_id = maybe_task_id
                .map_or(T::TaskId::initial_value(), Some)
                .ok_or(Error::<T, I>::TaskIdIncrementFailed)?;

            ensure!(
                !Tasks::<T, I>::contains_key(task_id),
                Error::<T, I>::TaskAlreadyExists
            );
            let r = serde_json::from_slice::<serde::de::IgnoredAny>(data.as_slice());
            ensure!(r.is_ok(), Error::<T, I>::TaskDataInvalidJson);

            Tasks::<T, I>::insert(
                task_id.clone(),
                TaskFor::<T, I> {
                    task_id,
                    worker_address,
                    data,
                    state: TaskState::Sign,
                    tx_hash: None,
                    amount,
                    tips,
                },
            );

            Self::deposit_event(Event::TaskAdded { task_id });

            let new_task_id = task_id
                .increment()
                .ok_or(Error::<T, I>::TaskIdIncrementFailed)?;

            *maybe_task_id = Some(new_task_id);

            Ok(())
        })
    }

    pub(crate) fn change_state_task(
        _origin: T::AccountId,
        task_id: T::TaskId,
        state: TaskState,
    ) -> DispatchResult {
        Tasks::<T, I>::try_mutate(task_id, |maybe_task| -> DispatchResult {
            let task = maybe_task.as_mut().ok_or(Error::<T, I>::TaskNotExist)?;
            task.state = state;

            Self::deposit_event(Event::TaskChanged { task_id });

            Ok(())
        })
    }

    pub(crate) fn sign_task_with_data(
        _origin: T::AccountId,
        task_id: T::TaskId,
        signed_data: BoundedVec<u8, T::MaxSignedDataLength>,
        tx_hash: BoundedVec<u8, T::MaxTxHashLength>,
    ) -> DispatchResult {
        Tasks::<T, I>::try_mutate(task_id, |maybe_task| -> DispatchResult {
            let task = maybe_task.as_mut().ok_or(Error::<T, I>::TaskNotExist)?;
            task.state = TaskState::Upload;
            task.tx_hash = Some(tx_hash);

            TasksSignedData::<T, I>::insert(task_id, signed_data);

            Self::deposit_event(Event::TaskChanged { task_id });

            Ok(())
        })
    }

    pub(crate) fn move_task_to_result(_origin: T::AccountId, task_id: T::TaskId) -> DispatchResult {
        let task = Tasks::<T, I>::take(task_id).ok_or(Error::<T, I>::TaskNotExist)?;

        ensure!(
            task.state == TaskState::Clear,
            Error::<T, I>::TaskMustBeInClearState
        );
        ensure!(task.tx_hash.is_some(), Error::<T, I>::TaskHasNoResult);

        let deposit = task
            .amount
            .checked_add(&task.tips)
            .ok_or(Error::<T, I>::DepositOverflow)?;

        let pallet_account = Self::account_id();

        if !deposit.is_zero() {
            T::Currency::transfer(
                &pallet_account,
                &task.worker_address,
                deposit,
                ExistenceRequirement::KeepAlive,
            )?;
        }

        TasksResults::<T, I>::insert(
            task_id,
            TaskResultFor::<T, I> {
                task_id,
                worker_address: task.worker_address,
                tx_hash: task.tx_hash.unwrap(),
            },
        );

        TasksSignedData::<T, I>::remove(task_id);

        Self::deposit_event(Event::TaskCleared { task_id });

        Ok(())
    }

    pub fn get_tasks_by_state(state: TaskState) -> Vec<TaskFor<T, I>> {
        Tasks::<T, I>::iter()
            .filter(|(_, v)| v.state == state)
            .map(|(_, v)| v)
            .collect::<Vec<TaskFor<T, I>>>()
    }
}
