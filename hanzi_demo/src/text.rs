#![allow(static_mut_refs)]

use crate::*;

use rusttype::Font;

use embedded_cli::Command;
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::{Drawable, prelude::*, text::Text};
use embedded_graphics_core::{draw_target::DrawTarget, geometry::Point, pixelcolor::RgbColor};
use embedded_text::{
    TextBox,
    alignment::HorizontalAlignment,
    style::{HeightMode, TabSize, TextBoxStyleBuilder},
};
use embedded_ttf::FontTextStyleBuilder;

const HARMONYOS_SANS_SC_LIGHT: &[u8] =
    include_bytes!("../display/fonts/HarmonyOS_Sans_SC_Regular.ttf");

// 全局文档存储
static mut DOCUMENTS: Option<HashMap<String, String>> = None;

// 初始化全局文档存储
fn init_documents() {
    unsafe {
        if DOCUMENTS.is_none() {
            DOCUMENTS = Some(HashMap::new());

            // 预加载一些示例文档
            if let Some(docs) = &mut DOCUMENTS {
                docs.insert("demo.txt".to_string(), DEMO_TEXT.to_string());
                docs.insert("readme.md".to_string(), README_TEXT.to_string());
                docs.insert("poem.txt".to_string(), POEM_TEXT.to_string());
            }
        }
    }
}

// 获取文档存储的可变引用
fn get_documents_mut() -> &'static mut HashMap<String, String> {
    unsafe {
        init_documents();
        DOCUMENTS.as_mut().unwrap()
    }
}

// 获取文档存储的不可变引用
fn get_documents() -> &'static HashMap<String, String> {
    unsafe {
        init_documents();
        DOCUMENTS.as_ref().unwrap()
    }
}

const DEMO_TEXT: &str = r#"
"在这个世界，没有绝对的光明，也没有绝对的黑暗，只有无尽的灰。"

"每个人的心中都有一座神殿，供奉着他们自己的神。"

"命运的齿轮一旦开始转动，就无法停止，只能等待它将我们带向何方。"

"真正的力量，不在于你能控制多少，而在于你能承受多少。"

"在无尽的时间长河中，我们都是过客，唯有信念，能让我们留下痕迹。"

"总有些事情，高于其他。"

"灰雾之上的愚者，高贵的盥洗室之神！"

"不属于这个时代的愚者，灰雾之上的神秘主宰，执掌好运的黄黑之王。"

    --Lord of the Mysteries
"#;

const README_TEXT: &str = r#"ECOS 文本编辑器使用指南

快捷键说明：
===========
导航:
  ↑/↓       - 上下移动一行
  ←/→       - 左右移动一个字符
  Ctrl+↑/↓  - 上下滚动
  Home/End  - 行首/行尾
  PgUp/PgDn - 翻页

编辑:
  Enter     - 插入新行
  Backspace - 删除前一个字符
  Delete    - 删除后一个字符
  Ctrl+S    - 保存文件
  Ctrl+Q    - 退出编辑器

文件操作:
  Ctrl+N    - 新建文件
  Ctrl+O    - 打开文件
  Ctrl+S    - 保存文件
  Ctrl+W    - 关闭文件

模式切换:
  F1        - 阅读模式
  F2        - 编辑模式
  F3        - 文件浏览
  Esc       - 返回上级"#;

const POEM_TEXT: &str = r#"登鹳雀楼 - 王之涣

白日依山尽，
黄河入海流。
欲穷千里目，
更上一层楼。"#;

// 文本命令定义
#[derive(Command, Debug)]
pub(crate) enum TextSample<'a> {
    /// 启动文本阅读器
    #[command(name = "reader")]
    Reader,

    /// 启动文本编辑器
    #[command(name = "editor")]
    Editor,

    /// 显示文件列表
    #[command(name = "list")]
    List,

    /// 创建新文件
    #[command(name = "new")]
    New {
        /// 文件名
        filename: &'a str,
    },

    /// 打开文件
    #[command(name = "open")]
    Open {
        /// 文件名
        filename: &'a str,
    },
}

