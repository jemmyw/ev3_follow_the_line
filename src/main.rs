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

fn next_state(s: States) -> States {
    return match s {
        States::Following => States::SearchLeft,
        States::SearchLeft => States::SearchRight,
        States::SearchRight => States::SearchLeft,
        s => s,
    };
}

fn on_line(sen: &ColorSensor) -> bool {
    let val = sen.get_value0();

    let v = match val {
        Ok(v) => v,
        Err(_) => panic!("problem reading color sensor"),
    };
    println!("sensor value: {}", v);

    return v > 20;
}

fn main() -> Ev3Result<()> {
    println!("running");

    // Get large motor on port outA.
    let left_motor = LargeMotor::get(MotorPort::OutB)?;
    let right_motor = LargeMotor::get(MotorPort::OutC)?;

    println!("got motors");

    let forward_speed = -50;
    let search_speed = -30;
    let search_degs = [1, 3, 6, 12];
    let timeout = Duration::from_millis(250);

    // Set command "run-direct".
    left_motor.run_direct()?;
    right_motor.run_direct()?;

    // Find color sensor. Always returns the first recognised one.
    let color_sensor = ColorSensor::find()?;
    println!("got sensor");

    // Switch to reflect mode.
    color_sensor.set_mode_col_reflect()?;

    let mut state = (States::Following, 0, 0);

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
        state = match on_line(&color_sensor) {
            true => (States::Following, 0, 0),
            false => match state {
                (States::Following, _, _) => (States::SearchLeft, 0, 0),
                (s, times, index) => {
                    if times == search_degs[index] {
                        if index + 1 == search_degs.len() {
                            (States::Stop, 0, 0)
                        } else {
                            (next_state(s), 0, index + 1)
                        }
                    } else {
                        (s, times + 1, index)
                    }
                }
            },
        };

        match state {
            (States::Following, _, _) => go_forward(),
            (States::SearchLeft, _, _) => go_left(),
            (States::SearchRight, _, _) => go_right(),
            (States::Stop, _, _) => break,
        }

        thread::sleep(timeout);
    }

    println!("Stopping");
    go_stop();
    Ok(())
}
