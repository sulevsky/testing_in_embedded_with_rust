#![no_std]
#![no_main]

use application::SmaBuffer;
use bme280_driver::Baro;
use bme280_driver::baro::BARO_DEVICE_ADDR_SDO_VCC;
use defmt::info;

use defmt_rtt as _;
use panic_probe as _;
use stm32f4xx_hal::i2c;
use stm32f4xx_hal::{gpio::GpioExt, prelude::*};

#[cortex_m_rt::entry]
fn main() -> ! {
    info!("Starting initialization");

    let cp = cortex_m::Peripherals::take().expect("Could not initialize core peripherals");
    let dp =
        stm32f4xx_hal::pac::Peripherals::take().expect("Could not initialize device peripherals");
    let mut rcc = dp.RCC.constrain();
    let mut delay = cp.SYST.delay(&rcc.clocks);

    let mut sma_buffer: SmaBuffer<16> = SmaBuffer::new();

    let gpiob = dp.GPIOB.split(&mut rcc);
    let i2c1_scl = gpiob.pb6.into_alternate_open_drain::<4>();
    let i2c1_sda = gpiob.pb7.into_alternate_open_drain::<4>();

    let mut i2c = stm32f4xx_hal::i2c::I2c::new(
        dp.I2C1,
        (i2c1_scl, i2c1_sda),
        i2c::Mode::standard(100.kHz()),
        &mut rcc,
    );
    let bme280 = Baro::new(BARO_DEVICE_ADDR_SDO_VCC)
        .calibrated(&mut i2c)
        .expect("Could not calibrate baro");
    info!("Finished initialization");

    info!("Running...");
    loop {
        let sensor_data = bme280
            .read_sensor(&mut i2c)
            .expect("Could not read baro sensor");
        sma_buffer.push(sensor_data.temperature);
        info!("Temperature is {}, C", sma_buffer.average() as f32 / 100.0,);
        delay.delay_ms(1000);
    }
}
