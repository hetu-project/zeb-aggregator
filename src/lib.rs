pub mod restful;
pub use restful::*;
pub mod db;
pub mod nodes;

pub mod business {
    include!(concat!(env!("OUT_DIR"), "/bussiness.rs"));
}

pub mod vlc {
    include!(concat!(env!("OUT_DIR"), "/vlc.rs"));
}

pub mod zmessage {
    include!(concat!(env!("OUT_DIR"), "/zmessage.rs"));
}

pub mod gateway_response {
    include!(concat!(env!("OUT_DIR"), "/gateway.rs"));
}