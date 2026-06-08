pub const BARO_DEVICE_ADDR_SDO_GND: u8 = 0x76;
pub const BARO_DEVICE_ADDR_SDO_VCC: u8 = 0x77;

const ID_ADDR: u8 = 0xD0;
const CALIBRATION_ADDR: u8 = 0x88;
const PRESS_ADDR: u8 = 0xF7;
const CTRL_MEAS_ADDR: u8 = 0xF4;

pub struct Uncalibrated;
pub struct Calibrated {
    calibration_data: CalibrationData,
}

#[derive(Debug)]
pub struct CalibrationData {
    t1: u16,
    t2: i16,
    t3: i16,
    p1: u16,
    p2: i16,
    p3: i16,
    p4: i16,
    p5: i16,
    p6: i16,
    p7: i16,
    p8: i16,
    p9: i16,
}
pub struct SensorData {
    pub temperature: i32,
    pub pressure: i32,
}

pub struct Baro<State> {
    calibration_state: State,
    i2c_device_address: u8,
}

impl<State> Baro<State> {
    pub fn read_id<T: embedded_hal::i2c::I2c>(&self, i2c: &mut T) -> Result<u8, T::Error> {
        let mut buf = [0u8; 1];
        i2c.write_read(self.i2c_device_address, &[ID_ADDR], &mut buf)?;
        Ok(buf[0])
    }
}

impl Baro<Uncalibrated> {
    pub fn new(i2c_device_address: u8) -> Self {
        Baro {
            i2c_device_address,
            calibration_state: Uncalibrated,
        }
    }

    pub fn calibrated<T: embedded_hal::i2c::I2c>(
        self,
        i2c: &mut T,
    ) -> Result<Baro<Calibrated>, T::Error> {
        let calibration_data = self.read_calibration_data(i2c)?;
        // continuous measurement
        // 0xF4: osrs_t=X1, osrs_p=X1, mode=normal (11)
        // 0b001_001_11 = 0x27
        i2c.write(self.i2c_device_address, &[CTRL_MEAS_ADDR, 0x27])?;

        Ok(Baro {
            i2c_device_address: self.i2c_device_address,
            calibration_state: Calibrated { calibration_data },
        })
    }
    fn to_calibration_data(arr: &[u8; 24]) -> CalibrationData {
        CalibrationData {
            t1: u16::from_le_bytes(arr[0..2].try_into().unwrap()),
            t2: i16::from_le_bytes(arr[2..4].try_into().unwrap()),
            t3: i16::from_le_bytes(arr[4..6].try_into().unwrap()),
            p1: u16::from_le_bytes(arr[6..8].try_into().unwrap()),
            p2: i16::from_le_bytes(arr[8..10].try_into().unwrap()),
            p3: i16::from_le_bytes(arr[10..12].try_into().unwrap()),
            p4: i16::from_le_bytes(arr[12..14].try_into().unwrap()),
            p5: i16::from_le_bytes(arr[14..16].try_into().unwrap()),
            p6: i16::from_le_bytes(arr[16..18].try_into().unwrap()),
            p7: i16::from_le_bytes(arr[18..20].try_into().unwrap()),
            p8: i16::from_le_bytes(arr[20..22].try_into().unwrap()),
            p9: i16::from_le_bytes(arr[22..24].try_into().unwrap()),
        }
    }

    fn read_calibration_data<T: embedded_hal::i2c::I2c>(
        &self,
        i2c: &mut T,
    ) -> Result<CalibrationData, T::Error> {
        let mut buf = [0u8; 24];
        i2c.write_read(self.i2c_device_address, &[CALIBRATION_ADDR], &mut buf)?;
        Ok(Self::to_calibration_data(&buf))
    }
}

