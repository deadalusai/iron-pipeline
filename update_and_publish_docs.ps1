
$ErrorActionPreference = 'Stop'

function exec {
    param(
        [scriptblock] $Block = $(throw 'Block is required'),
        [string] $ErrorMessage = 'Bad command result'
    )

    $global:LastExitCode = 0
    & $Block
    if ($LastExitCode -ne 0) {
        throw ('Exec: ' + $LastExitCode)
    }
}

function check-exitcode {
    # Robocopy is a special snowflake and will return a non-zero exit code even when successful
    # Check the robocopy exit code for failures (see http://ss64.com/nt/robocopy-exit.html)
    if ($global:LastExitCode -eq 0) {
        write-verbose 'The source and destination trees are synchronized'
    }
    if ($global:LastExitCode -band 1) {
        write-verbose 'One or more files were copied successfully'
    }
    if ($global:LastExitCode -band 2) {
        write-warning 'Some Extra files or directories were detected. Examine the output log for details.'
    }
    if ($global:LastExitCode -band 4) {
        write-warning 'Some Mismatched files or directories were detected. Examine the output log. Housekeeping might be required.'
    }
    if ($global:LastExitCode -band 8) {
        write-error 'Some files or directories could not be copied (copy errors occurred and the retry limit was exceeded).'
        return;
    }
    if ($global:LastExitCode -band 16) {
        write-error 'Serious error. Robocopy did not copy any files.'
        return;
    }
    # all ok
    $global:LastExitCode = 0
}

function prompt-user {
    param ([string] $title, [string] $message)

    $yes = New-Object System.Management.Automation.Host.ChoiceDescription "&Yes", "Continues the operation"
    $no = New-Object System.Management.Automation.Host.ChoiceDescription "&No", "Cancels the operation"

    $options = [System.Management.Automation.Host.ChoiceDescription[]]($yes, $no)
    $result = $host.UI.PromptForChoice($title, $message, $options, 0) 
    
    return $result -eq 0
}

# Update the cargo docs which will be checked out (as gh-pages branch) in a different directory

$gh_pages_path = '..\iron-pipeline-gh-pages'
$gh_pages_index_html = '<meta http-equiv=refresh content="0;url=gol/index.html">'

push-location (split-path -parent $PSCommandPath)
try {

    if (-not (test-path $gh_pages_path)) {
        write-error "Could not find gh-pages directory: $gh_pages_path"
    }

    write-verbose 'Invoking cargo doc'
    exec { cargo doc --no-deps }

    write-verbose 'Recreating index.html'
    $gh_pages_index_html | out-file -enc utf8 '.\target\doc\index.html'

    write-verbose 'Mirroring to gh-pages directory'
    exec { 
        robocopy /mir '.\target\doc' $gh_pages_path /xd .git /njh /ndl
        (check-exitcode)
    }

    push-location $gh_pages_path
    try {
        $continue = prompt-user -title 'Ready to publish' -message 'Commit changes to GitHub?'
        if ($continue) {
            exec { git add -A }
            exec { git commit -m "Updating cargo documentation" }
            exec { git push origin gh-pages }
        }
    }
    finally {
        pop-location
    }
}
finally {
    pop-location
}

write-verbose 'Done'