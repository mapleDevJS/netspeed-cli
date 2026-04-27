
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'netspeed-cli' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'netspeed-cli'
        for ($i = 1; $i -lt $commandElements.Count; $i++) {
            $element = $commandElements[$i]
            if ($element -isnot [StringConstantExpressionAst] -or
                $element.StringConstantType -ne [StringConstantType]::BareWord -or
                $element.Value.StartsWith('-') -or
                $element.Value -eq $wordToComplete) {
                break
        }
        $element.Value
    }) -join ';'

    $completions = @(switch ($command) {
        'netspeed-cli' {
            [CompletionResult]::new('--no-download', '--no-download', [CompletionResultType]::ParameterName, 'Do not perform download test')
            [CompletionResult]::new('--no-upload', '--no-upload', [CompletionResultType]::ParameterName, 'Do not perform upload test')
            [CompletionResult]::new('--single', '--single', [CompletionResultType]::ParameterName, 'Only use a single connection instead of multiple')
            [CompletionResult]::new('--bytes', '--bytes', [CompletionResultType]::ParameterName, 'Display values in bytes instead of bits')
            [CompletionResult]::new('--simple', '--simple', [CompletionResultType]::ParameterName, 'Suppress verbose output, only show basic information')
            [CompletionResult]::new('--csv', '--csv', [CompletionResultType]::ParameterName, 'Output in CSV format')
            [CompletionResult]::new('--csv-delimiter', '--csv-delimiter', [CompletionResultType]::ParameterName, 'Single character delimiter for CSV output (default: ",")')
            [CompletionResult]::new('--csv-header', '--csv-header', [CompletionResultType]::ParameterName, 'Print CSV headers')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output in JSON format')
            [CompletionResult]::new('--format', '--format', [CompletionResultType]::ParameterName, 'Output format (supersedes --json, --csv, --simple)')
            [CompletionResult]::new('--server', '--server', [CompletionResultType]::ParameterName, 'Specify a server ID to test against (can be supplied multiple times)')
            [CompletionResult]::new('--exclude', '--exclude', [CompletionResultType]::ParameterName, 'Exclude a server from selection (can be supplied multiple times)')
            [CompletionResult]::new('--source', '--source', [CompletionResultType]::ParameterName, 'Source IP address to bind to (IPv4 or IPv6)')
            [CompletionResult]::new('--timeout', '--timeout', [CompletionResultType]::ParameterName, 'HTTP timeout in seconds (default: 10)')
            [CompletionResult]::new('--generate-completion', '--generate-completion', [CompletionResultType]::ParameterName, 'Generate shell completion script')
            [CompletionResult]::new('--quiet', '--quiet', [CompletionResultType]::ParameterName, 'Suppress all progress output (JSON/CSV still go to stdout)')
            [CompletionResult]::new('--minimal', '--minimal', [CompletionResultType]::ParameterName, 'Minimal ASCII-only output (no Unicode box-drawing characters)')
            [CompletionResult]::new('--profile', '--profile', [CompletionResultType]::ParameterName, 'User profile for customized output (gamer, streamer, remote-worker, power-user, casual)')
            [CompletionResult]::new('--theme', '--theme', [CompletionResultType]::ParameterName, 'Output color theme (dark, light, high-contrast, monochrome)')
            [CompletionResult]::new('--strict-config', '--strict-config', [CompletionResultType]::ParameterName, 'Enable strict config mode - show warnings for invalid config values')
            [CompletionResult]::new('--ca-cert', '--ca-cert', [CompletionResultType]::ParameterName, 'Path to a custom CA certificate file (PEM/DER format)')
            [CompletionResult]::new('--tls-version', '--tls-version', [CompletionResultType]::ParameterName, 'Minimum TLS version to use (1.2 or 1.3)')
            [CompletionResult]::new('--pin-certs', '--pin-certs', [CompletionResultType]::ParameterName, 'Enable certificate pinning for speedtest.net servers')
            [CompletionResult]::new('--list', '--list', [CompletionResultType]::ParameterName, 'Display a list of speedtest.net servers sorted by distance')
            [CompletionResult]::new('--history', '--history', [CompletionResultType]::ParameterName, 'Display test history')
            [CompletionResult]::new('--dry-run', '--dry-run', [CompletionResultType]::ParameterName, 'Validate configuration and exit without running tests')
            [CompletionResult]::new('--no-emoji', '--no-emoji', [CompletionResultType]::ParameterName, 'Disable emoji output (for environments where emojis don''t render well)')
            [CompletionResult]::new('--show-config-path', '--show-config-path', [CompletionResultType]::ParameterName, 'Show the configuration file path and exit')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
