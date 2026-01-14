#ifndef __MAIN_H__
#define __MAIN_H__

#include <stdint.h>

#include "string.h"
#include "stdio.h"
#include "libgcc.h"
#include "generated/autoconf.h"
#include "timer.h"
#include "hp_uart.h"
#include "board.h"
#include "qspi.h"
#include "st7735.h"

#define LOGO_DISPLAY_TIME_MS 2000
#define GAME_SPEED 100
#define GRID_W 40
#define GRID_H 40
#define CELL_SIZE 3
#define SNAKE_MAX 256
#define GAME_OFFSET_X 4
#define GAME_OFFSET_Y 4
#define GAME_WIDTH (GRID_W * CELL_SIZE)
#define GAME_HEIGHT (GRID_H * CELL_SIZE)
#define SCREEN_WIDTH 128
#define SCREEN_HEIGHT 128

typedef enum {
    DIR_U = 0,
    DIR_D,
    DIR_L,
    DIR_R
} DIR_T;

typedef struct {
    uint8_t x;
    uint8_t y;
} POINT_T;

typedef struct {
    POINT_T body[SNAKE_MAX];
    uint16_t len;
    DIR_T dir;
    DIR_T next;
    uint8_t grow;
    uint8_t alive;
    uint32_t color;
} SNAKE_T;

typedef struct {
    POINT_T pos;
    uint8_t type;
    uint8_t exist;
    uint32_t color;
} FOOD_T;

extern SNAKE_T snake;
extern FOOD_T food;
extern uint8_t score;
extern uint8_t hi_score;
extern uint8_t game_state;
extern uint32_t game_tick;
extern uint16_t rand_seed;

char uart_getc_nowait(void);
char uart_getc_wait(void);
uint16_t rand16(void);
void game_init(void);
void input_proc(void);
void snake_move(void);
void food_gen(void);
void draw_all(void);
void show_start(void);
void show_menu(void);
void show_over(void);
void game_run(void);
void lcd_draw_rect(uint16_t x, uint16_t y, uint16_t w, uint16_t h, uint32_t color);
void lcd_draw_cell(uint8_t gx, uint8_t gy, uint32_t color);

#endif
