use std::{
    ops::{Add, AddAssign, Div, Neg, RangeInclusive},
    time::Instant,
};

use anyhow::{Context, Result};
use rppal::i2c::I2c;

use crate::{math::Vec3D, utilites};

#[allow(non_upper_case_globals)]
const g: f64 = 9.80665; // [m/s^2] | Don't know which value of g the sensor has been calibrated with, so I'm using standard gravity: https://en.wikipedia.org/wiki/Gravity_of_Earth

#[derive(Debug, serde::Serialize, Default, Clone, Copy)]
pub struct SensorSample<V, T> {
    acceleration: V,
    angular_velocity: V,
    temperature: T,
}

impl<V: Add<Output = V>, T: Add<Output = T>> Add<SensorSample<V, T>> for SensorSample<V, T> {
    type Output = Self;

    fn add(self, rhs: SensorSample<V, T>) -> Self::Output {
        Self::new(
            self.acceleration + rhs.acceleration,
            self.angular_velocity + rhs.angular_velocity,
            self.temperature + rhs.temperature,
        )
    }
}

impl<V: AddAssign, T: AddAssign> AddAssign<SensorSample<V, T>> for SensorSample<V, T> {
    fn add_assign(&mut self, rhs: SensorSample<V, T>) {
        self.acceleration += rhs.acceleration;
        self.angular_velocity += rhs.angular_velocity;
        self.temperature += rhs.temperature;
    }
}

impl<V: Neg<Output = V>, T: Neg<Output = T>> Neg for SensorSample<V, T> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(
            -self.acceleration,
            -self.angular_velocity,
            -self.temperature,
        )
    }
}

impl<R: Into<f64> + Copy, V: Div<f64, Output = V>, T: Div<f64, Output = T>> Div<R>
    for SensorSample<V, T>
{
    type Output = Self;

    fn div(self, rhs: R) -> Self::Output {
        Self::new(
            self.acceleration / rhs.into(),
            self.angular_velocity / rhs.into(),
            self.temperature / rhs.into(),
        )
    }
}

impl<V, T> SensorSample<V, T> {
    pub fn new(acceleration: V, angular_velocity: V, temperature: T) -> Self {
        Self {
            acceleration,
            angular_velocity,
            temperature,
        }
    }
}

#[derive(Default)]
pub struct InterruptStatus {
    pub fifo_buffer_overflow: bool, // true: FIFO buffer overflow has generated interrupt
    pub i2c_master_interrupt: bool, // true: I2C Master interrupt source has generated interrupt
    pub data_ready: bool, // true: Data ready interrupt (occurs when a write operation to all sensor registers has been completed) has caused interrupt
}

pub struct InterruptConfiguration {
    pub level: bool,                // false: Active high | true: Active low
    pub open: bool,                 // false: Push-pull | true: open drain
    pub launch: bool,               // fales: 50us pulse | true: High until clear
    pub clear: bool, // false: Status cleared only by reading register 58 | true: Status cleared by any read operation
    pub fsync_level: bool, // false: Active high | true: Active low
    pub fsync_interrupt: bool, // false: FSYNC interrupts disabled | true: FSYNC interrupts enabled
    pub i2c_bypass: bool, // false: Bypass disabled | true (+register 106 bit 5 = 0): Bypass enabled
    pub fifo_buffer_overflow: bool, // true: Enables FIFO buffer overflow to generate interrupt
    pub i2c_master_interrupt: bool, // true: Enables I2C Master interrupt sources to generate interrupts
    pub data_ready: bool, // true: Enables data ready interrupt (occurs when a write operation to all sensor registers has been completed)
    pub interrupt_pin: Option<rppal::gpio::InputPin>,
}

#[allow(clippy::derivable_impls)]
impl Default for InterruptConfiguration {
    fn default() -> Self {
        Self {
            level: false,
            open: false,
            launch: false,
            clear: false,
            fsync_level: false,
            fsync_interrupt: false,
            i2c_bypass: false,
            fifo_buffer_overflow: false,
            i2c_master_interrupt: false,
            data_ready: false,
            interrupt_pin: None,
        }
    }
}

