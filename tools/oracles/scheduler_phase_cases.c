/* SPDX-License-Identifier: AGPL-3.0-or-later */
int main(void)
{
    for (int mode = 0; mode < 5; ++mode) {
        for (int boundary = 0; boundary < 2; ++boundary) {
            for (int operation = 0; operation < 3; ++operation) {
                printf("T %d %d %d\n", mode, boundary, operation);
                enabled.boolean = mode != 0;
                logging.boolean = mode == 4;
                gScrVarPub.time = boundary ? VAR_NAME_LOW_MASK : 7;
                gScrVarPub.timeArrayId = mode == 1 ? 0 : 100;
                bucket = mode != 2;
                gScrExecuteTime = 100;
                raw_calls = resumes = logs = frees = 0;
                if (operation != 1) Scr_RunCurrentThreads();
                if (operation != 0) Scr_IncTime();
                printf("G %u %d %d %d %d %d %d\n", gScrVarPub.time,
                       gScrExecuteTime, raw_calls, resumes, logs, frees, bucket);
            }
        }
    }
    return 0;
}
