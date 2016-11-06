/**
 * This file is supposed to be used by Rust's ffi bindings. It contains
 * simple yet useful functions for decoding any supported file format
 * into a stream of PCM data using ffmpeg 3.2
 */

#include <stdio.h>
#include <pthread.h>

#include <libavcodec/avcodec.h>
#include <libavformat/avformat.h>
#include <libavdevice/avdevice.h>
#include <libswresample/swresample.h>
#include <ao/ao.h>

// --------
// Useful functions
// --------

#define DELETE(ptr) if (ptr != NULL) { free(ptr); ptr = NULL; }

// --------
// Mutexes
// --------

/**
 * Reference counter mutex.
 */
pthread_mutex_t rc_mutex;

/**
 * Reference counter.
 */
int rc;

// --------
// Enums / Structs / Callbacks
// --------

enum EarwaxError {
    IO_ERROR = 100,
    AUDIO_STREAM_NOT_FOUND,
    DECODER_NOT_FOUND,
    UNABLE_TO_OPEN_DECODER,
};

typedef struct {
    // General.
    AVFormatContext* format_ctx;
    AVCodecContext* codec_ctx;
    int stream_index;
    // The following members are for decoding state. All of them usually contain
    // a value that represents the current decoded chunk, and not all the
    // decoded data.
    SwrContext* swr;
    AVPacket packet;
    AVFrame* frame;
    uint8_t* buffer;
} EarwaxContext;

// --------
// API
// --------

/**
 * Initializes ffmpeg.
 */
void earwax_init();

/**
 * Shuts down ffmpeg.
 */
void earwax_shutdown();


/**
 * Creates a new EarwaxContext.
 * @param url url of the file to read from.
 * @return `0` on success. Any EarwaxError or AVERROR on error.
 */
int earwax_new(EarwaxContext** ctx_ptr, const char* url);

/**
 * Drops the given EarwaxContext context.
 * Does nothing if the passed pointer to
 * the context pointer is NULL.
 */
void earwax_drop(EarwaxContext** ctx);

/**
 * This function changes the pointer of data
 * to were the next chunk of decoded information
 * is.
 * @returns the number of bytes in data.
 */
int earwax_spit(EarwaxContext* ctx, char** data);

// --------
// API Definition
// --------

void earwax_init() {
    pthread_mutex_lock(&rc_mutex);
    if (rc <= 0) {
        av_register_all();
        avformat_network_init();
    }
    pthread_mutex_unlock(&rc_mutex);
}

void earwax_shutdown() {
    pthread_mutex_lock(&rc_mutex);
    if (rc <= 0) {
        avformat_network_deinit();
    }
    pthread_mutex_unlock(&rc_mutex);
}

int earwax_new(EarwaxContext** ctx_ptr, const char* url) {
    pthread_mutex_lock(&rc_mutex);
    rc++;

    *ctx_ptr = malloc(sizeof(EarwaxContext));
    EarwaxContext* ctx = *ctx_ptr;
    ctx->format_ctx = NULL;
    ctx->codec_ctx = NULL;
    ctx->stream_index = -1;
    /* ctx->packet = NULL; */
    ctx->frame = NULL;
    ctx->buffer = NULL;

    int ret_code = 0;

    // Format context.
    if (avformat_open_input(&ctx->format_ctx, url, NULL, NULL) != 0) {
        ret_code = IO_ERROR;
        goto FAIL;
    }
    if(avformat_find_stream_info(ctx->format_ctx, NULL) < 0) {
        ret_code = IO_ERROR;
        goto FAIL;
    }

    // Codec context.
    ctx->codec_ctx = avcodec_alloc_context3(NULL);
    for (int i = 0; i < ctx->format_ctx->nb_streams; ++i) {
        AVCodecParameters* codecpar = ctx->format_ctx->streams[i]->codecpar;
        if (codecpar->codec_type == AVMEDIA_TYPE_AUDIO) {
            avcodec_parameters_to_context(ctx->codec_ctx, codecpar);
            ctx->stream_index = i;
        }
    }
    if (ctx->stream_index < 0) {
        ret_code = AUDIO_STREAM_NOT_FOUND;
        goto FAIL;
    }

    // Codec loading.
    AVCodec* codec;
    codec = avcodec_find_decoder(ctx->codec_ctx->codec_id);
    if (codec == NULL) {
        ret_code = DECODER_NOT_FOUND;
        goto FAIL;
    }
    if (avcodec_open2(ctx->codec_ctx, codec, NULL) < 0) {
        ret_code = UNABLE_TO_OPEN_DECODER;
        goto FAIL;
    }

    // Swr.
    ctx->swr = swr_alloc();
    av_opt_set_int(ctx->swr, "in_channel_layout", ctx->codec_ctx->channel_layout, 0);
    av_opt_set_int(ctx->swr, "out_channel_layout", ctx->codec_ctx->channel_layout, 0);
    av_opt_set_int(ctx->swr, "in_sample_rate", ctx->codec_ctx->sample_rate, 0);
    av_opt_set_int(ctx->swr, "out_sample_rate", ctx->codec_ctx->sample_rate, 0);
    av_opt_set_sample_fmt(ctx->swr, "in_sample_fmt", ctx->codec_ctx->sample_fmt, 0);
    av_opt_set_sample_fmt(ctx->swr, "out_sample_fmt", AV_SAMPLE_FMT_S16, 0);
    swr_init(ctx->swr);

    // Packet init.
    av_init_packet(&ctx->packet);

    // Frame space.
    ctx->frame = av_frame_alloc();

    // Buffer for decoded chunks. Decoded PCM data is signed int of 16 bits, so we can
    // multiply by either sizeof(int16_t) or sizeof(uint16_t).
    ctx->buffer = malloc(
        (ctx->codec_ctx->frame_size + AV_INPUT_BUFFER_PADDING_SIZE)
        * ctx->codec_ctx->channels
        * sizeof(uint16_t)
    );

    // Up to this point everything is right!
    goto SUCCESS;

    FAIL:
        pthread_mutex_unlock(&rc_mutex);
        earwax_drop(ctx_ptr);
        return ret_code;
    SUCCESS:
        pthread_mutex_unlock(&rc_mutex);
        return 0;
}

