
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
            cand --csv-delimiter 'Single character delimiter for CSV output (default: ",")'
            cand --format 'Output format (supersedes --json, --csv, --simple)'
            cand --server 'Specify a server ID to test against (can be supplied multiple times)'
            cand --exclude 'Exclude a server from selection (can be supplied multiple times)'
            cand --source 'Source IP address to bind to'
            cand --timeout 'HTTP timeout in seconds (default: 10)'
            cand --generate-completion 'Generate shell completion script'
            cand --no-download 'Do not perform download test'
            cand --no-upload 'Do not perform upload test'
            cand --single 'Only use a single connection instead of multiple'
            cand --bytes 'Display values in bytes instead of bits'
            cand --simple 'Suppress verbose output, only show basic information'
            cand --csv 'Output in CSV format'
            cand --csv-header 'Print CSV headers'
            cand --json 'Output in JSON format'
            cand --list 'Display a list of speedtest.net servers sorted by distance'
            cand --history 'Display test history'
            cand --quiet 'Suppress all progress output (JSON/CSV still go to stdout)'
            cand --no-color 'Disable all colors (equivalent to NO_COLOR=1)'
            cand --no-emoji 'Replace emoji/icons with plain text (for terminals without emoji support)'
            cand --dry-run 'Validate configuration and exit without running tests'
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
        }
    ]
    $completions[$command]
}
