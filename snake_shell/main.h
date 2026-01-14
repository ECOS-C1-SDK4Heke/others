#ifndef _MAIN_H_
#define _MAIN_H_

#include "string.h"
#include "stdio.h"
#include "libgcc.h"
#include "generated/autoconf.h"
#include "timer.h"
#include "hp_uart.h"
#include "board.h"

#define GRID_W 16
#define GRID_H 12
#define SNAKE_MAX 50
#define GAME_SPEED 180

typedef enum { DIR_U, DIR_R, DIR_D, DIR_L, DIR_N } DIR_T;

typedef struct { uint8_t x, y; } POINT_T;

typedef struct {
    POINT_T body[SNAKE_MAX];
    uint8_t len;
    DIR_T dir;
    DIR_T next;
    uint8_t grow;
    uint8_t alive;
} SNAKE_T;

typedef struct {
    POINT_T pos;
    uint8_t type;
    uint8_t exist;
} FOOD_T;

extern SNAKE_T snake;
extern FOOD_T food;
extern uint8_t score;
extern uint8_t hi_score;
extern uint8_t game_state;
extern uint32_t game_tick;

void game_init(void);
void game_run(void);
void input_proc(void);
void snake_move(void);
void food_gen(void);
void draw_all(void);
void show_start(void);
void show_over(void);
char uart_getc_nowait(void);
char uart_getc_wait(void);
uint16_t rand16(void);

#endif
