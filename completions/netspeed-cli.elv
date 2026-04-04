
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
            cand --server 'Specify a server ID to test against (can be supplied multiple times)'
            cand --exclude 'Exclude a server from selection (can be supplied multiple times)'
            cand --mini 'URL of the Speedtest Mini server'
            cand --source 'Source IP address to bind to'
            cand --timeout 'HTTP timeout in seconds (default: 10)'
            cand --generate-completion 'Generate shell completion script'
            cand --no-download 'Do not perform download test'
            cand --no-upload 'Do not perform upload test'
            cand --single 'Only use a single connection instead of multiple'
            cand --bytes 'Display values in bytes instead of bits'
            cand --share 'Generate and provide a URL to the speedtest.net share results image'
            cand --simple 'Suppress verbose output, only show basic information'
            cand -v 'Enable verbose/debug logging'
            cand --verbose 'Enable verbose/debug logging'
            cand --csv 'Output in CSV format'
            cand --csv-header 'Print CSV headers'
            cand --json 'Output in JSON format'
            cand --list 'Display a list of speedtest.net servers sorted by distance'
            cand --secure 'Use HTTPS instead of HTTP'
            cand --no-pre-allocate 'Do not pre-allocate upload data'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
            cand -V 'Print version'
            cand --version 'Print version'
        }
    ]
    $completions[$command]
}