void earwax_drop(EarwaxContext** ctx_ptr) {
    pthread_mutex_lock(&rc_mutex);
    // Delete inner pointers.
    if (*ctx_ptr != NULL) {
        EarwaxContext* ctx = *ctx_ptr;
        DELETE(ctx->buffer);
        av_frame_free(&ctx->frame);
        av_packet_unref(&ctx->packet);
        swr_free(&ctx->swr);
        avcodec_free_context(&ctx->codec_ctx);
        avformat_close_input(&ctx->format_ctx);
        avformat_free_context(ctx->format_ctx);
        DELETE(*ctx_ptr);
        rc--;
    }
    pthread_mutex_unlock(&rc_mutex);
}

int next_chunk(EarwaxContext* ctx, char** data) {
    if (avcodec_receive_frame(ctx->codec_ctx, ctx->frame) == 0) {
        swr_convert(
            ctx->swr,
            &ctx->buffer, ctx->frame->nb_samples,
            (const uint8_t**) ctx->frame->extended_data, ctx->frame->nb_samples
        );
        (*data) = ctx->buffer;

        // Return the written bytes:
        // Each sample is two bytes, and we have n channels.
        return ctx->frame->nb_samples * ctx->frame->channels * sizeof(uint16_t);
    }
    else {
        (*data) = NULL;
        return 0;
    }
}

int earwax_spit(EarwaxContext* ctx, char** data) {
    int chunk_size = next_chunk(ctx, data);
    if (chunk_size > 0) {
        // Return pending chunks.
        return chunk_size;
    }
    else {
        // Ask for more chunks.
        if (av_read_frame(ctx->format_ctx, &ctx->packet) >= 0) {
            if (ctx->packet.stream_index == ctx->stream_index) {
                avcodec_send_packet(ctx->codec_ctx, &ctx->packet);
            }
            av_packet_unref(&ctx->packet);
        }

        return next_chunk(ctx, data);
    }
}

int main(int argc, char* argv[]) {
    earwax_init();
    EarwaxContext* ctx = NULL;
    earwax_new(&ctx, argv[1]);

    if (ctx != NULL) {
        // Ao stuff
        ao_initialize();
        ao_device *device;
        ao_sample_format format;
        int default_driver = ao_default_driver_id();
        memset(&format, 0, sizeof(format));
        format.bits = 16;
        format.channels = 2;
        format.rate = 44100;
        format.byte_format = AO_FMT_LITTLE;
        device = ao_open_live(default_driver, &format, NULL);

        // Play music
        char* data;
        size_t len;
        while (len = earwax_spit(ctx, &data)) {
            ao_play(device, data, len);
        }

        ao_close(device);
        ao_shutdown();

        earwax_drop(&ctx);
    }

    earwax_shutdown();
}
