#!/bin/bash

APP_DIR="$(cd "$(dirname "$0")" && pwd)"
APP_NAME="adguardvpn-web"
BINARY="$APP_DIR/$APP_NAME"
CONFIG="$APP_DIR/config.toml"
PID_FILE="$APP_DIR/$APP_NAME.pid"
LOG_FILE="$APP_DIR/$APP_NAME.log"

start() {
    if [ -f "$PID_FILE" ] && kill -0 "$(cat "$PID_FILE")" 2>/dev/null; then
        echo "$APP_NAME 已在运行 (PID: $(cat "$PID_FILE"))"
        return 1
    fi

    if [ ! -f "$BINARY" ]; then
        echo "错误: 找不到 $BINARY"
        exit 1
    fi

    echo "启动 $APP_NAME ..."
    nohup "$BINARY" "$CONFIG" >> "$LOG_FILE" 2>&1 &
    echo $! > "$PID_FILE"
    sleep 1

    if kill -0 "$(cat "$PID_FILE")" 2>/dev/null; then
        echo "$APP_NAME 已启动 (PID: $(cat "$PID_FILE"))"
    else
        echo "启动失败，请查看日志: $LOG_FILE"
        rm -f "$PID_FILE"
        exit 1
    fi
}

stop() {
    if [ ! -f "$PID_FILE" ]; then
        echo "$APP_NAME 未运行"
        return 1
    fi

    local pid=$(cat "$PID_FILE")
    if kill -0 "$pid" 2>/dev/null; then
        echo "停止 $APP_NAME (PID: $pid) ..."
        kill "$pid"
        for i in $(seq 1 10); do
            if ! kill -0 "$pid" 2>/dev/null; then
                break
            fi
            sleep 1
        done

        if kill -0 "$pid" 2>/dev/null; then
            echo "强制终止 ..."
            kill -9 "$pid"
        fi

        echo "$APP_NAME 已停止"
    else
        echo "$APP_NAME 未在运行 (残留 PID 文件)"
    fi
    rm -f "$PID_FILE"
}

restart() {
    stop
    sleep 1
    start
}

status() {
    if [ -f "$PID_FILE" ] && kill -0 "$(cat "$PID_FILE")" 2>/dev/null; then
        local pid=$(cat "$PID_FILE")
        echo "$APP_NAME 运行中 (PID: $pid)"
        if command -v ss &>/dev/null; then
            local port=$(grep -oP 'port\s*=\s*\K\d+' "$CONFIG" 2>/dev/null || echo "3000")
            ss -tlnp 2>/dev/null | grep ":$port " | head -1
        fi
    else
        echo "$APP_NAME 未运行"
        [ -f "$PID_FILE" ] && rm -f "$PID_FILE"
    fi
}

log() {
    if [ -f "$LOG_FILE" ]; then
        tail -f "$LOG_FILE"
    else
        echo "日志文件不存在: $LOG_FILE"
    fi
}

case "$1" in
    start)   start   ;;
    stop)    stop    ;;
    restart) restart ;;
    status)  status  ;;
    log)     log     ;;
    *)
        echo "用法: $0 {start|stop|restart|status|log}"
        exit 1
        ;;
esac
