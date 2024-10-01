#!/bin/sh

# Global variables
status="Running"
log_folder="$PWD/logs"
pids=""

# Function to print the menu
show_menu() {
    clear
    echo "2 Game Orchestrator"
    echo "------------------------"
    echo "Current status: $status"
    echo "------------------------"
    echo "Enter Q to quit"
    echo "Enter R to restart the games."
    echo "Enter U to update logs."
    echo ""
}

# Function to filter content
filter_content() {
    content="$1"
    echo "$content" | grep -v -E "wgpu_hal::auxil::dxgi::exception" | \
                      grep -v -E "id3d12commandqueue::executecommandlists" | \
                      grep -v -E "d3d12_resource_state_render_target" | \
                      grep -v -E "d3d12_resource_state_[common|present]" | \
                      grep -v -E "invalid_subresource_state"
}

# Function to start all processes
start_all_processes() {
    mkdir -p "$log_folder"
    RUST_LOG=bones_framework::networking=trace cargo run -- --auto-matchmaking --inputs-logging > "$log_folder/game1_raw.log" 2> "$log_folder/game1_error_raw.log" &
    pid1=$!
    RUST_LOG=bones_framework::networking=trace cargo run -- --auto-matchmaking > "$log_folder/game2_raw.log" 2> "$log_folder/game2_error_raw.log" &
    pid2=$!
    pids="$pid1 $pid2"
    echo "Started processes with PIDs: $pids"
    
    # Wait for windows to appear
    sleep 2
    
    # Rename windows
    window_id1=$(xdotool search --pid $pid1 | head -n 1)
    window_id2=$(xdotool search --pid $pid2 | head -n 1)
    
    if [ -n "$window_id1" ]; then
        xdotool set_window --name "Game 1" $window_id1
    fi
    
    if [ -n "$window_id2" ]; then
        xdotool set_window --name "Game 2" $window_id2
    fi
}

# Function to stop all processes
stop_all_processes() {
    echo "Stopping processes with PIDs: $pids"
    for pid in $pids; do
        if kill -0 $pid 2>/dev/null; then
            echo "Killing process $pid"
            kill -TERM $pid
            sleep 1
            if kill -0 $pid 2>/dev/null; then
                echo "Process $pid did not terminate, forcing kill"
                kill -KILL $pid
            fi
        else
            echo "Process $pid is not running"
        fi
    done
    pids=""
}

# Function to filter and update log files
update_filtered_logs() {
    for game in 1 2; do
        if [ -f "$log_folder/game${game}_raw.log" ]; then
            filter_content "$(cat "$log_folder/game${game}_raw.log")" > "$log_folder/game${game}.log"
        fi
        if [ -f "$log_folder/game${game}_error_raw.log" ]; then
            filter_content "$(cat "$log_folder/game${game}_error_raw.log")" > "$log_folder/game${game}_error.log"
        fi
    done
    echo "Logs updated."
}

# Function to handle cleanup
cleanup() {
    status="Closing"
    show_menu
    echo "Cleaning up..."
    stop_all_processes
    update_filtered_logs
    exit
}

# Set up trap for SIGINT and SIGTERM
trap cleanup INT TERM

# Main script
show_menu
start_all_processes

while true; do
    printf "Enter command (Q/R/U): "
    read key
    case $key in
        Q|q)
            cleanup
            ;;
        R|r)
            status="Restarting"
            show_menu
            echo "Restarting processes..."
            stop_all_processes
            sleep 1
            status="Running"
            show_menu
            start_all_processes
            ;;
        U|u)
            echo "Updating logs..."
            update_filtered_logs
            ;;
        *)
            echo "Invalid command. Please enter Q, R, or U."
            ;;
    esac
done