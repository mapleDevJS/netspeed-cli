name: Bug Report
description: Report a bug or unexpected behavior
title: "[Bug]: "
labels: ["bug"]
body:
  - type: markdown
    attributes:
      value: |
        Thanks for taking the time to fill out this bug report!
  - type: input
    id: version
    attributes:
      label: Version
      description: What version of netspeed-cli are you running?
      placeholder: e.g., 0.3.0
    validations:
      required: true
  - type: input
    id: os
    attributes:
      label: Operating System
      description: What OS are you using?
      placeholder: e.g., macOS 14.2, Ubuntu 22.04, Windows 11
    validations:
      required: true
  - type: textarea
    id: what-happened
    attributes:
      label: What happened?
      description: Also tell us, what did you expect to happen?
      placeholder: Describe the bug
    validations:
      required: true
  - type: textarea
    id: repro-steps
    attributes:
      label: Steps to Reproduce
      description: How can we reproduce this issue?
      placeholder: |
        1. Run command '...'
        2. See error
    validations:
      required: true
  - type: textarea
    id: logs
    attributes:
      label: Relevant Output
      description: Please copy and paste any relevant log output
      render: shell
