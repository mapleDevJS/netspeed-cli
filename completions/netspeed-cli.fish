complete -c netspeed-cli -l csv-delimiter -d 'Single character delimiter for CSV output (default: ",")' -r
complete -c netspeed-cli -l format -d 'Output format (supersedes --json, --csv, --simple)' -r -f -a "json\t''
csv\t''
simple\t''
detailed\t''"
complete -c netspeed-cli -l server -d 'Specify a server ID to test against (can be supplied multiple times)' -r
complete -c netspeed-cli -l exclude -d 'Exclude a server from selection (can be supplied multiple times)' -r
complete -c netspeed-cli -l source -d 'Source IP address to bind to' -r
complete -c netspeed-cli -l timeout -d 'HTTP timeout in seconds (default: 10)' -r
complete -c netspeed-cli -l generate-completion -d 'Generate shell completion script' -r -f -a "bash\t''
zsh\t''
fish\t''
power-shell\t''
elvish\t''"
complete -c netspeed-cli -l no-download -d 'Do not perform download test'
complete -c netspeed-cli -l no-upload -d 'Do not perform upload test'
complete -c netspeed-cli -l single -d 'Only use a single connection instead of multiple'
complete -c netspeed-cli -l bytes -d 'Display values in bytes instead of bits'
complete -c netspeed-cli -l simple -d 'Suppress verbose output, only show basic information'
complete -c netspeed-cli -l csv -d 'Output in CSV format'
complete -c netspeed-cli -l csv-header -d 'Print CSV headers'
complete -c netspeed-cli -l json -d 'Output in JSON format'
complete -c netspeed-cli -l list -d 'Display a list of speedtest.net servers sorted by distance'
complete -c netspeed-cli -l history -d 'Display test history'
complete -c netspeed-cli -s h -l help -d 'Print help'
complete -c netspeed-cli -s V -l version -d 'Print version'
