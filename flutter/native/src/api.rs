// This is the entry point of your Rust library.
// When adding new code to your project, note that only items used
// here will be transformed to their Dart equivalents.

use std::sync::Arc;
use std::time::Duration;

use lazy_static::lazy_static;
use moksha_core::model::PaymentRequest;
use moksha_fedimint::FedimintWallet;
use moksha_wallet::client::HttpClient;
use moksha_wallet::localstore::LocalStore;
use moksha_wallet::localstore::SqliteLocalStore;
use moksha_wallet::wallet::Wallet;
use reqwest::Url;
use std::sync::Mutex as StdMutex;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;
use tokio::time::{sleep_until, Instant};

lazy_static! {
    static ref RUNTIME: Arc<StdMutex<Runtime>> = Arc::new(StdMutex::new(
        Runtime::new().expect("Failed to create runtime")
    ));
    static ref LOCALSTORE: Arc<Mutex<Option<SqliteLocalStore>>> = Arc::new(Mutex::new(None));
    static ref HTTPCLIENT: Arc<Mutex<Option<HttpClient>>> = Arc::new(Mutex::new(None));
}

macro_rules! lock_runtime {
    () => {
        match RUNTIME.lock() {
            Ok(lock) => lock,
            Err(err) => {
                let err: anyhow::Error =
                    anyhow::anyhow!("Failed to lock the runtime mutex: {}", err);
                return Err(err.into());
            }
        }
    };
}

pub fn init_cashu() -> anyhow::Result<String> {
    let rt = lock_runtime!();

    let db_path = Wallet::db_path();

    let new_localstore = rt.block_on(async {
        SqliteLocalStore::with_path(db_path.clone())
            .await
            .map_err(anyhow::Error::from)
    })?;

    rt.block_on(async {
        let mut db = LOCALSTORE.lock().await;
        new_localstore.migrate().await;
        *db = Some(new_localstore);

        let mut cl = HTTPCLIENT.lock().await;
        let client = HttpClient::new();
        *cl = Some(client);
    });

    drop(rt);
    Ok(db_path)
}

pub fn get_balance() -> anyhow::Result<u64> {
    let rt = lock_runtime!();

    let result = rt.block_on(async {
        let db = LOCALSTORE.lock().await;

        let total = match db.as_ref() {
            Some(db) => db.get_proofs().await?.total_amount(),
            None => 0,
        };

        Ok(total)
    });
    drop(rt);
    result
}

fn _create_local_wallet() -> anyhow::Result<Wallet> {
    let mint_url = Url::parse("http://127.0.0.1:3338").expect("invalid url"); // FIXME redundant
    let rt = lock_runtime!();

    let result = rt.block_on(async move {
        let client = HTTPCLIENT.lock().await;
        let client = client.as_ref().expect("HTTPClient not set");

        let localstore = LOCALSTORE.lock().await;
        let localstore = localstore.as_ref().expect("DB not set");

        Ok(Wallet::builder()
            .with_client(Box::new(client.to_owned()))
            .with_localstore(Box::new(localstore.to_owned()))
            .with_mint_url(mint_url)
            .build()
            .await?)
    });
    drop(rt);
    result
}

pub fn mint_tokens(amount: u64, hash: String) -> anyhow::Result<u64> {
    let wallet = _create_local_wallet().map_err(anyhow::Error::from)?;
    let rt = lock_runtime!();

    let result = rt.block_on(async {
        for _ in 0..30 {
            sleep_until(Instant::now() + Duration::from_millis(1_000)).await;
            let mint_result = wallet.mint_tokens(amount.into(), hash.clone()).await;

            match mint_result {
                Ok(value) => {
                    return Ok(value.total_amount());
                }
                Err(moksha_wallet::error::MokshaWalletError::InvoiceNotPaidYet(_, _)) => {
                    continue;
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        Err(moksha_wallet::error::MokshaWalletError::InvoiceNotPaidYet(
            amount,
            "Invoice not paid yet".to_string(),
        ))
    });

    drop(rt);
    result.map_err(anyhow::Error::from)
}

pub fn get_mint_payment_request(amount: u64) -> anyhow::Result<FlutterPaymentRequest> {
    let wallet = _create_local_wallet().map_err(anyhow::Error::from)?;
    let rt = lock_runtime!();

    let result = rt.block_on(async {
        wallet
            .get_mint_payment_request(amount)
            .await
            .map_err(anyhow::Error::from)
    })?;

    drop(rt);
    Ok(result.into())
}

#[derive(Clone)]
pub struct FlutterPaymentRequest {
    pub pr: String,
    pub hash: String,
}

impl From<PaymentRequest> for FlutterPaymentRequest {
    fn from(value: PaymentRequest) -> Self {
        Self {
            pr: value.pr,
            hash: value.hash,
        }
    }
}

pub fn pay_invoice(invoice: String) -> anyhow::Result<bool> {
    let wallet = _create_local_wallet().map_err(anyhow::Error::from)?;
    let rt = lock_runtime!();

    let result = rt.block_on(async {
        wallet
            .pay_invoice(invoice)
            .await
            .map_err(anyhow::Error::from)
    })?;

    drop(rt);
    Ok(result.paid)
}

pub fn import_token(token: String) -> anyhow::Result<u64> {
    let deserialized_token = token.try_into().map_err(anyhow::Error::from)?;
    let wallet = _create_local_wallet().map_err(anyhow::Error::from)?;
    let rt = lock_runtime!();

    rt.block_on(async {
        wallet
            .receive_tokens(&deserialized_token)
            .await
            .map_err(anyhow::Error::from)
    })?;

    drop(rt);
    Ok(deserialized_token.total_amount())
}

pub fn join_federation(federation: String) -> anyhow::Result<()> {
    let rt = lock_runtime!();
    let workdir = Wallet::config_dir();

    rt.block_on(async {
        FedimintWallet::connect(workdir, &federation)
            .await
            .map_err(anyhow::Error::from)
    })?;

    drop(rt);
    Ok(())
}

// pub fn fedimint_mint_token(amount: u64, hash: String) -> anyhow::Result<LnReceiveState> {
//     let rt = lock_runtime!();
//     let workdir = Wallet::config_dir();

//     let result = rt.block_on(async {
//         let wallet = FedimintWallet::new(workdir).await?;
//         wallet.mint(amount).await.map_err(anyhow::Error::from)
//     })?;

//     drop(rt);
//     Ok(result)
// }

#[cfg(test)]
mod tests {
    use super::{get_balance, init_cashu};

    #[test]
    fn test_get_balance() -> anyhow::Result<()> {
        let tmp = tempfile::tempdir().expect("Could not create tmp dir");
        let tmp_dir = tmp.path().to_str().expect("Could not create tmp dir");

        std::env::set_var(
            moksha_wallet::wallet::ENV_DB_PATH,
            format!("{}/wallet.db", tmp_dir),
        );
        let _ = init_cashu()?;
        let balance = get_balance().expect("Could not get balance");
        assert_eq!(0, balance);
        Ok(())
    }
}
