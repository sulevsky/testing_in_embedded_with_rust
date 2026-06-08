#![no_std]
#![no_main]

#[embedded_test::tests]
mod tests {
    use bme280_driver::Baro;
    use bme280_driver::baro::BARO_DEVICE_ADDR_SDO_VCC;
    use defmt::assert_eq;
    use defmt_rtt as _;
    use stm32f4xx_hal::{gpio::GpioExt, i2c, prelude::*};

    struct Peripherals {
        core_peripherals: cortex_m::Peripherals,
        device_peripherals: stm32f4xx_hal::pac::Peripherals,
    }

    #[init]
    fn init() -> Peripherals {
        Peripherals {
            core_peripherals: cortex_m::Peripherals::take().unwrap(),
            device_peripherals: stm32f4xx_hal::pac::Peripherals::take().unwrap(),
        }
    }

    #[test]
    fn bme280_connection_test(peripherals: Peripherals) {
        let dp = peripherals.device_peripherals;
        let mut rcc = dp.RCC.constrain();
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
            .unwrap();
        let id = bme280.read_id(&mut i2c).unwrap();
        assert_eq!(id, 0x60);
    }
}
