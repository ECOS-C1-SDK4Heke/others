use crate::*;

use rusttype::Font;

use embedded_cli::Command;
use embedded_graphics::{Drawable, prelude::*, text::Text};
use embedded_graphics_core::{draw_target::DrawTarget, geometry::Point, pixelcolor::RgbColor};
use embedded_ttf::FontTextStyleBuilder;

const HARMONYOS_SANS_SC_LIGHT: &[u8] =
    include_bytes!("../display/fonts/HarmonyOS_Sans_SC_Regular.ttf");

// 字体演示命令定义
#[derive(Command, Debug)]
pub(crate) enum FontSample {
    /// 启动 TTF 字体演示
    #[command(name = "font")]
    Start,
}

// 字体演示处理函数
pub(crate) fn handle_font_display<T>(
    manager: &mut DisplayManager,
) -> Result<(), core::convert::Infallible> {
    println!("\r\n=== 启动 TTF 字体演示 ===");
    println!("正在启动字体演示...\r\n");

    // 创建字体演示实例
    let mut font_demo = FontDemo::new();

    // 开始交互式演示循环
    loop {
        let byte = Uart::read_byte_nonblock();
        if let Some(byte) = byte {
            // 处理输入，传递 manager 引用
            if font_demo.handle_input(byte, manager) {
                // 用户输入了 'q'，退出演示
                println!("\r\n返回命令行模式...\r\n");
                break;
            }
        }
        #[cfg(feature = "target-ui-sim")]
        {
            manager.update_window();
        }
    }

    Ok(())
}

// 字体演示状态机
pub(crate) struct FontDemo {
    current_demo: Option<FontDemoType>,
    should_exit: bool,
}

// 演示类型枚举
#[derive(Clone, Copy)]
enum FontDemoType {
    Basic,
    Sizes,
    Chinese,
    Mixed,
    Animated,
}

impl FontDemo {
    pub fn new() -> Self {
        Self {
            current_demo: None,
            should_exit: false,
        }
    }

    // 处理键盘输入
    pub fn handle_input(&mut self, byte: u8, manager: &mut DisplayManager) -> bool {
        match byte {
            b'1' => {
                self.play_demo(FontDemoType::Basic, manager);
                false
            }
            b'2' => {
                self.play_demo(FontDemoType::Sizes, manager);
                false
            }
            b'3' => {
                self.play_demo(FontDemoType::Chinese, manager);
                false
            }
            b'4' => {
                self.play_demo(FontDemoType::Mixed, manager);
                false
            }
            b'5' => {
                self.play_demo(FontDemoType::Animated, manager);
                false
            }
            b'n' | b'N' => {
                self.next_demo(manager);
                false
            }
            b'p' | b'P' => {
                self.prev_demo(manager);
                false
            }
            b'q' | b'Q' => {
                println!("退出字体演示...\r\n");
                self.should_exit = true;
                true
            }
            _ => {
                // 忽略其他输入
                false
            }
        }
    }

    // 播放指定演示
    fn play_demo(&mut self, demo_type: FontDemoType, manager: &mut DisplayManager) {
        self.current_demo = Some(demo_type);

        match demo_type {
            FontDemoType::Basic => {
                println!("切换到: 基本字体渲染\r\n");
                basic_font_demo(manager);
            }
            FontDemoType::Sizes => {
                println!("切换到: 不同字体大小\r\n");
                font_sizes_demo(manager);
            }
            FontDemoType::Chinese => {
                println!("切换到: 中文字体渲染\r\n");
                chinese_font_demo(manager);
            }
            FontDemoType::Mixed => {
                println!("切换到: 混合文本和图形\r\n");
                mixed_graphics_demo(manager);
            }
            FontDemoType::Animated => {
                println!("切换到: 动画文本\r\n");
                animated_text_demo(manager);
            }
        }

        println!("\r\n演示结束，输入命令继续...\r\n");
    }

    // 播放下一个演示
    fn next_demo(&mut self, manager: &mut DisplayManager) {
        let next = match self.current_demo {
            Some(FontDemoType::Basic) => FontDemoType::Sizes,
            Some(FontDemoType::Sizes) => FontDemoType::Chinese,
            Some(FontDemoType::Chinese) => FontDemoType::Mixed,
            Some(FontDemoType::Mixed) => FontDemoType::Animated,
            Some(FontDemoType::Animated) => FontDemoType::Basic,
            None => FontDemoType::Basic,
        };
        self.play_demo(next, manager);
    }

