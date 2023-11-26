/*!
 * Detect all devices connected on the I2C/TWI bus.  Useful if you can't figure out the address of
 * an I2C device.
 *
 * This example will check all possible addresses on the I2C bus for whether a device responds to
 * them.  It will output a table of the results.  This check is done twice, once for reading and
 * once for writing, as some devices only respond to one of the two operations.
 *
 * ATTENTION: Randomly reading from and writing to devices can lead to unexpected results.  Some
 * devices do not cope well with this.  Use with care!
 *
 * Connections
 * -----------
 *  - `A4`: I2C SDA signal
 *  - `A5`: I2C SCL signal
 */
#![no_std]
#![no_main]


use panic_halt as _;
use hd44780_driver::{HD44780, DisplayMode, Display, Cursor, CursorBlink};
use string_buf_emb_rs_lib::StringBuffer;



#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut serial = arduino_hal::default_serial!(dp, pins, 9600);
    let mut adc = arduino_hal::adc::Adc::new(dp.ADC, Default::default());
    let i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(), // SDA
        pins.a5.into_pull_up_input(), // SCL
        50000,
    );

    let vrx = pins.a3.into_analog_input(&mut adc);
    let sw = pins.d2.into_pull_up_input();

    let mut delay = arduino_hal::Delay::new()   ;

    let mut lcd = HD44780::new_i2c(i2c, 0x27, &mut delay).unwrap();
    lcd.clear(&mut delay).unwrap();
    lcd.set_display_mode(
        DisplayMode {
            display: Display::On,
            cursor_visibility: Cursor::Visible,
            cursor_blink: CursorBlink::On,
        }, &mut delay,
    ).unwrap();
    
    let mut str_buf = StringBuffer::from("");
    let mut is_x_pressed = false;
    loop {

        let x = vrx.analog_read(&mut adc);

        if !is_x_pressed && (x > 999 || x < 5) {
            is_x_pressed = true;
            let direction = str_buf.set_cursor_x(x);
            let x = str_buf.get_cursor_pos();
            if x > 15 && x < 17 {
                lcd.set_cursor_pos(40, &mut delay).unwrap();
            }else if x >= 17 && x < 31{
                lcd.shift_cursor(if direction > 0 {hd44780_driver::Direction::Right} else {hd44780_driver::Direction::Left},&mut delay).unwrap();
                
            }
            else if x >= 31{
                lcd.set_cursor_pos(0, &mut delay).unwrap();
                str_buf.reset_cursor();
            }
            else{
                lcd.shift_cursor(if direction > 0 {hd44780_driver::Direction::Right} else {hd44780_driver::Direction::Left},&mut delay).unwrap();
            }
            ufmt::uwriteln!(&mut serial, "x: {}", x).unwrap();
        } else if is_x_pressed && x < 999 && x > 5 {
            is_x_pressed = false;
        }

       


   
        if sw.is_low()  {
            
            arduino_hal::delay_ms(250);
            
            ufmt::uwriteln!(&mut serial, "x: {}", x).unwrap();
            ufmt::uwriteln!(&mut serial, "str: {}", str_buf.to_str()).unwrap();
            ufmt::uwriteln!(&mut serial, "pos: {}", str_buf.get_cursor_pos()).unwrap();
            ufmt::uwriteln!(&mut serial, "actual pos: {}", str_buf.get_display_pos()).unwrap();
            str_buf.select_char();
            lcd.reset(&mut delay).unwrap();
            lcd.clear(&mut delay).unwrap();

            let str_len = str_buf.to_str().len() > 15;
            if !str_len{
                lcd.write_str(str_buf.to_str(), &mut delay).unwrap();
            }else{
                let str1 = str_buf.to_str().get(0..15).unwrap();
                let str2 = str_buf.to_str().get(15..).unwrap();
                lcd.write_str(str1, &mut delay).unwrap();
                lcd.set_cursor_pos(40, &mut delay).unwrap();
                lcd.write_str(str2, &mut delay).unwrap();
            }


            // lcd.set_cursor_pos(str_buf.get_cursor_pos(), &mut delay).unwrap();
            
        }
    }
}