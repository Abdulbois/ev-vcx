use futures::Future;
use indy::vdr;
use indy::vdr::VDR;

use crate::error::prelude::*;
use crate::settings;
use crate::settings::pool::get_indy_pool_networks;

pub const DEFAULT_NETWORK: &'static str = "sov";

pub struct VDRInfo {
    // list of namespaces paining to unique Ledgers
    pub namespace_list: Vec<String>,
    pub vdr: VDR,
}

pub static mut VDR_INFO: Option<VDRInfo> = None;

pub fn get_vdr<'a>() -> VcxResult<&'a VDRInfo> {
    unsafe {
        VDR_INFO.as_ref()
            .ok_or(VcxError::from_msg(VcxErrorKind::NoPoolOpen,
                                      "There is no Pool opened"))
    }
}

pub fn get_namespace() -> String {
    unsafe {
        VDR_INFO.as_ref()
            .and_then(|vdr_info| vdr_info.namespace_list.get(0).cloned())
            .unwrap_or(DEFAULT_NETWORK.to_string())
    }
}

pub fn reset_vdr() {
    unsafe {
        VDR_INFO = None;
    }
}


pub fn init_vdr() -> VcxResult<()> {
    debug!("init_vdr >>>");

    if settings::indy_mocks_enabled() { return Ok(()); }

    if get_vdr().is_ok() {
        debug!("VDR is already initialized.");
        return Ok(());
    }

    let networks = get_indy_pool_networks()?;
    let mut namespace_list = Vec::new();

    let mut vdr_builder = vdr::vdr_builder_create()?;

    for network in networks {
        let taa_config = network.taa_config.map(|taa_config| json!(taa_config).to_string());
        if let Some(namespace) = network.namespace_list.first() {
            namespace_list.push(namespace.to_string());
            vdr::vdr_builder_register_indy_ledger(&mut vdr_builder,
                                                  &json!(network.namespace_list).to_string(),
                                                  &network.genesis_transactions,
                                                  taa_config.as_deref()).wait()?;
        }
    }

    let vdr = vdr::vdr_builder_finalize(vdr_builder)?;
    vdr::ping(&vdr, &json!(namespace_list).to_string()).wait()?;

    unsafe {
        VDR_INFO = Some(VDRInfo {
            namespace_list,
            vdr,
        })
    }

    Ok(())
}

pub fn close_vdr() -> VcxResult<()> {
    debug!("close_vdr >>>");
    unsafe {
        let vdr_info = VDR_INFO.take()
            .ok_or(VcxError::from_msg(VcxErrorKind::NoPoolOpen,
                                      "There is no VDR opened"))?;

        vdr::cleanup(vdr_info.vdr).wait()?;
        reset_vdr();
    }
    Ok(())
}

#[cfg(test)]
pub mod tests {
    use super::*;
    #[cfg(feature = "pool_tests")]
    use crate::utils::devsetup::SetupLibraryWalletPoolZeroFees;
    use settings::pool::IndyPoolConfig;

    pub fn delete_test_pool() {
        close_vdr().ok();
    }

    pub fn open_test_pool() {
        let test_pool_ip = ::std::env::var("TEST_POOL_IP").unwrap_or("127.0.0.1".to_string());

        let pool_configs: Vec<IndyPoolConfig> = vec![IndyPoolConfig {
            genesis_transactions: get_txns(&test_pool_ip),
            namespace_list: vec![DEFAULT_NETWORK.to_string()],
            taa_config: None
        }];
        settings::set_config_value(settings::CONFIG_INDY_POOL_NETWORKS, &json!(pool_configs).to_string());
        init_vdr().unwrap();
    }

