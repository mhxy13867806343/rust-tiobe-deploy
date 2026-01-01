# TIOBE 排行榜服务部署指南

## 在宝塔服务器上部署

### 1. 安装 Rust（如果没有）
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### 2. 上传并解压
将 `rust-tiobe-deploy.tar.gz` 上传到服务器，解压：
```bash
tar -xzf rust-tiobe-deploy.tar.gz
cd rust-tiobe-deploy
```

### 3. 编译
```bash
cargo build --release
```

### 4. 运行
```bash
./target/release/rust-tiobe
```

服务将在 http://127.0.0.1:3000 启动

### 5. 使用 systemd 管理（推荐）
创建服务文件 `/etc/systemd/system/rust-tiobe.service`：
```ini
[Unit]
Description=TIOBE Index Service
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/www/wwwroot/rust-tiobe-deploy
ExecStart=/www/wwwroot/rust-tiobe-deploy/target/release/rust-tiobe
Restart=always
RestartSec=3

[Install]
WantedBy=multi-user.target
```

启动服务：
```bash
systemctl daemon-reload
systemctl enable rust-tiobe
systemctl start rust-tiobe
```

### 6. Nginx 反向代理（宝塔面板配置）
在网站配置中添加：
```nginx
location / {
    proxy_pass http://127.0.0.1:3000;
    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;
}
```
