// SPDX-License-Identifier: AGPL-3.0-or-later
static void packet_vectors(void) {
    const int lengths[] = {0, 1, 1299, 1300, 1301, 2600};
    byte body[2600], unsent[4096];
    for (int i = 0; i < 2600; ++i) body[i] = (i * 7 + 3) & 0xff;
    packet_case = 0;
    for (int route = 0; route < 2; ++route) {
        for (int i = 0; i < 6; ++i) {
            netchan_t channel = {0};
            channel.outgoingSequence = 0x12345678;
            channel.sock = route;
            channel.unsentBuffer = unsent;
            channel.unsentBufferSize = sizeof(unsent);
            Netchan_Transmit(&channel, lengths[i], body);
            while (channel.unsentFragments) Netchan_TransmitNextFragment(&channel);
            ++packet_case;
        }
    }
}

static void token_vectors(void) {
    const char *cases[] = {
        "", "getinfo abc", "GETSTATUS \"hello world\"", "a\"b\"c",
        "a/* hidden */b//rest", "\"\"\"\"", "a /* unterminated",
        "\"unterminated value", "a \t\r b", "x/y / z", "a%test", "name \xff\xfe"
    };
    for (unsigned int i = 0; i < sizeof(cases) / sizeof(cases[0]); ++i) {
        char storage[10000];
        char *arguments[MAX_STRING_TOKENS + 1];
        cmdTokenizeParams_t params = {0, sizeof(storage), storage, arguments};
        Cmd_TokenizeStringInternal(cases[i], qfalse, &params);
        printf("tokens ");
        print_hex(cases[i], strlen(cases[i]));
        printf(" %d ", params.cmd_argc);
        if (!params.cmd_argc) putchar('-');
        for (int j = 0; j < params.cmd_argc; ++j) {
            if (j) putchar(',');
            print_hex(arguments[j], strlen(arguments[j]));
        }
        putchar('\n');
    }
}

static void info_vectors(void) {
    const char *keys[] = {
        "hostname", "protocol", "hostname", "Key", "key", "Key", "mapname", "key"
    };
    const char *values[] = {
        "Rust", "6", "Updated", "One", "Two", "", "mp_crash", "Final"
    };
    char info[MAX_INFO_STRING] = {0};
    for (int i = 0; i < 8; ++i) {
        Info_SetValueForKey(info, keys[i], values[i]);
        printf("info %d ", i);
        print_hex(info, strlen(info));
        putchar(' ');
        const char *value = Info_ValueForKey(info, "KEY");
        print_hex(value, strlen(value));
        putchar('\n');
    }
}

static uint32_t next_index_random(uint32_t *state) {
    *state ^= *state << 13;
    *state ^= *state >> 17;
    *state ^= *state << 5;
    return *state;
}

static void index_vectors(void) {
    const int kinds[] = {6, 10};
    const int entity_boundary[] = {0, 1, 15, 16, 31, 32, 48, 1022, 1023};
    const int client_boundary[] = {0, 1, 2, 15, 16, 31, 32, 62, 63};
    for (int kind = 0; kind < 2; ++kind) {
        int bits = kinds[kind];
        for (int seed = 0; seed < 32; ++seed) {
            byte bytes[512] = {0};
            int indices[32], count = 0, index = -1;
            uint32_t state = seed + 1;
            msg_t msg;
            MSG_Init(&msg, bytes, sizeof(bytes));
            MSG_ClearLastReferencedEntity(&msg);
            while (count < 32) {
                if (seed == 0) {
                    if (count == 9) break;
                    index = kind ? entity_boundary[count] : client_boundary[count];
                } else {
                    index += 1 + next_index_random(&state) % 20;
                }
                if (index >= (1 << bits)) break;
                indices[count++] = index;
                MSG_WriteEntityIndex(NULL, &msg, index, bits);
            }
            printf("index %d ", bits);
            print_hex(bytes, msg.cursize);
            putchar(' ');
            MSG_BeginReading(&msg);
            MSG_ClearLastReferencedEntity(&msg);
            for (int i = 0; i < count; ++i) {
                if (i) putchar(',');
                int decoded = MSG_ReadEntityIndex(&msg, bits);
                assert(decoded == indices[i]);
                printf("%d", decoded);
            }
            putchar('\n');
        }
    }
}

