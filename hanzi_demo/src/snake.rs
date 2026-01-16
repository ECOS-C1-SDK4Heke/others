//! ebd-snake v0.0.2 的依赖太老了，就直接把源码拿来适配新的了 ...

use crate::*;

#[cfg(feature = "need-ecos")]
use ecos_ssc1::bindings;
use embedded_cli::Command;
use embedded_graphics::prelude::*;
#[allow(unused)] // 硬件真实环境需要
use embedded_hal::delay::DelayNs;
use rand::{Rng, SeedableRng, rngs::SmallRng};

// 贪吃蛇命令定义
#[derive(Command, Debug)]
pub(crate) enum SnakeSample<'a> {
    /// 启动贪吃蛇游戏
    #[command(name = "snake")]
    Start,

    /// 设置游戏难度
    #[command(name = "snake-difficulty")]
    Difficulty {
        /// 难度级别 (easy, normal, hard)
        #[arg(short = 'l', long = "level")]
        level: Option<&'a str>,

        /// 游戏速度 (1-10)
        #[arg(short = 's', long = "speed")]
        speed: Option<u8>,
    },

    /// 显示游戏帮助
    #[command(name = "snake-help")]
    Help,

    /// 设置随机种子
    #[command(name = "snake-seed")]
    Seed {
        /// 随机种子 (十六进制)
        #[arg()]
        seed: u64,
    },
}

// 贪吃蛇游戏处理函数
pub(crate) fn handle_snake_sample<'a, T>(
    manager: &mut DisplayManager,
    command: SnakeSample<'a>,
) -> Result<(), core::convert::Infallible> {
    match command {
        SnakeSample::Start => {
            println!("\r\n=== 启动贪吃蛇游戏 ===");
            start_snake_game(manager)
        }
        SnakeSample::Difficulty { level, speed } => {
            println!("\r\n=== 设置游戏难度 ===");
            set_game_difficulty(level, speed)
        }
        SnakeSample::Help => {
            println!("\r\n=== 贪吃蛇游戏帮助 ===");
            show_snake_help()
        }
        SnakeSample::Seed { seed } => {
            println!("\r\n=== 设置随机种子 ===");
            set_random_seed(seed)
        }
    }
}

// ===========================================
// HSL颜色表示和转换
// ============================================

#[derive(Debug, Clone, Copy)]
struct Hsl {
    h: f32, // 色调 0-360
    s: f32, // 饱和度 0-1
    l: f32, // 亮度 0-1
}

impl Hsl {
    fn new(h: f32, s: f32, l: f32) -> Self {
        Self {
            h: h.max(0.0).min(360.0),
            s: s.max(0.0).min(1.0),
            l: l.max(0.0).min(1.0),
        }
    }

    // HSL转RGB
    fn to_rgb(&self) -> DisplayColor {
        let (r, g, b) = self.to_rgb888();
        DisplayColor::new(
            (r as u32 * 31 / 255) as u8,
            (g as u32 * 63 / 255) as u8,
            (b as u32 * 31 / 255) as u8,
        )
    }

    fn to_rgb888(&self) -> (u8, u8, u8) {
        let h = self.h / 360.0;
        let s = self.s;
        let l = self.l;

        let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
        let x = c * (1.0 - ((h * 6.0) % 2.0 - 1.0).abs());
        let m = l - c / 2.0;

        let (r1, g1, b1) = if h < 1.0 / 6.0 {
            (c, x, 0.0)
        } else if h < 2.0 / 6.0 {
            (x, c, 0.0)
        } else if h < 3.0 / 6.0 {
            (0.0, c, x)
        } else if h < 4.0 / 6.0 {
            (0.0, x, c)
        } else if h < 5.0 / 6.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };

        let r = ((r1 + m) * 255.0) as u8;
        let g = ((g1 + m) * 255.0) as u8;
        let b = ((b1 + m) * 255.0) as u8;

        (r, g, b)
    }

    // 计算两个HSL颜色的角度差
    fn angle_difference(&self, other: &Hsl) -> f32 {
        let diff = (self.h - other.h).abs();
        diff.min(360.0 - diff) // 取最短弧
    }

    // 生成与给定颜色有足够角度差的随机颜色
    fn random_with_min_difference(rng: &mut SmallRng, base: &Hsl, min_diff: f32) -> Hsl {
        let mut attempts = 0;
        loop {
            let h = rng.random_range(0.0..360.0);
            let s = rng.random_range(0.5..0.9); // 较高饱和度保证鲜艳
            let l = rng.random_range(0.4..0.7); // 中等亮度

            let new_color = Hsl::new(h, s, l);
            if new_color.angle_difference(base) >= min_diff {
                return new_color;
            }

            attempts += 1;
            if attempts > 100 {
                // 如果找不到足够差异的颜色，强制旋转180度
                return Hsl::new((base.h + 180.0) % 360.0, s, l);
            }
        }
    }

    // 调整亮度
    fn lighter(&self, amount: f32) -> Hsl {
        Hsl::new(self.h, self.s, (self.l + amount).min(0.9).max(0.1))
    }
}

