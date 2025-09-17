use crate::*;

impl<T: Config<I>, I: 'static> Pallet<T, I> {
    /// Adds a new task to the pallet.
    ///
    /// # It ensures
    /// - Validates the task data and ensures the deposit is sufficient.
    /// - Inserts the task into storage and increments the task ID.
    ///
    /// # Parameters
    /// - `origin`: The account ID of the task creator.
    /// - `worker_address`: The account ID of the worker assigned to the task.
    /// - `data`: The task data as a bounded vector.
    /// - `amount`: The deposit amount for the task.
    /// - `tips`: The tips for the task.
    ///
    /// # Errors
    /// - `DepositTooSmallForNewAccount`: If the deposit is less than the minimum balance.
    /// - `DepositOverflow`: If the deposit calculation overflows.
    /// - `TaskAlreadyExists`: If a task with the same ID already exists.
    /// - `TaskDataInvalidJson`: If the task data is not valid JSON.
    ///
    /// # Events
    /// - Emits `TaskAdded` when a new task is successfully added.
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

    /// Changes the state of an existing task.
    ///
    /// # It ensures
    /// - Updates the state of the specified task in storage.
    ///
    /// # Parameters
    /// - `origin`: The account ID of the caller.
    /// - `task_id`: The ID of the task to update.
    /// - `state`: The new state to set for the task.
    ///
    /// # Errors
    /// - `TaskNotExist`: If the task does not exist in storage.
    ///
    /// # Events
    /// - Emits `TaskChanged` when the task state is successfully updated.
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

    /// Signs a task with the provided data and transaction hash.
    ///
    /// # It ensures
    /// - Updates the task state to `Upload` and stores the signed data and transaction hash.
    ///
    /// # Parameters
    /// - `origin`: The account ID of the caller.
    /// - `task_id`: The ID of the task to sign.
    /// - `signed_data`: The signed data as a bounded vector.
    /// - `tx_hash`: The transaction hash as a bounded vector.
    ///
    /// # Errors
    /// - `TaskNotExist`: If the task does not exist in storage.
    ///
    /// # Events
    /// - Emits `TaskChanged` when the task is successfully signed.
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

    /// Moves a task to the result storage after completion.
    ///
    /// # It ensures
    /// - Transfers the deposit and tips to the worker's account.
    /// - Removes the task from the active tasks storage and adds it to the results storage.
    ///
    /// # Parameters
    /// - `origin`: The account ID of the caller.
    /// - `task_id`: The ID of the task to move.
    ///
    /// # Errors
    /// - `TaskNotExist`: If the task does not exist in storage.
    /// - `TaskMustBeInClearState`: If the task is not in the `Clear` state.
    /// - `TaskHasNoResult`: If the task has no associated result.
    /// - `DepositOverflow`: If the deposit calculation overflows.
    ///
    /// # Events
    /// - Emits `TaskCleared` when the task is successfully moved to the result storage.
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

    /// Retrieves all tasks in the specified state.
    ///
    /// # It ensures
    /// - Filters tasks by the given state and returns them as a vector.
    ///
    /// # Parameters
    /// - `state`: The state to filter tasks by.
    pub fn get_tasks_by_state(state: TaskState) -> Vec<TaskFor<T, I>> {
        Tasks::<T, I>::iter()
            .filter(|(_, v)| v.state == state)
            .map(|(_, v)| v)
            .collect::<Vec<TaskFor<T, I>>>()
    }
}
