use anyhow::{anyhow, Result};
use esp_idf_svc::nvs::{EspNvs, EspNvsPartition, NvsDefault};
use log::{debug, info};
use postcard::experimental::max_size::MaxSize;
use postcard::{from_bytes, to_vec};
use serde::{Deserialize, Serialize};
use static_assertions::{const_assert, const_assert_eq};
use std::cell::RefCell;
use std::fmt::Debug;
use std::mem;
use std::rc::Rc;



pub struct NvsNamespace {
    pub name: &'static str,
    nvs: EspNvs<NvsDefault>,
}
impl NvsNamespace {
    pub fn access(
        partition: EspNvsPartition<NvsDefault>,
        namespace: &'static str,
        read_only: bool,
    ) -> Result<Self> {
        EspNvs::new(partition, namespace, !read_only)
            .map_err(|e| anyhow!(e).context(format!("Could't get namespace {namespace}")))
            .map(|nvs| {
                info!("Got namespace {namespace} from default partition");
                NvsNamespace {
                    name: namespace,
                    nvs,
                }
            })
    }
    pub fn set_struct<StructType, const SIZE_OF_STRUCT_TYPE: usize>(
        &mut self,
        key: &str,
        value: &StructType,
    ) -> Result<()>
    where
        StructType: Serialize + MaxSize,
    {
        match self
            .nvs
            .set_raw(key, &to_vec::<StructType, SIZE_OF_STRUCT_TYPE>(&value)?)
        {
            Ok(_) => {
                debug!("Key {key} updated for namespace {}", self.name);
                Ok(())
            }
            Err(e) => Err(anyhow!(e).context(format!(
                "key {key} not updated for namespace {} {e:?}",
                self.name
            ))),
        }
    }
    pub fn get_struct<'buff, 'deserialize, StructType>(
        &mut self,
        key: &str,
        buf: &'buff mut [u8],
    ) -> Result<Option<StructType>>
    where
        'buff: 'deserialize,
        StructType: Deserialize<'deserialize> + MaxSize + Debug + Clone,
    {
        let get_raw = self.nvs.get_raw(key, buf);
        let buff = match get_raw {
            Ok(v) => {
                if let Some(the_struct) = v {
                    the_struct
                } else {
                    return Ok(None);
                }
            }
            Err(e) => {
                return Err(anyhow!(e).context(format!(
                    "Couldn't get key {key} from namespace {} because {e:?}",
                    self.name
                )))
            }
        };
        let s = from_bytes::<StructType>(buff)?;
        Ok(Some(s))
    }
}


