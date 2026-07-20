#!/bin/bash

# ============================================
# Train Availability Monitor
# Monitors Alibaba.ir and Raja.ir in parallel
# ============================================

# Colors for terminal output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Configuration
MAX_BACKOFF=60
INITIAL_BACKOFF=2
DISPLAY_TIME=3  # Seconds to show results before countdown

# Service configurations
declare -A SERVICES=(
    ["alibaba"]="https://ws.alibaba.ir/api/v2/train/available/eyJGcm9tIjoxNjEsIlRvIjo3MiwiRGVwYXJ0dXJlRGF0ZSI6IjIwMjYtMDctMjNUMDA6MDA6MDAiLCJUaWNrZXR5cGUiOjEsIklzRXhjbHVzaXZlQ29tcGFydG1lbnQiOmZhbHNlLCJQYXNzZW5nZXJDb3VudCI6NCwiUmV0dXJuRGF0ZSI6IjIwMjYtMDgtMDdUMDA6MDA6MDAiLCJTZXJ2aWNlVHlwZSI6bnVsbCwiQ2hhbm5lbCI6MSwiQXZhaWxhYmxlVGFyZ2V0VHlwZSI6bnVsbCwiUmVxdWVzdGVyIjpudWxsLCJVc2VySWQiOjAsIk9ubHlXaXRoSG90ZWwiOmZhbHNlLCJGb3JjZVVwZGF0ZSI6bnVsbH0="
    ["raja"]="https://raja.ir/api/v2/train/available/eyJGcm9tIjoxNjEsIlRvIjo3MiwiRGVwYXJ0dXJlRGF0ZSI6IjIwMjYtMDctMjNUMDA6MDA6MDAiLCJUaWNrZXRUeXBlIjoxLCJJc0V4Y2x1c2l2ZUNvbXBhcnRtZW50IjpmYWxzZSwiUGFzc2VuZ2VyQ291bnQiOjQsIlJldHVybkRhdGUiOiIyMDI2LTA4LTA3VDAwOjAwOjAwIiwiU2VydmljZVR5cGUiOm51bGwsIkNoYW5uZWwiOjEsIkF2YWlsYWJsZVRhcmdldFR5cGUiOm51bGwsIlJlcXVlc3RlciI6bnVsbCwiVXNlcklkIjowLCJPbmx5V2l0aEhvdGVsIjpmYWxzZSwiRm9yY2VVcGRhdGUiOm51bGx9"
)

# Headers for Alibaba
ALIBABA_HEADERS=(
    -H 'User-Agent: Mozilla/5.0 (X11; Linux x86_64; rv:152.0) Gecko/20100101 Firefox/152.0'
    -H 'Accept: application/json, text/plain, */*'
    -H 'Accept-Language: en-US,en;q=0.9'
    -H 'Accept-Encoding: gzip, deflate, br, zstd'
    -H 'Referer: https://www.alibaba.ir/'
    -H 'ab-channel: WEB-NEW,PRODUCTION,CSR,www.alibaba.ir,desktop,Firefox,152.0,N,N,Linux,x86_64,3.266.2'
    -H 'tracing-sessionid: 1784538224780'
    -H 'ab-alohomora: cv4ToauSzwKwxCv4iHPw9G'
    -H 'tracing-device: N,Firefox,152.0,N,N,Linux'
    -H 'Origin: https://www.alibaba.ir'
    -H 'Sec-GPC: 1'
    -H 'Sec-Fetch-Dest: empty'
    -H 'Sec-Fetch-Mode: cors'
    -H 'Sec-Fetch-Site: same-site'
    -H 'Connection: keep-alive'
    --compressed
)

# Headers for Raja (customize as needed)
RAJA_HEADERS=(
    -H 'User-Agent: Mozilla/5.0 (X11; Linux x86_64; rv:152.0) Gecko/20100101 Firefox/152.0'
    -H 'Accept: application/json, text/plain, */*'
    -H 'Accept-Language: en-US,en;q=0.9'
    -H 'Accept-Encoding: gzip, deflate, br, zstd'
    -H 'Referer: https://raja.ir/'
    -H 'Origin: https://raja.ir'
    -H 'Connection: keep-alive'
    --compressed
)

# State tracking
declare -A SERVICE_STATE
declare -A LAST_RESULT
declare -A BACKOFF
declare -A REQUEST_COUNT
declare -A SUCCESS_COUNT
declare -A LAST_HTTP_CODE
declare -A LAST_ERROR_MSG

