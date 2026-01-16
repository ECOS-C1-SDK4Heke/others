#![cfg_attr(feature = "need-ecos", no_std)]
#![cfg_attr(feature = "need-ecos", no_main)]

#[cfg(feature = "target-st7735")]
use ecos_ebui::{St7735Config as DisplayConfig, St7735Manager as DisplayManager};
#[cfg(feature = "target-ui-sim")]
use embedded_cli::Command;
#[cfg(feature = "target-ui-sim")]
use embedded_graphics::pixelcolor::Rgb888;
#[cfg(feature = "target-ui-sim")]
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
#[cfg(feature = "target-ui-sim")]
pub use std::collections::HashMap;
#[cfg(feature = "target-ui-sim")]
pub use std::mem;

#[cfg(feature = "target-st7735")]
pub type DisplayColor = embedded_graphics::pixelcolor::Rgb565;
#[cfg(feature = "target-ui-sim")]
pub type DisplayColor = Rgb888;

#[cfg(feature = "need-ecos")]
use ecos_ssc1::{Uart, ecos_main};
#[cfg(not(feature = "need-ecos"))]
pub use uart_simulator::UartSimulator as Uart;

use embedded_cli::{CommandGroup, cli::CliBuilder, command::RawCommand};

#[cfg(feature = "cmd-cli")]
mod cli;
#[cfg(feature = "cmd-cli")]
use cli::{CmdSample, handle_sample};

#[cfg(all(feature = "cmd-cli", feature = "need-ecos"))]
use cli::EbdWriter;

#[cfg(feature = "cmd-font")]
mod font;
#[cfg(feature = "cmd-font")]
use font::{FontSample, handle_font_display};

#[cfg(feature = "cmd-text")]
mod text;
#[cfg(feature = "cmd-text")]
use text::{TextSample, handle_text_display};

#[cfg(feature = "cmd-snake")]
mod snake;
#[cfg(feature = "cmd-snake")]
use snake::{SnakeSample, handle_snake_sample};

#[cfg(feature = "target-ui-sim")]
use uart_simulator::{UartSimulatorWriter, read_byte_nonblock_sim};

