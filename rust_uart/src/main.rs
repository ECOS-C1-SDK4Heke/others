#![no_std]
#![no_main]

use ecos_ssc1::{Timer, Uart, ecos_main, print, println};

#[ecos_main(tick)]
fn main() -> ! {
    println!();
    println!("============================");
    println!("  Rust UART Test Heke1228");
    println!("============================");
    println!("Type characters, 'q' to quit");

    let mut count = 0;

    loop {
        if let Some(c) = Uart::read_byte_nonblock() {
            count += 1;
            print!("{}: ", count);

            match c {
                b'\n' => println!("[ENTER]"),
                b'\r' => println!("[RETURN]"),
                b'q' => {
                    println!("Quitting...");
                    break;
                }
                _ if c >= 32 && c <= 126 => {
                    println!("'{}' (0x{:02X})", c as char, c);
                }
                _ => {
                    println!("[0x{:02X}]", c);
                }
            }
        }

        Timer::delay_ms(10);
    }

    println!("Goodbye!");

    loop {
        unsafe {
            core::arch::asm!("wfi");
        }
    }
}
