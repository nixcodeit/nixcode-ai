# Security Policy

## Supported Versions

We currently provide security updates for the following versions of nixcode-ai:

| Version | Supported          |
| ------- | ------------------ |
| Latest  | :white_check_mark: |

## Reporting a Vulnerability

The nixcode-ai team takes security vulnerabilities seriously. We appreciate your efforts to responsibly disclose your findings.

### How to Report a Vulnerability

1. **Do NOT report security vulnerabilities through public GitHub issues.**

2. **Email the details to**: [kontakt@nixcode.it]

3. **Include the following information**:
   - Description of the vulnerability
   - Steps to reproduce the issue
   - Potential impact
   - Any related code or files
   - Any suggested fixes if available

### What to Expect

- We will acknowledge receipt of your vulnerability report within 48 hours.
- We will provide a more detailed response within 7 days, indicating the next steps in handling your report.
- We will keep you informed about our progress towards a fix and announcement.
- We may ask for additional information or guidance as needed.

## Disclosure Policy

- The vulnerability will be kept confidential until a fix is developed and tested.
- We will coordinate with you to determine an appropriate disclosure date.
- Credit for the discovery will be given to the researcher or team who reported the vulnerability (unless they prefer to remain anonymous).

## Security Considerations for Users

As nixcode-ai interacts with LLM APIs like Anthropic's Claude:

1. **API Key Security**: Store your API keys securely and never commit them to version control.
2. **Data Privacy**: Be aware that content sent to LLMs may be processed on external servers.
3. **Tool Access**: The tool system allows the LLM to interact with your filesystem - use with appropriate caution.

## Security Design

nixcode-ai implements several security measures:

- API keys are handled using the `secrecy` crate to minimize exposure
- Filesystem operations are limited to the current working directory
- Tools have explicit permissions for what they can access

## Security Updates

Security updates will be announced through:
- GitHub releases
- Release notes
- README updates when appropriate

We encourage all users to stay updated with the latest version of nixcode-ai.