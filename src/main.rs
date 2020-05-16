extern crate ev3dev_lang_rust;

use ev3dev_lang_rust::motors::{LargeMotor, MotorPort};
use ev3dev_lang_rust::sensors::ColorSensor;
use ev3dev_lang_rust::Ev3Result;
use std::thread;
use std::time::Duration;

enum States {
    Following,
    SearchLeft,
    SearchRight,
    Stop,
}

fn on_line(sen: &ColorSensor) -> bool {
    let val = sen.get_value0();

    let r = match val {
        Ok(v) => v < 20,
        Err(_) => panic!("problem reading color sensor"),
    };

    return r;
}

fn main() -> Ev3Result<()> {
    // Get large motor on port outA.
    let left_motor = LargeMotor::get(MotorPort::OutB)?;
    let right_motor = LargeMotor::get(MotorPort::OutC)?;

    let forward_speed = -50;
    let search_speed = -15;

    // Set command "run-direct".
    left_motor.run_direct()?;
    right_motor.run_direct()?;

    // Find color sensor. Always returns the first recognised one.
    let color_sensor = ColorSensor::find()?;

    // Switch to reflect mode.
    color_sensor.set_mode_col_reflect()?;

    let timeout = Duration::from_millis(500);
    let mut state = (States::Following, 0);

    let go = |l, r| {
        let _r = match left_motor.set_duty_cycle_sp(l) {
            Ok(_) => match right_motor.set_duty_cycle_sp(r) {
                Ok(v) => v,
                Err(_) => panic!("Could not set right motor"),
            },
            Err(_) => panic!("Could not set left motor"),
        };

        return;
    };

    let go_forward = || {
        go(forward_speed, forward_speed);
    };

    let go_left = || {
        go(search_speed, -search_speed);
    };

    let go_right = || {
        go(-search_speed, search_speed);
    };

    let go_stop = || {
        go(0, 0);
    };

    loop {
        let next_state = match on_line(&color_sensor) {
            true => (States::Following, 0),
            false => match state {
                (States::Following, _) => (States::SearchLeft, 0),
                (States::SearchLeft, 6) => (States::SearchRight, 0),
                (States::SearchRight, 6) => (States::Stop, 0),
                (s, l) => (s, l + 1),
            },
        };

        state = next_state;

        match state {
            (States::Following, _) => go_forward(),
            (States::SearchLeft, _) => go_left(),
            (States::SearchRight, _) => go_right(),
            (States::Stop, _) => break,
        }

        thread::sleep(timeout);
    }

    go_stop();
    Ok(())
}