    // 播放上一个演示
    fn prev_demo(&mut self, manager: &mut DisplayManager) {
        let prev = match self.current_demo {
            Some(FontDemoType::Basic) => FontDemoType::Animated,
            Some(FontDemoType::Sizes) => FontDemoType::Basic,
            Some(FontDemoType::Chinese) => FontDemoType::Sizes,
            Some(FontDemoType::Mixed) => FontDemoType::Chinese,
            Some(FontDemoType::Animated) => FontDemoType::Mixed,
            None => FontDemoType::Animated,
        };
        self.play_demo(prev, manager);
    }
}

// 演示1: 基本字体渲染
fn basic_font_demo(manager: &mut DisplayManager) -> bool {
    #[allow(unused)]
    use embedded_hal::delay::DelayNs;

    println!("=== 演示1: 基本字体渲染 ===");

    // 清屏为黑色
    manager.display.clear(DisplayColor::BLACK).unwrap();
    #[cfg(feature = "target-ui-sim")]
    manager.update_window();
    manager.delay.delay_ms(500);

    // 加载字体
    println!("加载 HarmonyOS Sans SC Light 字体...");
    let font = match Font::try_from_bytes(HARMONYOS_SANS_SC_LIGHT) {
        Some(font) => {
            println!("字体加载成功");
            font
        }
        None => {
            println!("字体加载失败！");
            return false;
        }
    };

    // 创建字体样式 - 小号字体
    let small_style = FontTextStyleBuilder::new(font.clone())
        .font_size(12) // 12像素高
        .text_color(DisplayColor::WHITE)
        .build();

    // 创建字体样式 - 中号字体
    let medium_style = FontTextStyleBuilder::new(font.clone())
        .font_size(16) // 16像素高
        .text_color(DisplayColor::new(0, 31, 0)) // 绿色
        .build();

    // 创建字体样式 - 大号字体
    let large_style = FontTextStyleBuilder::new(font)
        .font_size(20) // 20像素高
        .text_color(DisplayColor::new(31, 0, 0)) // 红色
        .build();

    // 绘制小号文本
    println!("绘制小号文本...");
    Text::new("Hello TTF!", Point::new(10, 15), small_style)
        .draw(&mut manager.display)
        .unwrap();

    #[cfg(feature = "target-ui-sim")]
    manager.update_window();

    manager.delay.delay_ms(1000);

    // 检查是否按了 'q' 键
    if let Some(byte) = Uart::read_byte_nonblock() {
        if byte == b'q' || byte == b'Q' {
            println!("\r\n用户中断演示");
            return true;
        }
    }

    // 绘制中号文本
    println!("绘制中号文本...");
    Text::new("Embedded", Point::new(10, 40), medium_style)
        .draw(&mut manager.display)
        .unwrap();

    #[cfg(feature = "target-ui-sim")]
    manager.update_window();

    manager.delay.delay_ms(1000);

    // 检查是否按了 'q' 键
    if let Some(byte) = Uart::read_byte_nonblock() {
        if byte == b'q' || byte == b'Q' {
            println!("\r\n用户中断演示");
            return true;
        }
    }

    // 绘制大号文本
    println!("绘制大号文本...");
    Text::new("ECOS", Point::new(10, 70), large_style)
        .draw(&mut manager.display)
        .unwrap();

    #[cfg(feature = "target-ui-sim")]
    manager.update_window();

    manager.delay.delay_ms(2000);

    false
}

// 演示2: 不同字体大小
fn font_sizes_demo(manager: &mut DisplayManager) -> bool {
    #[allow(unused)]
    use embedded_hal::delay::DelayNs;

    println!("=== 演示2: 不同字体大小 ===");

    // 清屏为深蓝色
    manager.display.clear(DisplayColor::new(0, 0, 15)).unwrap();
    #[cfg(feature = "target-ui-sim")]
    manager.update_window();
    manager.delay.delay_ms(500);

    // 加载字体
    let font = match Font::try_from_bytes(HARMONYOS_SANS_SC_LIGHT) {
        Some(font) => {
            println!("字体加载成功");
            font
        }
        None => {
            println!("字体加载失败！");
            return false;
        }
    };

    // 绘制不同大小的字体
    let mut y = 10;
    for (size, text) in [
        (8, "Size 8"),
        (10, "Size 10"),
        (12, "Size 12"),
        (14, "Size 14"),
        (16, "Size 16"),
        (18, "Size 18"),
        (20, "Size 20"),
    ] {
        // 检查是否按了 'q' 键
        if let Some(byte) = Uart::read_byte_nonblock() {
            if byte == b'q' || byte == b'Q' {
                println!("\r\n用户中断演示");
                return true;
            }
        }

        let style = FontTextStyleBuilder::new(font.clone())
            .font_size(size)
            .text_color(DisplayColor::WHITE)
            .build();

        Text::new(text, Point::new(10, y), style)
            .draw(&mut manager.display)
            .unwrap();

        #[cfg(feature = "target-ui-sim")]
        manager.update_window();

        y += size as i32 + 5;
        manager.delay.delay_ms(300);
    }

    manager.delay.delay_ms(1000);
    false
}

