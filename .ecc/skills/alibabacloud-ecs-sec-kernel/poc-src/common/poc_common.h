/**
 * sec-kernel PoC/Exp common header
 *
 * Defines unified output format, safety constraints, CTF challenge mode
 * CLI argument parsing framework, and syscall logging.
 * All PoC/Exp C programs must #include this file.
 */
#ifndef SEC_KERNEL_POC_COMMON_H
#define SEC_KERNEL_POC_COMMON_H

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdarg.h>
#include <unistd.h>
#include <signal.h>
#include <time.h>
#include <sys/types.h>
#include <errno.h>

/* ========================================================================
 * Output format constants
 * Python framework parses stdout to get results
 * ======================================================================== */
#define POC_RESULT_EXPLOITABLE     "POC_RESULT:EXPLOITABLE"
#define POC_RESULT_NOT_EXPLOITABLE "POC_RESULT:NOT_EXPLOITABLE"
#define POC_RESULT_ERROR           "POC_RESULT:ERROR"

#define POC_EVIDENCE_BEGIN         "POC_EVIDENCE_BEGIN"
#define POC_EVIDENCE_END           "POC_EVIDENCE_END"

#define POC_STEP_PASS              "[PASS]"
#define POC_STEP_FAIL              "[FAIL]"
#define POC_STEP_INFO              "[INFO]"
#define POC_STEP_WARN              "[WARN]"

/* ========================================================================
 * Safety constraints
 * ======================================================================== */
#define POC_MAX_RUNTIME_SEC   10    /* Maximum runtime (seconds) */
#define POC_NO_SYSTEM_MODIFY  1     /* Modification of system files is prohibited */

/* Evidence output directory environment variable */
#define POC_EVIDENCE_DIR_ENV  "SEC_KERNEL_OUTPUT"

/* Legacy environment variable for target file (backward compat) */
#define POC_TARGET_FILE_ENV   "POC_TARGET_FILE"

/* ========================================================================
 * CTF mode definitions
 * ======================================================================== */
#define POC_MODE_READ   "read_root_file"
#define POC_MODE_WRITE  "write_root_file"
#define POC_MODE_UAF    "uaf"

/* ========================================================================
 * UAF spray corruption verification
 * ======================================================================== */
#define UAF_SPRAY_PATTERN     0xDEADBEEF
#define UAF_FLAG_CORRUPTED    "UAF_FLAG:CORRUPTED"
#define UAF_FLAG_NOT_CORRUPTED "UAF_FLAG:NOT_CORRUPTED"
#define UAF_RESULT_EXPLOITABLE    "UAF_RESULT:EXPLOITABLE"
#define UAF_RESULT_NOT_EXPLOITABLE "UAF_RESULT:NOT_EXPLOITABLE"

/* Forward declarations - implementations after poc_log */
static void __attribute__((unused)) poc_print_uaf_corrupted(int count);
static void __attribute__((unused)) poc_print_uaf_not_corrupted(void);

/* ========================================================================
 * CTF argument structure
 * ======================================================================== */
#define MAX_SPRAY_FDS 512

typedef struct {
    const char *mode;        /* "read_root_file" | "write_root_file" | "uaf" */
    const char *root_file;   /* root privilege target file path */
    const char *write_value; /* CTF write value (write mode) */
    const char *log_file;    /* log file path (workspace/poc-CVE-XXXX-XXXXX.log) */
    int verbose;             /* verbose output switch */
    int spray_fds[MAX_SPRAY_FDS]; /* spray file descriptors for UAF mode */
    int spray_count;         /* number of spray objects allocated */
} poc_args_t;

/* ========================================================================
 * Safety initialization (constructor - auto-invoked before main)
 * ======================================================================== */

static void poc_alarm_handler(int sig) {
    (void)sig;
    fprintf(stderr, "POC_RESULT:ERROR\nTimeout after %d seconds\n", POC_MAX_RUNTIME_SEC);
    _exit(124);
}

