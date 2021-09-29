use indy_sys::CommandHandle;
use libc::c_char;

use crate::connection::Connections;
use crate::credential::Credentials;
use crate::credential_def::CredentialDef;
use crate::disclosed_proof::DisclosedProofs;
use crate::issuer_credential::IssuerCredentials;
use crate::object_cache::Handle;
use crate::proof::Proofs;
use crate::schema::CreateSchema;
use crate::utils::cstring::{
    raw_slice_to_vec as to_buf, CStringUtils::c_str_to_opt_string as to_str,
};
use crate::utils::error;
use crate::utils::libindy::error_codes::map_indy_error_code;
use crate::wallet_backup::WalletBackup;
use std::collections::HashMap;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};
use std::sync::Mutex;
use std::time::Duration;

fn log_timeout(e: RecvTimeoutError) -> u32 {
    match e {
        RecvTimeoutError::Timeout => warn!("Timed out waiting for callback"),
        RecvTimeoutError::Disconnected => warn!("Channel to libindy was disconnected unexpectedly"),
    }
    error::TIMEOUT_LIBINDY_ERROR.code_num
}

pub struct Recv<T>(Receiver<(u32, T)>);

impl<T> Recv<T> {
    fn inner_recv(self, t: Duration) -> Result<T, u32> {
        let (err, rest) = self.0.recv_timeout(t).map_err(log_timeout)?;
        if err == 0 {
            Ok(rest)
        } else {
            Err(map_indy_error_code(err))
        }
    }
    pub fn recv_short(self) -> Result<T, u32> {
        self.inner_recv(Duration::from_secs(5))
    }
    pub fn recv_medium(self) -> Result<T, u32> {
        self.inner_recv(Duration::from_secs(15))
    }
    pub fn recv_long(self) -> Result<T, u32> {
        self.inner_recv(Duration::from_secs(50))
    }
    pub fn recv(self) -> Result<T, u32> {
        self.recv_medium()
    }
    pub fn recv_with(self, t: Duration) -> Result<T, u32> {
        self.inner_recv(t)
    }
}

mod cb {
    use super::*;

    // NOTE: using a global map is a hack to get around the fact that closures cannot be
    // extern fn; callers are given an extern fn callback that immediately sends its
    // parameters down a channel when invoked, allowing the receiver to recover the values
    // after the callback is executed on the threadpool
    lazy_static! {
        static ref API_MOCK: Mutex<HashMap<CommandHandle, Senders>> = Default::default();
    }

    fn gen_handle() -> CommandHandle {
        static N: AtomicI32 = AtomicI32::new(0);
        // NOTE: `Ordering::Relaxed` is okay because this is effectively a counter;
        // no synchronization on `N` takes place
        N.fetch_add(1, Ordering::Relaxed) + 1
    }

    pub(super) fn register<T, U>(
        to_senders: fn(Sender<(u32, T)>) -> Senders,
        cb: U,
    ) -> (CommandHandle, U, Recv<T>) {
        let handle = gen_handle();
        let (sender, receiver) = channel();
        API_MOCK.lock().unwrap().insert(handle, to_senders(sender));
        (handle, cb, Recv(receiver))
    }

    pub(super) fn unregister(handle: CommandHandle) -> Option<Senders> {
        API_MOCK.lock().unwrap().remove(&handle)
    }
}

macro_rules! impl_returns {
    ($senders:ident,
     $(fn $name:ident($($id:ident: $t:ty),*) -> $recv:ty {
        $send:expr
     })+) => {
        #[allow(non_camel_case_types)]
        #[derive(Debug)]
        pub enum $senders {
            $($name(Sender<(u32, $recv)>)),+
        }
        $(pub fn $name() -> (CommandHandle, extern fn(CommandHandle, u32, $($t),*), Recv<$recv>) {
            extern fn cb(h: CommandHandle, err: u32, $($id: $t),*) {
                #[allow(unreachable_patterns)]
                match cb::unregister(h) {
                    Some($senders::$name(s)) => s.send((err, $send))
                        .unwrap_or_else(|e| warn!("Unable to send through libindy callback in vcx: {:?}", e)),
                    Some(x) => warn!(concat!("Expected `", stringify!($name), "`, found: {:?}"), x),
                    None => warn!(concat!("unable to find `", stringify!($name), "` in map"))
                }
            }
            cb::register($senders::$name, cb)
        })+
    }
}

