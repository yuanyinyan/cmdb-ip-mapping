use std::borrow::BorrowMut;

use rlink::core;
use rlink::core::element::Record;
use rlink::core::function::{CoProcessFunction, Context};
use rlink_connector_kafka::KafkaRecord;

use crate::ip_mapping_config::{update_ip_mapping_by_id, IpMappingItem};

#[derive(Debug, Function)]
pub struct IpMappingCoProcessFunction {}

impl IpMappingCoProcessFunction {
    pub fn new() -> Self {
        IpMappingCoProcessFunction {}
    }
}

impl CoProcessFunction for IpMappingCoProcessFunction {
    fn open(&mut self, _context: &Context) -> core::Result<()> {
        Ok(())
    }

    fn process_left(&mut self, record: Record) -> Box<dyn Iterator<Item = Record>> {
        Box::new(vec![record].into_iter())
    }

    fn process_right(
        &mut self,
        stream_seq: usize,
        mut record: Record,
    ) -> Box<dyn Iterator<Item = Record>> {
        let kafka_record = KafkaRecord::new(record.borrow_mut());
        let payload = match kafka_record.get_kafka_payload() {
            Ok(payload) => payload,
            _ => return Box::new(vec![].into_iter()),
        };
        let line = match String::from_utf8(payload.to_vec()) {
            Ok(line) => line,
            _ => return Box::new(vec![].into_iter()),
        };
        info!(
            "ip mapping config update:stream_seq={},value={}",
            stream_seq, line
        );
        match update_ip_mapping(line) {
            Ok(()) => {}
            Err(e) => error!("update ip mapping error.{}", e),
        }
        Box::new(vec![].into_iter())
    }

    fn close(&mut self) -> core::Result<()> {
        Ok(())
    }
}

fn update_ip_mapping(context: String) -> serde_json::Result<()> {
    let response: IpMappingChangeResponse = serde_json::from_str(context.as_str())?;
    let is_del = response.change_type.eq("delete");
    update_ip_mapping_by_id(response.info, is_del);
    Ok(())
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct IpMappingChangeResponse {
    #[serde(rename = "changeType")]
    change_type: String,
    info: IpMappingItem,
}