// ===========================================
// 从源码复制过来的结构体和实现（增强版）
// ============================================

use embedded_graphics_core::{draw_target::DrawTarget, primitives::Rectangle};

#[derive(PartialEq, Debug, Clone, Copy)]
pub(crate) enum Direction {
    Left,
    Right,
    Up,
    Down,
    None,
}

struct Snake<T: PixelColor, const MAX_SIZE: usize> {
    parts: [Pixel<T>; MAX_SIZE],
    len: usize,
    direction: Direction,
    size_x: u8,
    size_y: u8,
}

struct SnakeIntoIterator<'a, T: PixelColor, const MAX_SIZE: usize> {
    snake: &'a Snake<T, MAX_SIZE>,
    index: usize,
}

impl<'a, T: PixelColor, const MAX_SIZE: usize> IntoIterator for &'a Snake<T, MAX_SIZE> {
    type Item = Pixel<T>;
    type IntoIter = SnakeIntoIterator<'a, T, MAX_SIZE>;

    fn into_iter(self) -> Self::IntoIter {
        SnakeIntoIterator {
            snake: self,
            index: 0,
        }
    }
}

impl<'a, T: PixelColor, const MAX_SIZE: usize> Iterator for SnakeIntoIterator<'a, T, MAX_SIZE> {
    type Item = Pixel<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.snake.len {
            let cur = self.snake.parts[self.index];
            self.index += 1;
            return Some(cur);
        }
        None
    }
}

impl<T: PixelColor, const MAX_SIZE: usize> Snake<T, MAX_SIZE> {
    fn new(color: T, size_x: u8, size_y: u8) -> Snake<T, MAX_SIZE> {
        // 初始化蛇在屏幕中央
        let mut parts = [Pixel::<T>(Point { x: 0, y: 0 }, color); MAX_SIZE];
        let initial_x = (size_x as i32) / 2;
        let initial_y = (size_y as i32) / 2;

        for i in 0..5 {
            parts[i] = Pixel::<T>(
                Point {
                    x: initial_x - i as i32,
                    y: initial_y,
                },
                color,
            );
        }

        Snake {
            parts,
            len: 5,
            direction: Direction::Right,
            size_x,
            size_y,
        }
    }

    fn set_direction(&mut self, direction: Direction) {
        // 防止直接反向移动
        match direction {
            Direction::Left if self.direction != Direction::Right => self.direction = direction,
            Direction::Right if self.direction != Direction::Left => self.direction = direction,
            Direction::Up if self.direction != Direction::Down => self.direction = direction,
            Direction::Down if self.direction != Direction::Up => self.direction = direction,
            Direction::None => {} // 不做任何操作
            _ => {}               // 其他情况保持不变
        }
    }

    fn grow(&mut self) {
        if self.len < MAX_SIZE - 1 {
            self.len += 1;
        }
    }

    fn make_step(&mut self) {
        let mut i = self.len;
        while i > 0 {
            self.parts[i] = self.parts[i - 1];
            i -= 1;
        }

        match self.direction {
            Direction::Left => {
                if self.parts[0].0.x == 0 {
                    self.parts[0].0.x = (self.size_x - 1) as i32;
                } else {
                    self.parts[0].0.x -= 1;
                }
            }
            Direction::Right => {
                if self.parts[0].0.x == (self.size_x - 1) as i32 {
                    self.parts[0].0.x = 0;
                } else {
                    self.parts[0].0.x += 1;
                }
            }
            Direction::Up => {
                if self.parts[0].0.y == 0 {
                    self.parts[0].0.y = (self.size_y - 1) as i32;
                } else {
                    self.parts[0].0.y -= 1;
                }
            }
            Direction::Down => {
                if self.parts[0].0.y == (self.size_y - 1) as i32 {
                    self.parts[0].0.y = 0;
                } else {
                    self.parts[0].0.y += 1;
                }
            }
            Direction::None => {}
        }
    }