// 文本处理函数
pub(crate) fn handle_text_display<'a, T>(
    manager: &mut DisplayManager,
    command: TextSample<'a>,
) -> Result<(), core::convert::Infallible> {
    match command {
        TextSample::Reader => {
            println!("\r\n=== 启动文本阅读器 ===");
            let mut reader = TextReader::new(manager);
            reader.run()
        }
        TextSample::Editor => {
            println!("\r\n=== 启动文本编辑器 ===");
            let mut editor = TextEditor::new(manager);
            editor.run()
        }
        TextSample::List => {
            println!("\r\n=== 文件列表 ===");
            let docs = get_documents();
            for (filename, content) in docs.iter() {
                let lines: Vec<&str> = content.lines().collect();
                let first_line = lines.first().unwrap_or(&"");
                println!("{}: {}", filename, first_line);
            }
            Ok(())
        }
        TextSample::New { filename } => {
            println!("\r\n=== 创建新文件: {} ===", filename);
            let docs = get_documents_mut();
            docs.insert(filename.to_string(), String::new());
            println!("文件创建成功");
            Ok(())
        }
        TextSample::Open { filename } => {
            println!("\r\n=== 打开文件: {} ===", filename);
            let docs = get_documents();
            let filename_str = filename.to_string();
            if let Some(content) = docs.get(&filename_str) {
                println!("文件内容:");
                println!("{}", content);
            } else {
                println!("文件不存在");
            }
            Ok(())
        }
    }
}

// 文本阅读器
pub(crate) struct TextReader<'a> {
    manager: &'a mut DisplayManager,
    font: Option<Font<'static>>,
    current_file: String,
    scroll_offset: i32,
}

impl<'a> TextReader<'a> {
    pub fn new(manager: &'a mut DisplayManager) -> Self {
        Self {
            manager,
            font: None,
            current_file: "demo.txt".to_string(),
            scroll_offset: 0,
        }
    }

    pub fn run(&mut self) -> Result<(), core::convert::Infallible> {
        // 加载字体
        self.font = match Font::try_from_bytes(HARMONYOS_SANS_SC_LIGHT) {
            Some(font) => {
                println!("字体加载成功");
                Some(font)
            }
            None => {
                println!("字体加载失败");
                return Ok(());
            }
        };

        println!("\r\n=== 文本阅读器 ===");
        println!("可用命令:");
        println!("  n/p - 下一个/上一个文档");
        println!("  ↑/↓ - 滚动文本");
        println!("  f   - 切换文件");
        println!("  q   - 退出");
        println!("==================\r\n");

        self.display_current_document()?;

        // 主循环
        loop {
            if let Some(byte) = Uart::read_byte_nonblock() {
                match byte {
                    b'q' | b'Q' => {
                        println!("\r\n退出阅读器");
                        break;
                    }
                    b'n' | b'N' => {
                        self.next_document();
                        self.display_current_document()?;
                    }
                    b'p' | b'P' => {
                        self.prev_document();
                        self.display_current_document()?;
                    }
                    b'f' | b'F' => {
                        self.select_file();
                        self.display_current_document()?;
                    }
                    b'[' | b'k' | b'K' => {
                        // 上滚
                        self.scroll_offset = (self.scroll_offset - 10).max(0);
                        self.display_current_document()?;
                    }
                    b']' | b'j' | b'J' => {
                        // 下滚
                        self.scroll_offset += 10;
                        self.display_current_document()?;
                    }
                    _ => {}
                }
            }
            #[cfg(feature = "target-ui-sim")]
            {
                self.manager.update_window();
            }
        }

        Ok(())
    }

    fn display_current_document(&mut self) -> Result<(), core::convert::Infallible> {
        // 清屏
        let _ = self.manager.display.clear(DisplayColor::new(0, 0, 8));

        let docs = get_documents();
        if let Some(content) = docs.get(&self.current_file) {
            println!("显示文档: {}", self.current_file);

            // 创建字体样式
            let font = match &self.font {
                Some(font) => font,
                None => return Ok(()),
            };

            let style = FontTextStyleBuilder::new(font.clone())
                .font_size(24) // 大号字体
                .text_color(DisplayColor::WHITE)
                .build();

            // 创建文本框样式 - 使用更安全的参数
            let textbox_style = TextBoxStyleBuilder::new()
                .alignment(HorizontalAlignment::Left)
                .height_mode(HeightMode::FitToText) // 改为FitToText避免溢出
                .tab_size(TabSize::Spaces(4))
                .build();

            // 计算显示区域，确保不会出现负数
            let scroll_offset = self.scroll_offset.max(0); // 确保scroll_offset非负
            let display_bounds =
                Rectangle::new(Point::new(5, 5 - scroll_offset), Size::new(118, 118));

            // 确保显示区域在合理范围内
            if display_bounds.size.height > 0 && display_bounds.size.width > 0 {
                // 创建并绘制文本框
                let text_box =
                    TextBox::with_textbox_style(content, display_bounds, style, textbox_style);

                let _ = text_box.draw(&mut self.manager.display);
            }

            // 绘制滚动条
            self.draw_scrollbar();

            // 绘制状态栏
            self.draw_status_bar();
        }

        Ok(())
    }

