/* SPDX-License-Identifier: AGPL-3.0-or-later */
Word VM_ExecuteInternal(void)
{
    Word key = keys[current], values = ((unsigned short *)archives[current])[2];
    require(++executions <= 16 && freed[current], "resume bounded body after archive free");
    require(gFs[0] == archives[current][0] && gFs[1] == key && gFs[2] == 0,
            "resume restored instruction/local frame");
    require(gFs[3] == (Word)gScrVmPub + 0x320 + 8 * values
            && gFs[4] == (Word)gScrVmPub + 0x320, "resume restored value stack");
    require(gScrVmPub[2] == 1 && gScrVmPub[3] == (Word)gScrVmPub + 0x38
            && gScrVmPub[8] == archives[current][0] && gScrVmPub[15] == key,
            "resume restored call frame");
    require(thread_count == 0 && restored[current], "resume cleared thread state");
    for (Word item = 0; item < values; ++item) {
        require(gScrVmPub[0x328 / 4 + 2 * item] == 40 + item
                && gScrVmPub[0x32c / 4 + 2 * item] == (item == 2 ? 0 : 6),
                "resume restored scalar archive");
    }
    text("S "); decimal(key); text(" "); decimal(gFs[0]); text(" "); decimal(gScrVmPub[2]);
    text(" "); decimal(gFs[2]); text(" "); decimal(values);
    for (Word item = 0; item < values; ++item) {
        text(" "); decimal(gScrVmPub[0x328 / 4 + 2 * item]);
        text(" "); decimal(gScrVmPub[0x32c / 4 + 2 * item]);
    }
    text("\n");
    event("E", key);
    if (scenario == 3 && key == 1 && executions < 3) enqueue(1, 0, 0, 7);
    if (scenario == 4 && key == 1) enqueue(3, 1, 0, 7);
    if (scenario == 5 && key == 1) enqueue(3, 0, 0, 7);
    if (scenario == 6 && key == 1) { enqueue(3, 0, 0, 7); enqueue(4, 1, 3, 8); }
    gScrVmPub[2] = 0;
    gScrVmPub[3] = (Word)gScrVmPub + 0x20;
    gScrVmPub[0x328 / 4] = executions;
    gScrVmPub[0x32c / 4] = 6;
    return 900 + executions;
}
static void reset(void)
{
    for (Word item = 0; item < sizeof(gScrVmPub) / 4; ++item) gScrVmPub[item] = 0;
    for (Word item = 0; item < 32; ++item) {
        gScrVarPub[item] = restored[item] = freed[item] = 0;
        for (Word word = 0; word < 16; ++word) archives[item][word] = 0;
    }
    for (Word item = 0; item < 5; ++item) gFs[item] = 0;
    queued = allocated = current = executions = clocks = bucket_refs = 0;
    clears = frees = decrements = value_drops = object_drops = 0;
    gScrVmPub[0] = (Word)local_variables;
    gScrVmPub[3] = (Word)gScrVmPub + 0x20;
    gScrVmPub[4] = (Word)gScrVmPub + 0x320;
    gScrVarPub[5] = 7;
    gScrVarPub[12] = 88;
    thread_count = 123;
}
void scope_start(void)
{
    for (scenario = 0; scenario < 10; ++scenario) {
        reset(); event("R", scenario);
        if (scenario) enqueue(1, 0, scenario == 8 ? 3 : scenario == 7 ? 1 : 0, scenario == 9 ? 8 : 7);
        if (scenario == 2) { enqueue(2, 0, 0, 7); enqueue(3, 0, 0, 7); }
        if (scenario >= 4 && scenario <= 6) enqueue(2, 0, 0, 7);
        VM_Resume(77);
        require(!bucket_refs && !queued && gScrVarPub[31] == 1
                && gScrVmPub[4] == (Word)gScrVmPub + 0x320, "resume final state");
        text("G "); decimal(executions); text(" "); decimal(clocks); text(" "); decimal(clears);
        text(" "); decimal(frees); text(" "); decimal(decrements); text(" "); decimal(value_drops);
        text(" "); decimal(object_drops); text(" "); decimal(gScrVmGlob[6]); text("\n");
    }
    scope_exit(0);
}
