#!/bin/bash

# ============================================
# TEST VERSION - Monitors Google & YouTube
# For testing the monitoring system
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

# Service configurations - TEST VERSION with ping URLs
declare -A SERVICES=(
    ["google"]="https://www.google.com"
    ["youtube"]="https://www.youtube.com"
)

# Headers for both services
COMMON_HEADERS=(
    -H 'User-Agent: Mozilla/5.0 (X11; Linux x86_64; rv:152.0) Gecko/20100101 Firefox/152.0'
    -H 'Accept: text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8'
    -H 'Accept-Language: en-US,en;q=0.9'
    -H 'Accept-Encoding: gzip, deflate, br'
    -H 'Connection: keep-alive'
    --compressed
    --max-time 5
)

# State tracking
declare -A SERVICE_STATE
declare -A BACKOFF
declare -A REQUEST_COUNT
declare -A SUCCESS_COUNT
declare -A LAST_HTTP_CODE
declare -A RESPONSE_TIME

# Initialize state
for service in "${!SERVICES[@]}"; do
    SERVICE_STATE["$service"]="waiting"
    BACKOFF["$service"]=$INITIAL_BACKOFF
    REQUEST_COUNT["$service"]=0
    SUCCESS_COUNT["$service"]=0
    LAST_HTTP_CODE["$service"]="---"
    RESPONSE_TIME["$service"]="---"
done

# Temp directory for parallel processing
TEMP_DIR=$(mktemp -d)
trap 'rm -rf "$TEMP_DIR"; echo -e "\n${RED}${BOLD}Stopped${NC}"; exit 0' INT TERM

# Function to print header
print_header() {
    clear
    echo -e "${BOLD}${BLUE}╔══════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BOLD}${BLUE}║        🧪 TEST MONITOR - Google & YouTube                      ║${NC}"
    echo -e "${BOLD}${BLUE}║        Testing notification system                            ║${NC}"
    echo -e "${BOLD}${BLUE}╚══════════════════════════════════════════════════════════════════╝${NC}"
    echo -e "${CYAN}Started: $(date '+%Y-%m-%d %H:%M:%S')${NC}"
    echo -e "${CYAN}Press Ctrl+C to stop${NC}"
    echo -e "${YELLOW}${BOLD}⚠️  This is a TEST version - pinging Google & YouTube${NC}"
    echo -e "${BOLD}${BLUE}────────────────────────────────────────────────────────────────────${NC}"
}

# Function to check a service (ping test)
check_service() {
    local service=$1
    local url=${SERVICES["$service"]}
    local response_file="$TEMP_DIR/${service}_response"
    local status_file="$TEMP_DIR/${service}_status"
    local time_file="$TEMP_DIR/${service}_time"

    # Time the request
    local start_time=$(date +%s%N)

    # Make request and capture both stdout and stderr
    local http_code
    local body
    local error_msg

    # Use a temp file for stderr
    local stderr_file="$TEMP_DIR/${service}_stderr"

    # Make the request
    response=$(curl -s -w "%{http_code}" -o "$response_file" "${COMMON_HEADERS[@]}" "$url" 2>"$stderr_file")
    http_code=$?

    local end_time=$(date +%s%N)
    local duration=$(( ($end_time - $start_time) / 1000000 )) # Convert to milliseconds

    # Read error if any
    if [ -f "$stderr_file" ]; then
        error_msg=$(cat "$stderr_file")
        rm -f "$stderr_file"
    fi

    # Check if curl succeeded
    if [ $http_code -eq 0 ]; then
        # Get the actual HTTP status code from the response
        # We need to read it from the file or use a different method
        # Let's use a different approach to get the status code

        # Re-run with -w to get status code properly
        local temp_response=$(mktemp)
        local status_code=$(curl -s -o "$temp_response" -w "%{http_code}" "${COMMON_HEADERS[@]}" "$url" 2>/dev/null)
        local curl_exit=$?

        if [ $curl_exit -eq 0 ] && [ -n "$status_code" ]; then
            # Success - copy the response
            cat "$temp_response" > "$response_file"
            echo "$status_code" > "$status_file"
            echo "$duration" > "$time_file"
        else
            # Failed
            echo "000" > "$status_file"
            echo "" > "$response_file"
            echo "$duration" > "$time_file"
        fi
        rm -f "$temp_response"
    else
        # Curl failed
        echo "000" > "$status_file"
        echo "" > "$response_file"
        echo "$duration" > "$time_file"
    fi
}