#[derive(Clone, Copy)]
pub enum ExternalFrameSynchronization {
    InputDisabled = 0,
    TempLow,
    GyroXLow,
    GyroYLow,
    GyroZLow,
    AccelXLow,
    AccelYLow,
    AccelZLow,
}

impl Default for ExternalFrameSynchronization {
    fn default() -> Self {
        Self::InputDisabled
    }
}

#[derive(Clone, Copy)]
pub enum Filter {
    // Configuration of Digital Low Pass Filter (DLPF) (register 25)
    Disabled = 0,           // DLPF disabled. Gyroscope Output Rate: 8kHz
    BwAc260HzBwGy256Hz = 1, // DLPF enabled. Gyroscope Output Rate: 1kHz
    BwAc184HzBwGy188Hz = 2, // DLPF enabled. Gyroscope Output Rate: 1kHz
    BwAc94HzBwGy98Hz = 3,   // DLPF enabled. Gyroscope Output Rate: 1kHz
    BwAc44HzBwGy42Hz = 4,   // DLPF enabled. Gyroscope Output Rate: 1kHz
    BwAc21HzBwGy20Hz = 5,   // DLPF enabled. Gyroscope Output Rate: 1kHz
    BwAc10HzBwGy10Hz = 6,   // DLPF enabled. Gyroscope Output Rate: 1kHz
    BwAc5HzBwGy5Hz = 7,     // DLPF disabled. Gyroscope Output Rate: 8kHz
}

impl Default for Filter {
    fn default() -> Self {
        Self::Disabled
    }
}