    fn draw_scrollbar(&mut self) {
        use embedded_graphics::primitives::{PrimitiveStyle, Rectangle};

        let docs = get_documents();
        if let Some(content) = docs.get(&self.current_file) {
            // 计算总行数
            let line_count = content.lines().count() as i32;
            let visible_lines = 5; // 大约5行24像素字体

            if line_count > visible_lines {
                // 绘制滚动条轨道
                let _ = Rectangle::new(Point::new(122, 5), Size::new(3, 118))
                    .into_styled(PrimitiveStyle::with_fill(DisplayColor::new(30, 30, 30)))
                    .draw(&mut self.manager.display);

                // 计算滑块位置和大小
                let track_height = 118;
                let slider_height = (track_height * visible_lines / line_count).max(10);
                let slider_position = if line_count * 24 / 10 > 0 {
                    (track_height - slider_height) * self.scroll_offset
                        / (line_count * 24 / 10).max(1)
                } else {
                    0
                };

                // 绘制滑块
                let _ = Rectangle::new(
                    Point::new(122, 5 + slider_position as i32),
                    Size::new(3, slider_height as u32),
                )
                .into_styled(PrimitiveStyle::with_fill(DisplayColor::CYAN))
                .draw(&mut self.manager.display);
            }
        }
    }

    fn draw_status_bar(&mut self) {
        use embedded_graphics::mono_font::{MonoTextStyle, ascii::FONT_6X10};
        use embedded_graphics::primitives::{PrimitiveStyle, Rectangle};

        // 状态栏背景
        let _ = Rectangle::new(Point::new(0, 123), Size::new(128, 5))
            .into_styled(PrimitiveStyle::with_fill(DisplayColor::new(20, 20, 20)))
            .draw(&mut self.manager.display);

        // 状态栏文本
        let style = MonoTextStyle::new(&FONT_6X10, DisplayColor::WHITE);

        // 文件名
        let filename_text = if self.current_file.len() > 12 {
            format!("{}...", &self.current_file[..9])
        } else {
            self.current_file.clone()
        };

        let _ =
            Text::new(&filename_text, Point::new(2, 124), style).draw(&mut self.manager.display);

        // 滚动位置
        let docs = get_documents();
        if let Some(content) = docs.get(&self.current_file) {
            let line_count = content.lines().count();
            let scroll_text = format!("{}/{}", self.scroll_offset / 24 + 1, line_count.max(1));

            let _ = Text::new(&scroll_text, Point::new(100, 124), style)
                .draw(&mut self.manager.display);
        }
    }

    fn next_document(&mut self) {
        let docs = get_documents();
        let mut files: Vec<&String> = docs.keys().collect();
        files.sort();

        if let Some(pos) = files.iter().position(|&f| f == &self.current_file) {
            if pos + 1 < files.len() {
                self.current_file = files[pos + 1].clone();
                self.scroll_offset = 0;
            }
        }
    }

    fn prev_document(&mut self) {
        let docs = get_documents();
        let mut files: Vec<&String> = docs.keys().collect();
        files.sort();

        if let Some(pos) = files.iter().position(|&f| f == &self.current_file) {
            if pos > 0 {
                self.current_file = files[pos - 1].clone();
                self.scroll_offset = 0;
            }
        }
    }