static void __attribute__((constructor)) poc_safety_init(void) {
    /* Set timeout alarm */
    signal(SIGALRM, poc_alarm_handler);
    alarm(POC_MAX_RUNTIME_SEC);

    /* Ensure stdout is unbuffered (real-time output) */
    setvbuf(stdout, NULL, _IONBF, 0);
}

/* ========================================================================
 * Syscall logging system
 * ======================================================================== */

/* Global log file pointer */
static FILE *g_poc_log_fp __attribute__((unused)) = NULL;

/**
 * Get formatted timestamp for logging
 */
static inline void poc_log_timestamp(char *buf, size_t len) {
    time_t now = time(NULL);
    struct tm *tm_info = localtime(&now);
    strftime(buf, len, "[%Y-%m-%d %H:%M:%S]", tm_info);
}

/**
 * Initialize log file
 * Call after poc_parse_args() succeeds, or manually with a path.
 */
static void __attribute__((unused)) poc_log_init(const char *log_file) {
    if (!log_file || log_file[0] == '\0') return;
    g_poc_log_fp = fopen(log_file, "a");
    if (!g_poc_log_fp) {
        /* Fallback: extract basename and write to /tmp/ */
        const char *basename = strrchr(log_file, '/');
        if (basename) basename++; else basename = log_file;

        char fallback[256];
        snprintf(fallback, sizeof(fallback), "/tmp/%s", basename);
        g_poc_log_fp = fopen(fallback, "a");

        if (g_poc_log_fp) {
            fprintf(stdout, "%s Log fallback to: %s\n", POC_STEP_WARN, fallback);
        } else {
            fprintf(stderr, "%s Failed to open log file: %s and fallback %s (%s)\n",
                    POC_STEP_WARN, log_file, fallback, strerror(errno));
        }
    }
}

/**
 * Close log file
 */
static void __attribute__((unused)) poc_log_close(void) {
    if (g_poc_log_fp) {
        fclose(g_poc_log_fp);
        g_poc_log_fp = NULL;
    }
}

/**
 * Write log entry (outputs to both stdout and log file if open)
 */
static void __attribute__((unused)) poc_log(const char *fmt, ...) {
    char ts[32];
    poc_log_timestamp(ts, sizeof(ts));

    va_list args;

    /* stdout */
    printf("%s ", ts);
    va_start(args, fmt);
    vprintf(fmt, args);
    va_end(args);
    printf("\n");

    /* log file */
    if (g_poc_log_fp) {
        fprintf(g_poc_log_fp, "%s ", ts);
        va_start(args, fmt);
        vfprintf(g_poc_log_fp, fmt, args);
        va_end(args);
        fprintf(g_poc_log_fp, "\n");
        fflush(g_poc_log_fp);
    }
}

/**
 * Log a syscall result with errno details
 *
 * Example output:
 *   [2026-05-08 12:34:56] syscall: socket(AF_ALG, SOCK_SEQPACKET, 0) = 3 (errno=0 OK)
 *   [2026-05-08 12:34:56] syscall: splice(4, NULL, pipe[1], NULL, 4096, 0) = -1 (errno=1 EPERM)
 */
static void __attribute__((unused)) poc_log_syscall(const char *syscall_name, long ret, int saved_errno) {
    char ts[32];
    poc_log_timestamp(ts, sizeof(ts));

    const char *err_str = (saved_errno == 0) ? "OK" : strerror(saved_errno);

    /* stdout */
    printf("%s syscall: %s = %ld (errno=%d %s)\n", ts, syscall_name, ret, saved_errno, err_str);

    /* log file */
    if (g_poc_log_fp) {
        fprintf(g_poc_log_fp, "%s syscall: %s = %ld (errno=%d %s)\n",
                ts, syscall_name, ret, saved_errno, err_str);
        fflush(g_poc_log_fp);
    }
}

