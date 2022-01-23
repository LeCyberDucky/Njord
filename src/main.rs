#![feature(duration_constants)]

use std::{
    thread,
    time::{Duration, Instant},
};

use anyhow::Result;
use njord::{gy521, utilites};
use rppal::{gpio::Gpio, i2c::I2c, system::DeviceInfo};

// BCM pin numbering
const GPIO_LED: u8 = 21;
const GPIO_INTERRUPT: u8 = 4;

fn main() -> Result<()> {
    /*********
     * Setup *
     *********/
    let (tx, kill_signal) = crossbeam_channel::unbounded();
    ctrlc::set_handler(move || tx.send(()).expect("Unable to send signal on channel."))
        .expect("Unable to set Ctrl-C handler");

    let mut i2c = I2c::new()?;

    let mut sensor = gy521::GY521::new(
        Default::default(),
        Default::default(),
        gy521::PowerSettings {
            clock_source: gy521::ClockSource::GyroX, // Use gyroscope as clock source for higher accuracy
            ..Default::default()
        },
        0x68,
        4e5,
        Default::default(),
        Default::default(),
        Default::default(),
        gy521::Configuration {
            filter: gy521::Filter::BwAc184HzBwGy188Hz,
            ..Default::default()
        },
        0,
        gy521::InterruptConfiguration {
            // Use pull-up resistor only on one end. Not both on the sensor and the Raspberry pi. See:
            // https://raspberrypi.stackexchange.com/questions/97995/rpi-python-i2c-ioerror-errno-121-remote-i-o-error-problem-how-to-fix-it
            interrupt_pin: Some(Gpio::new()?.get(GPIO_INTERRUPT)?.into_input()),
            data_ready: true,
            open: false, // Something with internal pull-push stuff for the sensor?
            ..Default::default()
        },
    );

    sensor.initialize(&mut i2c)?;
    thread::sleep(Duration::SECOND); // Let stuff start up
    let mut calibration_status_count = 0;
    let calibration_duration = Duration::from_secs(5 * 60);
    let status_period = Duration::from_secs(30);
    sensor.calibrate(
        10_000,
        Duration::from_millis(100),
        calibration_duration,
        &mut i2c,
        &kill_signal,
        status_period,
        || {
            let expected_status_updates =
                calibration_duration.as_micros() / status_period.as_micros();
            println!("Status update: \t{calibration_status_count}/{expected_status_updates}");
            calibration_status_count += 1;
        },
    );

    let mut led = Gpio::new()?.get(GPIO_LED)?.into_output();
    let mut blink_count = 0;
    let blink_period = Duration::from_millis(800);

    let memory_capacity = 5000;
    let mut samples =
        utilites::Memory::<(gy521::SensorSample<njord::math::Vec3D, f64>, Instant)>::new(
            memory_capacity,
        );
    let mut errors = utilites::Memory::new(memory_capacity);

    println!("Blinking an LED on a {}.", DeviceInfo::new()?.model());
    println!("I2C clock frequency: {} Hz", i2c.clock_speed().unwrap());

    let sampling_period = Duration::from_millis(100); // Time between stored samples
    let interrupt_timeout = Duration::from_secs_f64(1.5 / sensor.sample_rate); // Timeout of more than one sampling period (in case of minor delay?), but less than two sampling periods
    let mut sample_count = 0;
    let sampling_begin = std::time::SystemTime::now();
    let clock = Instant::now();
    loop {
        if kill_signal.try_recv().is_ok() {
            break;
        }

        let (sample, sampling_instant) = sensor.wait_for_sample(&mut i2c, Some(interrupt_timeout));

        match sample {
            Ok(sample) => {
                if let Some(sample) = sample {
                    let id = samples.len();
                    if id > 0 {
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

        if (clock.elapsed().as_micros() as u128 / blink_period.as_micros()) > blink_count {
            led.toggle();
            blink_count += 1;
            println!(
                "Samples: {} | Elapsed time: {}",
                samples.count(),
                clock.elapsed().as_micros(),
            );
        }

        if samples.len() == memory_capacity {
            break; // Let's have a look at the samples
        }
    }

    led.set_low();
    sensor.sleep(&mut i2c)?;

    println!("Writing data.");

    let data_file = std::fs::File::create("Data/Calibrated data.yaml")?;
    serde_yaml::to_writer(
        data_file,
        &samples
            .data
            .iter()
            .map(|(sample, instant)| (sample, sampling_begin + instant.duration_since(clock)))
            .collect::<Vec<_>>(),
    )?;
    let error_file = std::fs::File::create("Data/Calibrated errors.yaml")?;
    serde_yaml::to_writer(
        error_file,
        &errors
            .data
            .iter()
            .map(|(error, instant)| {
                (
                    error.to_string(),
                    sampling_begin + instant.duration_since(clock),
                )
            })
            .collect::<Vec<_>>(),
    )?;

    println!("Errors encountered: {}", errors.len());

    Ok(())
}