# Initialize state
for service in "${!SERVICES[@]}"; do
    SERVICE_STATE["$service"]="waiting"
    LAST_RESULT["$service"]=""
    BACKOFF["$service"]=$INITIAL_BACKOFF
    REQUEST_COUNT["$service"]=0
    SUCCESS_COUNT["$service"]=0
    LAST_HTTP_CODE["$service"]="---"
    LAST_ERROR_MSG["$service"]=""
done

# Temp directory for parallel processing
TEMP_DIR=$(mktemp -d)
trap 'rm -rf "$TEMP_DIR"; echo -e "\n${RED}${BOLD}Stopped${NC}"; exit 0' INT TERM

# Function to print header
print_header() {
    clear
    echo -e "${BOLD}${BLUE}╔══════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BOLD}${BLUE}║                  🚆 TRAIN AVAILABILITY MONITOR                   ║${NC}"
    echo -e "${BOLD}${BLUE}║                   Monitoring Alibaba.ir & Raja.ir                ║${NC}"
    echo -e "${BOLD}${BLUE}╚══════════════════════════════════════════════════════════════════╝${NC}"
    echo -e "${CYAN}Started: $(date '+%Y-%m-%d %H:%M:%S')${NC}"
    echo -e "${CYAN}Press Ctrl+C to stop${NC}"
    echo -e "${BOLD}${BLUE}────────────────────────────────────────────────────────────────────${NC}"
}

# Function to check a service
check_service() {
    local service=$1
    local url=${SERVICES["$service"]}
    local response_file="$TEMP_DIR/${service}_response"
    local status_file="$TEMP_DIR/${service}_status"
    local error_file="$TEMP_DIR/${service}_error"
    
    # Select headers based on service
    local headers
    if [ "$service" == "alibaba" ]; then
        headers=("${ALIBABA_HEADERS[@]}")
    else
        headers=("${RAJA_HEADERS[@]}")
    fi
    
    # Make request with timeout
    response=$(curl -s -w "\n%{http_code}" --max-time 10 "${headers[@]}" "$url" 2>$error_file)
    local exit_code=$?
    
    # Extract HTTP code and body
    local http_code=$(echo "$response" | tail -n1)
    local body=$(echo "$response" | sed '$d')
    
    # Read error message if any
    local error_msg=""
    if [ -f "$error_file" ]; then
        error_msg=$(cat "$error_file")
        rm -f "$error_file"
    fi
    
    # Handle curl errors
    if [ $exit_code -ne 0 ] || [ -z "$http_code" ]; then
        echo "ERROR|000|${error_msg:-Connection failed}" > "$status_file"
        echo "" > "$response_file"
        return
    fi
    
    # Save results
    echo "$body" > "$response_file"
    echo "$http_code" > "$status_file"
}

