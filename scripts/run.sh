#!/bin/bash

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SERVICE_NAME="llmrs"
PORT=3000

GREEN="\033[0;32m"
YELLOW="\033[1;33m"
RED="\033[0;31m"
NC="\033[0m"

show_help() {
    echo -e "LLM.rs 服务管理脚本"
    echo -e "用法: $0 [命令]"
    echo -e ""
    echo -e "命令:"
    echo -e "  start    启动服务"
    echo -e "  stop     停止服务"
    echo -e "  restart  重启服务"
    echo -e "  status   查看服务状态"
    echo -e "  build    编译项目（release 版本）"
    echo -e "  clean    清理编译产物"
    echo -e "  install  安装 systemd 服务"
    echo -e "  help     显示此帮助信息"
    echo -e ""
}

start_service() {
    echo -e "${YELLOW}正在启动 LLM.rs 服务...${NC}"
    
    if sudo lsof -i :$PORT > /dev/null 2>&1; then
        echo -e "${RED}错误: 端口 $PORT 已被占用!${NC}"
        echo -e "${YELLOW}正在停止占用端口的进程...${NC}"
        sudo kill -9 $(sudo lsof -t -i :$PORT) > /dev/null 2>&1
    fi
    
    if [ -f "/etc/systemd/system/$SERVICE_NAME.service" ]; then
        sudo systemctl daemon-reload
        sudo systemctl start $SERVICE_NAME
        sudo systemctl enable $SERVICE_NAME
        echo -e "${GREEN}服务启动成功!${NC}"
        echo -e "${GREEN}服务已设置为开机自启${NC}"
    else
        echo -e "${RED}错误: systemd 服务文件不存在!${NC}"
        echo -e "${YELLOW}请先运行: $0 install${NC}"
        exit 1
    fi
}

stop_service() {
    echo -e "${YELLOW}正在停止 LLM.rs 服务...${NC}"
    
    if [ -f "/etc/systemd/system/$SERVICE_NAME.service" ]; then
        sudo systemctl stop $SERVICE_NAME
        echo -e "${GREEN}服务停止成功!${NC}"
    else
        if sudo lsof -i :$PORT > /dev/null 2>&1; then
            sudo kill -9 $(sudo lsof -t -i :$PORT) > /dev/null 2>&1
            echo -e "${GREEN}服务进程已终止!${NC}"
        else
            echo -e "${YELLOW}服务未运行或端口未占用${NC}"
        fi
    fi
}

restart_service() {
    echo -e "${YELLOW}正在重启 LLM.rs 服务...${NC}"
    stop_service
    sleep 2
    start_service
}

check_status() {
    echo -e "${YELLOW}正在查看 LLM.rs 服务状态...${NC}"
    
    if [ -f "/etc/systemd/system/$SERVICE_NAME.service" ]; then
        sudo systemctl status $SERVICE_NAME
    else
        if sudo lsof -i :$PORT > /dev/null 2>&1; then
            echo -e "${GREEN}服务正在运行 (端口 $PORT 被占用)${NC}"
            sudo lsof -i :$PORT
        else
            echo -e "${YELLOW}服务未运行${NC}"
        fi
    fi
}

build_project() {
    echo -e "${YELLOW}正在编译 LLM.rs 项目...${NC}"
    cd "$PROJECT_DIR" && cargo build --release
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}编译成功!${NC}"
    else
        echo -e "${RED}编译失败!${NC}"
        exit 1
    fi
}

clean_project() {
    echo -e "${YELLOW}正在清理 LLM.rs 编译产物...${NC}"
    cd "$PROJECT_DIR" && cargo clean
    echo -e "${GREEN}清理成功!${NC}"
}

install_service() {
    echo -e "${YELLOW}正在安装 systemd 服务...${NC}"
    
    sudo bash -c "cat > /etc/systemd/system/$SERVICE_NAME.service << 'EOF'
[Unit]
Description=LLM.rs Embedding Service
After=network.target

[Service]
User=root
WorkingDirectory=$PROJECT_DIR
ExecStart=$PROJECT_DIR/target/release/llmrs
Restart=always
RestartSec=5
Environment=\"RUST_LOG=info\"

[Install]
WantedBy=multi-user.target
EOF"
    
    if [ $? -eq 0 ]; then
        sudo systemctl daemon-reload
        echo -e "${GREEN}systemd 服务安装成功!${NC}"
        echo -e "${GREEN}服务文件路径: /etc/systemd/system/$SERVICE_NAME.service${NC}"
    else
        echo -e "${RED}systemd 服务安装失败!${NC}"
        exit 1
    fi
}

main() {
    case "$1" in
        start)
            start_service
            ;;
        stop)
            stop_service
            ;;
        restart)
            restart_service
            ;;
        status)
            check_status
            ;;
        build)
            build_project
            ;;
        clean)
            clean_project
            ;;
        install)
            install_service
            ;;
        help)
            show_help
            ;;
        *)
            show_help
            exit 1
            ;;
    esac
}

main "$@"