    fn select_file(&mut self) {
        println!("\r\n=== 选择文件 ===");
        let docs = get_documents();
        let mut files: Vec<&String> = docs.keys().collect();
        files.sort();

        for (i, filename) in files.iter().enumerate() {
            println!("  {} - {}", i + 1, filename);
        }
        println!("  0 - 取消");
        print!("选择文件编号: ");

        // 简单输入处理
        let mut input = String::new();
        let mut input_complete = false;

        while !input_complete {
            if let Some(byte) = Uart::read_byte_nonblock() {
                match byte {
                    b'\r' | b'\n' => {
                        input_complete = true;
                    }
                    b'0'..=b'9' => {
                        input.push(byte as char);
                        print!("{}", byte as char);
                    }
                    _ => {}
                }
            }
        }

        println!();

        if let Ok(index) = input.parse::<usize>() {
            if index > 0 && index <= files.len() {
                self.current_file = files[index - 1].clone();
                self.scroll_offset = 0;
                println!("已选择: {}", self.current_file);
            }
        }
    }
}

// 文本编辑器
pub(crate) struct TextEditor<'a> {
    manager: &'a mut DisplayManager,
    font: Option<Font<'static>>,
    current_file: String,
    content: String,
    cursor_pos: (usize, usize), // (行, 列)
    scroll_offset: (i32, i32),  // (水平滚动, 垂直滚动)
    mode: EditorMode,
}

#[derive(PartialEq)]
enum EditorMode {
    Normal,
    Insert,
    Command,
}

impl<'a> TextEditor<'a> {
    pub fn new(manager: &'a mut DisplayManager) -> Self {
        Self {
            manager,
            font: None,
            current_file: "demo.txt".to_string(),
            content: String::new(),
            cursor_pos: (0, 0),
            scroll_offset: (0, 0),
            mode: EditorMode::Normal,
        }
    }

    pub fn run(&mut self) -> Result<(), core::convert::Infallible> {
        // 加载字体
        self.font = match Font::try_from_bytes(HARMONYOS_SANS_SC_LIGHT) {
            Some(font) => {
                println!("字体加载成功");
                Some(font)
            }
            None => {
                println!("字体加载失败");
                return Ok(());
            }
        };

        // 加载当前文件内容
        let current_file = self.current_file.to_string();
        self.load_file(&current_file);

        println!("\r\n=== 文本编辑器 ===");
        println!("模式: Normal (按 i 进入插入模式)");
        println!("快捷键:");
        println!("  i     - 进入插入模式");
        println!("  Esc   - 返回Normal模式");
        println!("  :     - 进入命令模式");
        println!("  h/j/k/l - 左/下/上/右移动");
        println!("  w/b   - 向前/后移动一个词");
        println!("  0/$   - 行首/行尾");
        println!("  dd    - 删除当前行");
        println!("  x     - 删除字符");
        println!("  u     - 撤销");
        println!("  Ctrl+S - 保存");
        println!("  :q    - 退出");
        println!("==================\r\n");

        self.display_editor()?;

        // 主循环
        let mut last_key: Option<u8> = None;
        let mut command_buffer = String::new();

        loop {
            if let Some(byte) = Uart::read_byte_nonblock() {
                match self.mode {
                    EditorMode::Normal => {
                        if self.handle_normal_mode(byte, &mut last_key)? {
                            break;
                        }
                    }
                    EditorMode::Insert => {
                        if self.handle_insert_mode(byte)? {
                            break;
                        }
                    }
                    EditorMode::Command => {
                        if self.handle_command_mode(byte, &mut command_buffer)? {
                            break;
                        }
                    }
                }

                self.display_editor()?;
            }
        }

        Ok(())
    }

    fn handle_normal_mode(
        &mut self,
        byte: u8,
        last_key: &mut Option<u8>,
    ) -> Result<bool, core::convert::Infallible> {
        match byte {
            b'i' | b'I' => {
                println!("进入插入模式");
                self.mode = EditorMode::Insert;
            }
            b':' => {
                println!("进入命令模式");
                self.mode = EditorMode::Command;
                print!(":");
            }
            b'q' | b'Q' => {
                println!("\r\n退出编辑器");
                return Ok(true);
            }
            b'h' => self.move_cursor_left(),
            b'j' => self.move_cursor_down(),
            b'k' => self.move_cursor_up(),
            b'l' => self.move_cursor_right(),
            b'w' => self.move_word_forward(),
            b'b' => self.move_word_backward(),
            b'0' => self.move_to_line_start(),
            b'$' => self.move_to_line_end(),
            b'x' => self.delete_char(),
            b'd' => {
                if let Some(b'd') = *last_key {
                    self.delete_line();
                    *last_key = None;
                } else {
                    *last_key = Some(b'd');
                }
            }
            b'u' => {
                // 简单撤销 - 这里只是重新加载文件
                let current_file = self.current_file.to_string();
                self.load_file(&current_file);
                println!("撤销");
            }
            b'\x13' => {
                // Ctrl+S
                self.save_file();
                println!("文件已保存");
            }
            _ => {
                *last_key = Some(byte);
            }
        }
        Ok(false)
    }