#[derive(Default)]
pub struct Configuration {
    pub external_frame_synchronization: ExternalFrameSynchronization,
    pub filter: Filter,
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub enum WakeFrequency {
    Freq1_25Hz = 0,
    Freq5Hz = 1,
    Freq20Hz = 2,
    Freq40Hz = 3,
}

#[allow(dead_code)]
pub enum PowerMode {
    Active,
    Cycle(WakeFrequency),
    Reset,
    Sleep,
}

impl Default for PowerMode {
    fn default() -> Self {
        Self::Active
    }
}

#[derive(Clone, Copy)]
pub enum ClockSource {
    InternalOscillator8MHz = 0,
    GyroX = 1,
    GyroY = 2,
    GyroZ = 3,
    External33kHz = 4,
    External19MHz = 5,
    Stop = 7,
}

impl Default for ClockSource {
    fn default() -> Self {
        Self::InternalOscillator8MHz
    }
}

pub struct PowerSettings {
    pub mode: PowerMode,
    pub clock_source: ClockSource,
    pub accelerometer_x_active: bool,
    pub accelerometer_y_active: bool,
    pub accelerometer_z_active: bool,
    pub gyroscope_x_active: bool,
    pub gyroscope_y_active: bool,
    pub gyroscope_z_active: bool,
    pub thermometer_active: bool,
}

impl Default for PowerSettings {
    fn default() -> Self {
        Self {
            mode: Default::default(),
            clock_source: Default::default(),
            accelerometer_x_active: true,
            accelerometer_y_active: true,
            accelerometer_z_active: true,
            gyroscope_x_active: true,
            gyroscope_y_active: true,
            gyroscope_z_active: true,
            thermometer_active: true,
        }
    }
}

struct Register {
    address: u8,
    value: u8,
}

impl Register {
    fn new(address: u8, value: u8) -> Self {
        Self { address, value }
    }
}

pub struct SettingsRegisters {
    pwr_mgmt_1: Register,
    pwr_mgmt_2: Register,
    int_pin_cfg: Register,
    int_enable: Register,
    int_status: Register,
    config: Register, // Filter configuration
}

impl SettingsRegisters {
    fn new(
        pwr_mgmt_1: Register,
        pwr_mgmt_2: Register,
        int_pin_cfg: Register,
        int_enable: Register,
        int_status: Register,
        config: Register,
    ) -> Self {
        Self {
            pwr_mgmt_1,
            pwr_mgmt_2,
            int_pin_cfg,
            int_enable,
            int_status,
            config,
        }
    }
}

impl Default for SettingsRegisters {
    fn default() -> Self {
        SettingsRegisters::new(
            Register::new(0x6B, 0),
            Register::new(0x6C, 0),
            Register::new(0x37, 0),
            Register::new(0x38, 0),
            Register::new(0x3A, 0),
            Register::new(0x1A, 0),
        )
    }
}

pub struct DataRegisters {
    accelerometer: RangeInclusive<u8>,
    thermometer: RangeInclusive<u8>,
    gyroscope: RangeInclusive<u8>,
    data_range: RangeInclusive<u8>,
}

impl DataRegisters {
    fn new(
        accelerometer: RangeInclusive<u8>,
        thermometer: RangeInclusive<u8>,
        gyroscope: RangeInclusive<u8>,
    ) -> Result<Self> {
        (*accelerometer.end() == *thermometer.start() - 1
            && *thermometer.end() == *gyroscope.start() - 1)
            .then_some(())
            .context("Register range expected to be continuous.")?;
        let data_range = *accelerometer
            .start()
            .min(thermometer.start().min(gyroscope.start()))
            ..=*accelerometer
                .end()
                .max(thermometer.end().max(gyroscope.end()));

        Ok(Self {
            accelerometer: *accelerometer.start() - *data_range.start()
                ..=*accelerometer.end() - *data_range.start(),
            thermometer: *thermometer.start() - *data_range.start()
                ..=*thermometer.end() - *data_range.start(),
            gyroscope: *gyroscope.start() - *data_range.start()
                ..=*gyroscope.end() - *data_range.start(),
            data_range,
        })
    }
}

impl Default for DataRegisters {
    fn default() -> Self {
        DataRegisters::new(0x3B..=0x40, 0x41..=0x42, 0x43..=0x48)
            .expect("Unable to create data registers.")
    }
}

pub struct GyroscopeConfiguration {
    #[allow(dead_code)]
    range: RangeInclusive<isize>, // Full-Scale Range [degree/s]
    scale_factor: f64,         // Sensitivity Scale Factor [LSB/(degree/s)]
    output_rate: f64,          // [Hz]
    calibration_offset: Vec3D, // [egree/s]
}

#[allow(dead_code)]
impl GyroscopeConfiguration {
    pub const A: Self = Self {
        range: -250..=250,
        scale_factor: 131.0,
        output_rate: 1e3,
        calibration_offset: Vec3D {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
    };
    pub const B: Self = Self {
        range: -500..=500,
        scale_factor: 65.5,
        output_rate: 1e3,
        calibration_offset: Vec3D {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
    };
    pub const C: Self = Self {
        range: -1000..=1000,
        scale_factor: 32.8,
        output_rate: 1e3,
        calibration_offset: Vec3D {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
    };
    pub const D: Self = Self {
        range: -2000..=2000,
        scale_factor: 16.4,
        output_rate: 1e3,
        calibration_offset: Vec3D {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
    };
}

impl Default for GyroscopeConfiguration {
    fn default() -> Self {
        Self::A
    }
}

pub struct AccelerometerConfiguration {
    #[allow(dead_code)]
    range: RangeInclusive<isize>, // Full-Scale Range [g]
    scale_factor: usize,       // Sensitivity Scale Factor [LSB/g]
    output_rate: f64,          // [Hz]
    calibration_offset: Vec3D, // [g]
}

#[allow(dead_code)]
impl AccelerometerConfiguration {
    pub const A: Self = Self {
        range: -2..=2,
        scale_factor: 16_384,
        output_rate: 1e3,
        calibration_offset: Vec3D {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
    };
    pub const B: Self = Self {
        range: -4..=4,
        scale_factor: 8_192,
        output_rate: 1e3,
        calibration_offset: Vec3D {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
    };
    pub const C: Self = Self {
        range: -8..=8,
        scale_factor: 4_096,
        output_rate: 1e3,
        calibration_offset: Vec3D {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
    };
    pub const D: Self = Self {
        range: -16..=16,
        scale_factor: 2_048,
        output_rate: 1e3,
        calibration_offset: Vec3D {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
    };
}

impl Default for AccelerometerConfiguration {
    fn default() -> Self {
        Self::A
    }
}

pub struct ThermometerConfiguration {
    #[allow(dead_code)]
    range: RangeInclusive<isize>, // [degree C]
    #[allow(dead_code)]
    sensitivity: usize, // [LSB/(degree C)]
    #[allow(dead_code)]
    offset: isize, // [LSB]
    offset_celcius: f64,     // [degree C]
    calibration_offset: f64, // [degree C]
}

impl Default for ThermometerConfiguration {
    fn default() -> Self {
        Self {
            range: -40..=85,
            sensitivity: 340,
            offset: -521,
            offset_celcius: 36.53, // See section 4.18 in revision 4.2 of register map
            calibration_offset: 0.0,
        }
    }
}

// Not splitting up into individual sensors for gyroscope and accelerometer, since data needs to be read in one go (burst reading) for all sensors, to ensure that data is from the same sampling instance. See: https://stackoverflow.com/questions/65117246/mpu-6050-burst-read-auto-increment
#[non_exhaustive]
pub struct GY521 {
    pub acceleration: Vec3D,
    pub angular_velocity: Vec3D,
    pub temperature: f64,
    pub data_registers: DataRegisters,
    pub settings_registers: SettingsRegisters,
    pub power_settings: PowerSettings,
    pub i2c_address: u16,
    pub i2c_data_access_rate: f64, // [Hz]
    pub gyroscope_configuration: GyroscopeConfiguration,
    pub accelerometer_configuration: AccelerometerConfiguration,
    pub thermometer_configuration: ThermometerConfiguration,
    // pub gyroscope_output_rate: f64,     // [Hz]
    // pub accelerometer_output_rate: f64, // [Hz]
    pub configuration: Configuration, // Register 26
    pub sample_rate_divider: u8, // Register 25: Used for determining sample rate: How often sensor samples should be output to the data registers, FIFO, or DMP. With a sample rate above the accelerometer output rate, the same accelerometer data will be output multiple times
    pub sample_rate: f64,        // [Hz]
    pub interrupt_configuration: InterruptConfiguration,
}

impl GY521 {
    pub fn new(
        data_registers: DataRegisters,
        settings_registers: SettingsRegisters,
        power_settings: PowerSettings,
        i2c_address: u16,
        i2c_data_access_rate: f64,
        mut gyroscope_configuration: GyroscopeConfiguration,
        accelerometer_configuration: AccelerometerConfiguration,
        thermometer_configuration: ThermometerConfiguration,
        configuration: Configuration,
        sample_rate_divider: u8,
        interrupt_configuration: InterruptConfiguration,
    ) -> Self {
        gyroscope_configuration.output_rate = match configuration.filter {
            Filter::BwAc260HzBwGy256Hz | Filter::BwAc5HzBwGy5Hz => 8e3,
            _ => 1e3,
        };

        let sample_rate = gyroscope_configuration.output_rate / (1.0 + sample_rate_divider as f64);

        Self {
            data_registers,
            settings_registers,
            power_settings,
            i2c_address,
            i2c_data_access_rate,
            gyroscope_configuration,
            accelerometer_configuration,
            thermometer_configuration,
            configuration,
            sample_rate_divider,
            sample_rate,
            interrupt_configuration,
            acceleration: Default::default(),
            angular_velocity: Default::default(),
            temperature: Default::default(),
        }
    }

    // Raw acceleration, temperature, and angular velocity readings shifted to be signed integer values
    fn read_raw(&self, i2c: &I2c) -> Result<SensorSample<[i16; 3], i16>> {
        fn concat_bytes(low: u8, high: u8) -> u16 {
            low as u16 | ((high as u16) << 8)
        }

        fn shift_to_signed(value: u16) -> i16 {
            if value >= 0x8000 {
                -((0xFFFF - value) as i16 + 1)
            } else {
                value as i16
            }
        }

        let mut data = vec![0u8; self.data_registers.data_range.len()];
        i2c.block_read(*self.data_registers.data_range.start(), &mut data)?;

        let acceleration = &data[*self.data_registers.accelerometer.start() as usize
            ..=*self.data_registers.accelerometer.end() as usize];
        let acceleration = [
            shift_to_signed(concat_bytes(acceleration[1], acceleration[0])),
            shift_to_signed(concat_bytes(acceleration[3], acceleration[2])),
            shift_to_signed(concat_bytes(acceleration[5], acceleration[4])),
        ];

        let temperature = &data[*self.data_registers.thermometer.start() as usize
            ..=*self.data_registers.thermometer.end() as usize];
        let temperature = shift_to_signed(concat_bytes(temperature[1], temperature[0]));

        let angular_velocity = &data[*self.data_registers.gyroscope.start() as usize
            ..=*self.data_registers.gyroscope.end() as usize];
        let angular_velocity = [
            shift_to_signed(concat_bytes(angular_velocity[1], angular_velocity[0])),
            shift_to_signed(concat_bytes(angular_velocity[3], angular_velocity[2])),
            shift_to_signed(concat_bytes(angular_velocity[5], angular_velocity[4])),
        ];

        Ok(SensorSample::new(
            acceleration,
            angular_velocity,
            temperature,
        ))
    }

    // Reads (acceleration, temperature, angular_velocity)
    pub fn read(&mut self, i2c: &I2c) -> Result<SensorSample<Vec3D, f64>> {
        let sample = self.read_raw(i2c)?;
        let acceleration = Vec3D::new(
            sample.acceleration[0],
            sample.acceleration[1],
            sample.acceleration[2],
        );
        self.acceleration = acceleration / self.accelerometer_configuration.scale_factor as f64
            + self.accelerometer_configuration.calibration_offset;

        let angular_velocity = Vec3D::new(
            sample.angular_velocity[0],
            sample.angular_velocity[1],
            sample.angular_velocity[2],
        );
        self.angular_velocity = angular_velocity / self.gyroscope_configuration.scale_factor as f64
            + self.gyroscope_configuration.calibration_offset;

        self.temperature = sample.temperature as f64 / self.thermometer_configuration.sensitivity as f64
            + self.thermometer_configuration.offset_celcius // See section 4.18 in revision 4.2 of register map
            + self.thermometer_configuration.calibration_offset;

        Ok(SensorSample::new(
            self.acceleration,
            self.angular_velocity,
            self.temperature,
        ))
    }

    pub fn initialize(&mut self, i2c: &mut I2c) -> Result<()> {
        i2c.set_slave_address(self.i2c_address)?;

        // Set power settings
        let mut pwr_mgmt_1 = 0u8; // First power management register
        let mut pwr_mgmt_2 = 0u8; // Second power management register
        match self.power_settings.mode {
            PowerMode::Active => pwr_mgmt_1 = 0,
            PowerMode::Cycle(wake_up_frequency) => {
                pwr_mgmt_1 |= 1 << 5;
                pwr_mgmt_2 |= (wake_up_frequency as u8) << 6;
            }
            PowerMode::Reset => pwr_mgmt_1 |= 1 << 7,
            PowerMode::Sleep => pwr_mgmt_1 |= 1 << 6,
        }

        if !self.power_settings.thermometer_active {
            pwr_mgmt_1 |= 1 << 3;
        }

        pwr_mgmt_1 |= self.power_settings.clock_source as u8;

        pwr_mgmt_2 |= (!self.power_settings.accelerometer_x_active as u8) << 5;
        pwr_mgmt_2 |= (!self.power_settings.accelerometer_y_active as u8) << 4;
        pwr_mgmt_2 |= (!self.power_settings.accelerometer_z_active as u8) << 3;
        pwr_mgmt_2 |= (!self.power_settings.gyroscope_x_active as u8) << 2;
        pwr_mgmt_2 |= (!self.power_settings.gyroscope_y_active as u8) << 1;
        #[allow(clippy::identity_op)]
        pwr_mgmt_2 |= (!self.power_settings.gyroscope_z_active as u8) << 0;

        // Updating stored configuration only after successfully sending commands to sensor
        i2c.smbus_write_byte(self.settings_registers.pwr_mgmt_1.address, pwr_mgmt_1)?;
        self.settings_registers.pwr_mgmt_1.value = pwr_mgmt_1;
        i2c.smbus_write_byte(self.settings_registers.pwr_mgmt_2.address, pwr_mgmt_2)?;
        self.settings_registers.pwr_mgmt_2.value = pwr_mgmt_2;

        // Set interrupt settings
        if let Some(interrupt_pin) = &mut self.interrupt_configuration.interrupt_pin {
            interrupt_pin
                .set_interrupt(if self.interrupt_configuration.level {
                    rppal::gpio::Trigger::FallingEdge
                } else {
                    rppal::gpio::Trigger::RisingEdge
                })
                .context("Unable to configure interrupt pin.")?;

            let mut int_pin_cfg = 0u8;
            let mut int_enable = 0u8;

            int_pin_cfg |= (self.interrupt_configuration.level as u8) << 7;
            int_pin_cfg |= (self.interrupt_configuration.open as u8) << 6;
            int_pin_cfg |= (self.interrupt_configuration.launch as u8) << 5;
            int_pin_cfg |= (self.interrupt_configuration.clear as u8) << 4;
            int_pin_cfg |= (self.interrupt_configuration.fsync_level as u8) << 3;
            int_pin_cfg |= (self.interrupt_configuration.fsync_interrupt as u8) << 2;
            int_pin_cfg |= (self.interrupt_configuration.i2c_bypass as u8) << 1;

            int_enable |= (self.interrupt_configuration.fifo_buffer_overflow as u8) << 4;
            int_enable |= (self.interrupt_configuration.i2c_master_interrupt as u8) << 3;
            int_enable |= (self.interrupt_configuration.data_ready as u8) << 0;

            i2c.smbus_write_byte(self.settings_registers.int_pin_cfg.address, int_pin_cfg)?;
            self.settings_registers.int_pin_cfg.value = int_pin_cfg;
            i2c.smbus_write_byte(self.settings_registers.int_enable.address, int_enable)?;
            self.settings_registers.int_enable.value = int_enable;
        }

        // Set filter settings
        let mut config = 0u8;
        config |= (self.configuration.filter as u8) << 0;
        config |= (self.configuration.external_frame_synchronization as u8) << 3;
        i2c.smbus_write_byte(self.settings_registers.config.address, config)?;
        self.settings_registers.config.value = config;

        Ok(())
    }

    /// Calculates calibration coefficients for acceleration and angular velocity based on {sample_size} samples.
    /// Samples for a duration of {sampling_duration}, attempting to sample with a period of {sampling_period}.
    /// Only the last {sample_size} samples are used.
    /// Attempts to execute {status_action} once every {status_period}.
    ///
    /// Gyroscope output is expected to be 0 degrees/s for all axes under steady conditions.
    /// Accelerometer output is exepcted to be 0g for the x-, and y-axes, and 1g for the z-axis.
    /// The thermometer is not calibrated, because that can not be done simply by letting the sensor sit around in peace like for the other sensors.
    pub fn calibrate<F>(
        &mut self,
        sample_size: usize,
        sampling_period: std::time::Duration,
        calibration_duration: std::time::Duration,
        i2c: &mut I2c,
        kill_signal: &crossbeam_channel::Receiver<()>,
        status_period: std::time::Duration,
        mut status_action: F,
    ) where
        F: FnMut(),
    {
        // 1.: Collect data for a while
        let interrupt_timeout = std::time::Duration::from_secs_f64(1.5 / self.sample_rate); // Timeout of more than one sampling period (in case of minor delay?), but less than two sampling periods

        let mut samples = utilites::Memory::new(sample_size);
        let mut errors = utilites::Memory::new(sample_size);

        let mut sample_count = 0;
        let mut status_count = 0;

        let clock = Instant::now();
        loop {
            if kill_signal.try_recv().is_ok() {
                break;
            }

            let (sample, sampling_instant) = self.wait_for_sample(i2c, Some(interrupt_timeout));

            match sample {
                Ok(sample) => {
                    if let Some(sample) = sample {
                        let id = samples.len();
                        if id > 0 {
                            // if sampling_instant.duration_since(samples[id - 1].1) >= sampling_period
                            if sampling_instant.duration_since(clock).as_nanos()
                                >= sample_count * sampling_period.as_nanos()
                            {
                                samples.push((sample, sampling_instant));
                                sample_count += 1;
                            }
                        } else {
                            samples.push((sample, sampling_instant));
                        }
                    }
                }
                Err(error) => {
                    errors.push((error, sampling_instant));
                }
            }

            if clock.elapsed().as_nanos() / status_period.as_nanos() >= status_count {
                status_action();
                status_count += 1;
            }

            if clock.elapsed() >= calibration_duration {
                break; // Let's have a look at the samples
            }
        }

        // 2.: Compute offsets
        let sum = samples.data.iter().fold(
            SensorSample::<Vec3D, f64>::default(),
            |sum, (sample, _time)| sum + *sample,
        );

        let mut offsets = -sum / samples.len() as f64;
        println!("Offsets: {:#?}", offsets);
        offsets.acceleration.z = 1.0 + offsets.acceleration.z; // 1g expected for z acceleration
        self.gyroscope_configuration.calibration_offset += offsets.angular_velocity;
        self.accelerometer_configuration.calibration_offset += offsets.acceleration;
        println!(
            "Gyroscope offsets: {:#?}",
            self.gyroscope_configuration.calibration_offset
        );
        println!(
            "Accelerometer offsets: {:#?}",
            self.accelerometer_configuration.calibration_offset
        );
    }

    /// Set the power settings' clock source.
    pub fn set_clock_source(&mut self, clock_source: ClockSource, i2c: &mut I2c) -> Result<()> {
        let mut pwr_mgmt_1 = self.settings_registers.pwr_mgmt_1.value;
        pwr_mgmt_1 &= u8::MAX << 2; // Reset clock source settings
        pwr_mgmt_1 |= clock_source as u8;
        i2c.smbus_write_byte(self.settings_registers.pwr_mgmt_1.address, pwr_mgmt_1)?;
        self.power_settings.clock_source = clock_source;
        self.settings_registers.pwr_mgmt_1.value = pwr_mgmt_1;
        Ok(())
    }

    pub fn sleep(&mut self, i2c: &mut I2c) -> Result<()> {
        let mut pwr_mgmt_1 = self.settings_registers.pwr_mgmt_1.value;
        pwr_mgmt_1 |= 1 << 6;
        i2c.smbus_write_byte(self.settings_registers.pwr_mgmt_1.address, pwr_mgmt_1)?;
        self.settings_registers.pwr_mgmt_1.value = pwr_mgmt_1;
        Ok(())
    }

    pub fn wait_for_interrupt(
        &mut self,
        i2c: &mut I2c,
        reset: bool,
        timeout: Option<std::time::Duration>,
    ) -> Result<Option<InterruptStatus>> {
        assert!(self.interrupt_configuration.interrupt_pin.is_some());
        let interrupt = self
            .interrupt_configuration
            .interrupt_pin
            .as_mut()
            .unwrap()
            .poll_interrupt(reset, timeout)
            .context("Unable to poll interrupt.")?;

        Ok(match interrupt {
            Some(_) => {
                let interrupt_byte = i2c
                    .smbus_read_byte(self.settings_registers.int_status.address)
                    .context("Unable to read interrupt status.")?;
                Some(InterruptStatus {
                    fifo_buffer_overflow: (interrupt_byte & (1 << 4)) != 0,
                    i2c_master_interrupt: (interrupt_byte & (1 << 3)) != 0,
                    data_ready: (interrupt_byte & (1 << 0)) != 0,
                })
            }
            None => None, // Timeout waiting for interrupt, I think
        })
    }

    pub fn wait_for_sample(
        &mut self,
        i2c: &mut I2c,
        timeout: Option<std::time::Duration>,
    ) -> (Result<Option<SensorSample<Vec3D, f64>>>, Instant) {
        let interrupt = self
            .wait_for_interrupt(i2c, true, timeout)
            .context("Cannot poll for interrupt.");

        match interrupt {
            Ok(interrupt) => match interrupt {
                Some(interrupt_status) if interrupt_status.data_ready => {
                    let sampling_instant = Instant::now();
                    let sample = self.read(i2c).context("Unable to read sensors.");
                    match sample {
                        Ok(sample) => (Ok(Some(sample)), sampling_instant),
                        Err(error) => (Err(error), sampling_instant),
                    }
                }
                _ => (Ok(None), Instant::now()),
            },
            // Occasinally, what seems to be instabillity in the I2C connection, will cause an error. We record the error and try again. Tja, kannste machen nix ??\_(???)_/??
            Err(error) => (Err(error), Instant::now()),
        }
    }
}

impl Default for GY521 {
    fn default() -> Self {
        Self::new(
            Default::default(),
            Default::default(),
            Default::default(),
            0x68, // I2C default slave address
            4e5,
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            0,
            Default::default(),
        )
    }
}
