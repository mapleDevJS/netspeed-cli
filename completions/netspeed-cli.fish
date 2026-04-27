complete -c netspeed-cli -l no-download -d 'Do not perform download test' -r -f -a "true\t''
false\t''"
complete -c netspeed-cli -l no-upload -d 'Do not perform upload test' -r -f -a "true\t''
false\t''"
complete -c netspeed-cli -l single -d 'Only use a single connection instead of multiple' -r -f -a "true\t''
false\t''"
complete -c netspeed-cli -l bytes -d 'Display values in bytes instead of bits' -r -f -a "true\t''
false\t''"
complete -c netspeed-cli -l simple -d 'Suppress verbose output, only show basic information' -r -f -a "true\t''
false\t''"
complete -c netspeed-cli -l csv -d 'Output in CSV format' -r -f -a "true\t''
false\t''"
complete -c netspeed-cli -l csv-delimiter -d 'Single character delimiter for CSV output (default: ",")' -r
complete -c netspeed-cli -l csv-header -d 'Print CSV headers' -r -f -a "true\t''
false\t''"
complete -c netspeed-cli -l json -d 'Output in JSON format' -r -f -a "true\t''
false\t''"
complete -c netspeed-cli -l format -d 'Output format (supersedes --json, --csv, --simple)' -r -f -a "json\t'Machine-readable JSON output'
jsonl\t'JSON Lines for logging (one JSON object per line)'
csv\t'CSV format for spreadsheet analysis'
minimal\t'Ultra-minimal: just grade + speeds (e.g., "B+ 150.5↓ 25.3↑ 12ms")'
simple\t'Minimal one-line summary'
compact\t'Key metrics with quality ratings'
detailed\t'Full analysis with per-metric grades (default)'
dashboard\t'Rich terminal dashboard with capability matrix'"
complete -c netspeed-cli -l server -d 'Specify a server ID to test against (can be supplied multiple times)' -r
complete -c netspeed-cli -l exclude -d 'Exclude a server from selection (can be supplied multiple times)' -r
complete -c netspeed-cli -l source -d 'Source IP address to bind to (IPv4 or IPv6)' -r
complete -c netspeed-cli -l timeout -d 'HTTP timeout in seconds (default: 10)' -r
complete -c netspeed-cli -l generate-completion -d 'Generate shell completion script' -r -f -a "bash\t''
zsh\t''
fish\t''
powershell\t''
elvish\t''"
complete -c netspeed-cli -l quiet -d 'Suppress all progress output (JSON/CSV still go to stdout)' -r -f -a "true\t''
false\t''"
complete -c netspeed-cli -l minimal -d 'Minimal ASCII-only output (no Unicode box-drawing characters)' -r -f -a "true\t''
false\t''"
complete -c netspeed-cli -l profile -d 'User profile for customized output (gamer, streamer, remote-worker, power-user, casual)' -r
complete -c netspeed-cli -l theme -d 'Output color theme (dark, light, high-contrast, monochrome)' -r
complete -c netspeed-cli -l strict-config -d 'Enable strict config mode - show warnings for invalid config values' -r -f -a "true\t''
false\t''"
complete -c netspeed-cli -l ca-cert -d 'Path to a custom CA certificate file (PEM/DER format)' -r
complete -c netspeed-cli -l tls-version -d 'Minimum TLS version to use (1.2 or 1.3)' -r
complete -c netspeed-cli -l pin-certs -d 'Enable certificate pinning for speedtest.net servers' -r -f -a "true\t''
false\t''"
complete -c netspeed-cli -l list -d 'Display a list of speedtest.net servers sorted by distance'
complete -c netspeed-cli -l history -d 'Display test history'
complete -c netspeed-cli -l dry-run -d 'Validate configuration and exit without running tests'
complete -c netspeed-cli -l no-emoji -d 'Disable emoji output (for environments where emojis don\'t render well)'
complete -c netspeed-cli -l show-config-path -d 'Show the configuration file path and exit'
complete -c netspeed-cli -s h -l help -d 'Print help (see more with \'--help\')'
complete -c netspeed-cli -s V -l version -d 'Print version'