// 演示3: 中文字体渲染
fn chinese_font_demo(manager: &mut DisplayManager) -> bool {
    #[allow(unused)]
    use embedded_hal::delay::DelayNs;

    println!("=== 演示3: 中文字体渲染 ===");

    // 清屏为深灰色
    manager.display.clear(DisplayColor::new(8, 8, 8)).unwrap();
    #[cfg(feature = "target-ui-sim")]
    manager.update_window();
    manager.delay.delay_ms(500);

    // 加载字体
    let font = match Font::try_from_bytes(HARMONYOS_SANS_SC_LIGHT) {
        Some(font) => {
            println!("字体加载成功");
            font
        }
        None => {
            println!("字体加载失败！");
            return false;
        }
    };

    // 创建字体样式
    let style = FontTextStyleBuilder::new(font)
        .font_size(16)
        .text_color(DisplayColor::CYAN)
        .build();

    // 绘制中文文本
    println!("绘制中文文本...");

    // 第一行
    Text::new("嵌入式系统", Point::new(10, 20), style.clone())
        .draw(&mut manager.display)
        .unwrap();

    #[cfg(feature = "target-ui-sim")]
    manager.update_window();

    manager.delay.delay_ms(1000);

    // 检查是否按了 'q' 键
    if let Some(byte) = Uart::read_byte_nonblock() {
        if byte == b'q' || byte == b'Q' {
            println!("\r\n用户中断演示");
            return true;
        }
    }

    // 第二行
    Text::new("ECOS开发板", Point::new(10, 45), style.clone())
        .draw(&mut manager.display)
        .unwrap();

    #[cfg(feature = "target-ui-sim")]
    manager.update_window();

    manager.delay.delay_ms(1000);

    // 检查是否按了 'q' 键
    if let Some(byte) = Uart::read_byte_nonblock() {
        if byte == b'q' || byte == b'Q' {
            println!("\r\n用户中断演示");
            return true;
        }
    }

    // 第三行
    Text::new("图形界面演示", Point::new(10, 70), style.clone())
        .draw(&mut manager.display)
        .unwrap();

    #[cfg(feature = "target-ui-sim")]
    manager.update_window();

    manager.delay.delay_ms(1000);

    // 检查是否按了 'q' 键
    if let Some(byte) = Uart::read_byte_nonblock() {
        if byte == b'q' || byte == b'Q' {
            println!("\r\n用户中断演示");
            return true;
        }
    }

    // 第四行
    Text::new("TTF字体渲染", Point::new(10, 95), style)
        .draw(&mut manager.display)
        .unwrap();

    #[cfg(feature = "target-ui-sim")]
    manager.update_window();

    manager.delay.delay_ms(2000);
    false
}

