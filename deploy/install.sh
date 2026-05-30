#!/bin/bash
set -e

APP_NAME="adguardvpn-web"
INSTALL_DIR="/opt/$APP_NAME"
SERVICE_FILE="/etc/systemd/system/$APP_NAME.service"

BINARY="$APP_NAME"
CONFIG="config.toml"
SERVICE="deploy/$APP_NAME.service"

# 检查文件
for f in "$BINARY" "$CONFIG" "$SERVICE"; do
    if [ ! -f "$f" ]; then
        echo "错误: 找不到 $f"
        echo "请将此脚本放在以下文件同级目录执行:"
        echo "  ./$BINARY"
        echo "  ./$CONFIG"
        echo "  ./$SERVICE"
        exit 1
    fi
done

echo "=== 安装 $APP_NAME ==="

# 创建目录
echo "[1/4] 创建安装目录 $INSTALL_DIR"
mkdir -p "$INSTALL_DIR"

# 复制文件
echo "[2/4] 复制文件"
cp "$BINARY" "$INSTALL_DIR/"
chmod +x "$INSTALL_DIR/$BINARY"

if [ ! -f "$INSTALL_DIR/$CONFIG" ]; then
    cp "$CONFIG" "$INSTALL_DIR/"
    echo "  已复制 config.toml (首次安装)"
else
    echo "  跳过 config.toml (已存在，保留原有配置)"
fi

# 安装 systemd service
echo "[3/4] 安装 systemd 服务"
cp "$SERVICE" "$SERVICE_FILE"
systemctl daemon-reload
systemctl enable "$APP_NAME"

# 启动服务
echo "[4/4] 启动服务"
systemctl restart "$APP_NAME"
sleep 1

# 显示状态
echo ""
echo "=== 安装完成 ==="
systemctl status "$APP_NAME" --no-pager -l

echo ""
echo "管理命令:"
echo "  systemctl status $APP_NAME    # 查看状态"
echo "  systemctl restart $APP_NAME   # 重启"
echo "  systemctl stop $APP_NAME      # 停止"
echo "  journalctl -u $APP_NAME -f    # 查看日志"
echo ""
echo "配置文件: $INSTALL_DIR/$CONFIG"
