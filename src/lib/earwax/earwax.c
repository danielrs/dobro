/**
 * This file is supposed to be used by Rust's ffi bindings. It contains
 * simple yet useful functions for decoding any supported file format
 * into a stream of PCM data using ffmpeg 2.8.
 */

#include <pthread.h>

#include <libavcodec/avcodec.h>
#include <libavformat/avformat.h>
#include <libavdevice/avdevice.h>
#include <libswresample/swresample.h>

// --------
// Useful functions / macros
// --------

#define DELETE(ptr) if (ptr != NULL) { free(ptr); ptr = NULL; }

// --------
// Mutexes
// --------
// Used for synchronizing init and shutdown
// of ffmpeg.

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

/**
 * Errors for different earwax functions.
 */
enum EarwaxError {
    IO_ERROR = 100,
    AUDIO_STREAM_NOT_FOUND,
    DECODER_NOT_FOUND,
    UNABLE_TO_OPEN_DECODER,
};

/**
 * Type of rational numbers.
 */
typedef struct {
    int64_t num;
    int64_t den;
} EarwaxRational;

/**
 * Used for returning audio info
 * to the user.
 */
typedef struct {
    int bitrate; /**< In bytes, the bitrate of the audio. */
    int sample_rate; /**< The rate of samples per-second. */
    int64_t start_time; /**< Start time in time_base units. */
    int64_t duration; /**< Duration in time_base units. */
    EarwaxRational time_base; /**< Unit of time for this stream, wher 1/60 means 60 frames per second. */
} EarwaxInfo;

/**
 * Main container for an instance.
 */
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
    // Information for this audio.
    EarwaxInfo info;
} EarwaxContext;

/**
 * Structure for returning each chunk data during
 * decoding. **Note** that samples of the
 * PCM data are always:
 *
 * 1. Signed integer of 16 bits.
 * 2. Interleaved to 2 channels (stereo).
 */
typedef struct {
    char* data; /**< Pointer to the decoded data. */
    size_t size; /**< Size in bytes of the decoded data. */
    int64_t time; /**< Time of chunks inside the stream in time_base units. */
} EarwaxChunk;

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
 * Reads information from the provided context and sets it
 * in the provided pointer.
 */
void earwax_get_info(const EarwaxContext* ctx, EarwaxInfo* info);

/**
 * This function changes the pointer of data
 * to were the next chunk of decoded information
 * is.
 * @returns the number of bytes in data.
 */
int earwax_spit(EarwaxContext* ctx, EarwaxChunk* chunk);

/**
 * Moves the current position of the decoder to the given time (in time_base units).
 * Useful for rewinding or fast-forwarding.
 */
int earwax_seek(EarwaxContext* ctx, int64_t pts);

// --------
// Private API
// --------

/**
 * Creates a chunk from the given frame in ctx.
 * @returns The size in bytes of the chunk data.
 */
int next_chunk(EarwaxContext* ctx, EarwaxChunk* chunk);

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

    *ctx_ptr = calloc(1, sizeof(EarwaxContext));
    EarwaxContext* ctx = *ctx_ptr;

    int ret_code = 0;

    // Format context and stream information.
    if (avformat_open_input(&ctx->format_ctx, url, NULL, NULL) != 0) {
        ret_code = IO_ERROR;
        goto FAIL;
    }
    if(avformat_find_stream_info(ctx->format_ctx, NULL) < 0) {
        ret_code = IO_ERROR;
        goto FAIL;
    }

    // Codec context.
    ctx->stream_index = -1;
    ctx->codec_ctx = avcodec_alloc_context3(NULL);
    for (int i = 0; i < ctx->format_ctx->nb_streams; ++i) {
        AVCodecContext* codec_ctx = ctx->format_ctx->streams[i]->codec;
        if (codec_ctx->codec_type == AVMEDIA_TYPE_AUDIO) {
            avcodec_copy_context(ctx->codec_ctx, codec_ctx);
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

    // Swr for resampling.
    ctx->swr = swr_alloc();
    av_opt_set_int(ctx->swr, "in_channel_layout", ctx->codec_ctx->channel_layout, 0);
    av_opt_set_int(ctx->swr, "out_channel_layout", AV_CH_LAYOUT_STEREO, 0);
    av_opt_set_int(ctx->swr, "in_sample_rate", ctx->codec_ctx->sample_rate, 0);
    av_opt_set_int(ctx->swr, "out_sample_rate", ctx->codec_ctx->sample_rate, 0);
    av_opt_set_sample_fmt(ctx->swr, "in_sample_fmt", ctx->codec_ctx->sample_fmt, 0);
    av_opt_set_sample_fmt(ctx->swr, "out_sample_fmt", AV_SAMPLE_FMT_S16, 0);
    swr_init(ctx->swr);

    // Frame space.
    ctx->frame = av_frame_alloc();

    // Buffer for decoded chunks. Decoded PCM data is signed int of 16 bits, so we can
    // multiply by either sizeof(int16_t) or sizeof(uint16_t).
    ctx->buffer = malloc(
        (ctx->codec_ctx->frame_size + AV_INPUT_BUFFER_PADDING_SIZE)
        * ctx->codec_ctx->channels
        * sizeof(uint16_t)
    );

    // Sets information.
    earwax_get_info(ctx, &ctx->info);

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
        av_frame_unref(ctx->frame);
        av_frame_free(&ctx->frame);
        av_free_packet(&ctx->packet);
        swr_free(&ctx->swr);
        avcodec_free_context(&ctx->codec_ctx);
        avformat_close_input(&ctx->format_ctx);
        avformat_free_context(ctx->format_ctx);
        DELETE(*ctx_ptr);

        rc--;
    }
    pthread_mutex_unlock(&rc_mutex);
}

