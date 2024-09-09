use crate::wifi::WifiCredentials;
use anyhow::{anyhow, Result};
use esp_idf_svc::nvs::{EspNvs, EspNvsPartition, NvsDefault};
use log::{debug, info};
use postcard::experimental::max_size::MaxSize;
use postcard::{from_bytes, to_vec};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use static_assertions::{const_assert, const_assert_eq};
use std::cell::RefCell;
use std::fmt::Debug;
use std::mem;
use std::ops::DerefMut;
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
    pub fn set_struct<StructType>(
        &mut self,
        key: &str,
        value: &StructType,
        buffer: &mut [u8],
    ) -> Result<()>
    where
        StructType: Serialize,
    {
        let buff = postcard::to_slice(&value, buffer)?;
        match self.nvs.set_raw(key, buff) {
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
    pub fn get_struct<'buff, StructType>(
        &mut self,
        key: &str,
        buffer: &mut [u8],
    ) -> Result<Option<StructType>>
    where
        StructType: DeserializeOwned + Debug + Clone,
    {
        let get_raw = self.nvs.get_raw(key, buffer);
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

pub trait SaveInNvs: Sized + Serialize + DeserializeOwned + Debug + Clone + MaxSize {
    fn namespace() -> &'static str;
    fn key() -> &'static str;
     
    fn get_struct_buffer<'a>() ->impl AsMut<[u8]>;
    

    fn get_saved_in_nvs(partition: EspNvsPartition<NvsDefault>) -> Result<Option<Self>> {
        let mut nvs = NvsNamespace::access(partition, Self::namespace(), false)?;
        
        let mut struct_buffer = Self::get_struct_buffer();
        let s = nvs.get_struct::<Self>(Self::key(), struct_buffer.as_mut())?;
        Ok(s)
    }
    fn get_saved_in_nvs_with_default(
        partition: EspNvsPartition<NvsDefault>,
        default: Self,
    ) -> Result<Self> {
        let value = Self::get_saved_in_nvs(partition)?;
        Ok(value.unwrap_or_else(|| {
            debug!(
                "falling back to default for nvs access for type {}",
                std::any::type_name::<Self>()
            );
            default
        }))
    }
    fn set_saved_in_nvs(&self, partition: EspNvsPartition<NvsDefault>) -> Result<()> {
        let mut nvs = NvsNamespace::access(partition, Self::namespace(), false)?;
        let mut struct_buffer = Self::get_struct_buffer();
        nvs.set_struct::<Self>(Self::key(), self, struct_buffer.as_mut())
    }
}
