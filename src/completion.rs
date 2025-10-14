pub static POSE_COMPLETE: &str = r#"
_pose_complete() {
    local cur prev cmds opts opts_list opts_config opts_list_services opts_slug opts_get opts_list_commands
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    cmds="list config slug get help"
    opts="-f --file --verbose -q --quiet --no-docker --no-consistency --no-interpolate -h --help -V --version"
    opts_list="-p --pretty -h --help"
    opts_config="-o --output -t --tag --tag-filter --ignore-unauthorized --no-slug --offline --progress --threads -h --help"
    opts_list_services="-f --filter -h --help"
    opts_slug="-h --help"
    opts_get="-o --output --timeout-connect -m --max-time -H --header -h --help"
    opts_list_commands="services images depends dependents volumes networks configs secrets profiles envs help"
    pretty_values="full oneline"

    _filedir() {
        local IFS=$'\n'
        COMPREPLY=( $(compgen -f -- "$1") )
    }

    if [[ ${cur} == -* ]] ; then
        case "${prev}" in
            pose)
                COMPREPLY=( $(compgen -W "${opts}" -- ${cur}) )
                return 0
                ;;
            list)
                COMPREPLY=( $(compgen -W "${opts_list}" -- ${cur}) )
                return 0
                ;;
            config)
                COMPREPLY=( $(compgen -W "${opts_config}" -- ${cur}) )
                return 0
                ;;
            slug)
                COMPREPLY=( $(compgen -W "${opts_slug}" -- ${cur}) )
                return 0
                ;;
            get)
                COMPREPLY=( $(compgen -W "${opts_get}" -- ${cur}) )
                return 0
                ;;
            services)
                COMPREPLY=( $(compgen -W "${opts_list_services}" -- ${cur}) )
                return 0
                ;;
        esac
    elif [[ ${cur} == * ]] ; then
        case "${prev}" in
            pose)
                COMPREPLY=( $(compgen -W "${cmds}" -- ${cur}) )
                return 0
                ;;
            list)
                COMPREPLY=( $(compgen -W "${opts_list_commands}" -- ${cur}) )
                return 0
                ;;
            -f|--file)
                _filedir "${cur}"
                return 0
                ;;
            -p|--pretty)
                COMPREPLY=( $(compgen -W "${pretty_values}" -- ${cur}) )
                return 0
                ;;
        esac
    fi
}

complete -F _pose_complete pose"#;