/**
 * Output UAF corruption result
 */
static void __attribute__((unused)) poc_print_uaf_corrupted(int count) {
    printf("UAF_FLAG:CORRUPTED(%d objects)\n", count);
    printf("UAF_RESULT:EXPLOITABLE\n");
    poc_log("UAF_FLAG:CORRUPTED(%d objects)", count);
}

/**
 * Output UAF not corrupted result
 */
static void __attribute__((unused)) poc_print_uaf_not_corrupted(void) {
    printf("UAF_FLAG:NOT_CORRUPTED\n");
    printf("UAF_RESULT:NOT_EXPLOITABLE\n");
    poc_log("UAF_FLAG:NOT_CORRUPTED");
}

/* ========================================================================
 * CTF result output functions
 * ======================================================================== */

/* CTF write_root_file mode: read-before / write-attempt / read-after markers */
#define CTF_READ_BEFORE   "CTF_READ_BEFORE:"
#define CTF_WRITE_ATTEMPT "CTF_WRITE:"
#define CTF_READ_AFTER    "CTF_READ_AFTER:"

/**
 * Output CTF flag (successful read content or write confirmation)
 */
static void __attribute__((unused)) poc_print_flag(const char *flag) {
    printf("CTF_FLAG:%s\n", flag);
    poc_log("CTF_FLAG:%s", flag);
}

/**
 * Output CTF failure reason
 */
static void __attribute__((unused)) poc_print_fail(const char *reason) {
    printf("CTF_FAIL:%s\n", reason);
    poc_log("CTF_FAIL:%s", reason);
}

/**
 * Output unsupported mode message
 */
static void __attribute__((unused)) poc_print_unsupported(const char *mode, const char *reason) {
    printf("CTF_UNSUPPORTED:%s:%s\n", mode, reason);
    poc_log("CTF_UNSUPPORTED mode=%s reason=%s", mode, reason);
}

/* ========================================================================
 * CTF CLI argument parsing
 * ======================================================================== */

/**
 * Print usage/help for CTF mode
 */
static void __attribute__((unused)) poc_print_usage(const char *prog_name) {
    fprintf(stderr, "Usage: %s [OPTIONS]\n", prog_name ? prog_name : "poc");
    fprintf(stderr, "\nCTF Challenge Mode Options:\n");
    fprintf(stderr, "  --mode <mode>         Challenge mode: read_root_file|write_root_file|uaf\n");
    fprintf(stderr, "  --root-file <path>    Target file path requiring root privilege\n");
    fprintf(stderr, "  --write-value <val>   Value to write (for write mode)\n");
    fprintf(stderr, "  --log-file <path>     Log file path\n");
    fprintf(stderr, "  --verbose             Enable verbose output\n");
    fprintf(stderr, "  --help                Show this help message\n");
    fprintf(stderr, "\nLegacy Mode (no arguments):\n");
    fprintf(stderr, "  Uses environment variables: SEC_KERNEL_OUTPUT, POC_TARGET_FILE\n");
}

/**
 * Parse CTF command-line arguments
 *
 * Supports:
 *   --mode read_root_file|write_root_file
 *   --root-file /path/to/target
 *   --write-value ctf{XXXX}
 *   --log-file /path/to/log
 *   --verbose
 *   --help
 *
 * Returns 0 on success, -1 on failure (invalid args or --help requested)
 *
 * If argc <= 1 (no args), returns 0 with args zeroed (legacy mode compat).
 */
