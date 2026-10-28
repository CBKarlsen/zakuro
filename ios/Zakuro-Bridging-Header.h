// what Swift sees of crates/zakuro-apple, which has the details
#include <stdbool.h>
#include <stdint.h>

typedef struct Game Game;

Game *zakuro_open(const char *rom, const char *data_dir, uint64_t program_id, uint32_t resolution);
bool zakuro_run_frame(Game *game);
const uint8_t *zakuro_screen(Game *game, uint32_t screen, uint32_t *width, uint32_t *height);
void zakuro_set_buttons(uint32_t buttons);
void zakuro_set_touch(uint16_t x, uint16_t y, bool released);
void zakuro_close(Game *game);