void earwax_get_info(const EarwaxContext* ctx, EarwaxInfo* info) {
    memset(info, 0, sizeof(EarwaxInfo));

    // Tries to find more meaningful values.
    if (ctx != NULL) {
        AVStream* stream = ctx->format_ctx->streams[ctx->stream_index];

        info->bitrate = ctx->codec_ctx->bit_rate;
        info->sample_rate = ctx->codec_ctx->sample_rate;
        info->start_time = stream->start_time;
        info->duration = stream->duration;
        info->time_base.num = stream->time_base.num;
        info->time_base.den = stream->time_base.den;
    }
}

int earwax_spit(EarwaxContext* ctx, EarwaxChunk* chunk) {
    if (ctx->packet.size > 0) {
        int got_frame = 0;
        int bytes =  avcodec_decode_audio4(ctx->codec_ctx, ctx->frame, &got_frame, &(ctx->packet));

        if (bytes > 0) {
            ctx->packet.size -= bytes;
            ctx->packet.data += bytes;
        }

        if (got_frame) {
            // Next frame.
            return next_chunk(ctx, chunk);
        }
        else if (ctx->packet.size > 0 && bytes > 0) {
            // Still processing the packet.
            return earwax_spit(ctx, chunk);
        }
        else {
            // We are done with the packet.
            ctx->packet.size = 0;
            ctx->packet.data = NULL;
            return earwax_spit(ctx, chunk);
        }
    }
    else {
        // Free packet and unrefs frame.
        av_frame_unref(ctx->frame);
        av_free_packet(&ctx->packet);

        if (av_read_frame(ctx->format_ctx, &(ctx->packet)) == 0) {
            if (ctx->packet.stream_index == ctx->stream_index) {
                return earwax_spit(ctx, chunk);
            }
            else {
                // Drop unwanted package.
                ctx->packet.size = 0;
                ctx->packet.data = NULL;
                return earwax_spit(ctx, chunk);
            }
        }
    }

    return 0;
}

int earwax_seek(EarwaxContext* ctx, int64_t pts) {
    int64_t start_time = ctx->info.start_time;
    int64_t duration = ctx->info.duration;

    if (pts < start_time) pts = start_time;
    if (pts > duration) pts = duration;

    return av_seek_frame(ctx->format_ctx, ctx->stream_index, pts, AVSEEK_FLAG_BACKWARD);
}

// --------
// Private API Definition
// --------

int next_chunk(EarwaxContext* ctx, EarwaxChunk* chunk) {
    swr_convert(
        ctx->swr,
        &ctx->buffer, ctx->frame->nb_samples,
        (const uint8_t**) ctx->frame->extended_data, ctx->frame->nb_samples
    );

    // Each sample is two bytes (uint16_t), and we have n channels.
    chunk->data = ctx->buffer;
    chunk->size = ctx->frame->nb_samples * ctx->frame->channels * sizeof(uint16_t);
    chunk->time = ctx->frame->pkt_pts;

    return chunk->size;
}

/**
 * Example program for testing the library.
 */

/* #include <ao/ao.h> */
/* int main(int argc, char* argv[]) { */
/*     earwax_init(); */

/*     while(1) { */
/*         EarwaxContext* ctx = NULL; */
/*         EarwaxInfo info; */
/*         earwax_new(&ctx, argv[1]); */
/*         earwax_get_info(ctx, &info); */

/*         printf("Bit rate: %d\n", info.bitrate); */
/*         printf("Sample rate: %d\n", info.sample_rate); */
/*         printf("Duration: %d\n", info.duration); */

/*         if (ctx != NULL) { */
/*             // Ao stuff */
/*             ao_initialize(); */
/*             ao_device *device; */
/*             ao_sample_format format; */
/*             int default_driver = ao_default_driver_id(); */
/*             memset(&format, 0, sizeof(format)); */

/*             format.bits = 16; */
/*             format.channels = 2; */
/*             format.rate = 44100; */
/*             format.byte_format = AO_FMT_NATIVE; */
/*             device = ao_open_live(default_driver, &format, NULL); */

/*             // Play music */
/*             EarwaxChunk chunk; */
/*             while (earwax_spit(ctx, &chunk)) { */
/*                 printf("%d / %d\n", chunk.time, info.duration); */
/*                 ao_play(device, chunk.data, chunk.size); */
/*             } */

/*             ao_close(device); */
/*             ao_shutdown(); */
/*         } */

/*         earwax_drop(&ctx); */
/*     } */

/*     earwax_shutdown(); */
/* } */