    fn check_collision(&self) -> bool {
        let head = self.parts[0].0;

        // 检查是否撞墙
        if head.x < 0 || head.x >= self.size_x as i32 || head.y < 0 || head.y >= self.size_y as i32
        {
            return true;
        }

        // 检查是否撞到自己（从第二个开始检查，因为第一个是头）
        for i in 1..self.len {
            if self.parts[i].0 == head {
                return true;
            }
        }

        false
    }
}

// 食物状态机
#[derive(Clone, Copy)]
enum FoodAnimationState {
    Shrinking { scale: f32, direction: f32 }, // direction: -1缩小, 1放大
    Growing { scale: f32, direction: f32 },
    Static { scale: f32 },
}

impl FoodAnimationState {
    fn new() -> Self {
        Self::Static { scale: 1.0 }
    }

    fn update(&mut self) -> f32 {
        match *self {
            Self::Shrinking {
                ref mut scale,
                ref direction,
            } => {
                *scale += 0.05 * direction;
                if *scale <= 0.85 {
                    return Self::Growing {
                        scale: 0.85,
                        direction: 1.0,
                    }
                    .update();
                }
                *scale
            }
            Self::Growing {
                ref mut scale,
                ref direction,
            } => {
                *scale += 0.05 * direction;
                if *scale >= 1.15 {
                    return Self::Shrinking {
                        scale: 1.15,
                        direction: -1.0,
                    }
                    .update();
                }
                *scale
            }
            Self::Static { scale } => scale,
        }
    }

    fn current_scale(&self) -> f32 {
        match *self {
            Self::Shrinking { scale, .. } => scale,
            Self::Growing { scale, .. } => scale,
            Self::Static { scale } => scale,
        }
    }
}

struct Food {
    size_x: u8,
    size_y: u8,
    place: Pixel<DisplayColor>,
    color_hsl: Hsl,
    animation_state: FoodAnimationState,
    base_score: u32,
}

impl Food {
    fn new(color_hsl: Hsl, size_x: u8, size_y: u8) -> Self {
        Food {
            size_x,
            size_y,
            place: Pixel(Point { x: 0, y: 0 }, color_hsl.to_rgb()),
            color_hsl,
            animation_state: FoodAnimationState::new(),
            base_score: 10, // 基础分数
        }
    }

    fn replace<const MAX_SIZE: usize>(
        &mut self,
        snake: &Snake<DisplayColor, MAX_SIZE>,
        rng: &mut SmallRng,
        background_hsl: &Hsl,
        snake_hsl: &Hsl,
    ) {
        let mut p: Point;
        'outer: loop {
            p = Point {
                x: rng.random_range(0..self.size_x) as i32,
                y: rng.random_range(0..self.size_y) as i32,
            };

            for part in snake.into_iter() {
                if p == part.0 {
                    continue 'outer;
                }
            }
            break;
        }

        // 生成与背景和蛇都有足够差异的食物颜色
        let mut food_hsl;
        let mut attempts = 0;

        loop {
            // 生成随机颜色
            food_hsl = Hsl::new(
                rng.random_range(0.0..360.0),
                rng.random_range(0.7..0.95), // 高饱和度
                rng.random_range(0.5..0.8),  // 中等亮度
            );

            // 计算与背景和蛇的角度差
            let diff_to_bg = food_hsl.angle_difference(background_hsl);
            let diff_to_snake = food_hsl.angle_difference(snake_hsl);

            // 确保与两者都有足够差异
            if diff_to_bg >= 30.0 && diff_to_snake >= 30.0 {
                break;
            }

            attempts += 1;
            if attempts > 50 {
                // 如果找不到合适的颜色，强制旋转
                food_hsl = Hsl::new((background_hsl.h + 180.0) % 360.0, food_hsl.s, food_hsl.l);
                break;
            }
        }

        // 根据颜色差异计算分数加成
        let angle_score = (food_hsl.angle_difference(background_hsl) / 360.0 * 20.0) as u32;
        self.base_score = 10 + angle_score; // 基础分10 + 角度加成

        self.place = Pixel::<DisplayColor> {
            0: p,
            1: food_hsl.to_rgb(),
        };
        self.color_hsl = food_hsl;
        self.animation_state = FoodAnimationState::new();
    }

    fn get_pixel(&self) -> Pixel<DisplayColor> {
        self.place
    }

    fn get_score(&self) -> u32 {
        self.base_score
    }

    fn update_animation(&mut self) -> f32 {
        self.animation_state.update()
    }

    fn current_scale(&self) -> f32 {
        self.animation_state.current_scale()
    }
}

