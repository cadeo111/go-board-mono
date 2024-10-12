use crate::storage::SaveInNvs;
use anyhow::{Context, Result};
use esp_idf_svc::hal::reset::restart;
use esp_idf_svc::nvs::{EspNvsPartition, NvsDefault};
use log::{debug, info, log};
use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};

/// What to do after a restart (ie go to settings panel)
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone, MaxSize)]
pub enum RecoverOption {
    None = 0,
    ForceSettingsPanel = 1,
}

impl SaveInNvs for RecoverOption {
    fn namespace() -> &'static str {
        "recovery"
    }

    fn key() -> &'static str {
        "option"
    }
    fn get_struct_buffer<'a>() -> impl AsMut<[u8]> {
        [0; Self::POSTCARD_MAX_SIZE]
    }
}

pub fn restart_with_recover_option(
    option: RecoverOption,
    nvs: EspNvsPartition<NvsDefault>,
) -> Result<!> {
    option
        .set_saved_in_nvs(nvs)
        .context(format!("failed adding recover option: {option:?}"))?;
    info!("Restarting with the recover option: {option:?}", );
    restart();
}

pub fn get_and_clear_recover_option(nvs: EspNvsPartition<NvsDefault>) -> Result<RecoverOption> {
    // get the current option
    let option = RecoverOption::get_saved_in_nvs_with_default(nvs.clone(), RecoverOption::None)
        .context("failed to get recover option!")?;
    // clear the current saved option
    RecoverOption::None.set_saved_in_nvs(nvs)?;
    info!("got the recovery option: {option:?}");
    // return the option before clearing
    Ok(option)
}