    fn handle_insert_mode(&mut self, byte: u8) -> Result<bool, core::convert::Infallible> {
        match byte {
            b'\x1b' => {
                // Esc
                println!("返回Normal模式");
                self.mode = EditorMode::Normal;
            }
            b'\r' | b'\n' => {
                self.insert_newline();
            }
            b'\x08' | b'\x7f' => {
                // Backspace
                self.backspace();
            }
            b'\x04' => {
                // Ctrl+D
                println!("退出插入模式");
                return Ok(true);
            }
            _ if byte >= 0x20 && byte <= 0x7e => {
                // 可打印字符
                self.insert_char(byte as char);
            }
            _ => {}
        }
        Ok(false)
    }

    fn handle_command_mode(
        &mut self,
        byte: u8,
        buffer: &mut String,
    ) -> Result<bool, core::convert::Infallible> {
        match byte {
            b'\r' | b'\n' => {
                println!();
                let command = buffer.clone();
                buffer.clear();

                if self.execute_command(&command) {
                    return Ok(true);
                }

                self.mode = EditorMode::Normal;
            }
            b'\x1b' => {
                // Esc
                println!();
                buffer.clear();
                self.mode = EditorMode::Normal;
            }
            b'\x08' | b'\x7f' => {
                // Backspace
                if !buffer.is_empty() {
                    buffer.pop();
                    print!("\x08 \x08");
                }
            }
            _ if byte >= 0x20 && byte <= 0x7e => {
                // 可打印字符
                buffer.push(byte as char);
                print!("{}", byte as char);
            }
            _ => {}
        }
        Ok(false)
    }

    fn execute_command(&mut self, command: &str) -> bool {
        let trimmed = command.trim();
        match trimmed {
            "q" | "quit" => {
                println!("退出编辑器");
                return true;
            }
            "w" | "write" => {
                self.save_file();
                println!("文件已保存");
            }
            "wq" => {
                self.save_file();
                println!("文件已保存并退出");
                return true;
            }
            "e" | "edit" => {
                // 重新加载当前文件
                let current_file = self.current_file.to_string();
                self.load_file(&current_file);
                println!("重新加载文件");
            }
            "ls" => {
                println!("文件列表:");
                let docs = get_documents();
                for filename in docs.keys() {
                    println!("  {}", filename);
                }
            }
            cmd if cmd.starts_with("e ") => {
                let filename = cmd[2..].trim();
                self.current_file = filename.to_string();
                self.load_file(filename);
                println!("编辑文件: {}", filename);
            }
            cmd if cmd.starts_with("w ") => {
                let filename = cmd[2..].trim();
                self.current_file = filename.to_string();
                self.save_file_as(filename);
                println!("另存为: {}", filename);
            }
            _ => {
                println!("未知命令: {}", trimmed);
            }
        }
        false
    }

    fn display_editor(&mut self) -> Result<(), core::convert::Infallible> {
        // 清屏
        let _ = self.manager.display.clear(DisplayColor::new(8, 8, 16));

        // 创建字体样式
        let font = match &self.font {
            Some(font) => font,
            None => return Ok(()),
        };

        let style = FontTextStyleBuilder::new(font.clone())
            .font_size(24) // 大号字体
            .text_color(DisplayColor::WHITE)
            .build();

        // 创建文本框样式
        let textbox_style = TextBoxStyleBuilder::new()
            .alignment(HorizontalAlignment::Left)
            .height_mode(HeightMode::Exact(
                embedded_text::style::VerticalOverdraw::Hidden,
            ))
            .tab_size(TabSize::Spaces(4))
            .build();

        // 计算显示区域（为状态栏留出空间）
        let display_bounds = Rectangle::new(
            Point::new(5 - self.scroll_offset.0, 5 - self.scroll_offset.1),
            Size::new(118, 100),
        );

        // 创建并绘制文本框
        let text_box =
            TextBox::with_textbox_style(&self.content, display_bounds, style, textbox_style);

        let _ = text_box.draw(&mut self.manager.display);

        // 绘制光标
        self.draw_cursor();

        // 绘制状态栏
        self.draw_editor_status_bar();

        // 绘制滚动条
        self.draw_editor_scrollbar();

        Ok(())
    }