pub struct SnakeGame<const MAX_SNAKE_SIZE: usize> {
    snake: Snake<DisplayColor, MAX_SNAKE_SIZE>,
    food: Food,
    rng: SmallRng,
    food_age: u8,
    food_lifetime: u8,
    size_x: u8,
    size_y: u8,
    scale_x: u8,
    scale_y: u8,
    score: u32,
    game_over: bool,
    last_tail: Option<Pixel<DisplayColor>>,
    waiting_for_start: bool,
    background_hsl: Hsl,
    snake_hsl: Hsl,
    last_bg_change: u32,
    bg_change_interval: u32,
}

impl<const MAX_SIZE: usize> SnakeGame<MAX_SIZE> {
    pub fn new(
        size_x: u8,
        size_y: u8,
        scale_x: u8,
        scale_y: u8,
        seed: u64,
        food_lifetime: u8,
    ) -> Self {
        let mut rng = SmallRng::seed_from_u64(seed);

        // 生成初始背景色
        let background_hsl = Hsl::new(
            rng.random_range(0.0..360.0),
            rng.random_range(0.1..0.3),   // 低饱和度背景
            rng.random_range(0.05..0.15), // 暗色背景
        );

        // 生成与背景有足够差异的蛇颜色
        let snake_hsl = Hsl::random_with_min_difference(&mut rng, &background_hsl, 60.0);
        let snake_color = snake_hsl.to_rgb();

        let snake =
            Snake::<DisplayColor, MAX_SIZE>::new(snake_color, size_x / scale_x, size_y / scale_y);

        // 先创建游戏对象，然后再设置食物
        let mut game = SnakeGame {
            snake,
            food: Food::new(Hsl::new(0.0, 0.0, 0.0), size_x / scale_x, size_y / scale_y), // 临时颜色
            rng,
            food_age: 0,
            food_lifetime,
            size_x,
            size_y,
            scale_x,
            scale_y,
            score: 0,
            game_over: false,
            last_tail: None,
            waiting_for_start: true,
            background_hsl,
            snake_hsl,
            last_bg_change: 0,
            bg_change_interval: 50, // 临时值，后面会更新
        };

        // 更新bg_change_interval
        game.bg_change_interval = game.rng.random_range(50..150); // 每50-150帧变化一次

        // 设置食物
        game.food.replace(
            &game.snake,
            &mut game.rng,
            &game.background_hsl,
            &game.snake_hsl,
        );
        game
    }

    pub fn set_direction(&mut self, direction: Direction) {
        if !self.waiting_for_start {
            self.snake.set_direction(direction);
        } else {
            // 如果正在等待开始，按任意键（除了Q）开始游戏
            self.waiting_for_start = false;
            self.snake.set_direction(direction);
        }
    }

    pub fn update(&mut self, frame_count: u32) {
        if self.game_over || self.waiting_for_start {
            return;
        }

        // 定期改变背景色
        if frame_count - self.last_bg_change > self.bg_change_interval {
            self.background_hsl = Hsl::new(
                self.rng.random_range(0.0..360.0),
                self.rng.random_range(0.1..0.3),
                self.rng.random_range(0.05..0.15),
            );
            self.last_bg_change = frame_count;
            self.bg_change_interval = self.rng.random_range(50..150);
        }

        // 保存尾部位置，用于局部刷新
        if self.snake.len > 0 {
            self.last_tail = Some(self.snake.parts[self.snake.len - 1]);
        }

        // 移动蛇
        self.snake.make_step();

        // 检查碰撞
        if self.snake.check_collision() {
            self.game_over = true;
            return;
        }

        // 检查是否吃到食物
        let head = self.snake.parts[0].0;
        let food_pos = self.food.get_pixel().0;

        if head == food_pos {
            // 吃到食物，根据颜色差异计算分数
            let food_score = self.food.get_score();
            self.snake.grow();
            self.score += food_score;
            self.food.replace(
                &self.snake,
                &mut self.rng,
                &self.background_hsl,
                &self.snake_hsl,
            );
            self.food_age = 0;
        } else {
            // 更新食物年龄
            self.food_age += 1;
            if self.food_age >= self.food_lifetime {
                self.food.replace(
                    &self.snake,
                    &mut self.rng,
                    &self.background_hsl,
                    &self.snake_hsl,
                );
                self.food_age = 0;
            }
        }

        // 更新食物动画
        self.food.update_animation();
    }

