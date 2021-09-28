use dirs;
use std::env;
use std::path::PathBuf;

pub fn indy_home_path() -> PathBuf {
    let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/home/indy"));
    let mut indy_client_dir = ".indy_client";
    if cfg!(target_os = "ios") {
        indy_client_dir = "Documents/.indy_client";
    }
    path.push(indy_client_dir);

    if cfg!(target_os = "android") {
        path = android_indy_client_dir_path();
    }
    path
}

pub fn android_indy_client_dir_path() -> PathBuf {
    let external_storage = env::var("EXTERNAL_STORAGE");
    let android_dir: String;
    match external_storage {
        Ok(val) => android_dir = val + "/.indy_client",
        Err(err) => {
            panic!("Failed to find external storage path {:?}", err)
        }
    }

    PathBuf::from(android_dir)
}

pub fn genesis_transactions_path(name: &str) -> PathBuf {
    let mut path = indy_home_path();
    path.push("genesis_transactions");
    path.push(format!("{}.txn", name));
    path
}