#[derive(CommandGroup)]
enum Group<'a> {
    #[cfg(feature = "cmd-cli")]
    Cmd(CmdSample<'a>),
    #[cfg(feature = "cmd-font")]
    Font(FontSample),
    #[cfg(feature = "cmd-text")]
    Text(TextSample<'a>),
    #[cfg(feature = "cmd-snake")]
    Snake(SnakeSample<'a>),
    #[cfg(feature = "target-ui-sim")]
    Quit(QuitCommand),
    Others(RawCommand<'a>),
}

#[cfg(feature = "target-ui-sim")]
pub struct DisplayManager {
    pub display: SimulatorDisplay<DisplayColor>,
    pub window: Window,
    #[cfg(feature = "need-ecos")]
    pub delay: ecos_ssc1::delay::Delay,
    #[cfg(not(feature = "need-ecos"))]
    pub delay: SimulatorDelay,
}

#[cfg(feature = "target-ui-sim")]
impl DisplayManager {
    pub fn update_window(&mut self) {
        self.window.update(&self.display);
    }

    pub fn display_mut(&mut self) -> &mut SimulatorDisplay<DisplayColor> {
        &mut self.display
    }

    pub fn window_mut(&mut self) -> &mut Window {
        &mut self.window
    }
}

#[cfg(feature = "target-ui-sim")]
pub struct SimulatorDelay;

#[cfg(feature = "target-ui-sim")]
impl SimulatorDelay {
    pub fn delay_ms(&self, ms: u32) {
        std::thread::sleep(std::time::Duration::from_millis(ms as u64));
    }

    pub fn delay_us(&self, us: u32) {
        std::thread::sleep(std::time::Duration::from_micros(us as u64));
    }
}

#[cfg(feature = "need-ecos")]
#[ecos_main(tick, qspi(0))]
fn main() -> ! {
    run_main()
}

#[cfg(not(feature = "need-ecos"))]
fn main() -> ! {
    run_main()
}

fn run_main() -> ! {
    // 初始化显示管理器
    #[cfg(feature = "target-st7735")]
    let config = DisplayConfig {
        dc_pin: 14,
        rst_pin: None,
        width: 128,
        height: 128,
        rgb: true,
        inverted: false,
    };

    #[cfg(feature = "target-st7735")]
    let mut manager = match DisplayManager::new(config) {
        Ok(manager) => {
            println!("显示管理器创建成功");
            manager
        }
        Err(e) => {
            println!("显示管理器创建失败: {:?}", e);
            loop {}
        }
    };

    #[cfg(feature = "target-ui-sim")]
    let mut manager = {
        let display =
            SimulatorDisplay::<DisplayColor>::new(embedded_graphics::prelude::Size::new(128, 128));
        let output_settings = OutputSettingsBuilder::new().scale(2).build();
        let window = Window::new("ECOS Simulator", &output_settings);

        DisplayManager {
            display,
            window,
            #[cfg(feature = "need-ecos")]
            delay: ecos_ssc1::delay::Delay,
            #[cfg(not(feature = "need-ecos"))]
            delay: SimulatorDelay,
        }
    };

    #[cfg(feature = "target-st7735")]
    match manager.init() {
        Ok(_) => println!("显示初始化成功！"),
        Err(e) => {
            println!("显示初始化失败: {:?}", e);
            loop {}
        }
    }

    #[cfg(feature = "target-ui-sim")]
    {
        println!("模拟器显示初始化成功！");
        manager.window.update(&manager.display);
    }

    #[allow(static_mut_refs)]
    let (command_buffer, history_buffer) = unsafe {
        static mut COMMAND_BUFFER: [u8; 128] = [0; 128];
        static mut HISTORY_BUFFER: [u8; 256] = [0; 256];
        (COMMAND_BUFFER.as_mut(), HISTORY_BUFFER.as_mut())
    };

    let mut cli = CliBuilder::default()
        .writer({
            #[cfg(all(feature = "cmd-cli", feature = "need-ecos"))]
            {
                EbdWriter {}
            }
            #[cfg(feature = "target-ui-sim")]
            {
                UartSimulatorWriter {}
            }
        })
        .command_buffer(command_buffer)
        .history_buffer(history_buffer)
        .prompt("\u{001b}[92mheke1228\u{001b}[0m@\u{001b}[94mecos-ssc1\u{001b}[0m$ ")
        .build()
        .unwrap();

    loop {
        let byte = {
            #[cfg(feature = "need-ecos")]
            {
                Uart::read_byte_nonblock()
            }
            #[cfg(feature = "target-ui-sim")]
            {
                read_byte_nonblock_sim()
            }
        };

        if let Some(byte) = byte {
            // 在闭包内部使用 &mut manager
            let _ = cli.process_byte::<Group, _>(
                byte,
                &mut Group::processor(|_cli, command| match command {
                    #[cfg(feature = "cmd-cli")]
                    Group::Cmd(cmd) => handle_sample(cmd),
                    #[cfg(feature = "cmd-font")]
                    Group::Font(cmd) => match cmd {
                        FontSample::Start => {
                            // 启动字体演示，直接使用 &mut manager
                            handle_font_display::<DisplayColor>(&mut manager)
                        }
                    },
                    #[cfg(feature = "cmd-text")]
                    Group::Text(cmd) => {
                        // 处理文本命令，直接使用 &mut manager
                        handle_text_display::<DisplayColor>(&mut manager, cmd)
                    }
                    #[cfg(feature = "cmd-snake")]
                    Group::Snake(cmd) => handle_snake_sample::<DisplayColor>(&mut manager, cmd),
                    #[cfg(feature = "target-ui-sim")]
                    Group::Quit(cmd) => match cmd {
                        QuitCommand::Quit | QuitCommand::Exit | QuitCommand::Close => {
                            println!("正在退出程序...");
                            std::process::exit(0);
                        }
                    },
                    Group::Others(cmd) => {
                        println!("暂时不支持命令：\n{:#?}", cmd);
                        Ok(())
                    }
                }),
            );

            #[cfg(feature = "target-ui-sim")]
            {
                // 直接使用 manager，不需要单独的引用
                manager.window.update(&manager.display);
            }
        }

        #[cfg(feature = "target-ui-sim")]
        {
            // 处理SDL事件，避免窗口无响应
            for event in manager.window.events() {
                use embedded_graphics_simulator::SimulatorEvent;
                #[cfg(feature = "target-ui-sim")]
                use embedded_graphics_simulator::sdl2::Keycode;

                match event {
                    SimulatorEvent::Quit => std::process::exit(0),
                    SimulatorEvent::KeyDown {
                        keycode: Keycode::Escape,
                        ..
                    } => {
                        std::process::exit(0);
                    }
                    _ => {}
                }
            }
        }
    }
}

#[cfg(not(feature = "need-ecos"))]
mod uart_simulator {
    use core::fmt;
    use std::collections::VecDeque;
    use std::io::{self, Read, Write};
    use std::sync::Mutex;
    use std::time::{Duration, Instant};

    static INPUT_QUEUE: Mutex<VecDeque<u8>> = Mutex::new(VecDeque::new());

    pub struct UartSimulator;

    impl UartSimulator {
        pub fn init() {
            println!("UART 模拟器初始化完成");
        }

        pub fn write_byte(b: u8) {
            print!("{}", b as char);
            io::stdout().flush().unwrap();
        }

        pub fn write_str(s: &str) {
            print!("{}", s);
            io::stdout().flush().unwrap();
        }

        pub fn read_byte_nonblock() -> Option<u8> {
            // 首先检查是否有缓存的输入
            if let Ok(mut queue) = INPUT_QUEUE.lock() {
                if !queue.is_empty() {
                    return queue.pop_front();
                }
            }

            // 使用带超时的非阻塞读取
            let start = Instant::now();
            let timeout = Duration::from_millis(10);
            let mut buffer = [0u8; 1];

            // 尝试读取，但只等待一小段时间
            while start.elapsed() < timeout {
                // 检查是否有数据（使用平台特定的方法）
                #[cfg(unix)]
                {
                    use libc::{POLLIN, poll, pollfd};
                    use std::os::unix::io::AsRawFd;

                    let fd = io::stdin().as_raw_fd();
                    let mut fds = [pollfd {
                        fd,
                        events: POLLIN,
                        revents: 0,
                    }];

                    let remaining_timeout = (timeout - start.elapsed()).as_millis() as i32;
                    let ret = unsafe { poll(fds.as_mut_ptr(), 1, remaining_timeout) };

                    if ret > 0 && (fds[0].revents & POLLIN) != 0 {
                        match io::stdin().read(&mut buffer) {
                            Ok(1) => return Some(buffer[0]),
                            _ => return None,
                        }
                    }
                }

                #[cfg(windows)]
                {
                    use windows_sys::Win32::Foundation::{HANDLE, STD_INPUT_HANDLE};
                    use windows_sys::Win32::System::Console::{
                        GetNumberOfConsoleInputEvents, GetStdHandle,
                    };

                    unsafe {
                        let handle = GetStdHandle(STD_INPUT_HANDLE);
                        if handle != -1 {
                            let mut events = 0u32;
                            if GetNumberOfConsoleInputEvents(handle, &mut events) != 0 && events > 0
                            {
                                match io::stdin().read(&mut buffer) {
                                    Ok(1) => return Some(buffer[0]),
                                    _ => return None,
                                }
                            }
                        }
                    }
                }

                #[cfg(not(any(unix, windows)))]
                {
                    // 通用方案：直接尝试读取
                    match io::stdin().read(&mut buffer) {
                        Ok(1) => return Some(buffer[0]),
                        Ok(_) => return None,
                        Err(_) => return None,
                    }
                }

                // 短暂休眠以避免忙等待
                std::thread::sleep(Duration::from_millis(1));
            }

            None
        }

        pub fn read_byte_blocking() -> u8 {
            loop {
                if let Some(b) = Self::read_byte_nonblock() {
                    return b;
                }
            }
        }

        pub fn write_bytes(bytes: &[u8]) {
            print!("{}", String::from_utf8_lossy(bytes));
            io::stdout().flush().unwrap();
        }
    }

    pub struct UartSimulatorWriter;

    impl fmt::Write for UartSimulatorWriter {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            print!("{}", s);
            io::stdout().flush().unwrap();
            Ok(())
        }
    }

    impl embedded_io::ErrorType for UartSimulatorWriter {
        type Error = core::convert::Infallible;
    }

    impl embedded_io::Write for UartSimulatorWriter {
        fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
            print!("{}", String::from_utf8_lossy(buf));
            io::stdout().flush().unwrap();
            Ok(buf.len())
        }

        fn flush(&mut self) -> Result<(), Self::Error> {
            io::stdout().flush().unwrap();
            Ok(())
        }
    }

    pub fn read_byte_nonblock_sim() -> Option<u8> {
        // 首先检查是否有缓存的输入
        if let Ok(mut queue) = INPUT_QUEUE.lock() {
            if !queue.is_empty() {
                return queue.pop_front();
            }
        }

        // 检查标准输入是否有数据
        let mut buffer = [0u8; 1];
        if let Ok(1) = io::stdin().read(&mut buffer) {
            Some(buffer[0])
        } else {
            None
        }
    }
}

#[cfg(feature = "target-ui-sim")]
#[derive(Command, Debug)]
enum QuitCommand {
    /// 退出程序
    #[command(name = "quit")]
    Quit,
    /// 退出程序
    #[command(name = "exit")]
    Exit,
    /// 退出程序
    #[command(name = "close")]
    Close,
}