    pub fn draw<D: DrawTarget<Color = DisplayColor>>(&mut self, target: &mut D) {
        // 绘制背景（如果背景色变化了）
        if self.last_bg_change == 0 {
            // 初次绘制
            let _ = target.clear(self.background_hsl.to_rgb());
        }

        let mut scaled_display = ScaledDisplay {
            target,
            size_x: self.size_x / self.scale_x,
            size_y: self.size_y / self.scale_y,
            scale_x: self.scale_x,
            scale_y: self.scale_y,
        };

        if self.waiting_for_start {
            // 显示等待开始提示
            use embedded_graphics::{
                mono_font::{MonoTextStyle, ascii::FONT_6X9},
                text::Text,
            };

            let text_color = if self.background_hsl.l > 0.5 {
                DisplayColor::BLACK
            } else {
                DisplayColor::WHITE
            };

            let style = MonoTextStyle::new(&FONT_6X9, text_color);
            let _ = Text::new("按任意键开始", Point::new(30, 60), style).draw(&mut scaled_display);
            return;
        }

        if self.game_over {
            return;
        }

        // 局部刷新：清除旧的尾部
        if let Some(old_tail) = self.last_tail {
            let style = embedded_graphics::primitives::PrimitiveStyle::with_fill(
                self.background_hsl.to_rgb(),
            );
            Rectangle::new(
                Point::new(
                    old_tail.0.x * self.scale_x as i32,
                    old_tail.0.y * self.scale_y as i32,
                ),
                embedded_graphics::prelude::Size::new(self.scale_x as u32, self.scale_y as u32),
            )
            .into_styled(style)
            .draw(scaled_display.target)
            .ok();
        }

        // 绘制蛇
        for i in 0..self.snake.len {
            let part = self.snake.parts[i];
            let color = if i == 0 {
                // 蛇头用亮色
                let head_hsl = self.snake_hsl.lighter(0.2);
                head_hsl.to_rgb()
            } else {
                // 蛇身用普通颜色
                part.1
            };

            let snake_pixel = Pixel(part.0, color);
            let _ = snake_pixel.draw(&mut scaled_display);
        }

        // 绘制食物（带缩放）
        let food_pixel = self.food.get_pixel();
        let food_scale = self.food.current_scale();

        // 计算缩放后的食物大小和位置
        let food_x = food_pixel.0.x as f32;
        let food_y = food_pixel.0.y as f32;
        let scaled_size = (self.scale_x as f32 * food_scale) as u32;

        if scaled_size > 0 {
            let style = embedded_graphics::primitives::PrimitiveStyle::with_fill(food_pixel.1);
            let rect_x = (food_x * self.scale_x as f32
                + self.scale_x as f32 * (1.0 - food_scale) / 2.0) as i32;
            let rect_y = (food_y * self.scale_y as f32
                + self.scale_y as f32 * (1.0 - food_scale) / 2.0) as i32;

            Rectangle::new(
                Point::new(rect_x, rect_y),
                embedded_graphics::prelude::Size::new(scaled_size, scaled_size),
            )
            .into_styled(style)
            .draw(scaled_display.target)
            .ok();
        }
    }

    pub fn is_game_over(&self) -> bool {
        self.game_over
    }

    pub fn is_waiting_for_start(&self) -> bool {
        self.waiting_for_start
    }

    pub fn get_score(&self) -> u32 {
        self.score
    }

    pub fn reset(&mut self, seed: u64) {
        self.rng = SmallRng::seed_from_u64(seed);

        // 生成新的背景色
        self.background_hsl = Hsl::new(
            self.rng.random_range(0.0..360.0),
            self.rng.random_range(0.1..0.3),
            self.rng.random_range(0.05..0.15),
        );

        // 生成新的蛇颜色
        self.snake_hsl = Hsl::random_with_min_difference(&mut self.rng, &self.background_hsl, 60.0);
        let snake_color = self.snake_hsl.to_rgb();

        self.snake = Snake::<DisplayColor, MAX_SIZE>::new(
            snake_color,
            self.size_x / self.scale_x,
            self.size_y / self.scale_y,
        );

        self.food.replace(
            &self.snake,
            &mut self.rng,
            &self.background_hsl,
            &self.snake_hsl,
        );
        self.food_age = 0;
        self.score = 0;
        self.game_over = false;
        self.last_tail = None;
        self.waiting_for_start = true;
        self.last_bg_change = 0;
        self.bg_change_interval = self.rng.random_range(50..150);
    }
}

/// 缩放显示适配器
struct ScaledDisplay<'a, T: DrawTarget> {
    target: &'a mut T,
    size_x: u8,
    size_y: u8,
    scale_x: u8,
    scale_y: u8,
}

