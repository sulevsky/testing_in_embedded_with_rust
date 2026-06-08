mod tests {
    use application::SmaBuffer;
    use bme280_driver::Baro;
    use embedded_hal_mock::eh1::i2c::{self, Transaction};

    #[test]
    fn filtered_temperature_basic_test() {
        let dummy_i2c_address = 0x42;
        let expected = vec![
            Transaction::write_read(dummy_i2c_address, vec![0x88], vec![0; 24]),
            Transaction::write(dummy_i2c_address, vec![0xF4, 0x27]),
            Transaction::write_read(dummy_i2c_address, vec![0xF7], vec![0; 6]),
        ];
        let mut i2c_mock = i2c::Mock::new(&expected);
        let baro = Baro::new(dummy_i2c_address)
            .calibrated(&mut i2c_mock)
            .unwrap();
        let mut sma_buffer: SmaBuffer<8> = SmaBuffer::new();
        let sensor_data = baro.read_sensor(&mut i2c_mock).unwrap();
        sma_buffer.push(sensor_data.temperature);
        i2c_mock.done();
        assert_eq!(sma_buffer.average(), sensor_data.temperature);
    }
}
