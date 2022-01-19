# Name:
[Nordic god of seafaring](https://en.wikipedia.org/wiki/Nj%C3%B6r%C3%B0r)

# Compiling and running:
https://dev.to/h_ajsf/cross-compiling-rust-for-raspberry-pi-4iai

1) Compile and copy executable from Windows to Raspberry Pi

In a terminal window on windows do:

`cls && cargo +nightly build && scp C:\Users\<CARGO_TARGET_DIR>\armv7-unknown-linux-gnueabihf\debug\njord <PI_USERNAME>@raspberrypi.local:/home/<PROJECT_PATH>` 

End programs using `Ctrl + C`. If you use `Ctrl + Z`, it will only be suspended, with the process still being alive in the background. Then you can't overwrite it with `scp`. Use `ps` to see active processes, and `kill -SIGKILL process_ID` to kill a process, so the file can be overwritten again. 

2) On the Raspberry Pi (via ssh) do: 

`chmod +x njord`

`./njord`

# Copying data from the Pi to Windows: 
`cls && scp <PI_USERNAME>@raspberrypi.local:/home/<PROJECT_PATH>/Data/Data.yaml "C:\Users\<USERNAME>\Desktop"`

# Hardware
## Raspberry Pi pinout
https://pinout.xyz/
https://www.etechnophiles.com/raspberry-pi-3-b-pinout-with-gpio-functions-schematic-and-specs-in-detail/

# Software
## Gy-521 calibration 

- https://www.fierceelectronics.com/components/compensating-for-tilt-hard-iron-and-soft-iron-effects
- https://thecavepearlproject.org/2015/05/22/calibrating-any-compass-or-accelerometer-for-arduino/
- https://github.com/jrowberg/i2cdevlib/blob/master/RaspberryPi_bcm2835/MPU6050/examples/IMU_zero.cpp
- https://forum.arduino.cc/t/arduino-mpu-6050-calibration-and-sensitivity-understanding-please-help/559612/10
- https://arduino.stackexchange.com/questions/63437/how-do-i-know-if-gy-521-mpu6050-has-been-calibrated-properly
- https://learn.adafruit.com/mpu6050-6-dof-accelerometer-and-gyro?view=all 
- https://www.electronicwings.com/raspberry-pi/mpu6050-accelerometergyroscope-interfacing-with-raspberry-pi
- https://www.electronicwings.com/sensors-modules/mpu6050-gyroscope-accelerometer-temperature-sensor-module
- https://tutorials-raspberrypi.com/measuring-rotation-and-acceleration-raspberry-pi/
- https://stackoverflow.com/questions/44722374/how-to-perform-mpu6050-accelerometer-temperature-calibration
- http://42bots.com/tutorials/arduino-script-for-mpu-6050-auto-calibration/
- https://www.i2cdevlib.com/forums/topic/91-how-to-decide-gyro-and-accelerometer-offsett/
- https://forum.makerforums.info/t/as-promised-earlier-here-is-a-quick-example-of-imu-temperature-calibration-fit/70194
- https://github.com/universam1/iSpindel/issues/6 
- https://github.com/ZHomeSlice/Simple_MPU6050/blob/803c3f5d2307984c03857329e34d45cacd4edc04/Simple_MPU6050.cpp#L709 
- https://github.com/jrowberg/i2cdevlib/commit/3f0f9fcad2375502647f3f6cb697df8c05609058
- https://makersportal.com/blog/calibration-of-an-inertial-measurement-unit-with-raspberry-pi 


## I2C instability (error 121):
- https://raspberrypi.stackexchange.com/questions/120913/python-i2c-error-with-smbus-and-gy-521-mpu6050-bytes-omitted-resulting-in-cra
- https://forums.raspberrypi.com/viewtopic.php?t=258678
- https://forum.dexterindustries.com/t/solved-again-seeing-errno-121-i2c-transfer-remote-i-o-error/7968/15
- https://raspberrypi.stackexchange.com/questions/127241/i2c-communication-error-oserror-errno-121-remote-i-o-error-while-using-paj76
- https://raspberrypi.stackexchange.com/questions/91410/rpi-to-arduino-i2c-block-data-read-fails-with-errno-121-remote-i-o-error-pyt
- https://raspberrypi.stackexchange.com/questions/93902/i2c-programm-stops-running-after-a-few-seconds-remote-i-o-error-nr-121
- https://raspberrypi.stackexchange.com/questions/89021/how-to-fix-remote-i-o-error-whilst-using-i2c-oled
- https://raspberrypi.stackexchange.com/questions/97995/rpi-python-i2c-ioerror-errno-121-remote-i-o-error-problem-how-to-fix-it 
- https://stackoverflow.com/questions/69279076/raspi-i2c-ioerror-errno-121-remote-i-o-error
- https://stackoverflow.com/questions/45324851/smbus-on-the-rpi-gives-ioerror-errno-121-remote-i-o-error
- https://stackoverflow.com/questions/52735862/getting-ioerror-errno-121-remote-i-o-error-with-smbus-on-python-raspberry-w 
- https://raspberrypi.stackexchange.com/questions/120913/python-i2c-error-with-smbus-and-gy-521-mpu6050-bytes-omitted-resulting-in-cra

# Other
- Burst reading? MPU 6000 register map, page 29
  - https://stackoverflow.com/questions/65117246/mpu-6050-burst-read-auto-increment

- Understanding accelerometer data sheet https://blog.endaq.com/accelerometer-specifications-decoding-a-datasheet