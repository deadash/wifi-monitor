## WiFi Monitor Project

`wifi-monitor` 是一个用于监控指定 MAC 地址设备在线状态的工具，并根据配置文件中的条件通过 webhook 发送通知。

### 配置文件

在运行项目之前，需要将项目中的 `Config.toml.template` 文件重命名为 `Config.toml`，并根据需要修改配置。

配置文件示例如下：

```toml
# 表示被监控的 MAC 地址列表
monitored_macs = [
    # "*", # 表示匹配所有MAC地址
    "aa:bb:cc:dd:ee:ff", # 使用 iw event来查看设备mac地址
]

[webhook_configs.online]
# 在8点到12点之间如果上线则发送通知
command = '''
curl -X POST \
-k \
-d '{"event": "{{ event }}", "mac": "{{ mac_address }}"}' \
-H "Content-Type: application/json" \
https://your.n8n/webhook-test/{uuid}
'''
time_condition = { TimeRange = ["08:00", "12:00"] }

[webhook_configs.offline]
# 在18点之后如果注销则发送通知
command = '''
curl -X POST \
-k \
-d '{"event": "{{ event }}", "mac": "{{ mac_address }}"}' \
-H "Content-Type: application/json" \
https://your.n8n/webhook-test/{uuid}
'''
time_condition = { After = "18:00" }
```

### 编译和运行

#### 环境要求

- WSL2 或 Linux 环境
- Rust 工具链
- `cross` 工具（用于交叉编译）

#### 安装 Rust 工具链

如果尚未安装 Rust 工具链，请按照以下步骤安装：

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

#### 安装 `cross` 工具

`cross` 是一个用于简化 Rust 交叉编译的工具，可以通过以下命令安装：

```bash
cargo install cross
```

#### 编译项目

根据目标平台编译项目，可以在 WSL2 或 Linux 下执行以下命令：

```bash
cd /mnt/d/Project/wifi-monitor/

# 针对 aarch64 平台编译
cross build --target aarch64-unknown-linux-gnu --release

# 针对 armv7 平台编译
cross build --target armv7-unknown-linux-gnueabihf --release
```

> 注意：在编译之前，确保已经安装所需的目标平台。可以使用 `rustup target add <target>` 命令添加目标平台。

#### 运行项目

编译完成后，可以在 OpenWRT 或其他兼容的 Linux 设备上运行生成的二进制文件。首先，将编译好的文件传输到目标设备，然后运行 `wifi-monitor`：

```bash
scp target/aarch64-unknown-linux-gnu/release/wifi-monitor user@device_ip:/path/to/destination
ssh user@device_ip
chmod +x /path/to/destination/wifi-monitor
/path/to/destination/wifi-monitor
```