impl<'a, T: DrawTarget> DrawTarget for ScaledDisplay<'a, T> {
    type Color = T::Color;
    type Error = T::Error;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for pixel in pixels {
            let style = embedded_graphics::primitives::PrimitiveStyle::with_fill(pixel.1);
            Rectangle::new(
                Point::new(
                    pixel.0.x * self.scale_x as i32,
                    pixel.0.y * self.scale_y as i32,
                ),
                embedded_graphics::prelude::Size::new(self.scale_x as u32, self.scale_y as u32),
            )
            .into_styled(style)
            .draw(self.target)?;
        }
        Ok(())
    }
}

impl<'a, T: DrawTarget> embedded_graphics_core::geometry::Dimensions for ScaledDisplay<'a, T> {
    fn bounding_box(&self) -> embedded_graphics::primitives::Rectangle {
        embedded_graphics::primitives::Rectangle::new(
            Point::new(0, 0),
            embedded_graphics::prelude::Size::new(self.size_x as u32, self.size_y as u32),
        )
    }
}

// ===========================================
// 游戏状态管理（简化版）
// ============================================

struct SnakeGameState {
    game: SnakeGame<200>,
    level: u8,
    is_paused: bool,
    high_score: u32,
    current_seed: u64,
    update_interval: u32,  // 更新间隔（毫秒）
    last_update_time: u64, // 上次更新时间
    frame_counter: u32,    // 简单的帧计数器
}

impl SnakeGameState {
    fn new(seed: u64, speed: u32) -> Self {
        let game = SnakeGame::<200>::new(
            128, // 显示宽度
            128, // 显示高度
            2,   // 横向缩放
            2,   // 纵向缩放
            seed, 100, // 食物寿命
        );

        Self {
            game,
            level: 1,
            is_paused: false,
            high_score: 0,
            current_seed: seed,
            update_interval: speed.max(50), // 最小50ms
            last_update_time: 0,
            frame_counter: 0,
        }
    }

    fn update(&mut self, current_time: u64) {
        if self.is_paused || self.game.is_waiting_for_start() {
            return;
        }

        // 检查是否到了更新时间
        if current_time < self.last_update_time + self.update_interval as u64 {
            return; // 还没到更新时间
        }

        self.last_update_time = current_time;
        self.frame_counter += 1;

        // 更新游戏状态
        self.game.update(self.frame_counter);

        // 更新等级
        let new_level = (self.game.get_score() / 100) as u8 + 1;
        if new_level > self.level {
            self.level = new_level;
            println!("🎮 升级到 Level {}!", self.level);
        }
    }

    fn draw<D: DrawTarget<Color = DisplayColor>>(&mut self, display: &mut D) {
        // 绘制游戏
        self.game.draw(display);

        // 绘制UI
        use embedded_graphics::{
            mono_font::{MonoTextStyle, ascii::FONT_6X9},
            primitives::{PrimitiveStyle, Rectangle},
            text::Text,
        };

        // 清除UI区域
        let ui_bg = DisplayColor::new(0, 0, 0);
        let ui_rect = Rectangle::new(Point::new(0, 0), Size::new(128, 20))
            .into_styled(PrimitiveStyle::with_fill(ui_bg));
        let _ = ui_rect.draw(display);

        let style = MonoTextStyle::new(&FONT_6X9, DisplayColor::WHITE);

        // 绘制分数
        let score_text = format!("S:{}", self.game.get_score());
        let _ = Text::new(&score_text, Point::new(2, 8), style).draw(display);

        // 绘制等级
        let level_text = format!("L:{}", self.level);
        let _ = Text::new(&level_text, Point::new(50, 8), style).draw(display);

        // 绘制最高分
        let high_text = format!("H:{}", self.high_score);
        let _ = Text::new(&high_text, Point::new(98, 8), style).draw(display);

        // 显示特殊状态
        if self.game.is_waiting_for_start() {
            let _ = Text::new("按任意键开始", Point::new(30, 60), style).draw(display);
        } else if self.is_paused {
            let pause_rect = Rectangle::new(Point::new(30, 50), Size::new(70, 30))
                .into_styled(PrimitiveStyle::with_fill(DisplayColor::BLACK));
            let _ = pause_rect.draw(display);
            let _ = Text::new("PAUSED", Point::new(40, 60), style).draw(display);
        } else if self.game.is_game_over() {
            let game_over_rect = Rectangle::new(Point::new(10, 50), Size::new(110, 50))
                .into_styled(PrimitiveStyle::with_fill(DisplayColor::BLACK));
            let _ = game_over_rect.draw(display);
            let _ = Text::new("GAME OVER", Point::new(30, 60), style).draw(display);
            let _ = Text::new("Press R to restart", Point::new(10, 80), style).draw(display);
        }
    }

