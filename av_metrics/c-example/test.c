#include <stdio.h>

#include <av_metrics.h>

#define CREATE_METRICS(func, metrics) \
    void test_ ## func(int frame) { \
        const char video_path1[] = "../testfiles/yuv444p8_input.y4m"; \
        const char video_path2[] = "../testfiles/yuv444p8_output.y4m"; \
        const AVMContext *val = avm_calculate_ ## func(&video_path1[0], \
                                                      &video_path2[0], frame); \
        \
        printf("%s - Y: %f  U: %f  V: %f  Avg: %f\n", \
               metrics, val->y, val->u, val->v, val->avg); \
        \
        avm_drop_context(val); \
    }

#define CREATE_CIEDE_METRICS(func) \
    void test_ ## func(int frame) { \
        const char video_path1[] = "../testfiles/yuv444p8_input.y4m"; \
        const char video_path2[] = "../testfiles/yuv444p8_output.y4m"; \
        double val = avm_calculate_ ## func(&video_path1[0], \
                                        &video_path2[0], frame); \
        \
        printf("CIEDE2000 - %f\n", val); \
    }

#define TEST_VIDEO_METRICS(frame) \
    do { \
           printf("\n\nLimit: %d\n\n", frame); \
           test_video_psnr(frame); \
           test_video_apsnr(frame); \
           test_video_psnr_hvs(frame); \
           test_video_ssim(frame); \
           test_video_msssim(frame); \
           test_video_ciede(frame); \
    } while(0)

#define TEST_FRAME_METRICS(frame) \
    do { \
           printf("\n\nFrame: %d\n\n", frame); \
           test_frame_psnr(frame); \
           test_frame_psnr_hvs(frame); \
           test_frame_ssim(frame); \
           test_frame_msssim(frame); \
           test_frame_ciede(frame); \
    } while(0)

CREATE_METRICS(video_psnr, "PSNR");
CREATE_METRICS(video_apsnr, "APSNR");
CREATE_METRICS(video_psnr_hvs, "PSNR_HVS");
CREATE_METRICS(video_ssim, "SSIM");
CREATE_METRICS(video_msssim, "MSSSIM");
CREATE_CIEDE_METRICS(video_ciede);

CREATE_METRICS(frame_psnr, "PSNR");
CREATE_METRICS(frame_psnr_hvs, "PSNR_HVS");
CREATE_METRICS(frame_ssim, "SSIM");
CREATE_METRICS(frame_msssim, "MSSSIM");
CREATE_CIEDE_METRICS(frame_ciede);

int main() {

    // Test metrics on videos
    TEST_VIDEO_METRICS(0);
    TEST_VIDEO_METRICS(2);

    // Test metrics on frames
    TEST_FRAME_METRICS(0);
    TEST_FRAME_METRICS(2);

    return 0;
}
