//! Benchmarking setup for pallet-template

use super::*;
use frame::{deps::frame_benchmarking::v2::*, prelude::*};

#[benchmarks]
mod benchmarks {
    use super::*;
    #[cfg(test)]
    use crate::pallet::Pallet as Template;
    use frame_system::RawOrigin;

    #[benchmark]
    fn create_task() {
        let caller: T::AccountId = whitelisted_caller();
        let task_id = T::TaskId::initial_value().unwrap();
        let worker_address: T::AccountId = whitelisted_caller();
        let data: BoundedVec<u8, T::MaxDataLength> = vec![1, 2, 3].try_into().unwrap();

        #[extrinsic_call]
        create_task(
            RawOrigin::Signed(caller),
            worker_address.clone(),
            data.clone(),
        );

        let task = Tasks::<T>::get(task_id).unwrap();
        assert_eq!(task.task_id, task_id);
        assert_eq!(task.worker_address, worker_address);
        assert_eq!(task.data, data);
        assert_eq!(task.state, TaskState::Sign);
    }

    impl_benchmark_test_suite!(Template, crate::mock::new_test_ext(), crate::mock::Test);
}