    fn handle_input(&mut self, byte: u8) -> bool {
        match byte {
            b'w' | b'W' | b'i' | b'I' => {
                self.game.set_direction(Direction::Up);
                false
            }
            b's' | b'S' | b'k' | b'K' => {
                self.game.set_direction(Direction::Down);
                false
            }
            b'a' | b'A' | b'j' | b'J' => {
                self.game.set_direction(Direction::Left);
                false
            }
            b'd' | b'D' | b'l' | b'L' => {
                self.game.set_direction(Direction::Right);
                false
            }
            b' ' => {
                if !self.game.is_waiting_for_start() {
                    self.is_paused = !self.is_paused;
                    println!("游戏 {}", if self.is_paused { "暂停" } else { "继续" });
                }
                false
            }
            b'r' | b'R' => {
                if self.game.is_game_over() {
                    let final_score = self.game.get_score();
                    if final_score > self.high_score {
                        self.high_score = final_score;
                        println!("🎉 新纪录！最高分: {}", self.high_score);
                    }
                    self.game.reset(self.current_seed);
                    self.level = 1;
                    self.is_paused = false;
                    self.frame_counter = 0;
                    println!("重新开始游戏...");
                }
                false
            }
            b'q' | b'Q' => {
                println!("退出贪吃蛇游戏");
                if self.game.get_score() > self.high_score {
                    self.high_score = self.game.get_score();
                }
                true
            }
            _ => {
                if self.game.is_waiting_for_start() {
                    self.game.set_direction(Direction::None);
                }
                false
            }
        }
    }

    fn get_score(&self) -> u32 {
        self.game.get_score()
    }

    fn get_level(&self) -> u8 {
        self.level
    }
}

// 启动贪吃蛇游戏
fn start_snake_game(manager: &mut DisplayManager) -> Result<(), core::convert::Infallible> {
    println!("\r\n=== 贪吃蛇游戏开始 ===");
    println!("游戏特性:");
    println!("  • 随机变化的背景颜色");
    println!("  • 动态缩放的食物 (0.85-1.15倍)");
    println!("  • 颜色差异越大，分数越高");
    println!("");
    println!("游戏控制:");
    println!("  W/A/S/D 或 I/J/K/L - 控制方向");
    println!("  空格键              - 暂停/继续");
    println!("  R                  - 重新开始");
    println!("  Q                  - 退出游戏");
    println!("======================\r\n");

    // 创建游戏状态 - 200ms更新一次
    let mut game_state = SnakeGameState::new(0x10086, 200);

    // 获取初始时间（毫秒）
    #[cfg(feature = "target-ui-sim")]
    let start_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    #[cfg(feature = "need-ecos")]
    let start_time = unsafe { bindings::get_sys_tick() } as u64;

    let mut last_score = 0;

    loop {
        // 获取当前时间
        #[cfg(feature = "target-ui-sim")]
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
            - start_time;

        #[cfg(feature = "need-ecos")]
        let current_time = unsafe { bindings::get_sys_tick() } as u64 - start_time;

        // 处理用户输入
        if let Some(byte) = crate::Uart::read_byte_nonblock() {
            if game_state.handle_input(byte) {
                break;
            }
        }

        // 更新游戏状态
        game_state.update(current_time);

        // 绘制游戏
        game_state.draw(&mut manager.display);

        // 更新窗口显示
        #[cfg(feature = "target-ui-sim")]
        {
            manager.update_window();
        }

        // 显示分数变化
        let current_score = game_state.get_score();
        if current_score != last_score {
            println!(
                "当前得分: {}, 等级: {}",
                current_score,
                game_state.get_level()
            );
            last_score = current_score;
        }

        // 控制帧率（大约60FPS）
        #[cfg(feature = "target-ui-sim")]
        std::thread::sleep(std::time::Duration::from_millis(16));
        #[cfg(not(feature = "target-ui-sim"))]
        manager.delay.delay_ms(16);
    }

    // 显示最终结果
    println!("\r\n游戏结束！");
    println!("最终得分: {}", game_state.get_score());
    println!("最高分: {}", game_state.high_score);
    println!("达到等级: {}", game_state.get_level());
    println!("======================\r\n");

    // 最后清屏
    let _ = manager.display.clear(DisplayColor::BLACK);

    Ok(())
}

