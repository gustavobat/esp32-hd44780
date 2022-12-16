use esp_idf_hal::peripherals::Peripherals;
use hd44780_driver::{HD44780, DisplayMode, Cursor, CursorBlink, Display};

fn main() {
    esp_idf_sys::link_patches();
    let peripherals = Peripherals::take().unwrap();

    let pins = peripherals.pins;

    let mut delay = esp_idf_hal::delay::FreeRtos;

    let mut lcd = HD44780::new_4bit(
        // Register Select pin
        pins.gpio18.into_output().unwrap(),
        // Enable pin
        pins.gpio19.into_output().unwrap(),
        // Data pins
        pins.gpio27.into_output().unwrap(),
        pins.gpio26.into_output().unwrap(),
        pins.gpio25.into_output().unwrap(),
        pins.gpio33.into_output().unwrap(),
        &mut delay,
        )
        .unwrap();

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

    // Display the following string
    lcd.write_str("Hello from Rust!", &mut delay).unwrap();
}

