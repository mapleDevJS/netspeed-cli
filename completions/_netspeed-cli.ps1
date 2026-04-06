
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
            [CompletionResult]::new('--csv-delimiter', '--csv-delimiter', [CompletionResultType]::ParameterName, 'Single character delimiter for CSV output (default: ",")')
            [CompletionResult]::new('--format', '--format', [CompletionResultType]::ParameterName, 'Output format (supersedes --json, --csv, --simple)')
            [CompletionResult]::new('--server', '--server', [CompletionResultType]::ParameterName, 'Specify a server ID to test against (can be supplied multiple times)')
            [CompletionResult]::new('--exclude', '--exclude', [CompletionResultType]::ParameterName, 'Exclude a server from selection (can be supplied multiple times)')
            [CompletionResult]::new('--source', '--source', [CompletionResultType]::ParameterName, 'Source IP address to bind to')
            [CompletionResult]::new('--timeout', '--timeout', [CompletionResultType]::ParameterName, 'HTTP timeout in seconds (default: 10)')
            [CompletionResult]::new('--generate-completion', '--generate-completion', [CompletionResultType]::ParameterName, 'Generate shell completion script')
            [CompletionResult]::new('--no-download', '--no-download', [CompletionResultType]::ParameterName, 'Do not perform download test')
            [CompletionResult]::new('--no-upload', '--no-upload', [CompletionResultType]::ParameterName, 'Do not perform upload test')
            [CompletionResult]::new('--single', '--single', [CompletionResultType]::ParameterName, 'Only use a single connection instead of multiple')
            [CompletionResult]::new('--bytes', '--bytes', [CompletionResultType]::ParameterName, 'Display values in bytes instead of bits')
            [CompletionResult]::new('--simple', '--simple', [CompletionResultType]::ParameterName, 'Suppress verbose output, only show basic information')
            [CompletionResult]::new('--csv', '--csv', [CompletionResultType]::ParameterName, 'Output in CSV format')
            [CompletionResult]::new('--csv-header', '--csv-header', [CompletionResultType]::ParameterName, 'Print CSV headers')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output in JSON format')
            [CompletionResult]::new('--list', '--list', [CompletionResultType]::ParameterName, 'Display a list of speedtest.net servers sorted by distance')
            [CompletionResult]::new('--history', '--history', [CompletionResultType]::ParameterName, 'Display test history')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
