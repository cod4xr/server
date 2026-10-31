// SPDX-License-Identifier: AGPL-3.0-or-later
static void obfuscation_vectors(void) {
    const char *commands[] = {"", "status", "hello %quoted", "\xff\x80raw"};
    const int lengths[] = {0, 1, 2, 3, 31, 128};
    client_t client = {0};
    client.challenge = 0x1234abcd;
    client.netchan.outgoingSequence = 0x101;
    client.serverId = 0x55;
    client.messageAcknowledge = 0x1234;
    client.reliableAcknowledge = 3;
    for (int direction = 0; direction < 2; ++direction) {
        for (int command = 0; command < 4; ++command) {
            memset(client.lastClientCommandString, 0, sizeof(client.lastClientCommandString));
            memset(client.reliableCommands[3].command, 0, sizeof(client.reliableCommands[3].command));
            strcpy(client.lastClientCommandString, commands[command]);
            strcpy(client.reliableCommands[3].command, commands[command]);
            for (int sample = 0; sample < 6; ++sample) {
                int length = lengths[sample];
                byte original[128], transformed[132] = {0};
                for (int i = 0; i < length; ++i) original[i] = (i * 73 + 11) & 0xff;
                if (direction == 0) {
                    transformed[0] = 0xaa;
                    transformed[1] = 0xbb;
                    transformed[2] = 0xcc;
                    transformed[3] = 0xdd;
                    memcpy(transformed + 4, original, length);
                    SV_Netchan_Encode(&client, transformed, length + 4);
                    assert(transformed[0] == 0xaa && transformed[3] == 0xdd);
                } else {
                    memcpy(transformed, original, length);
                    SV_Netchan_Decode(&client, transformed, length);
                }
                printf("xor %d ", direction);
                print_hex(commands[command], strlen(commands[command]));
                putchar(' ');
                print_hex(original, length);
                putchar(' ');
                print_hex(transformed + (direction == 0 ? 4 : 0), length);
                putchar('\n');
            }
        }
    }
    // Record the original empty-string bug separately: the routine advances
    // into stale tail bytes after reading an empty command's initial NUL.
    byte stale_command[] = {0, 'X', 'Y', 0};
    byte original[] = {11, 84, 157, 230};
    byte transformed[sizeof(original)];
    memset(client.reliableCommands[3].command, 0, sizeof(client.reliableCommands[3].command));
    memcpy(client.reliableCommands[3].command, stale_command, sizeof(stale_command));
    memcpy(transformed, original, sizeof(original));
    SV_Netchan_Decode(&client, transformed, sizeof(transformed));
    printf("stale-empty ");
    print_hex(stale_command, sizeof(stale_command));
    putchar(' ');
    print_hex(original, sizeof(original));
    putchar(' ');
    print_hex(transformed, sizeof(transformed));
    putchar('\n');
}

int main(void) {
    obfuscation_vectors();
    return 0;
}