// NOTE: To add support for more functions, append an entry to the macro with this syntax:
// fn <public name>(<extern fn params, excluding command handle and error code>) -> <type to receive on success> { <compute from params> }
impl_returns! {
    Senders,
    fn return_u32() -> () { () }
    fn return_u32_u32(a: u32) -> u32 { a }
    fn return_u32_cxnh(a: Handle<Connections>) -> Handle<Connections> { a }
    fn return_u32_csh(a: Handle<CreateSchema>) -> Handle<CreateSchema> { a }
    fn return_u32_ih(a: Handle<IssuerCredentials>) -> Handle<IssuerCredentials> { a }
    fn return_u32_crdh(a: Handle<Credentials>) -> Handle<Credentials> { a }
    fn return_u32_cdh(a: Handle<CredentialDef>) -> Handle<CredentialDef> { a }
    fn return_u32_dph(a: Handle<DisclosedProofs>) -> Handle<DisclosedProofs> { a }
    fn return_u32_ph(a: Handle<Proofs>) -> Handle<Proofs> { a }
    fn return_u32_wh(a: Handle<WalletBackup>) -> Handle<WalletBackup> { a }
    fn return_u32_str(a: *const c_char) -> Option<String> { to_str(a) }
    fn return_u32_u32_str(a: u32, b: *const c_char) -> (u32, Option<String>) { (a, to_str(b)) }
    fn return_u32_csh_str(a: Handle<CreateSchema>, b: *const c_char) -> (Handle<CreateSchema>, Option<String>) { (a, to_str(b)) }
    fn return_u32_bool(a: bool) -> bool { a }
    fn return_u32_dph_str(a: Handle<DisclosedProofs>, b: *const c_char) -> (Handle<DisclosedProofs>, Option<String>) { (a, to_str(b)) }
    fn return_u32_crdh_str(a: Handle<Credentials>, b: *const c_char) -> (Handle<Credentials>, Option<String>) { (a, to_str(b)) }
    fn return_u32_bin(a: *const u8, len: u32) -> Vec<u8> { to_buf(a, len) }
    fn return_u32_cdh_str_str_str(a: Handle<CredentialDef>, b: *const c_char, c: *const c_char, d: *const c_char) -> (Handle<CredentialDef>, Option<String>, Option<String>, Option<String>) { (a, to_str(b), to_str(c), to_str(d)) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;

    #[test]
    fn test_return_u32() {
        let (h, cb, r) = return_u32();
        cb(h, 0);
        assert!(r.recv().is_ok());

        let (h, cb, r) = return_u32();
        cb(h, 123);
        assert!(r.recv().is_err());
    }

    #[test]
    fn test_return_u32_u32() {
        let test_val = 23455;

        let (h, cb, r) = return_u32_u32();
        cb(h, 0, test_val);
        assert_eq!(r.recv().unwrap(), test_val);

        let (h, cb, r) = return_u32_u32();
        cb(h, 123, test_val);
        assert!(r.recv().is_err());
    }

    #[test]
    fn test_return_u32_str() {
        let test_cstr = "Journey before destination\0";
        let test_str = &test_cstr[..test_cstr.len() - 1];

        let (h, cb, r) = return_u32_str();
        cb(h, 0, test_cstr.as_ptr().cast());
        assert_eq!(r.recv().unwrap().as_deref(), Some(test_str));

        let (h, cb, r) = return_u32_str();
        cb(h, 0, ptr::null());
        assert_eq!(r.recv().unwrap(), None);

        let (h, cb, r) = return_u32_str();
        cb(h, 123, test_cstr.as_ptr().cast());
        assert!(r.recv().is_err());
    }
}
