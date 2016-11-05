/**
 * This file is supposed to be used by Rust's ffi bindings. It contains
 * simple yet useful functions for decoding any supported file format
 * into a stream of PCM data using ffmpeg 3.2
 */

#include <stdio.h>

void earwax_init() {
    printf("Earwax Init!\n");
}
