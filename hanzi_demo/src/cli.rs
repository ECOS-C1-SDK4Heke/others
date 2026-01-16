#[allow(unused)]
use crate::*;

use embedded_cli::{
    Command,
    arguments::{self, FromArgumentError},
};

#[cfg(feature = "need-ecos")]
pub(crate) struct EbdWriter;

#[cfg(feature = "need-ecos")]
impl embedded_io::ErrorType for EbdWriter {
    type Error = core::convert::Infallible;
}

#[cfg(feature = "need-ecos")]
impl embedded_io::Write for EbdWriter {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        Uart::write_bytes(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[derive(Command)]
pub(crate) enum CmdSample<'a> {
    /// 打印欢迎消息
    Hello {
        /// 要问候的名字 - 可选
        #[arg(short = 'n', long = "name")]
        name: Option<&'a str>,

        /// 重复次数
        #[arg(short = 'c', long = "count")]
        count: Option<u8>,
    },

    /// 普普通通的命令，无参数
    Plain,

    /// 数组操作命令：解析多个数字
    Array {
        /// 多个数字参数（逗号或空格分隔）
        #[arg()]
        numbers: IntArray<'a>,

        /// 操作类型
        #[arg(short = 'o', long = "operation")]
        operation: Option<&'a str>,
    },

    /// 点坐标命令
    Point {
        /// 点坐标，格式：x,y
        #[arg()]
        point: Point,
    },
}

pub(crate) fn handle_sample<'a>(command: CmdSample<'a>) -> Result<(), core::convert::Infallible> {
    match command {
        CmdSample::Hello { name, count } => {
            let name = name.unwrap_or("世界");
            let count = count.unwrap_or(1);

            println!("\r\n你好，{}！", name);

            if count > 1 {
                println!("\n重复 {} 次", count);
                for i in 1..count {
                    println!("\r\n第{}次：你好，{}！", i + 1, name);
                }
            }

            println!("\r\n");
            Ok(())
        }
        CmdSample::Plain => {
            println!("\r\n平平无奇的命令...\r\n");
            Ok(())
        }
        CmdSample::Array { numbers, operation } => {
            println!("数组操作:");
            println!("  原始输入: {}", numbers.numbers);
            println!("  解析的数字 ({}个):", numbers.len());

            for (i, num) in numbers.iter().enumerate() {
                println!("    [{}]: {}", i + 1, num);
            }

            let op = operation.unwrap_or("sum");
            match op {
                "sum" => {
                    if !numbers.is_empty() {
                        let sum: i32 = numbers.iter().sum();
                        println!("  总和: {}", sum);
                    }
                }
                "avg" => {
                    if !numbers.is_empty() {
                        let sum: i32 = numbers.iter().sum();
                        let avg = sum as f32 / numbers.len() as f32;
                        println!("  平均值: {:.2}", avg);
                    }
                }
                "min" => {
                    if !numbers.is_empty() {
                        if let Some(min) = numbers.iter().min() {
                            println!("  最小值: {}", min);
                        }
                    }
                }
                "max" => {
                    if !numbers.is_empty() {
                        if let Some(max) = numbers.iter().max() {
                            println!("  最大值: {}", max);
                        }
                    }
                }
                _ => println!("  未知操作: {}", op),
            };
            Ok(())
        }
        CmdSample::Point { point } => {
            println!("点坐标:");
            println!("  X = {:.2}", point.x);
            println!("  Y = {:.2}", point.y);
            #[allow(unused)] // 硬件真实环境需要
            use micromath::F32Ext;
            println!(
                "  距离原点: {:.2}",
                (point.x * point.x + point.y * point.y).sqrt()
            );
            Ok(())
        }
    }
}

// 自定义数组类型：解析逗号分隔的数字列表
pub(crate) struct IntArray<'a> {
    numbers: &'a str,  // 原始字符串
    parsed: [i32; 10], // 最多10个数字
    len: usize,        // 实际数量
}

impl<'a> IntArray<'a> {
    fn iter(&self) -> impl Iterator<Item = i32> + '_ {
        self.parsed[0..self.len].iter().copied()
    }

    fn len(&self) -> usize {
        self.len
    }

    fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl<'a> arguments::FromArgument<'a> for IntArray<'a> {
    fn from_arg(arg: &'a str) -> Result<Self, FromArgumentError<'a>> {
        let mut parsed = [0i32; 10];
        let mut len = 0;

        // 支持逗号分隔
        let parts: Vec<&str> = if arg.contains(',') {
            arg.split(',').collect()
        } else {
            arg.split_whitespace().collect()
        };

        for (i, part) in parts.iter().enumerate() {
            if i >= 10 {
                return Err(FromArgumentError {
                    value: arg,
                    expected: "参数数目最大为10",
                });
            }

            let trimmed = part.trim();
            if trimmed.is_empty() {
                continue;
            }

            parsed[len] = trimmed.parse().map_err(|_| FromArgumentError {
                value: trimmed,
                expected: "无效的数字格式",
            })?;
            len += 1;
        }

        if len == 0 {
            return Err(FromArgumentError {
                value: arg,
                expected: "至少提供一个参数",
            });
        }

        Ok(IntArray {
            numbers: arg,
            parsed,
            len,
        })
    }
}

// 点坐标类型
pub(crate) struct Point {
    x: f32,
    y: f32,
}

impl<'a> arguments::FromArgument<'a> for Point {
    fn from_arg(arg: &'a str) -> Result<Self, FromArgumentError<'a>> {
        let parts: Vec<&str> = arg.split(',').collect();
        if parts.len() != 2 {
            return Err(FromArgumentError {
                value: arg,
                expected: "点坐标格式应该是x,y",
            });
        }

        let x = parts[0].trim().parse().map_err(|_| FromArgumentError {
            value: parts[0],
            expected: "无效的X",
        })?;
        let y = parts[1].trim().parse().map_err(|_| FromArgumentError {
            value: parts[1],
            expected: "无效的Y",
        })?;

        Ok(Point { x, y })
    }
}
