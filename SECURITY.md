# Security Policy

Novel OS is an actively developed operating-system kernel and should not be used for production workloads or to protect sensitive data. Several subsystems are intentionally incomplete, and the default release is a development platform rather than a security-certified operating system.

## Reporting a vulnerability

Please do not publish exploitable details before maintainers have had a reasonable opportunity to investigate. Open a private security report through the hosting service used for this repository, or contact the project maintainers through the security contact configured for that repository. Include the affected commit, platform, reproduction steps, expected behavior, observed behavior, and any required QEMU or hardware configuration.

Do not include passwords, private keys, access tokens, personal data, or unrelated confidential material in a report.

## Scope

Security-sensitive areas include boot and descriptor setup, page-table construction, user-pointer validation, system calls, capability handles, IPC ownership, process teardown, interrupt handling, DMA, persistent storage, and the experimental SMP path. A successful compile or a serial marker is not sufficient evidence that a security boundary is complete. Changes in these areas require focused tests and an update to the relevant design or verification document.