static void string_vectors(void) {
    const char *values[] = {"", "hello", "a%b", "a\nb", "\xff\x80raw", "\"quotes\""};
    for (int big = 0; big < 2; ++big) {
        for (int sample = 0; sample < 6; ++sample) {
            byte bytes[128] = {0};
            char decoded[8192];
            msg_t msg;
            MSG_Init(&msg, bytes, sizeof(bytes));
            if (big) MSG_WriteBigString(&msg, values[sample]);
            else MSG_WriteString(&msg, values[sample]);
            printf("string %d ", big);
            print_hex(bytes, msg.cursize);
            putchar(' ');
            print_hex(values[sample], strlen(values[sample]));
            putchar(' ');
            MSG_BeginReading(&msg);
            MSG_ReadString(&msg, decoded, big ? 8192 : 1024);
            print_hex(decoded, strlen(decoded));
            putchar('\n');
        }
    }
}

static void connect_vectors(void) {
    const char *cases[] = {
        "connect \"\\protocol\\6\\challenge\\0\\qport\\0\"",
        "connect \"\\protocol\\21\\challenge\\-2147483648\\qport\\-32768\"",
        "connect \"\\protocol\\21\\challenge\\2147483647\\qport\\65535\"",
        "CONNECT \"\\PROTOCOL\\21\\Challenge\\-1\\QPORT\\-1\"",
        "/* prefix */ connect \"protocol\\6\\challenge\\123\\qport\\32767\"",
        "connect \"\\protocol\\ +00021 \\challenge\\ +042 \\qport\\ +00042 \"",
        "connect \"\\protocol\\21\\challenge\\-00123\\qport\\32768\\future\\yes\" trailing",
        "connect \"\\protocol\\0\\challenge\\1\\qport\\10\"",
        "connect \"\\protocol\\2147483647\\challenge\\2\\qport\\11\"",
        "connect \"\\protocol\\21\\challenge\\0\\qport\\12\\name\\Player\xff\"",
        "connect \"\\protocol\\21\\challenge\\1\\qport\\13\\password\\synthetic secret\"",
        "connect \"\\protocol\\21\\challenge\\2\\qport\\14\\xver\\21.2\" // rest"
    };
    for (unsigned i = 0; i < sizeof(cases) / sizeof(cases[0]); ++i) {
        char storage[10000];
        char *arguments[MAX_STRING_TOKENS + 1];
        cmdTokenizeParams_t params = {0, sizeof(storage), storage, arguments};
        Cmd_TokenizeStringInternal(cases[i], qfalse, &params);
        assert(params.cmd_argc >= 2);
        int protocol = atoi(Info_ValueForKey(arguments[1], "protocol"));
        int challenge = atoi(Info_ValueForKey(arguments[1], "challenge"));
        unsigned short port = atoi(Info_ValueForKey(arguments[1], "qport"));
        printf("connect ");
        print_hex(cases[i], strlen(cases[i]));
        printf(" %d %d %u\n", protocol, challenge, (unsigned)port);
    }
}

static void challenge_vectors(void) {
    const int server_values[] = {0, 1, -1, INT32_MIN, INT32_MAX};
    const char *client_values[] = {"0", "1", "-1", "-2147483648", "2147483647",
                                    " +00042 ", "-00012", ""};
    netadr_t source = {NA_IP6, 28960};
    challenge_fixture = 1;
    for (unsigned server = 0; server < 5; ++server) {
        for (unsigned client = 0; client < 8; ++client) {
            fixture_server_challenge = server_values[server];
            fixture_client_argument = client_values[client];
            fixture_client_challenge = atoi(fixture_client_argument);
            SV_GetChallenge(&source);
        }
    }
}

int main(void) {
    packet_vectors();
    token_vectors();
    info_vectors();
    index_vectors();
    string_vectors();
    connect_vectors();
    challenge_vectors();
    return 0;
}
