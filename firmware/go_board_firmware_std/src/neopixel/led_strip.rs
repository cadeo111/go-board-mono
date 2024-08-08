use crate::neopixel::rgb::Rgb;
use anyhow::{anyhow, Result as Result};
use esp_idf_svc::hal::gpio::OutputPin;
use esp_idf_svc::hal::peripheral::Peripheral;
use esp_idf_svc::hal::rmt::{RmtChannel, RmtTransmitConfig};
use esp_idf_svc::sys::{EspError, rmt_config};
// 
// pub type LedStrip<const SIZE:usize> = [Rgb;SIZE];
// 
// 
// pub fn ws2812_set_pixel<const SIZE:usize>(strip: &mut LedStrip<SIZE>, index: usize, rgb: Rgb) -> Result<()> {
//    
//     /*
//     esp_err_t ret = ESP_OK;
//     ws2812_t *ws2812 = __containerof(strip, ws2812_t, parent);
//     STRIP_CHECK(index < ws2812->strip_len, "index out of the maximum number of leds", err, ESP_ERR_INVALID_ARG);
//     uint32_t start = index * 3;
//     // In thr order of GRB
//     ws2812->buffer[start + 0] = green & 0xFF;
//     ws2812->buffer[start + 1] = red & 0xFF;
//     ws2812->buffer[start + 2] = blue & 0xFF;
//     return ESP_OK;
// err:
//     return ret;
//     
//     */
//     if index >= strip.len() {
//          return Err(anyhow!("index out of the maximum number of leds"));
//     }
//     
//     strip[index] = rgb;
//     
//     Ok(())
// }
// pub fn ws2812_refresh<const SIZE:usize>(strip: &mut LedStrip<SIZE>) -> Result<()> {
//    
//     /*
//     esp_err_t ret = ESP_OK;
//     ws2812_t *ws2812 = __containerof(strip, ws2812_t, parent);
//     STRIP_CHECK(rmt_write_sample(ws2812->rmt_channel, ws2812->buffer, ws2812->strip_len * 3, true) == ESP_OK,
//                 "transmit RMT samples failed", err, ESP_FAIL);
//     return rmt_wait_tx_done(ws2812->rmt_channel, pdMS_TO_TICKS(timeout_ms));
// err:
//     return ret;
//     */
//     
//     
//     
//     
//     Ok(())
// }
// 
// pub fn ws2812_init<const SIZE:usize>(led_pin: impl Peripheral<P=impl OutputPin>, channel: impl Peripheral<P: RmtChannel>)->Result<LedStrip<SIZE>>{
//     
//     let led_strip : LedStrip<SIZE> = [Rgb::new(0,0,0); SIZE];
//     
//     
//     
//     
//     
//     /*led_strip_t * led_strip_init(uint8_t channel, uint8_t gpio, uint16_t led_num)
// {
//     static led_strip_t *pStrip;
// 
//     rmt_config_t config = RMT_DEFAULT_CONFIG_TX(gpio, channel);
//     // set counter clock to 40MHz
//     config.clk_div = 2;
// 
//     ESP_ERROR_CHECK(rmt_config(&config));
//     ESP_ERROR_CHECK(rmt_driver_install(config.channel, 0, 0));
// 
//     // install ws2812 driver
//     led_strip_config_t strip_config = LED_STRIP_DEFAULT_CONFIG(led_num, (led_strip_dev_t)config.channel);
// 
//     pStrip = led_strip_new_rmt_ws2812(&strip_config);
// 
//     if ( !pStrip ) {
//         ESP_LOGE(TAG, "install WS2812 driver failed");
//         return NULL;
//     }
// 
//     // Clear LED strip (turn off all LEDs)
//     ESP_ERROR_CHECK(pStrip->clear(pStrip, 100));
// 
//     return pStrip;
// }*/
//     
//     
//     
//     
//     
//     
//     
//     Ok(led_strip)
// }
// 
