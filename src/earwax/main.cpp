extern "C" {
#include <libavcodec/avcodec.h>
#include <libavformat/avformat.h>
#include <libavdevice/avdevice.h>
#include <ao/ao.h>
#include <libswresample/swresample.h>
}

#include <limits>

void log(const char* message) {
    printf("%s\n", message);
}

static const char* url = "http://audio-dc6-t1-2-v4v6.pandora.com/access/7147589208482120810.mp4?version=5&lid=1945804&token=gJXZtn8TvKXBkRXEJopnZwN6vNaFIF2b%2F%2BLsMLYIXq5x%2Bu97MJ1nu98qwxYWQybhz1igmcfg5tzcoD9QnFc%2BSycjweaE%2FfRHkgvGueE6uLHrAeSSXI%2Bzgi%2BJuUbFUbZK3QQxWNrtx49FToGk2SXGH1Q3H8EvA4gDeyrb%2B9AUGcB%2BZqZsLC2sjyDLijHAYpbnRNo3uK0ugTtT0xxKU7lYiN1lRpImoBicTZcRwJpW0CM8FogXvdbEqcGAzK%2B3mWBvSfqzrlGsIa90KisdF71V9jRCFQogmpNbEOnUS%2FDouqHR%2BDGVM7561Qf5iIB1xe2%2BUkwaGwV6Xbs%3D";

int main(int argc, char* argv[]) {
    av_register_all();
    avformat_network_init();
    AVFormatContext* format_ctx = NULL;
    AVCodecContext* codec_ctx = avcodec_alloc_context3(NULL);
    AVCodec* codec = NULL;

    if (avformat_open_input(&format_ctx, argv[1], NULL, NULL) != 0) {
        return -1;
    }

    /* if (avformat_find_stream_info(format_ctx, NULL) < 0) { */
    /*     return -1; */
    /* } */

    av_dump_format(format_ctx, 0, url, 0);

    int audio_index = -1;
    for (int i = 0; i < format_ctx->nb_streams; ++i) {
        AVCodecParameters* codecpar = format_ctx->streams[i]->codecpar;
        if (codecpar->codec_type == AVMEDIA_TYPE_AUDIO) {
            avcodec_parameters_to_context(codec_ctx, codecpar);
            audio_index = i;
        }
    }

    codec = avcodec_find_decoder(codec_ctx->codec_id);
    if (codec == NULL) {
        log("Codec not found.");
        return -1;
    }

    if (avcodec_open2(codec_ctx, codec, NULL) < 0) {
        log("Error opening codec.");
        return - 1;
    }

    // Prepare format.
    AVSampleFormat sfmt = codec_ctx->sample_fmt;

    // Ao stuff
    ao_initialize();
    ao_device *device;
    ao_sample_format format;
    int default_driver = ao_default_driver_id();
    memset(&format, 0, sizeof(format));

    if(sfmt==AV_SAMPLE_FMT_U8 || sfmt==AV_SAMPLE_FMT_U8P){
        log("ONE");
        format.bits=16;
    } else if(sfmt==AV_SAMPLE_FMT_S16 || sfmt==AV_SAMPLE_FMT_S16P){
        log("TWO");
        format.bits=16;
    } else if(sfmt==AV_SAMPLE_FMT_S32 || sfmt==AV_SAMPLE_FMT_S32P){
        log("THREE");
        format.bits=16;
    } else if(sfmt==AV_SAMPLE_FMT_FLT) {
        log("FOUR 1");
        format.bits=16;
    } else if(sfmt==AV_SAMPLE_FMT_FLTP) {
        log("FOUR 2");
        format.bits=16;
    } else if(sfmt==AV_SAMPLE_FMT_DBL || sfmt==AV_SAMPLE_FMT_DBLP) {
        log("FIVE");
        format.bits=16;
    } else {
        log("Unsupported format.");
    }
    format.channels = codec_ctx->channels;
    format.rate = codec_ctx->sample_rate;
    format.byte_format = AO_FMT_NATIVE;

    device = ao_open_live(default_driver, &format, NULL);
    if (device == NULL) {
        printf("%d", default_driver);
        log("Error opening device.");
        return -1;
    }

    // Sets up resample
    SwrContext* swr = swr_alloc();
    av_opt_set_int(swr, "in_channel_layout", codec_ctx->channel_layout, 0);
    av_opt_set_int(swr, "out_channel_layout", codec_ctx->channel_layout, 0);
    av_opt_set_int(swr, "in_sample_rate", codec_ctx->sample_rate, 0);
    av_opt_set_int(swr, "out_sample_rate", codec_ctx->sample_rate, 0);
    av_opt_set_sample_fmt(swr, "in_sample_fmt", AV_SAMPLE_FMT_FLTP, 0);
    av_opt_set_sample_fmt(swr, "out_sample_fmt", AV_SAMPLE_FMT_S16, 0);
    swr_init(swr);

    // Iterate over packets.
    AVPacket packet;
    AVFrame *frame = av_frame_alloc();
    av_init_packet(&packet);
    while (av_read_frame(format_ctx, &packet) >= 0) {
        if (packet.stream_index == audio_index) {
            avcodec_send_packet(codec_ctx, &packet);
            while (avcodec_receive_frame(codec_ctx, frame) == 0) {
                /* if (frame->format == AV_SAMPLE_FMT_FLTP) log("SDFSDF"); */
                /* printf("Data: %d\n", frame->data[0][0]); */
                int planesize = frame->linesize[0] / sizeof(float);
                uint8_t* buff = new uint8_t[planesize * 4];
                for (int i = 0; i < planesize; ++i) {
                    swr_convert(swr, &buff, frame->nb_samples, (const uint8_t**) frame->extended_data, frame->nb_samples);
                    /* /1* printf("CHN1: %f\n", ((float*) frame->extended_data[0])[i]); *1/ */
                    /* /1* printf("CHN2: %f\n", ((float*) frame->extended_data[1])[i]); *1/ */
                    /* float f1 = ((float*) frame->extended_data[0])[i]; */
                    /* float f2 = ((float*) frame->extended_data[1])[i]; */
                    /* buff[i * 2 + 0] = f1 * 32768; */
                    /* buff[i * 2 + 1] = f2 * 32768; */
                    /* // Float view */
                    /* if (f1 >= 1.0) printf("f1: %f\n", f1); */
                    /* if (f2 >= 1.0) printf("f2: %f\n", f2); */
                    /* // Byte view */
                    /* char* view = (char*) &buff[i]; */
                    /* /1* if (view[0] >= 126) printf("0: %d\n", view[0]); *1/ */
                    /* if (view[1] >= 126) printf("1: %d\n", view[1]); */
                    /* /1* if (view[2] >= 126) printf("2: %d\n", view[2]); *1/ */
                    /* if (view[3] >= 126) printf("3: %d\n", view[3]); */
                }
                ao_play(device, (char*) buff, planesize * sizeof(int16_t) * 2);
                delete buff;
            }
        }
        av_packet_unref(&packet);
    }

    av_frame_free(&frame);
    avcodec_free_context(&codec_ctx);

    ao_close(device);
    ao_shutdown();

    return 0;
}