    fn draw_cursor(&mut self) {
        use embedded_graphics::primitives::{PrimitiveStyle, Rectangle};

        // 计算光标位置
        let lines: Vec<&str> = self.content.lines().collect();
        if self.cursor_pos.0 < lines.len() {
            let current_line = lines[self.cursor_pos.0];
            let prefix = if self.cursor_pos.1 < current_line.len() {
                &current_line[..self.cursor_pos.1]
            } else {
                current_line
            };

            // 计算光标X位置（简单估算，每个字符12像素宽）
            let cursor_x = 5 + (prefix.chars().count() as i32 * 12) - self.scroll_offset.0;
            let cursor_y = 5 + (self.cursor_pos.0 as i32 * 24) - self.scroll_offset.1;

            // 绘制光标（根据模式不同显示不同样式）
            let cursor_color = match self.mode {
                EditorMode::Insert => DisplayColor::GREEN,
                _ => DisplayColor::CYAN,
            };

            let _ = Rectangle::new(Point::new(cursor_x.max(5), cursor_y), Size::new(2, 24))
                .into_styled(PrimitiveStyle::with_fill(cursor_color))
                .draw(&mut self.manager.display);
        }
    }

    fn draw_editor_status_bar(&mut self) {
        use embedded_graphics::mono_font::{MonoTextStyle, ascii::FONT_6X10};
        use embedded_graphics::primitives::{PrimitiveStyle, Rectangle};

        // 状态栏背景
        let _ = Rectangle::new(Point::new(0, 108), Size::new(128, 20))
            .into_styled(PrimitiveStyle::with_fill(DisplayColor::new(10, 10, 20)))
            .draw(&mut self.manager.display);

        let style = MonoTextStyle::new(&FONT_6X10, DisplayColor::WHITE);

        // 模式指示器
        let mode_text = match self.mode {
            EditorMode::Normal => "NORMAL",
            EditorMode::Insert => "INSERT",
            EditorMode::Command => "COMMAND",
        };

        let _ = Text::new(mode_text, Point::new(2, 110), style).draw(&mut self.manager.display);

        // 文件名
        let filename_text = if self.current_file.len() > 10 {
            format!("{}...", &self.current_file[..7])
        } else {
            self.current_file.clone()
        };

        let _ =
            Text::new(&filename_text, Point::new(50, 110), style).draw(&mut self.manager.display);

        // 光标位置
        let pos_text = format!("{}:{}", self.cursor_pos.0 + 1, self.cursor_pos.1 + 1);

        let _ = Text::new(&pos_text, Point::new(100, 110), style).draw(&mut self.manager.display);

        // 行数统计
        let line_count = self.content.lines().count();
        let lines_text = format!("{}L", line_count);

        let _ = Text::new(&lines_text, Point::new(2, 120), style).draw(&mut self.manager.display);
    }

    fn draw_editor_scrollbar(&mut self) {
        use embedded_graphics::primitives::{PrimitiveStyle, Rectangle};

        let line_count = self.content.lines().count() as i32;
        let visible_lines = 4; // 大约4行24像素字体

        if line_count > visible_lines {
            // 水平滚动条
            let _ = Rectangle::new(Point::new(0, 105), Size::new(128, 2))
                .into_styled(PrimitiveStyle::with_fill(DisplayColor::new(40, 40, 40)))
                .draw(&mut self.manager.display);

            // 垂直滚动条
            let _ = Rectangle::new(Point::new(126, 0), Size::new(2, 108))
                .into_styled(PrimitiveStyle::with_fill(DisplayColor::new(40, 40, 40)))
                .draw(&mut self.manager.display);
        }
    }

    // 编辑器操作函数
    fn load_file(&mut self, filename: &str) {
        let docs = get_documents();
        if let Some(content) = docs.get(filename) {
            self.content = content.clone();
            self.cursor_pos = (0, 0);
            self.scroll_offset = (0, 0);
        }
    }

    fn save_file(&self) {
        let docs = get_documents_mut();
        docs.insert(self.current_file.clone(), self.content.clone());
    }

