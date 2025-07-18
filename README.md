<div align="center">
<p align="center">
  <a href="https://www.edgee.cloud">
    <picture>
      <source media="(prefers-color-scheme: dark)" srcset="https://cdn.edgee.cloud/img/component-dark.svg">
      <img src="https://cdn.edgee.cloud/img/component.svg" height="100" alt="Edgee">
    </picture>
  </a>
</p>
</div>

<h1 align="center">SendGrid component for Edgee</h1>

[![Coverage Status](https://coveralls.io/repos/github/edgee-cloud/sendgrid-component/badge.svg)](https://coveralls.io/github/edgee-cloud/sendgrid-component)
[![GitHub issues](https://img.shields.io/github/issues/edgee-cloud/sendgrid-component.svg)](https://github.com/edgee-cloud/sendgrid-component/issues)
[![Edgee Component Registry](https://img.shields.io/badge/Edgee_Component_Registry-Public-green.svg)](https://www.edgee.cloud/edgee/sendgrid)


This component provides a simple way to send emails via SendGrid on [Edgee](https://www.edgee.cloud),
served directly at the edge. You map the component to a specific endpoint such as `/contact`, and
then you invoke it from your frontend code.


## Quick Start

1. Download the latest component version from our [releases page](../../releases)
2. Place the `sendgrid.wasm` file in your server (e.g., `/var/edgee/components`)
3. Add the following configuration to your `edgee.toml`:

```toml
[[components.edge_functions]]
id = "sendgrid"
file = "/var/edgee/components/sendgrid.wasm"
settings.api_key = "SG.abc.xyz"
settings.from_email = "from@example.com" # your verified sender identity
settings.subject = "Contact request" # optional (only used when no template_id is provided)
settings.template_id = "d-abcxyz" # optional
settings.edgee_path = "/path" # exact match
settings.edgee_path_prefix = "/prefix" # will match /prefix/anything
```
Note that either `edgee_path` or `edgee_path_prefix` must be set, but not both.

### How to use the HTTP endpoint

You can send requests to the endpoint as follows:

```javascript

// using static text
await fetch('/contact', {
  method: 'POST',
  body: JSON.stringify({
    "message": "hello world!", // static content (plain text)
    "email": "test@example.com"
    })
});

// using a dynamic template
await fetch('/contact', {
  method: 'POST',
  body: JSON.stringify({
    "data": {"name": "John"}, // dynamic data for your template
    "email": "test@example.com"
    })
});

```

## Development

### Building from Source
Prerequisites:
- [Rust](https://www.rust-lang.org/tools/install)

Build command:
```bash
edgee component build
```

Test command (with local HTTP emulator):
```bash
edgee component test
```

Test coverage command:
```bash
make test.coverage[.html]
```

### Contributing
Interested in contributing? Read our [contribution guidelines](./CONTRIBUTING.md)

### Security
Report security vulnerabilities to [security@edgee.cloud](mailto:security@edgee.cloud)
