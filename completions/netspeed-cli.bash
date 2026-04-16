_netspeed-cli() {
    local i cur prev opts cmd
    COMPREPLY=()
    if [[ "${BASH_VERSINFO[0]}" -ge 4 ]]; then
        cur="$2"
    else
        cur="${COMP_WORDS[COMP_CWORD]}"
    fi
    prev="$3"
    cmd=""
    opts=""

    for i in "${COMP_WORDS[@]:0:COMP_CWORD}"
    do
        case "${cmd},${i}" in
            ",$1")
                cmd="netspeed__cli"
                ;;
            *)
                ;;
        esac
    done

    case "${cmd}" in
        netspeed__cli)
            opts="-h -V --no-download --no-upload --single --bytes --simple --csv --csv-delimiter --csv-header --json --format --list --server --exclude --source --timeout --generate-completion --history --quiet --dry-run --no-emoji --minimal --profile --theme --show-config-path --help --version"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 1 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --csv-delimiter)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --format)
                    COMPREPLY=($(compgen -W "json jsonl csv minimal simple compact detailed dashboard" -- "${cur}"))
                    return 0
                    ;;
                --server)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --exclude)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --source)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --timeout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --generate-completion)
                    COMPREPLY=($(compgen -W "bash zsh fish powershell elvish" -- "${cur}"))
                    return 0
                    ;;
                --profile)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --theme)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
    esac
}

if [[ "${BASH_VERSINFO[0]}" -eq 4 && "${BASH_VERSINFO[1]}" -ge 4 || "${BASH_VERSINFO[0]}" -gt 4 ]]; then
    complete -F _netspeed-cli -o nosort -o bashdefault -o default netspeed-cli
else
    complete -F _netspeed-cli -o bashdefault -o default netspeed-cli
fi
