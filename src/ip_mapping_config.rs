use std::collections::HashMap;

use dashmap::mapref::one::Ref;
use dashmap::DashMap;
use rlink::utils::date_time::current_timestamp;
use rlink::utils::http::client::get;
use tokio::time::Instant;

lazy_static! {
    // ip and cmdbInfo config
    static ref GLOBAL_IP_MAPPING: DashMap<String, Vec<IpMappingItem>> = DashMap::new();
    // cmdbId and ip config
    static ref GLOBAL_CMDB_ID_IP_MAP: DashMap<String, String> = DashMap::new();
}

pub fn load_ip_mapping_task(url: &str) {
    let url = url.to_string();
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(load_remote_ip_mapping(url.as_str()));

    // 每天凌晨4点取全量IpMapping数据
    let now = current_timestamp().as_secs();
    let period = 24 * 60 * 60;
    let mut diff = ((now / period + 1) * period - 4 * 60 * 60 - now) as i64;
    if diff < 0 {
        diff = diff + period as i64;
    }
    let start = Instant::now() + std::time::Duration::from_secs(diff as u64);

    std::thread::spawn(move || {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let mut interval =
                tokio::time::interval_at(start, std::time::Duration::from_secs(period));
            loop {
                interval.tick().await;

                load_remote_ip_mapping(url.as_str()).await;
            }
        });
    });
}

pub fn get_ip_mapping_config(ip: &str) -> Option<Ref<'_, String, Vec<IpMappingItem>>> {
    let config: &DashMap<String, Vec<IpMappingItem>> = &*GLOBAL_IP_MAPPING;
    config.get(ip)
}

fn update_ip_mapping_config(conf: HashMap<String, Vec<IpMappingItem>>) {
    let ip_mapping_config: &DashMap<String, Vec<IpMappingItem>> = &*GLOBAL_IP_MAPPING;
    let cmdb_id_ip_config: &DashMap<String, String> = &*GLOBAL_CMDB_ID_IP_MAP;
    let mut count = 0;
    for (ip, vec) in conf {
        for item in &vec {
            cmdb_id_ip_config.insert(item.id.clone(), ip.clone());
        }
        ip_mapping_config.insert(ip, vec);
        count += 1;
    }
    info!("update ip mapping config,size={}", count);
}

pub fn update_ip_mapping_by_id(item: IpMappingItem, is_del: bool) {
    let ip_mapping_config: &DashMap<String, Vec<IpMappingItem>> = &*GLOBAL_IP_MAPPING;
    let cmdb_id_ip_config: &DashMap<String, String> = &*GLOBAL_CMDB_ID_IP_MAP;
    let cmdb_id = item.id.clone();
    match cmdb_id_ip_config.get(cmdb_id.as_str()) {
        Some(ip_conf) => {
            let ip = (*ip_conf).clone();
            if is_del {
                info!("remove ip mapping:{}", ip);
                ip_mapping_config.remove(ip.as_str());
            } else {
                info!("update ip mapping:{}-{:?}", ip, item);
                let vec = match ip_mapping_config.get(ip.as_str()) {
                    Some(config) => (*config).clone(),
                    None => {
                        vec![item]
                    }
                };
                ip_mapping_config.insert(ip, vec);
            }
        }
        None => {
            if item.primary_ip.is_some() {
                let ip = item.primary_ip.as_ref().unwrap();
                info!("add ip mapping:{}-{:?}", ip, item);
                ip_mapping_config.insert(ip.to_string(), vec![item]);
            };
        }
    }
}

async fn load_remote_ip_mapping(url: &str) {
    match get(url).await {
        Ok(context) => {
            info!("load ip mapping conf");
            match parse_conf(context) {
                Ok(conf) => {
                    update_ip_mapping_config(conf);
                }
                Err(e) => {
                    error!("ip mapping config parse error.{}", e);
                }
            }
        }
        Err(e) => error!("get ip mapping config error. {}", e),
    }
}

fn parse_conf(context: String) -> serde_json::Result<HashMap<String, Vec<IpMappingItem>>> {
    let response: IpMappingResponse = serde_json::from_str(context.as_str())?;
    Ok(response.result)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct IpMappingResponse {
    code: i8,
    result: HashMap<String, Vec<IpMappingItem>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IpMappingItem {
    pub id: String,
    #[serde(rename = "primaryIp")]
    pub primary_ip: Option<String>,
    #[serde(rename = "otherIp")]
    pub other_ip: Option<String>,
    #[serde(rename = "appUk")]
    pub app_uk: Option<String>,
    #[serde(rename = "groupEnvironment")]
    pub group_environment: Option<String>,
    #[serde(rename = "logicIdcUk")]
    pub logical_idc_uk: Option<String>,
    #[serde(rename = "areaUk")]
    pub area_uk: Option<String>,
    #[serde(rename = "port")]
    pub port: Option<String>,
}

impl IpMappingItem {
    pub fn new() -> Self {
        IpMappingItem {
            id: String::new(),
            primary_ip: None,
            other_ip: None,
            app_uk: None,
            group_environment: None,
            logical_idc_uk: None,
            area_uk: None,
            port: None,
        }
    }
}