// 设置游戏难度
fn set_game_difficulty<'a>(
    level: Option<&'a str>,
    speed: Option<u8>,
) -> Result<(), core::convert::Infallible> {
    println!("当前设置:");

    if let Some(lvl) = level {
        match lvl.to_lowercase().as_str() {
            "easy" => println!("  难度: 简单 (速度慢，颜色变化平缓)"),
            "normal" => println!("  难度: 普通 (速度中等，颜色变化正常)"),
            "hard" => println!("  难度: 困难 (速度快，颜色变化频繁)"),
            _ => println!("  难度: 未知 (使用 easy/normal/hard)"),
        }
    } else {
        println!("  难度: 未设置");
    }

    if let Some(spd) = speed {
        if spd >= 1 && spd <= 10 {
            let actual_speed = 250 - (spd as u32 * 20); // 1=230ms, 10=50ms
            println!("  速度: {} ({}ms/帧)", spd, actual_speed);
        } else {
            println!("  速度: 无效 (应为 1-10)");
        }
    } else {
        println!("  速度: 未设置");
    }

    println!("\r\n使用示例:");
    println!("  snake-difficulty --level easy --speed 3");
    println!("  snake-difficulty -l normal -s 6");

    Ok(())
}

// 显示游戏帮助
fn show_snake_help() -> Result<(), core::convert::Infallible> {
    println!("贪吃蛇游戏说明:");
    println!("================");
    println!("游戏特性:");
    println!("  • 背景颜色随机变化");
    println!("  • 蛇颜色与背景保持60°以上差异");
    println!("  • 食物颜色与背景/蛇都保持30°以上差异");
    println!("  • 食物动态缩放 (0.85-1.15倍)");
    println!("  • 颜色差异越大，分数加成越高");
    println!("  • 状态机模式，流畅动画");
    println!("");
    println!("基本命令:");
    println!("  snake            - 开始游戏");
    println!("  snake-difficulty - 设置游戏难度和速度");
    println!("  snake-seed       - 设置随机种子");
    println!("  snake-help       - 显示此帮助信息");
    println!("");
    println!("游戏内控制:");
    println!("  任意键           - 开始游戏（等待状态时）");
    println!("  W/A/S/D 或 I/J/K/L - 控制蛇的移动方向");
    println!("  空格键              - 暂停/继续游戏");
    println!("  R                  - 游戏结束后重新开始");
    println!("  Q                  - 退出游戏");
    println!("");
    println!("计分规则:");
    println!("  • 基础分: 10分");
    println!("  • 颜色差异加成: 最多+20分");
    println!("  • 每100分升一级");
    println!("");
    println!("颜色系统:");
    println!("  • 背景: 暗色，低饱和度，定期变化");
    println!("  • 蛇: 与背景差异60°以上");
    println!("  • 食物: 与背景和蛇都差异30°以上");
    println!("  • 蛇头: 比蛇身亮20%");
    println!("");
    println!("动画效果:");
    println!("  • 食物: 0.85-1.15倍动态缩放");
    println!("  • 状态机驱动，不依赖延时");
    println!("  • 局部刷新，性能优化");
    println!("");
    println!("难度设置:");
    println!("  简单 (easy)   - 速度慢，颜色变化少");
    println!("  普通 (normal) - 默认难度");
    println!("  困难 (hard)   - 速度快，颜色变化多");
    println!("");
    println!("速度设置 (1-10):");
    println!("  1: 最慢 (230ms/帧)，10: 最快 (50ms/帧)");
    println!("");
    println!("随机种子:");
    println!("  使用 snake-seed <十六进制数> 设置随机种子");
    println!("  相同的种子会产生相同的颜色序列");

    Ok(())
}

// 设置随机种子
fn set_random_seed(seed: u64) -> Result<(), core::convert::Infallible> {
    println!("设置随机种子: 0x{:X}", seed);
    println!("下次启动游戏时将使用此种子");

    // 显示一些示例随机数
    let mut rng = SmallRng::seed_from_u64(seed);
    println!("示例随机数:");
    for i in 0..5 {
        let value: i32 = rng.random_range(-1000..1000);
        println!("  [{}]: {}", i + 1, value);
    }

    // 显示示例颜色
    println!("示例颜色 (HSL格式):");
    for i in 0..3 {
        let hsl = Hsl::new(
            rng.random_range(0.0..360.0),
            rng.random_range(0.3..0.9),
            rng.random_range(0.3..0.7),
        );
        println!(
            "  [{}]: H={:.1}°, S={:.2}, L={:.2}",
            i + 1,
            hsl.h,
            hsl.s,
            hsl.l
        );
    }

    Ok(())
}