# Function to parse and display results
parse_results() {
    local service=$1
    local status_file="$TEMP_DIR/${service}_status"
    local response_file="$TEMP_DIR/${service}_response"
    
    # Default values
    local status="unknown"
    local http_code="000"
    local train_count=0
    local message=""
    local color=$YELLOW
    local error_detail=""
    
    # Read status
    if [ -f "$status_file" ]; then
        read -r status_line < "$status_file"
        IFS='|' read -r status http_code message <<< "$status_line"
        LAST_HTTP_CODE["$service"]=$http_code
        LAST_ERROR_MSG["$service"]=$message
    fi
    
    # Read and parse response
    if [ -f "$response_file" ]; then
        local body=$(cat "$response_file")
        
        if [ "$status" != "ERROR" ] && [ -n "$body" ] && [ "$body" != "null" ]; then
            # Try to parse train count
            if command -v jq &> /dev/null; then
                train_count=$(echo "$body" | jq -r '.Data | length // 0' 2>/dev/null)
                if [ -z "$train_count" ] || [ "$train_count" = "null" ]; then
                    train_count=0
                fi
            fi
        fi
    fi
    
    # Determine status display
    local status_text=""
    local status_color=$NC
    
    # Update state
    if [ "$status" = "ERROR" ]; then
        status_text="❌ ERROR"
        status_color=$RED
        SERVICE_STATE["$service"]="error"
        error_detail=" ($message)"
    elif [ "$http_code" = "200" ]; then
        if [ "$train_count" -gt 0 ]; then
            status_text="✅ AVAILABLE"
            status_color=$GREEN
            SERVICE_STATE["$service"]="available"
            SUCCESS_COUNT["$service"]=$((SUCCESS_COUNT["$service"] + 1))
        else
            status_text="⏳ No trains"
            status_color=$YELLOW
            SERVICE_STATE["$service"]="waiting"
        fi
    elif [ "$http_code" = "000" ] || [ -z "$http_code" ]; then
        status_text="⏳ No response"
        status_color=$YELLOW
        SERVICE_STATE["$service"]="timeout"
    else
        status_text="⚠️ HTTP $http_code"
        status_color=$RED
        SERVICE_STATE["$service"]="error"
    fi
    
    # Build display line
    local backoff=${BACKOFF["$service"]}
    local requests=${REQUEST_COUNT["$service"]}
    local successes=${SUCCESS_COUNT["$service"]}
    
    local service_name=$(echo "$service" | tr '[:lower:]' '[:upper:]')
    
    # Backoff indicator
    local backoff_indicator=""
    if [ "${SERVICE_STATE["$service"]}" != "available" ]; then
        backoff_indicator=" 🔄 ${backoff}s"
    else
        backoff_indicator=" ✨ RESET"
    fi
    
    # HTTP code display
    local http_display=""
    if [ "$http_code" != "000" ] && [ "$http_code" != "---" ]; then
        http_display=" [${http_code}]"
    fi
    
    printf "${BOLD}%-10s${NC} ${status_color}%-20s${NC} Trains: %3d | Req: %4d | OK: %4d${http_display}${backoff_indicator}\n" \
        "$service_name" "$status_text$error_detail" "$train_count" "$requests" "$successes"
}

# Function to send notification
send_notification() {
    local service=$1
    local train_count=$2
    
    local service_name=$(echo "$service" | tr '[:lower:]' '[:upper:]')
    
    # Desktop notification
    notify-send "🚆 Train Alert - $service_name" \
        "✅ $train_count train(s) available!" \
        -u critical \
        -i dialog-information 2>/dev/null
    
    # Terminal notification with clear visual
    echo -e "\n${GREEN}${BOLD}═══════════════════════════════════════════════════════════════════${NC}"
    echo -e "${GREEN}${BOLD}🔔 $service_name: $train_count train(s) available!${NC}"
    echo -e "${GREEN}${BOLD}═══════════════════════════════════════════════════════════════════${NC}"
    echo -e "\a"  # Beep
}

# Function to display detailed service info
show_service_details() {
    local service=$1
    local response_file="$TEMP_DIR/${service}_response"
    local status_file="$TEMP_DIR/${service}_status"
    
    if [ -f "$response_file" ] && [ -f "$status_file" ]; then
        local http_code=$(cat "$status_file" 2>/dev/null)
        local body=$(cat "$response_file" 2>/dev/null)
        
        if [ -n "$body" ] && [ "$body" != "null" ] && [ "$http_code" = "200" ]; then
            if command -v jq &> /dev/null; then
                # Check if there are any trains
                local train_count=$(echo "$body" | jq -r '.Data | length // 0' 2>/dev/null)
                if [ -n "$train_count" ] && [ "$train_count" -gt 0 ]; then
                    echo -e "\n${GREEN}${BOLD}📋 $service Details:${NC}"
                    echo "$body" | jq '.Data[] | {TrainName: .TrainName, DepartureTime: .DepartureTime, ArrivalTime: .ArrivalTime, Price: .Price}' 2>/dev/null | head -20
                fi
            fi
        fi
    fi
}

