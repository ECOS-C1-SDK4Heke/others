#include "main.h"

SNAKE_T snake;
FOOD_T food;
uint8_t score = 0;
uint8_t hi_score = 0;
uint8_t game_state = 0;
uint32_t game_tick = 0;
uint16_t rand_seed = 233;

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

void game_init(void) {
    snake.len = 3;
    snake.dir = DIR_R;
    snake.next = DIR_R;
    snake.grow = 0;
    snake.alive = 1;
    uint8_t sx = GRID_W/2;
    uint8_t sy = GRID_H/2;
    for(uint8_t i=0; i<snake.len; i++) {
        snake.body[i].x = sx - i;
        snake.body[i].y = sy;
    }
    food.exist = 0;
    score = 0;
    game_tick = 0;
    printf("\033[2J\033[H");
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
    POINT_T tail = snake.body[snake.len-1];
    for(int i=snake.len-1; i>0; i--) {
        snake.body[i] = snake.body[i-1];
    }
    POINT_T head = snake.body[0];
    switch(snake.dir) {
        case DIR_U: head.y--; break;
        case DIR_D: head.y++; break;
        case DIR_L: head.x--; break;
        case DIR_R: head.x++; break;
        default: break;
    }
    
    if(head.x >= GRID_W || head.y >= GRID_H || head.x < 0 || head.y < 0) {
        snake.alive = 0;
        return;
    }
    
    snake.body[0] = head;
    for(uint8_t i=1; i<snake.len; i++) {
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
        for(uint8_t i=0; i<snake.len; i++) {
            if(food.pos.x == snake.body[i].x &&
               food.pos.y == snake.body[i].y) {
                ok = 0;
                break;
            }
        }
        if(ok) {
            food.type = rand16() % 3;
            food.exist = 1;
            return;
        }
        tries++;
    }
}

void draw_all(void) {
    printf("\033[0;0H");
    printf("+");
    for(uint8_t i=0; i<GRID_W; i++) printf("-");
    printf("+\n");
    for(uint8_t y=0; y<GRID_H; y++) {
        printf("|");
        for(uint8_t x=0; x<GRID_W; x++) {
            char c = ' ';
            if(snake.alive && x == snake.body[0].x && y == snake.body[0].y) {
                c = 'O';
            } else {
                uint8_t isbody = 0;
                for(uint8_t i=1; i<snake.len; i++) {
                    if(x == snake.body[i].x && y == snake.body[i].y) {
                        c = 'o';
                        isbody = 1;
                        break;
                    }
                }
                if(!isbody && food.exist && x == food.pos.x && y == food.pos.y) {
                    if(food.type == 0) c = '@';
                    else if(food.type == 1) c = '#';
                    else c = '$';
                }
            }
            printf("%c", c);
        }
        printf("|\n");
    }
    printf("+");
    for(uint8_t i=0; i<GRID_W; i++) printf("-");
    printf("+\n");
    
    if(GRID_W > 50) {
        printf("Score:%u Hi:%u Len:%u WASD:Move P:Pause Q:Quit\n",
               score, hi_score, snake.len);
    } else {
        printf("Score:%u Hi:%u Len:%u\n", score, hi_score, snake.len);
        printf("WASD P:Pause Q:Quit\n");
    }
}

void show_start(void) {
    printf("\033[2J\033[H");
    printf("\n\n");
    printf("  SNAKE GAME\n");
    printf("  ----------\n");
    printf("  S: Start Game\n");
    printf("  Q: Quit Game\n");
    printf("  Enter: Refresh Screen\n");
    printf("  Select: ");
    
    while(1) {
        char k = uart_getc_wait();
        
        if(k == 's' || k == 'S') {
            printf("Starting...\n");
            delay_ms(500);
            break;
        }
        if(k == 'q' || k == 'Q') {
            printf("\nQuitting game...\n");
            while(1);
        }
        if(k == '\r' || k == '\n') {
            printf("\033[2J\033[H");
            printf("\n\n");
            printf("  SNAKE GAME\n");
            printf("  ----------\n");
            printf("  S: Start Game\n");
            printf("  Q: Quit Game\n");
            printf("  Enter: Refresh Screen\n");
            printf("  Select: ");
        }
    }
}

void show_over(void) {
    printf("\033[2J\033[H");
    printf("\n\n");
    printf("  GAME OVER\n");
    printf("  ---------\n");
    printf("  Score: %u\n", score);
    printf("  HiScore: %u\n", hi_score);
    printf("  Length: %u\n", snake.len);
    printf("\n");
    printf("  R: Restart Game\n");
    printf("  Q: Quit Game\n");
    printf("  Choose: ");
    
    while(1) {
        char k = uart_getc_wait();
        
        if(k == 'r' || k == 'R') {
            printf("Restarting...\n");
            delay_ms(500);
            game_init();
            return;
        }
        if(k == 'q' || k == 'Q') {
            printf("\nQuitting game...\n");
	    game_state = 0-1;
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

static inline void hide_cursor(void) {
    printf("\033[?25l");
}

static inline void show_cursor(void) {
    printf("\033[?25h");
}

void main(void) {
    sys_uart_init();
    hide_cursor();
    game_state = 0;
    
    while(1) {
        if(game_state == 0) {
            show_start();
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
	    if (game_state == 0-1) break;
            game_state = 1;
        }
    }

    show_cursor();
}
