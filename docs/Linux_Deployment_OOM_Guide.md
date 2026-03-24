# Linux Server Deployment OOM (Out Of Memory) Troubleshooting & Fix Guide

*English | [中文](#linux-服务器部署-oom-排查与修复指南)*

If you deploy `Antigravity-Tools-LS` on a Linux server and suddenly encounter `UnexpectedEof`, `transport error`, or connection drops without any application-level error stacks when sending messages (e.g., invoking `SendUserCascadeMessage`), it is highly likely that **the process was killed by the OS due to insufficient memory**.

This document logs troubleshooting methodologies and standard solutions for low-spec servers.

## 1. Symptom Description

- **Server-side Log Characteristics**:
  The Language Server (`ls_core`) initializes normally:
  `initialized server successfully in 898ms`
  However, during subsequent conversation processing, the application process logs **suddenly stop**.

- **Client Proxy (Rust) Error Characteristics**:
  Because the underlying Language Server is suddenly destroyed by the OS, the waiting client receives a TCP reset and catches exceptions like:
  ```
  SendUserCascadeMessage failed. Status: Status { code: Unknown, message: "transport error", source: Some(tonic::transport::Error(Transport, hyper::Error(Io, Custom { kind: UnexpectedEof, error: "peer closed connection without sending TLS close_notify...
  ```

## 2. Root Cause: Why Out of Memory?

The native Language Server relies on a complex stack of CGO dependencies and low-level processing modules (e.g., Tokenizers, local computations, or miniature ONNX models). When processing conversation contexts, it may **instantaneously request massive amounts of Virtual Memory (VM) and physical pages from the OS**.

If your server only has 1GB or 2GB of RAM:
1. The `ls_core` process suddenly demands a massive chunk of physical memory (peaks may exceed 2GB).
2. The Linux kernel determines memory exhaustion and triggers the **OOM-Killer (Out-Of-Memory Killer)**.
3. The kernel sends a `SIGKILL` to violently terminate `ls_core`.

**How to verify?**
Run the following command to check kernel logs. If you find an `oom-killer` record, it confirms the diagnosis:
```bash
dmesg -T | grep -iE "kill|oom"
```
*(Example log: `Out of memory: Killed process 54591 (ls_core) total-vm:3814840kB`)*

## 3. Standard Solution: Configure Swap Memory

For budget/low-spec servers that cannot upgrade physical RAM, the most immediate solution is to allocate a disk-based **Swapfile**.

Swap acts as a safety net. When `ls_core` heavily requests memory, inactive pages are offloaded to disk, protecting the process from termination.

### Steps (Ubuntu/Debian example, adding 4GB Swap):

```bash
# 1. Create a 4GB placeholder file
sudo fallocate -l 4G /swapfile

# 2. Modify permissions (root read/write only for security)
sudo chmod 600 /swapfile

# 3. Format as swap
sudo mkswap /swapfile

# 4. Enable swap
sudo swapon /swapfile

# 5. Verify swap is active
free -h
```

*(Optional)* To make Swap persistent across reboots, add it to `/etc/fstab`:
```bash
echo '/swapfile none swap sw 0 0' | sudo tee -a /etc/fstab
```

### Restart Container After Deployment

After modifying the host environment, you **must restart the proxy Docker container** to clear error states and reset the `ls_core` process pool:
```bash
docker restart antigravity-ls
```

Once completed, you can easily handle massive memory surges triggered by subsequent message requests.

---

# Linux 服务器部署 OOM 排查与修复指南

*中文 | [English](#linux-server-deployment-oom-out-of-memory-troubleshooting--fix-guide)*

如果在 Linux 服务器上部署 `Antigravity-Tools-LS` 后，发现在前端发送消息（调用 `SendUserCascadeMessage` 等操作）时，突然遇到 `UnexpectedEof`，或日志中提示 `transport error`、连接断开，且没有任何应用层的报错堆栈，极有可能是因为**服务器内存不足导致进程被强杀**。

本文档记录了排查此类问题的思路以及低配服务器上的标准解决方案。

## 1. 故障现象描述

- **服务端日志特征**：
  Language Server (`ls_core`) 初始化均正常：
  `initialized server successfully in 898ms`
  但在后续执行到具体对话处理时，应用进程的日志**突然中断**。
- **客户端代理（Rust）报错特征**：
  由于底层的 Language Server 突然被操作系统销毁，处于等待响应的客户端由于 TCP 强制复位，会捕获到类似如下的异常：
  ```
  SendUserCascadeMessage failed. Status: Status { code: Unknown, message: "transport error", source: Some(tonic::transport::Error(Transport, hyper::Error(Io, Custom { kind: UnexpectedEof, error: "peer closed connection without sending TLS close_notify...
  ```

## 2. 根因分析：为何是内存不足？

Windsurf / Codeium 的原生 Language Server 包含了一套复杂的 CGO 依赖和底层处理模块（例如：Tokenizer、局部计算甚至是小型 ONNX 模型），在发送消息处理语境时，它可能**瞬间向操作系统申请大量虚拟内存（Virtual Memory, VM）及物理页**。

如果您的服务器只分配了 1GB 或 2GB 内存：
1. `ls_core` 进程突然大规模索取物理内存（峰值可能超过 2GB）。
2. Linux 内核判定系统内存耗尽，触发 **OOM-Killer (Out-Of-Memory Killer)** 机制。
3. 内核直接发送 `SIGKILL` 信号将 `ls_core` 暴力斩杀。

**如何确诊？** 
您可以执行以下命令查看系统内核日志，若发现 `oom-killer` 记录即可确认：
```bash
dmesg -T | grep -iE "kill|oom"
```
（排查范例中，日志显示 `Out of memory: Killed process 54591 (ls_core) total-vm:3814840kB`）

## 3. 标准解决方案：配置 Swap（虚拟内存）

对于没有条件大幅提升物理内存的平价/低配服务器，最立竿见影的解决方式是分配基于硬盘的 **Swapfile（交换空间）**。

Swap 能够充当备用内存垫底，在 `ls_core` 高峰索要不常用内存页时将其卸到硬盘，避免进程遭到毒手。

### 操作步骤 (以 Ubuntu/Debian 为例，添加 4GB Swap)：

```bash
# 1. 创建 4GB 容量的占位文件
sudo fallocate -l 4G /swapfile

# 2. 修改文件权限（仅允许 root 读写，保证安全）
sudo chmod 600 /swapfile

# 3. 格式化为交换分区
sudo mkswap /swapfile

# 4. 启用交换分区
sudo swapon /swapfile

# 5. 验证 Swap 是否生效
free -h
```

*(可选)* 若要重启后始终生效，请把 Swap 加入 `/etc/fstab` 表中：
```bash
echo '/swapfile none swap sw 0 0' | sudo tee -a /etc/fstab
```

### 部署后重启容器

修改完宿主机环境后，必须**重启代理系统的 Docker 容器**，清理错误状态及重置 `ls_core` 进程池：
```bash
docker restart antigravity-ls
```

完成上述操作后，即可顺利应对发送消息时可能触发的内存激增浪涌。
