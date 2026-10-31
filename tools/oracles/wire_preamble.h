// SPDX-License-Identifier: AGPL-3.0-or-later
// Minimal test host for unmodified upstream functions. Never a product library.
#include <stdint.h>
#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdarg.h>

typedef unsigned char byte;
typedef int qboolean;
typedef int netsrc_t;
typedef struct { int type; unsigned short port; } netadr_t;
enum { qfalse = 0, qtrue = 1, NS_CLIENT = 0, NS_SERVER = 1 };
enum { ERR_DROP = 1, CON_CHANNEL_SYSTEM = 0, CON_CHANNEL_NETWORK = 1 };
#define MAX_INFO_STRING 1024
#define MAX_STRING_CHARS 1024
#define BIG_INFO_STRING 8192
#define BIG_INFO_KEY 8192
#define BIG_INFO_VALUE 8192
#define MAX_INFO_VALUE 1024
#define MAX_STRING_TOKENS 1024
#define MAX_PACKETLEN 1400
#define FRAGMENT_SIZE 1300
#define FRAGMENT_BIT (1u << 31)
#define Com_Printf(...) ((void)0)
#define Com_PrintWarning(...) ((void)0)
#define Com_PrintError(...) ((void)0)
#define Com_DPrintf(...) ((void)0)
#define Com_Error(...) abort()
#define Com_sprintf snprintf
#define Com_Memcpy memcpy
#define assertx(condition, ...) assert(condition)

struct snapshotInfo_s { int unused; };

typedef struct {
    byte *data, *splitData;
    int maxsize, cursize, splitSize, readcount, bit, lastRefEntity;
    qboolean overflowed, readonly;
} msg_t;

typedef struct {
    int outgoingSequence, unsentFragmentStart, unsentLength, unsentBufferSize;
    int incomingSequence, sock;
    qboolean unsentFragments;
    byte *unsentBuffer;
    netadr_t remoteAddress;
} netchan_t;

typedef struct { int boolean, integer; } cvar_t;
static cvar_t quiet = {0, 0};
static cvar_t port = {0, 0xbeef};
static cvar_t *showpackets = &quiet;
static cvar_t *qport = &port;
static cvar_t *msg_printEntityNums = &quiet;
static char *netsrcString[2] = {"client", "server"};
static int packet_case;
static int challenge_fixture;
static int fixture_server_challenge, fixture_client_challenge;

typedef struct {
    int cmd_argc, availableBuf;
    char *cmd_tokenized;
    char **cmd_argv;
} cmdTokenizeParams_t;

static void Q_bstrcpy(char *target, const char *source) {
    memmove(target, source, strlen(source) + 1);
}

static void Q_strncpyz(char *target, const char *source, int capacity) {
    size_t length = strlen(source);
    if (length >= (size_t)capacity) length = capacity - 1;
    memcpy(target, source, length);
    target[length] = 0;
}

static int Q_stricmp(const char *left, const char *right) {
    for (;;) {
        unsigned char a = *left++;
        unsigned char b = *right++;
        if (a >= 'A' && a <= 'Z') a += 'a' - 'A';
        if (b >= 'A' && b <= 'Z') b += 'a' - 'A';
        if (a != b || a == 0) return a - b;
    }
}

static void print_hex(const void *data, int length) {
    const unsigned char *bytes = data;
    if (length == 0) putchar('~');
    for (int i = 0; i < length; ++i) printf("%02x", bytes[i]);
}

static qboolean NET_SendPacket(netsrc_t sock, int length, const void *data,
                              netadr_t *address) {
    (void)sock;
    (void)address;
    if (challenge_fixture) {
        printf("challenge %d %d ", fixture_server_challenge, fixture_client_challenge);
    } else {
        printf("packet %d ", packet_case);
    }
    print_hex(data, length);
    putchar('\n');
    return qtrue;
}
