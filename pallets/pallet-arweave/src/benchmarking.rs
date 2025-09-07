//! Benchmarking setup for pallet-template

use super::*;
use frame::{deps::frame_benchmarking::v2::*, prelude::*};
use frame_system::RawOrigin;

#[benchmarks]
mod benchmarks {
    use super::*;
    #[cfg(test)]
    use crate::pallet::Pallet as Template;

    #[benchmark]
    fn create_task() {
        let caller: T::AccountId = whitelisted_caller();
        let task_id = T::TaskId::initial_value().unwrap();
        let worker_address: T::AccountId = whitelisted_caller();
        let data_s = "{}";
        let data_v: Vec<u8> = data_s.as_bytes().to_vec();
        let data: BoundedVec<u8, T::MaxDataLength> = data_v.try_into().unwrap();

        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        let amount = BalanceOf::<T>::from(T::Currency::minimum_balance() * 2u32.into());
        let tips = BalanceOf::<T>::from(T::Currency::minimum_balance());

        #[extrinsic_call]
        create_task(
            RawOrigin::Signed(caller),
            worker_address.clone(),
            data.clone(),
            amount,
            tips,
        );

        let task = Tasks::<T>::get(task_id).unwrap();
        assert_eq!(task.task_id, task_id);
        assert_eq!(task.worker_address, worker_address);
        assert_eq!(task.data, data);
        assert_eq!(task.state, TaskState::Sign);
    }

    impl_benchmark_test_suite!(Template, crate::mock::new_test_ext(), crate::mock::Test);
}
