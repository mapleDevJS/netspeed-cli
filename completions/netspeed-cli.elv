
use builtin;
use str;

set edit:completion:arg-completer[netspeed-cli] = {|@words|
    fn spaces {|n|
        builtin:repeat $n ' ' | str:join ''
    }
    fn cand {|text desc|
        edit:complex-candidate $text &display=$text' '(spaces (- 14 (wcswidth $text)))$desc
    }
    var command = 'netspeed-cli'
    for word $words[1..-1] {
        if (str:has-prefix $word '-') {
            break
        }
        set command = $command';'$word
    }
    var completions = [
        &'netspeed-cli'= {
            cand --no-download 'Do not perform download test'
            cand --no-upload 'Do not perform upload test'
            cand --single 'Only use a single connection instead of multiple'
            cand --bytes 'Display values in bytes instead of bits'
            cand --simple 'Suppress verbose output, only show basic information'
            cand --csv 'Output in CSV format'
            cand --csv-delimiter 'Single character delimiter for CSV output (default: ",")'
            cand --csv-header 'Print CSV headers'
            cand --json 'Output in JSON format'
            cand --format 'Output format (supersedes --json, --csv, --simple)'
            cand --server 'Specify a server ID to test against (can be supplied multiple times)'
            cand --exclude 'Exclude a server from selection (can be supplied multiple times)'
            cand --source 'Source IP address to bind to (IPv4 or IPv6)'
            cand --timeout 'HTTP timeout in seconds (default: 10)'
            cand --generate-completion 'Generate shell completion script'
            cand --quiet 'Suppress all progress output (JSON/CSV still go to stdout)'
            cand --minimal 'Minimal ASCII-only output (no Unicode box-drawing characters)'
            cand --profile 'User profile for customized output (gamer, streamer, remote-worker, power-user, casual)'
            cand --theme 'Output color theme (dark, light, high-contrast, monochrome)'
            cand --strict-config 'Enable strict config mode - show warnings for invalid config values'
            cand --ca-cert 'Path to a custom CA certificate file (PEM/DER format)'
            cand --tls-version 'Minimum TLS version to use (1.2 or 1.3)'
            cand --pin-certs 'Restrict TLS connections to speedtest.net and ookla.com domains'
            cand --list 'Display a list of speedtest.net servers sorted by distance'
            cand --history 'Display test history'
            cand --dry-run 'Validate configuration and exit without running tests'
            cand --no-emoji 'Disable emoji output (for environments where emojis don''t render well)'
            cand --show-config-path 'Show the configuration file path and exit'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
            cand -V 'Print version'
            cand --version 'Print version'
        }
    ]
    $completions[$command]
}
