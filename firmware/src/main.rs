#![no_std]
#![no_main]

use core::time::Duration;

use embassy_executor::Spawner;
use embassy_rp::{
    bind_interrupts, gpio,
    peripherals::{PIO0, USB},
    pio::{InterruptHandler as PioInterruptHandler, Pio},
    pio_programs::pwm::{PioPwm, PioPwmProgram},
    usb::{Driver, InterruptHandler as UsbInterruptHandler},
};
use embassy_time::Timer;
use gpio::{Level, Output};

use servo::ServoBuilder;

use log::*;
use usb::{usb_task, SERVO_DEGREES};
use {defmt_rtt as _, panic_probe as _};

mod servo;
mod usb;

// Bind the interupts to the corresponding handlers
bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => UsbInterruptHandler<USB>;
    PIO0_IRQ_0 => PioInterruptHandler<PIO0>;
});

// This is the main function for the program. Where execution starts.
#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // This returns the peripherals struct
    let p = embassy_rp::init(Default::default());

    // This handles the usb
    let driver = Driver::new(p.USB, Irqs);

    // Spawn the usb_task, and pass the driver for it.
    spawner.spawn(usb_task(driver)).unwrap();

    // Defining the pins that are to be used with the program
    // Note that the LED pin on the Pico W is PIN_16
    let mut led = Output::new(p.PIN_25, Level::Low);

    // This defines a Servo, not really in use rn, but it will be more integrated in the final code,
    // Mostly detached for easy testing
    let Pio {
        mut common,
        sm0,
        sm1,
        sm2,
        ..
    } = Pio::new(p.PIO0, Irqs);
    let prg = PioPwmProgram::new(&mut common);

    let degree_rotation = 100;
    let min_pulse = 500;
    let max_pulse = 2500;

    let pwm_pio = PioPwm::new(&mut common, sm0, p.PIN_2, &prg);
    let mut thumb_servo = ServoBuilder::new(pwm_pio)
        .set_max_degree_rotation(degree_rotation) // Example of adjusting values for MG996R servo
        .set_min_pulse_width(Duration::from_micros(min_pulse)) // This value was detemined by a rough experiment.
        .set_max_pulse_width(Duration::from_micros(max_pulse)) // Along with this value.
        .build();

    let pwm_pio = PioPwm::new(&mut common, sm1, p.PIN_3, &prg);
    let mut four_fingers_servo = ServoBuilder::new(pwm_pio)
        .set_max_degree_rotation(degree_rotation)
        .set_min_pulse_width(Duration::from_micros(min_pulse))
        .set_max_pulse_width(Duration::from_micros(max_pulse))
        .build();

    let pwm_pio = PioPwm::new(&mut common, sm2, p.PIN_4, &prg);
    let mut arm_servo = ServoBuilder::new(pwm_pio)
        .set_max_degree_rotation(degree_rotation)
        .set_min_pulse_width(Duration::from_micros(min_pulse))
        .set_max_pulse_width(Duration::from_micros(max_pulse))
        .build();

    led.set_high(); // turn on the led

    info!("Getting started");

    thumb_servo.start();
    arm_servo.start();
    four_fingers_servo.start();

    loop {
        let servo_request = SERVO_DEGREES.receive().await;
        match servo_request.servo {
            usb::Servo::Thumb => thumb_servo.rotate(servo_request.degrees.into()),
            usb::Servo::Arm => arm_servo.rotate(servo_request.degrees.into()),
            usb::Servo::Fingers => four_fingers_servo.rotate(servo_request.degrees.into()),
        }

        Timer::after_millis(10).await;
    }
}
