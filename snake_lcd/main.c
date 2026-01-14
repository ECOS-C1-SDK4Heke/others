#include "main.h"
#include "logo.h"
#include "menu.h"
#include "over.h"

SNAKE_T snake;
FOOD_T food;
uint8_t score = 0;
uint8_t hi_score = 0;
uint8_t game_state = 0;
uint32_t game_tick = 0;
uint16_t rand_seed = 233;
st7735_device_t lcd;

const uint32_t snake_colors[] = {
    0xF800, 0x07E0, 0x001F, 0xF81F, 0xFFE0, 0x07FF, 0xFF00, 0xFC1F,
    0xD7E0, 0xFD20, 0xFBEA, 0x9F3F, 0xAF5F, 0xC618, 0xFD00, 0x87FF
};

const uint32_t food_colors[] = {
    0xF800, 0x07E0, 0x001F
};

char uart_getc_nowait(void) {
    int32_t c = REG_UART_0_DATA;
    if(c != -1) {
        return (char)c;
    }
    return 0;
}

char uart_getc_wait(void) {
    int32_t c = -1;
    while(c == -1) {
        c = REG_UART_0_DATA;
    }
    return (char)c;
}

uint16_t rand16(void) {
    rand_seed = rand_seed * 114514 + 360123;
    return rand_seed;
}

void lcd_draw_rect(uint16_t x, uint16_t y, uint16_t w, uint16_t h, uint32_t color) {
    uint32_t color_32bit = (color << 16) | color;
    st7735_fill(&lcd, x, y, x + w, y + h, color_32bit);
}

void lcd_draw_cell(uint8_t gx, uint8_t gy, uint32_t color) {
    uint16_t x = GAME_OFFSET_X + gx * CELL_SIZE;
    uint16_t y = GAME_OFFSET_Y + gy * CELL_SIZE;
    lcd_draw_rect(x, y, CELL_SIZE, CELL_SIZE, color);
}

void game_init(void) {
    snake.len = 3;
    snake.dir = DIR_R;
    snake.next = DIR_R;
    snake.grow = 0;
    snake.alive = 1;
    snake.color = snake_colors[rand16() % 16];

    uint8_t sx = GRID_W / 2;
    uint8_t sy = GRID_H / 2;
    for(uint8_t i = 0; i < snake.len; i++) {
        snake.body[i].x = sx - i;
        snake.body[i].y = sy;
    }

    food.exist = 0;
    food.color = 0xF800;
    score = 0;
    game_tick = 0;

    lcd_draw_rect(GAME_OFFSET_X - 1, GAME_OFFSET_Y - 1, GAME_WIDTH + 2, GAME_HEIGHT + 2, 0xFFFF);
    lcd_draw_rect(GAME_OFFSET_X, GAME_OFFSET_Y, GAME_WIDTH, GAME_HEIGHT, 0x0000);
}

void input_proc(void) {
    char k = uart_getc_nowait();
    if(k) {
        if(k == 'w' || k == 'W') {
            if(snake.dir != DIR_D) snake.next = DIR_U;
        } else if(k == 's' || k == 'S') {
            if(snake.dir != DIR_U) snake.next = DIR_D;
        } else if(k == 'a' || k == 'A') {
            if(snake.dir != DIR_R) snake.next = DIR_L;
        } else if(k == 'd' || k == 'D') {
            if(snake.dir != DIR_L) snake.next = DIR_R;
        } else if(k == 'p' || k == 'P') {
            game_state = 2;
        } else if(k == 'q' || k == 'Q') {
            printf("\nQuitting game...\n");
            while(1);
        }
    }
}

void snake_move(void) {
    if(!snake.alive) return;

    snake.dir = snake.next;
    POINT_T tail = snake.body[snake.len - 1];

    for(int i = snake.len - 1; i > 0; i--) {
        snake.body[i] = snake.body[i - 1];
    }

    POINT_T head = snake.body[0];
    switch(snake.dir) {
        case DIR_U: head.y--; break;
        case DIR_D: head.y++; break;
        case DIR_L: head.x--; break;
        case DIR_R: head.x++; break;
    }

    if(head.x >= GRID_W || head.y >= GRID_H) {
        snake.alive = 0;
        return;
    }

    snake.body[0] = head;
    for(uint8_t i = 1; i < snake.len; i++) {
        if(head.x == snake.body[i].x && head.y == snake.body[i].y) {
            snake.alive = 0;
            return;
        }
    }

    if(food.exist && head.x == food.pos.x && head.y == food.pos.y) {
        score += food.type + 1;
        snake.grow += food.type + 1;
        food.exist = 0;
        if(score > hi_score) hi_score = score;
    }

    if(snake.grow > 0 && snake.len < SNAKE_MAX) {
        snake.body[snake.len] = tail;
        snake.len++;
        snake.grow--;
    }
}