# Main monitoring loop
main_loop() {
    local iteration=0
    
    while true; do
        iteration=$((iteration + 1))
        print_header
        
        echo -e "\n${BOLD}${CYAN}▶ Check #$iteration - $(date '+%H:%M:%S')${NC}"
        echo -e "${BLUE}────────────────────────────────────────────────────────────────────${NC}"
        
        # Launch parallel checks
        local pids=()
        for service in "${!SERVICES[@]}"; do
            # Increment request count
            REQUEST_COUNT["$service"]=$((REQUEST_COUNT["$service"] + 1))
            
            # Launch check in background
            check_service "$service" &
            pids+=($!)
        done
        
        # Wait for all checks to complete with timeout
        for pid in "${pids[@]}"; do
            wait $pid 2>/dev/null
        done
        
        # Parse and display results
        local any_available=false
        local available_services=()
        
        for service in "${!SERVICES[@]}"; do
            parse_results "$service"
            
            # Check if available
            if [ "${SERVICE_STATE["$service"]}" = "available" ]; then
                any_available=true
                available_services+=("$service")
                
                # Parse train count for notification
                local response_file="$TEMP_DIR/${service}_response"
                if [ -f "$response_file" ]; then
                    local body=$(cat "$response_file")
                    local train_count=$(echo "$body" | jq -r '.Data | length // 0' 2>/dev/null)
                    if [ -n "$train_count" ] && [ "$train_count" -gt 0 ]; then
                        send_notification "$service" "$train_count"
                        # Show details for available service
                        show_service_details "$service"
                    fi
                fi
            fi
        done
        
        # Update backoff for services
        for service in "${!SERVICES[@]}"; do
            if [ "${SERVICE_STATE["$service"]}" = "available" ]; then
                # Reset backoff on success
                BACKOFF["$service"]=$INITIAL_BACKOFF
            elif [ "${SERVICE_STATE["$service"]}" != "error" ]; then
                # Increase backoff exponentially
                BACKOFF["$service"]=$((BACKOFF["$service"] * 2))
                if [ ${BACKOFF["$service"]} -gt $MAX_BACKOFF ]; then
                    BACKOFF["$service"]=$MAX_BACKOFF
                fi
            fi
        done
        
        # Show which services are being monitored
        echo -e "${BLUE}────────────────────────────────────────────────────────────────────${NC}"
        echo -ne "${CYAN}🔄 Monitoring:${NC} "
        for service in "${!SERVICES[@]}"; do
            local state=${SERVICE_STATE["$service"]}
            local color=$YELLOW
            if [ "$state" = "available" ]; then
                color=$GREEN
            elif [ "$state" = "error" ]; then
                color=$RED
            fi
            echo -ne "${color}${service}${NC} "
        done
        echo ""
        
        # Show backoff status
        echo -ne "${CYAN}⏱️  Backoff:${NC} "
        for service in "${!SERVICES[@]}"; do
            local backoff=${BACKOFF["$service"]}
            local state=${SERVICE_STATE["$service"]}
            local color=$YELLOW
            if [ "$state" = "available" ]; then
                color=$GREEN
            elif [ "$state" = "error" ]; then
                color=$RED
            fi
            echo -ne "${color}${service}=${backoff}s${NC} "
        done
        echo ""
        
        # Determine wait time (use max backoff as base)
        local max_wait=0
        for service in "${!SERVICES[@]}"; do
            if [ "${SERVICE_STATE["$service"]}" != "available" ]; then
                local b=${BACKOFF["$service"]}
                if [ $b -gt $max_wait ]; then
                    max_wait=$b
                fi
            fi
        done
        
        if [ $max_wait -eq 0 ]; then
            max_wait=$INITIAL_BACKOFF
        fi
        
        # Display results for a moment before countdown
        echo -e "\n${GREEN}✅ Results displayed - showing for ${DISPLAY_TIME}s${NC}"
        
        # Countdown with clear display
        local countdown_start=$(date +%s)
        local countdown_end=$((countdown_start + DISPLAY_TIME))
        
        while [ $(date +%s) -lt $countdown_end ]; do
            local remaining=$((countdown_end - $(date +%s)))
            echo -ne "\r${CYAN}⏳ Next check in: ${BOLD}${remaining}${NC}${CYAN}s (showing results)${NC}   "
            sleep 1
        done
        
        # Then show the actual wait with backoff
        echo -ne "\r${CYAN}⏳ Waiting ${max_wait}s before next check...${NC}   "
        sleep $max_wait
        echo -e "\n"
    done
}

# Check for jq
if ! command -v jq &> /dev/null; then
    echo -e "${YELLOW}⚠️  jq not installed. Install for better JSON parsing:${NC}"
    echo "    sudo apt install jq"
    echo ""
    echo -e "${YELLOW}Continuing with limited parsing...${NC}"
    sleep 2
fi

# Check for notify-send
if ! command -v notify-send &> /dev/null; then
    echo -e "${YELLOW}⚠️  notify-send not installed. Install for desktop notifications:${NC}"
    echo "    sudo apt install libnotify-bin"
    echo ""
    echo -e "${YELLOW}Continuing with terminal notifications only...${NC}"
    sleep 2
fi

# Start monitoring
main_loop