    pub fn get_txns(test_pool_ip: &str) -> String {
        vec![format!(r#"{{"reqSignature":{{}},"txn":{{"data":{{"data":{{"alias":"Node1","blskey":"4N8aUNHSgjQVgkpm8nhNEfDf6txHznoYREg9kirmJrkivgL4oSEimFF6nsQ6M41QvhM2Z33nves5vfSn9n1UwNFJBYtWVnHYMATn76vLuL3zU88KyeAYcHfsih3He6UHcXDxcaecHVz6jhCYz1P2UZn2bDVruL5wXpehgBfBaLKm3Ba","blskey_pop":"RahHYiCvoNCtPTrVtP7nMC5eTYrsUA8WjXbdhNc8debh1agE9bGiJxWBXYNFbnJXoXhWFMvyqhqhRoq737YQemH5ik9oL7R4NTTCz2LEZhkgLJzB3QRQqJyBNyv7acbdHrAT8nQ9UkLbaVL9NBpnWXBTw4LEMePaSHEw66RzPNdAX1","client_ip":"{}","client_port":9702,"node_ip":"{}","node_port":9701,"services":["VALIDATOR"]}},"dest":"Gw6pDLhcBcoQesN72qfotTgFa7cbuqZpkX3Xo6pLhPhv"}},"metadata":{{"from":"Th7MpTaRZVRYnPiabds81Y"}},"type":"0"}},"txnMetadata":{{"seqNo":1,"txnId":"fea82e10e894419fe2bea7d96296a6d46f50f93f9eeda954ec461b2ed2950b62"}},"ver":"1"}}"#, test_pool_ip, test_pool_ip),
             format!(r#"{{"reqSignature":{{}},"txn":{{"data":{{"data":{{"alias":"Node2","blskey":"37rAPpXVoxzKhz7d9gkUe52XuXryuLXoM6P6LbWDB7LSbG62Lsb33sfG7zqS8TK1MXwuCHj1FKNzVpsnafmqLG1vXN88rt38mNFs9TENzm4QHdBzsvCuoBnPH7rpYYDo9DZNJePaDvRvqJKByCabubJz3XXKbEeshzpz4Ma5QYpJqjk","blskey_pop":"Qr658mWZ2YC8JXGXwMDQTzuZCWF7NK9EwxphGmcBvCh6ybUuLxbG65nsX4JvD4SPNtkJ2w9ug1yLTj6fgmuDg41TgECXjLCij3RMsV8CwewBVgVN67wsA45DFWvqvLtu4rjNnE9JbdFTc1Z4WCPA3Xan44K1HoHAq9EVeaRYs8zoF5","client_ip":"{}","client_port":9704,"node_ip":"{}","node_port":9703,"services":["VALIDATOR"]}},"dest":"8ECVSk179mjsjKRLWiQtssMLgp6EPhWXtaYyStWPSGAb"}},"metadata":{{"from":"EbP4aYNeTHL6q385GuVpRV"}},"type":"0"}},"txnMetadata":{{"seqNo":2,"txnId":"1ac8aece2a18ced660fef8694b61aac3af08ba875ce3026a160acbc3a3af35fc"}},"ver":"1"}}"#, test_pool_ip, test_pool_ip),
             format!(r#"{{"reqSignature":{{}},"txn":{{"data":{{"data":{{"alias":"Node3","blskey":"3WFpdbg7C5cnLYZwFZevJqhubkFALBfCBBok15GdrKMUhUjGsk3jV6QKj6MZgEubF7oqCafxNdkm7eswgA4sdKTRc82tLGzZBd6vNqU8dupzup6uYUf32KTHTPQbuUM8Yk4QFXjEf2Usu2TJcNkdgpyeUSX42u5LqdDDpNSWUK5deC5","blskey_pop":"QwDeb2CkNSx6r8QC8vGQK3GRv7Yndn84TGNijX8YXHPiagXajyfTjoR87rXUu4G4QLk2cF8NNyqWiYMus1623dELWwx57rLCFqGh7N4ZRbGDRP4fnVcaKg1BcUxQ866Ven4gw8y4N56S5HzxXNBZtLYmhGHvDtk6PFkFwCvxYrNYjh","client_ip":"{}","client_port":9706,"node_ip":"{}","node_port":9705,"services":["VALIDATOR"]}},"dest":"DKVxG2fXXTU8yT5N7hGEbXB3dfdAnYv1JczDUHpmDxya"}},"metadata":{{"from":"4cU41vWW82ArfxJxHkzXPG"}},"type":"0"}},"txnMetadata":{{"seqNo":3,"txnId":"7e9f355dffa78ed24668f0e0e369fd8c224076571c51e2ea8be5f26479edebe4"}},"ver":"1"}}"#, test_pool_ip, test_pool_ip),
             format!(r#"{{"reqSignature":{{}},"txn":{{"data":{{"data":{{"alias":"Node4","blskey":"2zN3bHM1m4rLz54MJHYSwvqzPchYp8jkHswveCLAEJVcX6Mm1wHQD1SkPYMzUDTZvWvhuE6VNAkK3KxVeEmsanSmvjVkReDeBEMxeDaayjcZjFGPydyey1qxBHmTvAnBKoPydvuTAqx5f7YNNRAdeLmUi99gERUU7TD8KfAa6MpQ9bw","blskey_pop":"RPLagxaR5xdimFzwmzYnz4ZhWtYQEj8iR5ZU53T2gitPCyCHQneUn2Huc4oeLd2B2HzkGnjAff4hWTJT6C7qHYB1Mv2wU5iHHGFWkhnTX9WsEAbunJCV2qcaXScKj4tTfvdDKfLiVuU2av6hbsMztirRze7LvYBkRHV3tGwyCptsrP","client_ip":"{}","client_port":9708,"node_ip":"{}","node_port":9707,"services":["VALIDATOR"]}},"dest":"4PS3EDQ3dW1tci1Bp6543CfuuebjFrg36kLAUcskGfaA"}},"metadata":{{"from":"TWwCRQRZ2ZHMJFn9TzLp7W"}},"type":"0"}},"txnMetadata":{{"seqNo":4,"txnId":"aa5e817d7cc626170eca175822029339a444eb0ee8f0bd20d3b0b76e566fb008"}},"ver":"1"}}"#, test_pool_ip, test_pool_ip)]
            .join("\n")
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_open_close_pool() {
        open_test_pool();
        let _vdr = get_vdr().unwrap();
        close_vdr().unwrap();

    }
}