impl Baro<Calibrated> {
    pub fn read_sensor<T: embedded_hal::i2c::I2c>(
        &self,
        i2c: &mut T,
    ) -> Result<SensorData, T::Error> {
        let calibration_data = &self.calibration_state.calibration_data;
        let mut buf = [0u8; 6];
        i2c.write_read(self.i2c_device_address, &[PRESS_ADDR], &mut buf)?;
        let raw_press = ((buf[0] as u32) << 12) | ((buf[1] as u32) << 4) | (buf[2] as u32) >> 4;
        let raw_temp = ((buf[3] as u32) << 12) | ((buf[4] as u32) << 4) | (buf[5] as u32) >> 4;

        let var1 = ((((raw_temp as i32) >> 3) - ((calibration_data.t1 as i32) << 1))
            * (calibration_data.t2 as i32))
            >> 11;
        let var2 = ((((((raw_temp as i32) >> 4) - (calibration_data.t1 as i32))
            * (((raw_temp as i32) >> 4) - (calibration_data.t1 as i32)))
            >> 12)
            * (calibration_data.t3 as i32))
            >> 14;
        let t_fine = var1 + var2;
        let temperature = (t_fine * 5 + 128) >> 8;

        let var1 = (t_fine as i64) - 128000;
        let var2 = var1 * var1 * (calibration_data.p6 as i64);
        let var2 = var2 + ((var1 * (calibration_data.p5 as i64)) << 17);
        let var2 = var2 + ((calibration_data.p4 as i64) << 35);
        let var1 = ((var1 * var1 * (calibration_data.p3 as i64)) >> 8)
            + ((var1 * (calibration_data.p2 as i64)) << 12);
        let var1 = (((1i64 << 47) + var1) * (calibration_data.p1 as i64)) >> 33;

        if var1 == 0 {
            return Ok(SensorData {
                temperature,
                pressure: 0,
            });
        }

        let p = 1048576i64 - raw_press as i64;
        let p = (((p << 31) - var2) * 3125) / var1;
        let var1 = ((calibration_data.p9 as i64) * (p >> 13) * (p >> 13)) >> 25;
        let var2 = ((calibration_data.p8 as i64) * p) >> 19;
        let p = ((p + var1 + var2) >> 8) + ((calibration_data.p7 as i64) << 4);

        let pressure = (p >> 8) as i32;

        Ok(SensorData {
            temperature,
            pressure,
        })
    }
}

#[cfg(test)]
mod tests {

    extern crate std;
    use embedded_hal_mock::eh1::i2c::{self, Transaction};

    use std::prelude::v1::*;

    use super::*;
    #[test]
    fn read_id() {
        let expected = vec![Transaction::write_read(0x77, vec![0xD0], vec![0x42])];
        let mut i2c = i2c::Mock::new(&expected);
        let baro = Baro::new(BARO_DEVICE_ADDR_SDO_VCC);
        let id = baro.read_id(&mut i2c).unwrap();
        i2c.done();
        assert_eq!(id, 0x42);
    }

    #[test]
    fn calibrated() {
        let expected = vec![
            Transaction::write_read(0x77, vec![0x88], (1..=24).into_iter().collect()),
            Transaction::write(0x77, vec![0xF4, 0x27]),
        ];

        let mut i2c_mock = i2c::Mock::new(&expected);
        let baro = Baro::new(BARO_DEVICE_ADDR_SDO_VCC)
            .calibrated(&mut i2c_mock)
            .unwrap();
        i2c_mock.done();
        let calibration_data = baro.calibration_state.calibration_data;
        assert_eq!(calibration_data.t1, 0b0000_0010_0000_0001 as u16); // 2 << 8 | 1 
        assert_eq!(calibration_data.t2, 0b0000_0100_0000_0011 as i16); // 4 << 8 | 3
    }

    #[test]
    fn read_sensor_with_empty_values() {
        let expected = vec![
            Transaction::write_read(0x77, vec![0x88], vec![0; 24]),
            Transaction::write(0x77, vec![0xF4, 0x27]),
            Transaction::write_read(0x77, vec![0xF7], vec![0; 6]),
        ];
        let mut i2c_mock = i2c::Mock::new(&expected);
        let baro = Baro::new(BARO_DEVICE_ADDR_SDO_VCC)
            .calibrated(&mut i2c_mock)
            .unwrap();
        let sensor_data = baro.read_sensor(&mut i2c_mock).unwrap();
        i2c_mock.done();
        assert_eq!(sensor_data.pressure, 0);
        assert_eq!(sensor_data.temperature, 0);
    }
}