    fn save_file_as(&self, filename: &str) {
        let docs = get_documents_mut();
        docs.insert(filename.to_string(), self.content.clone());
    }

    fn move_cursor_left(&mut self) {
        if self.cursor_pos.1 > 0 {
            self.cursor_pos.1 -= 1;
        } else if self.cursor_pos.0 > 0 {
            let lines: Vec<&str> = self.content.lines().collect();
            self.cursor_pos.0 -= 1;
            self.cursor_pos.1 = lines[self.cursor_pos.0].chars().count();
        }
    }

    fn move_cursor_right(&mut self) {
        let lines: Vec<&str> = self.content.lines().collect();
        if self.cursor_pos.0 < lines.len() {
            let current_line_len = lines[self.cursor_pos.0].chars().count();
            if self.cursor_pos.1 < current_line_len {
                self.cursor_pos.1 += 1;
            } else if self.cursor_pos.0 + 1 < lines.len() {
                self.cursor_pos.0 += 1;
                self.cursor_pos.1 = 0;
            }
        }
    }

    fn move_cursor_up(&mut self) {
        if self.cursor_pos.0 > 0 {
            self.cursor_pos.0 -= 1;
            let lines: Vec<&str> = self.content.lines().collect();
            let line_len = lines[self.cursor_pos.0].chars().count();
            self.cursor_pos.1 = self.cursor_pos.1.min(line_len);
        }
    }

    fn move_cursor_down(&mut self) {
        let lines: Vec<&str> = self.content.lines().collect();
        if self.cursor_pos.0 + 1 < lines.len() {
            self.cursor_pos.0 += 1;
            let line_len = lines[self.cursor_pos.0].chars().count();
            self.cursor_pos.1 = self.cursor_pos.1.min(line_len);
        }
    }

    fn move_word_forward(&mut self) {
        let lines: Vec<&str> = self.content.lines().collect();
        if self.cursor_pos.0 < lines.len() {
            let line = lines[self.cursor_pos.0];
            let remaining = &line[self.cursor_pos.1..];

            // 找到下一个单词的开始
            let mut in_word = false;
            let mut new_pos = self.cursor_pos.1;

            for (i, ch) in remaining.chars().enumerate() {
                if ch.is_alphanumeric() {
                    if !in_word && i > 0 {
                        new_pos = self.cursor_pos.1 + i;
                        break;
                    }
                    in_word = true;
                } else {
                    in_word = false;
                }
            }

            if new_pos == self.cursor_pos.1 && self.cursor_pos.0 + 1 < lines.len() {
                // 移动到下一行开头
                self.cursor_pos.0 += 1;
                self.cursor_pos.1 = 0;
            } else {
                self.cursor_pos.1 = new_pos;
            }
        }
    }

    fn move_word_backward(&mut self) {
        if self.cursor_pos.1 > 0 {
            let lines: Vec<&str> = self.content.lines().collect();
            let line = lines[self.cursor_pos.0];
            let before: Vec<char> = line[..self.cursor_pos.1].chars().collect();

            // 反向查找单词开始
            let mut last_alpha = None;
            let mut last_non_alpha = None;

            for i in (0..before.len()).rev() {
                if before[i].is_alphanumeric() {
                    if last_non_alpha.is_some() {
                        self.cursor_pos.1 = last_non_alpha.unwrap();
                        return;
                    }
                    last_alpha = Some(i);
                } else if last_alpha.is_some() {
                    last_non_alpha = Some(i + 1);
                }
            }

            if let Some(pos) = last_alpha {
                self.cursor_pos.1 = pos;
            } else {
                self.cursor_pos.1 = 0;
            }
        } else if self.cursor_pos.0 > 0 {
            // 移动到上一行末尾
            let lines: Vec<&str> = self.content.lines().collect();
            self.cursor_pos.0 -= 1;
            self.cursor_pos.1 = lines[self.cursor_pos.0].chars().count();
        }
    }

    fn move_to_line_start(&mut self) {
        self.cursor_pos.1 = 0;
    }

    fn move_to_line_end(&mut self) {
        let lines: Vec<&str> = self.content.lines().collect();
        if self.cursor_pos.0 < lines.len() {
            self.cursor_pos.1 = lines[self.cursor_pos.0].chars().count();
        }
    }

