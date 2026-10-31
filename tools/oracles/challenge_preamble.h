// SPDX-License-Identifier: AGPL-3.0-or-later
// Platform/auth hooks are never exercised: this oracle tests response framing.
#ifndef __cdecl
#define __cdecl
#endif
#define __optimize3
#define __regparm1
#define QDECL
#define Q_vsnprintf vsnprintf
#define MAX_MSGLEN 0x20000
#define AUTHORIZE_SERVER_NAME "unused.invalid"
#define PORT_AUTHORIZE 0
enum { NA_IP = 1, NA_IP6, NA_BAD, NA_DOWN };
static struct { netadr_t authorizeAddress; } svs = {{NA_DOWN, 0}};
static const char *fixture_client_argument;
static int NET_CookieHash(netadr_t *address) {
    (void)address; return fixture_server_challenge;
}
static const char *SV_Cmd_Argv(int index) {
    assert(index == 1); return fixture_client_argument;
}
static int NET_StringToAdr(const char *name, netadr_t *address, int type) {
    (void)name; (void)address; (void)type; abort();
}
static int NET_CompareBaseAdr(netadr_t *left, netadr_t *right) {
    (void)left; (void)right; abort();
}
static const char *NET_AdrToString(netadr_t *address) { (void)address; abort(); }
static const char *NET_AdrToStringShort(netadr_t *address) { (void)address; abort(); }
static unsigned short BigShort(unsigned short value) { (void)value; abort(); }