# Function to parse and display results
parse_results() {
    local service=$1
    local status_file="$TEMP_DIR/${service}_status"
    local response_file="$TEMP_DIR/${service}_response"
    local time_file="$TEMP_DIR/${service}_time"

    # Default values
    local http_code="000"
    local response_time="---"

    # Read status
    if [ -f "$status_file" ]; then
        http_code=$(cat "$status_file" | tr -d '\n')
        LAST_HTTP_CODE["$service"]=$http_code
    fi

    # Read response time
    if [ -f "$time_file" ]; then
        response_time=$(cat "$time_file" | tr -d '\n')
        RESPONSE_TIME["$service"]="${response_time}ms"
    fi

    # Determine status based on HTTP code
    local status_text=""
    local status_color=$NC

    if [ -z "$http_code" ] || [ "$http_code" = "000" ]; then
        status_text="❌ FAILED"
        status_color=$RED
        SERVICE_STATE["$service"]="error"
    elif [ "$http_code" -ge 200 ] && [ "$http_code" -lt 400 ]; then
        status_text="✅ ONLINE"
        status_color=$GREEN
        SERVICE_STATE["$service"]="available"
        SUCCESS_COUNT["$service"]=$((SUCCESS_COUNT["$service"] + 1))
    elif [ "$http_code" -ge 400 ] && [ "$http_code" -lt 500 ]; then
        status_text="⚠️ CLIENT ERROR"
        status_color=$RED
        SERVICE_STATE["$service"]="error"
    elif [ "$http_code" -ge 500 ]; then
        status_text="⚠️ SERVER ERROR"
        status_color=$RED
        SERVICE_STATE["$service"]="error"
    else
        status_text="⚠️ HTTP $http_code"
        status_color=$YELLOW
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

    printf "${BOLD}%-10s${NC} ${status_color}%-20s${NC} ${CYAN}%10s${NC} | Req: %4d | OK: %4d [%s]${backoff_indicator}\n" \
        "$service_name" "$status_text" "${RESPONSE_TIME["$service"]}" "$requests" "$successes" "$http_code"
}

# Function to send notification
send_notification() {
    local service=$1
    local response_time=$2

    local service_name=$(echo "$service" | tr '[:lower:]' '[:upper:]')

    # Desktop notification
    notify-send "🧪 Test Alert - $service_name" \
        "✅ Service is ONLINE! (${response_time})" \
        -u critical \
        -i dialog-information 2>/dev/null

    # Terminal notification with clear visual
    echo -e "\n${GREEN}${BOLD}═══════════════════════════════════════════════════════════════════${NC}"
    echo -e "${GREEN}${BOLD}🔔 $service_name is ONLINE! (Response: ${response_time})${NC}"
    echo -e "${GREEN}${BOLD}═══════════════════════════════════════════════════════════════════${NC}"
    echo -e "\a"  # Beep

    # Show a test notification summary
    echo -e "\n${YELLOW}📝 Test Summary:${NC}"
    echo -e "  • Desktop notification should appear"
    echo -e "  • Sound should play (beep)"
    echo -e "  • Terminal shows success message"
    echo -e "  • Backoff should reset to ${INITIAL_BACKOFF}s"
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

        # Wait for all checks to complete
        for pid in "${pids[@]}"; do
            wait $pid 2>/dev/null
        done

        # Parse and display results
        local any_available=false

        for service in "${!SERVICES[@]}"; do
            parse_results "$service"

            # Check if available
            if [ "${SERVICE_STATE["$service"]}" = "available" ]; then
                any_available=true
                # Send notification
                send_notification "$service" "${RESPONSE_TIME["$service"]}"
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

        # Show service status
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

        # Show test instructions on first run
        if [ "$iteration" -eq 1 ]; then
            echo -e "\n${YELLOW}${BOLD}📋 What to check:${NC}"
            echo -e "  1. ✅ Desktop notification popup should appear"
            echo -e "  2. 🔔 Sound/beep should play"
            echo -e "  3. 📊 Status should show ${GREEN}ONLINE${NC} for both services"
            echo -e "  4. 🔄 Backoff should reset to ${INITIAL_BACKOFF}s on success"
            echo -e "  5. 📈 Response time should be visible"
        fi

        # Determine wait time
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

# Check for required tools
echo -e "${BOLD}${BLUE}🧪 Testing Monitor System Setup${NC}"
echo -e "${BLUE}────────────────────────────────────────────────────────────────────${NC}"

# Check for curl
if ! command -v curl &> /dev/null; then
    echo -e "${RED}❌ curl not installed. Please install: sudo apt install curl${NC}"
    exit 1
else
    echo -e "${GREEN}✅ curl found${NC}"
fi

# Check for notify-send
if ! command -v notify-send &> /dev/null; then
    echo -e "${YELLOW}⚠️  notify-send not installed. Install for desktop notifications:${NC}"
    echo "    sudo apt install libnotify-bin"
    echo -e "${YELLOW}Continuing with terminal notifications only...${NC}"
else
    echo -e "${GREEN}✅ notify-send found${NC}"
fi

# Check internet connectivity
echo -e "${CYAN}Testing internet connectivity...${NC}"
if ping -c 1 8.8.8.8 &> /dev/null; then
    echo -e "${GREEN}✅ Internet connection detected${NC}"
else
    echo -e "${YELLOW}⚠️  No internet connection. Tests may fail.${NC}"
fi

echo -e "${BLUE}────────────────────────────────────────────────────────────────────${NC}"
echo -e "\n${GREEN}Starting test in 3 seconds...${NC}"
sleep 3

# Start monitoring
main_loop