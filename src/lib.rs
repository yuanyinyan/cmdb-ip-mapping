//! # cmdb-ip-mapping
//!
//! A library to get appuk by ip from cmdb in [rlink](https://docs.rs/rlink) task.
//!
//! # Example
//!
//! ## Get appUk by ip
//! ```
//! use cmdb_ip_mapping::ip_mapping_config::{load_ip_mapping_task, get_ip_mapping_config};
//!
//! let ip = "test_ip";
//! let ip_mapping_url = "test_mapping_url";
//!
//! load_ip_mapping_task(ip_mapping_url);
//!
//! let vec = get_ip_mapping_config(ip);
//! let item = vec.unwrap().get(0).unwrap().clone();
//! assert!(item.group_environment.as_ref().unwrap().eq("qa"))
//! ```
//!
//! ## Consume ip-mapping incremental change
//! ```
//! use std::collections::HashMap;
//! use rlink::functions::broadcast_flat_map::BroadcastFlagMapFunction;
//! use cmdb_ip_mapping::ip_mapping_connect::IpMappingCoProcessFunction;
//! use rlink_connector_kafka::{BOOTSTRAP_SERVERS, GROUP_ID, create_input_format};
//!
//! let ip_mapping_input_format = {
//!     let mut conf_map = HashMap::new();
//!     conf_map.insert(BOOTSTRAP_SERVERS.to_string(), ip_mapping_kafka_servers);
//!     conf_map.insert(GROUP_ID.to_string(), ip_mapping_group_id);
//!     create_input_format(conf_map, vec![ip_mapping_kafka_topic], Some(10000), None)
//! };
//!
//! let ip_mapping_stream = env
//!     .register_source(ip_mapping_input_format, 1)
//!     .flat_map(BroadcastFlagMapFunction::new());
//!
//! data_stream
//!     .connect(
//!          vec![CoStream::from(ip_mapping_stream)],
//!          IpMappingCoProcessFunction::new(),
//!     )
//! ```

#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate rlink_derive;

pub mod ip_mapping_config;
pub mod ip_mapping_connect;

#[cfg(test)]
mod tests {
    use crate::ip_mapping_config::{load_ip_mapping_task, get_ip_mapping_config};

    #[test]
    fn it_works() {
        let ip = "10.99.5.49";
        let ip_mapping_url = "http://10.99.5.49:8080/mapping/all/v2";

        load_ip_mapping_task(ip_mapping_url);

        let vec = get_ip_mapping_config(ip);
        let item = vec.unwrap().get(0).unwrap().clone();
        assert!(item.group_environment.as_ref().unwrap().eq("qa"))
    }
}