void food_gen(void) {
    if(food.exist) return;

    uint8_t tries = 0;
    while(tries < 30) {
        food.pos.x = rand16() % GRID_W;
        food.pos.y = rand16() % GRID_H;
        uint8_t ok = 1;
        for(uint8_t i = 0; i < snake.len; i++) {
            if(food.pos.x == snake.body[i].x && food.pos.y == snake.body[i].y) {
                ok = 0;
                break;
            }
        }
        if(ok) {
            food.type = rand16() % 3;
            food.color = food_colors[food.type];
            food.exist = 1;
            return;
        }
        tries++;
    }
}

void draw_all(void) {
    static POINT_T last_tail = {0, 0};

    if(snake.alive) {
        for(uint8_t i = 0; i < snake.len; i++) {
            if(i == 0) {
                lcd_draw_cell(snake.body[i].x, snake.body[i].y, 0xFFFF);
            } else {
                lcd_draw_cell(snake.body[i].x, snake.body[i].y, snake.color);
            }
        }

        if(snake.grow == 0 && snake.len > 0) {
            lcd_draw_cell(last_tail.x, last_tail.y, 0x0000);
        }

        if(snake.len > 0) {
            last_tail = snake.body[snake.len - 1];
        }
    }

    if(food.exist) {
        lcd_draw_cell(food.pos.x, food.pos.y, food.color);
    }
}

void show_menu(void) {
    st7735_fill_img(&lcd, 0, 0, SCREEN_WIDTH, SCREEN_HEIGHT, (const uint32_t *)menu);
    
    printf("Snake Game Menu\n");
    printf("S: Start Game\n");
    printf("Q: Quit Game\n");
    printf("Select: ");
    
    while(1) {
        char k = uart_getc_wait();
        if(k == 's' || k == 'S') {
            printf("Starting...\n");
            delay_ms(500);
            lcd_draw_rect(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT, 0x0000);
            return;
        }
        if(k == 'q' || k == 'Q') {
            printf("\nQuitting game...\n");
            while(1);
        }
    }
}

void show_start(void) {
    st7735_fill_img(&lcd, 0, 0, SCREEN_WIDTH, SCREEN_HEIGHT, (const uint32_t *)LOGO);
    delay_ms(LOGO_DISPLAY_TIME_MS);
    lcd_draw_rect(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT, 0xFFFF);
    delay_ms(500);
    lcd_draw_rect(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT, 0x0000);
}

void show_over(void) {
    st7735_fill_img(&lcd, 0, 0, SCREEN_WIDTH, SCREEN_HEIGHT, (const uint32_t *)over);

    printf("\nGAME OVER\n");
    printf("Score: %u\n", score);
    printf("HiScore: %u\n", hi_score);
    printf("Length: %u\n", snake.len);
    printf("R: Restart Game\n");
    printf("Q: Quit Game\n");
    printf("Choose: ");

    while(1) {
        char k = uart_getc_wait();
        if(k == 'r' || k == 'R') {
            printf("Restarting...\n");
            delay_ms(500);
            lcd_draw_rect(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT, 0x0000);
            game_init();
            return;
        }
        if(k == 'q' || k == 'Q') {
            printf("\nQuitting game...\n");
            game_state = 0 - 1;
            return;
        }
    }
}

void game_run(void) {
    input_proc();
    if(game_tick % 2 == 0) {
        snake_move();
        food_gen();
    }
    draw_all();
    game_tick++;
    delay_ms(GAME_SPEED);
    if(!snake.alive) game_state = 3;
}

void main(void) {
    sys_uart_init();
    printf("\033[?25l");

    sys_tick_init();

    qspi_config_t qspi_config = {
        .clkdiv = 0,
    };
    qspi_init(&qspi_config);

    lcd.dc_pin = 2;
    lcd.screen_width = SCREEN_WIDTH;
    lcd.screen_height = SCREEN_HEIGHT;
    lcd.rotation = 0;
    lcd.horizontal_offset = 2;
    lcd.vertical_offset = 3;

    st7735_init(&lcd);
    lcd_draw_rect(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT, 0x0000);

    game_state = 0;

    while(1) {
        if(game_state == 0) {
            show_start();
            show_menu();
            game_init();
            game_state = 1;
        } else if(game_state == 1) {
            game_run();
        } else if(game_state == 2) {
            printf("\nPAUSED\nPress any key...");
            uart_getc_wait();
            game_state = 1;
        } else if(game_state == 3) {
            show_over();
            if(game_state == 0 - 1) break;
            game_state = 1;
        }
    }

    printf("\033[?25h");
}
