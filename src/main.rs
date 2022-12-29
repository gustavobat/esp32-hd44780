use embedded_hal::prelude::_embedded_hal_blocking_delay_DelayMs;
use core::time::Duration;
use embedded_hal::digital::v2::*;
use embedded_hal::serial::Read;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::serial::*;
use esp_idf_hal::prelude::*;
use hd44780_driver::{HD44780, DisplayMode, Cursor, CursorBlink, Display};
use esp_idf_svc::systime::EspSystemTime;
use embedded_svc::sys_time::SystemTime;

macro_rules! timeout_block {
    ($timeout_sec:literal, $expression:expr) => {
        let end = EspSystemTime{}.now() + Duration::from_secs($timeout_sec);
        while (EspSystemTime{}.now() < end) {
            $expression
        }
    };
}

fn main() {
    esp_idf_sys::link_patches();
    let peripherals = Peripherals::take().unwrap();

    let pins = peripherals.pins;

    // Config LCD
    let mut delay = esp_idf_hal::delay::FreeRtos;

    let mut lcd = HD44780::new_4bit(
        // Register Select pin
        pins.gpio19.into_output().unwrap(), 
        // Enable pin
        pins.gpio18.into_output().unwrap(),
        // Data pins
        pins.gpio27.into_output().unwrap(),
        pins.gpio26.into_output().unwrap(),
        pins.gpio25.into_output().unwrap(),
        pins.gpio33.into_output().unwrap(),
        &mut delay,
        ).unwrap();

    // Unshift display and set cursor to 0
    lcd.reset(&mut delay).unwrap();

    // Clear existing characters
    lcd.clear(&mut delay).unwrap(); 

    // Configure display mode
    let dm = DisplayMode{
        display: Display::On,
        cursor_visibility: Cursor::Visible,
        cursor_blink: CursorBlink::On,
    };
    lcd.set_display_mode(dm, &mut delay).unwrap();

    // Config STINGR comm
    let config = config::Config::default()
        .baudrate(Hertz(9_600))
        .parity_none()
        .data_bits(config::DataBits::DataBits8)
        .stop_bits(config::StopBits::STOP1);

    let serial: Serial<UART1, _, _> = Serial::new(
        peripherals.uart1,
        Pins {
            tx: pins.gpio1,
            rx: pins.gpio3,
            cts: None,
            rts: None,
        },
        config
        ).unwrap();
    
    let (mut tx, mut rx) = serial.split();

    let mut rts = pins.gpio22.into_output().unwrap();
    let cts = pins.gpio23.into_input().unwrap();

    // Attempt to get ESN from STINGR
    let mut could_connect = false;
    let n_attempts = 4;
    for i in 1..=n_attempts {
        let msg = "ATTEMPT ".to_owned() + i.to_string().as_str() + "/" + n_attempts.to_string().as_str();
        lcd.write_str(&msg, &mut delay).unwrap();
        lcd.set_cursor_xy((0,1), &mut delay).unwrap();

        // Start stingr exchange
        rts.set_low().unwrap();
        while cts.is_high().unwrap() {
            delay.delay_ms(25_u32);
        }
        // Wait before sending command
        delay.delay_ms(10_u32);
        // Query ESN
        let written_len = tx.write_bytes(&[0xAA, 0x05, 0x01, 0x50, 0xD5]).unwrap();
        let mut buf = [0_u8; 9];
        if written_len != 5 {
            lcd.write_str("WRITING ERROR", &mut delay).unwrap();
            could_connect = false;
        } else {
            timeout_block!(2, {
                match rx.read() {
                    Ok(byte) => {
                        could_connect = true;
                        lcd.write_byte(byte, &mut delay).unwrap();
                    },
                    Err(_) => ();
                };
            });

        }
        if could_connect {
            //lcd.write_str("SUCCESS", &mut delay).unwrap();
            delay.delay_ms(1000_u32);
            lcd.clear(&mut delay).unwrap();
            //lcd.write_bytes(&buf, &mut delay).unwrap();
            break;
        } else {
            //lcd.write_str("FAIL", &mut delay).unwrap();
            delay.delay_ms(1000_u32);
            lcd.clear(&mut delay).unwrap(); 
        }
    }
    if !could_connect {
        lcd.write_str("ERROR", &mut delay).unwrap();
    }
}

