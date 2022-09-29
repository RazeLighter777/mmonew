#![feature(const_type_name)]
#![feature(arbitrary_enum_discriminant)]
#![allow(incomplete_features)]
#![feature(unsize)]
#![feature(specialization)]
#![allow(unused)]
#![deny(warnings)]
pub mod block_type;
pub mod chunk;
pub mod component;
pub mod effect;
pub mod entity_id;
mod hashing;
pub mod raws;
pub mod resource;
pub mod server_request_type;
pub mod server_response_type;
