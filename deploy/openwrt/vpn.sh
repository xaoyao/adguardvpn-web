#!/bin/sh

# AdGuard VPN Web Controller - OpenWrt 兼容管理脚本
# 使用 BusyBox 兼容的 POSIX shell 语法

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
BINARY="$SCRIPT_DIR/adguardvpn-web"
CONF="$SCRIPT_DIR/config.toml"
PID_FILE="/var/run/adguardvpn-web.pid"
LOG_FILE="/var/log/adguardvpn-web.log"

# 获取进程 PID（BusyBox 兼容）
get_pid() {
    if [ -f "$PID_FILE" ]; then
        PID=$(cat "$PID_FILE")
        if [ -d "/proc/$PID" ]; then
            echo "$PID"
            return 0
        fi
        rm -f "$PID_FILE"
    fi
    # 使用 ps 查找（BusyBox 兼容）
    PID=$(ps | grep '[a]dguardvpn-web' | awk '{print $1}' | head -1)
    if [ -n "$PID" ]; then
        echo "$PID"
        return 0
    fi
    return 1
}

do_start() {
    PID=$(get_pid)
    if [ -n "$PID" ]; then
        echo "服务已在运行 (PID: $PID)"
        return 1
    fi

    if [ ! -x "$BINARY" ]; then
        echo "错误: 找不到可执行文件或无执行权限"
        echo "运行: chmod +x $BINARY"
        return 1
    fi

    if [ ! -f "$CONF" ]; then
        echo "错误: 找不到配置文件 $CONF"
        return 1
    fi

    # 创建日志目录
    mkdir -p "$(dirname "$LOG_FILE")" 2>/dev/null

    # 后台启动
    "$BINARY" "$CONF" > "$LOG_FILE" 2>&1 &
    PID=$!
    echo "$PID" > "$PID_FILE"
    sleep 1

    if [ -d "/proc/$PID" ]; then
        echo "已启动 (PID: $PID)"
        echo "日志: $LOG_FILE"
    else
        echo "启动失败，查看日志: $LOG_FILE"
        rm -f "$PID_FILE"
        return 1
    fi
}

do_stop() {
    PID=$(get_pid)
    if [ -z "$PID" ]; then
        echo "服务未运行"
        return 0
    fi

    echo "正在停止 (PID: $PID)..."
    kill "$PID" 2>/dev/null

    # 等待进程退出
    i=0
    while [ $i -lt 5 ]; do
        [ ! -d "/proc/$PID" ] && break
        sleep 1
        i=$((i + 1))
    done

    # 强制终止
    if [ -d "/proc/$PID" ]; then
        kill -9 "$PID" 2>/dev/null
    fi

    rm -f "$PID_FILE"
    echo "已停止"
}

do_status() {
    PID=$(get_pid)
    if [ -n "$PID" ]; then
        echo "运行中 (PID: $PID)"
        # 显示运行时间（BusyBox 兼容）
        if [ -d "/proc/$PID" ]; then
            START=$(cat "/proc/$PID/stat" 2>/dev/null | awk '{print $22}')
            if [ -n "$START" ]; then
                # 简单的运行时间显示
                echo "进程状态: 正常"
            fi
        fi
        # 显示最近日志
        if [ -f "$LOG_FILE" ]; then
            echo "最近日志:"
            tail -3 "$LOG_FILE" | sed 's/^/    /'
        fi
    else
        echo "未运行"
    fi
}

do_restart() {
    do_stop
    sleep 1
    do_start
}

do_log() {
    if [ -f "$LOG_FILE" ]; then
        tail -f "$LOG_FILE"
    else
        echo "日志文件不存在: $LOG_FILE"
    fi
}

case "$1" in
    start)   do_start   ;;
    stop)    do_stop    ;;
    restart) do_restart ;;
    status)  do_status  ;;
    log)     do_log     ;;
    *)
        echo "用法: $0 {start|stop|restart|status|log}"
        exit 1
        ;;
esac