// 演示4: 混合文本和图形
fn mixed_graphics_demo(manager: &mut DisplayManager) -> bool {
    use embedded_graphics::primitives::{PrimitiveStyle, Rectangle};
    #[allow(unused)]
    use embedded_hal::delay::DelayNs;

    println!("=== 演示4: 混合文本和图形 ===");

    // 清屏为黑色
    manager.display.clear(DisplayColor::BLACK).unwrap();
    #[cfg(feature = "target-ui-sim")]
    manager.update_window();
    manager.delay.delay_ms(500);

    // 加载字体
    let font = match Font::try_from_bytes(HARMONYOS_SANS_SC_LIGHT) {
        Some(font) => {
            println!("字体加载成功");
            font
        }
        None => {
            println!("字体加载失败！");
            return false;
        }
    };

    // 绘制背景矩形
    println!("绘制背景矩形...");
    Rectangle::new(Point::new(5, 5), Size::new(118, 50))
        .into_styled(PrimitiveStyle::with_fill(DisplayColor::new(15, 0, 0)))
        .draw(&mut manager.display)
        .unwrap();

    #[cfg(feature = "target-ui-sim")]
    manager.update_window();

    // 绘制文字
    let style = FontTextStyleBuilder::new(font.clone())
        .font_size(14)
        .text_color(DisplayColor::YELLOW)
        .build();

    Text::new("ECOS Display", Point::new(15, 25), style)
        .draw(&mut manager.display)
        .unwrap();

    let style2 = FontTextStyleBuilder::new(font.clone())
        .font_size(12)
        .text_color(DisplayColor::GREEN)
        .build();

    Text::new("ST7735 + TTF", Point::new(15, 45), style2)
        .draw(&mut manager.display)
        .unwrap();

    #[cfg(feature = "target-ui-sim")]
    manager.update_window();

    // 绘制装饰性元素
    Rectangle::new(Point::new(5, 60), Size::new(118, 58))
        .into_styled(PrimitiveStyle::with_stroke(DisplayColor::BLUE, 2))
        .draw(&mut manager.display)
        .unwrap();

    // 绘制小字
    let small_style = FontTextStyleBuilder::new(font)
        .font_size(10)
        .text_color(DisplayColor::WHITE)
        .build();

    Text::new("Version 1.0", Point::new(20, 80), small_style.clone())
        .draw(&mut manager.display)
        .unwrap();

    Text::new("128x128 RGB", Point::new(20, 100), small_style.clone())
        .draw(&mut manager.display)
        .unwrap();

    #[cfg(feature = "target-ui-sim")]
    manager.update_window();

    // 等待期间检查 'q' 键
    for _ in 0..30 {
        manager.delay.delay_ms(100);

        if let Some(byte) = Uart::read_byte_nonblock() {
            if byte == b'q' || byte == b'Q' {
                println!("\r\n用户中断演示");
                return true;
            }
        }
    }

    false
}

