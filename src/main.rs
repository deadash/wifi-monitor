use std::{collections::HashMap, process::Stdio};
use chrono::Local;
use curl_parser::ParsedRequest;
use serde::{Deserialize, Serialize};
use log::{info, warn, error};
use anyhow::{Context, Result};
use time_condition::{is_time_condition_satisfied, TimeCondition};
use tokio::{fs, io::{AsyncBufReadExt, BufReader}, process::Command};

mod time_condition;

#[derive(Debug, Serialize, Deserialize)]
struct WebhookConfig {
    command: String,
    time_condition: Option<TimeCondition>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    webhook_configs: HashMap<String, WebhookConfig>,
    monitored_macs: Vec<String>,
}

async fn load_config(config_path: &str) -> Result<Config> {
    let config_str = fs::read_to_string(config_path).await?;
    let config: Config = toml::from_str(&config_str)?;
    Ok(config)
}


fn setup_logging() -> Result<()> {
    // 设置文件日志
    let file_log = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(fern::log_file("app.log")?);

    // 设置命令行日志
    let stdout_log = fern::Dispatch::new()
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout());

    // 合并日志配置
    fern::Dispatch::new()
        .chain(file_log)
        .chain(stdout_log)
        .apply()?;

    Ok(())
}


#[tokio::main]
async fn main() -> Result<()> {
    setup_logging()?;

    info!("Loading configuration...");
    let config = load_config("Config.toml").await.context("Failed to load configuration")?;

    info!("Starting to monitor iw events...");
    monitor_iw_events(&config).await?;

    Ok(())
}

async fn monitor_iw_events(config: &Config) -> Result<()> {
    let mut process = Command::new("iw")
        .arg("event")
        .stdout(Stdio::piped())
        .spawn()
        .context("Failed to start iw event monitoring")?;

    let stdout = process.stdout.take().ok_or_else(|| anyhow::anyhow!("Failed to capture iw event output"))?;
    let reader = BufReader::new(stdout);

    let mut lines = reader.lines();

    while let Some(line) = lines.next_line().await? {
        info!("iw event: {}", line);

        if let Some(event) = parse_event(&line) {
            if let Err(e) = handle_event(&event, &config).await {
                error!("handle event: {e:?}");
            }
        }
    }

    Ok(())
}

enum IwEvent {
    New(String), // 新连接的MAC地址
    Del(String), // 断开连接的MAC地址
}

fn parse_event(line: &str) -> Option<IwEvent> {
    info!("Parsing iw event line: {}", line); // 先打印日志
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 4 && parts[2] == "station" {
        let mac_address = parts[3].to_string();
        match parts[1] {
            "new" => Some(IwEvent::New(mac_address)),
            "del" => Some(IwEvent::Del(mac_address)),
            _ => {
                warn!("Unrecognized iw event type: {}", parts[1]);
                None
            },
        }
    } else {
        warn!("Failed to parse iw event line: {}", line);
        None
    }
}

async fn handle_event(event: &IwEvent, config: &Config) -> Result<()> {
    let now = Local::now();

    let (event_key, mac_address) = match event {
        IwEvent::New(mac) => {
            info!("New device connected: MAC: {}", mac);
            ("online", mac)
        },
        IwEvent::Del(mac) => {
            info!("Device disconnected: MAC: {}", mac);
            ("offline", mac)
        },
    };

    // 过滤不在监控列表中的MAC地址
    if !config.monitored_macs.contains(mac_address) && !config.monitored_macs.contains(&"*".to_string()) {
        info!("MAC address {} is not in the monitored list. Skipping...", mac_address);
        return Ok(());
    }

    // 根据事件类型（上线或下线）获取相应的 Webhook 配置
    if let Some(webhook_config) = config.webhook_configs.get(event_key) {
        // 检查时间条件是否满足
        if is_time_condition_satisfied(&webhook_config.time_condition, &now) {
            // 执行 Webhook 命令
            execute_command(&webhook_config.command, mac_address, event_key).await?;
        }
    }

    Ok(())
}

async fn execute_command(command: &str, mac_address: &str, event_key: &str) -> Result<()> {
    // 创建 context HashMap
    let mut context = HashMap::new();
    context.insert("mac_address", mac_address);
    context.insert("event", event_key);

    // 加载命令
    let parsed = ParsedRequest::load(command, Some(&context))
        .with_context(|| format!("Failed to parse the command: {}", command))?;

    // 构建 reqwest 请求
    let req = parsed.build_reqwest()?;

    // 执行请求
    let resp = req.send().await
        .context("Failed to execute the reqwest command")?;

    // 确保响应状态是成功的
    if resp.status().is_success() {
        Ok(())
    } else {
        anyhow::bail!("Request failed with status: {}", resp.status());
    }
}