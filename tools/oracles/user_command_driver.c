// SPDX-License-Identifier: AGPL-3.0-or-later
// Deterministic input generation only. Decoding functions are original source.
static uint32_t random_state = 0x725394abu;

static uint32_t next_random(void) {
    random_state ^= random_state << 13;
    random_state ^= random_state >> 17;
    random_state ^= random_state << 5;
    return random_state;
}

static void print_command(const usercmd_t *command) {
    uint32_t yaw;
    memcpy(&yaw, &command->meleeChargeYaw, sizeof(yaw));
    printf("%d,%u,%u,%u,%u,%u,%u,%d,%d,%08x,%u,%u,%u",
        command->serverTime, (unsigned)command->buttons,
        (unsigned)command->angles[0], (unsigned)command->angles[1],
        (unsigned)command->angles[2], (unsigned)command->weapon,
        (unsigned)command->offHandIndex, (int)command->forwardmove,
        (int)command->rightmove, yaw, (unsigned)command->meleeChargeDist,
        (unsigned)command->selectedLocation[0], (unsigned)command->selectedLocation[1]);
}

static uint64_t all_melee_angles(void) {
    uint64_t hash = UINT64_C(0xcbf29ce484222325);
    for (unsigned raw = 0; raw < 65536; ++raw) {
        byte data[64];
        msg_t message;
        MSG_Init(&message, data, sizeof(data));
        MSG_WriteBit1(&message);  // byte timestamp delta
        MSG_WriteByte(&message, 0);
        MSG_WriteBit1(&message);  // changed
        MSG_WriteBit1(&message);  // full branch
        MSG_WriteBit0(&message);  // primary button
        MSG_WriteBit0(&message);  // pitch unchanged
        MSG_WriteBit0(&message);  // yaw unchanged
        MSG_WriteBit0(&message);  // movement unchanged
        MSG_WriteBit0(&message);  // roll unchanged
        MSG_WriteBit0(&message);  // upper buttons unchanged
        MSG_WriteBit0(&message);  // weapon unchanged
        MSG_WriteBit0(&message);  // offhand unchanged
        MSG_WriteBit1(&message);  // melee angle changed
        MSG_WriteShort(&message, raw);
        MSG_WriteBit0(&message);  // distance unchanged
        usercmd_t baseline = {0}, decoded;
        baseline.buttons = 4;
        MSG_BeginReading(&message);
        MSG_ReadDeltaUsercmdKey(&message, 0, &baseline, &decoded);
        assert(!message.overflowed);
        uint32_t bits;
        memcpy(&bits, &decoded.meleeChargeYaw, sizeof(bits));
        for (unsigned byte = 0; byte < 4; ++byte) {
            hash ^= (bits >> (byte * 8)) & 255;
            hash *= UINT64_C(0x100000001b3);
        }
    }
    return hash;
}

int main(void) {
    _Static_assert(sizeof(int) == 4 && sizeof(float) == 4, "Original command profile");
    printf("# melee_angles_fnv64 %016llx\n", (unsigned long long)all_melee_angles());
    for (unsigned sample = 0; sample < 512; ++sample) {
        byte data[64];
        for (unsigned i = 0; i < sizeof(data); ++i) data[i] = next_random();
        usercmd_t baseline = {0};
        baseline.serverTime = (int)(next_random() % 1000000);
        if (sample % 11 == 0) baseline.serverTime = INT32_MAX - 10;
        baseline.buttons = next_random() & 0x1fffff;
        for (unsigned axis = 0; axis < 3; ++axis) baseline.angles[axis] = next_random() & 65535;
        baseline.weapon = next_random() & 127;
        baseline.offHandIndex = next_random() & 127;
        // Deliberately cover both threshold boundaries and noncanonical baseline axes.
        static const char axes[] = {-128, -127, -11, -10, 0, 10, 11, 127};
        baseline.forwardmove = axes[sample % 8];
        baseline.rightmove = axes[(sample / 8) % 8];
        baseline.meleeChargeYaw = (float)((int)(next_random() % 65536) - 32768) * (360.0f / 65536.0f);
        if (sample % 7 == 0) baseline.meleeChargeYaw = 18.453125f;
        baseline.meleeChargeDist = next_random();
        baseline.selectedLocation[0] = next_random();
        baseline.selectedLocation[1] = next_random();
        unsigned key = next_random();
        // Force explicit no-change, lite, and full branches in addition to random cases.
        if (sample % 8 == 0) { key = 0; data[0] = 1; }
        if (sample % 8 == 1) { key = 0; data[0] = 3; }
        if (sample % 8 == 2) { key = 0; data[0] = 15; }
        msg_t message;
        MSG_Init(&message, data, sizeof(data));
        message.cursize = sizeof(data);
        MSG_BeginReading(&message);
        int signed_key;
        memcpy(&signed_key, &key, sizeof(signed_key));
        usercmd_t decoded;
        MSG_ReadDeltaUsercmdKey(&message, signed_key, &baseline, &decoded);
        assert(!message.overflowed);
        int used = message.readcount;
        unsigned tail_bits = MSG_ReadBits(&message, 3);
        unsigned tail_byte = MSG_ReadByte(&message);
        printf("%08x ", key);
        for (unsigned i = 0; i < sizeof(data); ++i) printf("%02x", data[i]);
        putchar(' '); print_command(&baseline);
        putchar(' '); print_command(&decoded);
        printf(" %d %u %u\n", used, tail_bits, tail_byte);
    }
    return 0;
}
