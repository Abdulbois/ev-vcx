#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Transport {
    return_route: String,
}

impl Default for Transport {
    fn default() -> Transport {
        Transport {
            return_route: String::from("thread")
        }
    }
}

#[macro_export]
macro_rules! return_route (($type:ident) => (
    impl $type {
        pub fn request_return_route(mut self) -> $type {
            self.transport = Some(Transport::default());
            self
        }

    }
));