    fn insert_char(&mut self, ch: char) {
        let lines: Vec<&str> = self.content.lines().collect();
        if self.cursor_pos.0 < lines.len() {
            let mut new_lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();
            let line = &mut new_lines[self.cursor_pos.0];

            if self.cursor_pos.1 <= line.chars().count() {
                let char_index = line
                    .char_indices()
                    .nth(self.cursor_pos.1)
                    .map(|(i, _)| i)
                    .unwrap_or(line.len());
                line.insert_str(char_index, &ch.to_string());
                self.cursor_pos.1 += 1;
                self.content = new_lines.join("\n");
            }
        }
    }

    fn insert_newline(&mut self) {
        let lines: Vec<&str> = self.content.lines().collect();
        if self.cursor_pos.0 < lines.len() {
            let mut new_lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();
            let line = &mut new_lines[self.cursor_pos.0];

            if self.cursor_pos.1 <= line.chars().count() {
                let char_index = line
                    .char_indices()
                    .nth(self.cursor_pos.1)
                    .map(|(i, _)| i)
                    .unwrap_or(line.len());
                let remainder = line.split_off(char_index);
                new_lines.insert(self.cursor_pos.0 + 1, remainder);
                self.content = new_lines.join("\n");
                self.cursor_pos.0 += 1;
                self.cursor_pos.1 = 0;
            }
        }
    }

    fn backspace(&mut self) {
        if self.cursor_pos.1 > 0 {
            let lines: Vec<&str> = self.content.lines().collect();
            let mut new_lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();

            let line_idx = self.cursor_pos.0;
            if line_idx < new_lines.len()
                && self.cursor_pos.1 <= new_lines[line_idx].chars().count()
            {
                let line = &mut new_lines[line_idx];
                let char_index = line
                    .char_indices()
                    .nth(self.cursor_pos.1 - 1)
                    .map(|(i, ch)| (i, ch.len_utf8()))
                    .unwrap_or((line.len(), 0));
                line.replace_range(char_index.0..char_index.0 + char_index.1, "");
                self.cursor_pos.1 -= 1;
                self.content = new_lines.join("\n");
            }
        } else if self.cursor_pos.0 > 0 {
            let lines: Vec<&str> = self.content.lines().collect();
            let mut new_lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();

            let current_idx = self.cursor_pos.0;
            let prev_idx = current_idx - 1;

            let current_line = mem::take(&mut new_lines[current_idx]);
            let prev_line = &mut new_lines[prev_idx];
            let prev_len_before = prev_line.chars().count();
            prev_line.push_str(&current_line);
            new_lines.remove(current_idx);
            self.content = new_lines.join("\n");
            self.cursor_pos.0 = prev_idx;
            self.cursor_pos.1 = prev_len_before;
        }
    }

    fn delete_char(&mut self) {
        let lines: Vec<&str> = self.content.lines().collect();
        if self.cursor_pos.0 < lines.len() {
            let mut new_lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();

            if self.cursor_pos.1 < new_lines[self.cursor_pos.0].chars().count() {
                let line = &mut new_lines[self.cursor_pos.0];
                let char_index = line
                    .char_indices()
                    .nth(self.cursor_pos.1)
                    .map(|(i, ch)| (i, ch.len_utf8()))
                    .unwrap_or((line.len(), 0));
                line.replace_range(char_index.0..char_index.0 + char_index.1, "");
            } else if self.cursor_pos.0 + 1 < lines.len() {
                let next_line = new_lines[self.cursor_pos.0 + 1].clone();
                new_lines.remove(self.cursor_pos.0 + 1);
                new_lines[self.cursor_pos.0].push_str(&next_line);
            }

            self.content = new_lines.join("\n");
        }
    }

    fn delete_line(&mut self) {
        let lines: Vec<&str> = self.content.lines().collect();
        let mut new_lines: Vec<String>;
        new_lines = lines.iter().map(|s| s.to_string()).collect();

        if self.cursor_pos.0 < lines.len() {
            new_lines.remove(self.cursor_pos.0);

            if new_lines.is_empty() {
                new_lines.push(String::new());
            }

            if self.cursor_pos.0 >= lines.len() - 1 {
                self.cursor_pos.0 = lines.len() - 2;
            }
            self.cursor_pos.1 = 0;
        }
        self.content = new_lines.join("\n");
    }
}
