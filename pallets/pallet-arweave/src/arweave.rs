use crate::*;

use polkadot_sdk::*;

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use sp_runtime::offchain::{http, Duration};

impl<T: Config<I>, I: 'static> Pallet<T, I> {
    pub fn arweave_fee(data_len: usize) -> Result<u64, http::Error> {
        let deadline = sp_io::offchain::timestamp().add(Duration::from_millis(5_000));
        let url = format!("https://arweave.net/price/{}", data_len);
        let request = http::Request::get(&url);
        let pending = request
            .deadline(deadline)
            .send()
            .map_err(|_| http::Error::IoError)?;
        let response = pending
            .try_wait(deadline)
            .map_err(|_| http::Error::DeadlineReached)??;
        if response.code != 200 {
            log!(warn, "Unexpected status code: {}", response.code);
            return Err(http::Error::Unknown);
        }
        let body = response.body().collect::<Vec<u8>>();
        let body_str = alloc::str::from_utf8(&body).map_err(|_| {
            log!(warn, "No UTF8 body");
            http::Error::Unknown
        })?;

        let num: u64 = body_str.parse().map_err(|_| {
            log!(warn, "can not parse as u64");
            http::Error::Unknown
        })?;

        Ok(num)
    }

    pub fn arweave_last_tx() -> Result<String, http::Error> {
        let deadline = sp_io::offchain::timestamp().add(Duration::from_millis(5_000));
        let request = http::Request::get("https://arweave.net/tx_anchor");
        let pending = request
            .deadline(deadline)
            .send()
            .map_err(|_| http::Error::IoError)?;
        let response = pending
            .try_wait(deadline)
            .map_err(|_| http::Error::DeadlineReached)??;
        if response.code != 200 {
            log!(warn, "Unexpected status code: {}", response.code);
            return Err(http::Error::Unknown);
        }
        let body = response.body().collect::<Vec<u8>>();
        let body_str = alloc::str::from_utf8(&body).map_err(|_| {
            log!(warn, "No UTF8 body");
            http::Error::Unknown
        })?;

        Ok(body_str.to_string())
    }

    pub fn arweave_post_transaction(body: Vec<&[u8]>) -> Result<(), http::Error> {
        let deadline = sp_io::offchain::timestamp().add(Duration::from_millis(5_000));
        let request = http::Request::post("https://arweave.net/tx", body);
        let pending = request
            .deadline(deadline)
            .add_header("Accept", "application/json")
            .add_header("Content-Type", "application/json")
            .send()
            .map_err(|_| http::Error::IoError)?;
        let response = pending
            .try_wait(deadline)
            .map_err(|_| http::Error::DeadlineReached)??;
        if response.code != 200 {
            log!(warn, "Unexpected status code: {}", response.code);
            return Err(http::Error::Unknown);
        }

        Ok(())
    }

    pub fn arweave_get_transaction(tx_id: Vec<u8>) -> Result<u16, OffchainWorkerError> {
        let tx_id = String::from_utf8(tx_id).unwrap();
        let deadline = sp_io::offchain::timestamp().add(Duration::from_millis(5_000));
        let url = format!("https://arweave.net/tx/{}", tx_id);
        let request = http::Request::get(&url);
        let pending = request
            .deadline(deadline)
            .send()
            .map_err(|_| http::Error::IoError)
            .map_err(|e| OffchainWorkerError::HttpRequestError(e))?;
        let response = pending
            .try_wait(deadline)
            .map_err(|_| http::Error::DeadlineReached)
            .map_err(|e| OffchainWorkerError::HttpRequestError(e))?
            .map_err(|e| OffchainWorkerError::HttpRequestError(e))?;
        
        Ok(response.code)
    }
}
