#!/bin/bash

# Already sourced
if type __setup >/dev/null 2>&1; then
    return
fi

LIB_PATH="${BASH_SOURCE[0]}"
ROOT_DIR=$(cd ${BASH_SOURCE[0]%/*}/../.. && pwd) # The first element is this script
# ${BASH_SOURCE[-1]} not working for old bash
__idx=$((${#BASH_SOURCE[@]} - 1))
SCRIPT_PATH=${BASH_SOURCE[$__idx]}
SCRIPT_NAME=${SCRIPT_NAME:-$(basename ${BASH_SOURCE[$__idx]})} # The last element is the command script
SCRIPT_ARGS="$*"
PATH=$PATH:/usr/sbin:/sbin
DEBUG_MODE=${DEBUG_MODE:-"0"}
_SHELL=$(ps -p $$ -o args= | awk '{print $1}')
export ROOT_DIR SCRIPT_NAME SCRIPT_ARGS PATH DEBUG_MODE _SHELL

# not sourced from bash
if [ $SCRIPT_NAME != "base_functions" ]; then
    set -e
    set -u
    set -o pipefail
fi

__TEMPFILES=('')
__LOOPDEVS=('')
__EXCEPTIONS=('')
__FD_INFO=${__FD_INFO:-2}
__LOG_FILE=${__LOG_FILE:-""}
__CONSOLE_OUTPUT=${__CONSOLE_COLOR:-"yes"}
__CONSOLE_TS=${__CONSOLE_TS:-"no"}
__CONSOLE_COLOR=${__CONSOLE_COLOR:-"yes"}

[ -f ${ROOT_DIR}/etc/environment ] && . ${ROOT_DIR}/etc/environment

_call() {
    echo ". $LIB_PATH; trap - EXIT; $(declare -pf); $*" | $_SHELL
}

__umount_all()
{
    local tmp_mnt=$1
    local ret=0

    if [ -z "$tmp_mnt" ] || [ ! -d "$tmp_mnt" ]; then
        return $ret
    fi

    # Flush unwritten writes
    sync
    tmp_mnt=$(cd $tmp_mnt && pwd)
    if awk '{print $2}' /proc/mounts | grep -qw ^$tmp_mnt; then
        # In case it has sub-mounts
        mnt_list=$(awk '{print $2}' /proc/mounts | grep ^$tmp_mnt | sort -ur)
        for mnt in $mnt_list; do
            umount -d $mnt || ret=$?
        done
    fi

    return $ret
}

__cleanup_tmps_loops()
{
    for f in "${__TEMPFILES[@]}"; do
        [ -n "$f" ] || continue
        if [ -f "$f" ]; then
            rm -f $f
        fi

        if [ -d "$f" ]; then
            __umount_all $f
            rmdir $f
        fi
    done

    for dev in "${__LOOPDEVS[@]}"; do
        [ -n "$dev" ] || continue
        if ls /dev/mapper/"$(basename $dev)"* >/dev/null 2>&1; then
            kpartx -d $dev
        fi
        if losetup -a | grep -q $dev:; then
            losetup -d $dev
        fi
    done
}

# Clean up temp files, mountpoints, loop devices
__cleanup()
{
    set +x
    local ret=$1
    local func=$2
    local lineno=$3
    local comm=$4

    trap - EXIT

    for f in "${__EXCEPTIONS[@]}"; do
        [ -n "$f" ] || continue
        $f
    done

    __cleanup_tmps_loops

    if type cleanup >/dev/null 2>&1; then
        cleanup $ret
    fi

    if [ $ret -ne 0 ]; then
        if [ -z "$func" ]; then
            location="$lineno"
        else
            location="$func():$lineno"
        fi
        if echo $comm | grep -qw -e ^exit -e ^return; then
            if echo $comm | grep -qw ^return; then
                info "exit ($comm) near script $SCRIPT_NAME:$location"
            fi
            exit $ret
        fi
        die "Running command ($comm) near script $SCRIPT_NAME:$location failed" $ret
    fi

    return $ret
}

__interrupt()
{
    info "Interrupted by signal $1"
    shift
    __cleanup "$@"
}

# Set up trap handler and debugger
__setup()
{
    trap 'exit $?' SIGUSR2
    trap '__cleanup $? "${FUNCNAME[0]-main}" "$LINENO" "$BASH_COMMAND"' EXIT
    trap '__interrupt SIGINT 130 "${FUNCNAME[0]-main}" "$LINENO" "$BASH_COMMAND"' SIGINT
    trap '__interrupt SIGTERM 130 "${FUNCNAME[0]-main}" "$LINENO" "$BASH_COMMAND"' SIGTERM

    if [ "$DEBUG_MODE" != "1" ]; then
        eval DEBUG_MODE="\${DEBUG_$(basename $SCRIPT_NAME .sh | sed 's/-/_/g' | sed 's/\./_/g')-0}"
    fi
    if [ "$DEBUG_MODE" = "1" ] || [[ $- =~ x ]]; then
        enable_debug
    fi

    if [ -n "$__LOG_FILE" ]; then
        # Adjust pipe to reflect correct script name
        exec 2> >(__add_ts >>$__LOG_FILE)
    fi
}

# Create temp files/directories
# __cleanup() will clean up these files
setup_tmp_file()
{
    local var=$1
    local file_name=${2:-""}

    [[ -n $file_name ]] || file_name=$(mktemp /tmp/${SCRIPT_NAME}-XXXXXXXX)
    __TEMPFILES=("$file_name" "${__TEMPFILES[@]}")
    eval $var=$file_name
}

setup_tmp_mnt()
{
    local var=$1
    local dir_name=${2:-""}

    [[ -n $dir_name ]] || dir_name=$(mktemp -d /tmp/${SCRIPT_NAME}-XXXXXXXX)
    __TEMPFILES=("$dir_name" "${__TEMPFILES[@]}")
    eval $var=$dir_name
}

setup_loop_dev()
{
    local var=$1
    local image_file=$2
    local loop_dev=""

    if losetup -h 2>&1 | grep -q -- '--show'; then
        loop_dev=$(losetup -f --show $image_file)
    else
        loop_dev=$(losetup -f)
        losetup $loop_dev $image_file
    fi
    # Only kpartx works for loop device
    kpartx -va $loop_dev >&2
    __LOOPDEVS=("$loop_dev" "${__LOOPDEVS[@]}")
    eval $var=$loop_dev
}

set_exception()
{
    local func="$*"

    __EXCEPTIONS=("$func" "${__EXCEPTIONS[@]}")
}

unset_exception()
{
    unset __EXCEPTIONS[0]
}

# Add timestamp and script name before logging
__add_ts()
{
    awk -v SCRIPT="${SCRIPT_NAME}" '{ print strftime("%c: ") SCRIPT ": " $0; system("")}'
}

setup_log_file()
{
    set +x
    local old_log="$__LOG_FILE"
    export __LOG_FILE=$1
    local msg=${2:-""}

    if [ $__FD_INFO -eq 2 ]; then
        # preserve the original stderr to be used to report informaion
        export __FD_INFO=7
        eval "exec $__FD_INFO>&2"
    fi

    if [ "$__LOG_FILE" != "$old_log" ]; then
        if [ -d /dev/fd ]; then
            exec 2> >(__add_ts >>$__LOG_FILE)
            if [ $__CONSOLE_OUTPUT = "no" ]; then
                exec 1> >(__add_ts >>$__LOG_FILE)
            fi
        else
            die "Could not find /dev/fd directory, try to run \"ln -s /proc/self/fd /dev/fd\" to fix it"
        fi
    fi

    if [ -n "$msg" ]; then
        __output_with_log "green" $msg
    fi
    __output_with_log "green" "Running \"$0 $SCRIPT_ARGS\" with log file: $__LOG_FILE"
}

# Options:
# "ts|no_ts": output w/? timestamps
# "color|no_color": output w/? colors
# eg.
# setup_console "ts" "no_color"
#
setup_console()
{
    local opt=""
    for opt in "$@"; do
        case $opt in
            ts)
                export __CONSOLE_TS=yes
                ;;
            no_ts)
                export __CONSOLE_TS=no
                ;;
            color)
                export __CONSOLE_COLOR=yes
                ;;
            no_color)
                export __CONSOLE_COLOR=no
                ;;
            no_output)
                export __CONSOLE_OUTPUT=no
                exec 1>/dev/null
                exec 2>/dev/null
                ;;
        esac
    done
}

__support_color()
{
    type tput >/dev/null 2>&1 || return 1
    [ $__CONSOLE_COLOR = "yes" ] || return 1
    [ -t $1 ] && [ -n "$(tput colors)" ] && [ "$(tput colors)" -ge 8 ]
}

__color()
{
    local color=$1
    shift

    case $color in
        red)
            echo "$(tput setaf 1)$(tput bold)$*$(tput sgr0)"
            ;;
        green)
            echo "$(tput setaf 2)$(tput bold)$*$(tput sgr0)"
            ;;
        yellow)
            echo "$(tput setaf 3)$(tput bold)$*$(tput sgr0)"
            ;;
        *)
            echo "$*"
    esac
}

__output()
{
    local fd=$1
    local color=$2
    shift 2
    local msg="$*"

    if __support_color $fd; then
        __color $color "$msg" >&$fd
    else
        echo "$msg" >&$fd
    fi
}

# Output to both console and the log file
__output_with_log()
{
    set +x
    local color=$1
    shift
    local msg="$*"

    msg_with_ts="$(echo "$msg" | __add_ts)"
    if  [ $__FD_INFO -ne 2 ]; then
        # log file always with timestamps, already set
        __output 2 $color "$msg"
    fi

    if [ $__CONSOLE_OUTPUT = "yes" ]; then
        if [ "$__CONSOLE_TS" = "no" ]; then
            __output $__FD_INFO $color "$msg"
        else
            __output $__FD_INFO $color "$msg_with_ts"
        fi
    fi

    if [ "$DEBUG_MODE" = "1" ]; then
        set -x
    fi
}

__ret_info()
{
    local ret=$1

    if [ "$ret" -ne 0 ]; then
        echo " (ret=$ret)"
    fi
}

output()
{
    echo "$*"
}

info()
{
    set +x
    __output_with_log "default" "$*"
}

warn()
{
    set +x
    __output_with_log "yellow" "WARNING: $*"
}

err()
{
    local last_ret=$?
    local msg="$1"
    local ret=${2-0}

    set +x
    [ "$ret" -eq 0 ] && ret=$last_ret
    __output_with_log "red" "ERROR: $msg$(__ret_info $ret)"
}

die()
{
    local last_ret=$?
    local msg="$1"
    local ret=${2-0}

    set +x
    [ "$ret" -eq 0 ] && ret=$last_ret
    __output_with_log "red" "FATAL: $msg$(__ret_info $ret)"

    sync # To flush logs
    [ "$ret" -ne 0 ] || ret=1
    exit $ret
}

check_root_permission()
{
    if [ "$EUID" -ne 0 ]; then
        die "Please run as root"
    fi
}

enable_debug()
{
    set +x
    export DEBUG_MODE=1
    export PS4='+${BASH_SOURCE:-""}:${LINENO}: ${FUNCNAME[0]:+${FUNCNAME[0]}(): }'
    set -x
}

disable_debug()
{
    set +x
    export DEBUG_MODE=0
}

__setup
