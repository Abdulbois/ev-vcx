use rand::Rng;

use dashmap::DashMap;
use dashmap::mapref::entry::Entry;
use crate::error::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;
use super::error::prelude::VcxResult;
use std::marker::PhantomData;
use std::cmp;

// NOTE: we manually implement standard traits to get around
// limitations of derive macros (the bounds on T are too strict)
#[derive(Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct Handle<T>(u32, #[serde(skip)] PhantomData<T>);

impl<T> Handle<T> {
    pub const fn dummy() -> Self {
        Self(0, PhantomData)
    }
}

impl<T> PartialEq for Handle<T> {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl<T> Eq for Handle<T> {}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self(self.0, PhantomData)
    }
}

impl<T> Copy for Handle<T> {}

impl<T> cmp::PartialOrd<u32> for Handle<T> {
    fn partial_cmp(&self, other: &u32) -> Option<cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

impl<T> PartialEq<u32> for Handle<T> {
    fn eq(&self, other: &u32) -> bool {
        self.0 == *other
    }
}

impl<T> fmt::Debug for Handle<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<T> fmt::Display for Handle<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<T> From<Handle<T>> for u32 {
    fn from(h: Handle<T>) -> u32 {
        h.0
    }
}

impl<T> Default for Handle<T> {
    fn default() -> Self {
        Self::dummy()
    }
}

// this is only for testing purposes; allowing this in regular code would
// defeat the whole purpose of the type guarantees
#[cfg(test)]
impl<T> From<u32> for Handle<T> {
    fn from(n: u32) -> Self {
        Self(n, PhantomData)
    }
}

pub struct ObjectCache<T> {
    store: DashMap<u32, T>,
}

impl<T> Default for ObjectCache<T> {
    fn default() -> ObjectCache<T> {
        ObjectCache {
            store: Default::default(),
        }
    }
}

impl<T> ObjectCache<T> {
    pub fn has_handle(&self, handle: Handle<T>) -> bool {
        self.store.contains_key(&handle.into())
    }

    pub fn get<F, R>(&self, handle: Handle<T>, closure: F) -> VcxResult<R>
    where
        F: FnOnce(&T) -> VcxResult<R>,
    {
        closure(
            self.store
                .get(&handle.into())
                .ok_or_else(|| VcxError::from_msg(
                    VcxErrorKind::InvalidHandle,
                    format!("Object not found for handle: {}", handle),
                ))?
                .value(),
        )
    }

    pub fn get_mut<F, R>(&self, handle: Handle<T>, closure: F) -> VcxResult<R>
    where
        F: FnOnce(&mut T) -> VcxResult<R>,
    {
        closure(
            self.store
                .get_mut(&handle.into())
                .ok_or_else(|| VcxError::from_msg(
                    VcxErrorKind::InvalidHandle,
                    format!("Object not found for handle: {}", handle),
                ))?
                .value_mut(),
        )
    }

    pub fn add(&self, obj: T) -> VcxResult<Handle<T>> {
        let mut rng = rand::thread_rng();
        loop {
            // use the Entry API to avoid calculating the hash twice
            if let Entry::Vacant(v) = self.store.entry(rng.gen()) {
                let handle = Handle(*v.key(), PhantomData);
                v.insert(obj);
                // TODO: decide if this needs to return Result since this is infallible
                // TODO: is it okay for a handle to be zero?
                return Ok(handle);
            }
        }
    }

    pub fn insert(&self, handle: Handle<T>, obj: T) -> VcxResult<()> {
        // TODO: decide if we should keep returning an error since
        // DashMap doesn't return a Result (infalliable)
        self.store.insert(handle.into(), obj);
        Ok(())
    }

    pub fn release(&self, handle: Handle<T>) -> VcxResult<()> {
        if self.store.remove(&handle.into()).is_some() {
            Ok(())
        } else {
            Err(VcxError::from_msg(
                VcxErrorKind::InvalidHandle,
                format!("Object not found for handle: {}", handle),
            ))
        }
    }

    pub fn drain(&self) -> VcxResult<()> {
        // TODO: decide if we should keep returning an error since
        // DashMap doesn't return a Result (infalliable)
        self.store.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::object_cache::ObjectCache;
    use std::thread;
    use crate::utils::devsetup::SetupDefaults;

    lazy_static! {
        static ref TEST_CACHE: ObjectCache<String> = Default::default();
    }

    #[test]
    fn create_test() {
        let _setup = SetupDefaults::init();

        let _c: ObjectCache<u32> = Default::default();
    }

    #[test]
    fn get_closure() {
        let _setup = SetupDefaults::init();

        let test: ObjectCache<u32> = Default::default();
        let handle = test.add(2222).unwrap();
        let rtn = test.get(handle, |&obj| Ok(obj));
        assert_eq!(2222, rtn.unwrap())
    }

    #[test]
    fn to_string_test() {
        let _setup = SetupDefaults::init();

        let test: ObjectCache<u32> = Default::default();
        let handle = test.add(2222).unwrap();
        let string: String = test.get(handle, |_| Ok(String::from("TEST"))).unwrap();

        assert_eq!("TEST", string);
    }

    #[test]
    fn mut_object_test() {
        let _setup = SetupDefaults::init();

        let test: ObjectCache<String> = Default::default();
        let handle = test.add(String::from("TEST")).unwrap();

        test.get_mut(handle, |obj| {
            obj.to_lowercase();
            Ok(())
        })
        .unwrap();

        let string: String = test.get(handle, |obj| Ok(obj.clone())).unwrap();

        assert_eq!("TEST", string);
    }

    #[test]
    fn multi_thread_get() {
        for i in 0..2000 {
            let test_str = format!("TEST_MULTI_{}", i.to_string());
            let test_str1 = test_str.clone();
            let handle = TEST_CACHE.add(test_str).unwrap();
            let t1 = thread::spawn(move || {
                TEST_CACHE
                    .get_mut(handle, |s| {
                        s.insert_str(0, "THREAD1_");
                        Ok(())
                    })
                    .unwrap()
            });
            let t2 = thread::spawn(move || {
                TEST_CACHE
                    .get_mut(handle, |s| {
                        s.push_str("_THREAD2");
                        Ok(())
                    })
                    .unwrap()
            });
            t1.join().unwrap();
            t2.join().unwrap();
            TEST_CACHE
                .get(handle, |s| {
                    let expected_str = format!("THREAD1_{}_THREAD2", test_str1);
                    assert_eq!(&expected_str, s);
                    Ok(())
                })
                .unwrap();
        }
    }
}