static int __attribute__((unused)) poc_parse_args(int argc, char *argv[], poc_args_t *args) {
    if (!args) return -1;

    /* Initialize defaults */
    memset(args, 0, sizeof(poc_args_t));
    args->mode = NULL;
    args->root_file = NULL;
    args->write_value = NULL;
    args->log_file = NULL;
    args->verbose = 0;

    /* No arguments = legacy mode, return success with empty struct */
    if (argc <= 1) return 0;

    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "--help") == 0 || strcmp(argv[i], "-h") == 0) {
            poc_print_usage(argv[0]);
            return -1;
        } else if (strcmp(argv[i], "--mode") == 0) {
            if (i + 1 >= argc) {
                fprintf(stderr, "Error: --mode requires an argument\n");
                return -1;
            }
            args->mode = argv[++i];
            /* Validate mode */
            if (strcmp(args->mode, POC_MODE_READ) != 0 &&
                strcmp(args->mode, POC_MODE_WRITE) != 0 &&
                strcmp(args->mode, POC_MODE_UAF) != 0) {
                fprintf(stderr, "Error: invalid mode '%s' (expected: %s|%s|%s)\n",
                        args->mode, POC_MODE_READ, POC_MODE_WRITE, POC_MODE_UAF);
                return -1;
            }
        } else if (strcmp(argv[i], "--root-file") == 0) {
            if (i + 1 >= argc) {
                fprintf(stderr, "Error: --root-file requires an argument\n");
                return -1;
            }
            args->root_file = argv[++i];
        } else if (strcmp(argv[i], "--write-value") == 0) {
            if (i + 1 >= argc) {
                fprintf(stderr, "Error: --write-value requires an argument\n");
                return -1;
            }
            args->write_value = argv[++i];
        } else if (strcmp(argv[i], "--log-file") == 0) {
            if (i + 1 >= argc) {
                fprintf(stderr, "Error: --log-file requires an argument\n");
                return -1;
            }
            args->log_file = argv[++i];
        } else if (strcmp(argv[i], "--verbose") == 0 || strcmp(argv[i], "-v") == 0) {
            args->verbose = 1;
        } else {
            fprintf(stderr, "Error: unknown option '%s'\n", argv[i]);
            poc_print_usage(argv[0]);
            return -1;
        }
    }

    /* Validate: if mode is set, root-file is required */
    if (args->mode && !args->root_file) {
        fprintf(stderr, "Error: --root-file is required when --mode is specified\n");
        return -1;
    }

    /* Validate: write mode requires write-value */
    if (args->mode &&
        strcmp(args->mode, POC_MODE_WRITE) == 0 &&
        !args->write_value) {
        fprintf(stderr, "Error: --write-value is required for mode '%s'\n", args->mode);
        return -1;
    }

    return 0;
}

/* ========================================================================
 * Legacy helper functions (backward compatible)
 * ======================================================================== */

/**
 * Get current timestamp string (ISO format)
 */
static inline void poc_get_timestamp(char *buf, size_t len) {
    time_t now = time(NULL);
    struct tm *tm_info = localtime(&now);
    strftime(buf, len, "%Y-%m-%dT%H:%M:%S%z", tm_info);
}

/**
 * Get evidence output directory from environment
 */
static inline const char* poc_get_output_dir(void) {
    const char *dir = getenv(POC_EVIDENCE_DIR_ENV);
    return dir ? dir : "/tmp";
}

/**
 * Print step result (legacy format)
 */
static inline void poc_print_step(const char *status, const char *fmt, ...) {
    va_list args;
    printf("%s ", status);
    va_start(args, fmt);
    vprintf(fmt, args);
    va_end(args);
    printf("\n");
}

/**
 * Print system info for evidence collection
 */
static inline void poc_print_system_info(void) {
    char timestamp[64];
    poc_get_timestamp(timestamp, sizeof(timestamp));

    printf("%s\n", POC_EVIDENCE_BEGIN);
    printf("Timestamp: %s\n", timestamp);
    printf("Unix Epoch: %ld\n", (long)time(NULL));
    printf("UID: %d\n", getuid());
    printf("EUID: %d\n", geteuid());
    printf("GID: %d\n", getgid());
    printf("PID: %d\n", getpid());
}

#endif /* SEC_KERNEL_POC_COMMON_H */