// 演示5: 动画文本
fn animated_text_demo(manager: &mut DisplayManager) -> bool {
    use embedded_graphics::primitives::{PrimitiveStyle, Rectangle};
    #[allow(unused)]
    use embedded_hal::delay::DelayNs;

    println!("=== 演示5: 动画文本 ===");
    println!("按 'q' 键可以提前退出动画演示\r\n");

    // 清屏为黑色
    manager.display.clear(DisplayColor::BLACK).unwrap();
    #[cfg(feature = "target-ui-sim")]
    manager.update_window();
    manager.delay.delay_ms(500);

    // 加载字体
    let font = match Font::try_from_bytes(HARMONYOS_SANS_SC_LIGHT) {
        Some(font) => {
            println!("字体加载成功");
            font
        }
        None => {
            println!("字体加载失败！");
            return false;
        }
    };

    // 创建不同的文本样式
    let styles = [
        FontTextStyleBuilder::new(font.clone())
            .font_size(16)
            .text_color(DisplayColor::RED)
            .build(),
        FontTextStyleBuilder::new(font.clone())
            .font_size(16)
            .text_color(DisplayColor::GREEN)
            .build(),
        FontTextStyleBuilder::new(font.clone())
            .font_size(16)
            .text_color(DisplayColor::BLUE)
            .build(),
        FontTextStyleBuilder::new(font.clone())
            .font_size(16)
            .text_color(DisplayColor::YELLOW)
            .build(),
        FontTextStyleBuilder::new(font.clone())
            .font_size(16)
            .text_color(DisplayColor::CYAN)
            .build(),
        FontTextStyleBuilder::new(font.clone())
            .font_size(16)
            .text_color(DisplayColor::MAGENTA)
            .build(),
    ];

    // 动画：彩色文本滚动
    println!("彩色文本滚动动画...");
    println!("按 'q' 键退出动画\r\n");

    let mut frame = 0;
    let mut user_interrupted = false;
    let mut prev_offsets: [i32; 6] = [0; 6];

    while frame < 60 && !user_interrupted {
        // 检查用户是否按了 'q' 键
        if let Some(byte) = Uart::read_byte_nonblock() {
            if byte == b'q' || byte == b'Q' {
                println!("\r\n用户中断动画演示");
                user_interrupted = true;
                break;
            }
        }

        // 清除上一帧的内容
        if frame > 0 {
            for i in 0..6 {
                let y = 10 + (i * 20) as i32;
                // 清除前一帧文本区域（稍微扩大清除区域确保完全清除）
                Rectangle::new(Point::new(prev_offsets[i] - 5, y - 5), Size::new(140, 25))
                    .into_styled(PrimitiveStyle::with_fill(DisplayColor::BLACK))
                    .draw(&mut manager.display)
                    .unwrap_or_else(|e| {
                        println!("清除错误: {:?}", e);
                    });
            }
        }

        // 绘制当前帧的多行彩色文本
        for (i, style) in styles.iter().enumerate() {
            let y = 10 + (i * 20) as i32;
            let offset = ((frame as i32) * 2 + (i as i32) * 10) % 150 - 50;

            // 保存当前偏移量用于下一帧清除
            prev_offsets[i] = offset;

            Text::new("ECOS Display Demo", Point::new(offset, y), style.clone())
                .draw(&mut manager.display)
                .unwrap();
        }

        #[cfg(feature = "target-ui-sim")]
        manager.update_window();

        manager.delay.delay_ms(50);
        frame += 1;
    }

    // 如果用户没有中断，继续淡入淡出效果
    if !user_interrupted {
        println!("淡入淡出效果...");
        println!("按 'q' 键跳过此效果\r\n");

        // 重新加载字体
        let font = match Font::try_from_bytes(HARMONYOS_SANS_SC_LIGHT) {
            Some(font) => font,
            None => return false,
        };

        // 淡入效果
        for brightness in 0..=15 {
            // 检查用户是否按了 'q' 键
            if let Some(byte) = Uart::read_byte_nonblock() {
                if byte == b'q' || byte == b'Q' {
                    println!("\r\n用户中断淡入效果");
                    user_interrupted = true;
                    break;
                }
            }

            // 先清除上一帧
            if brightness > 0 {
                Rectangle::new(Point::new(25, 40), Size::new(80, 25))
                    .into_styled(PrimitiveStyle::with_fill(DisplayColor::BLACK))
                    .draw(&mut manager.display)
                    .unwrap_or_else(|e| {
                        println!("清除错误: {:?}", e);
                    });
            }

            // 创建渐变颜色
            let color = DisplayColor::new(brightness, brightness, brightness);
            let style = FontTextStyleBuilder::new(font.clone())
                .font_size(20)
                .text_color(color)
                .build();

            Text::new("Fade In", Point::new(30, 50), style)
                .draw(&mut manager.display)
                .unwrap();

            #[cfg(feature = "target-ui-sim")]
            manager.update_window();

            manager.delay.delay_ms(30);
        }

        // 如果不是用户中断，等待一下然后淡出
        if !user_interrupted {
            // 等待期间检查 'q' 键
            for _ in 0..10 {
                manager.delay.delay_ms(100);

                #[cfg(feature = "target-ui-sim")]
                manager.update_window();

                if let Some(byte) = Uart::read_byte_nonblock() {
                    if byte == b'q' || byte == b'Q' {
                        println!("\r\n用户中断等待");
                        user_interrupted = true;
                        break;
                    }
                }
            }

            if !user_interrupted {
                // 淡出效果
                for brightness in (0..=15).rev() {
                    // 检查用户是否按了 'q' 键
                    if let Some(byte) = Uart::read_byte_nonblock() {
                        if byte == b'q' || byte == b'Q' {
                            println!("\r\n用户中断淡出效果");
                            user_interrupted = true;
                            break;
                        }
                    }

                    // 先清除上一帧
                    if brightness < 15 {
                        Rectangle::new(Point::new(25, 40), Size::new(80, 25))
                            .into_styled(PrimitiveStyle::with_fill(DisplayColor::BLACK))
                            .draw(&mut manager.display)
                            .unwrap_or_else(|e| {
                                println!("清除错误: {:?}", e);
                            });
                    }

                    // 创建渐变颜色
                    let color = DisplayColor::new(brightness, brightness, brightness);
                    let style = FontTextStyleBuilder::new(font.clone())
                        .font_size(20)
                        .text_color(color)
                        .build();

                    Text::new("Fade Out", Point::new(30, 50), style)
                        .draw(&mut manager.display)
                        .unwrap();

                    #[cfg(feature = "target-ui-sim")]
                    manager.update_window();

                    manager.delay.delay_ms(30);
                }
            }
        }
    }

    // 最后清屏
    manager
        .display
        .clear(DisplayColor::BLACK)
        .unwrap_or_else(|e| {
            println!("清屏错误: {:?}", e);
        });

    #[cfg(feature = "target-ui-sim")]
    manager.update_window();

    // 返回是否被用户中断
    user_interrupted
}
